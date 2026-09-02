//! Replay a recorded IQ file as a source (real-time paced or as fast as possible).

use crate::iqfile::{Format, Reader};
use crate::source::{BlockSink, IqSource, RunningSource};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub struct FileSource {
    pub path: PathBuf,
    pub format: Format,
    pub sample_rate: u32,
    pub realtime: bool,
    pub block: usize,
    pub center_hz: f64,
}

struct Running {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl RunningSource for Running {
    fn stop(mut self: Box<Self>) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl IqSource for FileSource {
    fn describe(&self) -> String {
        format!("file {} @ {} S/s realtime={}", self.path.display(), self.sample_rate, self.realtime)
    }

    fn start(self: Box<Self>, mut sink: BlockSink) -> Result<Box<dyn RunningSource>> {
        let mut reader = Reader::open(&self.path, self.format)?;
        sink.center_handle().store(self.center_hz.to_bits(), std::sync::atomic::Ordering::Relaxed);
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let thread = std::thread::Builder::new().name("file-source".into()).spawn(move || {
            let t0 = Instant::now();
            let mut sent: u64 = 0;
            let mut buf = Vec::new();
            loop {
                if stop2.load(Ordering::Relaxed) {
                    break;
                }
                let n = match reader.read_block(&mut buf, self.block) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                sent += n as u64;
                if self.realtime {
                    let due = t0 + Duration::from_secs_f64(sent as f64 / self.sample_rate as f64);
                    if let Some(d) = due.checked_duration_since(Instant::now()) {
                        std::thread::sleep(d);
                    }
                }
                sink.push(std::mem::take(&mut buf), 0);
            }
        })?;
        Ok(Box::new(Running { stop, thread: Some(thread) }))
    }
}
