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
use std::collections::HashMap;
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
    pub centers_hz: Vec<f64>,
    pub current_center_hz: f64,
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
    centers_hz: Vec<f64>,
    counters: Arc<SourceCounters>,
    /// File sources legitimately stop at end of file; only live SDRs are watched.
    watchdog: bool,
    stats: Arc<Mutex<(PipelineStats, f64)>>,
    running: Option<Box<dyn RunningSource>>,
}

pub struct Runtime {
    receivers: Vec<ReceiverHandle>,
    pub events: broadcast::Sender<Arc<Event>>,
    pub chirps: Option<Arc<Mutex<crate::chirps::ChirpStore>>>,
}

/// A receiver that has delivered no samples for this long is considered
/// stalled. An SDR that drops off USB (and comes back) leaves libairspy /
/// librtlsdr silently idle: the process stays "active" while decoding nothing,
/// which is exactly what happened 2026-09-08 (NESDR) and 2026-09-11 (Airspy).
pub const STALL_TIMEOUT: Duration = Duration::from_secs(15);

/// Receivers whose block counter has not advanced for `STALL_TIMEOUT`.
/// Checked from a watchdog thread; empty means healthy.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Stalled {
    pub receivers: Vec<String>,
}

fn make_source(rc: &ReceiverConfig) -> Result<Box<dyn IqSource>> {
    Ok(match &rc.kind {
        ReceiverKind::Airspy { serial, gain, bias_tee } => {
            let serial = serial.as_ref().map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16)).transpose().context("airspy serial must be hex")?;
            Box::new(AirspySource { serial, center_hz: rc.centers()[0] as u32, sample_rate: rc.sample_rate, gain: *gain, bias_tee: *bias_tee })
        }
        ReceiverKind::Rtlsdr { index, gain, bias_tee } => Box::new(RtlsdrSource { index: *index, centers_hz: rc.centers().iter().map(|c| *c as u32).collect(), dwell: Duration::from_secs_f64(rc.dwell_s), sample_rate: rc.sample_rate, gain: *gain, bias_tee: *bias_tee }),
        ReceiverKind::File { path, format, realtime } => Box::new(FileSource {
            path: path.into(),
            format: match format {
                FileFormat::Cu8 => Format::Cu8,
                FileFormat::Cs16 => Format::Cs16,
            },
            sample_rate: rc.sample_rate,
            realtime: *realtime,
            block: 65536,
            center_hz: rc.centers()[0],
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
        // Undecoded bursts go to one writer thread so the receiver threads never touch SQLite.
        // Bounded so a slow disk can't grow memory; per-family admission below keeps a
        // hopper from filling it, so no receiver starves (that happened at 256 slots
        // once the store hit its cap and the writer slowed down).
        let (chirp_tx, chirp_rx) = mpsc::sync_channel::<sensormon_core::pipeline::Chirp>(1024);
        let chirps = match &cfg.chirps {
            Some(cc) => {
                let store = Arc::new(Mutex::new(crate::chirps::ChirpStore::open(cc.clone()).context("open chirp store")?));
                let store2 = store.clone();
                std::thread::Builder::new().name("chirp-store".into()).spawn(move || {
                    while let Ok(c) = chirp_rx.recv() {
                        if let Err(e) = store2.lock().unwrap().insert(&c) {
                            eprintln!("chirp store: {e:#}");
                        }
                    }
                })?;
                eprintln!("chirps: recording undecoded bursts to {} (cap {:.0} MB)", cc.path, cc.max_bytes as f64 / 1e6);
                Some(store)
            }
            None => None,
        };
        let chirp_receivers: Vec<String> = cfg.chirps.as_ref().map(|c| c.receivers.clone()).unwrap_or_default();
        let family_interval = cfg.chirps.as_ref().map(|c| c.min_family_interval_s).unwrap_or(0.0);
        let mut receivers = Vec::new();
        for rc in &cfg.receivers {
            let source = make_source(rc)?;
            let source_desc = source.describe();
            let (block_tx, block_rx) = mpsc::sync_channel::<Block>(64);
            let counters = Arc::new(SourceCounters::default());
            let stats = Arc::new(Mutex::new((PipelineStats::default(), 0.0f64)));
            let rate = SampleRate(rc.sample_rate);
            let receiver_id = ReceiverId(rc.name.clone());
            let stats2 = stats.clone();
            let ev_tx = rx_events_tx.clone();
            let name = rc.name.clone();
            let chirp_tx = chirp_tx.clone();
            let keep_chirps = chirps.is_some() && (chirp_receivers.is_empty() || chirp_receivers.contains(&rc.name));
            let max_bands = rc.max_band_pipelines.max(1);
            // Samples to discard after a retune while the PLL settles and the old band's tail flushes.
            let settle = rate.samples_in(0.06) as u64;
            std::thread::Builder::new().name(format!("rx-{name}")).spawn(move || {
                // One pipeline per band the receiver visits (lazily created); each keeps
                // its own noise floor and sample clock. `global` counts every sample the
                // SDR produced, so a band's pipeline can skip forward over the time it
                // wasn't being received.
                let mut pipelines: HashMap<u64, (Pipeline, SampleIndex)> = HashMap::new();
                let mut lru: Vec<u64> = Vec::new(); // most recently used last
                // per-family admission: last time an example of each family was sent
                let mut family_last: HashMap<String, std::time::Instant> = HashMap::new();
                let mut family_sweep = std::time::Instant::now();
                let mut evicted = PipelineStats::default(); // counters of bands no longer resident
                let mut global = SampleIndex(0);
                let mut current: Option<u64> = None;
                let mut settle_left: u64 = 0;
                let mut blocks: u64 = 0;
                while let Ok(block) = block_rx.recv() {
                    global = global.offset(block.dropped_before as i64);
                    let key = block.center_hz.to_bits();
                    if current != Some(key) {
                        current = Some(key);
                        settle_left = settle;
                    }
                    let n = block.samples.len() as u64;
                    if settle_left > 0 {
                        settle_left = settle_left.saturating_sub(n);
                        global = global.offset(n as i64);
                        continue;
                    }
                    if !pipelines.contains_key(&key) && pipelines.len() >= max_bands {
                        if let Some(old) = lru.first().copied() {
                            if let Some((p, _)) = pipelines.remove(&old) {
                                let st = p.stats();
                                evicted.samples += st.samples;
                                evicted.bursts += st.bursts;
                                evicted.bursts_lost_from_ring += st.bursts_lost_from_ring;
                                evicted.demodulated += st.demodulated;
                                evicted.decoded_frames += st.decoded_frames;
                            }
                            lru.remove(0);
                        }
                    }
                    lru.retain(|k| *k != key);
                    lru.push(key);
                    let (pipeline, seen_upto) = pipelines.entry(key).or_insert_with(|| {
                        let mut p = Pipeline::new(receiver_id.clone(), rate, Hertz(block.center_hz), PipelineConfig::default(), Utc::now());
                        p.set_keep_undecoded(keep_chirps);
                        (p, global)
                    });
                    let gap = seen_upto.distance_to(global);
                    if gap > 0 {
                        pipeline.skip_samples(gap, Utc::now());
                    }
                    if blocks % 64 == 0 {
                        pipeline.anchor(pipeline.next_index(), Utc::now() - chrono::Duration::milliseconds((n as f64 / rate.hz() * 1e3) as i64));
                    }
                    for ev in pipeline.push(&block.samples) {
                        let _ = ev_tx.send(ev);
                    }
                    for c in pipeline.take_chirps() {
                        let now = std::time::Instant::now();
                        if family_interval > 0.0 {
                            let fam = c.family();
                            if family_last.get(&fam).is_some_and(|t| now.duration_since(*t).as_secs_f64() < family_interval) {
                                continue;
                            }
                            family_last.insert(fam, now);
                            if now.duration_since(family_sweep).as_secs() > 600 {
                                family_last.retain(|_, t| now.duration_since(*t).as_secs_f64() < 10.0 * family_interval);
                                family_sweep = now;
                            }
                        }
                        let _ = chirp_tx.try_send(c); // full queue: drop rather than stall the receiver
                    }
                    global = global.offset(n as i64);
                    *seen_upto = global;
                    blocks += 1;
                    let mut total = evicted;
                    for (p, _) in pipelines.values() {
                        let st = p.stats();
                        total.samples += st.samples;
                        total.bursts += st.bursts;
                        total.bursts_lost_from_ring += st.bursts_lost_from_ring;
                        total.demodulated += st.demodulated;
                        total.decoded_frames += st.decoded_frames;
                    }
                    *stats2.lock().unwrap() = (total, block.center_hz);
                }
            })?;
            let center = Arc::new(std::sync::atomic::AtomicU64::new(rc.centers()[0].to_bits()));
            let running = source.start(BlockSink::new(block_tx, counters.clone(), center)).with_context(|| format!("start receiver {}", rc.name))?;
            eprintln!("receiver {}: {}", rc.name, source_desc);
            let watchdog = !matches!(rc.kind, ReceiverKind::File { .. });
            receivers.push(ReceiverHandle { name: rc.name.clone(), source_desc, sample_rate: rc.sample_rate, centers_hz: rc.centers(), counters, watchdog, stats, running: Some(running) });
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
        drop(chirp_tx);
        Ok(Runtime { receivers, events: events_tx, chirps })
    }

    /// Per-receiver (name, block counter) snapshot for the stall watchdog
    /// (live SDR receivers only).
    pub fn block_counters(&self) -> Vec<(String, u64)> {
        self.receivers.iter().filter(|r| r.watchdog).map(|r| (r.name.clone(), r.counters.blocks.load(Ordering::Relaxed))).collect()
    }

    pub fn status(&self) -> Vec<ReceiverStatus> {
        self.receivers
            .iter()
            .map(|r| {
                // one lock: the (stats, center) tuple is Copy
                let (ps, center) = *r.stats.lock().unwrap();
                ReceiverStatus {
                name: r.name.clone(),
                source: r.source_desc.clone(),
                sample_rate: r.sample_rate,
                centers_hz: r.centers_hz.clone(),
                current_center_hz: center,
                blocks: r.counters.blocks.load(Ordering::Relaxed),
                samples: r.counters.samples.load(Ordering::Relaxed),
                dropped_blocks: r.counters.dropped_blocks.load(Ordering::Relaxed),
                device_dropped_samples: r.counters.device_dropped_samples.load(Ordering::Relaxed),
                pipeline: ps.into(),
            }})
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
