//! Integrity checks used by the Fineoffset family.

/// CRC-8 with the given polynomial and init, no reflection, no final xor
/// (rtl_433's `crc8`). Fineoffset uses polynomial 0x31, init 0x00.
pub fn crc8(bytes: &[u8], poly: u8, init: u8) -> u8 {
    let mut rem = init;
    for &b in bytes {
        rem ^= b;
        for _ in 0..8 {
            rem = if rem & 0x80 != 0 { (rem << 1) ^ poly } else { rem << 1 };
        }
    }
    rem
}

/// Fineoffset's CRC-8 (poly 0x31, init 0).
pub fn crc8_fineoffset(bytes: &[u8]) -> u8 {
    crc8(bytes, 0x31, 0x00)
}

/// Byte sum modulo 256.
pub fn sum8(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |a, &b| a.wrapping_add(b))
}

/// XOR of all bytes.
pub fn xor8(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |a, &b| a ^ b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc8_known_vector() {
        // CRC-8/NRSC-5 style check value: poly 0x31, init 0xff over "123456789" is 0xF7;
        // with init 0x00 the same message gives 0xA2 (computed with the same algorithm as rtl_433).
        assert_eq!(crc8(b"123456789", 0x31, 0xff), 0xf7);
        assert_eq!(crc8(b"123456789", 0x31, 0x00), 0xa2);
    }

    #[test]
    fn sum_and_xor() {
        assert_eq!(sum8(&[0xff, 0x02]), 0x01);
        assert_eq!(xor8(&[0xf0, 0x0f, 0x01]), 0xfe);
    }
}
