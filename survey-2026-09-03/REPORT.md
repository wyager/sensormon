# RF environment survey — 2026-09-03 (overnight)

Two instruments, two receivers:

| | receiver | antenna | method | good for |
|---|---|---|---|---|
| A | NESDR (garage) | garage antenna, no filter | `rtl_power` 24–1700 MHz, 10 kHz bins, 1 h mean ×2 + 1 h peak-hold ×1 | continuous carriers, occupancy |
| B | NESDR (garage) | same | sensormon hopping 24–1700 MHz in 2.4 MHz steps, **5 s dwell**, every burst catalogued in the chirp store (`/var/lib/sensormon/chirps-survey.db`) | bursty emitters 3–250 ms |
| C | Airspy R2 (tower) | big antenna + 915 filter + arrestor | sensormon at 915 MHz, chirp store of everything undecoded | what shares the ISM band with our sensors |

Raw data: `sweep_*.csv` (rtl_power), `mean12.md` / `peak1.md` (sweep reports), `hop_pass*.md` (burst-emitter reports), `tower_chirps_coarse.md`. Tools: `../tools/rf-survey/`.

## TL;DR

1. **The garage NESDR's spectrum is dominated by nearby electronics, not by radio.** Three harmonic combs account for most of the strongest lines from 40 MHz to 1.6 GHz:
   - **148.5 MHz × 1…11** (148.496, 445.49, 594.0, 742.5, 891.0, 1039.5, 1336.5, 1485.0, 1633.5 MHz; ±66 kHz sidebands). 148.5 MHz is the HDMI 1080p60 pixel clock: a monitor/cable/mini-PC video output near the NESDR or its coax.
   - **5.4175 MHz × 8…150** (spread-spectrum dithered: each harmonic is a 200–400 kHz wide hump whose energy flickers). Reaches at least 812 MHz; strong at 157.0, 162.5, 173.3, 184.1, 189.5, 195.0, 200.3, 205.7, 211.1, 216.5, 243.6, 254.4, 259.8, 270.8, 287.0, 297.6, 303.2, 314.1, 324.9, 330.3, 340.9, 346.5, 357.4, 394.1, 399.5 MHz. A switching regulator with dithering (mini-PC, PoE injector, LED driver, EV charger, USB hub...).
   - **12.5 MHz × n** (250, 500, 812.5, 1000, 1250, 1625 MHz): a 12.5/25 MHz reference clock (USB/PCIe/Ethernet PHY).
   Several things the first pass of the sweep report labelled as "NOAA weather", "marine", "UHF satcom", "radar" are these harmonics. The 148.5 comb is at −38 dB above floor at the fundamental — 20 dB stronger than any real signal.
2. **Real over-the-air continuous signals** (mean sweep, both hours): ATSC pilots for TV channels 7 (174.31), 19 (500.31), 21 (512.31), 22 (518.31), 23 (524.31), 33 (584.31) and the 6 MHz-wide channels around them; NOAA weather at 162.40–162.55; LTE/5G downlinks at 763 (band 13/14), 781, 806–815 (public safety 800), 837/843 (band 5 uplink from nearby phones), 859, 889 MHz; FM broadcast 93.7 / 95.5 / 96.7 / 103.5 / 104.2 / 107.2 (all weak, ≤6 dB over floor — this is a rural FM environment); a few narrowband VHF-low carriers at 42.88 / 43.11 / 58.19 MHz; radar-band line at 1328.1 and aeronautical 1105.5. Nothing in 24–30 MHz (HF) or 50–54 MHz.
3. **Occupancy is low everywhere.** No band has more than 7% of its 5 kHz bins above threshold in the hour-long mean; the "58% NOAA" and "81% (peak-hold) NOAA" figures in the auto-generated tables are the 5.4175 MHz comb's 30th harmonic sitting on 162.5 MHz plus the real NOAA transmitter. The 902–928 MHz ISM band shows **0 of 4757 bins** active in every rtl_power sweep — see §Bursts for why that is a measurement artifact, not silence.
4. **Bursty emitters (5 s dwell survey):** after removing FM-modulation flicker and comb harmonics, the real burst emitters the garage NESDR can hear are all in the ISM bands and all already known: the 915 MHz meter hopper (also the dominant undecoded emitter at the tower), Fineoffset sensors, TPMS at 315/433. Everything else the detector flagged above ~900 MHz is broadband impulsive noise (≈4 ms, 0.3–0.9 MHz wide, 13–16 dB, no envelope structure), most likely from the same switching supplies. Signals shorter than 3 ms (ADS-B 1090, DME, keyfob preambles) are below the detector's minimum duration and are not surveyed.
5. **rtl_power cannot survey bursts at all.** With `-i 3600` it cycles all 599 windows continuously (~30 ms per visit) and averages; a 20 ms transmission every 9 s is caught with ~0.1% probability per visit even in peak-hold. Our WS90 (40 dB SNR on this very receiver) is invisible in all three sweeps. sensormon with a long dwell is the right instrument; that is what phase B does.

## A. Continuous carriers (rtl_power)

Full tables: `mean12.md` (two 1 h mean sweeps combined; "median" = present in both) and `peak1.md` (1 h peak-hold). Level = dB above the local floor (20th percentile within ±1 MHz).

Strongest lines that are **not** comb harmonics:

| MHz | level dB | what |
|---|---|---|
| 148.496 | 38 | HDMI pixel clock (fundamental of comb 1) |
| 584.31 (584.1–584.8) | 33 | ATSC pilot + channel 33 |
| 512.31 / 518.31 / 524.31 / 500.31 | 25 / 23 / 21 / 17 | ATSC pilots ch 21 / 22 / 23 / 19 |
| 174.31 | 19 | ATSC pilot ch 7 |
| 162.40–162.51 | 17 | NOAA weather radio (real; the 162.5 comb line sits on top) |
| 763.2, 781.2 | 16, 18 | 700 MHz LTE downlink |
| 806.3, 809.4, 815.6 | 16–17 | 800 MHz public safety / SMR |
| 837.1, 843.3 | 15, 18 | cellular 850 uplink (handsets nearby) |
| 859.4, 889.3 | 17, 17 | 800 SMR / cellular 850 downlink |
| 42.88, 43.11, 58.19 | 16–17 | narrowband VHF-low carriers (business/paging-style) |
| 660.0 | 17 | exactly 660.000: a clock, not a broadcaster |
| 1105.5, 1328.1 | 16, 19 | L-band lines (aeronautical / radar band; both narrow, possibly local clocks) |

Occupancy by allocation band (mean sweeps, fraction of 5 kHz bins ≥ 8 dB over floor): everything ≤ 7%; HF 0%, 6 m 0%, 2 m 0%, 400–420 0%, 902–928 0% (artifact, see B), 1400–1700 ≤ 0.3%. The "busy" bands are UHF TV 470–608 (7%), 70 cm 420–450 (7%, mostly the 445.49 comb line ±), 600 MHz (7%), 225–400 (5%, comb).

## B. Burst emitters (sensormon hop survey)

Setup: `deploy/radio-survey-hop-2026-09-03.toml` — 698 centers 25.2…1698 MHz, 2.4 MS/s, 5 s dwell (58 min per pass), `[chirps] receivers=["survey-nesdr"]`, 2 GB cap. Passes completed: 2 (10:10–12:07Z), 73,955 bursts in 63,904 groups, 1.23 GB of examples; the survey store is kept at `/var/lib/sensormon/chirps-survey.db` on radio for later digging (the NESDR was returned to its 315/433 duty at 12:10Z). Report: `hop_final.md` (`hop_survey_report.py --sweep ... --hide-continuous`).

What the detector saw: ≈37k bursts per pass in ≈32k new groups (a "group" is receiver × center to 10 kHz × bandwidth class × duration class). Nearly all groups are singletons: unique-frequency events. Breaking them down:

| class | where | what it actually is |
|---|---|---|
| FM-station flicker | 93.7, 95.5, 96.7, 103.5 MHz: 25–75 kHz wide, 5–10 ms, "FSK Δ30–60 kHz", 0.1–2 s apart, 15–19 dB | weak FM broadcast stations (3–6 dB continuous): ±75 kHz deviation makes individual bins flicker above the slowly-tracked floor |
| comb flicker | every 5.4175 MHz harmonic 119–480 MHz (and 96.7/95.5 coincide with FM), 25–100 kHz, 5–10 ms | spread-spectrum dither of the switching-supply comb; the sweep shows 3–6 dB continuous at each |
| broadband impulses | 942–1700 MHz: ≈50k singleton bursts over two passes, median 4 ms, 0.3–0.9 MHz wide, 13–16 dB, 1–6 simultaneous within 20 ms | noise-like (no envelope, spectral peakiness ≈ Gaussian). Impulsive/SMPS noise at the receiver, not transmissions |
| real burst emitters | 902–928 (meter hopper, Fineoffset), 315 / 433.92 (TPMS, remotes) | known ISM users — details below |

The detector fires on the first two classes because it keys on per-bin power over a slowly tracked floor and has no test for envelope contrast; a variance-aware (CFAR) floor would suppress both. Not changed tonight (sensitivity change — wants sign-off); the survey report works around it by cross-referencing the sweeps.

### Real burst emitters heard from the garage (all passes)

| band | what | evidence |
|---|---|---|
| 902–928 | **the AMI meter hopper** (same family as §C) | 547 bursts over two passes (≈5/s of dwell): 3–5 ms, 75–250 kHz wide, 14 dB median SNR, symbol-rate estimate 17–18k on every burst (identical to the tower's estimate), landing on unique centers 902.5–926 MHz (densest 905–911 and 921–926); ≈20/s at the tower. Stored examples show a clean 8 dB envelope step. |
| 902–928 | Fineoffset WS90 / WH51 / WH55 / WH31L | decoded (so absent from the chirp store by construction); the garage NESDR decodes the WS90 at ~40 dB SNR when parked on 915 MHz |
| 433.92 | TPMS / remotes | 4 bursts in two passes (night: cars parked); FSK Δ36–45 kHz, 17k symbol estimate |
| 315 | nothing real | the 300 "bursts" at 313.7–314.2 MHz are the 58th harmonic of the 5.4175 MHz comb (continuous, no envelope, noise-like spectrum); 315 MHz TPMS/keyfobs did not transmit during the dwells |
| 462–467 FRS/GMRS, MURS, 345 security, 868 | nothing | 465.5–466.0 MHz bursts = 86th comb harmonic; 0 bursts in the others |

Everything else in the store — including the ~27k singleton bursts from 942 MHz up — fails the envelope test (flat power through the 2 ms pre/post padding, spectral peakiness of Gaussian noise).

## C. What shares 915 MHz with our sensors (tower Airspy chirp store)

`tower_chirps_coarse.md`: 2,645 example captures / 13,720 groups covering only **98 s** — one emitter family fills a 50 MB store in under two minutes.

- **One frequency-hopping family** owns the band: constant-envelope 2-FSK bursts, two lengths (**≈1.6 ms** and **≈3–5 ms**), ≈200 kHz occupied bandwidth, landing on >170 distinct centers across the whole 913.8–916.2 MHz Airspy window (so certainly across all of 902–928), ≈20 bursts/s, 20–27 dB SNR at the tower. The 1.6 ms bursts are single unmodulated-looking pulses to rtl_433's analyzer; the longer ones carry ≈100 kbaud data. rtl_433's ERT (SCM/IDM), Neptune R900, Badger ORION and Landis+Gyr Gridstream (9.6/19.2/38.4k) decoders all decode nothing from stored examples. Short, fast, hopping, dense: this looks like an **AMI mesh (Itron/Silver Spring style) network** rather than one-way meter bubble-ups. Decoding it would be a project of its own (no open decoder exists).
- Everything else at 915 (WS90 ×2, WH51 ×5, WH55 ×3, WH31L) decodes; the hopper's bursts are short enough that they rarely collide with a 100 ms Fineoffset frame, and the per-burst extraction means they never capture the demodulator the way they did in rtl_433.
- Consequence for the chirp backlog: a 50 MB store at the tower is a ~2-minute window. Options (not done): per-family example caps (a hopper's 170 centers collapse to one family), a larger cap, or excluding the hopper's burst class from recording.

## D. Recommendations

1. **Clean up the garage receiver's environment** before trusting it for weak-signal work: find the 148.5 MHz HDMI source (unplug/replace the HDMI cable or monitor on the mini-PC; shielded cable + ferrite) and the 5.4 MHz dithered SMPS (unplug candidates one at a time while watching `rtl_power -f 240M:260M:10k -i 5` — the 45th–48th harmonics at 243.6/254.4/259.8 MHz are a convenient tell). Same drill as the 200 kHz comb hunt in the tower box.
2. For the tower box comb hunt, `make sdr-noise` remains the tool; nothing in tonight's data changes that plan.
3. For sensormon: a CFAR/variance-aware floor (or an envelope-contrast test before storing a chirp) would remove the two flicker classes and the impulsive class in one change, and would make the chirp store a usable backlog at the tower. Estimated effort: a day; wants your sign-off since it changes sensitivity.
4. Detector minimum duration (3 ms) hides ADS-B/DME/keyfob-class signals; lowering it is cheap if we ever want them, but the false-alarm rate above will get worse first.
