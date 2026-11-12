//! Fineoffset / Ecowitt WH52 3-in-1 soil probe (moisture, soil temperature,
//! electrical conductivity): 24 bytes after sync, family 0xA2, CRC over the
//! first 22 bytes in byte 22, sum of the first 23 in byte 23. Layout ported
//! from rtl_433's `src/devices/fineoffset_wh52.c`:
//!
//! ```text
//!  0  1  2  3  4  5  6  7  8  9 10 11 12 .. 19 20 21 22 23
//! A2 II II II Bt tt MM rr rE EE EE rr SS .. SS VV SS XX AA
//! ```
//!
//! B: 3-bit transmission boost counter; t: 13-bit temperature (0.1 °C, offset
//! 40); M: moisture %; E: 20-bit EC raw (µS/cm = raw / 25.6, rtl_433's
//! empirical fit); V: battery in 20 mV steps (one AA cell); S: per-unit
//! constants; r: uncharacterised.

use super::{fineoffset, Band, Decoded, Decoder, FrameError, Modulation, FINEOFFSET_BANDS};
use crate::crc::{crc8_fineoffset, sum8};
use crate::event::{Payload, Wh52};

pub const FRAME_LEN: usize = 24;

/// EC raw counts per µS/cm (rtl_433's calibration against a salt-water series).
pub const EC_RAW_PER_US_CM: f32 = 25.6;

pub fn decode_frame(b: &[u8]) -> Result<Wh52, FrameError> {
    if b.len() < FRAME_LEN {
        return Err(FrameError::Truncated);
    }
    if b[0] != 0xa2 {
        return Err(FrameError::WrongFamily);
    }
    if sum8(&b[..23]) != b[23] || crc8_fineoffset(&b[..22]) != b[22] {
        return Err(FrameError::Integrity);
    }
    let temp_raw = ((b[4] as u16 & 0x1f) << 8) | b[5] as u16;
    let ec_raw = u32::from_be_bytes([0, b[8] & 0x0f, b[9], b[10]]);
    let battery_mv = b[20] as u16 * 20;
    Ok(Wh52 {
        id: u32::from_be_bytes([0, b[1], b[2], b[3]]),
        battery_mv,
        // rtl_433's mapping: the usable 1.3 V..1.6 V of one cell.
        battery_level: ((battery_mv as f32 - 1300.0) / 300.0).clamp(0.0, 1.0),
        temperature_c: temp_raw as f32 * 0.1 - 40.0,
        moisture_pct: b[6],
        conductivity_us_cm: ec_raw as f32 / EC_RAW_PER_US_CM,
        ec_raw,
        boost: (b[4] & 0xe0) >> 5,
    })
}

pub struct Wh52Decoder;

impl Decoder for Wh52Decoder {
    fn name(&self) -> &'static str {
        "Fineoffset-WH52"
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
                Some(Decoded { payload: Payload::Wh52(p), raw, bit_offset: start, inverted: false })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(s: &str) -> Vec<u8> {
        (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
    }

    /// Captured off the air (2026-11-12, tower Airspy): a new probe lying in
    /// free air on a fresh lithium AA.
    const IN_AIR: &str = "a2009a77627d00c0000083166cc0137f7e098bd05a0f867a";

    #[test]
    fn decodes_captured_frame_in_air() {
        let p = decode_frame(&hex(IN_AIR)).unwrap();
        assert_eq!(p.id, 0x009a77);
        assert!((p.temperature_c - 23.7).abs() < 1e-3);
        assert_eq!(p.moisture_pct, 0);
        assert_eq!(p.ec_raw, 0x83);
        assert!((p.conductivity_us_cm - 5.1).abs() < 0.05);
        assert_eq!(p.battery_mv, 1800);
        assert_eq!(p.battery_level, 1.0);
        assert_eq!(p.boost, 3);
    }

    /// A second unit, and the first one while being handled (3 % moisture).
    #[test]
    fn decodes_other_captured_frames() {
        let p = decode_frame(&hex("a200a463e27a00c000007f166cc0138277ec7b905a0f3d2f")).unwrap();
        assert_eq!(p.id, 0x00a463);
        assert!((p.temperature_c - 23.4).abs() < 1e-3);
        assert_eq!(p.boost, 7);
        let p = decode_frame(&hex("a2009a77c26a03ec000082166cc0137f7e098bd05a0f7fee")).unwrap();
        assert_eq!(p.moisture_pct, 3);
        assert!((p.temperature_c - 21.8).abs() < 1e-3);
    }

    #[test]
    fn rejects_corruption_and_other_families() {
        let mut b = hex(IN_AIR);
        b[6] ^= 0x01;
        assert_eq!(decode_frame(&b), Err(FrameError::Integrity));
        let mut b = hex(IN_AIR);
        b[0] = 0x51;
        assert_eq!(decode_frame(&b), Err(FrameError::WrongFamily));
        assert_eq!(decode_frame(&hex(IN_AIR)[..23]), Err(FrameError::Truncated));
    }
}
