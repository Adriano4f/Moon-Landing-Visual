"""Reproduce parameter means and numerically check the shader's Hapke equations.

Reference: DOI-USGS/ISIS3 fadd004da2f733525209ee4f66ffdb3a9a9d193e,
isis/src/lro/apps/lrowacphomap/HapkeLROC.cpp, HGfunc and photometry.
Requires numpy. Downloads the small published parameter map only when absent.
"""
import math
import pathlib
import urllib.request

import numpy as np

URL = "https://pds.lroc.im-ldi.com/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/SDP/WAC_HAPKEPARAMMAP/WAC_HAPKEPARAMMAP_566NM.IMG"
W, B, C, BS0, HS = 0.3261142672, 0.2348928321, 0.3574639778, 1.7707997700, 0.0622193166
THETA = math.radians(23.65660095)


def h_function(x):
    if x == 0:
        return 1.0
    x = abs(x)
    gamma = math.sqrt(1 - W)
    r0 = (1 - gamma) / (1 + gamma)
    return 1 / (1 - W*x*(r0 + (1-2*r0*x)/2*math.log((1+x)/x)))


def reflectance(mu0, mu, phase_deg, opposition=True):
    if mu0 <= 0 or mu <= 0:
        return 0.0
    g = math.radians(phase_deg)
    cosg = math.cos(g)
    phase = (1+C)/2*(1-B*B)/(1-2*B*cosg+B*B)**1.5 + (1-C)/2*(1-B*B)/(1+2*B*cosg+B*B)**1.5
    shoe = 0 if phase_deg == 180 else 1/(1+math.tan(g/2)/HS)
    t = math.tan(THETA)
    chi = 1/math.sqrt(1+math.pi*t*t)
    sini, sine = math.sqrt(max(0,1-mu0*mu0)), math.sqrt(max(0,1-mu*mu))
    coti, cote = mu0/max(1e-6,sini), mu/max(1e-6,sine)
    e1i, e2i = math.exp(-2*coti/(math.pi*t)), math.exp(-coti*coti/(math.pi*t*t))
    e1e, e2e = math.exp(-2*cote/(math.pi*t)), math.exp(-cote*cote/(math.pi*t*t))
    etai, etae = chi*(mu0+sini*t*e2i/(2-e1i)), chi*(mu+sine*t*e2e/(2-e1e))
    cospsi = 1 if sini*sine < 1e-6 else max(-1,min(1,(cosg-mu*mu0)/(sini*sine)))
    psi = math.acos(cospsi)
    fpsi = 0 if psi >= math.pi-1e-6 else math.exp(-2*math.tan(psi/2))
    sin2 = (1-cospsi)/2
    if mu0 >= mu:
        den = 2-e1e-psi/math.pi*e1i
        mu0e = chi*(mu0+sini*t*(cospsi*e2e+sin2*e2i)/den)
        mue = chi*(mu+sine*t*(e2e-sin2*e2i)/den)
        q = mu0/etai
    else:
        den = 2-e1i-psi/math.pi*e1e
        mu0e = chi*(mu0+sini*t*(e2i-sin2*e2e)/den)
        mue = chi*(mu+sine*t*(cospsi*e2i+sin2*e2e)/den)
        q = mu/etae
    shadow = mue*mu0*chi/max(etae*etai*(1-fpsi+fpsi*chi*q),1e-6)
    multiple = h_function(max(mu0e,1e-6))*h_function(max(mue,1e-6))-1
    return W/4*mu0e/max(mu0e+mue,1e-6)*(phase*(1+(BS0 if opposition else 0)*shoe)+multiple)*shadow


if __name__ == "__main__":
    path = pathlib.Path.home()/"plan"/"WAC_HAPKEPARAMMAP_566NM.IMG"
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists():
        urllib.request.urlretrieve(URL,path)
    raw = path.read_bytes()
    label = raw[:5760].decode("ascii")
    for expected in ["RECORD_BYTES  = 1440", "^IMAGE        = 5", "PC_REAL", "BAND_SEQUENTIAL", "BANDS                      = 9"]:
        assert expected in label, expected
    data = np.frombuffer(raw,dtype="<f4",offset=5760).reshape(9,140,360)
    valid = np.all(np.isfinite(data) & (np.abs(data)<1e10),axis=0)&(data[0]>0)&(data[0]<1)
    weights = np.cos(np.deg2rad(69.5-np.arange(140)))[:,None]*valid
    means = (data.astype("f8")*weights).sum(axis=(1,2))/weights.sum()
    print("Area-weighted valid tiles:",int(valid.sum()),"(70S to 70N, all longitudes)")
    for name,value in zip(["w","b","c","Bc0","hc","Bs0","hs","theta","phi"],means):
        print(f"{name} = {value:.10f}")
    assert np.allclose(means[[0,1,2,5,6]],[W,B,C,BS0,HS],rtol=1e-8)
    center,limb = reflectance(1,1,0),reflectance(0.2,0.2,0)
    darkening = 1-limb/center
    print(f"Full Moon center={center:.8f}, mu=0.2={limb:.8f}, darkening={darkening:.2%}")
    assert 0 <= darkening < 0.15
    for phase in [0,1,5,30,60,90,120,170]:
        mu = math.cos(math.radians(phase/2))
        print(f"phase={phase:3} degrees, i=e=phase/2, I/F={reflectance(mu,mu,phase):.8f}")
    assert center > reflectance(1,1,0,False)*2
    assert reflectance(1,1,0) > reflectance(math.cos(math.radians(2.5)),math.cos(math.radians(2.5)),5)
    assert reflectance(-1,1,90) == 0
    for i in range(0,90,5):
        for e in range(0,90,5):
            for az in [0,30,90,180]:
                ir,er,ar = map(math.radians,[i,e,az])
                cosg = math.cos(ir)*math.cos(er)+math.sin(ir)*math.sin(er)*math.cos(ar)
                g = math.degrees(math.acos(max(-1,min(1,cosg))))
                r = reflectance(math.cos(ir),math.cos(er),g)
                assert math.isfinite(r) and r >= 0
    print("Photometry checks: limb profile, opposition surge, night side, 1,296 geometries OK")
