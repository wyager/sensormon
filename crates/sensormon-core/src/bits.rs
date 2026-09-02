//! Bit-string helpers for framing. Bits are MSB-first within bytes, matching
//! how FSK symbols arrive on the air.

/// Search `haystack` for `pattern` (given as bytes, `pattern_bits` long, MSB
/// first) starting at bit offset `from`. Returns the bit offset of the match.
pub fn find_pattern(haystack: &[bool], pattern: &[u8], pattern_bits: usize, from: usize) -> Option<usize> {
    if pattern_bits == 0 || haystack.len() < pattern_bits {
        return None;
    }
    let pat: Vec<bool> = (0..pattern_bits).map(|i| bit_at(pattern, i)).collect();
    (from..=haystack.len() - pattern_bits).find(|&off| haystack[off..off + pattern_bits] == pat[..])
}

/// Bit `i` (MSB-first) of a byte slice.
pub fn bit_at(bytes: &[u8], i: usize) -> bool {
    (bytes[i / 8] >> (7 - (i % 8))) & 1 == 1
}

/// Pack `n_bytes` bytes starting at bit offset `from`, MSB first. Returns
/// `None` if not enough bits remain.
pub fn extract_bytes(bits: &[bool], from: usize, n_bytes: usize) -> Option<Vec<u8>> {
    if from + n_bytes * 8 > bits.len() {
        return None;
    }
    Some(
        (0..n_bytes)
            .map(|b| (0..8).fold(0u8, |acc, k| (acc << 1) | bits[from + b * 8 + k] as u8))
            .collect(),
    )
}

/// Unpack bytes to MSB-first bits (test helper and inverse of `extract_bytes`).
pub fn to_bits(bytes: &[u8]) -> Vec<bool> {
    (0..bytes.len() * 8).map(|i| bit_at(bytes, i)).collect()
}

/// Flip every bit (FSK tone polarity is not knowable a priori).
pub fn inverted(bits: &[bool]) -> Vec<bool> {
    bits.iter().map(|b| !b).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_search() {
        let bytes = [0xaa, 0x2d, 0xd4, 0x51, 0x0f];
        let bits = to_bits(&bytes);
        assert_eq!(extract_bytes(&bits, 0, 5).unwrap(), bytes);
        assert_eq!(find_pattern(&bits, &[0x2d, 0xd4], 16, 0), Some(8));
        // unaligned: shift by 3 bits
        let mut shifted = vec![false, true, true];
        shifted.extend(&bits);
        assert_eq!(find_pattern(&shifted, &[0x2d, 0xd4], 16, 0), Some(11));
        assert_eq!(extract_bytes(&shifted, 11, 3).unwrap(), [0x2d, 0xd4, 0x51]);
        assert_eq!(extract_bytes(&bits, 30, 2), None);
    }
}
