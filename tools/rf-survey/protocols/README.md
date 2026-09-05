Research scripts behind `survey-2026-09-03/PROTOCOLS.md`. They read the sensormon chirp
store over HTTP (`--host` is hard-coded to radio in places), demodulate examples with a
tone-centred, clock-locked FSK slicer and dump frame structure. Not maintained tooling.

- `corpus.py hop|long` — collect demodulated frames for the 106.4 kbaud hopper / the
  50 kbaud 802.15.4g beacons into `hop.json` / `long.json` (needs `../groups_now.json`
  = `/chirps/groups?min_count=1`).
- `redemod.py hop` — re-demodulate cached IQ with the coherence rate estimate + PLL
  slicer (`hop2.json`).
- `an_hop.py`, `hstruct.py` — hopper byte/bit structure, entropy, CRC hunt.
- `an_154g.py` — 802.15.4g PHR parse, PN9 dewhitening, CRC-16 validation, MAC dump.
