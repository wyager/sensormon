//! Fineoffset WH46 / WH46D 7-in-1 indoor air quality sensor (NDIR CO₂,
//! Sensirion SPS30 particulates, temperature, humidity): 21 bytes after sync,
//! family 0x46, CRC over the first 19 bytes in byte 19, sum of the first 20 in
//! byte 20. Layout ported from rtl_433's `src/devices/fineoffset_wh46.c`:
//!
//! ```text
//!  0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18 19 20
//! YY II II II 0T TT HH Bp pp BP PP CC CC qq qq QQ QQ ?? ?? XX AA
//! ```
//!
//! T: 11-bit temperature (offset 40, ×0.1 °C); H: humidity; B: battery bars
//! (1 MSB in byte 7, 2 LSBs in byte 9 — 6 means USB power); p/P/q/Q: 14-bit
//! PM2.5 / PM10 / PM1 / PM4 in 0.1 µg/m³; C: CO₂ in ppm; ??: constant 0x0190.
//!
//! For the first transmission(s) after power-on the sensor sends all-ones
//! for CO₂ (0xFFFF) and every PM channel (0x3FFF = 1638.3) while the SCD30
//! and SPS30 warm up; those decode as `None`.

use super::{fineoffset, Band, Decoded, Decoder, FrameError, Modulation, FINEOFFSET_BANDS};
use crate::crc::{crc8_fineoffset, sum8};
use crate::event::{Payload, Wh46};

pub const FRAME_LEN: usize = 21;

/// The battery-bars value the sensor sends while on USB power.
pub const EXTERNAL_POWER_BARS: u8 = 6;
/// Raw CO₂ value meaning "not available" (sensor warming up).
pub const CO2_UNAVAILABLE: u16 = 0xffff;
/// Raw 14-bit PM value meaning "not available" (sensor warming up).
pub const PM_UNAVAILABLE: u16 = 0x3fff;

pub fn decode_frame(b: &[u8]) -> Result<Wh46, FrameError> {
    if b.len() < FRAME_LEN {
        return Err(FrameError::Truncated);
    }
    if b[0] != 0x46 {
        return Err(FrameError::WrongFamily);
    }
    if crc8_fineoffset(&b[..19]) != b[19] || sum8(&b[..20]) != b[20] {
        return Err(FrameError::Integrity);
    }
    let temp_raw = ((b[4] as u16 & 0x07) << 8) | b[5] as u16;
    let battery_bars = ((b[7] & 0x40) >> 4) | ((b[9] & 0xc0) >> 6);
    let pm = |hi: u8, lo: u8| {
        let raw = ((hi as u16 & 0x3f) << 8) | lo as u16;
        (raw != PM_UNAVAILABLE).then_some(raw as f32 * 0.1)
    };
    let co2 = u16::from_be_bytes([b[11], b[12]]);
    Ok(Wh46 {
        id: u32::from_be_bytes([0, b[1], b[2], b[3]]),
        battery_bars,
        external_power: battery_bars == EXTERNAL_POWER_BARS,
        battery_level: battery_bars.min(5) as f32 * 0.2,
        temperature_c: (temp_raw as f32 - 400.0) * 0.1,
        humidity_pct: b[6],
        co2_ppm: (co2 != CO2_UNAVAILABLE).then_some(co2),
        pm1_ug_m3: pm(b[13], b[14]),
        pm2_5_ug_m3: pm(b[7], b[8]),
        pm4_ug_m3: pm(b[15], b[16]),
        pm10_ug_m3: pm(b[9], b[10]),
        unknown: u16::from_be_bytes([b[17], b[18]]),
    })
}

pub struct Wh46Decoder;

impl Decoder for Wh46Decoder {
    fn name(&self) -> &'static str {
        "Fineoffset-WH46"
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
                Some(Decoded { payload: Payload::Wh46(p), raw, bit_offset: start, inverted: false })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example frame from rtl_433's `fineoffset_wh46.c` header, integrity
    /// bytes included — so this also pins down our CRC/sum coverage.
    const RTL433_SAMPLE: [u8; 21] = [
        0x46, 0x00, 0x27, 0xf1, 0x02, 0xb5, 0x33, 0x40, 0x32, 0x40, 0x39, 0x03, 0x0b, 0x00, 0x2a, 0x00, 0x36, 0x01, 0x90, 0xe4, 0x16,
    ];

    #[test]
    fn decodes_rtl433_sample_frame() {
        let p = decode_frame(&RTL433_SAMPLE).unwrap();
        assert_eq!(p.id, 0x0027f1);
        assert!((p.temperature_c - 29.3).abs() < 1e-4);
        assert_eq!(p.humidity_pct, 51);
        assert_eq!(p.battery_bars, 5);
        assert!(!p.external_power);
        assert!((p.battery_level - 1.0).abs() < 1e-6);
        assert!((p.pm2_5_ug_m3.unwrap() - 5.0).abs() < 1e-4);
        assert!((p.pm10_ug_m3.unwrap() - 5.7).abs() < 1e-4);
        assert_eq!(p.co2_ppm, Some(779));
        assert!((p.pm1_ug_m3.unwrap() - 4.2).abs() < 1e-4);
        assert!((p.pm4_ug_m3.unwrap() - 5.4).abs() < 1e-4);
        assert_eq!(p.unknown, 0x0190);
        let mut bad = RTL433_SAMPLE;
        bad[12] ^= 1;
        assert_eq!(decode_frame(&bad), Err(FrameError::Integrity));
        assert_eq!(decode_frame(&RTL433_SAMPLE[..20]), Err(FrameError::Truncated));
        let mut other = RTL433_SAMPLE;
        other[0] = 0x45;
        assert_eq!(decode_frame(&other), Err(FrameError::WrongFamily));
    }

    /// Build a valid frame from fields, the way the sensor firmware would.
    fn make_frame(id: u32, bars: u8, temp_c: f32, hum: u8, co2: u16, pm: [f32; 4]) -> Vec<u8> {
        let temp_raw = (temp_c * 10.0 + 400.0).round() as u16;
        let raw = |v: f32| (v * 10.0).round() as u16;
        let [pm1, pm25, pm4, pm10] = pm.map(raw);
        let mut b = vec![
            0x46, (id >> 16) as u8, (id >> 8) as u8, id as u8,
            (temp_raw >> 8) as u8 & 0x07, temp_raw as u8,
            hum,
            ((bars & 0x04) << 4) | (pm25 >> 8) as u8, pm25 as u8,
            ((bars & 0x03) << 6) | (pm10 >> 8) as u8, pm10 as u8,
            (co2 >> 8) as u8, co2 as u8,
            (pm1 >> 8) as u8, pm1 as u8,
            (pm4 >> 8) as u8, pm4 as u8,
            0x01, 0x90,
        ];
        b.push(crc8_fineoffset(&b));
        b.push(sum8(&b));
        b
    }

    #[test]
    fn external_power_and_negative_temperature() {
        let f = make_frame(0x0038e9, 6, -5.0, 30, 1450, [1.0, 2.5, 40.0, 50.0]);
        let p = decode_frame(&f).unwrap();
        assert_eq!(p.id, 0x0038e9);
        assert!(p.external_power);
        assert_eq!(p.battery_bars, 6);
        assert!((p.battery_level - 1.0).abs() < 1e-6);
        assert!((p.temperature_c + 5.0).abs() < 1e-4);
        assert_eq!(p.co2_ppm, Some(1450));
        assert!((p.pm4_ug_m3.unwrap() - 40.0).abs() < 1e-4);
        assert!((p.pm10_ug_m3.unwrap() - 50.0).abs() < 1e-4);
        let low = decode_frame(&make_frame(1, 1, 20.0, 50, 400, [0.0; 4])).unwrap();
        assert_eq!(low.battery_bars, 1);
        assert!((low.battery_level - 0.2).abs() < 1e-6);
    }

    #[test]
    fn decodes_from_bits_with_preamble() {
        let mut bytes = vec![0xaa, 0xaa, 0xaa, 0x2d, 0xd4];
        bytes.extend(&RTL433_SAMPLE);
        let bits = crate::bits::to_bits(&bytes);
        let d = Wh46Decoder.decode(&bits);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].raw, RTL433_SAMPLE);
        match &d[0].payload {
            Payload::Wh46(p) => assert_eq!((p.id, p.co2_ppm), (0x0027f1, Some(779))),
            _ => panic!(),
        }
    }

    /// A frame heard off the air from the WH46D (id 0038e9) on 2026-09-20,
    /// via the tower Airspy: 71.2 °F / 54 %, CO₂ 496 ppm, USB power.
    const LIVE_FRAME: [u8; 21] = [
        70, 0, 56, 233, 98, 106, 54, 64, 51, 128, 54, 1, 240, 0, 46, 0, 53, 1, 144, 219, 82,
    ];

    #[test]
    fn decodes_live_frame() {
        let p = decode_frame(&LIVE_FRAME).unwrap();
        assert_eq!(p.id, 0x0038e9);
        assert!(p.external_power);
        assert!((p.temperature_c - 21.8).abs() < 1e-4);
        assert_eq!(p.humidity_pct, 54);
        assert_eq!(p.co2_ppm, Some(496));
        assert!((p.pm1_ug_m3.unwrap() - 4.6).abs() < 1e-4);
        assert!((p.pm2_5_ug_m3.unwrap() - 5.1).abs() < 1e-4);
        assert!((p.pm4_ug_m3.unwrap() - 5.3).abs() < 1e-4);
        assert!((p.pm10_ug_m3.unwrap() - 5.4).abs() < 1e-4);
    }

    /// The first frame after power-on: CO₂ 0xFFFF and every PM channel
    /// 0x3FFF (the values that showed up as 65535 ppm / 1638.3 µg/m³ before
    /// they were treated as "not available"). Temperature and humidity are
    /// valid in that frame.
    #[test]
    fn warmup_frame_has_no_co2_or_pm() {
        let mut b = LIVE_FRAME;
        b[7] = 0x40 | 0x3f; // battery MSB kept, PM2.5 all ones
        b[8] = 0xff;
        b[9] = 0x80 | 0x3f; // battery LSBs kept, PM10 all ones
        b[10] = 0xff;
        b[11] = 0xff; // CO₂
        b[12] = 0xff;
        b[13] = 0x3f; // PM1
        b[14] = 0xff;
        b[15] = 0x3f; // PM4
        b[16] = 0xff;
        b[19] = crc8_fineoffset(&b[..19]);
        b[20] = sum8(&b[..20]);
        let p = decode_frame(&b).unwrap();
        assert_eq!(p.co2_ppm, None);
        assert_eq!((p.pm1_ug_m3, p.pm2_5_ug_m3, p.pm4_ug_m3, p.pm10_ug_m3), (None, None, None, None));
        assert!(p.external_power);
        assert_eq!(p.battery_bars, 6);
        assert!((p.temperature_c - 21.8).abs() < 1e-4);
        assert_eq!(p.humidity_pct, 54);
    }
}
