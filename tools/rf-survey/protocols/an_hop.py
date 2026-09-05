import json, collections, math, itertools
import numpy as np
fr=json.load(open('hop2.json')); print(len(fr),'frames; rates', np.percentile([f['rate'] for f in fr],[5,50,95]).round(0), 'tone Δ median', round(np.median([abs(f['tones'][0]-f['tones'][1]) for f in fr])/1e3,1),'kHz')
def tobytes(bits):
    bits=bits[:len(bits)//8*8]; return bytes(int(bits[i:i+8],2) for i in range(0,len(bits),8))
frames=[]
for f in fr:
    b=f['bits'][f['sync_at']:]
    # burst end: the demod slices past the end into noise; use dur to bound: symbols = dur_ms*rate/1000 from burst start, minus preamble before sync
    nsym=int(f['dur']*f['rate']/1000) - f['sync_at']
    b=b[:max(0,nsym)]
    frames.append((f, tobytes(b)))
L=collections.Counter(len(b) for _,b in frames); print('bytes after sync-start (incl 8-byte sync) histogram:', sorted(L.items()))
# byte position stats
print('\npos: distinct values / top values (first 24 positions)')
for pos in range(24):
    vals=[b[pos] for _,b in frames if len(b)>pos]
    c=collections.Counter(vals); ent=-sum(n/len(vals)*math.log2(n/len(vals)) for n in c.values())
    print(f"{pos:3d}: n={len(vals):3d} distinct={len(c):3d} H={ent:4.2f} top={[f'{v:02x}x{n}' for v,n in c.most_common(5)]}")
# length byte hypothesis: byte 7 (index 7) vs frame length
print('\nbyte[7] vs frame length:')
pairs=collections.Counter((b[7], len(b)) for _,b in frames if len(b)>8)
for (v,l),n in sorted(pairs.items(), key=lambda kv:-kv[1])[:15]: print(f"  byte7=0x{v:02x} ({v}) len={l} x{n}")
# entropy of payload region by frame length class (positions 8..)
for l in [l for l,_ in L.most_common(3)]:
    fs=[b for _,b in frames if len(b)==l]
    if len(fs)<5: continue
    ents=[]
    for pos in range(8,l):
        c=collections.Counter(b[pos] for b in fs); ents.append(-sum(n/len(fs)*math.log2(n/len(fs)) for n in c.values()))
    print(f"\nlen {l} ({len(fs)} frames): per-position entropy bits (max {math.log2(len(fs)):.1f}):", ' '.join(f"{e:.1f}" for e in ents))
    for b in fs[:6]: print('   ', b.hex())
# PN9 whitening test: dewhiten bytes after sync (start at index 7 or 8), both bit orders, look for lower entropy / a length match
def pn9():
    s=0x1ff; out=[]
    for _ in range(128):
        byte=0
        for i in range(8):
            bit=s&1; byte|=bit<<i
            fb=((s>>0)^(s>>5))&1; s=(s>>1)|(fb<<8)
        out.append(byte)
    return bytes(out)
PN=pn9(); rev=lambda x:int(f"{x:08b}"[::-1],2)
print('\nPN9 first bytes:', PN[:8].hex())
for start in (7,8):
    for order in ('msb','lsb'):
        ents=[]; fs=[b for _,b in frames if len(b)>=start+20]
        dw=[]
        for b in fs:
            d=bytes(((rev(x) if order=='lsb' else x)^PN[i]) for i,x in enumerate(b[start:start+20])); dw.append(d)
        for pos in range(20):
            c=collections.Counter(d[pos] for d in dw); ents.append(-sum(n/len(dw)*math.log2(n/len(dw)) for n in c.values()))
        print(f"dewhiten start={start} {order}: entropy", ' '.join(f"{e:.1f}" for e in ents), '| ex', dw[0].hex())
