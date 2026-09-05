import json, re, collections, sys
fr=json.load(open('long.json')); print(len(fr),'frames')
rev8=lambda x:int(f"{x:08b}"[::-1],2)
def pn9(n):
    s=0x1ff; out=[]
    for _ in range(n):
        byte=0
        for i in range(8):
            bit=s&1; byte|=bit<<(7-i); fb=((s>>0)^(s>>5))&1; s=(s>>1)|(fb<<8)
        out.append(byte)
    return bytes(out)
def crc16(data, poly, init, refin, refout, xorout):
    c=init
    for x in data:
        if refin: x=rev8(x)
        c^=x<<8
        for _ in range(8): c=((c<<1)^poly)&0xffff if c&0x8000 else (c<<1)&0xffff
    if refout: c=int(f"{c:016b}"[::-1],2)
    return c^xorout
VARS={'kermit':(0x1021,0,True,True,0),'xmodem':(0x1021,0,False,False,0),'ccitt-false':(0x1021,0xffff,False,False,0),'x25':(0x1021,0xffff,True,True,0xffff),'augccitt':(0x1021,0x1d0f,False,False,0),'mcrf4xx':(0x1021,0xffff,True,True,0)}
rows=[]
for f in fr:
    bits=f['bits']; m=re.search('(?:01){40,}1001000001001110', bits)
    if not m: continue
    p=m.end(); phr=bits[p:p+16]
    ms=int(phr[0]); fcs16=int(phr[3]); dw=int(phr[4]); L=int(phr[5:16],2); L_lsb=int(phr[5:16][::-1],2)
    psdu_bits=bits[p+16:p+16+8*L]
    if len(psdu_bits)<8*L: continue
    # octets LSB-first on air
    raw_msb=bytes(int(psdu_bits[i:i+8],2) for i in range(0,8*L,8))
    raw_lsb=bytes(rev8(b) for b in raw_msb)
    rows.append(dict(f=f, ms=ms, fcs16=fcs16, dw=dw, L=L, L_lsb=L_lsb, pre=(m.end()-m.start()-16), raw_msb=raw_msb, raw_lsb=raw_lsb))
print('PHR: mode-switch', collections.Counter(r['ms'] for r in rows), 'fcs16', collections.Counter(r['fcs16'] for r in rows), 'whitening', collections.Counter(r['dw'] for r in rows), 'len', collections.Counter(r['L'] for r in rows).most_common(5), 'len(lsb-first)', collections.Counter(r['L_lsb'] for r in rows).most_common(3))
print('preamble bits', collections.Counter(r['pre'] for r in rows).most_common(4))
# dewhiten candidates and CRC check
PN=pn9(300)
ok=collections.Counter(); example={}
for r in rows:
    for order in ('lsb','msb'):
        raw=r['raw_lsb'] if order=='lsb' else r['raw_msb']
        for wh in ('pn9-bitwise','pn9-byterev','none'):
            if wh=='pn9-bitwise': data=bytes(a^b for a,b in zip(raw,PN))
            elif wh=='pn9-byterev': data=bytes(a^rev8(b) for a,b in zip(raw,PN))
            else: data=raw
            for name,v in VARS.items():
                body,fcs=data[:-2],data[-2:]
                c=crc16(body,*v)
                for endian in ('le','be'):
                    if c==int.from_bytes(fcs,'little' if endian=='le' else 'big'):
                        ok[(order,wh,name,endian)]+=1; example.setdefault((order,wh,name,endian), data)
print('CRC-16 validated combos:', ok.most_common(5))
best=ok.most_common(1)[0][0] if ok else None
if best:
    order,wh,name,endian=best
    print(f"\nUsing {best}: decoding MAC headers")
    def dewhite(r):
        raw=r['raw_lsb'] if order=='lsb' else r['raw_msb']
        return bytes(a^b for a,b in zip(raw,PN)) if wh=='pn9-bitwise' else (bytes(a^rev8(b) for a,b in zip(raw,PN)) if wh=='pn9-byterev' else raw)
    types={0:'beacon',1:'data',2:'ack',3:'cmd',5:'multipurpose'}
    stats=collections.Counter(); srcs=collections.Counter()
    for r in rows:
        d=dewhite(r); v=VARS[name]
        if crc16(d[:-2],*v)!=int.from_bytes(d[-2:],'little' if endian=='le' else 'big'): continue
        fc=int.from_bytes(d[0:2],'little'); ftype=fc&7; sec=(fc>>3)&1; pend=(fc>>4)&1; ar=(fc>>5)&1; panc=(fc>>6)&1; seqsup=(fc>>8)&1; ie=(fc>>9)&1; dam=(fc>>10)&3; ver=(fc>>12)&3; sam=(fc>>14)&3
        pos=[2]; seq=None
        if not seqsup: seq=d[pos[0]]; pos[0]+=1
        def addr(mode):
            i=pos[0]
            if mode==0: return None
            if mode==2: a=d[i:i+2].hex(); pos[0]=i+2; return a
            a=d[i:i+8][::-1].hex(); pos[0]=i+8; return a
        dpan=None; dst=None; span=None; src=None
        if dam:
            dpan=d[pos[0]:pos[0]+2].hex(); pos[0]+=2; dst=addr(dam)
        if sam:
            if not panc: span=d[pos[0]:pos[0]+2].hex(); pos[0]+=2
            src=addr(sam)
        i=pos[0]
        stats[(types.get(ftype,ftype),'sec' if sec else 'plain','ver'+str(ver),'ie' if ie else '')]+=1; srcs[src]+=1
        print(f"  {r['f']['center']/1e6:.3f} MHz L={r['L']:3d} fc=0x{fc:04x} {types.get(ftype,ftype):5s} ver={ver} sec={sec} ie={ie} seq={seq} dstpan={dpan} dst={dst} srcpan={span} src={src} payload[{len(d)-2-i}]={d[i:i+16].hex()}...")
    print('\nframe classes:', stats.most_common(), '\nsources:', srcs.most_common(10))
else:
    for r in rows[:5]: print(r['L'], r['raw_lsb'].hex())
