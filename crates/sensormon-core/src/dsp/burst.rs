//! Time-frequency burst detection over the full capture bandwidth.
//!
//! Each hop of `fft_size` samples is turned into a power spectrum; a per-bin
//! noise floor is tracked (fast to fall, slow to rise, so bursts don't lift
//! it); bins above floor + threshold are "hot". Runs of hot bins (merging
//! across gaps up to `merge_gap`, so an FSK tone pair is one region) are
//! tracked over consecutive hops; when a region has been silent for
//! `end_gap` it becomes a `Burst`. Bins that stay hot longer than
//! `max_duration` are treated as carriers (comb lines, CW) and ignored.

use super::Iq;
use crate::units::{Db, Hertz, SampleIndex, SampleRate};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct BurstDetectorConfig {
    /// Samples per hop / FFT (no overlap). 512 at 2.5 MS/s → 4.9 kHz bins, 0.2 ms hops.
    pub fft_size: usize,
    /// Hot if above the tracked floor by this much (keeps an open region alive).
    pub threshold: Db,
    /// A new region needs at least one bin this far above the floor (hysteresis
    /// against chains of noise hits).
    pub open_threshold: Db,
    /// Hot bins closer than this are one region (FSK tones are ~70 kHz apart).
    pub merge_gap: Hertz,
    /// A region ends after this much silence.
    pub end_gap_s: f32,
    pub min_duration_s: f32,
    /// Longer than this is a carrier, not a burst.
    pub max_duration_s: f32,
    pub min_bandwidth: Hertz,
    pub max_bandwidth: Hertz,
    /// Ignore this many bins around DC (SDR DC spike) — 0 to disable.
    pub dc_guard_bins: usize,
    /// A region's edges are the outermost bins within this much of its peak, so
    /// leakage/splatter from a strong burst doesn't inflate its bandwidth.
    pub dynamic_range: Db,
    /// Variance-aware opening (CFAR-style): besides `open_threshold`, a new
    /// region needs `p > floor * (1 + variance_k * sigma)` where `sigma` is the
    /// bin's tracked standard deviation of `(p - floor) / floor`. For noise
    /// (3-bin smoothed power, Gamma(3)) sigma ≈ 0.58, so with k = 25 the extra
    /// term (1 + 14.4 → 11.9 dB) sits below `open_threshold` and changes
    /// nothing; a bin that flickers (FM modulation, dithered SMPS harmonics,
    /// 8-VSB) has sigma of several and demands a proportionally larger excursion.
    pub variance_k: f32,
    /// EMA rate of the variance tracker per hop (0.002 ≈ 50 ms at 0.1 ms hops).
    pub variance_alpha: f32,
}

impl Default for BurstDetectorConfig {
    fn default() -> Self {
        BurstDetectorConfig {
            fft_size: 512,
            threshold: Db(9.0),
            open_threshold: Db(13.0),
            merge_gap: Hertz::khz(100.0),
            end_gap_s: 0.0015,
            min_duration_s: 0.003,
            max_duration_s: 0.25,
            min_bandwidth: Hertz::khz(15.0),
            max_bandwidth: Hertz::khz(1200.0), // a clipped strong burst splatters far; extraction re-narrows it
            dc_guard_bins: 2,
            dynamic_range: Db(25.0),
            variance_k: 30.0,
            variance_alpha: 0.002,
        }
    }
}

/// A detected transmission: absolute sample span and RF frequency span.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Burst {
    pub start: SampleIndex,
    pub end: SampleIndex,
    /// Absolute RF edges of the occupied band.
    pub f_lo: Hertz,
    pub f_hi: Hertz,
    pub peak: Db,
    /// Noise floor (dBFS per detector bin) under the burst when it started.
    pub noise: Db,
    /// Width of the bins `noise` refers to.
    pub noise_bin_hz: Hertz,
}

impl Burst {
    pub fn center(&self) -> Hertz {
        (self.f_lo + self.f_hi) / 2.0
    }
    pub fn bandwidth(&self) -> Hertz {
        self.f_hi - self.f_lo
    }
}

#[derive(Clone, Debug)]
struct Region {
    lo_bin: usize,
    hi_bin: usize,
    start_hop: u64,
    last_hop: u64,
    peak: Db,
    noise: Db,
}

pub struct BurstDetector {
    cfg: BurstDetectorConfig,
    rate: SampleRate,
    center: Hertz,
    fft: Arc<dyn Fft<f32>>,
    window: Vec<f32>,
    scratch: Vec<Complex<f32>>,
    /// Tracked noise floor per bin, linear power (arithmetic mean of non-burst hops).
    floor: Vec<f32>,
    /// Tracked variance per bin of the relative excursion `(p - floor) / floor`
    /// (clipped at +30 so one strong burst can't blind its bin for long).
    var: Vec<f32>,
    /// Per-bin opening ratio this hop: `max(r_open, 1 + k * sqrt(var))`.
    open_ratio: Vec<f32>,
    /// Threshold ratios derived once from the dB config (see `ratios`).
    r_hot: f32,
    r_open: f32,
    trim_ratio: f32,
    rise_cap: f32,
    /// Scratch buffers reused every hop.
    p_lin: Vec<f32>,
    s_lin: Vec<f32>,
    hot: Vec<bool>,
    hot_streak: Vec<u32>,
    active: Vec<Region>,
    /// Samples not yet forming a whole hop.
    pending: Vec<Iq>,
    /// Absolute index of `pending[0]` (or of the next hop if pending is empty).
    pending_index: SampleIndex,
    hops_done: u64,
    /// Absolute index of hop 0.
    hop0_index: SampleIndex,
    /// Linear-power accumulator for the first `INIT_HOPS` hops.
    init_acc: Vec<f32>,
    init_hops: u32,
    trace_bin: Option<usize>,
}

const INIT_HOPS: u32 = 16;

/// Thresholds are configured in dB above the floor as it was originally
/// tracked (an average of dB values, i.e. the geometric mean of the noise).
/// The floor is now tracked in linear power (arithmetic mean). What gets
/// thresholded is the 3-bin smoothed power, a Gamma(3) variable, whose
/// geometric mean sits (10/ln 10)(ψ(3) − ln 3) ≈ 0.76 dB below its arithmetic
/// mean, so the same operating point is `threshold − 0.76 dB` as a ratio.
fn ratio(over_geometric_mean: Db) -> f32 {
    10f32.powf((over_geometric_mean.0 - 0.76) / 10.0)
}

fn to_db(lin: f32) -> f32 {
    10.0 * (lin + 1e-20).log10()
}

impl BurstDetector {
    pub fn new(cfg: BurstDetectorConfig, rate: SampleRate, center: Hertz, first_index: SampleIndex) -> Self {
        let n = cfg.fft_size;
        let fft = FftPlanner::<f32>::new().plan_fft_forward(n);
        // 4-term Blackman-Harris: -92 dB sidelobes, so a -7 dBFS burst doesn't leak
        // above a -60 dBFS floor across the band the way Hann (-31 dB) would.
        let window: Vec<f32> = (0..n)
            .map(|i| {
                let x = 2.0 * std::f32::consts::PI * i as f32 / n as f32;
                0.35875 - 0.48829 * x.cos() + 0.14128 * (2.0 * x).cos() - 0.01168 * (3.0 * x).cos()
            })
            .collect();
        let (r_hot, r_open, trim_ratio) = (ratio(cfg.threshold), ratio(cfg.open_threshold), 10f32.powf(cfg.dynamic_range.0 / 10.0));
        BurstDetector {
            cfg,
            rate,
            center,
            fft,
            window,
            scratch: vec![Complex::new(0.0, 0.0); n],
            floor: vec![1e-10; n],
            var: vec![1.0 / 3.0; n],
            open_ratio: vec![r_open; n],
            r_hot,
            r_open,
            trim_ratio,
            rise_cap: 10f32.powf(0.002),
            p_lin: vec![0.0; n],
            s_lin: vec![0.0; n],
            hot: vec![false; n],
            hot_streak: vec![0; n],
            active: Vec::new(),
            pending: Vec::with_capacity(n),
            pending_index: first_index,
            hops_done: 0,
            hop0_index: first_index,
            init_acc: vec![0.0; n],
            init_hops: 0,
            trace_bin: std::env::var("SENSORMON_BURST_TRACE").ok().and_then(|v| v.parse().ok()),
        }
    }

    pub fn config(&self) -> &BurstDetectorConfig {
        &self.cfg
    }

    fn hop_s(&self) -> f32 {
        self.cfg.fft_size as f32 / self.rate.hz() as f32
    }
    fn hops(&self, seconds: f32) -> u64 {
        (seconds / self.hop_s()).ceil().max(1.0) as u64
    }
    fn bin_hz(&self) -> f64 {
        self.rate.hz() / self.cfg.fft_size as f64
    }
    /// Absolute RF frequency of the lower edge of bin `k` (bins are FFT-shifted: k=0 is -fs/2).
    fn bin_freq(&self, k: usize) -> Hertz {
        Hertz(self.center.0 + (k as f64 - self.cfg.fft_size as f64 / 2.0) * self.bin_hz())
    }

    /// Feed samples (any length); returns bursts that completed.
    pub fn push(&mut self, block: &[Iq]) -> Vec<Burst> {
        let n = self.cfg.fft_size;
        let mut out = Vec::new();
        let mut pos = 0;
        while pos < block.len() {
            let take = (n - self.pending.len()).min(block.len() - pos);
            self.pending.extend_from_slice(&block[pos..pos + take]);
            pos += take;
            if self.pending.len() == n {
                let hop_start = self.pending_index;
                let pending = std::mem::take(&mut self.pending);
                self.process_hop(&pending, &mut out);
                self.pending = pending;
                self.pending.clear();
                self.pending_index = hop_start.offset(n as i64);
            }
        }
        out
    }

    fn process_hop(&mut self, hop: &[Iq], out: &mut Vec<Burst>) {
        let n = self.cfg.fft_size;
        let hop_idx = self.hops_done;
        self.hops_done += 1;
        for i in 0..n {
            self.scratch[i] = hop[i] * self.window[i];
        }
        self.fft.process(&mut self.scratch);
        // power per bin, fft-shifted, linear
        let norm = 1.0 / (n as f32 * 0.2576); // window power normalisation (BH4: mean w^2 ≈ 0.2576)
        for k in 0..n {
            let src = (k + n / 2) % n;
            self.p_lin[k] = self.scratch[src].norm_sqr() * norm;
        }
        let max_hops = self.hops(self.cfg.max_duration_s);
        let dc_lo = n / 2 - self.cfg.dc_guard_bins;
        let dc_hi = n / 2 + self.cfg.dc_guard_bins;
        // 3-bin smoothing knocks single-bin noise spikes down
        for k in 0..n {
            let l = self.p_lin[k.saturating_sub(1)];
            let r = self.p_lin[(k + 1).min(n - 1)];
            self.s_lin[k] = (l + self.p_lin[k] + r) * (1.0 / 3.0);
        }
        // Startup: average the first hops before detecting anything.
        if self.init_hops < INIT_HOPS {
            for k in 0..n {
                self.init_acc[k] += self.s_lin[k];
            }
            self.init_hops += 1;
            if self.init_hops == INIT_HOPS {
                for k in 0..n {
                    self.floor[k] = self.init_acc[k] / INIT_HOPS as f32;
                }
            }
            return;
        }
        let trace_bin = self.trace_bin;
        let (r_hot, r_open, rise_cap) = (self.r_hot, self.r_open, self.rise_cap);
        let (var_k, var_alpha) = (self.cfg.variance_k, self.cfg.variance_alpha);
        for k in 0..n {
            let f = self.floor[k];
            let p = self.s_lin[k];
            let is_hot = p > f * r_hot;
            // Variance of the relative excursion, tracked only while the bin is not
            // hot: a genuine burst must not blind its own bin afterwards, while a
            // flickering bin (FM modulation, dithered SMPS line) shows its spread in
            // the sub-threshold samples too (noise: e in [-1, ~3], var ≈ 1/3).
            if !is_hot {
                let e = (p - f) / f;
                self.var[k] += (e * e - self.var[k]) * var_alpha;
            }
            self.open_ratio[k] = r_open.max(1.0 + var_k * self.var[k].sqrt());
            self.hot_streak[k] = if is_hot { self.hot_streak[k] + 1 } else { 0 };
            let carrier = self.hot_streak[k] as u64 > max_hops;
            // Floor tracker: exponential average toward the current level, with the
            // upward step rate-limited (0.02 dB/hop) while hot so a burst lifts it by
            // at most a few dB while a permanent carrier is absorbed within ~0.1 s.
            let next = f + (p - f) * 0.02;
            self.floor[k] = if is_hot { next.min(f * rise_cap) } else { next };
            let in_dc = self.cfg.dc_guard_bins > 0 && (dc_lo..=dc_hi).contains(&k);
            self.hot[k] = is_hot && !carrier && !in_dc;
            if trace_bin == Some(k) && hop_idx.is_multiple_of(20) {
                eprintln!("hop {hop_idx} bin {k}: p={:.1} floor={:.1} hot={is_hot} streak={}", to_db(p), to_db(f), self.hot_streak[k]);
            }
        }
        // group hot bins into regions, bridging gaps up to merge_gap
        let gap_bins = (self.cfg.merge_gap.0 / self.bin_hz()).round() as usize;
        let mut regions: Vec<(usize, usize, f32, f32)> = Vec::new(); // lo, hi, peak_db, noise_db
        let (hot, s_lin, floor, open_ratio) = (&self.hot, &self.s_lin, &self.floor, &self.open_ratio);
        let mut k = 0;
        while k < n {
            if hot[k] {
                let lo = k;
                let mut hi = k;
                let mut peak = s_lin[k];
                let mut j = k + 1;
                let mut last_hot = k;
                while j < n && j - last_hot <= gap_bins {
                    if hot[j] {
                        last_hot = j;
                        hi = j;
                        peak = peak.max(s_lin[j]);
                    }
                    j += 1;
                }
                // trim to the bins within `dynamic_range` of the peak
                let cut = peak / self.trim_ratio;
                let lo_t = (lo..=hi).find(|&b| hot[b] && s_lin[b] >= cut).unwrap_or(lo);
                let hi_t = (lo..=hi).rev().find(|&b| hot[b] && s_lin[b] >= cut).unwrap_or(hi);
                let noise = floor[lo_t..=hi_t].iter().cloned().fold(f32::INFINITY, f32::min);
                // opening needs the peak bin to clear its own variance-aware ratio
                let opens = (lo_t..=hi_t).any(|b| hot[b] && s_lin[b] > floor[b] * open_ratio[b]) && peak > noise * self.r_open;
                if opens || self.active.iter().any(|a| lo_t <= a.hi_bin && a.lo_bin <= hi_t) {
                    // dB only per region (a handful per hop), never per bin
                    regions.push((lo_t, hi_t, to_db(peak), to_db(noise)));
                }
                k = hi + 1;
            } else {
                k += 1;
            }
        }
        // Match this hop's regions to active ones by actual overlap. An active
        // region's bounds are the union over its strong hops (within 10 dB of its
        // peak): a running union over *all* hops would let neighbouring noise hits
        // widen it without limit, while a single strongest hop can catch only one
        // of an FSK pair's tones and mis-center the burst.
        for (lo, hi, peak, noise) in regions {
            let found = self.active.iter().position(|a| lo <= a.hi_bin && a.lo_bin <= hi);
            match found {
                Some(i) => {
                    let a = &mut self.active[i];
                    a.last_hop = hop_idx;
                    if peak > a.peak.0 + 10.0 {
                        a.peak = Db(peak);
                        a.lo_bin = lo;
                        a.hi_bin = hi;
                    } else if peak >= a.peak.0 - 10.0 {
                        a.peak = Db(a.peak.0.max(peak));
                        a.lo_bin = a.lo_bin.min(lo);
                        a.hi_bin = a.hi_bin.max(hi);
                    }
                }
                None => {
                    if trace_bin.is_some() {
                        eprintln!("open region bins {lo}..{hi} at hop {hop_idx} peak {peak:.1} floor-there {:.1}", to_db(self.floor[lo]));
                    }
                    self.active.push(Region { lo_bin: lo, hi_bin: hi, start_hop: hop_idx, last_hop: hop_idx, peak: Db(peak), noise: Db(noise) });
                }
            }
        }
        // close regions that went silent
        let end_gap = self.hops(self.cfg.end_gap_s);
        let min_hops = self.hops(self.cfg.min_duration_s);
        let mut i = 0;
        while i < self.active.len() {
            let a = &self.active[i];
            let silent = hop_idx - a.last_hop;
            let duration = a.last_hop - a.start_hop + 1;
            if silent >= end_gap || duration > max_hops {
                let a = self.active.remove(i);
                if trace_bin.is_some() {
                    eprintln!("close region bins {}..{} hops {}..{} peak {:.1} noise {:.1} (silent {silent})", a.lo_bin, a.hi_bin, a.start_hop, a.last_hop, a.peak.0, a.noise.0);
                }
                let bw = Hertz((a.hi_bin - a.lo_bin + 1) as f64 * self.bin_hz());
                if duration >= min_hops && duration <= max_hops && bw >= self.cfg.min_bandwidth && bw <= self.cfg.max_bandwidth {
                    out.push(Burst {
                        start: self.hop0_index.offset((a.start_hop * n as u64) as i64),
                        end: self.hop0_index.offset(((a.last_hop + 1) * n as u64) as i64),
                        f_lo: self.bin_freq(a.lo_bin),
                        f_hi: self.bin_freq(a.hi_bin + 1),
                        peak: a.peak,
                        noise: a.noise,
                        noise_bin_hz: Hertz(self.bin_hz()),
                    });
                }
            } else {
                i += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    /// Noise plus one FSK-like tone pair burst plus a permanent carrier; expect exactly one burst.
    #[test]
    fn detects_burst_ignores_carrier() {
        let rate = SampleRate(2_500_000);
        let mut det = BurstDetector::new(BurstDetectorConfig::default(), rate, Hertz::mhz(915.0), SampleIndex(0));
        let mut seed = 0x1234_5678u32;
        let mut rnd = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed as f32 / u32::MAX as f32 - 0.5) * 0.002
        };
        let total = rate.samples_in(0.3);
        let b_start = rate.samples_in(0.1);
        let b_end = rate.samples_in(0.115);
        let mut x = Vec::with_capacity(total);
        let mut phase = 0.0f32;
        let mut carrier_phase = 0.0f64; // f64: an f32 2*pi*f*t loses precision after ~0.2 s and sprays spurs
        for n in 0..total {
            let mut s = Iq::new(rnd(), rnd());
            carrier_phase = (carrier_phase + 2.0 * std::f64::consts::PI * 400e3 / rate.hz()) % (2.0 * std::f64::consts::PI);
            s += Iq::from_polar(0.02, carrier_phase as f32); // carrier at +400 kHz
            if (b_start..b_end).contains(&n) {
                let tone: f32 = if (n / 145) % 2 == 0 { 35e3 } else { -35e3 };
                phase += 2.0 * PI * tone / rate.hz() as f32; // continuous-phase FSK like a real transmitter
                s += Iq::from_polar(0.05, phase);
            }
            x.push(s);
        }
        let mut bursts = Vec::new();
        for chunk in x.chunks(10_000) {
            bursts.extend(det.push(chunk));
        }
        let real: Vec<&Burst> = bursts.iter().filter(|b| (b.start.0 as usize) < b_end && (b.end.0 as usize) > b_start).collect();
        assert_eq!(real.len(), 1, "{bursts:?}");
        assert!(bursts.len() <= 3, "too many false bursts: {bursts:?}");
        let b = *real[0];
        assert!(b.start.0 as usize <= b_start + 512 && b.end.0 as usize >= b_end - 512, "{b:?}");
        assert!((b.center().0 - 915e6).abs() < 30e3, "{b:?}");
        assert!(b.bandwidth().0 > 60e3 && b.bandwidth().0 < 260e3, "{b:?}");
    }
}
