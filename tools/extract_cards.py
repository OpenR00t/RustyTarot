# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 The Rusty Tarot contributors

"""Cut individual cards out of scanned tarot sheets and recover their pixel art.

1. Each card is found as a non-white blob on the white scan, straightened with
   its minimum-area rectangle, and cropped tight to its black printed frame.
2. The printed pixel grid is recovered: 84 cells across, with the row spacing
   fitted per card (scanners stretch sheets slightly), sampling each cell's
   centre. Colours are then cleaned to a 48-colour palette.
3. All cards are brought to the same grid size and saved as PNGs named the way
   rusty_tarot expects (major_00_the_fool.png, …, back.png).

The result is true pixel art (~84×147), which the app enlarges with crisp
whole-pixel scaling instead of blurring a scan.

Sheets are read left-to-right, top-to-bottom; file names map to cards via the
`names` table below ("coins" = Pentacles; the card back is last on
major-arcana-03).

Needs: pip install opencv-python numpy
"""
import cv2, numpy as np, os, sys

# --- pixel-grid recovery -----------------------------------------------
def comb_score(prof, p, ph):
    # sum of profile sampled at boundaries ph + k*p (linear interp)
    xs=np.arange(ph, len(prof)-1.01, p)
    i=np.floor(xs).astype(int); f=xs-i
    return (prof[i]*(1-f)+prof[i+1]*f).mean()
def find_grid(prof, lo=2.6, hi=3.6):
    prof=prof-np.convolve(prof,np.ones(15)/15,'same')
    best=(-1e9,0,0)
    for p in np.arange(lo,hi,0.002):
        for ph in np.arange(0,p,0.1):
            s=comb_score(prof,p,ph)
            if s>best[0]: best=(s,p,ph)
    s,p,ph=best
    for ph2 in np.arange(max(0,ph-0.1),ph+0.1,0.01):
        s2=comb_score(prof,p,ph2)
        if s2>s: s,ph=s2,ph2
    return p,ph
def _phase(prof,p):
    return max(np.arange(0,p,0.05),key=lambda ph:comb_score(prof,p,ph))
def _origin(ph,p):
    o=ph+0.5
    while o-p>-p*0.5: o-=p
    return o
def _sample(f,x0,px,y0,py):
    H,W=f.shape[:2]
    cx=x0+(np.arange(int((W-x0)/px))+0.5)*px; cy=y0+(np.arange(int((H-y0)/py))+0.5)*py
    cx=cx[(cx>=0.5)&(cx<=W-1.5)]; cy=cy[(cy>=0.5)&(cy<=H-1.5)]
    mx,my=np.meshgrid(cx,cy)
    acc=0
    for dx in (-0.5,0,0.5):
        for dy in (-0.5,0,0.5):
            acc=acc+cv2.remap(f,(mx+dx).astype(np.float32),(my+dy).astype(np.float32),cv2.INTER_LINEAR)
    return acc/9.0
def _recon_err(f,small,x0,px,y0,py):
    H,W=f.shape[:2]
    ix=np.clip(np.floor((np.arange(W)-x0)/px).astype(int),0,small.shape[1]-1)
    iy=np.clip(np.floor((np.arange(H)-y0)/py).astype(int),0,small.shape[0]-1)
    rec=small[iy][:,ix]
    m=(slice(int(H*.08),int(H*.92)),slice(int(W*.08),int(W*.92)))
    return float(np.abs(rec[m]-f[m]).mean())
def degrid(card, nx=84):
    """Recover the printed pixel grid: nx cells across; the row spacing is chosen
    per card (scans can be stretched vertically) by best reconstruction."""
    f=card.astype(np.float32); g=f.mean(axis=2); H,W=g.shape
    norm=lambda pr: pr-np.convolve(pr,np.ones(15)/15,'same')
    prx=norm(np.abs(np.diff(g,axis=1))[int(H*.1):int(H*.9)].sum(axis=0))
    pry=norm(np.abs(np.diff(g,axis=0))[:,int(W*.1):int(W*.9)].sum(axis=1))
    px=W/nx; x0=_origin(_phase(prx,px),px)
    best=None
    for s in np.arange(0.99,1.075,0.004):
        py=px*s; y0=_origin(_phase(pry,py),py)
        small=_sample(f,x0,px,y0,py)
        e=_recon_err(f,small,x0,px,y0,py)
        if best is None or e<best[0]: best=(e,small,py)
    e,small,py=best
    return small.clip(0,255).astype(np.uint8),(px,py)
def quantize(img,k=32):
    Z=img.reshape(-1,3).astype(np.float32)
    crit=(cv2.TERM_CRITERIA_EPS+cv2.TERM_CRITERIA_MAX_ITER,30,0.5)
    _,lab,cen=cv2.kmeans(Z,k,None,crit,3,cv2.KMEANS_PP_CENTERS)
    return cen[lab.flatten()].reshape(img.shape).astype(np.uint8)

# --- card extraction ---------------------------------------------------

if len(sys.argv) != 3:
    sys.exit("usage: python extract_cards.py <folder of sheet scans> <output cards folder>")
SRC, OUT = sys.argv[1], sys.argv[2]
os.makedirs(OUT, exist_ok=True)
RANKS=["ace","two","three","four","five","six","seven","eight","nine","ten","page","knight","queen","king"]
MAJ=["the_fool","the_magician","the_high_priestess","the_empress","the_emperor","the_hierophant","the_lovers","the_chariot","strength","the_hermit","wheel_of_fortune","justice","the_hanged_man","death","temperance","the_devil","the_tower","the_star","the_moon","the_sun","judgement","the_world"]
names={}
names["pixel-tarot-major-arcana-01.jpg"]=[f"major_{i:02d}_{MAJ[i]}" for i in range(0,8)]
names["pixel-tarot-major-arcana-02.jpg"]=[f"major_{i:02d}_{MAJ[i]}" for i in range(8,16)]
names["pixel-tarot-major-arcana-03.jpg"]=[f"major_{i:02d}_{MAJ[i]}" for i in range(16,22)]+["back"]
for src,suit in [("coins","pentacles"),("cups","cups"),("swords","swords"),("wands","wands")]:
    names[f"pixel-tarot-minor-arcana-{src}-01.jpg"]=[f"{suit}_{r+1:02d}_{RANKS[r]}" for r in range(0,8)]
    names[f"pixel-tarot-minor-arcana-{src}-02.jpg"]=[f"{suit}_{r+1:02d}_{RANKS[r]}" for r in range(8,14)]

def rects(img):
    hsv=cv2.cvtColor(img,cv2.COLOR_BGR2HSV); gray=cv2.cvtColor(img,cv2.COLOR_BGR2GRAY)
    mask=((gray<190)|(hsv[...,1]>60)).astype(np.uint8)*255
    mask=cv2.morphologyEx(mask,cv2.MORPH_CLOSE,np.ones((9,9),np.uint8))
    cs,_=cv2.findContours(mask,cv2.RETR_EXTERNAL,cv2.CHAIN_APPROX_SIMPLE)
    rs=[cv2.minAreaRect(c) for c in cs if cv2.contourArea(c)>20000]
    out=[]
    for (cx,cy),(w,h),a in rs:
        if w>h: w,h,a=h,w,a+90
        if a>45: a-=90
        if a<-45: a+=90
        out.append(((cx,cy),(w,h),a))
    # rows by y, then x
    out.sort(key=lambda r:r[0][1]); rows=[]
    for r in out:
        if rows and abs(rows[-1][0][0][1]-r[0][1])<150: rows[-1].append(r)
        else: rows.append([r])
    return [r for row in rows for r in sorted(row,key=lambda r:r[0][0])]

def frame_edges(crop):
    """outermost dark frame line on each side"""
    g=cv2.cvtColor(crop,cv2.COLOR_BGR2GRAY).astype(float)
    dark=(g<90).astype(float)
    h,w=dark.shape
    colf=dark[int(h*.15):int(h*.85),:].mean(axis=0)
    rowf=dark[:,int(w*.15):int(w*.85)].mean(axis=1)
    def first(prof,rng):
        for i in rng:
            if prof[i]>0.55: return i
        return None
    l=first(colf,range(0,w//4)); r=first(colf,range(w-1,3*w//4,-1))
    t=first(rowf,range(0,h//4)); b=first(rowf,range(h-1,3*h//4,-1))
    return l,t,r,b

report=[]
results=[]
for fn,nm in names.items():
    img=cv2.imread(os.path.join(SRC,fn))
    rs=rects(img)
    assert len(rs)==len(nm),(fn,len(rs))
    for ((cx,cy),(w,h),a),name in zip(rs,nm):
        M=cv2.getRotationMatrix2D((cx,cy),a,1.0)
        rot=cv2.warpAffine(img,M,(img.shape[1],img.shape[0]),flags=cv2.INTER_CUBIC,borderValue=(255,255,255))
        pad=14
        x0,y0=int(round(cx-w/2-pad)),int(round(cy-h/2-pad))
        crop=rot[max(0,y0):y0+int(h)+2*pad, max(0,x0):x0+int(w)+2*pad]
        l,t,r,b=frame_edges(crop)
        if None in (l,t,r,b):
            report.append((name,"frame not found",a)); l,t,r,b=pad,pad,crop.shape[1]-pad,crop.shape[0]-pad
        card=crop[t:b+1,l:r+1]
        report.append((name,f"{card.shape[1]}x{card.shape[0]}",round(a,2)))
        px_art,(gpx,gpy)=degrid(card)
        report[-1]=report[-1]+(f"grid {px_art.shape[1]}x{px_art.shape[0]} p={gpx:.3f},{gpy:.3f}",)
        results.append((name,quantize(px_art,48)))
# every card onto the same pixel grid (median size), nearest-neighbour
ws=sorted(r[1].shape[1] for r in results); hs=sorted(r[1].shape[0] for r in results)
GW,GH=ws[len(ws)//2],hs[len(hs)//2]
for name,img in results:
    if img.shape[:2]!=(GH,GW):
        img=cv2.resize(img,(GW,GH),interpolation=cv2.INTER_NEAREST)
    cv2.imwrite(os.path.join(OUT,name+".png"),img)
for r in report: print(*r)
print("grid size",GW,GH)
