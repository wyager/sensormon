//! Fineoffset WS90: 32 bytes after sync, family 0x90, CRC over 31 then sum.

use super::{fineoffset, Decoded, Decoder, FrameError, Modulation};
use crate::crc::{crc8_fineoffset, sum8};
use crate::event::{Payload, Ws90};

pub const FRAME_LEN: usize = 32;

pub fn decode_frame(b: &[u8]) -> Result<Ws90, FrameError> {
    if b.len() < FRAME_LEN {
        return Err(FrameError::Truncated);
    }
    if b[0] != 0x90 {
        return Err(FrameError::WrongFamily);
    }
    // CRC over the first 31 bytes (payload + CRC byte) must be 0; byte 31 is the sum of the first 31.
    if crc8_fineoffset(&b[..31]) != 0 || sum8(&b[..31]) != b[31] {
        return Err(FrameError::Integrity);
    }
    let light_raw = u16::from_be_bytes([b[4], b[5]]);
    let battery_mv = b[6] as u16 * 20;
    let battery_level = if battery_mv < 1400 { 0 } else { ((battery_mv - 1400) / 16).min(100) };
    let flags = b[7];
    let temp_raw = ((flags as u16 & 0x03) << 8) | b[8] as u16;
    let wind_avg = ((flags as u16 & 0x10) << 4) | b[10] as u16;
    let wind_dir = ((flags as u16 & 0x20) << 3) | b[11] as u16;
    let wind_max = ((flags as u16 & 0x40) << 2) | b[12] as u16;
    let pressure = u16::from_be_bytes([b[14], b[15]]);
    let supercap = b[21] & 0x3f;
    Ok(Ws90 {
        id: u32::from_be_bytes([0, b[1], b[2], b[3]]),
        battery_mv,
        battery_level: battery_level as f32 / 100.0,
        temperature_c: (temp_raw != 0x3ff).then(|| (temp_raw as f32 - 400.0) * 0.1),
        humidity_pct: (b[9] != 0xff).then_some(b[9]),
        pressure_hpa: (pressure != 0x3fff).then(|| pressure as f32),
        wind_dir_deg: (wind_dir != 0x1ff).then_some(wind_dir),
        wind_avg_m_s: (wind_avg != 0x1ff).then(|| wind_avg as f32 * 0.1),
        wind_max_m_s: (wind_max != 0x1ff).then(|| wind_max as f32 * 0.1),
        uv_index: (b[13] != 0xff).then(|| b[13] as f32 * 0.1),
        light_lux: (light_raw != 0xffff).then(|| light_raw as f32 * 10.0),
        rain_total_mm: u16::from_be_bytes([b[19], b[20]]) as f32 * 0.1,
        rain_start: b[16] & 0x10 != 0,
        supercap_v: (supercap != 0x3f).then(|| supercap as f32 * 0.1),
        firmware: b[29],
        flags,
    })
}

pub struct Ws90Decoder;

impl Decoder for Ws90Decoder {
    fn name(&self) -> &'static str {
        "Fineoffset-WS90"
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
                Some(Decoded { payload: Payload::Ws90(p), raw, bit_offset: start, inverted: false })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(fill: impl Fn(&mut [u8; 32])) -> Vec<u8> {
        let mut b = [0u8; 32];
        b[0] = 0x90;
        fill(&mut b);
        // CRC byte 30 so that crc(b[..31]) == 0, then sum byte 31.
        b[30] = crc8_fineoffset(&b[..30]);
        b[31] = sum8(&b[..31]);
        b.to_vec()
    }

    #[test]
    fn roundtrip_and_invalid_markers() {
        let f = make_frame(|b| {
            b[1..4].copy_from_slice(&[0x01, 0x0b, 0x9b]); // id 68507
            b[4..6].copy_from_slice(&[0x00, 0x2a]); // light 42*10 lux
            b[6] = 123; // 2460 mV
            b[7] = 0x01; b[8] = 0xe6; // temp raw 0x1e6 = 486 → 8.6 C
            b[9] = 47;
            b[10] = 0; b[11] = 0xd9; b[12] = 3; // wind avg 0, dir 217, gust 0.3
            b[13] = 0xff; // uv invalid
            b[14] = 0x3f; b[15] = 0xff; // pressure invalid
            b[19] = 0x0b; b[20] = 0x0a; // rain 2826 → 282.6 mm
            b[21] = 52; // supercap 5.2 V
            b[29] = 159;
        });
        let p = decode_frame(&f).unwrap();
        assert_eq!(p.id, 68507);
        assert_eq!(p.battery_mv, 2460);
        assert_eq!(p.battery_level, 0.66);
        assert!((p.temperature_c.unwrap() - 8.6).abs() < 1e-4);
        assert_eq!(p.humidity_pct, Some(47));
        assert_eq!(p.wind_dir_deg, Some(217));
        assert_eq!(p.uv_index, None);
        assert_eq!(p.pressure_hpa, None);
        assert!((p.rain_total_mm - 282.6).abs() < 1e-3);
        assert!((p.supercap_v.unwrap() - 5.2).abs() < 1e-4);
        assert_eq!(p.firmware, 159);
        let mut bad = f.clone();
        bad[9] ^= 0x10;
        assert_eq!(decode_frame(&bad), Err(FrameError::Integrity));
    }
}
