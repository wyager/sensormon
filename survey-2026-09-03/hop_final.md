# Burst-emitter survey — receiver `survey-nesdr` (10.20.15.5:8433)

Store: 69671 example captures, 63904 raw groups (63904 for this receiver), 1232.8 MB; examples span 2.1 h. Bursts are 3–250 ms events ≥9 dB over the local floor; continuous carriers are invisible here (see the rtl_power sweep report).
'sweep dB' = continuous level at that frequency in the rtl_power mean sweeps; ≥6 dB means the 'bursts' are a continuous wideband signal flickering (FM modulation, spread-spectrum SMPS dither, TV) rather than a burst emitter — 1122 such emitters hidden.

## Harmonic combs (switching-supply / clock harmonics; not radio emitters)

- **10.8323 MHz** fundamental explains 263 emitters (harmonics 11…140, i.e. 119.2–1516.5 MHz)
- **31.2476 MHz** fundamental explains 199 emitters (harmonics 6…50, i.e. 187.5–1562.4 MHz)
- **28.0882 MHz** fundamental explains 215 emitters (harmonics 9…55, i.e. 252.8–1544.9 MHz)

## Top 40 emitters overall

| band | MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|---|
| FM broadcast | 94.74–95.47 (7 ch) | 25 | 5 | y | 30 | — | 108 | 14 | 1.1 | 4 | FM station 95.5 (modulation flicker) |
| FM broadcast | 96.64–96.79 (5 ch) | 50 | 10 | y | 40 | — | 101 | 18 | 0.3 | 4 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.64–96.78 (5 ch) | 25 | 10 | y | 50 | — | 91 | 17 | 0.3 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 103.54–104.35 (4 ch) | 50 | 10 | y | 30 | — | 84 | 16 | 3517.8 | 6 | FM station 103.5 (modulation flicker) |
| FM broadcast | 96.770 | 25 | 5 | y | 70 | — | 66 | 16 | 0.2 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.64–96.79 (3 ch) | 50 | 5 | y | 50 | — | 66 | 18 | 975.8 | 4 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.63–96.76 (4 ch) | 25 | 5 | y | 40 | — | 64 | 16 | 977.2 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 94.67–95.45 (2 ch) | 50 | 10 | y | 60 | — | 58 | 15 | 2.1 | 6 | FM station 95.5 (modulation flicker) |
| FM broadcast | 96.64–96.78 (2 ch) | 25 | 5 | y | 60 | — | 58 | 16 | 0.5 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 95.450 | 50 | 5 | y | 90 | — | 43 | 15 | 0.6 | 6 | FM station 95.5 (modulation flicker) |
| FM broadcast | 95.54–95.56 (2 ch) | 50 | 5 | y | 40 | — | 43 | 15 | 3516.4 | 4 | FM station 95.5 (modulation flicker) |
| FM broadcast | 96.63–96.75 (2 ch) | 25 | 10 | y | 30 | — | 42 | 16 | 0.1 | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.770 | 50 | 5 | y | 60 | — | 41 | 18 | — | 3 | FM station 96.7 (modulation flicker) |
| FM broadcast | 96.71–96.77 (2 ch) | 50 | 10 | y | 60 | — | 37 | 18 | 0.2 | 3 | FM station 96.7 (modulation flicker) |
| aviation (AM) | 119.07–119.19 (9 ch) | 25 | 5 | y | 30 | — | 34 | 14 | 3519.9 | -0 | 11× 10.8323 MHz comb |
| FM broadcast | 93.63–93.78 (4 ch) | 50 | 5 | y | 30 | — | 31 | 15 | 1.5 | 5 | FM station 93.7 (modulation flicker) |
| FM broadcast | 96.66–96.75 (3 ch) | 75 | 10 | y | 40 | — | 28 | 19 | 0.6 | 5 | FM station 96.7 (modulation flicker) |
| FM broadcast | 95.52–95.57 (4 ch) | 75 | 10 | y | 40 | — | 25 | 15 | 2.0 | 4 | FM station 95.5 (modulation flicker) |
| VHF TV ch 7-13 | 205.61–205.85 (13 ch) | 50 | 5 | y | 40 | — | 25 | 15 | 976.5 | 6 | 19× 10.8323 MHz comb |
| FM broadcast | 93.66–93.76 (3 ch) | 50 | 10 | y | 30 | — | 24 | 15 | 2.1 | 5 | FM station 93.7 (modulation flicker) |
| VHF TV ch 7-13 | 205.66–206.42 (9 ch) | 50 | 5 | y | 30 | — | 23 | 15 | 982.1 | 5 | 19× 10.8323 MHz comb |
| VHF TV ch 7-13 | 205.64–205.89 (13 ch) | 50 | 10 | y | 30 | — | 23 | 15 | 976.4 | 6 | 19× 10.8323 MHz comb |
| VHF TV ch 7-13 | 205.65–205.87 (14 ch) | 75 | 5 | y | 30 | — | 21 | 15 | 2.0 | 2 | 19× 10.8323 MHz comb |
| military aviation / satcom (UHF) | 281.52–281.71 (9 ch) | 75 | 5 | y | 40 | — | 20 | 16 | 976.3 | 5 | 26× 10.8323 MHz comb |
| FM broadcast | 93.68–93.75 (2 ch) | 50 | 5 | y | 70 | — | 19 | 15 | 0.3 | 5 | FM station 93.7 (modulation flicker) |
| FM broadcast | 93.74–93.77 (2 ch) | 50 | 5 | y | 50 | — | 18 | 16 | 0.4 | 3 | FM station 93.7 (modulation flicker) |
| VHF TV ch 7-13 | 205.65–205.85 (7 ch) | 75 | 10 | y | 40 | — | 18 | 16 | 1.2 | 3 | 19× 10.8323 MHz comb |
| aviation (AM) | 119.06–119.20 (6 ch) | 50 | 10 | y | 30 | — | 18 | 15 | 2.3 | 3 | 11× 10.8323 MHz comb |
| military aviation / satcom (UHF) | 286.71–287.10 (12 ch) | 125 | 10 | y | 30 | — | 18 | 16 | 0.6 | 4 |  |
| FM broadcast | 95.51–95.56 (2 ch) | 25 | 5 | y | 30 | — | 16 | 15 | 1.3 | 4 | FM station 95.5 (modulation flicker) |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 1000.40–1000.43 (3 ch) | 750 | 5 | n | — | — | 16 | 19 | 1.7 | -1 | 32× 31.2476 MHz comb |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 1000.40–1000.43 (4 ch) | 750 | 10 | n | — | — | 16 | 18 | 0.8 | -1 | 32× 31.2476 MHz comb |
| VHF TV ch 7-13 | 194.76–195.05 (10 ch) | 75 | 5 | y | 30 | — | 16 | 15 | 3521.1 | 6 | 18× 10.8323 MHz comb |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 1125.05–1125.34 (10 ch) | 500 | 5 | y | 30 | — | 15 | 15 | 3519.2 | 1 | 36× 31.2476 MHz comb |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 1105.96–1106.00 (5 ch) | 800 | 5 | y | 70 | — | 14 | 18 | 3515.6 | 3 |  |
| military aviation / satcom (UHF) | 286.76–287.09 (8 ch) | 50 | 5 | y | 30 | — | 14 | 15 | 1.0 | 4 |  |
| military aviation / satcom (UHF) | 286.71–287.16 (11 ch) | 100 | 10 | y | 30 | — | 14 | 16 | — | 4 |  |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 1000.40–1000.44 (5 ch) | 725 | 5 | n | — | — | 13 | 17 | 1.5 | -1 | 32× 31.2476 MHz comb |
| VHF TV ch 7-13 | 194.88–195.05 (5 ch) | 50 | 5 | y | 40 | — | 13 | 15 | 3.6 | 3 | 18× 10.8323 MHz comb |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 995.79–996.11 (8 ch) | 225 | 5 | y | 30 | — | 13 | 15 | 1.4 | 2 |  |

## Wide families (same burst class in ≥5 distinct 1 MHz cells of a band: hoppers, or a band-wide artifact)

| band | MHz range | cells | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR |
|---|---|---|---|---|---|---|---|---|---|
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 970.72–1132.29 | 27 | 725 | 5 | n | — | — | 109 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 960.68–1156.32 | 26 | 650 | 5 | n | — | — | 99 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 961.57–1134.45 | 27 | 775 | 5 | n | — | — | 97 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 961.73–1155.43 | 25 | 575 | 5 | n | — | — | 89 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 962.85–1153.48 | 27 | 625 | 5 | n | — | — | 88 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 961.66–1151.45 | 21 | 1100 | 5 | n | — | — | 87 | 17 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 962.92–1152.98 | 25 | 675 | 5 | n | — | — | 87 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 963.52–1153.92 | 24 | 800 | 5 | n | — | — | 84 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 960.68–1132.24 | 23 | 700 | 5 | n | — | — | 82 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 961.99–1152.46 | 24 | 550 | 5 | n | — | — | 82 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 967.79–1151.36 | 23 | 600 | 5 | n | — | — | 81 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 965.75–1156.31 | 20 | 950 | 5 | n | — | — | 80 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 975.05–1151.23 | 23 | 1175 | 5 | n | — | — | 79 | 17 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 962.79–1151.26 | 22 | 500 | 5 | n | — | — | 76 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 962.93–1153.84 | 20 | 1025 | 5 | n | — | — | 75 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 960.81–1132.36 | 19 | 875 | 5 | n | — | — | 73 | 16 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 961.98–1132.41 | 16 | 750 | 5 | n | — | — | 69 | 17 |
| VHF TV ch 7-13 | 191.96–215.24 | 6 | 50 | 5 | y | 40 | — | 64 | 15 |
| aeronautical navigation (DME/TACAN/ADS-B 1090) | 976.08–1111.89 | 17 | 1150 | 5 | n | — | — | 64 | 17 |
| AWS-3/L-band / fixed | 1431.68–1515.27 | 16 | 500 | 5 | n | — | — | 61 | 15 |

## Per band

### VHF TV ch 2-6 (mostly vacant) (54–88 MHz): 6 bursts, 2 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 86.500 | 25 | 5 | y | 60 | — | 3 | 14 | 1.8 | 4 |  |
| 87.060 | 25 | 5 | y | 100 | — | 3 | 13 | 0.9 | 4 |  |

### FM broadcast (88–108 MHz): 1241 bursts, 52 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 94.74–95.47 (7 ch) | 25 | 5 | y | 30 | — | 108 | 14 | 1.1 | 4 | FM station 95.5 (modulation flicker) |
| 96.64–96.79 (5 ch) | 50 | 10 | y | 40 | — | 101 | 18 | 0.3 | 4 | FM station 96.7 (modulation flicker) |
| 96.64–96.78 (5 ch) | 25 | 10 | y | 50 | — | 91 | 17 | 0.3 | 3 | FM station 96.7 (modulation flicker) |
| 103.54–104.35 (4 ch) | 50 | 10 | y | 30 | — | 84 | 16 | 3517.8 | 6 | FM station 103.5 (modulation flicker) |
| 96.770 | 25 | 5 | y | 70 | — | 66 | 16 | 0.2 | 3 | FM station 96.7 (modulation flicker) |
| 96.64–96.79 (3 ch) | 50 | 5 | y | 50 | — | 66 | 18 | 975.8 | 4 | FM station 96.7 (modulation flicker) |
| 96.63–96.76 (4 ch) | 25 | 5 | y | 40 | — | 64 | 16 | 977.2 | 3 | FM station 96.7 (modulation flicker) |
| 94.67–95.45 (2 ch) | 50 | 10 | y | 60 | — | 58 | 15 | 2.1 | 6 | FM station 95.5 (modulation flicker) |
| 96.64–96.78 (2 ch) | 25 | 5 | y | 60 | — | 58 | 16 | 0.5 | 3 | FM station 96.7 (modulation flicker) |
| 95.450 | 50 | 5 | y | 90 | — | 43 | 15 | 0.6 | 6 | FM station 95.5 (modulation flicker) |
| 95.54–95.56 (2 ch) | 50 | 5 | y | 40 | — | 43 | 15 | 3516.4 | 4 | FM station 95.5 (modulation flicker) |
| 96.63–96.75 (2 ch) | 25 | 10 | y | 30 | — | 42 | 16 | 0.1 | 3 | FM station 96.7 (modulation flicker) |

### aviation (AM) (108–137 MHz): 147 bursts, 20 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 119.07–119.19 (9 ch) | 25 | 5 | y | 30 | — | 34 | 14 | 3519.9 | -0 | 11× 10.8323 MHz comb |
| 119.06–119.20 (6 ch) | 50 | 10 | y | 30 | — | 18 | 15 | 2.3 | 3 | 11× 10.8323 MHz comb |
| 119.06–119.14 (3 ch) | 25 | 10 | y | 50 | — | 10 | 14 | 4498.4 | 3 | 11× 10.8323 MHz comb |
| 120.56–121.12 (5 ch) | 25 | 10 | y | 130 | — | 10 | 14 | 4498.9 | 5 |  |
| 120.420 | 25 | 10 | y | 100 | — | 7 | 14 | 0.7 | 4 |  |
| 119.06–119.13 (2 ch) | 25 | 5 | y | 60 | — | 7 | 15 | 2.1 | 2 | 11× 10.8323 MHz comb |
| 119.05–119.18 (4 ch) | 50 | 5 | y | 140 | — | 7 | 15 | 4497.6 | 2 | 11× 10.8323 MHz comb |
| 119.15–119.18 (3 ch) | 50 | 10 | y | 40 | — | 6 | 15 | 3516.4 | -1 | 11× 10.8323 MHz comb |
| 119.13–119.18 (4 ch) | 25 | 10 | y | 30 | — | 5 | 14 | — | -1 | 11× 10.8323 MHz comb |
| 119.07–119.14 (3 ch) | 50 | 20 | y | 40 | — | 5 | 15 | — | 3 | 11× 10.8323 MHz comb |
| 119.04–119.10 (4 ch) | 25 | 5 | y | 40 | — | 5 | 15 | — | 3 | 11× 10.8323 MHz comb |
| 119.090 | 25 | 5 | y | 140 | — | 4 | 15 | 4497.6 | 3 | 11× 10.8323 MHz comb |

### VHF business / MURS (151.82-154.6) (150.8–156 MHz): 3 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 155.45–155.47 (2 ch) | 25 | 5 | y | 60 | — | 3 | 13 | — | -3 |  |

### VHF business / public safety / wireless mics (162.6–174 MHz): 28 bursts, 7 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 165.07–165.23 (6 ch) | 25 | 5 | y | 30 | — | 6 | 13 | — | 5 |  |
| 166.05–166.22 (4 ch) | 75 | 5 | y | 30 | — | 5 | 14 | — | 6 |  |
| 165.09–165.24 (3 ch) | 50 | 5 | y | 40 | — | 4 | 14 | — | 6 |  |
| 167.66–167.74 (4 ch) | 100 | 10 | y | 40 | — | 4 | 15 | — | 3 |  |
| 167.720 | 50 | 10 | y | 50 | — | 3 | 15 | 1.7 | 3 |  |
| 167.88–167.92 (2 ch) | 50 | 10 | y | 40 | — | 3 | 14 | — | 5 |  |
| 173.17–173.43 (2 ch) | 175 | 20 | y | 30 | — | 3 | 17 | — | 3 | 16× 10.8323 MHz comb |

### VHF TV ch 7-13 (174–216 MHz): 490 bursts, 70 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 205.61–205.85 (13 ch) | 50 | 5 | y | 40 | — | 25 | 15 | 976.5 | 6 | 19× 10.8323 MHz comb |
| 205.66–206.42 (9 ch) | 50 | 5 | y | 30 | — | 23 | 15 | 982.1 | 5 | 19× 10.8323 MHz comb |
| 205.64–205.89 (13 ch) | 50 | 10 | y | 30 | — | 23 | 15 | 976.4 | 6 | 19× 10.8323 MHz comb |
| 205.65–205.87 (14 ch) | 75 | 5 | y | 30 | — | 21 | 15 | 2.0 | 2 | 19× 10.8323 MHz comb |
| 205.65–205.85 (7 ch) | 75 | 10 | y | 40 | — | 18 | 16 | 1.2 | 3 | 19× 10.8323 MHz comb |
| 194.76–195.05 (10 ch) | 75 | 5 | y | 30 | — | 16 | 15 | 3521.1 | 6 | 18× 10.8323 MHz comb |
| 194.88–195.05 (5 ch) | 50 | 5 | y | 40 | — | 13 | 15 | 3.6 | 3 | 18× 10.8323 MHz comb |
| 208.82–209.08 (6 ch) | 50 | 5 | y | 40 | — | 12 | 14 | 975.3 | 3 |  |
| 205.64–205.89 (7 ch) | 75 | 10 | y | 30 | — | 12 | 15 | 3520.2 | 6 | 19× 10.8323 MHz comb |
| 205.62–205.81 (7 ch) | 100 | 10 | y | 30 | — | 11 | 16 | 975.5 | 6 | 19× 10.8323 MHz comb |
| 197.74–198.39 (8 ch) | 50 | 10 | y | 30 | — | 11 | 14 | 4495.8 | 6 |  |
| 213.53–214.29 (6 ch) | 50 | 5 | y | 30 | — | 11 | 15 | 4496.3 | -1 |  |

### 1.25 m ham (216–225 MHz): 12 bursts, 2 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 217.69–218.48 (5 ch) | 50 | 5 | y | 40 | — | 9 | 14 | 1.3 | 4 |  |
| 218.25–218.29 (3 ch) | 100 | 10 | y | 40 | — | 3 | 15 | — | 2 |  |

### military aviation / satcom (UHF) (225–400 MHz): 832 bursts, 199 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 281.52–281.71 (9 ch) | 75 | 5 | y | 40 | — | 20 | 16 | 976.3 | 5 | 26× 10.8323 MHz comb |
| 286.71–287.10 (12 ch) | 125 | 10 | y | 30 | — | 18 | 16 | 0.6 | 4 |  |
| 286.76–287.09 (8 ch) | 50 | 5 | y | 30 | — | 14 | 15 | 1.0 | 4 |  |
| 286.71–287.16 (11 ch) | 100 | 10 | y | 30 | — | 14 | 16 | — | 4 |  |
| 281.58–281.73 (6 ch) | 100 | 5 | y | 40 | — | 11 | 15 | 2.6 | 4 | 26× 10.8323 MHz comb |
| 281.60–281.68 (6 ch) | 50 | 5 | y | 30 | — | 10 | 14 | — | 6 | 26× 10.8323 MHz comb |
| 281.65–281.70 (5 ch) | 100 | 10 | y | 30 | — | 9 | 16 | 2.1 | 5 | 26× 10.8323 MHz comb |
| 286.83–287.11 (5 ch) | 75 | 10 | y | 30 | — | 9 | 16 | 4496.6 | 4 |  |
| 232.64–232.93 (5 ch) | 50 | 5 | y | 30 | — | 8 | 14 | — | 4 |  |
| 286.75–287.09 (6 ch) | 100 | 10 | y | 50 | — | 8 | 17 | — | 4 |  |
| 367.90–368.16 (6 ch) | 125 | 10 | y | 40 | — | 8 | 15 | — | 3 | 34× 10.8323 MHz comb |
| 252.02–252.03 (2 ch) | 50 | 10 | y | 30 | — | 7 | 16 | 3518.9 | 1 |  |

### federal (406–420 MHz): 29 bursts, 8 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 409.96–410.30 (4 ch) | 200 | 5 | y | 40 | — | 7 | 18 | 4494.6 | 1 |  |
| 409.95–410.00 (3 ch) | 175 | 5 | y | 30 | — | 4 | 15 | — | 1 |  |
| 410.380 | 50 | 10 | y | 50 | — | 3 | 16 | 4497.7 | 6 |  |
| 408.02–408.03 (2 ch) | 50 | 5 | y | 30 | — | 3 | 16 | — | 3 |  |
| 410.06–410.08 (2 ch) | 150 | 5 | y | 30 | — | 3 | 15 | — | 1 |  |
| 410.01–410.30 (3 ch) | 125 | 5 | y | 50 | — | 3 | 17 | — | 4 |  |
| 409.96–410.11 (3 ch) | 100 | 5 | y | 40 | — | 3 | 13 | — | 1 |  |
| 410.02–410.08 (3 ch) | 250 | 10 | y | 40 | — | 3 | 16 | — | 1 |  |

### 70 cm ham (incl. 433.92 ISM part 15) (420–450 MHz): 50 bursts, 14 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 448.99–449.14 (4 ch) | 250 | 10 | y | 40 | — | 5 | 17 | — | 3 | 16× 28.0882 MHz comb |
| 449.02–449.20 (5 ch) | 150 | 5 | y | 40 | — | 5 | 14 | — | 4 | 16× 28.0882 MHz comb |
| 449.04–449.10 (3 ch) | 250 | 5 | y | 40 | — | 4 | 18 | — | 3 | 16× 28.0882 MHz comb |
| 420.63–421.12 (4 ch) | 150 | 5 | y | 30 | — | 4 | 16 | — | 4 | 15× 28.0882 MHz comb |
| 449.02–449.22 (4 ch) | 250 | 10 | y | 50 | — | 4 | 17 | — | 4 | 16× 28.0882 MHz comb |
| 449.57–449.70 (4 ch) | 250 | 10 | y | 50 | — | 4 | 16 | — | 5 | 16× 28.0882 MHz comb |
| 447.150 | 150 | 5 | y | 30 | — | 3 | 13 | 4495.8 | 4 |  |
| 449.19–449.27 (2 ch) | 175 | 5 | y | 50 | — | 3 | 18 | — | 3 | 16× 28.0882 MHz comb |
| 449.01–449.22 (2 ch) | 125 | 5 | y | 30 | — | 3 | 15 | — | 4 | 16× 28.0882 MHz comb |
| 449.14–449.17 (2 ch) | 175 | 5 | y | 40 | — | 3 | 17 | — | 4 | 16× 28.0882 MHz comb |
| 421.04–421.08 (2 ch) | 200 | 10 | y | 40 | — | 3 | 16 | — | 4 | 15× 28.0882 MHz comb |
| 449.17–449.22 (2 ch) | 150 | 5 | y | 30 | — | 3 | 18 | — | 4 | 16× 28.0882 MHz comb |

### UHF business / public safety / FRS-GMRS 462-467 (450–470 MHz): 13 bursts, 4 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 459.81–460.47 (4 ch) | 250 | 10 | y | 50 | — | 4 | 18 | — | 5 |  |
| 459.94–460.35 (2 ch) | 275 | 10 | y | 30 | — | 3 | 16 | — | 5 |  |
| 460.30–460.49 (2 ch) | 150 | 5 | y | 30 | — | 3 | 14 | — | -3 |  |
| 459.79–460.39 (3 ch) | 200 | 10 | y | 30 | — | 3 | 15 | — | 4 |  |

### UHF TV ch 14-36 / wireless mics (470–608 MHz): 24 bursts, 8 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 482.240 | 200 | 10 | y | 160 | — | 3 | 14 | 979.5 | 4 |  |
| 482.100 | 475 | 20 | y | 30 | — | 3 | 17 | 979.2 | 5 |  |
| 482.14–482.16 (2 ch) | 325 | 20 | y | 100 | — | 3 | 17 | — | 5 |  |
| 482.18–482.19 (2 ch) | 300 | 20 | n | — | — | 3 | 16 | — | 4 |  |
| 606.71–606.85 (2 ch) | 200 | 5 | y | 60 | — | 3 | 14 | — | 2 | 56× 10.8323 MHz comb |
| 606.06–606.10 (2 ch) | 275 | 5 | y | 30 | — | 3 | 16 | — | 2 |  |
| 541.53–541.59 (2 ch) | 125 | 10 | y | 40 | — | 3 | 15 | — | 1 | 50× 10.8323 MHz comb |
| 476.69–476.79 (3 ch) | 150 | 5 | y | 40 | — | 3 | 14 | — | 6 | 44× 10.8323 MHz comb |

### 600 MHz cellular (band 71) / TV ch 38-51 (614–698 MHz): 9 bursts, 3 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 692.65–692.81 (2 ch) | 225 | 10 | y | 50 | — | 3 | 14 | — | 3 |  |
| 692.55–693.14 (3 ch) | 175 | 5 | y | 40 | — | 3 | 13 | — | -1 |  |
| 692.74–693.09 (3 ch) | 200 | 5 | y | 30 | — | 3 | 15 | — | 1 |  |

### 700 MHz cellular (bands 12/13/14/17) / FirstNet (698–806 MHz): 53 bursts, 16 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 803.41–803.42 (2 ch) | 1175 | 20 | n | — | — | 5 | 18 | 2.9 | 1 |  |
| 803.63–803.64 (2 ch) | 725 | 20 | n | — | — | 4 | 17 | 1.7 | 1 |  |
| 797.29–797.34 (4 ch) | 950 | 5 | n | — | — | 4 | 27 | — | 2 |  |
| 803.56–803.59 (4 ch) | 825 | 20 | n | — | — | 4 | 18 | — | 1 |  |
| 803.510 | 975 | 20 | n | — | — | 3 | 18 | 3520.4 | 1 |  |
| 797.30–797.31 (2 ch) | 925 | 5 | n | — | — | 3 | 27 | — | 2 |  |
| 803.42–803.44 (2 ch) | 1125 | 20 | n | — | — | 3 | 19 | — | 1 |  |
| 802.69–802.71 (2 ch) | 825 | 20 | n | — | — | 3 | 16 | — | 1 |  |
| 802.62–802.78 (3 ch) | 575 | 20 | n | — | — | 3 | 16 | — | 1 |  |
| 802.75–802.89 (3 ch) | 700 | 20 | n | — | — | 3 | 15 | — | 1 |  |
| 802.65–803.49 (3 ch) | 900 | 20 | n | — | — | 3 | 15 | — | 1 |  |
| 802.80–803.04 (3 ch) | 650 | 20 | n | — | — | 3 | 16 | — | 0 |  |

### cellular 850 downlink (band 5) (869–894 MHz): 29 bursts, 7 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 875.50–875.53 (4 ch) | 875 | 5 | y | 40 | — | 6 | 14 | 3517.0 | 1 | 28× 31.2476 MHz comb |
| 875.52–875.55 (4 ch) | 900 | 5 | y | 40 | — | 5 | 15 | — | 1 | 28× 31.2476 MHz comb |
| 875.51–875.58 (3 ch) | 850 | 5 | y | 40 | — | 4 | 14 | — | 1 | 28× 31.2476 MHz comb |
| 875.52–875.53 (2 ch) | 950 | 5 | y | 40 | — | 4 | 15 | — | 1 | 28× 31.2476 MHz comb |
| 875.45–875.48 (3 ch) | 800 | 5 | y | 40 | — | 4 | 14 | — | 2 | 28× 31.2476 MHz comb |
| 875.51–875.52 (2 ch) | 925 | 5 | y | 40 | — | 3 | 16 | — | 1 | 28× 31.2476 MHz comb |
| 875.53–876.45 (3 ch) | 825 | 5 | n | — | — | 3 | 15 | — | 1 |  |

### 902-928 ISM: LoRa, Z-Wave 908/916, meters, Fineoffset 915 (902–928 MHz): 12 bursts, 4 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 923.50–924.27 (3 ch) | 550 | 5 | n | — | — | 3 | 14 | — | 1 |  |
| 923.09–923.41 (3 ch) | 225 | 5 | y | 110 | — | 3 | 13 | — | 1 |  |
| 925.56–925.97 (3 ch) | 525 | 5 | n | — | — | 3 | 13 | — | 3 |  |
| 924.42–924.48 (3 ch) | 400 | 5 | n | — | — | 3 | 13 | — | 2 |  |

### paging (929-932) / MAS / narrowband PCS (928–942 MHz): 3 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 940.88–941.37 (2 ch) | 150 | 5 | y | 50 | — | 3 | 13 | — | 1 |  |

### fixed / GSM-R-ish (942–960 MHz): 773 bursts, 219 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 957.28–957.35 (5 ch) | 500 | 5 | y | 90 | — | 10 | 16 | 0.6 | 1 |  |
| 950.23–950.24 (2 ch) | 325 | 5 | y | 80 | — | 8 | 15 | 3515.8 | -1 |  |
| 950.20–950.25 (4 ch) | 300 | 5 | y | 80 | — | 8 | 16 | 3518.6 | -1 |  |
| 944.93–945.43 (7 ch) | 350 | 5 | n | — | — | 8 | 15 | — | 6 |  |
| 944.84–945.31 (7 ch) | 550 | 5 | n | — | — | 7 | 15 | — | 3 |  |
| 950.20–950.24 (3 ch) | 275 | 5 | y | 80 | — | 6 | 15 | 3518.1 | -2 |  |
| 957.82–957.94 (5 ch) | 400 | 5 | n | — | — | 6 | 16 | — | 3 |  |
| 957.33–957.39 (2 ch) | 425 | 5 | y | 120 | — | 5 | 16 | 1.2 | 1 |  |
| 946.00–946.09 (3 ch) | 800 | 5 | n | — | — | 5 | 16 | — | 1 |  |
| 957.31–957.37 (3 ch) | 425 | 5 | y | 60 | — | 5 | 15 | — | 1 |  |
| 944.83–945.32 (5 ch) | 575 | 5 | n | — | — | 5 | 15 | — | 3 |  |
| 943.72–944.49 (5 ch) | 775 | 5 | n | — | — | 5 | 14 | — | -1 |  |

### aeronautical navigation (DME/TACAN/ADS-B 1090) (960–1215 MHz): 4690 bursts, 1303 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1000.40–1000.43 (3 ch) | 750 | 5 | n | — | — | 16 | 19 | 1.7 | -1 | 32× 31.2476 MHz comb |
| 1000.40–1000.43 (4 ch) | 750 | 10 | n | — | — | 16 | 18 | 0.8 | -1 | 32× 31.2476 MHz comb |
| 1125.05–1125.34 (10 ch) | 500 | 5 | y | 30 | — | 15 | 15 | 3519.2 | 1 | 36× 31.2476 MHz comb |
| 1105.96–1106.00 (5 ch) | 800 | 5 | y | 70 | — | 14 | 18 | 3515.6 | 3 |  |
| 1000.40–1000.44 (5 ch) | 725 | 5 | n | — | — | 13 | 17 | 1.5 | -1 | 32× 31.2476 MHz comb |
| 995.79–996.11 (8 ch) | 225 | 5 | y | 30 | — | 13 | 15 | 1.4 | 2 |  |
| 1038.56–1039.47 (8 ch) | 175 | 5 | y | 70 | — | 13 | 22 | 3514.9 | 2 | 37× 28.0882 MHz comb |
| 1105.98–1105.99 (2 ch) | 825 | 5 | y | 30 | — | 10 | 19 | 3518.0 | 3 |  |
| 995.79–996.18 (4 ch) | 300 | 5 | y | 30 | — | 10 | 16 | 2.6 | 2 |  |
| 1000.37–1000.39 (3 ch) | 825 | 5 | y | 60 | — | 10 | 19 | 1.7 | -1 | 32× 31.2476 MHz comb |
| 1105.97–1106.00 (4 ch) | 800 | 10 | y | 70 | — | 10 | 17 | 3516.9 | 3 |  |
| 1124.90–1125.34 (5 ch) | 525 | 5 | y | 30 | — | 10 | 17 | 3520.5 | 1 | 36× 31.2476 MHz comb |

### radar (1300–1400 MHz): 183 bursts, 54 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1332.54–1333.44 (6 ch) | 225 | 5 | y | 70 | — | 6 | 13 | — | 1 |  |
| 1382.83–1383.36 (6 ch) | 800 | 5 | n | — | — | 6 | 15 | — | 2 |  |
| 1355.51–1356.29 (4 ch) | 125 | 5 | y | 70 | — | 5 | 15 | — | 3 |  |
| 1383.56–1384.34 (5 ch) | 775 | 5 | n | — | — | 5 | 15 | — | 1 |  |
| 1333.27–1333.40 (2 ch) | 575 | 5 | y | 70 | — | 4 | 14 | 3517.1 | -2 |  |
| 1383.93–1384.21 (3 ch) | 1175 | 5 | n | — | — | 4 | 15 | — | 1 |  |
| 1387.72–1388.09 (3 ch) | 1025 | 5 | n | — | — | 4 | 15 | — | 1 |  |
| 1330.61–1330.95 (4 ch) | 575 | 5 | y | 70 | — | 4 | 14 | — | 1 |  |
| 1333.54–1333.99 (4 ch) | 450 | 5 | y | 70 | — | 4 | 14 | — | 2 |  |
| 1357.51–1358.03 (4 ch) | 650 | 5 | n | — | — | 4 | 14 | — | 3 |  |
| 1358.93–1359.33 (4 ch) | 425 | 5 | n | — | — | 4 | 14 | — | 3 |  |
| 1388.75–1389.15 (4 ch) | 825 | 5 | n | — | — | 4 | 13 | — | 2 |  |

### radio astronomy (protected) (1400–1427 MHz): 138 bursts, 43 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1411.60–1412.05 (5 ch) | 775 | 5 | n | — | — | 5 | 15 | — | -2 |  |
| 1409.15–1409.28 (3 ch) | 700 | 10 | n | — | — | 4 | 17 | — | 0 |  |
| 1410.76–1410.79 (3 ch) | 725 | 10 | y | 130 | — | 4 | 17 | — | 1 |  |
| 1412.75–1413.10 (4 ch) | 575 | 5 | n | — | — | 4 | 14 | — | 1 |  |
| 1412.80–1413.13 (4 ch) | 675 | 10 | n | — | — | 4 | 14 | — | 1 |  |
| 1412.87–1413.28 (4 ch) | 600 | 5 | n | — | — | 4 | 14 | — | 1 |  |
| 1411.66–1412.48 (4 ch) | 425 | 5 | n | — | — | 4 | 14 | — | 2 |  |
| 1408.00–1408.40 (4 ch) | 675 | 10 | n | — | — | 4 | 15 | — | 2 | 130× 10.8323 MHz comb |
| 1407.68–1408.17 (2 ch) | 375 | 10 | y | 80 | — | 3 | 13 | — | 2 | 130× 10.8323 MHz comb |
| 1408.61–1408.62 (2 ch) | 350 | 5 | y | 40 | — | 3 | 15 | — | 2 |  |
| 1406.12–1406.17 (3 ch) | 275 | 5 | y | 40 | — | 3 | 14 | — | -1 | 45× 31.2476 MHz comb |
| 1405.63–1405.87 (3 ch) | 950 | 5 | n | — | — | 3 | 15 | — | 2 | 45× 31.2476 MHz comb |

### AWS-3/L-band / fixed (1427–1518 MHz): 1759 bursts, 490 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1484.84–1485.47 (5 ch) | 275 | 5 | y | 70 | — | 12 | 16 | 2.3 | 2 |  |
| 1485.84–1486.45 (8 ch) | 500 | 5 | y | 70 | — | 9 | 15 | — | 1 |  |
| 1485.41–1485.44 (3 ch) | 325 | 5 | y | 40 | — | 8 | 17 | 0.6 | 3 |  |
| 1463.410 | 1175 | 5 | y | 140 | — | 7 | 17 | 1.6 | 1 |  |
| 1438.02–1438.48 (6 ch) | 850 | 5 | n | — | — | 7 | 15 | — | 1 | 46× 31.2476 MHz comb |
| 1485.83–1486.46 (6 ch) | 475 | 5 | y | 70 | — | 7 | 15 | — | 1 |  |
| 1484.95–1485.48 (6 ch) | 225 | 5 | y | 70 | — | 7 | 20 | — | 2 |  |
| 1464.55–1465.46 (7 ch) | 1100 | 5 | n | — | — | 7 | 16 | — | 1 |  |
| 1485.76–1486.30 (7 ch) | 325 | 5 | y | 70 | — | 7 | 14 | — | 4 |  |
| 1510.53–1511.29 (7 ch) | 500 | 5 | n | — | — | 7 | 15 | — | 1 |  |
| 1510.61–1511.36 (7 ch) | 550 | 5 | n | — | — | 7 | 14 | — | 1 |  |
| 1512.73–1513.42 (7 ch) | 575 | 5 | n | — | — | 7 | 14 | — | 3 |  |

### MSS / GPS L1 1575 (1518–1559 MHz): 234 bursts, 71 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1535.06–1535.40 (4 ch) | 1150 | 10 | n | — | — | 6 | 16 | 0.5 | 2 |  |
| 1539.63–1540.33 (6 ch) | 900 | 5 | n | — | — | 6 | 15 | — | 2 |  |
| 1538.80–1539.47 (5 ch) | 575 | 5 | n | — | — | 5 | 15 | — | 1 |  |
| 1538.78–1539.22 (3 ch) | 700 | 10 | n | — | — | 4 | 16 | — | 1 |  |
| 1536.77–1537.41 (3 ch) | 550 | 5 | n | — | — | 4 | 14 | — | 1 |  |
| 1518.59–1518.84 (4 ch) | 675 | 10 | n | — | — | 4 | 16 | — | 2 |  |
| 1523.01–1523.32 (4 ch) | 400 | 5 | n | — | — | 4 | 13 | — | 2 |  |
| 1535.51–1535.68 (4 ch) | 475 | 5 | n | — | — | 4 | 14 | — | 2 |  |
| 1537.57–1537.95 (4 ch) | 825 | 5 | n | — | — | 4 | 15 | — | 0 |  |
| 1537.64–1537.82 (4 ch) | 1050 | 5 | n | — | — | 4 | 16 | — | -1 |  |
| 1537.59–1537.94 (4 ch) | 650 | 5 | n | — | — | 4 | 15 | — | -1 |  |
| 1539.53–1540.38 (4 ch) | 800 | 10 | n | — | — | 4 | 15 | — | -1 |  |

### GNSS L1 (1559–1610 MHz): 44 bursts, 12 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 1563.81–1564.38 (5 ch) | 825 | 5 | n | — | — | 5 | 15 | — | 0 |  |
| 1565.59–1566.18 (5 ch) | 400 | 5 | n | — | — | 5 | 13 | — | -1 |  |
| 1565.66–1566.30 (5 ch) | 450 | 5 | n | — | — | 5 | 13 | — | 2 |  |
| 1560.57–1561.28 (4 ch) | 500 | 5 | n | — | — | 4 | 13 | — | 2 |  |
| 1562.82–1563.22 (4 ch) | 825 | 5 | n | — | — | 4 | 15 | — | 1 | 50× 31.2476 MHz comb |
| 1561.20–1561.46 (3 ch) | 200 | 5 | y | 100 | — | 3 | 12 | — | 1 |  |
| 1563.55–1563.97 (3 ch) | 525 | 5 | n | — | — | 3 | 14 | — | 2 |  |
| 1562.69–1562.83 (3 ch) | 225 | 5 | y | 40 | — | 3 | 14 | — | -0 | 50× 31.2476 MHz comb |
| 1565.61–1566.36 (3 ch) | 1125 | 5 | n | — | — | 3 | 15 | — | 2 |  |
| 1565.59–1566.34 (3 ch) | 550 | 5 | n | — | — | 3 | 15 | — | 2 |  |
| 1567.86–1568.33 (3 ch) | 450 | 5 | n | — | — | 3 | 13 | — | 1 |  |
| 1566.14–1566.32 (3 ch) | 525 | 5 | n | — | — | 3 | 14 | — | 2 |  |

