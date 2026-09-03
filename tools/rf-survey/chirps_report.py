#!/usr/bin/env python3
"""Identify what sensormon could not decode.

Pulls /chirps/groups from a sensormon instance, merges frequency-hopping
emitters (same bandwidth/duration/tone-spacing/symbol-rate signature across
many centers), and runs rtl_433 (all decoders, -A analyzer) over stored example
IQ to name what it can. Prints a markdown report.

  chirps_report.py --host 10.20.15.5:8433 --rtl433 /path/to/rtl_433 [--min-count 3]
"""
import argparse, json, os, re, subprocess, sys, tempfile, urllib.request
from collections import defaultdict
import numpy as np
from scipy.signal import resample_poly

def get(url):
    with urllib.request.urlopen(url, timeout=60) as r:
        return r.read()

def signature(g, coarse):
    # hopper-tolerant key. coarse: ignore bandwidth, bucket tone spacing to 10 kHz,
    # duration to short/medium/long, baud to 1 kbaud — one row per emitter family.
    ts = (g.get("tone_spacing_hz") or 0) / 1e3
    br = (g.get("symbol_rate") or 0)
    if coarse:
        d = g["duration_class_ms"]
        dur = "≤10" if d <= 10 else ("≤40" if d <= 40 else ">40")
        return (g["receiver"], None, dur, bool(g["fsk"]), round(ts / 10) * 10, round(br / 1000) * 1000 if br else 0)
    return (g["receiver"], g["bandwidth_khz"], f"≤{g['duration_class_ms']}", bool(g["fsk"]), round(ts / 5) * 5, round(br / 500) * 500 if br else 0)

def measure(cs8_path, rate):
    """Measure the burst ourselves: tone frequencies from the spectrum, symbol
    period from the run lengths of the FM discriminator, burst length. Returns a
    short text, or None if there is no 2-tone structure."""
    raw = np.fromfile(cs8_path, dtype=np.int8).astype(np.float32)
    x = raw[0::2] + 1j * raw[1::2]
    if len(x) < 64:
        return None
    env = np.abs(x); on = env > 0.35 * np.percentile(env, 99)
    idx = np.where(on)[0]
    if len(idx) < 32:
        return None
    seg = x[idx[0]:idx[-1] + 1]
    n = 1 << int(np.ceil(np.log2(len(seg))))
    P = np.abs(np.fft.fftshift(np.fft.fft(seg * np.hanning(len(seg)), n)))**2
    f = (np.arange(n) - n // 2) * rate / n
    k1 = int(np.argmax(P)); f1 = f[k1]
    far = np.abs(f - f1) > 8e3
    k2 = int(np.argmax(np.where(far, P, 0))); f2 = f[k2]
    if P[k2] < P[k1] / 100:
        return f"single tone at {f1/1e3:+.0f} kHz, {len(seg)/rate*1e3:.1f} ms (OOK/CW?)"
    inst = np.angle(seg[1:] * np.conj(seg[:-1])) * rate / (2 * np.pi)
    k = max(1, int(rate / 80e3)); inst = np.convolve(inst, np.ones(k) / k, mode="same")
    bits = inst > (f1 + f2) / 2
    ch = np.where(np.diff(bits.astype(np.int8)) != 0)[0]
    if len(ch) < 6:
        return f"tones {min(f1,f2)/1e3:+.0f}/{max(f1,f2)/1e3:+.0f} kHz, {len(seg)/rate*1e3:.1f} ms, <6 transitions"
    runs = np.diff(ch) / rate
    # symbol period ≈ mode of the shortest runs (the 10th–30th percentile band)
    q = np.percentile(runs, [10, 30]); short = runs[(runs >= q[0]) & (runs <= q[1])]
    T = float(np.median(short)) if len(short) else float(np.min(runs))
    return f"tones {min(f1,f2)/1e3:+.0f}/{max(f1,f2)/1e3:+.0f} kHz (Δ{abs(f1-f2)/1e3:.0f}), ~{1/T/1e3:.1f} kbaud, {len(seg)/rate*1e3:.1f} ms, {len(runs)+1} symbols-ish"

def to_rtl433_rate(cs8_path, rate):
    """rtl_433's FSK path only works up to ~500 kS/s: decimate wider captures."""
    if rate <= 500_000:
        return cs8_path, rate
    d = int(np.ceil(rate / 250_000))
    raw = np.fromfile(cs8_path, dtype=np.int8).astype(np.float32)
    x = raw[0::2] + 1j * raw[1::2]
    y = resample_poly(x, 1, d)
    out = np.empty(2 * len(y), dtype=np.int8)
    out[0::2] = np.clip(y.real, -128, 127); out[1::2] = np.clip(y.imag, -128, 127)
    p2 = cs8_path.replace(".cs8", f"_dec{d}.cs8")
    out.tofile(p2)
    return p2, int(round(rate / d))

def rtl433_identify(rtl433, cs8_path, rate, center):
    try:
        out = subprocess.run([rtl433, "-r", cs8_path, "-s", str(rate), "-f", str(int(center)), "-F", "json", "-M", "level"], capture_output=True, text=True, timeout=30)
    except Exception as e:
        return None, f"rtl_433 failed: {e}"
    models = []
    for line in out.stdout.splitlines():
        try:
            j = json.loads(line)
            if "model" in j:
                models.append(f'{j["model"]} id={j.get("id")}')
        except json.JSONDecodeError:
            pass
    # analyzer guess for unrecognised ones
    guess = ""
    try:
        an = subprocess.run([rtl433, "-r", cs8_path, "-s", str(rate), "-A"], capture_output=True, text=True, timeout=30).stderr
        m = re.search(r"Guessing modulation: (.*)", an)
        b = re.search(r"Total count:\s+(\d+),\s+width:\s+([\d.]+) ms", an)
        if m:
            guess = m.group(1).strip()
            if b:
                guess += f" ({b.group(1)} pulses, {b.group(2)} ms)"
    except Exception:
        pass
    return models, guess

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--host", default="10.20.15.5:8433")
    ap.add_argument("--rtl433", default=os.path.expanduser("~/projects/rtl_433_patches/rtl_433/build/src/rtl_433"))
    ap.add_argument("--min-count", type=int, default=2)
    ap.add_argument("--examples", type=int, default=2, help="examples per merged emitter to run through rtl_433")
    ap.add_argument("--coarse", action="store_true", help="merge by emitter family (ignore bandwidth/center); one row per hopper")
    a = ap.parse_args()
    base = f"http://{a.host}"
    stats = json.loads(get(f"{base}/chirps/stats"))
    groups = json.loads(get(f"{base}/chirps/groups?min_count=1"))
    merged = defaultdict(lambda: {"count": 0, "centers": set(), "groups": [], "snr": 0.0, "first": 1e18, "last": 0.0, "periods": []})
    for g in groups:
        k = signature(g, a.coarse)
        m = merged[k]
        m["count"] += g["count"]; m["centers"].add(g["center_hz"]); m["groups"].append(g)
        m["snr"] += g["mean_snr_db"] * g["count"]; m["first"] = min(m["first"], g["first_seen"]); m["last"] = max(m["last"], g["last_seen"])
        if g.get("typical_period_s"): m["periods"].append(g["typical_period_s"])
    rows = sorted(merged.items(), key=lambda kv: -kv[1]["count"])
    print(f"# Undecoded transmissions ({a.host})\n")
    print(f"Store: {stats['chirps']} example captures, {stats['groups']} raw groups, {stats['bytes']/1e6:.1f} MB; span {(stats['newest'] or 0) - (stats['oldest'] or 0):.0f} s.\n")
    print("| # | receiver | centers (MHz) | bw kHz | dur ms | FSK | tone Δ kHz | baud | count | mean SNR | period s | rtl_433 says |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    tmp = tempfile.mkdtemp()
    n = 0
    for k, m in rows:
        if m["count"] < a.min_count:
            continue
        n += 1
        rx, bw, dur, fsk, ts, br = k
        centers = sorted(m["centers"])
        cdesc = f"{centers[0]/1e6:.2f}" if len(centers) == 1 else f"{centers[0]/1e6:.2f}–{centers[-1]/1e6:.2f} ({len(centers)} hops)"
        period = sorted(m["periods"])[len(m["periods"]) // 2] if m["periods"] else None
        # identify using a couple of the strongest groups' examples
        idents, guesses, meas = [], [], []
        best_groups = sorted(m["groups"], key=lambda g: -g["mean_snr_db"])[: a.examples]
        for g in best_groups:
            ex = json.loads(get(f"{base}/chirps?group={g['id']}&limit=1"))
            if not ex:
                continue
            c = ex[0]
            path = os.path.join(tmp, f"chirp_{c['id']}.cs8")
            open(path, "wb").write(get(f"{base}/chirps/{c['id']}/iq"))
            mm = measure(path, c["sample_rate"])
            if mm: meas.append(mm)
            path, rate = to_rtl433_rate(path, c["sample_rate"])
            models, guess = rtl433_identify(a.rtl433, path, rate, c["center_hz"])
            if models:
                idents.extend(models)
            elif guess:
                guesses.append(guess)
        says = "; ".join(sorted(set(idents))) if idents else ("; ".join(sorted(set(guesses))) if guesses else "—")
        if meas:
            says += " · measured: " + meas[0]
        print(f"| {n} | {rx} | {cdesc} | {bw if bw is not None else 'any'} | {dur} | {'y' if fsk else 'n'} | {ts or '—'} | {br or '—'} | {m['count']} | {m['snr']/m['count']:.0f} | {f'{period:.1f}' if period else '—'} | {says} |")
    print(f"\n{n} emitter signatures with ≥{a.min_count} sightings.")

if __name__ == "__main__":
    main()
