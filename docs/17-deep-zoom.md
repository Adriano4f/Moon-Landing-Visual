# Satellite-style deep zoom

Scroll, pinch, or use **+ Zoom** to approach the Moon, up to **2048×** (about
3.39 km above the illustrative sphere). **Go to site** starts at 8×; zoom further
to inspect it. This remains a smooth analytic sphere, not a displaced-terrain
flight or landing simulation. This refreshed overview describes main `5e4d5b7`;
the archived source on `learn` predates these changes. Read
[chapter 23](23-deep-zoom-windows.md) for the full current walkthrough.

The camera moves toward the surface above 1×. Its near plane follows altitude,
and the fragment shader intersects an analytic sphere instead of magnifying
mesh triangles. CPU markers and GPU shading share camera distance/projection.
Moon-fixed tile culling and inverse picking include the dated IAU lunar rotation.
The selected date's Sun/Earth directions remain in use; Realistic uses Hapke,
Fast uses cheaper shading, and both retain phase-scaled Earthshine.

NASA Moon Trek WMTS imagery streams only as visible quadtree patches need it. The global equirectangular `LRO_WAC_Mosaic_Global_303ppd_v02` matrix has 256px JPEG tiles, levels 0–8, top-left longitude/latitude (-180, 90), and dimensions `2^(z+1)` by `2^z`. Its finest angular spacing is about 83 m at the equator. The north/south `LRO_WAC_Mosaic_NPole60_100m_v02` and `LRO_WAC_Mosaic_SPole60_100m_v02` polar stereographic services supply the caps instead of the global mosaic's stretched polar rows. Those matrices have levels 0–5, origin (-1,095,930 m, +1,095,930 m), and pixel spacing `8561.953125 / 2^z` m; the valid square uses `2^z` rows and columns. Finest exposed polar spacing is about 268 m. These matrix values and anonymous CORS headers were checked against Trek's capabilities and tile responses.

The lazy GPU array now reserves 160 layers: 40 MiB RGBA8 base storage, about
53.33 MiB including mipmaps, plus the 576×256 RGBA8 ancestor lookup table
(0.563 MiB). The wanted budget remains 80 tiles. Published layers are protected
while a complete replacement stages in spare layers. Only then is the page
table replaced; the prior complete mosaic stays enabled during loading instead
of flashing back to overview contrast. At most four 256px images upload per
frame, four requests run concurrently, and pending plus in-flight work is
bounded at six. Camera/zoom hysteresis avoids selection chatter. See
[chapter 21](21-atomic-loading-and-flicker.md) for the ownership invariants.

Browsers begin with a 512px photographic preview. Device/settings select the
target overview tier: desktop web normally 4k, mobile 2k, native 8k, subject to
limits and overrides. Realistic's global/polar heights and matching normals
activate as a compatible set with a visibility mask; they are not independently
exposed as soon as each image arrives. Fast performs no rendering-height work.

Above 16×, bounded detail can sharpen the WAC fallback: Fast requests a complete
four-by-four NAC photo window where coverage permits; Realistic exports a 512²
LOLA DEM window, derives normals and retains heights for fine shadow marching.
Regional coverage/NoData and viewport-dependent sample spacing still limit
quality. Global native DEM spacing is about 118.45 m; polar sources can be 5 m
or 30 m, but actual exported spacing may be coarser. Details and measured
examples are in chapters 20 and 23.

Fast photographs contain recorded shadows. Their contrast is combined with
cheap dated lighting and is labeled; they are not calibrated albedo or live
terrain shadows. Realistic excludes that photographic contrast and uses DEM
cast shadows. Magnification beyond the available source spacing still enlarges
existing samples. Trek availability, CORS and browser/driver memory limits
apply. Scientific visibility and exact-point terrain horizons remain separate.
