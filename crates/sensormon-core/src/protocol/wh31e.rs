//! Fineoffset WH31E / Ecowitt WN31 (Ambient Weather WH31E) multi-channel
//! temperature + humidity sensor (rtl_433: AmbientWeather-WH31E): 7 bytes from
//! family byte 0x30, CRC over 6 == 0, byte 6 = sum of first 6.
//!
//! ```text
//! 30 II BT TT HH XX AA
//! ```
//! `I` id, `B` battery-low bit + 3-bit channel (0-based), `T` 12-bit
//! temperature (0.1 °C, offset 400), `H` humidity %.

use super::{fineoffset, Band, Decoded, Decoder, FrameError, Modulation, FINEOFFSET_BANDS};
use crate::crc::{crc8_fineoffset, sum8};
use crate::event::{Payload, Wh31e};

pub const FRAME_LEN: usize = 7;

pub fn decode_frame(b: &[u8]) -> Result<Wh31e, FrameError> {
    if b.len() < FRAME_LEN {
        return Err(FrameError::Truncated);
    }
    if b[0] != 0x30 {
        return Err(FrameError::WrongFamily);
    }
    if crc8_fineoffset(&b[..6]) != 0 || sum8(&b[..6]) != b[6] {
        return Err(FrameError::Integrity);
    }
    let temp_raw = ((b[2] as u16 & 0x0f) << 8) | b[3] as u16;
    Ok(Wh31e {
        id: b[1],
        channel: ((b[2] & 0x70) >> 4) + 1,
        battery_ok: b[2] & 0x80 == 0,
        temperature_c: (temp_raw as f32 - 400.0) * 0.1,
        humidity_pct: b[4],
    })
}

pub struct Wh31eDecoder;

impl Decoder for Wh31eDecoder {
    fn name(&self) -> &'static str {
        "AmbientWeather-WH31E"
    }
    fn modulation(&self) -> Modulation {
        Modulation::Fsk { symbol_rate: fineoffset::SYMBOL_RATE }
    }
    fn bands(&self) -> &'static [Band] {
        FINEOFFSET_BANDS
    }
    fn decode(&self, bits: &[bool]) -> Vec<Decoded> {
        fineoffset::frame_starts(bits)
            .into_iter()
            .filter_map(|start| {
                let raw = fineoffset::frame_bytes(bits, start, FRAME_LEN)?;
                let p = decode_frame(&raw).ok()?;
                Some(Decoded { payload: Payload::Wh31e(p), raw, bit_offset: start, inverted: false })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example frame from rtl_433's `ambientweather_wh31e.c`.
    const RTL433_SAMPLE: [u8; 7] = [0x30, 0xc3, 0x81, 0xd5, 0x5c, 0x2a, 0xcf];

    #[test]
    fn decodes_rtl433_sample() {
        let p = decode_frame(&RTL433_SAMPLE).unwrap();
        assert_eq!(p.id, 0xc3);
        assert_eq!(p.channel, 1);
        assert!(!p.battery_ok);
        assert!((p.temperature_c - 6.9).abs() < 1e-4);
        assert_eq!(p.humidity_pct, 92);
    }

    /// Captured off the air from the crawlspace WH31E (2026-11-10, tower Airspy).
    #[test]
    fn decodes_captured_frame() {
        let p = decode_frame(&[0x30, 0xf7, 0x82, 0x54, 0x49, 0xc7, 0x0d]).unwrap();
        assert_eq!(p.id, 0xf7);
        assert_eq!(p.channel, 1);
        assert!(!p.battery_ok);
        assert!((p.temperature_c - 19.6).abs() < 1e-4);
        assert_eq!(p.humidity_pct, 73);
    }

    #[test]
    fn rejects_corruption_and_other_families() {
        let mut b = RTL433_SAMPLE;
        b[3] ^= 0x01;
        assert_eq!(decode_frame(&b), Err(FrameError::Integrity));
        let mut b = RTL433_SAMPLE;
        b[0] = 0x57;
        assert_eq!(decode_frame(&b), Err(FrameError::WrongFamily));
        assert_eq!(decode_frame(&RTL433_SAMPLE[..6]), Err(FrameError::Truncated));
    }
}
