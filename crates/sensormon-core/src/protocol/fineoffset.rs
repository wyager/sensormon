//! Fineoffset/Ecowitt framing shared by all their FSK sensors: 0xAA… preamble,
//! 0x2D 0xD4 sync word, then a family byte and payload, with CRC-8 (0x31) and
//! a byte sum. Bit rate 17.24 kbps.

use crate::bits;
use crate::units::Hertz;

pub const SYMBOL_RATE: Hertz = Hertz(17_240.0);
/// Last preamble byte plus the sync word; searched at any bit alignment.
pub const PREAMBLE_SYNC: [u8; 3] = [0xaa, 0x2d, 0xd4];

/// Bit offsets just past each sync word found in `bits`. Later sync words
/// may belong to repeats of the same frame or to a second sensor's frame.
pub fn frame_starts(bits: &[bool]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(off) = bits::find_pattern(bits, &PREAMBLE_SYNC, 24, from) {
        out.push(off + 24);
        from = off + 1;
    }
    out
}

/// The `n` bytes starting at a frame start, if the burst holds that many.
pub fn frame_bytes(bits: &[bool], start: usize, n: usize) -> Option<Vec<u8>> {
    bits::extract_bytes(bits, start, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_sync_at_any_alignment() {
        let mut bits = vec![true, false, true, false, true];
        bits.extend(bits::to_bits(&[0xaa, 0xaa, 0x2d, 0xd4, 0x51, 0x00]));
        let starts = frame_starts(&bits);
        assert_eq!(starts.len(), 1);
        assert_eq!(frame_bytes(&bits, starts[0], 1).unwrap(), [0x51]);
    }
}
