//! Sample sources. Each delivers contiguous blocks of `Iq` to a bounded
//! channel from its own thread (the SDR library's callback thread, or a file
//! pacer). A full channel drops the block and counts it rather than stalling
//! the USB callback.

use anyhow::Result;
use sensormon_core::dsp::Iq;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{SyncSender, TrySendError};
use std::sync::Arc;

/// A block of samples plus how many samples the source knows it lost just
/// before it (USB overruns, dropped blocks).
pub struct Block {
    pub samples: Vec<Iq>,
    pub dropped_before: u64,
}

/// Shared counters a source updates from its callback.
#[derive(Default)]
pub struct SourceCounters {
    pub blocks: AtomicU64,
    pub samples: AtomicU64,
    pub dropped_blocks: AtomicU64,
    pub device_dropped_samples: AtomicU64,
}

/// Non-blocking hand-off used by every source's callback.
pub struct BlockSink {
    tx: SyncSender<Block>,
    counters: Arc<SourceCounters>,
    pending_drop: u64,
}

impl BlockSink {
    pub fn new(tx: SyncSender<Block>, counters: Arc<SourceCounters>) -> Self {
        BlockSink { tx, counters, pending_drop: 0 }
    }

    pub fn push(&mut self, samples: Vec<Iq>, device_dropped: u64) {
        let n = samples.len() as u64;
        self.counters.device_dropped_samples.fetch_add(device_dropped, Ordering::Relaxed);
        let block = Block { samples, dropped_before: self.pending_drop + device_dropped };
        match self.tx.try_send(block) {
            Ok(()) => {
                self.pending_drop = 0;
                self.counters.blocks.fetch_add(1, Ordering::Relaxed);
                self.counters.samples.fetch_add(n, Ordering::Relaxed);
            }
            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
                self.pending_drop += n;
                self.counters.dropped_blocks.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// A running source; dropping it stops streaming.
pub trait RunningSource: Send {
    fn stop(self: Box<Self>);
}

/// Something that can start streaming into a sink.
pub trait IqSource {
    fn describe(&self) -> String;
    fn start(self: Box<Self>, sink: BlockSink) -> Result<Box<dyn RunningSource>>;
}

/// int16 interleaved I/Q → Iq.
pub fn cs16_to_iq(raw: &[i16]) -> Vec<Iq> {
    raw.chunks_exact(2).map(|c| Iq::new(c[0] as f32 / 32768.0, c[1] as f32 / 32768.0)).collect()
}

/// uint8 interleaved I/Q → Iq.
pub fn cu8_to_iq(raw: &[u8]) -> Vec<Iq> {
    raw.chunks_exact(2).map(|c| Iq::new((c[0] as f32 - 127.5) / 127.5, (c[1] as f32 - 127.5) / 127.5)).collect()
}
