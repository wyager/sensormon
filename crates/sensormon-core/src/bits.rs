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

/// Differential Manchester decode starting at bit `start`, at most `max_bits`
/// output bits. Exact port of rtl_433's `bitbuffer_differential_manchester_decode`:
/// the first long pulse sets the clock; thereafter each data bit is two raw
/// bits, equal raw bits = 1, differing = 0; a missing clock transition ends it.
/// Returns (decoded bits, raw position reached).
pub fn differential_manchester_decode(bits: &[bool], start: usize, max_bits: usize) -> (Vec<bool>, usize) {
    let len = bits.len().min(if max_bits > 0 { start + max_bits * 2 } else { usize::MAX });
    let mut out = Vec::with_capacity(max_bits);
    let mut ipos = start;
    let mut bit2 = false;
    while ipos < len {
        let bit1 = bits[ipos];
        ipos += 1;
        bit2 = *bits.get(ipos).unwrap_or(&false);
        ipos += 1;
        let bit3 = *bits.get(ipos).unwrap_or(&false);
        if bit1 != bit2 {
            if bit2 != bit3 {
                out.push(false);
            } else {
                bit2 = bit1;
                ipos -= 1;
                break;
            }
        } else {
            bit2 = !bit1;
            ipos -= 2;
            break;
        }
    }
    while ipos < len {
        let bit1 = bits[ipos];
        ipos += 1;
        if bit1 == bit2 {
            break; // clock missing
        }
        bit2 = *bits.get(ipos).unwrap_or(&false);
        ipos += 1;
        out.push(bit1 == bit2);
    }
    (out, ipos)
}

/// Differential Manchester encode (test helper / inverse of the decoder):
/// every bit period starts with a transition; a `0` has a mid-bit transition, a `1` none.
pub fn differential_manchester_encode(data: &[bool], mut level: bool) -> Vec<bool> {
    let mut out = Vec::with_capacity(data.len() * 2);
    for &d in data {
        level = !level; // clock transition at bit start
        out.push(level);
        if !d {
            level = !level; // mid-bit transition for 0
        }
        out.push(level);
    }
    out
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

    #[test]
    fn differential_manchester_roundtrip() {
        let data = to_bits(&[0xd9, 0x61, 0x05, 0xf2, 0x3a, 0x5c, 0x00, 0xc5, 0x77]);
        let raw = differential_manchester_encode(&data, false);
        let (dec, _) = differential_manchester_decode(&raw, 0, 80);
        assert_eq!(dec, data);
    }
}
