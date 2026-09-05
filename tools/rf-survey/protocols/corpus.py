import json, os, sys, urllib.request, re
import numpy as np
from scipy.signal import firwin, lfilter
BASE='http://10.20.15.5:8433'
def get(u):
    with urllib.request.urlopen(u, timeout=60) as r: return r.read()
def safe(fn):
    try: return fn()
    except Exception: return None
def load(cid):
    c=json.loads(get(f"{BASE}/chirps/{cid}")); p=f"iq/{cid}.cs8"
    if not os.path.exists(p): open(p,'wb').write(get(f"{BASE}/chirps/{cid}/iq"))
    raw=np.fromfile(p,dtype=np.int8).astype(np.float32); return c, raw[0::2]+1j*raw[1::2]
def center_on_signal(z, fs, rate):
    """Find the two strongest spectral peaks 20-160 kHz apart, mix their midpoint to DC, low-pass to the signal."""
    n=len(z); N=1<<int(np.ceil(np.log2(min(n,1<<16)))); seg=z[:N] if n>=N else np.concatenate([z,np.zeros(N-n)])
    P=np.abs(np.fft.fftshift(np.fft.fft(seg*np.hanning(N))))**2; f=(np.arange(N)-N//2)*fs/N
    sm=np.convolve(P,np.ones(5)/5,'same'); k1=int(np.argmax(sm)); f1=f[k1]
    mask=(np.abs(f-f1)>20e3)&(np.abs(f-f1)<160e3); k2=int(np.argmax(np.where(mask,sm,0))); f2=f[k2]
    fc=(f1+f2)/2; t=np.arange(n)/fs; y=z*np.exp(-2j*np.pi*fc*t)
    cut=min(0.45, (abs(f1-f2)/2+rate*0.8)/(fs/2)); y=lfilter(firwin(63,cut),1,y)[63:]
    return y, fc
def demod(z, fs, bw, rate_nom, sync_re, tol=0.03):
    y,fc=center_on_signal(z, fs, rate_nom if "rate_nom" in dir() else rate)
    env=np.abs(y); on=env>0.3*np.percentile(env,99); i=np.where(on)[0]
    if len(i)<50: return None
    y=y[i[0]:i[-1]+1]; d=np.angle(y[1:]*np.conj(y[:-1]))*fs/(2*np.pi)
    h,e=np.histogram(d,bins=200,range=(-fs/2,fs/2)); top=np.argsort(h)[::-1]; t1=(e[top[0]]+e[top[0]+1])/2
    far=[k for k in top if abs((e[k]+e[k+1])/2-t1)>6e3]
    if not far: return None
    t2=(e[far[0]]+e[far[0]+1])/2; mid=(t1+t2)/2
    best=None
    for rate in np.linspace(rate_nom*(1-tol), rate_nom*(1+tol), 9):
        sps=fs/rate; k=max(1,int(round(sps*0.6))); dd=np.convolve(d,np.ones(k)/k,'same')
        ch=np.where(np.diff(np.sign(dd-mid))!=0)[0]
        if len(ch)<4: continue
        for ph in (np.median(ch%sps), np.mean(ch%sps)):
            n=int((len(dd)-ph)/sps)
            idx=(ph+np.arange(n)*sps+sps/2).astype(int); idx=idx[idx<len(dd)]
            bits=''.join('1' if v>mid else '0' for v in dd[idx])
            for cand,inv in ((bits,False),(''.join('1' if b=='0' else '0' for b in bits),True)):
                m=sync_re.search(cand)
                if m:
                    err=m.group(0).count('x')
                    score=(0, -m.start())
                    if best is None or score>best[0]: best=(score, cand, m.start(), m.end(), rate, inv, (t1+fc,t2+fc), len(y)/fs*1e3)
    return best
def run(name, groups, rate, sync_re, maxn):
    out=[]; seen_c=set()
    for x in groups:
        cs=safe(lambda: json.loads(get(f"{BASE}/chirps?group={x['id']}&limit=3"))) or []
        for cc in cs:
            if cc['id'] in seen_c: continue
            seen_c.add(cc['id'])
            r=safe(lambda: load(cc['id']))
            if r is None: continue
            c,z=r
            from redemod import demod as demod_pll
            b=demod_pll(z, c['sample_rate'], c['bandwidth_hz'], rate, sync_re)
            if b is None: continue
            bits,s,rt,t,dur,coh=b; inv=False
            out.append(dict(id=cc['id'], center=c['center_hz'], time=c['time'], snr=c['snr_db'], dur=dur, rate=rt, inv=inv, tones=t, sync_at=s, bits=bits))
            json.dump(out, open(f'{name}.json','w')); print(name, len(out), 'of', len(seen_c), flush=True)
        if len(out)>=maxn: break
    json.dump(out, open(f'{name}.json','w')); print(name, len(out), 'frames from', len(seen_c), 'chirps')
os.makedirs('iq', exist_ok=True)
g=json.load(open('../groups_now.json')); tw=[x for x in g if x['receiver']=='tower-airspy' and x['examples']>0]
hop=[x for x in tw if x['duration_class_ms'] in (5,10,20,40) and x['mean_snr_db']>=22]; hop.sort(key=lambda x:-x['mean_snr_db'])
long=[x for x in tw if x['duration_class_ms']==160 and x['fsk'] and x['mean_snr_db']>=14]; long.sort(key=lambda x:-x['mean_snr_db']); print('long candidates', len(long), flush=True)
# hopper sync 39 6b f0 cf d8 76 76 (allow up to 3 errors via regex alternatives is hard; use exact 40-bit core)
if sys.argv[1]=='hop': run('hop', hop, 106400.0, re.compile('0011100101101011111100001100111111011000'), 300)
else: run('long', long, 50000.0, re.compile('(?:01){40,}1001000001001110'), 100)   # preamble + SFD 0x904E
