#!/usr/bin/env python3
"""Summarise rtl_power sweeps into an RF-environment report.

rtl_power CSV rows: date, time, hz_low, hz_high, hz_step, samples, dB...
Across all given files (mean and/or peak-hold sweeps) we compute, per 10 kHz
bin: median (floor), max, and how many sweeps it exceeded floor+thr. Then we
group adjacent "active" bins into signals, list them with a band label, and
print occupancy per allocation band. Output is markdown.

  sweep_report.py sweep_*.csv [--thr 8] [--top 60]
"""
import argparse, csv, glob, math, sys
from collections import defaultdict

# Coarse US allocation labels for context (not authoritative).
BANDS = [
    (24.0, 30.0, "HF (10 m ham / CB region)"), (30.0, 50.0, "VHF low: public safety, business"), (50.0, 54.0, "6 m ham"),
    (54.0, 88.0, "VHF TV ch 2-6 (mostly vacant)"), (88.0, 108.0, "FM broadcast"), (108.0, 137.0, "aviation (AM)"),
    (137.0, 144.0, "satellite / NOAA APT / military"), (144.0, 148.0, "2 m ham"), (148.0, 150.8, "military / satellite"),
    (150.8, 156.0, "VHF business / MURS (151.82-154.6)"), (156.0, 162.0, "marine VHF"), (162.4, 162.6, "NOAA weather radio"),
    (162.6, 174.0, "VHF business / public safety / wireless mics"), (174.0, 216.0, "VHF TV ch 7-13"), (216.0, 225.0, "1.25 m ham"),
    (225.0, 400.0, "military aviation / satcom (UHF)"), (400.0, 406.0, "radiosondes / meteorology"), (406.0, 420.0, "federal"),
    (420.0, 450.0, "70 cm ham (incl. 433.92 ISM part 15)"), (450.0, 470.0, "UHF business / public safety / FRS-GMRS 462-467"),
    (470.0, 608.0, "UHF TV ch 14-36 / wireless mics"), (608.0, 614.0, "radio astronomy (ch 37)"), (614.0, 698.0, "600 MHz cellular (band 71) / TV ch 38-51"),
    (698.0, 806.0, "700 MHz cellular (bands 12/13/14/17) / FirstNet"), (806.0, 824.0, "800 MHz public safety / SMR"),
    (824.0, 849.0, "cellular 850 uplink (band 5)"), (851.0, 869.0, "800 MHz SMR / public safety"), (869.0, 894.0, "cellular 850 downlink (band 5)"),
    (894.0, 902.0, "SMR"), (902.0, 928.0, "902-928 ISM: LoRa, Z-Wave 908/916, meters, Fineoffset 915"), (928.0, 942.0, "paging (929-932) / MAS / narrowband PCS"),
    (942.0, 960.0, "fixed / GSM-R-ish"), (960.0, 1215.0, "aeronautical navigation (DME/TACAN/ADS-B 1090)"), (1215.0, 1300.0, "23 cm ham / radar / GNSS L2"),
    (1300.0, 1400.0, "radar"), (1400.0, 1427.0, "radio astronomy (protected)"), (1427.0, 1518.0, "AWS-3/L-band / fixed"), (1518.0, 1559.0, "MSS / GPS L1 1575"),
    (1559.0, 1610.0, "GNSS L1"), (1610.0, 1660.0, "MSS uplink (Iridium 1616-1626)"), (1660.0, 1700.0, "radio astronomy / weather sats"),
]
KNOWN = [(315.0, 0.3, "315 MHz keyfobs/TPMS"), (345.0, 0.3, "345 MHz security sensors (Honeywell/2GIG)"), (390.0, 0.3, "390 MHz garage door openers"),
         (433.92, 0.3, "433.92 ISM"), (462.5625, 0.3, "FRS/GMRS ch1"), (467.6, 0.5, "GMRS repeater inputs"), (162.55, 0.05, "NOAA WX"), (162.4, 0.05, "NOAA WX"),
         (915.0, 1.0, "Fineoffset sensors"), (908.4, 0.2, "Z-Wave"), (916.0, 0.2, "Z-Wave"), (1090.0, 0.5, "ADS-B"), (1575.42, 1.5, "GPS L1")]

def band_of(mhz):
    for lo, hi, name in BANDS:
        if lo <= mhz < hi:
            return name
    return "?"

def known(mhz):
    for f, w, name in KNOWN:
        if abs(mhz - f) <= w:
            return name
    # ATSC pilot: 309.44 kHz above the channel's lower edge (ch 2-6, 7-13, 14-36)
    for lo, ch0, n in ((54.0, 2, 4), (76.0, 5, 2), (174.0, 7, 7), (470.0, 14, 23)):
        for i in range(n):
            edge = lo + 6 * i
            if abs(mhz - (edge + 0.30944)) < 0.02:
                return f"ATSC pilot, TV channel {ch0 + i}"
    return ""

def harmonic_note(f_mhz, peaks_mhz):
    """If f is within 30 kHz of k× another (weaker or stronger) peak, say so."""
    for g in peaks_mhz:
        if g < f_mhz / 1.5:
            k = round(f_mhz / g)
            if 2 <= k <= 14 and abs(f_mhz - k * g) < 0.03:
                return f"= {k}× {g:.3f}"
    return ""

def load(files):
    bins = defaultdict(list)  # freq -> list of dB across rows (all sweeps)
    n_sweeps = 0
    for fn in files:
        rows = 0
        with open(fn) as f:
            for r in csv.reader(f):
                if len(r) < 7:
                    continue
                lo, hi, step = float(r[2]), float(r[3]), float(r[4])
                vals = [float(x) for x in r[6:] if x.strip() not in ("", "nan", "-nan")]
                for i, v in enumerate(vals):
                    fq = lo + i * step
                    if fq >= hi:
                        break
                    if math.isfinite(v):
                        bins[round(fq / step) * step].append(v)
                rows += 1
        if rows:
            n_sweeps += 1
    return bins, n_sweeps

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="+")
    ap.add_argument("--thr", type=float, default=8.0, help="dB above local floor to count as a signal")
    ap.add_argument("--top", type=int, default=80)
    a = ap.parse_args()
    files = sorted(set(sum([glob.glob(f) for f in a.files], [])))
    bins, n_sweeps = load(files)
    if not bins:
        print("no data"); return
    freqs = sorted(bins)
    med = {f: sorted(v)[len(v) // 2] for f, v in bins.items()}
    mx = {f: max(v) for f, v in bins.items()}
    # local floor: rolling 20th percentile of medians over ±1 MHz
    step = freqs[1] - freqs[0]
    w = max(1, int(1e6 / step))
    meds = [med[f] for f in freqs]
    floor = []
    for i in range(len(freqs)):
        seg = sorted(meds[max(0, i - w): i + w + 1])
        floor.append(seg[len(seg) // 5])
    active = [(freqs[i], med[freqs[i]] - floor[i], mx[freqs[i]] - floor[i]) for i in range(len(freqs)) if mx[freqs[i]] - floor[i] >= a.thr]
    # group adjacent active bins into signals
    sigs = []
    for f, dmed, dmax in active:
        if sigs and f - sigs[-1]["hi"] <= 2 * step:
            s = sigs[-1]; s["hi"] = f; s["peak"] = max(s["peak"], dmax); s["med"] = max(s["med"], dmed)
            if dmax > s["peak_at_db"]:
                s["peak_at"], s["peak_at_db"] = f, dmax
        else:
            sigs.append({"lo": f, "hi": f, "peak": dmax, "med": dmed, "peak_at": f, "peak_at_db": dmax})
    sigs.sort(key=lambda s: -s["peak"])
    print(f"# RF survey — {len(files)} sweep files, {len(freqs)} bins of {step/1e3:.0f} kHz, {freqs[0]/1e6:.1f}–{freqs[-1]/1e6:.1f} MHz\n")
    print("Levels are dB above the local noise floor (20th percentile of the median spectrum over ±1 MHz). 'median' = present in most sweeps (continuous); 'peak' = strongest seen (bursty if peak ≫ median).\n")
    print("## Strongest signals\n")
    print("| MHz (peak) | span MHz | peak dB | median dB | character | band | note |")
    print("|---|---|---|---|---|---|---|")
    peaks_mhz = [s["peak_at"] / 1e6 for s in sigs[: max(a.top, 200)]]
    for s in sigs[: a.top]:
        f = s["peak_at"] / 1e6
        ch = "continuous" if s["med"] >= a.thr else ("intermittent" if s["med"] >= a.thr / 2 else "bursty")
        span = f"{s['lo']/1e6:.2f}–{s['hi']/1e6:.2f}" if s["hi"] > s["lo"] else f"{s['lo']/1e6:.2f}"
        notes = [x for x in (known(f), harmonic_note(f, peaks_mhz)) if x]
        print(f"| {f:.3f} | {span} | {s['peak']:.0f} | {s['med']:.0f} | {ch} | {band_of(f)} | {'; '.join(notes)} |")
    # occupancy per allocation band
    print("\n## Occupancy by band (fraction of bins ever ≥ threshold, and max level)\n")
    print("| band | MHz | bins active | max dB |")
    print("|---|---|---|---|")
    for lo, hi, name in BANDS:
        idx = [i for i, f in enumerate(freqs) if lo * 1e6 <= f < hi * 1e6]
        if not idx:
            continue
        act = sum(1 for i in idx if mx[freqs[i]] - floor[i] >= a.thr)
        m = max(mx[freqs[i]] - floor[i] for i in idx)
        print(f"| {name} | {lo:g}–{hi:g} | {act}/{len(idx)} ({100*act/len(idx):.0f}%) | {m:.0f} |")

if __name__ == "__main__":
    main()
