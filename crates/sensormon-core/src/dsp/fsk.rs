//! Non-coherent 2-FSK demodulation of one isolated burst.
//!
//! 1. Find the two tones as spectral peaks.
//! 2. Matched-filter each tone over one symbol (mix to DC, boxcar sum).
//! 3. Decision variable d = (|mark| - |space|) / (|mark| + |space|).
//! 4. Timing: fit a symbol lattice to the zero crossings of d (least squares,
//!    so a 0.1% clock error over a 500-bit frame is absorbed).
//! 5. Slice d at the lattice midpoints.

use super::extract::Baseband;
use super::power_dbfs;
use crate::units::{Db, Hertz, SampleIndex};
use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;

#[derive(Clone, Copy, Debug)]
pub struct FskParams {
    pub symbol_rate: Hertz,
    /// Accept tone pairs this far apart.
    pub min_separation: Hertz,
    pub max_separation: Hertz,
    /// Tone peaks must exceed the median spectrum by this much.
    pub min_peak: Db,
    /// Allowed symbol-clock error when fitting timing.
    pub max_rate_error: f64,
}

impl FskParams {
    pub fn fineoffset() -> Self {
        FskParams { symbol_rate: Hertz(17_240.0), min_separation: Hertz::khz(30.0), max_separation: Hertz::khz(160.0), min_peak: Db(6.0), max_rate_error: 0.02 }
    }
}

/// Demodulated symbols plus the measurements made on the way.
#[derive(Clone, Debug)]
pub struct Symbols {
    /// mark (higher tone) = true.
    pub bits: Vec<bool>,
    /// Absolute source-stream index of the first symbol's midpoint.
    pub t0: SampleIndex,
    pub f_mark: Hertz,
    pub f_space: Hertz,
    pub symbol_rate: Hertz,
    pub rssi: Db,
    pub snr: Db,
    pub noise: Db,
}

/// Find the two strongest tones in the burst. Returns (space, mark) in Hz relative to baseband DC.
fn find_tones(bb: &Baseband, p: &FskParams) -> Option<(f64, f64, Db, Db)> {
    // 8192 points is ~30 Hz bins at 250 kS/s: ample for locating tones, and
    // bounded cost on long bursts (only the first n samples are used).
    let n = bb.iq.len().next_power_of_two().clamp(256, 65536);
    let mut buf: Vec<Complex<f32>> = vec![Complex::new(0.0, 0.0); n];
    let m = bb.iq.len().min(n);
    for i in 0..m {
        let w = 0.5 - 0.5 * (2.0 * PI * i as f32 / m as f32).cos();
        buf[i] = bb.iq[i] * w;
    }
    FftPlanner::<f32>::new().plan_fft_forward(n).process(&mut buf);
    let fs = bb.sample_rate.hz();
    let bin_hz = fs / n as f64;
    let mut pw: Vec<f32> = buf.iter().map(|c| 10.0 * (c.norm_sqr() + 1e-20).log10()).collect();
    // exclude the capture's DC spike (mixed to a known offset), ±2 kHz
    let dc_bin = ((bb.source_dc_offset.0 / bin_hz).round() as i64).rem_euclid(n as i64) as usize;
    let dc_guard = (2e3 / bin_hz).ceil() as usize;
    for d in 0..=dc_guard {
        pw[(dc_bin + d) % n] = -300.0;
        pw[(dc_bin + n - d) % n] = -300.0;
    }
    let mut sorted = pw.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = sorted[n / 2];
    let freq = |k: usize| if k < n / 2 { k as f64 * bin_hz } else { (k as f64 - n as f64) * bin_hz };
    let k1 = (0..n).max_by(|&a, &b| pw[a].partial_cmp(&pw[b]).unwrap())?;
    let f1 = freq(k1);
    if pw[k1] - median < p.min_peak.0 {
        return None;
    }
    let min_sep_bins = (p.min_separation.0 / bin_hz) as i64;
    let k2 = (0..n)
        .filter(|&k| {
            let d = ((k as i64 - k1 as i64).rem_euclid(n as i64)).min((k1 as i64 - k as i64).rem_euclid(n as i64));
            d >= min_sep_bins
        })
        .max_by(|&a, &b| pw[a].partial_cmp(&pw[b]).unwrap())?;
    let f2 = freq(k2);
    if pw[k2] - median < p.min_peak.0 {
        return None;
    }
    let sep = (f1 - f2).abs();
    if sep < p.min_separation.0 || sep > p.max_separation.0 {
        return None;
    }
    // Tone power (dBFS) from this spectrum; noise from the detector's tracked
    // floor (robust, excludes bursts), scaled to one symbol-rate bandwidth —
    // what a matched-filter detector sees — so SNR is comparable across sample
    // rates and with rtl_433's figures rather than inflated by FFT gain.
    let scale = 10.0 * (m as f32).log10();
    let peak = Db(pw[k1].max(pw[k2]) - scale);
    let bins_per_symbol_bw = (p.symbol_rate.0 / bb.burst.noise_bin_hz.0).max(1.0) as f32;
    let noise = Db(bb.burst.noise.0 + 10.0 * bins_per_symbol_bw.log10());
    Some((f1.min(f2), f1.max(f2), peak, noise))
}

/// |boxcar matched filter| for one tone: mix to DC then sliding sum over `len` samples.
fn tone_energy(x: &[Complex<f32>], f_norm: f32, len: usize) -> Vec<f32> {
    let step = Complex::from_polar(1.0, -2.0 * PI * f_norm);
    let mut ph = Complex::new(1.0f32, 0.0);
    let mixed: Vec<Complex<f32>> = x
        .iter()
        .map(|&s| {
            let y = s * ph;
            ph *= step;
            y
        })
        .collect();
    let mut out = vec![0.0f32; x.len()];
    let mut acc = Complex::new(0.0f32, 0.0);
    for i in 0..x.len() {
        acc += mixed[i];
        if i >= len {
            acc -= mixed[i - len];
        }
        out[i] = acc.norm();
    }
    out
}

/// Symbol lattice through zero-crossing times. A periodogram over the crossing
/// instants picks the period (robust to sparse crossings and clock error), its
/// phase gives the lattice offset, then a least-squares pass refines both.
/// Returns (phase, period) in samples.
fn fit_lattice(crossings: &[f32], nominal_period: f32, max_err: f32) -> Option<(f32, f32)> {
    if crossings.len() < 6 {
        return None;
    }
    // Coarse estimate from the first crossings only (bounded cost on long
    // bursts); the least-squares refinement below uses all of them.
    let coarse = &crossings[..];
    let span = coarse[coarse.len() - 1] - coarse[0];
    // step fine enough that accumulated phase error over `coarse` stays < 0.05 symbol
    let rel_step = (0.05 * nominal_period / span.max(nominal_period)).clamp(1e-4, 1e-3);
    let steps = (2.0 * max_err / rel_step).ceil() as i32;
    let mut best = (0.0f32, nominal_period, -1.0f32);
    for i in -steps..=steps {
        let period = nominal_period * (1.0 + i as f32 * rel_step);
        let (mut re, mut im) = (0.0f32, 0.0f32);
        for &t in coarse {
            let a = 2.0 * PI * t / period;
            re += a.cos();
            im += a.sin();
        }
        let mag = (re * re + im * im).sqrt();
        if mag > best.2 {
            best = (im.atan2(re) / (2.0 * PI) * period, period, mag);
        }
    }
    let mut period = best.1;
    let mut phase = best.0;
    for _ in 0..3 {
        // assign integer symbol counts, reject crossings far from the lattice
        let mut sx = 0.0f64;
        let mut sy = 0.0f64;
        let mut sxx = 0.0f64;
        let mut sxy = 0.0f64;
        let mut cnt = 0usize;
        for &t in crossings {
            let k = ((t - phase) / period).round();
            let resid = t - (phase + k * period);
            if resid.abs() > 0.3 * period {
                continue;
            }
            let (x, y) = (k as f64, t as f64);
            sx += x;
            sy += y;
            sxx += x * x;
            sxy += x * y;
            cnt += 1;
        }
        if cnt < 6 {
            return None;
        }
        let nf = cnt as f64;
        let denom = nf * sxx - sx * sx;
        if denom.abs() < 1e-9 {
            return None;
        }
        let slope = (nf * sxy - sx * sy) / denom;
        let intercept = (sy - slope * sx) / nf;
        period = slope as f32;
        phase = intercept as f32;
        if ((period - nominal_period) / nominal_period).abs() > max_err {
            return None;
        }
    }
    Some((phase, period))
}

/// The tone pair of a burst (independent of symbol rate), so it can be found
/// once and reused for every symbol rate the decoders need.
#[derive(Clone, Copy, Debug)]
pub struct Tones {
    pub f_space: f64,
    pub f_mark: f64,
    pub peak: Db,
    pub noise: Db,
}

pub fn find_tone_pair(bb: &Baseband, p: &FskParams) -> Option<Tones> {
    find_tones(bb, p).map(|(f_space, f_mark, peak, noise)| Tones { f_space, f_mark, peak, noise })
}

pub fn demod_fsk(bb: &Baseband, p: &FskParams) -> Option<Symbols> {
    let tones = find_tone_pair(bb, p)?;
    demod_fsk_with_tones(bb, p, tones)
}

/// Demodulate at `p.symbol_rate` given an already-located tone pair.
pub fn demod_fsk_with_tones(bb: &Baseband, p: &FskParams, tones: Tones) -> Option<Symbols> {
    let Tones { f_space, f_mark, peak, noise } = tones;
    let fs = bb.sample_rate.hz();
    // floor, not round: a window shorter than a symbol costs a little SNR, longer causes ISI
    let sym_len = (fs / p.symbol_rate.0).floor().max(2.0) as usize;
    let e_mark = tone_energy(&bb.iq, (f_mark / fs) as f32, sym_len);
    let e_space = tone_energy(&bb.iq, (f_space / fs) as f32, sym_len);
    let d: Vec<f32> = e_mark.iter().zip(&e_space).map(|(m, s)| (m - s) / (m + s + 1e-9)).collect();
    // burst "on" region: where either tone energy is well above the noise-only level
    let e_tot: Vec<f32> = e_mark.iter().zip(&e_space).map(|(m, s)| m.max(*s)).collect();
    let pct = |q: usize| {
        let mut v = e_tot.clone();
        let k = (v.len() * q / 10).min(v.len() - 1);
        *v.select_nth_unstable_by(k, |a, b| a.partial_cmp(b).unwrap()).1
    };
    let on_thresh = pct(1) * 3.0 + pct(9) * 0.2;
    let on: Vec<bool> = e_tot.iter().map(|&e| e > on_thresh).collect();
    let first_on = on.iter().position(|&b| b)?;
    let last_on = on.iter().rposition(|&b| b)?;
    // zero crossings of d inside the on region, linearly interpolated
    let mut crossings = Vec::new();
    for i in (first_on + 1)..=last_on {
        let (a, b) = (d[i - 1], d[i]);
        if (a < 0.0) != (b < 0.0) && (a - b).abs() > 1e-6 {
            crossings.push(i as f32 - 1.0 + a / (a - b));
        }
    }
    let nominal = (fs / p.symbol_rate.0) as f32;
    let (phase, period) = fit_lattice(&crossings, nominal, p.max_rate_error as f32)?;
    // sample at lattice + period/2 (matched filter peaks at symbol end, crossings are mid-transition)
    let k_first = ((first_on as f32 - phase) / period).floor() as i64 - 1;
    let k_last = ((last_on as f32 - phase) / period).ceil() as i64 + 1;
    let mut bits = Vec::with_capacity((k_last - k_first).max(0) as usize);
    let mut t_first = None;
    for k in k_first..=k_last {
        let t = phase + period * (k as f32 + 0.5);
        let i = t.round();
        if i < 0.0 || i as usize >= d.len() {
            continue;
        }
        if t_first.is_none() {
            t_first = Some(t);
        }
        bits.push(d[i as usize] > 0.0);
    }
    let t_first = t_first?;
    let rssi = power_dbfs(&bb.iq[first_on..=last_on]);
    Some(Symbols {
        bits,
        t0: bb.start.offset((t_first as f64 * bb.decim as f64) as i64),
        f_mark: Hertz(bb.center.0 + f_mark),
        f_space: Hertz(bb.center.0 + f_space),
        symbol_rate: Hertz(fs / period as f64),
        rssi,
        snr: Db(peak.0 - noise.0),
        noise,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::burst::Burst;
    use crate::dsp::Iq;
    use crate::units::SampleRate;

    /// Synthesize a Fineoffset-style burst and check bits come back.
    fn synth(bits: &[bool], fs: f64, dev: f64, rate_err: f64, snr_db: f32, seed: u32) -> Baseband {
        let sym = fs / (17_240.0 * (1.0 + rate_err));
        let mut seed = seed;
        let mut rnd = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as f32 / u32::MAX as f32 - 0.5
        };
        let amp = 0.3f32;
        let nstd = amp / 10f32.powf(snr_db / 20.0) * 0.5;
        let pre = (0.003 * fs) as usize;
        let total = pre * 2 + (bits.len() as f64 * sym) as usize;
        let mut ph = 0.0f64;
        let mut iq = Vec::with_capacity(total);
        for n in 0..total {
            let mut s = Iq::new(rnd() * nstd, rnd() * nstd);
            if n >= pre && n < total - pre {
                let k = ((n - pre) as f64 / sym) as usize;
                let f = if bits[k.min(bits.len() - 1)] { dev } else { -dev };
                ph += 2.0 * std::f64::consts::PI * f / fs;
                s += Iq::from_polar(amp, ph as f32);
            }
            iq.push(s);
        }
        let burst = Burst { start: SampleIndex(0), end: SampleIndex(total as u64), f_lo: Hertz(915e6 - dev), f_hi: Hertz(915e6 + dev), peak: Db(-10.0), noise: Db(-50.0), noise_bin_hz: Hertz(4882.8) };
        Baseband { iq, sample_rate: SampleRate(fs as u32), center: Hertz(915e6), start: SampleIndex(0), decim: 1, source_dc_offset: Hertz(-100e3), burst }
    }

    fn frame_bits() -> Vec<bool> {
        let mut bytes = vec![0xaa; 8];
        bytes.extend([0x2d, 0xd4, 0x51, 0x0f, 0x3d, 0xb5, 0x12, 0x00, 0x1b, 0x00, 0x9e, 0x00, 0x00, 0x00, 0x77, 0x33]);
        crate::bits::to_bits(&bytes)
    }

    #[test]
    fn recovers_bits_clean_and_noisy_and_offrate() {
        let want = frame_bits();
        for (fs, dev, err, snr) in [(250e3, 35e3, 0.0, 40.0), (250e3, 35e3, 0.004, 15.0), (312.5e3, 30e3, -0.006, 12.0)] {
            let bb = synth(&want, fs, dev, err, snr, 7);
            let s = demod_fsk(&bb, &FskParams::fineoffset()).expect("demod");
            let pos = crate::bits::find_pattern(&s.bits, &[0x2d, 0xd4, 0x51], 24, 0).expect("sync in bits");
            let got = &s.bits[pos..pos + want.len() - 64];
            let diffs: Vec<usize> = got.iter().zip(&want[64..]).enumerate().filter(|(_, (a, b))| a != b).map(|(i, _)| i).collect();
            assert!(diffs.is_empty(), "fs={fs} err={err} snr={snr} rate={:.1} (nominal {:.1}) nbits={} diffs at {:?}", s.symbol_rate.0, 17240.0 / (1.0 + err), s.bits.len(), diffs);
            assert!((s.f_mark.0 - s.f_space.0 - 2.0 * dev).abs() < 3e3);
            assert!(((s.symbol_rate.0 - 17240.0 * (1.0 + err)) / 17240.0).abs() < 0.002, "rate {}", s.symbol_rate.0);
        }
    }
}
