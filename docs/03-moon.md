# Moon renderer reference

The independent `globe` crate renders an illustrative Moon; scientific terrain visibility is computed in `core/` and displayed by `app/`.

- **Coordinates:** latitude is north-positive and longitude east-positive. Unit directions are `(cos(lat) sin(lon), sin(lat), cos(lat) cos(lon))`: north is +y and longitude zero is +z. See [`math.rs`](../globe/src/math.rs).
- **Projection:** matrices use column-major storage. `View` supplies model rotation, zoom-dependent camera distance and an additive NDC pan shared by CPU overlays, picking and GPU projection. The fragment shader subtracts that lens shift when reconstructing sphere rays. See [`moon.vert`](../globe/src/moon.vert), [`moon.frag`](../globe/src/moon.frag) and [`deep.rs`](../globe/src/deep.rs).
- **Photometry:** the shader uses common LROC WAC 566 nm Hapke parameters with relative RGB texture modulation, not per-pixel spectral calibration. Photometry uses parallel observer rays, so the close zoom camera cannot create a localized opposition hotspot. Sun and Earth directions remain Moon-fixed from the ephemeris. Only reflected Earth light illuminates the solar night side, with an optional explicit exposure boost. This does not change terrain visibility results.
- **Grid:** the decorative zoom grid appears at 3x and above when zoom changes, holds for one second, then fades out over half a second. It does not snap clicks.
- **Color:** albedo is decoded from sRGB for linear-light shading and encoded for display; relief-derived object-space normals are decoded and renormalized in the shader.
- **GL lifetime:** renderer resources belong to the eframe/glow context. Painting uses an egui GL callback; explicit destruction releases GPU resources. See [`renderer.rs`](../globe/src/renderer.rs) and [`view.rs`](../globe/src/view.rs).

## Quality and anti-aliasing

The Lighting options contain one **Quality: Fast / Realistic** choice. Fast uses satellite contrast with a cheap lunar-Lambert response and no normal-map sampling, terrain marching or visibility mask. Height textures are requested only after choosing Realistic; base color/normal pairs remain transactionally loaded for mode/tier switching. Realistic retains Hapke, normals and DEM shadows without adding recorded photographic shadows. Earthshine remains separate in both modes. Mobile/2k devices default to Fast; desktop browsers default to the 4k tier and Realistic (native desktops retain their existing tier). The selection is remembered in eframe storage when available, otherwise for the lifetime of the view.

Separate compile-time shader variants remove terrain and Hapke work from Fast. Both rasterize a tight projected-disc quad with a two-pixel coverage margin, discard off-disc fragments, and blend analytic pixel-width limb coverage. Texture coordinates use seam-safe explicit derivatives; color/normal textures have trilinear mipmaps with up to 2x anisotropy when supported. Satellite arrays use trilinear mipmaps without anisotropy to keep Fast inexpensive. Packed heights and page tables remain unmipmapped data. Shadow masks retain bilinear upsampling and are cached by camera/Sun/terrain state. Uniform locations and sampler assignments are initialized once per program; only active textures are bound. Realistic stops requesting photographic tiles after its complete surface is ready. Loading, animation and the existing anti-aliased egui overlays retain event-driven repaints. No extra supersampling or post-process pass is needed; native MSAA remains enabled as before.

Loading remains transactional: global color/normal, both height maps and polar normals are staged together, with one GPU upload per frame. The previous complete surface remains visible during tier changes; initial Realistic loading uses the Fast overview until the complete set and visibility mask are ready. Satellite imagery stays on the overview until all currently requested tiles are resident, rather than exposing partially downloaded hemispheres. Failed replacements retain the existing complete surface.

## Terrain shadows

Global 2k/4k and south-polar 1k/2k heights use the same source DEMs and projections as the normals. Heights are linear RGB8: RG stores a 16-bit value (`value * 0.5 - 10000` metres), B marks validity. `heights.manifest.json` records scale, offset, dimensions, spacing, hashes and download sizes. The 8k visual tier uses 4k heights; no higher-resolution DEM is implied.

The shader marches along the Moon-fixed Sun direction over a sphere of radius 1,737,400 m, with 3 m bias, increasing steps out to 600 km, and a 0.53° solar disk for soft visibility. There are 32 steps in the 2k tier and 48 otherwise. Height-field visibility replaces the reference-sphere solar cutoff, allowing elevated terrain beyond the terminator without lighting deep-night normal-map facets.

A reduced-resolution visibility mask (up to 256² low tier / 512² high tier) is linearly upsampled and cached until camera, Sun, terrain tier or polar normals change. This is illustrative terrain lighting, not the scientific horizon solver: small ridges between samples can be missed, mask edges are softened, relief does not alter the globe silhouette, and the global/polar blend can expose source-resolution differences.

Sources, attribution and offline asset reproduction: [`globe/assets/README.md`](../globe/assets/README.md). Detailed learning material is preserved on [`learn`](https://github.com/Adriano4f/Moon-Landing-Visual/tree/learn).
