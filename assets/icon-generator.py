import math, sys
P=(512,120); R=(800,805); L=(330,725); W=128; rad=170
def towards(a,b,d):
    dx,dy=b[0]-a[0],b[1]-a[1]; n=math.hypot(dx,dy); return (a[0]+dx/n*d,a[1]+dy/n*d)
a=towards(P,R,rad); b=towards(P,L,rad)
rib=f"M{R[0]} {R[1]} L{a[0]:.1f} {a[1]:.1f} Q{P[0]} {P[1]} {b[0]:.1f} {b[1]:.1f} L{L[0]} {L[1]}"
y=545
def x_on(p,q,y): return p[0]+(q[0]-p[0])*(y-p[1])/(q[1]-p[1])
xl,xr=x_on(P,L,y),x_on(P,R,y)
bar=f"M{xl:.1f} {y} L{xr:.1f} {y}"
H=(286,778); HR=140; RX=150; RY=120; ROT=-24
# counter (hole) triangle, inner edges — used for the inner depth like the original
ty=P[1]+150
shape=f'''<path d="{rib}" stroke="#fff" stroke-width="{W}" stroke-linecap="round" fill="none"/>
<path d="{bar}" stroke="#fff" stroke-width="{W-34}" fill="none"/>
<ellipse cx="{H[0]}" cy="{H[1]}" rx="{RX}" ry="{RY}" transform="rotate({ROT} {H[0]} {H[1]})" fill="#fff"/>'''
svg=f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">
<defs>
 <linearGradient id="g" gradientUnits="userSpaceOnUse" x1="300" y1="140" x2="830" y2="920">
  <stop offset="0" stop-color="#FFDC3E"/><stop offset="0.36" stop-color="#FFA314"/>
  <stop offset="0.72" stop-color="#FF680A"/><stop offset="1" stop-color="#E2400B"/></linearGradient>
 <radialGradient id="head" gradientUnits="userSpaceOnUse" cx="{H[0]-55}" cy="{H[1]-65}" r="{HR*1.7}">
  <stop offset="0" stop-color="#FFE650" stop-opacity="0.95"/><stop offset="0.45" stop-color="#FFA014" stop-opacity="0.35"/><stop offset="1" stop-color="#FF8A10" stop-opacity="0"/></radialGradient>
 <!-- "goo": merges the strokes into one body with soft fillets at every joint -->
 <filter id="goo" filterUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024">
  <feGaussianBlur stdDeviation="10"/><feColorMatrix type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 28 -13"/></filter>
 <filter id="rim" filterUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024">
  <feGaussianBlur stdDeviation="10"/><feColorMatrix type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 28 -13"/><feMorphology operator="dilate" radius="3"/></filter>
 <filter id="b18" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="18"/></filter>
 <filter id="b8" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="8"/></filter>
 <mask id="body" maskUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024"><g filter="url(#goo)">{shape}</g></mask>
 <mask id="rimm" maskUnits="userSpaceOnUse" x="0" y="0" width="1024" height="1024"><g filter="url(#rim)">{shape}</g></mask>
</defs>
<rect width="1024" height="1024" fill="#C2340A" mask="url(#rimm)"/>
<g mask="url(#body)">
 <rect width="1024" height="1024" fill="url(#g)"/>
 <!-- inner depth around the A's counter, the same deep red the original uses inside its loop -->
 <path d="M{xl+20:.1f} {y-30} L{P[0]} {ty} L{xr-20:.1f} {y-30} Z" fill="none" stroke="#A52800" stroke-opacity="0.42" stroke-width="70" stroke-linejoin="round" filter="url(#b18)"/>
 <!-- depth where the stem meets the note head -->
 <ellipse cx="{H[0]+30}" cy="{H[1]+22}" rx="{RX+28}" ry="{RY+28}" transform="rotate({ROT} {H[0]+30} {H[1]+22})" fill="none" stroke="#8C2000" stroke-opacity="0.35" stroke-width="40" filter="url(#b18)"/>
 <ellipse cx="{H[0]}" cy="{H[1]}" rx="{RX}" ry="{RY}" transform="rotate({ROT} {H[0]} {H[1]})" fill="url(#head)"/>
 <!-- gloss -->
 <path d="M{P[0]-40} {P[1]+48} Q{P[0]-5} {P[1]+5} {P[0]+38} {P[1]+40}" stroke="#FFF7C0" stroke-opacity="0.6" stroke-width="20" stroke-linecap="round" fill="none" filter="url(#b8)"/>
 <path d="M{P[0]+86} {P[1]+210} L{R[0]-46} {R[1]-150}" stroke="#FFE580" stroke-opacity="0.3" stroke-width="18" stroke-linecap="round" fill="none" filter="url(#b8)"/>
 <ellipse cx="{H[0]-52}" cy="{H[1]-60}" rx="50" ry="27" transform="rotate(-38 {H[0]-52} {H[1]-60})" fill="#FFFAD0" fill-opacity="0.5" filter="url(#b8)"/>
</g>
</svg>'''
open(sys.argv[1],'w').write(svg)
