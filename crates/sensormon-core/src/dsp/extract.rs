//! Cut one burst out of the raw stream and bring it to a narrow baseband:
//! mix its center to DC, low-pass to its bandwidth, decimate.

use super::burst::Burst;
use super::fir::{filter_decimate, lowpass_taps};
use super::ring::SampleRing;
use super::Iq;
use crate::units::{Hertz, SampleIndex, SampleRate};
use std::f32::consts::PI;

/// A burst at low sample rate, centered on its own carrier.
#[derive(Clone, Debug)]
pub struct Baseband {
    pub iq: Vec<Iq>,
    pub sample_rate: SampleRate,
    /// Absolute RF frequency now at DC.
    pub center: Hertz,
    /// Absolute source-stream index corresponding to `iq[0]` (group delay compensated).
    pub start: SampleIndex,
    /// Source samples per baseband sample.
    pub decim: usize,
    /// Where the capture's own DC (SDR offset spike) landed in this baseband, relative to DC.
    pub source_dc_offset: Hertz,
    pub burst: Burst,
}

#[derive(Clone, Copy, Debug)]
pub struct ExtractConfig {
    /// Target baseband rate; the actual rate is `source / floor(source/target)`.
    pub target_rate: SampleRate,
    /// Extra time kept before and after the detected span.
    pub pad_s: f32,
    /// Low-pass cutoff = max(bandwidth/2 * factor, min_cutoff).
    pub cutoff_factor: f64,
    pub min_cutoff: Hertz,
    pub taps: usize,
}

impl Default for ExtractConfig {
    fn default() -> Self {
        // Wide by default: the FSK matched filters are only a symbol-rate wide, so
        // extra bandwidth here costs no SNR, while a burst whose detected center is
        // off by a tone spacing must still keep both tones inside the passband.
        ExtractConfig { target_rate: SampleRate(250_000), pad_s: 0.002, cutoff_factor: 1.2, min_cutoff: Hertz::khz(110.0), taps: 95 }
    }
}

/// `None` if the ring no longer holds the burst.
pub fn extract(ring: &SampleRing, burst: &Burst, source_rate: SampleRate, source_center: Hertz, cfg: &ExtractConfig) -> Option<Baseband> {
    let pad = source_rate.samples_in(cfg.pad_s as f64) as i64;
    let start = SampleIndex(burst.start.0.saturating_sub(pad as u64)).max(ring.oldest_index());
    let end = burst.end.offset(pad).min(ring.next_index());
    let raw = ring.get(start, end)?;
    // A region wide enough to hold two signals (a burst merged with a neighbour)
    // is kept whole at a higher baseband rate, so the tone search can still pick
    // the strongest pair anywhere inside it.
    let wanted = cfg.target_rate.hz().max(burst.bandwidth().0 * cfg.cutoff_factor * 2.5);
    let decim = ((source_rate.hz() / wanted).floor() as usize).max(1);
    let out_rate = SampleRate((source_rate.hz() / decim as f64).round() as u32);
    let offset = burst.center() - source_center; // Hz relative to capture center
    let cutoff = (burst.bandwidth().0 / 2.0 * cfg.cutoff_factor).max(cfg.min_cutoff.0).min(out_rate.hz() * 0.45);
    let w = -2.0 * PI * (offset.0 / source_rate.hz()) as f32;
    // mix to DC; rotate per sample with an incremental phasor to avoid trig per sample
    let step = Iq::from_polar(1.0, w);
    let mut ph = Iq::new(1.0, 0.0);
    let mixed: Vec<Iq> = raw
        .iter()
        .map(|&s| {
            let y = s * ph;
            ph *= step;
            y
        })
        .collect();
    let taps = lowpass_taps((cutoff / source_rate.hz()) as f32, cfg.taps, 8.0);
    let iq = filter_decimate(&mixed, &taps, decim);
    Some(Baseband { iq, sample_rate: out_rate, center: burst.center(), start: start.offset((cfg.taps / 2) as i64), decim, source_dc_offset: Hertz(-offset.0), burst: *burst })
}
