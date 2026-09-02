//! Fixed-capacity history of raw samples, addressed by absolute `SampleIndex`,
//! so a burst detected a few milliseconds after it ended can still be cut out.

use super::Iq;
use crate::units::SampleIndex;

pub struct SampleRing {
    buf: Vec<Iq>,
    /// Absolute index of the next sample to be written.
    next: SampleIndex,
    /// Number of valid samples (≤ capacity), for the startup phase.
    filled: usize,
}

impl SampleRing {
    pub fn new(capacity: usize, first_index: SampleIndex) -> Self {
        SampleRing { buf: vec![Iq::new(0.0, 0.0); capacity.max(1)], next: first_index, filled: 0 }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// Absolute index of the next sample that will be written.
    pub fn next_index(&self) -> SampleIndex {
        self.next
    }

    /// Oldest index still available.
    pub fn oldest_index(&self) -> SampleIndex {
        SampleIndex(self.next.0 - self.filled as u64)
    }

    /// Append a block; must be contiguous with what was pushed before.
    pub fn push(&mut self, block: &[Iq]) {
        let cap = self.buf.len();
        for &s in block {
            self.buf[(self.next.0 as usize) % cap] = s;
            self.next.0 += 1;
        }
        self.filled = (self.filled + block.len()).min(cap);
    }

    /// Copy out `[start, end)` if the whole range is still available.
    pub fn get(&self, start: SampleIndex, end: SampleIndex) -> Option<Vec<Iq>> {
        if end.0 <= start.0 || start < self.oldest_index() || end > self.next {
            return None;
        }
        let cap = self.buf.len();
        Some((start.0..end.0).map(|i| self.buf[(i as usize) % cap]).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_and_evicts() {
        let mut r = SampleRing::new(8, SampleIndex(100));
        let blk: Vec<Iq> = (0..10).map(|i| Iq::new(i as f32, 0.0)).collect();
        r.push(&blk[..5]);
        r.push(&blk[5..]);
        assert_eq!(r.next_index(), SampleIndex(110));
        assert_eq!(r.oldest_index(), SampleIndex(102));
        assert!(r.get(SampleIndex(100), SampleIndex(104)).is_none());
        let got = r.get(SampleIndex(102), SampleIndex(110)).unwrap();
        assert_eq!(got.iter().map(|s| s.re as i32).collect::<Vec<_>>(), (2..10).collect::<Vec<_>>());
    }
}
