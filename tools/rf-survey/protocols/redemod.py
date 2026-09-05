import json, os, re, sys, collections, math
import numpy as np
from scipy.signal import firwin, lfilter
def load_iq(cid):
    raw=np.fromfile(f"iq/{cid}.cs8",dtype=np.int8).astype(np.float32); return raw[0::2]+1j*raw[1::2]
def pll_slice(d, sps, mid, gain=0.15):
    """Transition-locked slicer: sample at the middle of each symbol, re-lock on every zero crossing."""
    s=np.sign(d-mid); bits=[]; t=0.0; phase=0.0
    # find first crossing to set phase
    ch=np.where(np.diff(s)!=0)[0]
    if len(ch)<2: return ''
    t=ch[0]+sps/2; last_c=ch[0]; ci=0; n=len(d)
    out=[]
    while t<n:
        out.append('1' if d[int(t)]>mid else '0')
        # next crossing before t+sps?
        nxt=t+sps
        while ci<len(ch) and ch[ci]<t: ci+=1
        if ci<len(ch) and ch[ci]<nxt:
            # crossing expected at t+sps/2 (symbol boundary); error = actual - expected
            err=ch[ci]-(t+sps/2); nxt+=gain*err
        t=nxt
    return ''.join(out)
def center_on_signal(z, fs, rate):
    """Find the two strongest spectral peaks 20-160 kHz apart, mix their midpoint to DC, low-pass to the signal."""
    n=len(z); N=1<<int(np.ceil(np.log2(min(n,1<<16)))); seg=z[:N] if n>=N else np.concatenate([z,np.zeros(N-n)])
    P=np.abs(np.fft.fftshift(np.fft.fft(seg*np.hanning(N))))**2; f=(np.arange(N)-N//2)*fs/N
    sm=np.convolve(P,np.ones(5)/5,'same'); k1=int(np.argmax(sm)); f1=f[k1]
    mask=(np.abs(f-f1)>20e3)&(np.abs(f-f1)<160e3); k2=int(np.argmax(np.where(mask,sm,0))); f2=f[k2]
    fc=(f1+f2)/2; t=np.arange(n)/fs; y=z*np.exp(-2j*np.pi*fc*t)
    cut=min(0.45, (abs(f1-f2)/2+rate*0.8)/(fs/2)); y=lfilter(firwin(63,cut),1,y)[63:]
    return y, fc
def demod(z, fs, bw, rate, sync_re, tol=0.02, steps=9):
    y,fc=center_on_signal(z, fs, rate)
    env=np.abs(y); on=env>0.3*np.percentile(env,99); i=np.where(on)[0]
    if len(i)<50: return None
    y=y[i[0]:i[-1]+1]; d=np.angle(y[1:]*np.conj(y[:-1]))*fs/(2*np.pi)
    h,e=np.histogram(d,bins=200,range=(-fs/2,fs/2)); top=np.argsort(h)[::-1]; t1=(e[top[0]]+e[top[0]+1])/2
    far=[k for k in top if abs((e[k]+e[k+1])/2-t1)>6e3]
    if not far: return None
    t2=(e[far[0]]+e[far[0]+1])/2; mid=(t1+t2)/2
    k=max(1,int(round(fs/rate*0.5))); dd=np.convolve(d,np.ones(k)/k,'same')
    # symbol rate from the phase coherence of all zero crossings on a fine grid (±4%, 25 Hz steps)
    ch=np.where(np.diff(np.sign(dd-mid))!=0)[0].astype(float)
    if len(ch)<8: return None
    grid=np.arange(rate*0.96, rate*1.04, 25.0)
    coh=np.abs(np.exp(2j*np.pi*np.outer(grid, ch)/fs).sum(axis=1))/len(ch)
    r0=grid[int(np.argmax(coh))]
    best=None
    for r in (r0, r0*0.998, r0*1.002):
        bits=pll_slice(dd, fs/r, mid, gain=0.08)
        for cand in (bits, ''.join('1' if b=='0' else '0' for b in bits)):
            m=sync_re.search(cand)
            if m and (best is None or m.start()<best[1]): best=(cand, m.start(), r, (t1+fc,t2+fc), len(y)/fs*1e3, float(coh.max()))
    return best
if __name__=='__main__':
    which=sys.argv[1]
    src=json.load(open(f'{which}.json'))
    rate=106400.0 if which=='hop' else 50000.0
    sync=re.compile('0011100101101011111100001100111111011000') if which=='hop' else re.compile('(?:01){40,}1001000001001110')
    out=[]
    for f in src:
        try: z=load_iq(f['id'])
        except FileNotFoundError: continue
        c=json.load(open(f'meta_{f["id"]}.json')) if os.path.exists(f'meta_{f["id"]}.json') else None
        fs=None
        # sample rate/bandwidth: infer from stored json in corpus (not saved); refetch cheaply from server meta
        import urllib.request
        try:
            with urllib.request.urlopen(f"http://10.20.15.5:8433/chirps/{f['id']}", timeout=30) as r: c=json.loads(r.read())
        except Exception:
            continue
        b=demod(z, c['sample_rate'], c['bandwidth_hz'], rate, sync)
        if b is None: continue
        bits,s,r,t,dur,coh=b
        out.append(dict(id=f['id'], center=c['center_hz'], time=c['time'], snr=c['snr_db'], dur=dur, rate=r, tones=t, sync_at=s, bits=bits, coh=coh))
    json.dump(out, open(f'{which}2.json','w')); print(which, len(out), 'frames re-demodulated')
