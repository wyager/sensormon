# Undecoded-emitter research notes (2026-09-04/05)

Source: the production chirp store on radio (`/chirps/*`, both receivers, ~44 h) plus
the survey store. Scripts: `tools/rf-survey/protocols/` (research quality: they fetch
example IQ from the store, demodulate with a clock-locked slicer, and analyse bit/byte
structure). Nothing here is wired into the decoder set yet.

## 1. 915 MHz AMI mesh hopper (tower; ~95% of all tower bursts, ~20 frames/s)

Physical layer (measured on 240 clean frames):

| parameter | value |
|---|---|
| modulation | 2-FSK, tone spacing 46 kHz (±23 kHz) |
| symbol rate | **106.37 kbaud** (crossing-phase coherence 0.9; harmonics at 212.7/319.1 k) |
| channels | > 170 distinct centers across the 913.8–916.2 MHz Airspy window, i.e. the whole 902–928 band; densest 905–911 and 921–926 MHz from the garage |
| preamble | ~20 bits of 0101 |
| sync | 60 bits `39 6B F0 CF D8 76 76 0` (first 7 bytes never vary) |
| payload | exactly 4 + 16·N bits, N even: 6 (55% of frames, ack-like), 28, 30, 34, 36, 40, 42, 44, 46, 52 bytes |
| postamble | 12 bits `0x1D8` (`000111011000`) |

Payload content (60-bit-sync alignment): byte 0 always `6B`; byte 1 takes ~6 values in
pairs differing in the LSB (`86/87`, `9A/9B`, `9C/9D`, `EA/EB`, `F6/F7`, `F0/F1`);
bytes 2–4 unique per frame (24 bits, counter/CRC-like); long frames carry a fixed
block `?D 80 01 C4 58` at bytes 5–9 and share blocks like `76 71 60 01 B4 E7 7B 3F 60`,
plus runs of 20+ zeros, `FF` runs and a 4-byte pattern (`3E BE A3 55`) repeated up to
8×. **Not encrypted, not whitened** (uniform ciphertext cannot contain those), but the
layout is not byte-obvious: fragments like `1D8`/`1C4`/`0007` recur inside payloads at
odd bit offsets, so it is either bit-packed or a line code. No standard CRC-8/16/32
validates any span in either bit order. rtl_433 (ERT SCM/IDM, Neptune R900, Badger
ORION, Landis+Gyr Gridstream 9.6/19.2/38.4k) decodes nothing. Identification needs the
meter vendor (ask the user: brand/model on the electric, gas or water meter). At least
8 distinct nodes seen (byte-1..4 field variety); 6-byte frames are probably acks.

Two examples (hex, after the 60-bit sync):
```
6b87161dea58                                                  (6 B, @915.601)
6beb8767677d8001c45800000077b4e01dec8ce06b807160007671601c5f16001deb9d81afae6b81c34fb4e01d86b9b5497c42e0   (52 B)
```

## 2. 50 kbaud beacons = IEEE 802.15.4g SUN-FSK, proprietary MAC (tower, ~230/h)

Fully decoded at the link layer (11/11 frames CRC-valid):

| layer | value |
|---|---|
| PHY | 2-FSK 50.0 kbaud, Δ≈50 kHz (802.15.4g SUN FSK operating mode 1) |
| preamble | **150 ms** of 0101 (7488 bits — a low-power wake-up preamble) |
| SFD | `0x904E` (uncoded) |
| PHR | mode-switch 0, FCS type 1 (16-bit), data whitening 1, length 32 |
| whitening | PN9 (x⁹+x⁵+1); found by brute force over seed/direction: seed 30 "reversed", equivalent to the standard sequence at another phase |
| FCS | CRC-16 poly 0x1021, init 0, MSB-first, big-endian — validates on all frames |

Dewhitened PSDU (32 bytes) of the two devices seen:
```
04 00 a0 03 09 5d c4 a2 80 f7 2a 00 e0 60 03 09 5d c4 00 4b 04 81 08 32 09 32 | 1f 7a ca 99 | 02 68
04 00 a0 02 8b 57 97 a1 f4 31 e4 00 60 50 02 8b 57 97 00 08 04 81 08 32 09 32 | e7 2c d2 57 | 65 67
```
Frame control `0x0004` is not a valid IEEE 802.15.4 MAC type, so the MAC is proprietary:
a device ID (`03 09 5D C4 …` / `02 8B 57 97 …`, repeated twice in the header), a
sequence counter at byte 19, a constant 6-byte body `04 81 08 32 09 32`, a 4-byte
authentication tag, FCS. The combination (15.4g FSK 50 kbps, long wake-up preamble,
4-byte MIC) matches Amazon Sidewalk's FSK link — best guess: a neighbour's Ring/Echo
class devices. Decoder value: device IDs, sequence numbers, timing only.

## 3. Others

- Unmodulated carrier bursts, 100–250 ms, near 914.8–915.0 MHz, ~12/h at the tower
  (the "sometimes on for seconds" emitter from August).
- Garage 315.0 MHz: ~10 bursts/h (receiver listens 50% of the time), periods 4–12 min
  with some ~40 min, 11–12 ms, narrow FSK (~10 kHz deviation), 14–17 dB. Too weak for
  rtl_433's 315 MHz decoders. Shape says door/tilt sensor or keypad supervision beacon,
  not TPMS. Unidentified; the user suspects it is their own device in the metal garage.
- Everything else in the garage store was the 5.4175 MHz SMPS comb (58th harmonic at
  314.2 MHz, 80th at 433.4 MHz) — 98% of garage bursts before the variance-aware
  detector (2026-09-05), 0.8 bursts/s after.

## Store hygiene learned here

- The 256-slot writer queue starved the second receiver once the store hit its cap
  (garage examples stopped 2026-09-04 16:26Z). Fixed by per-family admission at the
  producer (`min_family_interval_s`) plus a 1024-slot queue.
- `symbol_rate` in group stats is the Fineoffset-lattice fit (≈17 k) and is meaningless
  for other emitters; use the scripts' own estimators.
