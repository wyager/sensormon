//! Fineoffset WH51 soil moisture: 14 bytes after sync, family 0x51.

use super::{fineoffset, Band, Decoded, Decoder, FrameError, Modulation, FINEOFFSET_BANDS};
use crate::crc::{crc8_fineoffset, sum8};
use crate::event::{Payload, Wh51};

pub const FRAME_LEN: usize = 14;

pub fn decode_frame(b: &[u8]) -> Result<Wh51, FrameError> {
    if b.len() < FRAME_LEN {
        return Err(FrameError::Truncated);
    }
    if b[0] != 0x51 {
        return Err(FrameError::WrongFamily);
    }
    if sum8(&b[..13]) != b[13] || crc8_fineoffset(&b[..12]) != b[12] {
        return Err(FrameError::Integrity);
    }
    let battery_bits = b[4] & 0x1f;
    let battery_level = match battery_bits {
        16.. => 1.0,
        15 => 0.9,
        14 => 0.5,
        13 => 0.1,
        _ => 0.0,
    };
    Ok(Wh51 {
        id: u32::from_be_bytes([0, b[1], b[2], b[3]]),
        battery_mv: battery_bits as u16 * 100,
        battery_level,
        moisture_pct: b[6],
        boost: (b[4] & 0xe0) >> 5,
        ad_raw: (((b[7] & 0x01) as u16) << 8) | b[8] as u16,
    })
}

pub struct Wh51Decoder;

impl Decoder for Wh51Decoder {
    fn name(&self) -> &'static str {
        "Fineoffset-WH51"
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
                Some(Decoded { payload: Payload::Wh51(p), raw, bit_offset: start, inverted: false })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crc::{crc8_fineoffset, sum8};

    /// Build a valid frame from fields, the way the sensor firmware would.
    pub fn make_frame(id: u32, battery_bits: u8, boost: u8, moisture: u8, ad_raw: u16) -> Vec<u8> {
        let mut b = vec![0x51, (id >> 16) as u8, (id >> 8) as u8, id as u8, (boost << 5) | battery_bits, 0x00, moisture, (ad_raw >> 8) as u8 & 1, ad_raw as u8, 0, 0, 0];
        b.push(crc8_fineoffset(&b));
        b.push(sum8(&b));
        b
    }

    #[test]
    fn roundtrip() {
        let f = make_frame(0x0f3db5, 18, 0, 27, 158);
        let p = decode_frame(&f).unwrap();
        assert_eq!(p.id, 0x0f3db5);
        assert_eq!(p.moisture_pct, 27);
        assert_eq!(p.battery_mv, 1800);
        assert_eq!(p.battery_level, 1.0);
        assert_eq!(p.ad_raw, 158);
        let mut bad = f.clone();
        bad[6] ^= 1;
        assert_eq!(decode_frame(&bad), Err(FrameError::Integrity));
        assert_eq!(decode_frame(&f[..10]), Err(FrameError::Truncated));
    }

    #[test]
    fn decodes_from_bits_with_preamble() {
        let f = make_frame(0x0f423f, 15, 1, 40, 300);
        let mut bytes = vec![0xaa, 0xaa, 0xaa, 0x2d, 0xd4];
        bytes.extend(&f);
        let bits = crate::bits::to_bits(&bytes);
        let d = Wh51Decoder.decode(&bits);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].raw, f);
        match &d[0].payload {
            Payload::Wh51(p) => assert_eq!((p.id, p.battery_level, p.boost), (0x0f423f, 0.9, 1)),
            _ => panic!(),
        }
    }
}
