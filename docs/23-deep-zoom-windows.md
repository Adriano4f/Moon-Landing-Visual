# Deep zoom: sharper data without loading a whole sharper Moon

## 1. More zoom is not more information

Commit `dc156ad` raised maximum zoom from 1024× to 2048× and introduced bounded
visible-area detail. The later fine-shadow and flicker commits reuse and
stabilize those windows. This chapter describes main `5e4d5b7`.

Read [window selection and DEM conversion](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/detail.rs),
[the NAC catalogue](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/detail/regions.rs),
[photo fetching](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/detail/fetch.rs),
and [GPU replacement](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/renderer/detail.rs).

Magnifying a texel cannot reveal a smaller crater. The useful improvement is to
fetch finer measured data over a small visible region, while keeping global
overview resources small enough for startup. The two quality modes need
different information: photographs for Fast, elevations for Realistic.

## 2. Understand the old resolution ceiling

With lunar radius $R=1{,}737{,}400$ m, an equirectangular image of width $W$ has
equatorial sample spacing

$$
s_{\mathrm{eq}}=2\pi R/W.
$$

The 2k/4k/8k overview widths therefore give about 5330/2665/1333 m per texel
at the equator. These tiers are selected by device/settings, not automatically
swapped at every zoom step. East-west spacing includes a $\cos\phi$ factor
away from the equator.

The fallback south-polar normal atlases have projected spacing about 1070 m
at 1k and 535 m at 2k. Their coverage and preparation are explained historically
in chapter 15. Increasing a global overview to 8k does not automatically make
that polar source an 8k polar DEM.

The streamed WAC mosaic is much sharper than the overview but still bounded:

| WAC path | Maximum level | Finest spacing |
|---|---:|---:|
| Global 303ppd mosaic | 8 | about 83.29 m at equator |
| North/south polar mosaic | 5 | about 267.56 projected m |

These are WMTS sample spacings, not guarantees of independent native source
resolution or feature detectability. Those WAC maxima did not become unlimited.
Regional NAC and DEM
windows are an additional layer, not a fictional higher WAC level.

## 3. Move the camera instead of enlarging a bitmap

The shared camera model uses

$$
D(z)=1+\frac{5-1}{\max(z,1)}.
$$

$D$ is distance from the center in lunar radii. Thus the illustrative altitude
above the sphere is $4R/z$: about 6.79 km at 1024× and 3.39 km at 2048×.
These are navigation geometry, not an orbit solution or clearance above actual
terrain. A sphere does not know about a mountain higher than that altitude.

The near clipping distance adapts with altitude. GPU analytic intersection,
CPU marker projection and inverse picking share the camera/projection model,
so increasing maximum zoom must not introduce a second independent marker
camera. The sphere silhouette remains smooth but undisplaced.

## 4. Estimate a visible window

`Spec::wanted` returns no detail request below 16×. Above it, the selector
intersects the view center with the Moon and estimates a physical footprint:

$$
w=1.8R\,\frac{2(D-1)}{\min(P_{00},P_{11})}.
$$

The 1.8 factor leaves a reuse margin. The projection diagonal includes the
actual viewport aspect, so a resolution quoted at one viewport is not a
universal resolution for that zoom number.

Absolute latitude 75° chooses polar coordinates for DEM selection. Fast polar
NAC is only considered from 86°; intermediate latitudes do not magically get
that coverage. Global NAC requires a point and whole candidate window inside a
catalogued regional mosaic. If no suitable source exists, WAC remains usable.

`Spec` records geographic/projected bounds, pole, source, photo/DEM kind, level
and initial tile index. This explicit identity lets scheduling, decoding and
publication agree on which window the pixels represent.

## 5. Fast: assemble a complete four-by-four photo window

Global NAC regions have named Trek mosaic services such as
`LRO_NAC_Mosaic_00N234E_50cmp`. Their names describe source products; "50cmp"
does not mean the application displays half-meter pixels at every zoom.
The catalogue records individual bounds and maximum levels, commonly 14.

Polar sources are `LRO_NAC_Mosaic_SPole85.5_1mp` and
`LRO_NAC_Mosaic_NPole85.5_1mp`, capped at level 11 by this client.
The WMTS URL shape is:

```text
https://trek.nasa.gov/tiles/Moon/{EQ|SP|NP}/{source}/1.0.0/default/default028mm/{level}/{row}/{column}.png
```

For vertical world span $S$ (global $\pi R$, polar $2E$), the selector starts from

$$
\ell=\left\lfloor\log_2\frac{4S}{w}\right\rfloor,
$$

then clamps to the source maximum. Four 256-pixel tiles span the requested
dimension, so the completed image is 1024². Global matrices have twice as many
columns as rows; polar matrices are square.

At most four photo requests run concurrently. All sixteen decoded tiles must
be opaque before the assembled window becomes publishable. Source bounds do
not guarantee pixel coverage: actual NAC footprints can contain transparent
NoData. Rejecting such a window preserves a complete WAC fallback instead of a
patchwork of black holes and photographs.

Tile-window origins are rounded and photographic reuse uses central-half and
adjacent-level hysteresis from chapter 21. A new request does not erase the
old complete GPU texture. These are loading guarantees, not a guarantee that
every requested regional image contains scientifically uniform illumination.

## 6. Realistic: ask for elevations, not photographs

Realistic exports one 512² F32 TIFF from a Trek ImageServer. Global source is
`LRO_LOLA_DEM_Global_256ppd_v06`, whose north-south native sample spacing is
approximately

$$
s_0=\frac{\pi R}{180\cdot256}\approx118.45\ \mathrm{m}.
$$

Polar selection tries the corresponding `LRO_LOLA_DEM_*Pole875_5mp_v04` 5 m
service when the full padded window fits its extent, otherwise the
`LRO_LOLA_DEM_*Pole75_30mp_v04` 30 m service. The rendering selector does not
use the scientific any-point path's additional southern 10 m fallback.

To avoid exporting a falsely oversampled DEM, the chosen spacing is

$$
s=s_0\,2^{\max(0,\lceil\log_2(w/(512s_0))\rceil)}.
$$

The center is quantized in fractions of the window size so small motion can
reuse the result. Geographic longitude width compensates for latitude, with
a bounded cosine; geographic and projected windows clamp to supported extents.
Near a global seam this path clamps the rendering window rather than using the
scientific request builder's seam-split collection.

The export explicitly requests image bytes, F32, uncompressed TIFF, no raster
function, bilinear interpolation and no aspect-ratio adjustment. The browser
must be allowed by the service's CORS policy. Successful historical responses
are evidence of usability, not an uptime or quota promise.

## 7. Derive normals in object space

For each sample the decoder constructs neighboring three-dimensional positions
at their measured radii. Let $\mathbf a,\mathbf b$ span left/right neighbors and
$\mathbf c,\mathbf d$ span top/bottom. An unnormalized normal is

$$
\mathbf n=(\mathbf b-\mathbf a)\times(\mathbf c-\mathbf d).
$$

Its dot product with the radial direction determines whether to reverse it;
then it is normalized and encoded to RGB8 by mapping $[-1,1]$ to $[0,255]$.
Using positions, rather than assuming a universal meters-per-degree tangent
plane, accommodates the geographic and polar projections consistently.

Fine heights are packed alongside normals with half-meter quantization and a
$-15{,}000$ m offset. Matching normal/height textures publish together and feed
the fine shadow marcher described in chapter 20. Existing limb and unresolved
polar-normal fading still attenuate mapped lighting; having a fine window does
not imply unattenuated normal detail in every polar view. The original zoom-quality
change supplied normals only; current main also uses these heights for shadows.

The shader maps each surface direction back to window coordinates, flips row
orientation consistently, and feathers the outer six percent. It blends normals
for Realistic or photo contrast for Fast; those meanings do not share a lighting
path just because both use a texture window.

## 8. What resolution was actually observed?

The zoom-quality implementation session compared identical 1024× cameras.
These are captured viewport-dependent values, not a promise at every location:

| Detail path | Before | Captured 1024× after | Captured/estimated 2048× path |
|---|---:|---:|---:|
| Fast regional equator | 83.29 m | 20.82 m | about 10.41 m |
| Fast polar | 267.56 m | 16.72 m | about 8.36 m |
| Realistic equatorial normals, desktop | about 2665 m | 118.45 m | 118.45 m native limit |
| Realistic polar normals | about 535 m | 40 m | about 20 m |

Later fine-shadow screenshots used approximately 237 m equatorial and 120 m
polar DEM spacing for their different cameras. Calling those "5 m shadows"
because a 5 m service was available would misrepresent the actual export.

Likewise, a region without NAC or with rejected NoData keeps WAC. Very deep
zoom can still look blurry or nearly featureless there. Real source strip
brightness and possible filtering seams are not corrected by requesting a
larger magnification.

## 9. Budget the extra download per accepted window

A 512² F32 export contains 1,048,576 sample bytes before TIFF metadata, about
1.05 MB in the recorded responses. It supplies both normals and heights with
no second dataset transfer. A different center or spacing can require another
window, so this is a per-window cost, not a total-navigation cap.

Recorded Fast transfers were about 0.85 MB for one equatorial window and
1.94 MB for a polar jump that selected two successive windows; the final polar
window was about 1.02 MB. PNG compressibility varies with actual terrain and
NoData. These numbers include response headers and are historical measurements,
not hardcoded budgets in the scheduler.

The feature adds no startup detail request because of its zoom gate. Removing
unused preview normals offset some additional decoder code; the matched
desktop Fast startup experiment recorded about 18.63 → 18.55 MB. That comparison
must not be mixed with the different 2k render-modes startup case in chapter 22.

## 10. Verify the data boundary as carefully as the image

In current main, run workspace tests for window selection, TIFF decoding,
normal orientation, height packing and hysteresis. For browser verification,
hold one of sixteen photo tiles: no incomplete window may appear. Then release
it and observe one complete replacement. Fail a later replacement and confirm
the previous complete window remains available.

Save camera/projection/Sun values with before/after screenshots. A sharper
picture at a different zoom or lighting angle does not isolate the data change.
Also test an uncovered region, north/south projection signs, the longitude seam,
and exact clicking after zoom. None of the visual textures may redefine the
scientific observer coordinate or silently substitute a tile center.

The final model is intentionally layered: global overview, streamed WAC, then
a bounded optional photo or DEM window. Each layer has a resolution limit,
coverage rule and publication contract. Making those explicit is what lets a
small application display useful close-ups without pretending it downloaded
an entire Moon at NAC resolution.
