import json, collections, itertools, math
fr=json.load(open('hop2.json'))
def tob(b): b=b[:len(b)//8*8]; return bytes(int(b[i:i+8],2) for i in range(0,len(b),8))
rev=lambda x:int(f"{x:08b}"[::-1],2)
frames=[]
for f in fr:
    n=int(f['dur']*f['rate']/1000)-f['sync_at']; b=tob(f['bits'][f['sync_at']:f['sync_at']+n])
    # strip trailing zeros then the 01 d8 postamble if present
    b=b.rstrip(b'\x00')
    post = b.endswith(b'\x01\xd8')
    if post: b=b[:-2]
    frames.append((f,b,post))
print(len(frames),'frames; with 01d8 postamble:', sum(p for _,_,p in frames))
L=collections.Counter(len(b) for _,b,_ in frames); print('lengths (sync..before postamble):', sorted(L.items()))
print('\nbyte7:', collections.Counter(b[7] for _,b,_ in frames).most_common(6), ' byte8:', collections.Counter(b[8] for _,b,_ in frames if len(b)>8).most_common(8))
ids=collections.Counter(b[8:13].hex() for _,b,_ in frames if len(b)>=13); print('5-byte field distinct', len(ids), 'top', ids.most_common(6))
ids4=collections.Counter(b[9:13].hex() for _,b,_ in frames if len(b)>=13); print('bytes9-12 distinct', len(ids4), 'top', ids4.most_common(6))
# CRC hunt
def crc(data, width, poly, init, refin, refout, xorout):
    mask=(1<<width)-1; c=init; top=1<<(width-1)
    for x in data:
        if refin: x=rev(x)
        c^=x<<(width-8)
        for _ in range(8): c=((c<<1)^poly)&mask if c&top else (c<<1)&mask
    if refout: c=int(f"{c:0{width}b}"[::-1],2)
    return c^xorout
CAT=[(8,0x07,0,0),(8,0x31,0,0),(8,0x9b,0,0),(8,0x07,0xff,0),(8,0x1d,0xff,0),(8,0x2f,0xff,0xff),
     (16,0x1021,0xffff,0),(16,0x1021,0,0),(16,0x1021,0xffff,0xffff),(16,0x8005,0,0),(16,0x8005,0xffff,0),(16,0x3d65,0,0xffff),(16,0x1021,0x1d0f,0),(16,0xc867,0xffff,0),(16,0x0589,0,0),
     (32,0x04c11db7,0xffffffff,0xffffffff),(32,0x04c11db7,0xffffffff,0),(32,0x04c11db7,0,0),(32,0x1edc6f41,0xffffffff,0xffffffff),(32,0x814141ab,0,0)]
hits=collections.Counter(); tested=0
for (_,b,_) in frames:
    for w,poly,init,xo in CAT:
        nb=w//8
        for refin,refout in ((False,False),(True,True)):
            for start in range(5,10):
                for endcut in range(0,3):
                    end=len(b)-endcut
                    if end-nb-start<2: continue
                    body=b[start:end-nb]; want=b[end-nb:end]
                    c=crc(body,w,poly,init,refin,refout,xo)
                    if c==int.from_bytes(want,'big') or c==int.from_bytes(want,'little'):
                        hits[(w,hex(poly),hex(init),refin,start,endcut)]+=1
print('\nCRC hits:', hits.most_common(8))
# repeated substrings across frames (>=4 bytes) 
sub=collections.Counter()
for _,b,_ in frames:
    seen=set()
    for i in range(13,len(b)-4):
        s=b[i:i+4]
        if s not in seen: seen.add(s); sub[s.hex()]+=1
print('\n4-byte substrings shared by most frames (beyond byte 13):', [(k,v) for k,v in sub.most_common(12) if v>=4])
# print 16-byte-class frames grouped by 5-byte field
print('\nshort frames:')
for f,b,p in sorted(frames, key=lambda t: t[1][8:13])[:40]:
    if len(b)<=16: print(' ', b[7:].hex(), 'post' if p else '', f"@{f['center']/1e6:.3f}")
