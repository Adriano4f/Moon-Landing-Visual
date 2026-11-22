# From measured heights to date-driven terrain shadows

## 1. A normal is not an occluder

This chapter follows `f76de54`, `59feb98`, `25fadf2`, and `42ac498`, using the
implementation at main `5e4d5b7`. Read
[the shader](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/moon.frag),
[height preparation](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/assets/heights.py),
[mask ownership](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/renderer/heights.rs),
and [CPU reference tests](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/core/src/terrain_tests.rs).

A normal map says which way a small surface patch tilts. It says nothing about
a crater wall kilometers away. Correctly oriented local shading can therefore
light a crater floor whose Sun is hidden by the rim. Realistic mode now asks a
second question: does the path toward the Sun intersect the measured terrain?

The display algorithm remains an approximation to a height field, not an
exhaustive mission-planning visibility calculation. The scientific horizon and
its two-meter observer continue through core independently.

## 2. Store meters as data, not display color

The coarse global maps derive from the NASA CGI Moon Kit LOLA source
`ldem_16_uint.tif`; the south-polar maps derive from Trek's
`LRO_LOLA_DEM_SPole75_30mp_v04`. `heights.manifest.json` records dimensions,
source, spacing, range, byte size and SHA-256. Do not infer their spacing from
the nominal resolution of the original DEM.

| Packed asset | Dimensions | Manifest spacing |
|---|---:|---:|
| `height_2k.png` | 2048×1024 | 5330.276 m at equator |
| `height_4k.png` | 4096×2048 | 2665.138 m at equator |
| `polar_height_1k.png` | 1024² | 1070.244 projected m |
| `polar_height_2k.png` | 2048² | 535.122 projected m |

The manifest's global spacing is $2\pi R/W$, with $R=1{,}737{,}400$ m. East-west
ground spacing shrinks with latitude. Polar stereographic projected spacing
also differs from ground spacing away from the projection's true-scale pole.

Coarse height packing uses half-meter units with a $-10{,}000$ m offset:

$$
q=\mathrm{round}\left(\frac{h+10000}{0.5}\right),\qquad
R_b=\lfloor q/256\rfloor,\quad G_b=q\bmod256.
$$

Blue stores validity. With normalized texture channels the inverse is

```glsl
return dot(value.rg, vec2(65280.0, 255.0)) * 0.5 - 10000.0;
```

These are linear numerical textures, not sRGB photographs. Height textures use
no mipmaps: averaging away a ridge would change the visibility question. The
shader explicitly samples level zero. Interpolation between samples is still
an approximation; half-meter quantization does not imply half-meter spatial
resolution or measurement accuracy.

## 3. Put the ray on a sphere, not a tangent plane

At Moon-fixed unit direction $\mathbf p$, terrain height $h(\mathbf p)$ and
numerical bias $b$, construct

$$
\mathbf o=(R+h(\mathbf p)+b)\mathbf p,\qquad
\mathbf r(d)=\mathbf o+d\mathbf s,
$$

where $\mathbf s$ is the unit Moon-fixed Sun direction for the selected date.
For each distance $d$, evaluate

$$
\mathbf u(d)=\frac{\mathbf r(d)}{\lVert\mathbf r(d)\rVert},\qquad
c(d)=\lVert\mathbf r(d)\rVert-R-h(\mathbf u(d)).
$$

Positive clearance means the ray is above terrain. Negative clearance means
an occluder lies above the center-ray path. Curvature is already present in
the Euclidean radius: there is no need to subtract an extra flat-Earth curvature
term afterward. Doing both would double-count it.

As a useful check, terrain at the observer's same radial height falls below the
local horizontal with increasing distance. On a flat tangent-plane marcher it
would remain at zero elevation forever, greatly exaggerating distant shadows.

The visual bias avoids self-shadow acne caused by interpolation and finite
sampling. It is not the scientific mast height: the coarse value is 3 m and
the fully weighted fine-window value is 0.5 m.

## 4. Bound expensive work without pretending it is exact

Without fine detail, the shader uses growing distances controlled by
`u_shadow_steps`: 32 for the lower height tier, 48 for the higher tier. The
growth factor aims toward 600,000 m. Rays may stop earlier if they are rising
above $R+15{,}000$ m, beyond the validated height range.

A separate conservative night-side test detects a ray passing through the
deep interior sphere of radius $R-10{,}000$ m. Mapped normals cannot bring
sunlight through the Moon's interior.

With a fine DEM window, up to 768 near samples resolve local terrain at the
window spacing, beginning at half a spacing. On leaving the window or exhausting
the near budget, the growing distant pass continues. A sixteen-sample refinement
then examines the strongest distant candidate when available.

Growing steps are a performance compromise. A narrow distant ridge can lie
between them. Increasing the maximum range does not establish that every
occluder within that range was resolved.

## 5. The Sun is a disk

Binary blocked/unblocked shadows produce hard, unstable edges. The shader uses
a fixed solar angular radius of $0.267^\circ$, consistent with core's modeling
approximation. It accumulates a minimum angular clearance estimate

$$
\delta\approx\min_d\frac{c(d)}{d},\qquad
x=\mathrm{clamp}(\delta/\alpha,-1,1).
$$

Then the circular-segment visible fraction is

$$
V=\frac{\arccos(-x)+x\sqrt{\max(0,1-x^2)}}{\pi}.
$$

At $x=-1$ the disk is hidden; at zero half is visible; at $x=1$ all is visible.
This provides a geometrical penumbra approximation from the limiting skyline.
It is not atmospheric scattering, a random blur, or a per-date measured solar
radius. The clearance-over-distance approximation is most natural near the
horizon where the distinction matters.

Direct solar Hapke response is multiplied by $V$. Earthshine remains a separate
light response; this marcher does not trace terrain occultation for every
point of an extended Earth.

## 6. Cache visibility in screen space

Tracing hundreds of terrain samples for every full-resolution shaded pixel
would be expensive. `Mask` owns an RGBA8 texture and framebuffer. Each axis is
half the opaque viewport dimension, clamped to 256 for height tier 2 or 512 for
higher height tiers. The final pass samples it with linear filtering.

The cache key includes model and projection matrices, Sun direction, camera
distance, height tier and live-normal state. Size changes recreate the mask;
new fine heights explicitly invalidate its key. Static frames reuse the result.
Rotating or zooming legitimately invalidates a screen-space cache even if the
Moon-fixed Sun has not moved.

The upsampling is bilinear, **not depth-aware**. It smooths the reduced grid but
cannot reconstruct shadows narrower than its screen-space sampling. Do not
describe the 512² maximum as a 512² terrain DEM; it is a visibility image.

## 7. Reuse the already downloaded fine DEM

Chapter 23 explains 512² DEM detail exports. Originally they supplied only
normals, so close-ups had fine slopes but coarse shadows. The fine-shadow change
retains the same export's heights as well; it adds no DEM request.

The decoder validates dimensions and F32 data, derives object-space normals,
then appends four bytes per height sample. Fine packing has a different offset:

$$
h_{\mathrm{fine}}=0.5(256R_b+G_b)-15000.
$$

Using the coarse $-10{,}000$ offset here would move terrain by five kilometers.
The renderer uploads the matching normal and height textures together, and
invalidates the matching mask before activating dynamic lighting with the pair.
Chapter 21 explains why merely receiving both HTTP responses is insufficient.

`detail_coordinates` returns UV plus a smooth border weight. The outer six
percent of the window feathers its contribution. The sampler combines fine and
coarse heights with that weight; it also blends the ray-origin bias:

```glsl
float bias = mix(3.0, 0.5, detail_weight);
vec3 origin = p * (R + terrain_height(p) + bias);
```

Feathering only heights but switching bias abruptly would still leave a shadow
seam. The final `42ac498` correction addresses exactly that origin discontinuity.
The coarse global/south-polar maps remain available for distant occluders;
fine polar windows can also supply north-polar local detail.

## 8. Compare against the independent horizon geometry

For Sun azimuth $a$ and terrain horizon elevation $H(a)$, the center ray should
be blocked when solar elevation $e<H(a)$. A finite disk changes this into a
visible fraction around the boundary. Core's `horizon_from_terrain` and
`sun_fraction` supply an independent reference.

`detail_shadow_march_matches_core_horizons` compares a CPU shader-equivalent
marcher on synthetic quantized/interpolated terrain against that reference.
It includes equatorial/polar geometry, dated Sun vectors, crater rims and
azimuth/elevation sweeps. Partial-disc fraction differences must be below 0.16;
outside the solar disk the correct lit/shadow side is required.

The other tests cover curved geometry/distant fallback and continuity of the
origin at the window border. They are not comparisons against every pixel of a
downloaded LOLA crater. Run them from the **current main source checkout**:

```sh
cargo test -p lunar_core detail_shadow
```

## 9. Physical intent and practical limits

Realistic does not apply NAC/WAC photographic contrast, so its dynamic shadows
are not superimposed on photographs with a conflicting recorded Sun direction.
The temporary photographic loading fallback has no dynamic terrain shadows.
Fast intentionally retains recorded shadows and performs no height marching.

The added active fine-height storage is 1 MiB for 512² RGBA8. Source resolution,
window interpolation, finite ray samples, numerical bias and mask upsampling
all limit the image. The sphere silhouette remains undisplaced. A physically
motivated model is not a claim of exact shadows at arbitrary magnification.

Finally, fine marching made Realistic slower in the recorded SwiftShader runs.
[Chapter 22](22-quality-and-antialiasing.md) preserves the measurements with
their exclusions rather than presenting the cache as a cost-free optimization.
