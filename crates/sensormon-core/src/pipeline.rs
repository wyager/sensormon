//! One receiver's complete chain, as a pure state machine fed with sample
//! blocks: detect bursts → extract → demodulate → decode → `ReceiverEvent`s.

use crate::dsp::burst::{Burst, BurstDetector, BurstDetectorConfig};
use crate::dsp::extract::{extract, ExtractConfig};
use crate::dsp::fsk::{demod_fsk_with_tones, find_tone_pair, FskParams};
use crate::dsp::ring::SampleRing;
use crate::dsp::Iq;
use crate::event::{ReceiverEvent, ReceiverId, Signal};
use crate::protocol::{decode_all, decoders_for_center, decoders_for_rate, symbol_rates, Decoder};
use crate::units::{Hertz, SampleIndex, SampleRate};
use chrono::{DateTime, Duration, Utc};
use crate::dsp::fsk::Symbols;

#[derive(Clone, Debug)]
pub struct PipelineConfig {
    pub detector: BurstDetectorConfig,
    pub extract: ExtractConfig,
    pub fsk: FskParams,
    /// How much raw history to keep; must exceed max burst + detector latency.
    pub ring_seconds: f32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig { detector: BurstDetectorConfig::default(), extract: ExtractConfig::default(), fsk: FskParams::fineoffset(), ring_seconds: 1.0 }
    }
}

/// Counters for diagnostics and tests.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PipelineStats {
    pub samples: u64,
    pub bursts: u64,
    pub bursts_lost_from_ring: u64,
    pub demodulated: u64,
    pub decoded_frames: u64,
    /// CPU time per stage (nanoseconds), for profiling.
    pub ns_detect: u64,
    pub ns_extract: u64,
    pub ns_demod: u64,
    pub ns_decode: u64,
}

/// Per-burst diagnostic record (collected only when tracing is on).
#[derive(Clone, Debug)]
pub struct BurstTrace {
    pub burst: Burst,
    pub extracted: bool,
    pub symbols: Option<Symbols>,
    /// Bit offsets of Fineoffset sync words found (in either polarity).
    pub syncs: usize,
    pub frames: usize,
}

pub struct Pipeline {
    receiver: ReceiverId,
    rate: SampleRate,
    center: Hertz,
    cfg: PipelineConfig,
    ring: SampleRing,
    detector: BurstDetector,
    decoders: Vec<Box<dyn Decoder>>,
    next_index: SampleIndex,
    /// Wall-clock time of sample index `anchor_index`.
    anchor_time: DateTime<Utc>,
    anchor_index: SampleIndex,
    stats: PipelineStats,
    trace: bool,
    traces: Vec<BurstTrace>,
}

impl Pipeline {
    pub fn new(receiver: ReceiverId, rate: SampleRate, center: Hertz, cfg: PipelineConfig, start_time: DateTime<Utc>) -> Self {
        let ring = SampleRing::new(rate.samples_in(cfg.ring_seconds as f64), SampleIndex(0));
        let detector = BurstDetector::new(cfg.detector.clone(), rate, center, SampleIndex(0));
        Pipeline { receiver, rate, center, cfg, ring, detector, decoders: crate::protocol::decoders(), next_index: SampleIndex(0), anchor_time: start_time, anchor_index: SampleIndex(0), stats: PipelineStats::default(), trace: false, traces: Vec::new() }
    }

    /// Re-anchor stream time to wall clock (call occasionally from the runtime).
    pub fn anchor(&mut self, index: SampleIndex, time: DateTime<Utc>) {
        self.anchor_index = index;
        self.anchor_time = time;
    }

    pub fn stats(&self) -> PipelineStats {
        self.stats
    }

    pub fn set_trace(&mut self, on: bool) {
        self.trace = on;
    }

    pub fn take_traces(&mut self) -> Vec<BurstTrace> {
        std::mem::take(&mut self.traces)
    }

    pub fn time_of(&self, index: SampleIndex) -> DateTime<Utc> {
        let dt = (index.0 as i64 - self.anchor_index.0 as i64) as f64 / self.rate.hz();
        self.anchor_time + Duration::microseconds((dt * 1e6) as i64)
    }

    /// Feed the next contiguous block of samples.
    pub fn push(&mut self, block: &[Iq]) -> Vec<ReceiverEvent> {
        self.ring.push(block);
        self.next_index = self.next_index.offset(block.len() as i64);
        self.stats.samples += block.len() as u64;
        let t = std::time::Instant::now();
        let bursts = self.detector.push(block);
        self.stats.ns_detect += t.elapsed().as_nanos() as u64;
        let mut out = Vec::new();
        for b in bursts {
            self.stats.bursts += 1;
            out.extend(self.handle_burst(&b));
        }
        out
    }

    fn handle_burst(&mut self, b: &Burst) -> Vec<ReceiverEvent> {
        let mut tr = BurstTrace { burst: *b, extracted: false, symbols: None, syncs: 0, frames: 0 };
        let out = self.handle_burst_inner(b, &mut tr);
        if self.trace {
            self.traces.push(tr);
        }
        out
    }

    fn handle_burst_inner(&mut self, b: &Burst, tr: &mut BurstTrace) -> Vec<ReceiverEvent> {
        let t = std::time::Instant::now();
        let extracted = extract(&self.ring, b, self.rate, self.center, &self.cfg.extract);
        self.stats.ns_extract += t.elapsed().as_nanos() as u64;
        let Some(bb) = extracted else {
            self.stats.bursts_lost_from_ring += 1;
            return Vec::new();
        };
        tr.extracted = true;
        // Tone pair once per burst, then one slicing pass per distinct symbol rate.
        let mut out = Vec::new();
        let mut any_demod = false;
        let t = std::time::Instant::now();
        let tones = find_tone_pair(&bb, &self.cfg.fsk);
        self.stats.ns_demod += t.elapsed().as_nanos() as u64;
        let Some(tones) = tones else { return out };
        let relevant = decoders_for_center(&self.decoders, self.center);
        for rate in symbol_rates(&relevant) {
            let params = FskParams { symbol_rate: rate, ..self.cfg.fsk };
            let t = std::time::Instant::now();
            let demod = demod_fsk_with_tones(&bb, &params, tones);
            self.stats.ns_demod += t.elapsed().as_nanos() as u64;
            let Some(sym) = demod else { continue };
            any_demod = true;
            if self.trace && tr.symbols.is_none() {
                let inv = crate::bits::inverted(&sym.bits);
                tr.syncs = crate::protocol::fineoffset::frame_starts(&sym.bits).len() + crate::protocol::fineoffset::frame_starts(&inv).len();
                tr.symbols = Some(sym.clone());
            }
            let signal = Signal { center: bb.center, f_mark: sym.f_mark, f_space: sym.f_space, symbol_rate: sym.symbol_rate, rssi: sym.rssi, snr: sym.snr, noise: sym.noise };
            let time = self.time_of(sym.t0);
            let t = std::time::Instant::now();
            let decoded = decode_all(&decoders_for_rate(&relevant, rate), &sym.bits);
            self.stats.ns_decode += t.elapsed().as_nanos() as u64;
            self.stats.decoded_frames += decoded.len() as u64;
            tr.frames += decoded.len();
            out.extend(decoded.into_iter().map(|d| ReceiverEvent { receiver: self.receiver.clone(), time, signal, sensor: d.payload, raw: d.raw }));
        }
        if any_demod {
            self.stats.demodulated += 1;
        }
        out
    }

    /// The stream skipped `n` samples (dropped, or this pipeline's band was not
    /// being received while the SDR was tuned elsewhere): keep the sample
    /// clock and wall-clock anchor honest.
    pub fn skip_samples(&mut self, n: u64, now: DateTime<Utc>) {
        self.next_index = self.next_index.offset(n as i64);
        self.anchor(self.next_index, now);
    }

    pub fn next_index(&self) -> SampleIndex {
        self.next_index
    }
}
