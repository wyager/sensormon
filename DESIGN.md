# sensormon — design

> Original design sketch (pre-implementation). README.md describes what was
> actually built; where they differ (subprocess adapters → dlopen'ed drivers,
> rtl_433 JSON → own typed events, `sensormon-tools` folded into the binary),
> the code and the Decisions section at the bottom win.

Wideband multi-signal receiver for 915 MHz Fineoffset/Ecowitt sensors (and later
anything else FSK/OOK we care about). Replaces rtl_433 in the home2 pipeline.

Goal: **more bandwidth must never make decoding worse.** Every burst anywhere
in every receiver's capture window is detected, isolated, demodulated and
decoded independently. Multiple simultaneous bursts at different frequencies
are all decoded. Strong out-of-channel signals (SMPS combs, meters) are
irrelevant because each burst is filtered to its own occupied bandwidth before
demodulation.

## Principles

- **Pure DSP core, impure edges.** Everything from IQ samples to decoded events
  is a pure function of its inputs (`&[Complex<f32>]` in, values out) with no
  hidden state beyond explicitly-passed `State` structs. Only `source::*`
  (processes/USB), `sink::*` (HTTP/stdout) and `main` do I/O. Same shape as
  home2's `process_input`/`effect_world`.
- **Everything is testable offline** against recorded IQ. The corpus is the
  65 s Airspy capture (2.5 MS/s, known contents: WS90 ×7, WH51 ×4–6, WH57 ×1,
  WH55) plus NESDR captures. rtl_433's decodes are the ground truth to beat.
- **Typed all the way.** Frequencies are `Hertz(f64)` newtypes, sample rates
  `SampleRate(u32)`, times `Instant`/`DateTime<Utc>`; sensor payloads are
  enums, not JSON maps. No global mutable state, no `static mut`, no singletons.
- **One receiver = one independent pipeline** (thread) up to the decoded-event
  stream; a merge stage joins them. Adding a third SDR is a config line.

## Data flow

```
   [Airspy R2]        [NESDR]           config.toml: receivers, gains, freqs
       |                 |
  airspy_rx proc     rtl_sdr proc              (subprocess adapters; FFI later)
       |                 |
  IqSource ------- IqSource            trait: pull blocks of Complex<f32> + StreamSpec
       |                 |
  BurstDetector    BurstDetector       STFT energy vs. running noise floor per bin,
       |                 |             hysteresis in time; emits Burst{t0,t1,f_lo,f_hi}
  BurstExtractor   BurstExtractor      mix to baseband, low-pass to burst BW, decimate to 200 kS/s
       |                 |
  Demodulators     Demodulators        FSK (matched filter f1/f2, timing recovery), OOK later
       |                 |
  Framers/Decoders Framers/Decoders    Fineoffset framing (0xAA…0x2DD4, CRC-8, sum) → typed payload
       |                 |
  ReceiverEvent    ReceiverEvent       {receiver, time, freq, rssi/snr/noise, payload, raw bytes}
       \_______   _______/
               \ /
             Merger                    dedupe identical payloads across receivers within window
               |
             Event  ──► sinks: HTTP /events (rtl_433-compatible JSON lines), stdout JSON, (later: direct home-server)
```

## Crate layout

```
sensormon/
  Cargo.toml            workspace
  crates/
    sensormon-core/     pure: dsp, burst, demod, protocol, event, merge   (no I/O, no tokio)
    sensormon/          binary: config, source adapters, runtime, sinks
    sensormon-tools/    offline tools: decode a capture, dump bursts, compare vs rtl_433 jsonl
  samples/              IQ corpus + expected-events fixtures (git-lfs or gitignored + fetch script)
```

## Core types & interfaces (sensormon-core)

```rust
// units.rs — newtypes so Hz/samples/seconds can't be mixed up
pub struct Hertz(pub f64);
pub struct SampleRate(pub u32);
pub struct SampleIndex(pub u64);            // absolute index within a receiver's stream
pub struct Db(pub f32);

// stream.rs
pub struct StreamSpec { pub sample_rate: SampleRate, pub center: Hertz, pub receiver: ReceiverId }
pub struct ReceiverId(pub String);          // "tower-airspy", "garage-nesdr"

// burst.rs — time/frequency energy detection over the full capture bandwidth
pub struct BurstDetectorConfig {
    pub fft_size: usize,                    // e.g. 1024 @ 2.5 MS/s → 2.4 kHz bins, 0.4 ms hops
    pub hop: usize,
    pub threshold_db: Db,                   // over per-bin running noise floor
    pub min_duration: Duration, pub max_duration: Duration,
    pub min_bandwidth: Hertz, pub max_bandwidth: Hertz,
    pub cw_reject: bool,                    // ignore bins that are "always on" (comb lines, carriers)
}
pub struct BurstDetector { cfg: BurstDetectorConfig, noise_floor: Vec<f32>, active: Vec<ActiveRegion>, /* … */ }
pub struct Burst { pub start: SampleIndex, pub end: SampleIndex, pub f_lo: Hertz, pub f_hi: Hertz, pub peak_db: Db, pub noise_db: Db }
impl BurstDetector {
    pub fn new(cfg: BurstDetectorConfig, spec: &StreamSpec) -> Self;
    /// Feed one block; returns bursts that *completed* inside this block. Keeps a ring of
    /// recent samples so `extract` can serve completed bursts.
    pub fn push(&mut self, block: &[Complex<f32>], first_index: SampleIndex) -> Vec<Burst>;
}

// extract.rs — isolate one burst: frequency-shift to its center, FIR low-pass to its bandwidth, decimate
pub struct Baseband { pub iq: Vec<Complex<f32>>, pub sample_rate: SampleRate, pub center: Hertz, pub burst: Burst }
pub fn extract(ring: &SampleRing, burst: &Burst, spec: &StreamSpec, out_rate: SampleRate) -> Baseband;

// demod/fsk.rs — pure, per burst
pub struct FskParams { pub symbol_rate: Hertz, pub deviation_hint: Option<Hertz> }
pub struct Symbols { pub bits: Vec<bool>, pub t0: SampleIndex, pub f_mark: Hertz, pub f_space: Hertz, pub snr: Db, pub rssi: Db }
/// Estimate the two tones (spectral peaks), matched-filter each tone at the symbol rate,
/// recover timing from the preamble, slice. Returns None if no 2-FSK structure is found.
pub fn demod_fsk(bb: &Baseband, p: &FskParams) -> Option<Symbols>;

// protocol/mod.rs — decoders are pure functions over a bit vector
pub trait Decoder: Send + Sync {
    fn name(&self) -> &'static str;                 // "Fineoffset-WH51"
    fn modulation(&self) -> Modulation;             // Fsk{symbol_rate} | Ook{..}
    fn decode(&self, sym: &Symbols) -> Vec<Decoded>; // 0..n frames (repeats, multiple in one burst)
}
pub struct Decoded { pub payload: Payload, pub raw: Vec<u8>, pub bit_offset: usize }

// protocol/fineoffset.rs — shared framing: find 0xAA preamble + 0x2DD4 sync, byte-align, CRC-8 (poly 0x31), sum
pub fn find_frames(bits: &[bool], len: usize) -> Vec<(usize, Vec<u8>)>;
pub fn crc8_0x31(bytes: &[u8]) -> u8;
// protocol/{ws90,wh51,wh55,wh57,wh25}.rs — byte layouts ported from rtl_433's src/devices/fineoffset*.c

// event.rs
pub enum Payload { Ws90(Ws90), Wh51(Wh51), Wh55(Wh55), Wh57(Wh57), Wh25(Wh25) }   // typed fields, units in names
pub struct Signal { pub freq: Hertz, pub rssi: Db, pub snr: Db, pub noise: Db, pub f_mark: Hertz, pub f_space: Hertz }
pub struct ReceiverEvent { pub receiver: ReceiverId, pub time: DateTime<Utc>, pub signal: Signal, pub decoded: Decoded }
pub struct Event { pub time: DateTime<Utc>, pub payload: Payload, pub raw: Vec<u8>, pub heard_by: Vec<(ReceiverId, Signal)> }

// merge.rs — pure state machine
pub struct Merger { window: Duration, pending: Vec<(ReceiverEvent, Instant)> }
impl Merger {
    pub fn push(&mut self, ev: ReceiverEvent, now: Instant) -> Vec<Event>;   // emits when window closes
    pub fn flush(&mut self, now: Instant) -> Vec<Event>;
}

// json.rs — rtl_433-compatible serialization (field names/units identical) so home2 needs no change
pub fn to_rtl433_json(ev: &Event) -> String;
```

## Runtime (sensormon binary)

```rust
// source.rs
pub trait IqSource: Send { fn spec(&self) -> StreamSpec; fn read(&mut self, out: &mut Vec<Complex<f32>>) -> io::Result<usize>; }
pub struct AirspyRxProcess { .. }   // spawns `airspy_rx -r /dev/stdout -t 2 …`, parses int16 IQ
pub struct RtlSdrProcess   { .. }   // spawns `rtl_sdr -f … -s … -g … -`, parses uint8 IQ
pub struct FileSource      { .. }   // .cs16/.cu8/.cf32 at a given rate, optional realtime pacing
// A receiver pipeline = one std::thread: source → detector → extract → demod → decoders → mpsc<ReceiverEvent>
// main: spawn N pipelines, one Merger task, sinks (axum HTTP /events streaming JSON lines; stdout).
```

Config (`sensormon.toml`):
```toml
[[receiver]]
name = "tower-airspy"
kind = "airspy"            # airspy | rtlsdr | file
center_hz = 915_000_000
sample_rate = 2_500_000
gain = { lna = 12, mix = 12, vga = 12 }

[[receiver]]
name = "garage-nesdr"
kind = "rtlsdr"
center_hz = 915_000_000
sample_rate = 2_400_000
gain = 40.2

[merge]  window_ms = 2000
[http]   bind = "0.0.0.0:80"
```

## Deployment

Cross-compiled static binary (`x86_64-unknown-linux-musl`) copied to `radio`, run as
`sensormon.service` next to (initially) rtl_433 on a different port, so decode counts can be
compared for a while before switching `--rtl433-address`.

## Test strategy

- Unit: CRC/sum, each decoder against known byte strings (ported from rtl_433 test vectors and
  our captured raw bytes), FSK demod on synthesized bursts at several SNRs and rates.
- Golden: `sensormon-tools decode samples/airspy_65s.cs16` must reproduce ≥ the rtl_433 event set
  (fixture jsonl), and additionally decode the bursts rtl_433 misses; regression on every change.
- Bench: throughput on 2.5 MS/s must fit in < 1 vCPU on radio (N150-class).

## Decisions (2026-09-02)

1. Own typed event format (`Event { reception, sensor, raw }`), exported from
   `sensormon-core` for home automation code to import. No rtl_433 JSON compatibility.
2. In-process SDR library bindings from day one (dlopen'ed, hand-written FFI).
3. Standalone Cargo workspace, git tracked.
4. Cross-compiled glibc binary (`cargo zigbuild`) deployed to `radio`, run beside rtl_433.
5. Decoders v1: WS90, WH51, WH55, WH57(WH31L), WH25/WH32.
6. Merged events carry a map `receiver → Signal`.
7. IQ corpus lives in `samples/` (gitignored).
