//! Fineoffset WH57 lightning sensor (rtl_433: FineOffset-WH31L): 9 bytes from
//! family byte 0x57, CRC over 8 == 0, byte 8 = sum of first 8.

use super::{fineoffset, Decoded, Decoder, FrameError, Modulation};
use crate::crc::{crc8_fineoffset, sum8};
use crate::event::{LightningState, Payload, Wh31l};

pub const FRAME_LEN: usize = 9;

pub fn decode_frame(b: &[u8]) -> Result<Wh31l, FrameError> {
    if b.len() < FRAME_LEN {
        return Err(FrameError::Truncated);
    }
    if b[0] != 0x57 {
        return Err(FrameError::WrongFamily);
    }
    if crc8_fineoffset(&b[..8]) != 0 || sum8(&b[..8]) != b[8] {
        return Err(FrameError::Integrity);
    }
    let state = b[1] >> 4;
    let dist = b[5] & 0x3f;
    Ok(Wh31l {
        id: (((b[1] & 0x0f) as u32) << 16) | ((b[2] as u32) << 8) | b[3] as u32,
        battery_level: ((b[4] & 0x06) >> 1) as f32 * 0.5,
        state: match state {
            0 => LightningState::Reset,
            1 => LightningState::Interference,
            4 => LightningState::Noise,
            8 => LightningState::Strike,
            s => LightningState::Unknown(s),
        },
        storm_dist_km: (dist != 63).then_some(dist),
        strike_count: b[6],
        flags: ((state as u16) << 12) | ((b[4] as u16) << 4) | (b[5] >> 4) as u16,
    })
}

pub struct Wh31lDecoder;

impl Decoder for Wh31lDecoder {
    fn name(&self) -> &'static str {
        "FineOffset-WH31L"
    }
    fn modulation(&self) -> Modulation {
        Modulation::Fsk { symbol_rate: fineoffset::SYMBOL_RATE }
    }
    fn decode(&self, bits: &[bool]) -> Vec<Decoded> {
        fineoffset::frame_starts(bits)
            .into_iter()
            .filter_map(|start| {
                let raw = fineoffset::frame_bytes(bits, start, FRAME_LEN)?;
                let p = decode_frame(&raw).ok()?;
                Some(Decoded { payload: Payload::Wh31l(p), raw, bit_offset: start, inverted: false })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        // state=4 (noise), id 0xd123 → b[1]=0x40 | 0x0, b[2..4]
        let mut b = vec![0x57, 0x40, 0xd1, 0x23, 0x02, 0xbf, 20];
        b.push(crc8_fineoffset(&b));
        b.push(sum8(&b));
        let p = decode_frame(&b).unwrap();
        assert_eq!(p.id, 0x00d123);
        assert_eq!(p.state, LightningState::Noise);
        assert_eq!(p.battery_level, 0.5);
        assert_eq!(p.storm_dist_km, None);
        assert_eq!(p.strike_count, 20);
    }
}
