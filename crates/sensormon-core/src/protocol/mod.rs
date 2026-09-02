//! Protocol decoders: pure functions from demodulated bits to typed payloads.

pub mod fineoffset;
pub mod toyota_tpms;
pub mod wh25;
pub mod wh31l;
pub mod wh51;
pub mod wh55;
pub mod ws90;

use crate::bits;
use crate::event::Payload;
use crate::units::Hertz;

/// What a decoder expects the demodulator to have produced.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Modulation {
    /// 2-FSK, NRZ, mark (higher tone) = 1.
    Fsk { symbol_rate: Hertz },
}

/// One decoded frame out of a bit string.
#[derive(Clone, Debug, PartialEq)]
pub struct Decoded {
    pub payload: Payload,
    /// Frame bytes after the sync word, integrity bytes included.
    pub raw: Vec<u8>,
    /// Bit offset of the frame's first byte in the (possibly inverted) bit string.
    pub bit_offset: usize,
    /// Whether the bits had to be inverted (space = 1) to decode.
    pub inverted: bool,
}

/// Why a candidate frame was rejected. Useful for diagnostics counters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FrameError {
    #[error("not this sensor family")]
    WrongFamily,
    #[error("frame truncated")]
    Truncated,
    #[error("integrity check failed")]
    Integrity,
}

/// A decoder for one sensor family.
pub trait Decoder: Send + Sync {
    /// rtl_433-compatible name, e.g. "Fineoffset-WH51".
    fn name(&self) -> &'static str;
    fn modulation(&self) -> Modulation;
    /// RF bands this family transmits in; a capture centered outside all of
    /// them doesn't run this decoder (saves a demodulation pass per symbol rate).
    fn bands(&self) -> &'static [Band];
    /// Decode every frame present in `bits` (a burst may carry repeats).
    fn decode(&self, bits: &[bool]) -> Vec<Decoded>;
}

/// All decoders sensormon knows about.
pub fn decoders() -> Vec<Box<dyn Decoder>> {
    vec![
        Box::new(ws90::Ws90Decoder),
        Box::new(wh51::Wh51Decoder),
        Box::new(wh55::Wh55Decoder),
        Box::new(wh31l::Wh31lDecoder),
        Box::new(wh25::Wh25Decoder),
        Box::new(toyota_tpms::ToyotaTpmsDecoder),
    ]
}

/// An RF band a sensor family uses (center ± half-width).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Band {
    pub center: Hertz,
    pub half_width: Hertz,
}

impl Band {
    pub const fn mhz(center: f64, half_width: f64) -> Band {
        Band { center: Hertz(center * 1e6), half_width: Hertz(half_width * 1e6) }
    }
    pub fn contains(&self, f: Hertz) -> bool {
        (f.0 - self.center.0).abs() <= self.half_width.0
    }
}

/// ISM bands Fineoffset/Ecowitt sensors ship for.
pub const FINEOFFSET_BANDS: &[Band] = &[Band::mhz(433.92, 2.0), Band::mhz(868.35, 2.0), Band::mhz(915.0, 13.0)];

/// The decoders relevant to a capture centered at `center`.
pub fn decoders_for_center<'a>(decoders: &'a [Box<dyn Decoder>], center: Hertz) -> Vec<&'a Box<dyn Decoder>> {
    decoders.iter().filter(|d| d.bands().iter().any(|b| b.contains(center))).collect()
}

/// Distinct FSK symbol rates the given decoders need (one demod pass each).
pub fn symbol_rates(decoders: &[&Box<dyn Decoder>]) -> Vec<Hertz> {
    let mut rates: Vec<Hertz> = Vec::new();
    for d in decoders {
        let Modulation::Fsk { symbol_rate } = d.modulation();
        if !rates.iter().any(|r| (r.0 - symbol_rate.0).abs() < 1.0) {
            rates.push(symbol_rate);
        }
    }
    rates
}

/// Decoders that want this symbol rate.
pub fn decoders_for_rate<'a>(decoders: &[&'a Box<dyn Decoder>], rate: Hertz) -> Vec<&'a Box<dyn Decoder>> {
    decoders.iter().copied().filter(|d| matches!(d.modulation(), Modulation::Fsk { symbol_rate } if (symbol_rate.0 - rate.0).abs() < 1.0)).collect()
}

/// Run every decoder over the bits and over their inversion, de-duplicating
/// identical frames (a frame that decodes in both polarities is one frame).
pub fn decode_all(decoders: &[&Box<dyn Decoder>], bits: &[bool]) -> Vec<Decoded> {
    let inv = bits::inverted(bits);
    let mut out: Vec<Decoded> = Vec::new();
    for (inverted, b) in [(false, bits), (true, &inv[..])] {
        for d in decoders {
            for mut dec in d.decode(b) {
                dec.inverted = inverted;
                if !out.iter().any(|o| o.raw == dec.raw) {
                    out.push(dec);
                }
            }
        }
    }
    out
}
