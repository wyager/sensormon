# Burst-emitter survey — receiver `survey-nesdr` (10.20.15.5:8433)

Store: 38367 example captures, 35393 raw groups (35393 for this receiver), 650.3 MB; examples span 1.2 h. Bursts are 3–250 ms events ≥9 dB over the local floor; continuous carriers are invisible here (see the rtl_power sweep report).
'sweep dB' = continuous level at that frequency in the rtl_power mean sweeps; ≥6 dB means the 'bursts' are a continuous wideband signal flickering (FM modulation, spread-spectrum SMPS dither, TV) rather than a burst emitter — 790 such emitters hidden.

## Harmonic combs (switching-supply / clock harmonics; not radio emitters)

- **10.8322 MHz** fundamental explains 113 emitters (harmonics 11…137, i.e. 119.2–1484.0 MHz)
- **26.0980 MHz** fundamental explains 65 emitters (harmonics 8…60, i.e. 208.8–1565.9 MHz)
- **17.8641 MHz** fundamental explains 60 emitters (harmonics 12…86, i.e. 214.4–1536.3 MHz)

## Top 40 emitters overall

| band | MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|---|
| FM broadcast | 96.64–96.79 (4 ch) | 50 | 10 | y | 40 | — | 100 | 18 | 0.3 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.64–96.78 (4 ch) | 25 | 10 | y | 50 | — | 90 | 17 | 0.3 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.770 | 25 | 5 | y | 70 | — | 66 | 16 | 0.2 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.63–96.76 (3 ch) | 25 | 5 | y | 40 | — | 62 | 16 | 977.2 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 95.450 | 50 | 10 | y | 50 | — | 52 | 15 | 0.1 | 6 | FM station 95.5 (modulation flicker) |
| FM broadcast | 96.63–96.75 (2 ch) | 25 | 10 | y | 30 | — | 42 | 16 | 0.1 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.770 | 50 | 5 | y | 60 | — | 41 | 18 | — | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 95.56–95.57 (2 ch) | 50 | 5 | y | 110 | — | 38 | 15 | 0.7 | 3 | FM station 95.5 (modulation flicker) |
| FM broadcast | 96.71–96.77 (2 ch) | 50 | 10 | y | 60 | — | 37 | 18 | 0.2 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 103.54–104.35 (4 ch) | 50 | 10 | y | 30 | — | 37 | 16 | 0.2 | 6 | FM station 103.5 (modulation flicker) |
| FM broadcast | 93.63–93.78 (6 ch) | 50 | 5 | y | 30 | — | 26 | 15 | 0.8 | 5 | FM station 93.7 (modulation flicker) |
| FM broadcast | 95.52–95.57 (4 ch) | 75 | 10 | y | 40 | — | 25 | 15 | 2.0 | 4 | FM station 95.5 (modulation flicker) |
| FM broadcast | 103.54–104.36 (4 ch) | 25 | 5 | y | 40 | — | 25 | 16 | 1.3 | 6 | FM station 103.5 (modulation flicker) |
| FM broadcast | 96.64–96.76 (3 ch) | 75 | 10 | y | 30 | — | 23 | 19 | 1.9 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 95.53–95.57 (3 ch) | 50 | 10 | y | 30 | — | 20 | 15 | 0.4 | 4 | FM station 95.5 (modulation flicker) |
| aviation (AM) | 119.07–119.17 (7 ch) | 25 | 5 | y | 30 | — | 20 | 14 | 1.6 | 1 | 11× 10.8322 MHz comb |
| VHF TV ch 7-13 | 205.66–206.42 (8 ch) | 50 | 5 | y | 30 | — | 18 | 15 | 2.5 | 1 | 19× 10.8322 MHz comb |
| VHF TV ch 7-13 | 205.65–205.89 (11 ch) | 50 | 10 | y | 30 | — | 17 | 15 | 976.6 | 6 | 19× 10.8322 MHz comb |
| VHF TV ch 7-13 | 205.65–205.87 (11 ch) | 75 | 5 | y | 30 | — | 15 | 15 | 2.0 | 5 | 19× 10.8322 MHz comb |
| FM broadcast | 96.780 | 25 | 5 | y | 60 | — | 14 | 17 | 0.5 | 3 | FM station 96.7 (modulation flicker) |
| VHF TV ch 7-13 | 205.64–205.89 (8 ch) | 75 | 10 | y | 30 | — | 14 | 16 | 1.2 | 2 | 19× 10.8322 MHz comb |
| VHF TV ch 7-13 | 194.79–195.05 (9 ch) | 75 | 5 | y | 30 | — | 13 | 15 | 974.8 | 6 | 18× 10.8322 MHz comb |
| military aviation / satcom (UHF) | 281.52–281.71 (7 ch) | 75 | 5 | y | 40 | — | 12 | 16 | 1.5 | 5 | 26× 10.8322 MHz comb |
| VHF TV ch 7-13 | 205.71–205.85 (7 ch) | 50 | 5 | y | 40 | — | 12 | 15 | — | 2 | 19× 10.8322 MHz comb |
| military aviation / satcom (UHF) | 286.71–287.16 (9 ch) | 100 | 10 | y | 30 | — | 12 | 16 | — | 3 | 11× 26.0980 MHz comb |
| FM broadcast | 95.560 | 25 | 5 | y | 40 | — | 11 | 15 | 0.2 | 4 | FM station 95.5 (modulation flicker) |
| FM broadcast | 96.61–96.77 (2 ch) | 75 | 10 | y | 50 | — | 11 | 18 | 1.8 | 3 | FM station 96.7 (modulation flicker) |
| VHF TV ch 7-13 | 194.88–195.05 (5 ch) | 50 | 5 | y | 40 | — | 11 | 15 | 3.6 | 3 | 18× 10.8322 MHz comb |
| FM broadcast | 93.71–93.76 (2 ch) | 50 | 5 | y | 40 | — | 10 | 15 | 1.1 | 5 | FM station 93.7 (modulation flicker) |
| FM broadcast | 96.65–96.79 (2 ch) | 50 | 5 | y | 50 | — | 10 | 17 | 975.8 | -2 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.63–96.78 (4 ch) | 75 | 5 | y | 30 | — | 9 | 19 | 0.6 | 3 | FM station 96.7 (modulation flicker) |
| military aviation / satcom (UHF) | 281.61–281.70 (4 ch) | 75 | 5 | y | 30 | — | 9 | 15 | 976.3 | 4 | 26× 10.8322 MHz comb |
| VHF TV ch 7-13 | 205.61–205.81 (6 ch) | 100 | 10 | y | 30 | — | 9 | 16 | 975.5 | 6 | 19× 10.8322 MHz comb |
| VHF TV ch 7-13 | 213.53–214.50 (6 ch) | 50 | 5 | y | 30 | — | 9 | 15 | 0.6 | -0 |  |
| VHF TV ch 7-13 | 211.13–211.35 (6 ch) | 100 | 10 | y | 30 | — | 9 | 16 | 1.2 | 5 |  |
| military aviation / satcom (UHF) | 286.72–287.12 (7 ch) | 75 | 5 | y | 40 | — | 9 | 16 | — | 4 | 11× 26.0980 MHz comb |
| FM broadcast | 96.62–96.78 (2 ch) | 50 | 5 | y | 30 | — | 8 | 18 | 2.0 | 3 | FM station 96.7 (modulation flicker) |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 1000.40–1000.43 (3 ch) | 750 | 10 | n | — | — | 8 | 18 | 0.4 | -1 | 56× 17.8641 MHz comb |
| FM broadcast | 103.56–104.36 (3 ch) | 50 | 20 | y | 30 | — | 8 | 16 | 1.4 | 6 | FM station 103.5 (modulation flicker) |
| VHF TV ch 7-13 | 208.83–209.08 (5 ch) | 50 | 5 | y | 40 | — | 8 | 14 | 1.8 | 3 | 8× 26.0980 MHz comb |

## Wide families (same burst class in ≥5 distinct 1 MHz cells of a band: hoppers, or a band-wide artifact)

| band | MHz range | cells | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR |
|---|---|---|---|---|---|---|---|---|---|
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 970.72–1124.27 | 12 | 725 | 5 | n | — | — | 44 | 16 |
| VHF TV ch 7-13 | 191.96–215.24 | 6 | 50 | 5 | y | 40 | — | 43 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 961.66–1151.45 | 8 | 1100 | 5 | n | — | — | 32 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 979.77–1119.25 | 8 | 1025 | 5 | n | — | — | 27 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 965.75–1133.42 | 7 | 950 | 5 | n | — | — | 26 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 976.08–1111.89 | 7 | 1150 | 5 | n | — | — | 25 | 17 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 972.90–1122.75 | 8 | 875 | 5 | n | — | — | 25 | 16 |
| military aviation / satcom (UHF) | 254.60–368.04 | 5 | 125 | 10 | y | 40 | — | 24 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 968.65–1112.26 | 7 | 800 | 5 | n | — | — | 23 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 962.85–1115.26 | 7 | 625 | 5 | n | — | — | 22 | 15 |
| AWS-3/L-band / fixed | 1429.58–1483.71 | 6 | 725 | 5 | n | — | — | 19 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 986.01–1130.15 | 6 | 500 | 5 | n | — | — | 18 | 14 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 961.99–1010.07 | 6 | 550 | 5 | n | — | — | 18 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 982.18–1104.61 | 5 | 1175 | 5 | n | — | — | 16 | 18 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 979.50–1122.90 | 5 | 575 | 5 | n | — | — | 16 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 966.63–1115.45 | 5 | 675 | 5 | n | — | — | 15 | 15 |
| AWS-3/L-band / fixed | 1429.57–1467.43 | 5 | 525 | 5 | n | — | — | 15 | 14 |
| AWS-3/L-band / fixed | 1471.69–1516.36 | 5 | 650 | 5 | n | — | — | 15 | 15 |

## Per band

### VHF TV ch 2-6 (mostly vacant) (54–88 MHz): 6 bursts, 2 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 86.500 | 25 | 5 | y | 60 | — | 3 | 14 | 1.8 | 4 |  |
| 87.060 | 25 | 5 | y | 100 | — | 3 | 13 | 0.9 | 4 |  |

### FM broadcast (88–108 MHz): 846 bursts, 43 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 96.64–96.79 (4 ch) | 50 | 10 | y | 40 | — | 100 | 18 | 0.3 | 3 | FM station 96.7 (modulation flicker) |
| 96.64–96.78 (4 ch) | 25 | 10 | y | 50 | — | 90 | 17 | 0.3 | 3 | FM station 96.7 (modulation flicker) |
| 96.770 | 25 | 5 | y | 70 | — | 66 | 16 | 0.2 | 3 | FM station 96.7 (modulation flicker) |
| 96.63–96.76 (3 ch) | 25 | 5 | y | 40 | — | 62 | 16 | 977.2 | 3 | FM station 96.7 (modulation flicker) |
| 95.450 | 50 | 10 | y | 50 | — | 52 | 15 | 0.1 | 6 | FM station 95.5 (modulation flicker) |
| 96.63–96.75 (2 ch) | 25 | 10 | y | 30 | — | 42 | 16 | 0.1 | 3 | FM station 96.7 (modulation flicker) |
| 96.770 | 50 | 5 | y | 60 | — | 41 | 18 | — | 3 | FM station 96.7 (modulation flicker) |
| 95.56–95.57 (2 ch) | 50 | 5 | y | 110 | — | 38 | 15 | 0.7 | 3 | FM station 95.5 (modulation flicker) |
| 96.71–96.77 (2 ch) | 50 | 10 | y | 60 | — | 37 | 18 | 0.2 | 3 | FM station 96.7 (modulation flicker) |
| 103.54–104.35 (4 ch) | 50 | 10 | y | 30 | — | 37 | 16 | 0.2 | 6 | FM station 103.5 (modulation flicker) |
| 93.63–93.78 (6 ch) | 50 | 5 | y | 30 | — | 26 | 15 | 0.8 | 5 | FM station 93.7 (modulation flicker) |
| 95.52–95.57 (4 ch) | 75 | 10 | y | 40 | — | 25 | 15 | 2.0 | 4 | FM station 95.5 (modulation flicker) |

### aviation (AM) (108–137 MHz): 78 bursts, 15 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 119.07–119.17 (7 ch) | 25 | 5 | y | 30 | — | 20 | 14 | 1.6 | 1 | 11× 10.8322 MHz comb |
| 119.06–119.09 (2 ch) | 25 | 10 | y | 50 | — | 6 | 15 | 1.3 | 3 | 11× 10.8322 MHz comb |
| 119.07–119.14 (4 ch) | 25 | 10 | y | 40 | — | 6 | 15 | — | 3 | 11× 10.8322 MHz comb |
| 119.08–119.17 (4 ch) | 50 | 10 | y | 30 | — | 6 | 15 | — | -1 | 11× 10.8322 MHz comb |
| 119.04–119.10 (4 ch) | 25 | 5 | y | 40 | — | 5 | 15 | — | 3 | 11× 10.8322 MHz comb |
| 120.420 | 25 | 10 | y | 100 | — | 4 | 14 | 0.4 | 4 |  |
| 119.06–119.07 (2 ch) | 50 | 10 | y | 140 | — | 4 | 15 | 1.4 | 2 | 11× 10.8322 MHz comb |
| 119.09–119.18 (2 ch) | 25 | 5 | y | 140 | — | 4 | 15 | — | -3 | 11× 10.8322 MHz comb |
| 120.84–120.98 (3 ch) | 25 | 10 | y | 130 | — | 4 | 14 | — | 5 |  |
| 119.08–119.16 (4 ch) | 50 | 5 | y | 30 | — | 4 | 13 | — | 2 | 11× 10.8322 MHz comb |
| 119.060 | 25 | 5 | y | 50 | — | 3 | 15 | 2.1 | 2 | 11× 10.8322 MHz comb |
| 119.13–119.15 (2 ch) | 25 | 10 | y | 30 | — | 3 | 14 | — | -0 | 11× 10.8322 MHz comb |

### VHF business / public safety / wireless mics (162.6–174 MHz): 18 bursts, 5 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 173.37–173.40 (3 ch) | 125 | 20 | y | 40 | — | 6 | 19 | 0.3 | 6 | 16× 10.8322 MHz comb |
| 167.79–167.94 (2 ch) | 100 | 10 | y | 30 | — | 3 | 14 | — | 3 |  |
| 173.17–173.43 (2 ch) | 175 | 20 | y | 30 | — | 3 | 17 | — | 3 | 16× 10.8322 MHz comb |
| 165.07–165.20 (3 ch) | 25 | 5 | y | 30 | — | 3 | 13 | — | 5 |  |
| 166.05–166.22 (3 ch) | 75 | 5 | y | 30 | — | 3 | 15 | — | 6 |  |

### VHF TV ch 7-13 (174–216 MHz): 331 bursts, 56 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 205.66–206.42 (8 ch) | 50 | 5 | y | 30 | — | 18 | 15 | 2.5 | 1 | 19× 10.8322 MHz comb |
| 205.65–205.89 (11 ch) | 50 | 10 | y | 30 | — | 17 | 15 | 976.6 | 6 | 19× 10.8322 MHz comb |
| 205.65–205.87 (11 ch) | 75 | 5 | y | 30 | — | 15 | 15 | 2.0 | 5 | 19× 10.8322 MHz comb |
| 205.64–205.89 (8 ch) | 75 | 10 | y | 30 | — | 14 | 16 | 1.2 | 2 | 19× 10.8322 MHz comb |
| 194.79–195.05 (9 ch) | 75 | 5 | y | 30 | — | 13 | 15 | 974.8 | 6 | 18× 10.8322 MHz comb |
| 205.71–205.85 (7 ch) | 50 | 5 | y | 40 | — | 12 | 15 | — | 2 | 19× 10.8322 MHz comb |
| 194.88–195.05 (5 ch) | 50 | 5 | y | 40 | — | 11 | 15 | 3.6 | 3 | 18× 10.8322 MHz comb |
| 205.61–205.81 (6 ch) | 100 | 10 | y | 30 | — | 9 | 16 | 975.5 | 6 | 19× 10.8322 MHz comb |
| 213.53–214.50 (6 ch) | 50 | 5 | y | 30 | — | 9 | 15 | 0.6 | -0 |  |
| 211.13–211.35 (6 ch) | 100 | 10 | y | 30 | — | 9 | 16 | 1.2 | 5 |  |
| 208.83–209.08 (5 ch) | 50 | 5 | y | 40 | — | 8 | 14 | 1.8 | 3 | 8× 26.0980 MHz comb |
| 200.25–200.48 (4 ch) | 100 | 10 | y | 40 | — | 8 | 17 | 977.1 | 2 |  |

### military aviation / satcom (UHF) (225–400 MHz): 405 bursts, 101 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 281.52–281.71 (7 ch) | 75 | 5 | y | 40 | — | 12 | 16 | 1.5 | 5 | 26× 10.8322 MHz comb |
| 286.71–287.16 (9 ch) | 100 | 10 | y | 30 | — | 12 | 16 | — | 3 | 11× 26.0980 MHz comb |
| 281.61–281.70 (4 ch) | 75 | 5 | y | 30 | — | 9 | 15 | 976.3 | 4 | 26× 10.8322 MHz comb |
| 286.72–287.12 (7 ch) | 75 | 5 | y | 40 | — | 9 | 16 | — | 4 | 11× 26.0980 MHz comb |
| 281.64–281.73 (5 ch) | 100 | 5 | y | 40 | — | 8 | 15 | 2.6 | 4 | 26× 10.8322 MHz comb |
| 281.65–281.70 (5 ch) | 100 | 10 | y | 30 | — | 8 | 16 | 2.1 | 5 | 26× 10.8322 MHz comb |
| 254.60–254.66 (4 ch) | 125 | 10 | y | 40 | — | 7 | 16 | 1.1 | 5 |  |
| 281.60–281.70 (4 ch) | 50 | 5 | y | 50 | — | 7 | 14 | 980.5 | 4 | 26× 10.8322 MHz comb |
| 286.82–287.12 (6 ch) | 125 | 10 | y | 40 | — | 7 | 17 | — | 4 | 11× 26.0980 MHz comb |
| 281.44–281.49 (3 ch) | 50 | 5 | y | 30 | — | 6 | 15 | 3.9 | 5 | 26× 10.8322 MHz comb |
| 281.61–281.68 (4 ch) | 50 | 5 | y | 30 | — | 6 | 14 | — | 5 | 26× 10.8322 MHz comb |
| 367.95–368.36 (5 ch) | 125 | 5 | y | 30 | — | 6 | 16 | — | 3 | 34× 10.8322 MHz comb |

### federal (406–420 MHz): 11 bursts, 3 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 410.04–410.05 (2 ch) | 200 | 5 | y | 50 | — | 4 | 17 | — | 1 |  |
| 409.95–410.00 (3 ch) | 175 | 5 | y | 30 | — | 4 | 15 | — | 1 |  |
| 410.08–410.32 (3 ch) | 125 | 5 | y | 30 | — | 3 | 16 | — | 1 |  |

### 70 cm ham (incl. 433.92 ISM part 15) (420–450 MHz): 23 bursts, 7 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 449.04–449.10 (3 ch) | 250 | 5 | y | 40 | — | 4 | 18 | — | 3 |  |
| 449.13–449.20 (4 ch) | 150 | 5 | y | 40 | — | 4 | 14 | — | 4 |  |
| 449.19–449.27 (2 ch) | 175 | 5 | y | 50 | — | 3 | 18 | — | 3 |  |
| 449.14–449.17 (2 ch) | 175 | 5 | y | 40 | — | 3 | 17 | — | 4 |  |
| 421.04–421.08 (2 ch) | 200 | 10 | y | 40 | — | 3 | 16 | — | 4 |  |
| 420.77–421.11 (3 ch) | 125 | 5 | y | 30 | — | 3 | 14 | — | 5 |  |
| 449.57–449.70 (3 ch) | 250 | 10 | y | 50 | — | 3 | 16 | — | 5 |  |

### UHF business / public safety / FRS-GMRS 462-467 (450–470 MHz): 3 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 459.79–460.39 (3 ch) | 200 | 10 | y | 30 | — | 3 | 15 | — | 4 |  |

### UHF TV ch 14-36 / wireless mics (470–608 MHz): 18 bursts, 6 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 482.240 | 200 | 10 | y | 160 | — | 3 | 14 | 979.5 | 4 | 27× 17.8641 MHz comb |
| 482.100 | 475 | 20 | y | 30 | — | 3 | 17 | 979.2 | 5 | 27× 17.8641 MHz comb |
| 482.14–482.16 (2 ch) | 325 | 20 | y | 100 | — | 3 | 17 | — | 5 | 27× 17.8641 MHz comb |
| 482.18–482.19 (2 ch) | 300 | 20 | n | — | — | 3 | 16 | — | 4 | 27× 17.8641 MHz comb |
| 606.71–606.85 (2 ch) | 200 | 5 | y | 60 | — | 3 | 14 | — | 2 | 56× 10.8322 MHz comb |
| 476.69–476.79 (3 ch) | 150 | 5 | y | 40 | — | 3 | 14 | — | 6 | 44× 10.8322 MHz comb |

### 600 MHz cellular (band 71) / TV ch 38-51 (614–698 MHz): 3 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 692.55–693.14 (3 ch) | 175 | 5 | y | 40 | — | 3 | 13 | — | -1 |  |

### 700 MHz cellular (bands 12/13/14/17) / FirstNet (698–806 MHz): 3 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 803.39–803.42 (3 ch) | 1150 | 20 | n | — | — | 3 | 17 | — | 1 | 45× 17.8641 MHz comb |

### cellular 850 downlink (band 5) (869–894 MHz): 14 bursts, 3 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 875.52–875.55 (4 ch) | 900 | 5 | y | 40 | — | 5 | 15 | — | 1 | 49× 17.8641 MHz comb |
| 875.50–875.53 (4 ch) | 875 | 5 | y | 40 | — | 5 | 15 | — | 1 | 49× 17.8641 MHz comb |
| 875.52–875.53 (2 ch) | 950 | 5 | y | 40 | — | 4 | 15 | — | 1 | 49× 17.8641 MHz comb |

### 902-928 ISM: LoRa, Z-Wave 908/916, meters, Fineoffset 915 (902–928 MHz): 9 bursts, 3 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 923.50–924.27 (3 ch) | 550 | 5 | n | — | — | 3 | 14 | — | 1 |  |
| 923.09–923.41 (3 ch) | 225 | 5 | y | 110 | — | 3 | 13 | — | 1 |  |
| 925.56–925.97 (3 ch) | 525 | 5 | n | — | — | 3 | 13 | — | 3 |  |

### paging (929-932) / MAS / narrowband PCS (928–942 MHz): 3 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 940.88–941.37 (2 ch) | 150 | 5 | y | 50 | — | 3 | 13 | — | 1 |  |

### fixed / GSM-R-ish (942–960 MHz): 135 bursts, 41 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 944.93–945.43 (6 ch) | 350 | 5 | n | — | — | 6 | 15 | — | 6 |  |
| 948.66–949.14 (5 ch) | 1000 | 5 | n | — | — | 5 | 15 | — | 1 |  |
| 950.230 | 325 | 5 | y | 120 | — | 4 | 15 | 0.5 | -1 |  |
| 950.230 | 350 | 5 | y | 50 | — | 4 | 17 | 1.0 | -1 |  |
| 957.82–957.94 (3 ch) | 400 | 5 | n | — | — | 4 | 16 | — | 3 |  |
| 946.89–947.38 (4 ch) | 1175 | 5 | n | — | — | 4 | 15 | — | 1 | 53× 17.8641 MHz comb |
| 952.30–952.48 (4 ch) | 600 | 10 | y | 60 | — | 4 | 15 | — | 3 |  |
| 954.15–954.28 (4 ch) | 425 | 5 | y | 150 | — | 4 | 14 | — | 1 |  |
| 959.00–959.40 (4 ch) | 1100 | 5 | n | — | — | 4 | 15 | — | 3 |  |
| 950.230 | 350 | 10 | y | 140 | — | 3 | 17 | 1.0 | -1 |  |
| 955.760 | 1125 | 5 | y | 60 | — | 3 | 18 | 1.7 | -2 |  |
| 946.72–947.29 (2 ch) | 775 | 5 | n | — | — | 3 | 14 | — | -2 | 53× 17.8641 MHz comb |

### aeronautical navigation (DME/TACAN/ADS-B 1090) (960–1215 MHz): 935 bursts, 272 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1000.40–1000.43 (3 ch) | 750 | 10 | n | — | — | 8 | 18 | 0.4 | -1 | 56× 17.8641 MHz comb |
| 1000.40–1000.44 (5 ch) | 725 | 5 | n | — | — | 8 | 17 | 1.4 | -1 | 56× 17.8641 MHz comb |
| 1039.04–1039.47 (6 ch) | 175 | 5 | y | 70 | — | 8 | 22 | — | 1 |  |
| 1072.650 | 300 | 5 | y | 30 | — | 7 | 19 | 0.5 | 3 | 99× 10.8322 MHz comb |
| 1000.40–1000.43 (3 ch) | 750 | 5 | n | — | — | 7 | 20 | 0.5 | -1 | 56× 17.8641 MHz comb |
| 995.79–995.87 (5 ch) | 225 | 5 | y | 30 | — | 7 | 15 | 1.4 | 2 |  |
| 1039.31–1039.49 (5 ch) | 150 | 5 | y | 70 | — | 7 | 27 | — | 5 |  |
| 1038.60–1039.48 (7 ch) | 225 | 5 | y | 70 | — | 7 | 18 | — | 2 |  |
| 1105.95–1105.99 (2 ch) | 825 | 5 | y | 70 | — | 6 | 19 | 0.9 | 3 |  |
| 1000.38–1000.39 (2 ch) | 825 | 5 | y | 60 | — | 6 | 19 | 1.7 | -1 | 56× 17.8641 MHz comb |
| 1105.96–1105.99 (3 ch) | 825 | 10 | y | 70 | — | 6 | 18 | 0.8 | 3 |  |
| 1105.97–1106.00 (4 ch) | 800 | 10 | y | 70 | — | 6 | 18 | 2.6 | 3 |  |

### radar (1300–1400 MHz): 41 bursts, 13 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1382.92–1383.36 (4 ch) | 800 | 5 | n | — | — | 4 | 14 | — | 2 | 53× 26.0980 MHz comb |
| 1387.82–1388.32 (4 ch) | 450 | 5 | n | — | — | 4 | 14 | — | 1 |  |
| 1333.27–1333.40 (2 ch) | 575 | 5 | y | 70 | — | 3 | 14 | — | -2 |  |
| 1382.12–1382.20 (2 ch) | 375 | 5 | y | 80 | — | 3 | 14 | — | 1 |  |
| 1333.50–1334.28 (3 ch) | 250 | 5 | y | 70 | — | 3 | 15 | — | 2 |  |
| 1333.91–1333.99 (3 ch) | 450 | 5 | y | 70 | — | 3 | 14 | — | 2 |  |
| 1356.25–1356.29 (3 ch) | 125 | 5 | y | 70 | — | 3 | 14 | — | 3 |  |
| 1360.05–1360.18 (3 ch) | 725 | 5 | n | — | — | 3 | 14 | — | 1 |  |
| 1379.29–1379.39 (3 ch) | 875 | 5 | n | — | — | 3 | 16 | — | 1 |  |
| 1380.86–1381.46 (3 ch) | 875 | 5 | n | — | — | 3 | 14 | — | -1 |  |
| 1386.64–1386.75 (3 ch) | 900 | 5 | n | — | — | 3 | 16 | — | -1 | 128× 10.8322 MHz comb |
| 1387.60–1387.95 (3 ch) | 750 | 5 | n | — | — | 3 | 15 | — | 1 |  |

### radio astronomy (protected) (1400–1427 MHz): 16 bursts, 5 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1411.75–1412.05 (4 ch) | 775 | 5 | n | — | — | 4 | 14 | — | -2 |  |
| 1405.63–1405.87 (3 ch) | 950 | 5 | n | — | — | 3 | 15 | — | 2 |  |
| 1407.68–1408.31 (3 ch) | 375 | 10 | n | — | — | 3 | 13 | — | 1 | 130× 10.8322 MHz comb |
| 1411.66–1412.48 (3 ch) | 425 | 5 | n | — | — | 3 | 14 | — | 1 |  |
| 1426.51–1426.71 (3 ch) | 425 | 5 | n | — | — | 3 | 14 | — | 2 |  |

### AWS-3/L-band / fixed (1427–1518 MHz): 404 bursts, 124 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1484.84–1485.46 (4 ch) | 275 | 5 | y | 70 | — | 6 | 16 | — | 2 |  |
| 1508.78–1509.11 (4 ch) | 450 | 5 | n | — | — | 5 | 13 | — | 2 |  |
| 1478.66–1479.33 (5 ch) | 500 | 5 | n | — | — | 5 | 14 | — | 0 |  |
| 1484.95–1485.48 (5 ch) | 225 | 5 | y | 70 | — | 5 | 17 | — | 2 |  |
| 1484.51–1484.99 (5 ch) | 175 | 5 | y | 70 | — | 5 | 17 | — | 2 |  |
| 1485.84–1486.20 (5 ch) | 500 | 5 | y | 70 | — | 5 | 16 | — | 1 |  |
| 1512.73–1513.42 (5 ch) | 575 | 5 | n | — | — | 5 | 14 | — | 1 | 58× 26.0980 MHz comb |
| 1463.410 | 1175 | 5 | y | 140 | — | 4 | 16 | 0.7 | 1 |  |
| 1434.60–1434.62 (3 ch) | 1175 | 10 | n | — | — | 4 | 17 | — | 2 | 55× 26.0980 MHz comb |
| 1457.59–1458.08 (3 ch) | 725 | 5 | n | — | — | 4 | 13 | — | 1 |  |
| 1487.76–1488.44 (3 ch) | 475 | 5 | y | 70 | — | 4 | 16 | — | 2 | 57× 26.0980 MHz comb |
| 1513.77–1514.16 (3 ch) | 475 | 10 | n | — | — | 4 | 16 | — | 1 | 58× 26.0980 MHz comb |

### MSS / GPS L1 1575 (1518–1559 MHz): 53 bursts, 17 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1535.06–1535.39 (2 ch) | 1150 | 10 | n | — | — | 4 | 16 | 0.5 | 2 |  |
| 1523.01–1523.32 (4 ch) | 400 | 5 | n | — | — | 4 | 13 | — | 2 |  |
| 1519.69–1520.21 (3 ch) | 325 | 5 | n | — | — | 3 | 13 | — | 1 |  |
| 1521.00–1521.20 (3 ch) | 800 | 10 | n | — | — | 3 | 15 | — | 1 |  |
| 1534.52–1535.43 (3 ch) | 625 | 5 | n | — | — | 3 | 14 | — | 1 |  |
| 1537.57–1537.95 (3 ch) | 825 | 5 | n | — | — | 3 | 14 | — | -1 |  |
| 1536.53–1537.43 (3 ch) | 325 | 5 | n | — | — | 3 | 14 | — | 1 |  |
| 1536.71–1537.30 (3 ch) | 950 | 5 | n | — | — | 3 | 15 | — | 1 |  |
| 1537.64–1537.79 (3 ch) | 1050 | 5 | n | — | — | 3 | 16 | — | -2 |  |
| 1537.61–1537.94 (3 ch) | 650 | 5 | n | — | — | 3 | 15 | — | -1 |  |
| 1536.23–1536.42 (3 ch) | 275 | 5 | y | 80 | — | 3 | 13 | — | 0 | 86× 17.8641 MHz comb |
| 1539.56–1539.72 (3 ch) | 1100 | 10 | n | — | — | 3 | 15 | — | 1 | 59× 26.0980 MHz comb |

### GNSS L1 (1559–1610 MHz): 8 bursts, 2 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1565.59–1566.18 (5 ch) | 400 | 5 | n | — | — | 5 | 13 | — | -1 | 60× 26.0980 MHz comb |
| 1565.61–1566.36 (3 ch) | 1125 | 5 | n | — | — | 3 | 15 | — | 2 | 60× 26.0980 MHz comb |

