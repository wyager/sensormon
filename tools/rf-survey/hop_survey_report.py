#!/usr/bin/env python3
"""Burst-emitter survey from a sensormon chirp store.

Companion to sweep_report.py (which covers continuous carriers from rtl_power
sweeps): this covers *bursty* emitters, i.e. everything sensormon's burst
detector saw while a receiver hopped across the tuner range with a long dwell
(deploy/radio-survey-hop-*.toml). Groups are merged into "emitters" by a
signature (band, bandwidth class, duration class, FSK tone spacing, symbol
rate) so a frequency hopper shows up once, and reported per band with the
sweep_report band table + known-frequency labels.

  hop_survey_report.py --host 10.20.15.5:8433 [--receiver survey-nesdr] [--min-count 2]
"""
import argparse, json, math, os, sys, urllib.request
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sweep_report import BANDS, KNOWN, band_of, load as load_sweeps, find_combs  # noqa: E402
import statistics


def get(url):
    with urllib.request.urlopen(url, timeout=120) as r:
        return r.read()


CONTINUOUS_DB = 6.0  # a burst whose center sits on a ≥6 dB continuous carrier is that carrier flickering, not an emitter


def known_label(mhz):
    for f, tol, label in KNOWN:
        if abs(mhz - f) <= tol:
            return label
    if 88.0 <= mhz < 108.0:
        ch = round((mhz - 0.1) / 0.2) * 0.2 + 0.1   # US FM channels sit on odd tenths
        if abs(mhz - ch) <= 0.12:
            return f"FM station {ch:.1f} (modulation flicker)"
    return ""


def make_continuous_level(files):
    """Per-10 kHz continuous level (median across sweeps) above the local floor, from rtl_power CSVs."""
    if not files:
        return lambda mhz: None
    bins, _ = load_sweeps(files)
    steps = sorted(bins)
    med = {f: statistics.median(v) for f, v in bins.items()}
    # local floor: 20th percentile of medians within ±1 MHz (same definition as sweep_report)
    vals = [med[f] for f in steps]
    import bisect
    def level(mhz):
        hz = mhz * 1e6
        i = bisect.bisect_left(steps, hz)
        if i >= len(steps):
            return None
        lo = bisect.bisect_left(steps, hz - 1e6); hi = bisect.bisect_right(steps, hz + 1e6)
        window = sorted(vals[lo:hi])
        if not window:
            return None
        floor = window[int(0.2 * (len(window) - 1))]
        near = vals[max(0, i - 2): i + 3]
        return max(near) - floor
    return level


continuous_level = lambda mhz: None


def signature(g):
    spacing = g.get("tone_spacing_hz")
    rate = g.get("symbol_rate")
    return (
        band_of(g["center_hz"] / 1e6),
        round(g["center_hz"] / 1e6),                                 # 1 MHz cell
        g["bandwidth_khz"],
        g["duration_class_ms"],
        bool(g.get("fsk")),
        None if spacing is None else round(spacing / 10e3) * 10,   # 10 kHz classes
        None if rate is None else round(rate / 1e3),                 # 1 kbaud classes
    )


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--host", required=True)
    ap.add_argument("--receiver", default="survey-nesdr")
    ap.add_argument("--min-count", type=int, default=2)
    ap.add_argument("--top", type=int, default=60)
    ap.add_argument("--sweep", nargs="*", default=[], help="rtl_power mean-sweep CSVs: bursts sitting inside a continuous carrier get flagged")
    ap.add_argument("--hide-continuous", action="store_true", help="drop emitters flagged as inside a continuous carrier")
    a = ap.parse_args()
    global continuous_level
    continuous_level = make_continuous_level(a.sweep)
    base = f"http://{a.host}"
    stats = json.loads(get(f"{base}/chirps/stats"))
    groups = [g for g in json.loads(get(f"{base}/chirps/groups?min_count=1")) if g["receiver"] == a.receiver]

    emitters = defaultdict(list)
    for g in groups:
        emitters[signature(g)].append(g)

    rows = []
    for sig, gs in emitters.items():
        count = sum(g["count"] for g in gs)
        if count < a.min_count:
            continue
        centers = sorted(g["center_hz"] / 1e6 for g in gs)
        snr = sum(g["mean_snr_db"] * g["count"] for g in gs) / max(1, count)
        periods = sorted(g["typical_period_s"] for g in gs if g.get("typical_period_s"))
        period = periods[len(periods) // 2] if periods else None
        first = min(g["first_seen"] for g in gs)
        last = max(g["last_seen"] for g in gs)
        band, cell, bw, dur, fsk, spacing, rate = sig
        mid = centers[len(centers) // 2]
        rows.append(dict(band=band, cell=cell, lo=centers[0], hi=centers[-1], hops=len(gs), bw=bw, dur=dur, fsk=fsk,
                         spacing=spacing, rate=rate, count=count, snr=snr, period=period,
                         first=first, last=last, known=known_label(mid), sweep=continuous_level(mid)))
    rows.sort(key=lambda r: -r["count"])
    n_all = len(rows)
    if a.hide_continuous:
        rows = [r for r in rows if (r["sweep"] or 0) < CONTINUOUS_DB]
    combs, membership = find_combs([(r["lo"] + r["hi"]) / 2 for r in rows], [math.log2(r["count"] + 1) for r in rows],
                                   exclude=[bool(r["known"]) for r in rows])
    for r, mem in zip(rows, membership):
        if mem:
            r["comb"] = mem

    span_h = ((stats["newest"] or 0) - (stats["oldest"] or 0)) / 3600
    print(f"# Burst-emitter survey — receiver `{a.receiver}` ({a.host})\n")
    print(f"Store: {stats['chirps']} example captures, {stats['groups']} raw groups ({len(groups)} for this receiver), "
          f"{stats['bytes']/1e6:.1f} MB; examples span {span_h:.1f} h. Bursts are 3–250 ms events ≥9 dB over the local floor; "
          f"continuous carriers are invisible here (see the rtl_power sweep report).")
    if a.sweep:
        print(f"'sweep dB' = continuous level at that frequency in the rtl_power mean sweeps; ≥{CONTINUOUS_DB:g} dB means the 'bursts' are "
              f"a continuous wideband signal flickering (FM modulation, spread-spectrum SMPS dither, TV) rather than a burst emitter"
              + (f" — {n_all - len(rows)} such emitters hidden" if a.hide_continuous else "") + ".")
    print()

    def fmt(r):
        c = f"{r['lo']:.3f}" if r["hops"] == 1 else f"{r['lo']:.2f}–{r['hi']:.2f} ({r['hops']} ch)"
        sp = "—" if r["spacing"] is None else str(r["spacing"])
        rt = "—" if r["rate"] is None else f"{r['rate']}k"
        pe = "—" if r["period"] is None else f"{r['period']:.1f}"
        sw = "—" if r["sweep"] is None else f"{r['sweep']:.0f}"
        flag = "inside continuous carrier" if (r["sweep"] or 0) >= CONTINUOUS_DB else ""
        comb = f"{r['comb'][1]}× {r['comb'][0]:.4f} MHz comb" if r.get("comb") else ""
        note = "; ".join(x for x in (r["known"], flag, comb) if x)
        return f"| {c} | {r['bw']} | {r['dur']} | {'y' if r['fsk'] else 'n'} | {sp} | {rt} | {r['count']} | {r['snr']:.0f} | {pe} | {sw} | {note} |"

    hdr = "| MHz | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR | period s | sweep dB | note |\n|---|---|---|---|---|---|---|---|---|---|---|"
    if combs:
        print("## Harmonic combs (switching-supply / clock harmonics; not radio emitters)\n")
        for f0, n in combs:
            ks = sorted({r["comb"][1] for r in rows if r.get("comb") and abs(r["comb"][0] - f0) < 1e-9})
            print(f"- **{f0:.4f} MHz** fundamental explains {n} emitters (harmonics {ks[0]}…{ks[-1]}, i.e. {ks[0]*f0:.1f}–{ks[-1]*f0:.1f} MHz)")
        print()
    print(f"## Top {a.top} emitters overall\n")
    print("| band | " + hdr[2:].replace("\n|---", "\n|---|---", 1))
    for r in rows[:a.top]:
        print(f"| {r['band']} " + fmt(r))

    fam = defaultdict(list)
    for r in rows:
        fam[(r["band"], r["bw"], r["dur"], r["fsk"], r["spacing"], r["rate"])].append(r)
    wide = [(k, v) for k, v in fam.items() if len({r["cell"] for r in v}) >= 5]
    wide.sort(key=lambda kv: -sum(r["count"] for r in kv[1]))
    if wide:
        print("\n## Wide families (same burst class in ≥5 distinct 1 MHz cells of a band: hoppers, or a band-wide artifact)\n")
        print("| band | MHz range | cells | bw kHz | dur ms | FSK | tone Δ kHz | baud | bursts | SNR |\n|---|---|---|---|---|---|---|---|---|---|")
        for (band, bw, dur, fsk, sp, rt), v in wide[:20]:
            lo = min(r["lo"] for r in v); hi = max(r["hi"] for r in v); cnt = sum(r["count"] for r in v)
            snr = sum(r["snr"] * r["count"] for r in v) / cnt
            print(f"| {band} | {lo:.2f}–{hi:.2f} | {len({r['cell'] for r in v})} | {bw} | {dur} | {'y' if fsk else 'n'} | {sp if sp is not None else '—'} | {str(rt)+'k' if rt is not None else '—'} | {cnt} | {snr:.0f} |")

    print("\n## Per band\n")
    by_band = defaultdict(list)
    for r in rows:
        by_band[r["band"]].append(r)
    for lo, hi, name in BANDS:
        rs = by_band.get(name)
        if not rs:
            continue
        total = sum(r["count"] for r in rs)
        print(f"### {name} ({lo:g}–{hi:g} MHz): {total} bursts, {len(rs)} emitters\n")
        print(hdr)
        for r in rs[:12]:
            print(fmt(r))
        print()
    other = [b for b in by_band if b not in {n for _, _, n in BANDS}]
    for b in other:
        print(f"### {b}: {sum(r['count'] for r in by_band[b])} bursts\n")
        print(hdr)
        for r in by_band[b][:12]:
            print(fmt(r))
        print()


if __name__ == "__main__":
    main()
