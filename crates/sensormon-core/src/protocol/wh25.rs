//! Fineoffset WH25 / WH32 / WH32B indoor sensor: 8 bytes after sync, type nibble
//! 0xE (or 0xD for the pressure-less WH32), sum over 6 bytes, xor-nibble-swap
//! check for the WH25.

use super::{fineoffset, Decoded, Decoder, FrameError, Modulation};
use crate::crc::{sum8, xor8};
use crate::event::{Payload, Wh25, Wh25Variant};

pub const FRAME_LEN: usize = 8;

/// `burst_bits` selects the variant the way rtl_433 does (by transmission length).
pub fn decode_frame(b: &[u8], burst_bits: usize) -> Result<Wh25, FrameError> {
    if b.len() < FRAME_LEN {
        return Err(FrameError::Truncated);
    }
    let mut variant = if (160..190).contains(&burst_bits) || burst_bits > 510 { Wh25Variant::Wh32b } else { Wh25Variant::Wh25 };
    match b[0] & 0xf0 {
        0xd0 if variant == Wh25Variant::Wh32b => variant = Wh25Variant::Wh32,
        0xe0 => {}
        _ => return Err(FrameError::WrongFamily),
    }
    if sum8(&b[..6]) != b[6] {
        return Err(FrameError::Integrity);
    }
    let x = xor8(&b[..6]);
    let x = x.rotate_right(4);
    if variant == Wh25Variant::Wh25 && x != b[7] {
        return Err(FrameError::Integrity);
    }
    let temp_raw = ((b[1] as u16 & 0x03) << 8) | b[2] as u16;
    let pressure_raw = u16::from_be_bytes([b[4], b[5]]);
    Ok(Wh25 {
        id: ((b[0] & 0x0f) << 4) | (b[1] >> 4),
        variant,
        battery_ok: b[1] & 0x08 == 0,
        temperature_c: (temp_raw != 0x7ff).then_some((temp_raw as f32 - 400.0) * 0.1),
        humidity_pct: b[3],
        pressure_hpa: (pressure_raw != 0xffff).then_some(pressure_raw as f32 * 0.1),
    })
}

pub struct Wh25Decoder;

impl Decoder for Wh25Decoder {
    fn name(&self) -> &'static str {
        "Fineoffset-WH25"
    }
    fn modulation(&self) -> Modulation {
        Modulation::Fsk { symbol_rate: fineoffset::SYMBOL_RATE }
    }
    fn decode(&self, bits: &[bool]) -> Vec<Decoded> {
        fineoffset::frame_starts(bits)
            .into_iter()
            .filter_map(|start| {
                let raw = fineoffset::frame_bytes(bits, start, FRAME_LEN)?;
                let p = decode_frame(&raw, bits.len()).ok()?;
                Some(Decoded { payload: Payload::Wh25(p), raw, bit_offset: start, inverted: false })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_wh25() {
        // id 0x5a: b[0]=0xe5, b[1] high nibble 0xa; temp raw 0x1f4=500 → 10.0 C; hum 55; pressure 10132 → 1013.2
        let mut b = vec![0xe5, 0xa1, 0xf4, 55, 0x27, 0x94];
        b.push(sum8(&b));
        let x = xor8(&b[..6]);
        b.push(x.rotate_right(4));
        let p = decode_frame(&b, 480).unwrap();
        assert_eq!(p.id, 0x5a);
        assert!((p.temperature_c.unwrap() - 10.0).abs() < 1e-4);
        assert_eq!(p.humidity_pct, 55);
        assert!((p.pressure_hpa.unwrap() - 1013.2).abs() < 1e-3);
        assert!(p.battery_ok);
        assert_eq!(p.variant, Wh25Variant::Wh25);
    }
}
