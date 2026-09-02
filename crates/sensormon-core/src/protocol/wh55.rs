//! Fineoffset WH55 leak sensor: 9 bytes from the family byte (0x55), CRC over 9 == 0.

use super::{fineoffset, Band, Decoded, Decoder, FrameError, Modulation, FINEOFFSET_BANDS};
use crate::crc::crc8_fineoffset;
use crate::event::{Payload, Wh55};

pub const FRAME_LEN: usize = 9;

pub fn decode_frame(b: &[u8]) -> Result<Wh55, FrameError> {
    if b.len() < FRAME_LEN {
        return Err(FrameError::Truncated);
    }
    if b[0] != 0x55 {
        return Err(FrameError::WrongFamily);
    }
    if crc8_fineoffset(&b[..9]) != 0 {
        return Err(FrameError::Integrity);
    }
    Ok(Wh55 {
        id: u16::from_be_bytes([b[2], b[3]]),
        channel: (b[1] >> 4) + 1,
        battery_level: b[4] as f32 * 0.2,
        raw_value: u16::from_be_bytes([b[5], b[6]]),
        high_sensitivity: (b[7] >> 7) & 1 == 1,
        alarm: (b[7] >> 6) & 1 == 1,
    })
}

pub struct Wh55Decoder;

impl Decoder for Wh55Decoder {
    fn name(&self) -> &'static str {
        "Fineoffset-WH55"
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
                Some(Decoded { payload: Payload::Wh55(p), raw, bit_offset: start, inverted: false })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut b = vec![0x55, 0x10, 0x5f, 0x13, 0x05, 0x01, 0x2c, 0x80];
        b.push(crc8_fineoffset(&b));
        let p = decode_frame(&b).unwrap();
        assert_eq!((p.id, p.channel, p.raw_value), (0x5f13, 2, 300));
        assert!((p.battery_level - 1.0).abs() < 1e-6);
        assert!(p.high_sensitivity && !p.alarm);
    }
}
