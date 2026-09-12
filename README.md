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
{"reception":{"time":"2026-09-02T02:40:07.184Z","heard_by":{"garage-nesdr":{"center":915001234.0,"f_mark":915031000.0,"f_space":914962000.0,"symbol_rate":17246.0,"rssi":-10.0,"snr":41.0,"noise":-51.0}}},
 "sensor":{"model":"Fineoffset-WS90","id":68507,"battery_mv":2460,"temperature_c":33.5,"humidity_pct":50,...},
 "raw":[144,1,11,155,...]}
```

### Chirps: everything that *didn't* decode

With a `[chirps]` section in the config, every burst that yields no frame is
kept: `groups` aggregates recurring emitters (receiver, center to 10 kHz,
bandwidth class, duration class → count, first/last seen, mean SNR, FSK tone
spacing, symbol rate, last bits, typical period), and a few example IQ
captures per group are stored as peak-normalised int8 I/Q (`.cs8`, already
mixed to the burst's center) under a global cap (default 50 MB, oldest
evicted; group statistics survive eviction).

```
GET /chirps/stats                 counts, bytes, time span
GET /chirps/groups?receiver=&min_count=   recurring emitters, most frequent first
GET /chirps?group=N&limit=        example metadata for a group
GET /chirps/<id>                  one chirp's metadata
GET /chirps/<id>/iq               raw cs8; X-Sample-Rate / X-Center-Hz headers
```

An example can be fed straight to rtl_433 to try its ~250 decoders:
`rtl_433 -r chirp.cs8 -s <sample_rate> -f <center>` (also `-A` to analyse).
`decode-file --chirps-db path` does the same offline. `[chirps] receivers =
["name", ...]` restricts recording to some receivers.

Bursts also belong to an **emitter family** (`Chirp::family`: receiver,
bandwidth class, duration class, FSK tone-spacing class, symbol-rate class,
any center), so a frequency hopper is one family however many channels it
uses. Two budgets keep a busy family (the 915 MHz AMI mesh at the tower does
~20 bursts/s) from owning the store: the receiver thread forwards at most one
example per family per `min_family_interval_s` (default 10 s; this is also
what keeps the writer queue from starving the other receivers), and the store
keeps at most `max_examples_per_family` (default 50) examples per family.
Group counts are unaffected by either.

Each receiver's device is owned by a supervisor thread: if it can't be opened
(unplugged, not yet enumerated) the receiver logs and retries every 30 s while
the others run; a watchdog re-opens a receiver that has been open but silent
for 5 min (an SDR that drops off USB and comes back leaves libairspy/librtlsdr
silently idle — on 2026-09-11 the service sat "active" for four hours decoding
nothing), and exits the process (status 3, `Restart=always`) if re-opening
still yields nothing after 30 min. `GET /health` is `200 ok` when every SDR
receiver is open and delivering, else `503 stalled: …; unavailable: …`.

### RF survey mode

A hopping receiver with a long dwell turns sensormon into a burst-emitter
survey instrument: `deploy/radio-survey-hop-*.toml` hops the NESDR over
24–1700 MHz in 2.4 MHz steps with a 5 s dwell (`hop_hz`, `dwell_s`;
`max_band_pipelines` caps the per-band pipelines kept resident, LRU — an
evicted band re-learns its noise floor in a few ms when revisited). Pair it
with a few `rtl_power` sweeps for the continuous carriers, then:

```
tools/rf-survey/sweep_report.py  sweep_*.csv           # rtl_power: carriers, ATSC pilots, occupancy per band, harmonic combs
tools/rf-survey/hop_survey_report.py --host radio:8433 --sweep sweep_*mean*.csv --hide-continuous
                                                       # chirp store: burst emitters per band; flags flicker of continuous
                                                       # signals (FM modulation, spread-spectrum SMPS dither) and clock combs
tools/rf-survey/chirps_report.py --host radio:8433 --coarse   # merge hoppers, run rtl_433 over examples
```

Caveats learned the hard way: `rtl_power` cycles through all windows many
times per integration interval (~30 ms per visit), so it never sees short
bursts even in peak-hold mode; and the burst detector fires on continuous
noise-like signals whose per-bin power flickers (weak FM stations, dithered
switching supplies) — the report cross-references the sweeps to flag those.

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
- Region opening is variance-aware (CFAR-style, since 2026-09-05): each bin
  tracks the spread of its sub-threshold excursions and a new region needs
  `p > floor · max(open_ratio, 1 + k·σ)` (`variance_k` = 30, `variance_alpha`
  = 0.002). Noise bins (σ ≈ 0.58) are unaffected; flickering bins (weak FM
  stations, dithered SMPS harmonics, 8-VSB) need a proportionally larger
  excursion. On 3 s NESDR captures of the FM/TV/UHF bands the spurious burst
  count went 21/113/101 → 0/27/11 with all 25 corpus frames still decoded;
  detect-stage cost +~15% (≈0.2% of a core per receiver).
- FSK only; no OOK decoders yet.
- No golden-fixture test on the corpus yet (verification is manual
  `decode-file --summary` against the numbers above).
