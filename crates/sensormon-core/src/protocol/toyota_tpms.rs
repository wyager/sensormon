//! Toyota / Pacific Industries PMV-C210 TPMS: FSK, 52 µs bits (19.23 kbaud),
//! 14-bit sync then 72 bits differential-Manchester data (9 bytes), CRC-8
//! poly 0x07 init 0x80. Ported from rtl_433 `devices/tpms_toyota.c`.

use super::{Band, Decoded, Decoder, FrameError, Modulation};
use crate::bits::{differential_manchester_decode, find_pattern};
use crate::crc::crc8;
use crate::event::{Payload, ToyotaTpms};
use crate::units::Hertz;

pub const SYMBOL_RATE: Hertz = Hertz(1e6 / 52.0);
/// 315 MHz in North America / Japan; 433.92 MHz for European cars.
const TPMS_BANDS: &[Band] = &[Band::mhz(315.0, 2.0), Band::mhz(433.92, 2.0)];
/// 12 bits of the sync (`…1 0101 0011 11` → 0xa9e0); the last bit is handed to the DM decoder.
const SYNC: [u8; 2] = [0xa9, 0xe0];

pub fn decode_frame(b: &[u8]) -> Result<ToyotaTpms, FrameError> {
    if b.len() < 9 {
        return Err(FrameError::Truncated);
    }
    if crc8(&b[..8], 0x07, 0x80) != b[8] {
        return Err(FrameError::Integrity);
    }
    let pressure1 = ((b[4] & 0x7f) << 1) | (b[5] >> 7);
    let pressure2 = b[7] ^ 0xff;
    if pressure1 != pressure2 {
        return Err(FrameError::Integrity);
    }
    let temp = ((b[5] & 0x7f) << 1) | (b[6] >> 7);
    Ok(ToyotaTpms {
        id: u32::from_be_bytes([b[0], b[1], b[2], b[3]]),
        pressure_psi: pressure1 as f32 * 0.25 - 7.0,
        temperature_c: temp as f32 - 40.0,
        status: (b[4] & 0x80) | (b[6] & 0x7f),
    })
}

pub struct ToyotaTpmsDecoder;

impl Decoder for ToyotaTpmsDecoder {
    fn name(&self) -> &'static str {
        "Toyota-TPMS"
    }
    fn modulation(&self) -> Modulation {
        Modulation::Fsk { symbol_rate: SYMBOL_RATE }
    }
    fn bands(&self) -> &'static [Band] {
        TPMS_BANDS
    }
    fn decode(&self, bits: &[bool]) -> Vec<Decoded> {
        let mut out = Vec::new();
        let mut from = 0;
        while let Some(pos) = find_pattern(bits, &SYNC, 12, from) {
            if pos + 156 > bits.len() {
                break;
            }
            let (data, end) = differential_manchester_decode(bits, pos + 11, 80);
            if end - (pos + 11) >= 144 && data.len() >= 72 {
                let raw: Vec<u8> = (0..9).map(|i| (0..8).fold(0u8, |a, k| (a << 1) | data[i * 8 + k] as u8)).collect();
                if let Ok(p) = decode_frame(&raw) {
                    if !out.iter().any(|d: &Decoded| d.raw == raw) {
                        out.push(Decoded { payload: Payload::ToyotaTpms(p), raw, bit_offset: pos, inverted: false });
                    }
                }
            }
            from = pos + 2;
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bits::{differential_manchester_encode, to_bits};

    fn frame(id: u32, pressure_raw: u8, temp_raw: u8, status: u8) -> Vec<u8> {
        let mut b = vec![(id >> 24) as u8, (id >> 16) as u8, (id >> 8) as u8, id as u8];
        b.push((status & 0x80) | (pressure_raw >> 1));
        b.push(((pressure_raw & 1) << 7) | (temp_raw >> 1));
        b.push(((temp_raw & 1) << 7) | (status & 0x7f));
        b.push(!pressure_raw);
        b.push(crc8(&b, 0x07, 0x80));
        b
    }

    #[test]
    fn roundtrip_through_sync_and_differential_manchester() {
        let f = frame(0xd96105f2, 156, 65, 0x00); // 32 psi, 25 C
        let p = decode_frame(&f).unwrap();
        assert_eq!(p.id, 0xd96105f2);
        assert!((p.pressure_psi - 32.0).abs() < 1e-4);
        assert!((p.temperature_c - 25.0).abs() < 1e-4);
        // on-air: sync 0101 0101 0011 11, then DM-coded data starting from the sync's last transition
        let mut raw = to_bits(&[0x55, 0x3c]);
        raw.truncate(14);
        let level = *raw.last().unwrap();
        raw.extend(differential_manchester_encode(&to_bits(&f), level));
        raw.extend([false, true, false]); // trailer
        let d = ToyotaTpmsDecoder.decode(&raw);
        assert_eq!(d.len(), 1, "{raw:?}");
        assert_eq!(d[0].raw, f);
        let mut bad = f.clone();
        bad[2] ^= 1;
        assert_eq!(decode_frame(&bad), Err(FrameError::Integrity));
    }
}
