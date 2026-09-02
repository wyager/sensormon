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

/// Distinct FSK symbol rates the given decoders need (one demod pass each).
pub fn symbol_rates(decoders: &[Box<dyn Decoder>]) -> Vec<Hertz> {
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
pub fn decoders_for_rate<'a>(decoders: &'a [Box<dyn Decoder>], rate: Hertz) -> Vec<&'a Box<dyn Decoder>> {
    decoders.iter().filter(|d| matches!(d.modulation(), Modulation::Fsk { symbol_rate } if (symbol_rate.0 - rate.0).abs() < 1.0)).collect()
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
