# Burst-emitter survey — receiver `survey-nesdr` (10.20.15.5:8433)

Store: 7018 example captures, 5643 raw groups (5643 for this receiver), 73.6 MB; examples span 0.3 h. Bursts are 3–250 ms events ≥9 dB over the local floor; continuous carriers are invisible here (see the rtl_power sweep report).
'sweep dB' = continuous level at that frequency in the rtl_power mean sweeps; ≥6 dB means the 'bursts' are a continuous wideband signal flickering (FM modulation, spread-spectrum SMPS dither, TV) rather than a burst emitter — 455 such emitters hidden.

## Harmonic combs (switching-supply / clock harmonics; not radio emitters)

- **5.4175 MHz** fundamental explains 56 emitters (harmonics 22…89, i.e. 119.2–482.2 MHz)
- **7.4355 MHz** fundamental explains 26 emitters (harmonics 13…50, i.e. 96.7–371.8 MHz)
- **13.6460 MHz** fundamental explains 17 emitters (harmonics 7…27, i.e. 95.5–368.4 MHz)

## Top 25 emitters overall

| band | MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|---|
| FM broadcast | 96.64–96.79 (4 ch) | 50 | 10 | y | 40 | — | 100 | 18 | 0.3 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 96.64–96.78 (4 ch) | 25 | 10 | y | 50 | — | 90 | 17 | 0.3 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 96.770 | 25 | 5 | y | 70 | — | 66 | 16 | 0.2 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 96.63–96.76 (3 ch) | 25 | 5 | y | 40 | — | 62 | 16 | 977.2 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 95.450 | 50 | 10 | y | 50 | — | 52 | 15 | 0.1 | 6 | FM station 95.5 (modulation flicker); 7× 13.6460 MHz comb |
| FM broadcast | 96.63–96.75 (2 ch) | 25 | 10 | y | 30 | — | 42 | 16 | 0.1 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 96.770 | 50 | 5 | y | 60 | — | 41 | 18 | — | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 95.56–95.57 (2 ch) | 50 | 5 | y | 110 | — | 38 | 15 | 0.7 | 3 | FM station 95.5 (modulation flicker); 7× 13.6460 MHz comb |
| FM broadcast | 96.71–96.77 (2 ch) | 50 | 10 | y | 60 | — | 37 | 18 | 0.2 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 103.54–104.35 (4 ch) | 50 | 10 | y | 30 | — | 37 | 16 | 0.2 | 6 | FM station 103.5 (modulation flicker); 14× 7.4355 MHz comb |
| FM broadcast | 93.63–93.78 (6 ch) | 50 | 5 | y | 30 | — | 26 | 15 | 0.8 | 5 | FM station 93.7 (modulation flicker) |
| FM broadcast | 95.52–95.57 (4 ch) | 75 | 10 | y | 40 | — | 25 | 15 | 2.0 | 4 | FM station 95.5 (modulation flicker); 7× 13.6460 MHz comb |
| FM broadcast | 103.54–104.36 (4 ch) | 25 | 5 | y | 40 | — | 25 | 16 | 1.3 | 6 | FM station 103.5 (modulation flicker); 14× 7.4355 MHz comb |
| FM broadcast | 96.64–96.76 (3 ch) | 75 | 10 | y | 30 | — | 23 | 19 | 1.9 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 95.53–95.57 (3 ch) | 50 | 10 | y | 30 | — | 20 | 15 | 0.4 | 4 | FM station 95.5 (modulation flicker); 7× 13.6460 MHz comb |
| FM broadcast | 96.780 | 25 | 5 | y | 60 | — | 14 | 17 | 0.5 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| VHF TV ch 7-13 | 200.19–200.40 (10 ch) | 100 | 10 | y | 30 | — | 14 | 17 | 2.6 | 4 | 37× 5.4175 MHz comb |
| VHF TV ch 7-13 | 205.66–205.86 (9 ch) | 50 | 5 | y | 30 | — | 13 | 15 | — | 6 | 38× 5.4175 MHz comb |
| VHF TV ch 7-13 | 194.95–195.05 (5 ch) | 50 | 5 | y | 40 | — | 12 | 15 | 0.8 | 3 | 36× 5.4175 MHz comb |
| FM broadcast | 95.560 | 25 | 5 | y | 40 | — | 11 | 15 | 0.2 | 4 | FM station 95.5 (modulation flicker); 7× 13.6460 MHz comb |
| FM broadcast | 96.61–96.77 (2 ch) | 75 | 10 | y | 50 | — | 11 | 18 | 1.8 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| FM broadcast | 93.71–93.76 (2 ch) | 50 | 5 | y | 40 | — | 10 | 15 | 1.1 | 5 | FM station 93.7 (modulation flicker) |
| FM broadcast | 96.65–96.79 (2 ch) | 50 | 5 | y | 50 | — | 10 | 17 | 975.8 | -2 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| military aviation / satcom (UHF) | 286.85–287.12 (8 ch) | 75 | 5 | y | 30 | — | 10 | 17 | — | 4 | 53× 5.4175 MHz comb |
| FM broadcast | 96.63–96.78 (4 ch) | 75 | 5 | y | 30 | — | 9 | 19 | 0.6 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |

## Per band

### VHF TV ch 2-6 (mostly vacant) (54–88 MHz): 6 bursts, 2 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 86.500 | 25 | 5 | y | 60 | — | 3 | 14 | 1.8 | 4 |  |
| 87.060 | 25 | 5 | y | 100 | — | 3 | 13 | 0.9 | 4 |  |

### FM broadcast (88–108 MHz): 846 bursts, 43 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 96.64–96.79 (4 ch) | 50 | 10 | y | 40 | — | 100 | 18 | 0.3 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| 96.64–96.78 (4 ch) | 25 | 10 | y | 50 | — | 90 | 17 | 0.3 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| 96.770 | 25 | 5 | y | 70 | — | 66 | 16 | 0.2 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| 96.63–96.76 (3 ch) | 25 | 5 | y | 40 | — | 62 | 16 | 977.2 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| 95.450 | 50 | 10 | y | 50 | — | 52 | 15 | 0.1 | 6 | FM station 95.5 (modulation flicker); 7× 13.6460 MHz comb |
| 96.63–96.75 (2 ch) | 25 | 10 | y | 30 | — | 42 | 16 | 0.1 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| 96.770 | 50 | 5 | y | 60 | — | 41 | 18 | — | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| 95.56–95.57 (2 ch) | 50 | 5 | y | 110 | — | 38 | 15 | 0.7 | 3 | FM station 95.5 (modulation flicker); 7× 13.6460 MHz comb |
| 96.71–96.77 (2 ch) | 50 | 10 | y | 60 | — | 37 | 18 | 0.2 | 3 | FM station 96.7 (modulation flicker); 13× 7.4355 MHz comb |
| 103.54–104.35 (4 ch) | 50 | 10 | y | 30 | — | 37 | 16 | 0.2 | 6 | FM station 103.5 (modulation flicker); 14× 7.4355 MHz comb |
| 93.63–93.78 (6 ch) | 50 | 5 | y | 30 | — | 26 | 15 | 0.8 | 5 | FM station 93.7 (modulation flicker) |
| 95.52–95.57 (4 ch) | 75 | 10 | y | 40 | — | 25 | 15 | 2.0 | 4 | FM station 95.5 (modulation flicker); 7× 13.6460 MHz comb |

### aviation (AM) (108–137 MHz): 53 bursts, 11 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 119.12–119.17 (4 ch) | 25 | 5 | y | 30 | — | 8 | 14 | 1.3 | -1 | 22× 5.4175 MHz comb |
| 119.06–119.14 (2 ch) | 25 | 5 | y | 50 | — | 7 | 15 | 2.1 | 1 | 22× 5.4175 MHz comb |
| 119.07–119.18 (3 ch) | 25 | 5 | y | 140 | — | 7 | 15 | 0.4 | 3 | 22× 5.4175 MHz comb |
| 120.420 | 25 | 10 | y | 100 | — | 4 | 14 | 0.4 | 4 |  |
| 119.06–119.07 (2 ch) | 50 | 10 | y | 140 | — | 4 | 15 | 1.4 | 2 | 22× 5.4175 MHz comb |
| 119.06–119.12 (2 ch) | 25 | 10 | y | 60 | — | 4 | 15 | 1.3 | 2 | 22× 5.4175 MHz comb |
| 119.07–119.14 (3 ch) | 25 | 10 | y | 40 | — | 4 | 15 | — | 3 | 22× 5.4175 MHz comb |
| 119.10–119.15 (3 ch) | 25 | 10 | y | 30 | — | 4 | 15 | — | 2 | 22× 5.4175 MHz comb |
| 119.04–119.08 (3 ch) | 25 | 5 | y | 40 | — | 4 | 15 | — | 1 | 22× 5.4175 MHz comb |
| 120.84–120.98 (3 ch) | 25 | 10 | y | 130 | — | 4 | 14 | — | 5 |  |
| 119.08–119.14 (2 ch) | 50 | 20 | y | 40 | — | 3 | 15 | — | 1 | 22× 5.4175 MHz comb |

### VHF business / public safety / wireless mics (162.6–174 MHz): 8 bursts, 2 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 173.27–173.42 (4 ch) | 75 | 10 | y | 40 | — | 5 | 17 | — | 5 | 32× 5.4175 MHz comb |
| 173.17–173.43 (2 ch) | 175 | 20 | y | 30 | — | 3 | 17 | — | 3 | 32× 5.4175 MHz comb |

### VHF TV ch 7-13 (174–216 MHz): 193 bursts, 36 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 200.19–200.40 (10 ch) | 100 | 10 | y | 30 | — | 14 | 17 | 2.6 | 4 | 37× 5.4175 MHz comb |
| 205.66–205.86 (9 ch) | 50 | 5 | y | 30 | — | 13 | 15 | — | 6 | 38× 5.4175 MHz comb |
| 194.95–195.05 (5 ch) | 50 | 5 | y | 40 | — | 12 | 15 | 0.8 | 3 | 36× 5.4175 MHz comb |
| 200.22–200.44 (2 ch) | 50 | 5 | y | 40 | — | 8 | 18 | 0.5 | 3 | 37× 5.4175 MHz comb |
| 200.23–200.43 (5 ch) | 100 | 5 | y | 30 | — | 8 | 18 | — | 5 | 37× 5.4175 MHz comb |
| 205.70–205.87 (7 ch) | 75 | 5 | y | 30 | — | 8 | 16 | — | 5 | 38× 5.4175 MHz comb |
| 194.84–195.03 (3 ch) | 75 | 10 | y | 40 | — | 7 | 16 | 2.7 | 5 | 36× 5.4175 MHz comb |
| 194.79–195.04 (6 ch) | 75 | 5 | y | 30 | — | 7 | 15 | — | 6 | 36× 5.4175 MHz comb |
| 205.72–205.89 (6 ch) | 50 | 10 | y | 30 | — | 7 | 14 | — | 0 | 38× 5.4175 MHz comb |
| 200.22–200.26 (4 ch) | 100 | 20 | y | 30 | — | 6 | 20 | 1.3 | 5 |  |
| 205.64–205.89 (5 ch) | 75 | 10 | y | 30 | — | 6 | 15 | — | 6 | 38× 5.4175 MHz comb |
| 198.32–198.35 (2 ch) | 50 | 10 | y | 40 | — | 5 | 15 | 3.7 | 4 |  |

### military aviation / satcom (UHF) (225–400 MHz): 146 bursts, 36 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 286.85–287.12 (8 ch) | 75 | 5 | y | 30 | — | 10 | 17 | — | 4 | 53× 5.4175 MHz comb |
| 286.87–287.13 (6 ch) | 100 | 5 | y | 40 | — | 9 | 16 | — | 4 | 53× 5.4175 MHz comb |
| 286.86–287.11 (5 ch) | 75 | 5 | y | 40 | — | 7 | 16 | — | 4 | 53× 5.4175 MHz comb |
| 281.65–281.70 (4 ch) | 100 | 10 | y | 30 | — | 6 | 16 | 2.1 | 4 | 52× 5.4175 MHz comb |
| 286.86–287.16 (6 ch) | 100 | 10 | y | 30 | — | 6 | 15 | — | 2 | 53× 5.4175 MHz comb |
| 281.59–281.67 (3 ch) | 75 | 10 | y | 30 | — | 5 | 16 | — | 5 | 52× 5.4175 MHz comb |
| 286.84–287.14 (4 ch) | 225 | 20 | y | 30 | — | 5 | 17 | — | 1 | 53× 5.4175 MHz comb |
| 297.72–297.98 (4 ch) | 125 | 5 | y | 40 | — | 5 | 17 | — | 5 | 55× 5.4175 MHz comb |
| 281.44–281.49 (2 ch) | 50 | 5 | y | 30 | — | 4 | 15 | 3.9 | 6 |  |
| 281.64–281.66 (2 ch) | 100 | 5 | y | 40 | — | 4 | 16 | 2.6 | 5 | 52× 5.4175 MHz comb |
| 346.69–346.72 (2 ch) | 75 | 5 | y | 30 | — | 4 | 14 | 2.6 | 5 | 64× 5.4175 MHz comb |
| 281.38–281.44 (3 ch) | 175 | 10 | y | 30 | — | 4 | 19 | — | -0 |  |

### federal (406–420 MHz): 4 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 409.95–410.00 (3 ch) | 175 | 5 | y | 30 | — | 4 | 15 | — | 1 |  |

### 70 cm ham (incl. 433.92 ISM part 15) (420–450 MHz): 4 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 449.13–449.20 (4 ch) | 150 | 5 | y | 40 | — | 4 | 14 | — | 4 |  |

### UHF TV ch 14-36 / wireless mics (470–608 MHz): 3 bursts, 1 emitters

| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |
|---|---|---|---|---|---|---|---|---|---|---|
| 482.18–482.19 (2 ch) | 300 | 20 | n | — | — | 3 | 16 | — | 4 | 89× 5.4175 MHz comb |

