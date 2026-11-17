# Moon renderer reference

The independent `globe` crate renders an illustrative Moon; scientific terrain visibility is computed in `core/` and displayed by `app/`.

- **Coordinates:** latitude is north-positive and longitude east-positive. Unit directions are `(cos(lat) sin(lon), sin(lat), cos(lat) cos(lon))`: north is +y and longitude zero is +z. See [`math.rs`](../globe/src/math.rs).
- **Projection:** matrices use column-major storage. `View` supplies model rotation, zoom-dependent camera distance and an additive NDC pan shared by CPU overlays, picking and GPU projection. The fragment shader subtracts that lens shift when reconstructing sphere rays. See [`moon.vert`](../globe/src/moon.vert), [`moon.frag`](../globe/src/moon.frag) and [`deep.rs`](../globe/src/deep.rs).
- **Photometry:** the shader uses common LROC WAC 566 nm Hapke parameters with relative RGB texture modulation, not per-pixel spectral calibration. Photometry uses parallel observer rays, so the close zoom camera cannot create a localized opposition hotspot. Sun and Earth directions remain Moon-fixed from the ephemeris. Only reflected Earth light illuminates the solar night side, with an optional explicit exposure boost. This does not change terrain visibility results.
- **Grid:** the decorative zoom grid appears at 3x and above when zoom changes, holds for one second, then fades out over half a second. It does not snap clicks.
- **Color:** albedo is decoded from sRGB for linear-light shading and encoded for display; relief-derived object-space normals are decoded and renormalized in the shader.
- **GL lifetime:** renderer resources belong to the eframe/glow context. Painting uses an egui GL callback; explicit destruction releases GPU resources. See [`renderer.rs`](../globe/src/renderer.rs) and [`view.rs`](../globe/src/view.rs).

Sources, attribution and offline asset reproduction: [`globe/assets/README.md`](../globe/assets/README.md). Detailed learning material is preserved on [`learn`](https://github.com/Adriano4f/Moon-Landing-Visual/tree/learn).
