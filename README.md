# sensormon

Wideband, multi-signal receiver for 915 MHz Fineoffset/Ecowitt sensors. SDRs in
(Airspy, RTL-SDR, or recorded IQ files), decoded sensor events out (HTTP JSON
lines, and typed Rust structs you can import).

More capture bandwidth never makes it decode *worse*: every burst anywhere in
every receiver's window is detected in time–frequency, cut out, filtered to its
own bandwidth, demodulated and decoded independently, so simultaneous
transmissions at different frequencies all decode and strong out-of-channel
signals (SMPS combs, smart-meter hops) don't capture the demodulator.

## Status (2026-09-02)

On a 65 s Airspy R2 capture at 2.5 MS/s from the tower antenna, `rtl_433`
(best case, channel-filtered to 500 kS/s) decodes 14 frames; sensormon decodes
25: every WS90 beacon, a second (neighbour's) WS90, all five WH51 soil sensors,
two WH55 leak sensors and the WH57 lightning sensor. Runs at ~20× real time on
an M-series Mac and ~0.5 core on the N150 VM at 2.4 MS/s.

Deployed as `sensormon.service` on `radio` (NESDR, port 8433) next to the
rtl_433 pipeline (Airspy, port 80) for comparison.

## Layout

- `crates/sensormon-core` — pure, I/O-free: units, burst detector, extractor,
  FSK demod, Fineoffset framing + decoders (WS90, WH51, WH55, WH57/WH31L,
  WH25/WH32), typed events, cross-receiver merger, per-receiver `Pipeline`.
- `crates/sensormon` — the binary: TOML config, `dlopen`ed libairspy /
  librtlsdr drivers (no link-time dependency), file replay, threads,
  HTTP API.
- `samples/` — IQ corpus (gitignored; `airspy_915M_2500k_gain12_65s.cs16`,
  `nesdr_915M_2400k_bigant_19s.cu8`).
- `deploy/` — systemd unit and the radio config.

## Usage

```bash
# offline: decode a recording, print events as JSON lines (or --summary, --trace)
sensormon decode-file samples/airspy_915M_2500k_gain12_65s.cs16 --rate 2500000 --center 915000000 --format cs16 --summary

# live
sensormon run --config sensormon.toml     # see sensormon.example.toml
curl -sN http://radio:8433/events          # JSON lines, one Event per line
curl -s  http://radio:8433/stats
```

Event shape (control plane and data plane kept apart):

```json
{"reception":{"time":"2026-09-02T02:40:07.184Z","heard_by":{"garage-nesdr":{"center":915001234.0,"f_mark":915031000.0,"f_space":914962000.0,"symbol_rate":17246.0,"rssi":-52.0,"snr":41.0,"noise":-51.0}}},
 "sensor":{"model":"Fineoffset-WS90","id":68507,"battery_mv":2460,"temperature_c":33.5,"humidity_pct":50,...},
 "raw":[144,1,11,155,...]}
```

`sensormon_core::{Event, Reception, Signal, Payload, ...}` are the types to
import from home automation code.

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
IQ blocks ─► BurstDetector (512-pt STFT, per-bin tracked noise floor, hot-bin regions
              tracked over hops, carriers absorbed into the floor, bandwidth trimmed to
              25 dB below peak) ─► Burst {t, f_lo..f_hi}
          ─► extract (mix to burst center, FIR low-pass, decimate; wider bursts keep a
              higher rate so a burst merged with a neighbour still shows both tones)
          ─► demod_fsk (two strongest tones ≥30 kHz apart, per-tone boxcar matched
              filters, decision d=(|m|-|s|)/(|m|+|s|), symbol clock from a periodogram
              of d's zero crossings refined by least squares, slice at lattice midpoints)
          ─► decode_all (both polarities; 0xAA…0x2DD4 framing; CRC-8/0x31 + sum)
          ─► ReceiverEvent ─► Merger (identical raw bytes within 2 s across receivers)
          ─► Event ─► HTTP /events, stderr log
```

Diagnostics: `decode-file --trace --from S --to S` prints one line per burst
(span, band, peak, tones, rate, bits, syncs, frames); `SENSORMON_TRACE_BITS=1`
adds the bit string; `SENSORMON_BURST_TRACE=<bin>` traces the detector.
