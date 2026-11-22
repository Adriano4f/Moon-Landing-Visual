# Fast and Realistic: spend GPU work where it matters

## 1. One choice, two contracts

Commit `5a2dc55` replaced the separate live-shadow/photo terminology with
**Quality: Fast / Realistic**. This chapter follows current main `5e4d5b7` and
preserves historical measurements separately from current guarantees.
Read [quality state](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/lib.rs),
[shader setup](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/renderer/setup.rs),
[the fragment shader](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/moon.frag),
and [texture filtering](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/renderer/textures.rs).

Fast means photographs with bounded recorded contrast and cheap geometric
shading. It does not sample normal maps, march height rays or compute a terrain
visibility mask. Rendering height PNGs and fine DEM exports are lazy and are not
requested merely to render Fast. A user asking for a scientific custom-point
horizon can still cause terrain downloads: that is a separate feature.

Realistic means Hapke photometry, DEM-derived normals and terrain-cast shadows.
It excludes NAC/WAC photographic contrast so recorded shadows do not conflict
with the selected date's dynamic Sun. Its loading fallback remains consistent
photographic shading until the required resources and mask are ready.

The default is Realistic when selected tier is above 2 and the device is not
mobile; low-tier/mobile defaults to Fast. A saved explicit choice overrides the
default through eframe storage. Overview tier and Quality are related defaults,
not the same setting: choosing a sharper color map is not choosing shadow work.

## 2. Compile away work rather than just hiding its result

A runtime `if` can skip an expensive branch, but the program still declares its
resources and carries the branch structure. `build_program` creates Fast and
Realistic variants from the shared GLSL using `FAST_MODE`.

```glsl
#if !FAST_MODE
// Hapke, normal sampling, height sampling, terrain marching.
#else
// Cheap photographic-mode response.
#endif
```

That excerpt is a structural sketch; inspect the actual preprocessor blocks
for resource declarations as well as functions. The Fast program has no fine
height sampler. Merely checking a UI label would not establish that property.

The Fast response is explicitly:

$$
\mu_0=\max(\mathbf n_g\cdot\mathbf l,0),\quad
\mu=\max(n_{g,z},0),
$$

$$
f=0.08\left(0.65\frac{\mu_0}{\max(\mu_0+\mu,10^{-6})}
+0.35\mu_0\right).
$$

This blends a lunar-Lambert-like term with Lambert incidence. It uses the
geometric sphere normal, not unresolved normal-map relief. It is a display
approximation, not calibrated pixel-by-pixel reflectance inversion.

## 3. Why mesh MSAA was not sufficient

The opaque Moon now comes from an analytic ray/sphere intersection inside a
projected quad. The quad's outer triangles are not the Moon's silhouette.
Hardware multisampling of those triangles therefore does not directly provide
coverage of the implicit circular edge.

For a normalized ray $\mathbf d$ from camera $(0,0,D)$, the shader forms

$$
b=-Dd_z,\qquad c=(D-1)(D+1),\qquad \Delta=b^2-c.
$$

The sphere is hit when $\Delta\ge0$. The stable near intersection is

$$
t=\frac{c}{b+\sqrt{\max(\Delta,0)}}.
$$

This avoids subtracting two close quantities near the surface. The factorized
form of $c$ similarly helps when $D$ is close to one.

The silhouette is the zero crossing of $\Delta$. Instead of a hard discard
there, the shader estimates a pixel-wide transition:

```glsl
float width = max(fwidth(discriminant), 0.000001);
coverage = smoothstep(-0.5 * width, 0.5 * width, discriminant);
```

`fwidth` combines screen derivatives from neighboring fragment invocations.
The resulting alpha blends to the background over roughly one pixel. Both
variants use this analytic limb AA. It does not smooth texture detail or trace
subpixel terrain silhouettes; those are separate sampling problems.

Native four-sample MSAA and egui's antialiased overlay paths remain useful for
their own primitives. No extra full-resolution supersampling/postprocessing
pass was introduced. The choice avoids multiplying the expensive lunar shading
work just to smooth the implicit edge.

## 4. Filter a footprint, not an arbitrary nearest texel

A mip chain stores successively smaller images. Its area sum is approximately
$1+1/4+1/16+\cdots=4/3$ of the base texture. Bilinear filtering interpolates within
a level; trilinear filtering additionally blends neighboring mip levels.
Surface color/normal/detail textures generate mipmaps and use
`LINEAR_MIPMAP_LINEAR` minification with linear magnification.

Height and page-table data are exceptions. Numerical heights use level zero
without mipmaps; page-table entries encode layer/level identities and must not
be blended like colors. The visibility mask uses linear upsampling without a
surface mip chain. "All textures are trilinear" would be incorrect.

At grazing angles a pixel footprint is elongated. The optional anisotropic
extension allows surface filtering up to the minimum of the device maximum and
2×. Satellite texture arrays retain isotropic trilinear filtering; increasing
their anisotropy raised the measured SwiftShader cost. This policy supersedes
the earlier chapter-03 workaround that disabled anisotropy everywhere.

Normal maps are linear vectors encoded in RGB, so browser color conversion is
disabled on upload. Albedo uses sRGB GPU storage: sampling decodes it before
linear-space filtering. Confusing those formats changes the material rather
than merely its sharpness.

## 5. Longitude derivatives must not jump at the seam

Longitude itself wraps from $+\pi$ to $-\pi$. Differentiating its wrapped UV
directly can report a huge footprint across adjacent pixels at that seam,
selecting an inappropriately blurry mip level.

For a direction perturbation $d\mathbf p$, the continuous differential is

$$
du=\frac{p_z\,dp_x-p_x\,dp_z}{2\pi(p_x^2+p_z^2)},\qquad
dv=-\frac{dp_y}{\pi\sqrt{p_x^2+p_z^2}}.
$$

The implementation bounds the polar denominator, computes these derivatives
from the continuous direction, and supplies them to `textureGrad`. UV wrapping
then affects position without producing an artificial derivative discontinuity.
It cannot repair a genuine brightness mismatch already present in a mosaic.

## 6. Rasterize less empty space

`View::disc_bounds` computes the sphere's projected extent. The renderer draws
a tight two-triangle quad rather than shading a full viewport. The fragment
shader rejects misses before albedo, normals and Hapke work. On a small Moon
this avoids spending most of the pass on black background pixels.

The shader also returns black early when reflectance is nonpositive. In
Realistic, terrain visibility can reject direct solar work while preserving
the separate Earthshine contribution. A blanket night-side shortcut based only
on the geometric normal would be wrong near elevated, resolved terrain.

`Program` caches uniform locations and sampler assignments at link time. This
removes repeated string lookups from the hot path; it does not mean every
changing uniform is uploaded only once. Texture binding is limited to inputs
used by the active path. Shadow masks are reused while their cache key matches.

Repaints remain event-driven except while loading, animating or fading. A
finished stationary image should not consume a permanent animation loop merely
because the shader is capable of drawing quickly.

## 7. Read performance tables with their experiment attached

The following are historical implementation-session measurements for
`c8b68fe` → `5a2dc55`, not fresh timings for this documentation update.
Chrome used ANGLE/Vulkan SwiftShader software rendering, 1280×841 CSS pixels,
DPR 2 and forced 2k textures. Camera, projection and Sun/Earth inputs matched.
Readback fences measured the opaque Moon pass, excluding UI, uploads and mask
recomputation. Medians used 30 warmed samples after and 87–88 before.

| Path | View | Before | After |
|---|---|---:|---:|
| Satellite → Fast | Default | 189.00 ms | 26.85 ms |
| Satellite → Fast | South pole, 8× | 226.45 ms | 212.20 ms |
| Live → Realistic | Default | 188.30 ms | 53.80 ms |
| Live → Realistic | South pole, 8× | 230.60 ms | 226.50 ms |

The dramatic default-view gain is consistent with avoiding empty viewport
work. A highly zoomed Moon fills the viewport, so that particular optimization
has much less opportunity. These are not whole-frame times, presentation FPS,
or measurements on a physical GPU/phone. Legacy Satellite was selected through
a `u_live_shadows=0` override in the baseline experiment.

## 8. Later fine shadows changed the performance story

The `dc156ad` → `42ac498` experiment used desktop 4k, the same CSS size/DPR,
33 warmed samples and matched January 4 camera/Sun inputs. Cached-mask opaque
pass medians were:

| View | Before fine shadows | With fine shadows |
|---|---:|---:|
| Default | 48.7 ms | 66.4 ms |
| Equatorial crater | 186.1 ms | 255.3 ms |
| Polar crater | 192.1 ms | 305.2 ms |

Single mask-rebuild samples, **not medians**, were 42.9 → 526.5 ms equatorial
and 29.4 → 385.8 ms polar. Fine shadows were not performance-neutral, even in
the cached opaque measurements. Different tiers and code versions mean these
rows must not be spliced into the previous table as one continuous benchmark.

The raw session captures were not added to this documentation branch. These
tables preserve the reported experiments and their conditions; rerunning them
requires an instrumented browser release, matched uniforms and readback fences.
Never turn a software-renderer observation into a universal GPU budget claim.

## 9. Downloads and validation are separate budgets

The render-modes cold Fast run recorded 9,928,239 transferred bytes including
headers and prepared-site data, of which 3,169,163 were globe assets, with no
height PNG requests. Later deep-zoom/fine-shadow builds recorded about 18.56 MB
Fast startup in their desktop configuration. These are different build/config
measurements, not a controlled claim that one feature doubled startup traffic.

Normal/color base pairs still load together for atomic compatibility although
Fast does not sample normals. Heights remain lazy. The old preview-normal
download was removed; preview uses its color image in both initial slots.

For verification, combine workspace tests with a cold network trace and a
shader-uniform/program probe. Check an implicit limb close-up, grazing texture
rotation and a delayed Realistic activation. A fast build or a UI label cannot
prove that the expensive sampler is absent or the edge is actually antialiased.
