# sensormon

Note from Will: I had Claude create this because `rtl_433` wasn't quite working for me. I switched to an Airspy SDR, which does minimum 2.5MSps, and this resulted in it picking up some ambient SMPS noise that caused `rtl_433` to ignore the actual signals I care about. `sensormon` has a channelization mechanism that works better with wideband receivers and also seems to provide better sensitivity on high-dynamic-range receivers. 

Remainder is Claude-written.

Wideband, multi-signal receiver for 915 MHz Fineoffset/Ecowitt sensors. SDRs in
(Airspy, RTL-SDR, or recorded IQ files), decoded sensor events out (HTTP JSON
lines, and typed Rust structs you can import).

More capture bandwidth never makes it decode *worse*: every burst anywhere in
every receiver's window is detected in time–frequency, cut out, filtered to its
own bandwidth, demodulated and decoded independently, so simultaneous
transmissions at different frequencies all decode and strong out-of-channel
signals (SMPS combs, smart-meter hops) don't capture the demodulator.

## Status (2026-09-02)

Running as `sensormon.service` on the `radio` VM with two receivers:

| receiver | SDR | bands | decoders |
|---|---|---|---|
| `tower-airspy` | Airspy R2 via USB-over-fiber, 2.5 MS/s | 915 MHz | Fineoffset WS90 / WH51 / WH55 / WH57 / WH25 |
| `garage-nesdr` | NESDR SMArt v5, 2.4 MS/s | hops 315 ↔ 433.92 MHz, 15 s dwell | Toyota TPMS (315); Fineoffset (433) |

home2 reads `/events` (`--sensor-source sensormon`, the default).

On a 65 s Airspy capture from the tower antenna, `rtl_433` (best case, channel-
filtered to 500 kS/s) decodes 14 frames; sensormon decodes 25: every WS90
beacon, a second (neighbour's) WS90, all five WH51 soil sensors, two WH55 leak
sensors and the WH57 lightning sensor. That capture decodes in 2.3 s on an
M-series Mac (28× real time); live, both receivers together use ~26% of one
vCPU on the N150 VM.

## Layout

- `crates/sensormon-core` — pure, I/O-free: units, burst detector, extractor,
  FSK demod, framing + decoders (Fineoffset WS90, WH51, WH55, WH57/WH31L,
  WH25/WH32; Toyota TPMS), typed events, cross-receiver merger, per-receiver
  `Pipeline` with per-stage timers.
- `crates/sensormon` — the binary: TOML config, `dlopen`ed libairspy /
  librtlsdr drivers (no link-time dependency), RTL-SDR frequency hopping,
  file replay, threads, HTTP API.
- `samples/` — IQ corpus (gitignored; `airspy_915M_2500k_gain12_65s.cs16`,
  `nesdr_915M_2400k_bigant_19s.cu8`).
- `deploy/` — systemd unit and the radio config.

## Usage

```bash
# offline: decode a recording, print events as JSON lines (or --summary, --trace)
sensormon decode-file samples/airspy_915M_2500k_gain12_65s.cs16 --rate 2500000 --center 915000000 --format cs16 --summary

# live
sensormon run --config sensormon.toml     # see sensormon.example.toml / deploy/radio.toml
curl -sN http://radio:8433/events          # JSON lines, one Event per line
curl -s  http://radio:8433/stats           # per receiver: current band, blocks, drops, bursts, frames
```

A receiver has either `center_hz` or `hop_hz = [...]` plus `dwell_s` (RTL-SDR
only): the dongle is retuned every dwell, samples are tagged with the band
they came from, each band gets its own pipeline (own noise floor and sample
clock), and a 60 ms settle window after each retune is discarded. Decoders
declare the RF bands they live in, so a band only runs the symbol rates of
the decoders that can occur there.

Event shape (control plane and data plane kept apart):

```json
{"reception":{"time":"2026-09-02T02:40:07.184Z","heard_by":{"garage-nesdr":{"center":915001234.0,"f_mark":915031000.0,"f_space":914962000.0,"symbol_rate":17246.0,"rssi":-52.0,"snr":41.0,"noise":-51.0}}},
 "sensor":{"model":"Fineoffset-WS90","id":68507,"battery_mv":2460,"temperature_c":33.5,"humidity_pct":50,...},
 "raw":[144,1,11,155,...]}
```

`sensormon_core::{Event, Reception, Signal, Payload, ...}` are the types to
import from home automation code (`Payload` is an enum: `Ws90`, `Wh51`,
`Wh55`, `Wh31l`, `Wh25`, `ToyotaTpms`; fields the sensor can flag as
unavailable are `Option`s).

## Build / deploy

```bash
cargo test
cargo build --release
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.41   # for radio (Debian 13)
scp target/x86_64-unknown-linux-gnu/release/sensormon radio:/usr/local/bin/sensormon.new
ssh radio 'mv /usr/local/bin/sensormon.new /usr/local/bin/sensormon && systemctl restart sensormon'
```

Needs `brew install zig && cargo install cargo-zigbuild && rustup target add x86_64-unknown-linux-gnu`.
The target host needs `libairspy0` / `librtlsdr0` (loaded at runtime).

## How it works

```
IQ blocks ─► BurstDetector   512-pt STFT (Blackman-Harris); per-bin noise floor tracked as a
              │              linear EMA (rise rate-limited while a bin is hot, so bursts don't
              │              lift it and permanent carriers are absorbed); hot = above floor by a
              │              ratio; hot-bin regions tracked over hops, bounded by their strong hops,
              │              trimmed to 25 dB below peak. No per-bin transcendentals.
              ▼
          Burst {t, f_lo..f_hi}
              ▼
          extract            mix to burst center, FIR low-pass, decimate (≥250 kS/s; wider regions
              │              keep a proportionally higher rate so a burst merged with a neighbour
              ▼              still shows both tones)
          tone pair          two strongest peaks ≥30 kHz apart — found once per burst
              ▼
          demod (per symbol rate the band's decoders need)
              │              per-tone boxcar matched filters, d=(|m|-|s|)/(|m|+|s|); symbol clock
              │              from a periodogram of d's zero crossings refined by least squares;
              ▼              slice at lattice midpoints
          decode_all         both polarities; Fineoffset 0xAA…0x2DD4 framing + CRC-8/0x31 + sum;
              │              Toyota TPMS sync + differential Manchester + CRC-8/0x07
              ▼
          ReceiverEvent ─► Merger (identical raw bytes within 2 s across receivers)
              ▼
          Event ─► HTTP /events, stderr log
```

Performance notes: `decode-file` prints per-stage times. The costs that
mattered were the demod clock search on long bursts (now a bounded coarse
search + least squares), running the tone search once per symbol rate instead
of once per burst, filtering wide splatter regions at full rate, and the
detector's three transcendentals per bin per hop (now none). Every change was
checked against the sample corpus with the decode counts unchanged.

Diagnostics: `decode-file --trace --from S --to S` prints one line per burst
(span, band, peak, tones, rate, bits, syncs, frames); `SENSORMON_TRACE_BITS=1`
adds the bit string; `SENSORMON_BURST_TRACE=<bin>` traces the detector.
Live: `journalctl -fu sensormon` shows one line per event with each
receiver's band and SNR, and `retuned to …` on every hop.

## Known limits / next

- Detection and demod thresholds are hand-set (see `BurstDetectorConfig`,
  `FskParams`); a CFAR threshold from the measured noise statistics and
  limits derived from the decoder set are the planned follow-ups, as is a
  slow AGC on SDR gain.
- FSK only; no OOK decoders yet.
- No golden-fixture test on the corpus yet (verification is manual
  `decode-file --summary` against the numbers above).
