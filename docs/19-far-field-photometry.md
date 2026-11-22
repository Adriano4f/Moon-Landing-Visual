# Why the Moon looked like a nearby lamp

## 1. Separate the camera from the observer model

This chapter explains commit `448460d` as retained in main `5e4d5b7`.
Read [moon.frag](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/moon.frag)
and [the geometry tests](https://github.com/Adriano4f/Moon-Landing-Visual/blob/5e4d5b7/globe/src/math.rs).
The historical Hapke derivation in chapter 03 remains background, but its old
finite-camera photometric interpretation is superseded here.

The symptom was an illuminated patch that seemed to move over the surface while
turning the globe. That can happen without moving the Sun at all. A lunar
reflectance model depends on the viewing direction as well as the light
direction. If the camera is treated as a nearby physical observer, its rays fan
out across the globe and create a spatially varying phase angle.

Our interactive camera is an illustrative navigation device. Moving it toward
the Moon is useful for inspecting a crater; interpreting that movement as a new
physical observation geometry everywhere is a different modeling choice. The
fix keeps perspective for geometry but uses parallel rays for lunar photometry.

## 2. Three cosines feed the reflectance model

Let $\mathbf n$ be a unit surface normal, $\mathbf l$ the direction toward the
Sun, and $\mathbf v$ the direction toward the observer. Then

$$
\mu_0=\mathbf n\cdot\mathbf l,\qquad
\mu=\mathbf n\cdot\mathbf v,\qquad
\cos g=\mathbf l\cdot\mathbf v.
$$

Incidence controls how light arrives, emission controls how it leaves toward
the observer, and phase angle $g$ describes their relative directions. These
are not interchangeable values. A normal map changes the first two without
necessarily changing the third.

The shader's Hapke response includes a two-term Henyey–Greenstein phase
function, shadow-hiding opposition enhancement, multiple scattering and a
macroscopic roughness correction. A schematic factorization is

$$
I/F=\frac{w}{4}\frac{\mu_{0e}}{\mu_{0e}+\mu_e}
\left[P(g)(1+B(g))+H(\mu_{0e})H(\mu_e)-1\right]S.
$$

The suffix $e$ denotes the roughness-corrected cosines. This equation is a map
of the implementation, not permission to substitute another paper's symbols
or parameter convention without checking the shader.

The particularly revealing term is the opposition enhancement:

$$
B(g)=\frac{B_{S0}}{1+\tan(g/2)/h_S}.
$$

It becomes stronger near zero phase angle. This is a useful lunar effect, but
the location of zero phase must come from the chosen observer geometry, not
accidentally from an artificial close inspection camera.

## 3. Derive the artificial hotspot

For camera position $\mathbf C$ and surface point $\mathbf x$, the perspective
view direction is

$$
\mathbf v_{\mathrm{perspective}}(\mathbf x)
=\frac{\mathbf C-\mathbf x}{\lVert\mathbf C-\mathbf x\rVert}.
$$

Even if the Sun direction is constant, this vector changes across the disk.
Consequently $g(\mathbf x)$ changes across it. At points where the local view
direction nearly aligns with the Sun, the opposition term becomes brighter.
Rotating the Moon moves different terrain through this camera-dependent patch.

This is not necessarily an algebra error in Hapke. It is a mismatch between an
illustrative close camera and the intended distant-observer material model.
Increasing resolution, changing texture wrap, or adding shadow rays would not
remove the underlying spatial phase-angle variation.

One useful diagnostic is to hold the Moon-fixed Sun constant while changing
only the inspection camera distance. If the opposition peak sweeps or shrinks
with that distance, the observer term deserves inspection before the lighting
ephemeris is blamed.

## 4. Take the far-field limit deliberately

In view space the camera looks along negative z. For a distant observer, rays
from all surface points toward that observer are approximately parallel to
positive z. The shader therefore explicitly uses:

```glsl
vec3 photometric_view = vec3(0.0, 0.0, 1.0);
```

The directional lights are rotated into the same space:

```glsl
vec3 light = normalize(u_model * u_sun);
vec3 earth_light = normalize(u_model * u_earth);
```

`light_response` receives `photometric_view` for both direct sunlight and
Earthshine. Thus

$$
\cos g=\mathbf l\cdot(0,0,1)
$$

is constant over one draw. Terrain normals still affect incidence and emission,
and rotating the Moon still changes its appearance correctly relative to the
Moon-fixed Sun. What disappears is the local camera-distance phase hotspot.

The fix does **not** freeze the Sun in screen coordinates. `u_sun` is Moon-fixed
and comes from the selected date; the same model rotation is applied to terrain
normals and light. If the Sun were incorrectly attached to the camera, a fixed
crater would keep the same illumination as the Moon turned. That is a different
bug with a different diagnostic.

## 5. Do not remove the geometric view vector

The shader still computes:

```glsl
vec3 geometric = normalize(position);
vec3 view = normalize(vec3(0.0, 0.0, u_camera_distance) - position);
float normal_strength = smoothstep(
    0.35, 0.65, max(dot(geometric, view), 0.0));
```

This true perspective vector remains appropriate for geometric foreshortening
and reducing unresolved normal-map relief near the visible limb. The analytic
sphere intersection also uses the real camera. Photometric approximation and
geometric projection have different responsibilities.

A mapped normal can face away from the perspective observer even when the
underlying sphere is visible. Because the sphere has no displaced microgeometry,
that normal must not punch a black hole in the silhouette. The shader adds only
the missing view component before renormalizing it. This is an unresolved-relief
safeguard, not an actual geometry displacement or a cast-shadow solver.

Similarly, equirectangular polar normals are suppressed when reliable polar
normals are unavailable. Longitude singularities must not masquerade as physical
ridges. This fallback does not add information about polar terrain.

## 6. Local shading and terrain shadows answer different questions

Hapke evaluates the response of an illuminated facet. A mountain elsewhere can
block its Sun even when its local normal faces the light. The direct-light term
therefore becomes, schematically,

$$
L_{\mathrm{Sun}}=s_{\mathrm{Sun}}\,V_{\mathrm{Sun}}\,\mathrm{Hapke}(\mathbf n,\mathbf l,\mathbf v).
$$

Here $V_{\mathrm{Sun}}$ is the visible solar fraction from chapter 20. Changing
the observer model fixes neither missing occluders nor baked photograph shadows.
Those required the separate DEM-shadow and quality-mode changes.

Fast mode now uses a cheaper lunar-Lambert-like response instead of this Hapke
path. Its use of recorded photographic contrast is intentional and explicitly
not a scientific reconstruction of the selected date's shadows.

## 7. What the fix does not claim

- It is not an exact finite-distance spacecraft-observation simulator.
- It does not recalibrate the albedo image or invert baked illumination.
- It does not change the scientific topocentric Sun/Earth tracks.
- It does not displace the terrain silhouette.
- It does not imply the Moon should have uniform brightness across its disk.

Those distinctions prevent an attractive correction from being described as a
more general physical model than the code actually implements.

## 8. Reproduce the reasoning rather than trusting a screenshot

In a current main checkout, the `math.rs` regression checks assert the parallel
photometric vector and its use by `light_response`. Run the workspace tests to
exercise them. A useful visual experiment additionally fixes date and rotation,
changes zoom, then returns to the identical camera. Surface positions and local
phase behavior can be compared separately.

When recording evidence, save model/projection matrices and the Sun vector.
Comparing two differently lit dates or differently framed craters would not
isolate the observer fix. A full-Moon image with no obvious artifact is weaker
evidence than a controlled off-axis Sun test.
