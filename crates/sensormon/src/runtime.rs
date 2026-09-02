//! Threads and channels: one pipeline thread per receiver, one merger thread,
//! events fanned out to HTTP subscribers via a broadcast channel.

use crate::config::{AirspyGain, Config, FileFormat, ReceiverConfig, ReceiverKind, RtlsdrGain};
use crate::filesource::FileSource;
use crate::iqfile::Format;
use crate::sdr::{airspy::AirspySource, rtlsdr::RtlsdrSource};
use crate::source::{Block, BlockSink, IqSource, RunningSource, SourceCounters};
use anyhow::{Context, Result};
use chrono::Utc;
use sensormon_core::merge::Merger;
use sensormon_core::pipeline::{Pipeline, PipelineConfig, PipelineStats};
use sensormon_core::{Event, Hertz, ReceiverEvent, ReceiverId, SampleIndex, SampleRate};
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;

/// Snapshot of one receiver's health, for `/stats`.
#[derive(Clone, Debug, Serialize)]
pub struct ReceiverStatus {
    pub name: String,
    pub source: String,
    pub sample_rate: u32,
    pub center_hz: f64,
    pub blocks: u64,
    pub samples: u64,
    pub dropped_blocks: u64,
    pub device_dropped_samples: u64,
    pub pipeline: PipelineStatsJson,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct PipelineStatsJson {
    pub bursts: u64,
    pub demodulated: u64,
    pub decoded_frames: u64,
    pub bursts_lost_from_ring: u64,
}

impl From<PipelineStats> for PipelineStatsJson {
    fn from(s: PipelineStats) -> Self {
        PipelineStatsJson { bursts: s.bursts, demodulated: s.demodulated, decoded_frames: s.decoded_frames, bursts_lost_from_ring: s.bursts_lost_from_ring }
    }
}

struct ReceiverHandle {
    name: String,
    source_desc: String,
    sample_rate: u32,
    center_hz: f64,
    counters: Arc<SourceCounters>,
    stats: Arc<Mutex<PipelineStats>>,
    running: Option<Box<dyn RunningSource>>,
}

pub struct Runtime {
    receivers: Vec<ReceiverHandle>,
    pub events: broadcast::Sender<Arc<Event>>,
}

fn make_source(rc: &ReceiverConfig) -> Result<Box<dyn IqSource>> {
    Ok(match &rc.kind {
        ReceiverKind::Airspy { serial, gain, bias_tee } => {
            let serial = serial.as_ref().map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16)).transpose().context("airspy serial must be hex")?;
            Box::new(AirspySource { serial, center_hz: rc.center_hz as u32, sample_rate: rc.sample_rate, gain: *gain, bias_tee: *bias_tee })
        }
        ReceiverKind::Rtlsdr { index, gain, bias_tee } => Box::new(RtlsdrSource { index: *index, center_hz: rc.center_hz as u32, sample_rate: rc.sample_rate, gain: *gain, bias_tee: *bias_tee }),
        ReceiverKind::File { path, format, realtime } => Box::new(FileSource {
            path: path.into(),
            format: match format {
                FileFormat::Cu8 => Format::Cu8,
                FileFormat::Cs16 => Format::Cs16,
            },
            sample_rate: rc.sample_rate,
            realtime: *realtime,
            block: 65536,
        }),
    })
}

// keep the gain enums referenced for clarity in errors
#[allow(dead_code)]
fn _gain_types(_: AirspyGain, _: RtlsdrGain) {}

impl Runtime {
    pub fn start(cfg: &Config) -> Result<Runtime> {
        let (events_tx, _) = broadcast::channel::<Arc<Event>>(256);
        let (rx_events_tx, rx_events_rx) = mpsc::channel::<ReceiverEvent>();
        let mut receivers = Vec::new();
        for rc in &cfg.receivers {
            let source = make_source(rc)?;
            let source_desc = source.describe();
            let (block_tx, block_rx) = mpsc::sync_channel::<Block>(64);
            let counters = Arc::new(SourceCounters::default());
            let stats = Arc::new(Mutex::new(PipelineStats::default()));
            let rate = SampleRate(rc.sample_rate);
            let mut pipeline = Pipeline::new(ReceiverId(rc.name.clone()), rate, Hertz(rc.center_hz), PipelineConfig::default(), Utc::now());
            let stats2 = stats.clone();
            let ev_tx = rx_events_tx.clone();
            let name = rc.name.clone();
            std::thread::Builder::new().name(format!("rx-{name}")).spawn(move || {
                let mut index = SampleIndex(0);
                let mut blocks: u64 = 0;
                while let Ok(block) = block_rx.recv() {
                    if block.dropped_before > 0 {
                        // keep the sample clock honest: skip the index forward over what was lost
                        index = index.offset(block.dropped_before as i64);
                        pipeline.anchor(index, Utc::now());
                    }
                    if blocks % 64 == 0 {
                        pipeline.anchor(index, Utc::now() - chrono::Duration::milliseconds((block.samples.len() as f64 / rate.hz() * 1e3) as i64));
                    }
                    for ev in pipeline.push(&block.samples) {
                        let _ = ev_tx.send(ev);
                    }
                    index = index.offset(block.samples.len() as i64);
                    blocks += 1;
                    *stats2.lock().unwrap() = pipeline.stats();
                }
            })?;
            let running = source.start(BlockSink::new(block_tx, counters.clone())).with_context(|| format!("start receiver {}", rc.name))?;
            eprintln!("receiver {}: {}", rc.name, source_desc);
            receivers.push(ReceiverHandle { name: rc.name.clone(), source_desc, sample_rate: rc.sample_rate, center_hz: rc.center_hz, counters, stats, running: Some(running) });
        }
        drop(rx_events_tx);
        // merger thread
        let window = chrono::Duration::milliseconds(cfg.merge.window_ms as i64);
        let events_tx2 = events_tx.clone();
        std::thread::Builder::new().name("merger".into()).spawn(move || {
            let mut merger = Merger::new(window);
            loop {
                let out = match rx_events_rx.recv_timeout(Duration::from_millis(250)) {
                    Ok(ev) => merger.push(ev, Utc::now()),
                    Err(mpsc::RecvTimeoutError::Timeout) => merger.flush(Utc::now()),
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                };
                for ev in out {
                    let _ = events_tx2.send(Arc::new(ev));
                }
            }
        })?;
        Ok(Runtime { receivers, events: events_tx })
    }

    pub fn status(&self) -> Vec<ReceiverStatus> {
        self.receivers
            .iter()
            .map(|r| ReceiverStatus {
                name: r.name.clone(),
                source: r.source_desc.clone(),
                sample_rate: r.sample_rate,
                center_hz: r.center_hz,
                blocks: r.counters.blocks.load(Ordering::Relaxed),
                samples: r.counters.samples.load(Ordering::Relaxed),
                dropped_blocks: r.counters.dropped_blocks.load(Ordering::Relaxed),
                device_dropped_samples: r.counters.device_dropped_samples.load(Ordering::Relaxed),
                pipeline: (*r.stats.lock().unwrap()).into(),
            })
            .collect()
    }

    pub fn stop(&mut self) {
        for r in &mut self.receivers {
            if let Some(s) = r.running.take() {
                s.stop();
            }
        }
    }
}
