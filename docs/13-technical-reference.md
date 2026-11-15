# Karaya Explorer: Technical Reference

## 1. Purpose and interpretation

Karaya Explorer compares fixed, published lunar south-polar observation points and dates for access to sunlight and a direct line of sight to Earth. It combines a terrain-derived local horizon, hourly Sun and Earth directions, duration-weighted availability statistics, a panoramic angular view, and an independently rendered Moon globe.

The numerical results describe geometric opportunities, not electrical output, radio service, thermal conditions or landing safety. The globe is an illustration. Its coarse relief, photometric brightness, camera, direction diagrams and optional fill light do not determine the terrain horizon or availability statistics.

The implementation described here is the four-crate Rust workspace, its Python asset tools, seven prepared site datasets, a portable eight-year Sun/Earth ephemeris, and the native and WebAssembly application. The ephemeris supports arbitrary observer coordinates through the scientific API; the current interface retains the prepared site catalog. Units are meters, seconds and radians internally where specified; serialized sky angles and horizon bins use degrees. Longitudes increase eastward. Times are UTC Unix seconds.

## 2. Architecture and data flow

The scientific and rendering preparation pipelines are independent:

1. LOLA DEM files and fixed coordinates feed `prep`. The I/O-free `lunar_core` library traces terrain horizons. Horizons supplies observer-specific Sun and Earth tracks. Preparation validates their time grids and writes one JSON file per site, including terrain provenance.
2. NASA CGI Moon Kit imagery and relief feed the globe asset tools. These bake color and object-space normal textures at three resolutions. The independent `globe` crate loads those textures and renders an illustrative sphere.
3. `prep --ephemeris` obtains reference-observer apparent directions and ranges from Horizons. It reconstructs compact apparent Moon-centered positions, validates their decoded values against all seven independent prepared tracks, and writes a portable binary with a JSON provenance/validation sidecar. `lunar_core::ephemeris` converts these positions to arbitrary observer coordinates without I/O.

At runtime, `app` embeds the small site catalog and a validated 56-byte initial ephemeris record for 2027-01-01 00:00 UTC. It shows real initial Sun/Earth directions with that explicit date, then starts the selected site's JSON only after the first lit Moon draw. No terrain or availability result is invented while it loads. The selected track permanently replaces the initial light. The application validates scientific arrays, computes availability through `lunar_core`, manages the shared epoch and radio margin, and draws the panorama and summary cards. It passes site metadata, current directions and visibility flags into `globe`. No TIFF, DEM tile or terrain ray tracing enters the application or browser. The complete portable ephemeris remains external and is not fetched at startup.

| Component | Responsibility | Principal modules |
|:--|:--|:--|
| `lunar_core` | Projection, great-circle geometry, horizon profiles, track parsing, finite-distance ephemerides, visibility and statistics; no filesystem or network I/O | `core/src/lib.rs`, `core/src/ephemeris.rs`, `core/src/terrain_tests.rs` |
| `prep` | DEM download manifest, native TIFF sampling, Horizons HTTP requests and scientific serialization | `prep/src/main.rs`, `prep/src/dem.rs`, `prep/src/ephemeris.rs`, `prep/download_dem.py` |
| `globe` | Texture selection/loading, mesh, shaders, camera, markers and rays; no dependency on `lunar_core` | `globe/src/lib.rs`, `assets.rs`, `math.rs`, `view.rs`, `camera.rs`, `renderer.rs`, `renderer/setup.rs`, `renderer/textures.rs`, `field.rs` |
| `app` | Validated data loading, site/time synchronization, responsive controls, panorama and summaries | `app/src/lib.rs`, `data.rs`, `layout.rs`, `menu.rs`, `panorama.rs`, `panorama/view.rs`, `sky.rs` |

GPU objects belong to the renderer. Construction registers resources immediately so partial failures can be cleaned. Destruction occurs while the framework's GL context is current. Shared loading slots transfer completed images or diagnostics without transferring GPU ownership. Scientific data and globe texture failures are reported independently.

## 3. Sources, provenance and reproducibility

### 3.1. Scientific terrain

NASA's Planetary Geodynamics Data Archive provides [LOLA product 78](https://pgda.gsfc.nasa.gov/products/78) and [product 90](https://pgda.gsfc.nasa.gov/products/90). Product 78 contains locally adjusted 5 m/pixel surfaces; the selected Site01 tile covers Connecting Ridge. Product 90 provides wider adjusted south-polar surfaces. The selected inventory is:

| File | Map spacing, m | Dimensions | Compressed bytes |
|:--|--:|:--|--:|
| Site01_final_adj_5mpp_surf.tif | 5 | 3,200 by 3,200 | 40,980,810 |
| LDEM_83S_10MPP_ADJ.TIF | 10 | 43,280 by 43,280 | 5,116,709,568 |
| LDEM_80S_20MPP_ADJ.TIF | 20 | 30,400 by 30,400 | 2,696,082,051 |
| LDEM_75S_30MPP_ADJ.TIF | 30 | 30,496 by 30,496 | 2,802,850,273 |
| LDEM_60S_60MPP_ADJ.TIF | 60 | 31,040 by 31,040 | 3,089,110,093 |

The total is 13,745,732,795 bytes. Raw DEMs remain outside the repository. Pixel values are radial surface heights in meters above a 1,737,400 m reference sphere, not absolute radii or integer-encoded visualization heights. Both products identify the MOON_ME mean-Earth frame of DE421.

`prep/dem-manifest.json` records each URL, byte length and SHA-256 digest of the downloaded file. These digests identify retrieved bytes; they are not represented as publisher-supplied checksums. The downloader verifies the manifest. Opening a DEM in `prep` rechecks length and spatial metadata, not a fresh full-file digest on every run. JSON provenance records the manifest assets, sphere radius, DEM surface height and instrument height.

Primary terrain references are Barker et al. (2021), *Improved LOLA Elevation Maps for South Pole Landing Sites: Error Estimates and Their Impact on Illumination Conditions*, DOI [10.1016/j.pss.2020.105119](https://doi.org/10.1016/j.pss.2020.105119), and Barker et al. (2023), *The Planetary Science Journal* 4, 183, DOI [10.3847/PSJ/acf3e1](https://doi.org/10.3847/PSJ/acf3e1). Product 90 also has data DOI [10.60903/gsfcpgda-lola-spole](https://doi.org/10.60903/gsfcpgda-lola-spole).

### 3.2. Ephemerides and coordinates

Preparation queries the [JPL Horizons API](https://ssd.jpl.nasa.gov/api/horizons.api) with observer center `coord@301`, geodetic site coordinates, Sun body 10, Earth body 399, observer ephemerides, quantity 4, degree output, and `APPARENT=AIRLESS`. Observer altitude is the DEM surface height plus instrument height, converted to kilometers. No atmospheric refraction is introduced.

The [Horizons manual](https://ssd.jpl.nasa.gov/horizons/manual.html#lunar-mean-earth-frame-of-de421) specifies the lunar mean-Earth site frame of DE421. The retained preparation diagnostics identify the high-precision MEAN_ME pole/equator model and 1,737.4 km center radius. The service's DE441 orbital ephemeris does not imply a different lunar site-coordinate frame. No additional frame rotation is applied between the PGDA coordinates and Horizons requests.

The bundled interval is 2027-01-01 00:00 UTC through 2030-01-01 00:00 UTC, inclusive, at one-hour spacing: 26,305 samples per body and 26,304 hours of duration. Every site uses a 2 m instrument and 1,440 horizon bearings. Preparation enforces Horizons' 90,000-row query limit and verifies counts, first/last times, cadence and exact Sun/Earth alignment. The parser accepts minute or second precision and optional printed event markers; the final two numeric columns supply azimuth and elevation. Missing delimiters, empty output, nonfinite angles and unordered times are errors.

The four Table 2 south-polar coordinates come directly from Gläser et al. (2017), [Illumination conditions at the lunar poles: Implications for future exploration](https://ntrs.nasa.gov/api/citations/20170007365/downloads/20170007365.pdf). Three further printed coordinates are verified through that paper's Table 1, which attributes them to Mazarico et al. (2011) and Speyerer and Robinson (2013). This is explicitly secondary verification for those entries, not recovery of their original tables. Each catalog entry retains its source annotation and printed coordinate precision.

### 3.3. Globe and photometry assets

The globe uses NASA Scientific Visualization Studio's [CGI Moon Kit](https://svs.gsfc.nasa.gov/4720/), credited to Ernie Wright, with LROC color and LOLA relief. All color tiers derive from `lroc_color_16bit_srgb_8k.tif`. Relief derives from `ldem_16_uint.tif`, a 5,760 by 2,880, 16-pixels-per-degree map with encoding

$$z=0.5q-10{,}000\ \mathrm{m}.$$

This visualization relief is distinct from the high-resolution PGDA scientific terrain. Its color texture is aesthetic sRGB imagery, not measured per-pixel single-scattering albedo. `globe/assets/README.md` and `manifest.json` record source hashes, generated sizes and bake details.

Photometric parameters are cosine-latitude-weighted means of 50,400 valid one-degree tiles in the [LROC WAC 566 nm Hapke parameter map, version 1.1](https://pds.lroc.im-ldi.com/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/SDP/WAC_HAPKEPARAMMAP/WAC_HAPKEPARAMMAP_566NM.IMG). Coverage is 70 degrees south through 70 degrees north. The model is not a polar-specific fit. The reference is Sato et al. (2014), DOI [10.1002/2013JE004580](https://doi.org/10.1002/2013JE004580). Equation verification uses the open USGS ISIS3 implementation `HapkeLROC.cpp`, functions `HGfunc` and `photometry`, at commit `fadd004da2f733525209ee4f66ffdb3a9a9d193e`; it does not rely on an inaccessible publisher appendix.

### 3.4. Preparation commands and artifacts

From the repository root, with the external DEM directory chosen explicitly:

```sh
python3 prep/download_dem.py /home/ubuntu/data/lola
cargo run -p prep --release -- sites.json 2027-01-01 2030-01-01 1h app/assets/data /home/ubuntu/data/lola 2
cargo run -p prep --release -- --ephemeris app/assets/data/moon-ephemeris.bin /home/ubuntu/data/horizons-moon-cache app/assets/data
python3 globe/assets/prepare.py
python3 globe/assets/web_normals.py
python3 globe/assets/photometry.py
```

Omitting the DEM-directory argument selects the explicitly flat legacy preparation path. Production terrain comparisons require that argument. Per-site JSON contains coordinates, aligned tracks, an optional horizon, and optional terrain provenance. The older rounded-coordinate baseline remains separate from the seven-site catalog. Large JSON and texture payloads are external runtime resources, not embedded in the WASM executable.

The ephemeris command runs from the repository root, reads `sites.json`, and requires independent prepared reference files in its third argument. It caches original Horizons responses outside deployed assets. Each body uses eight inclusive annual chunks, at most 8,785 rows per request; repeated annual endpoints are removed. The result has 70,129 hourly samples per body from 2027-01-01 00:00 UTC through 2035-01-01 00:00 UTC. The generated `.meta.json` records the frame, method and complete validation maxima. The binary is 2,244,152 bytes; the measured fresh-cache preparation took 7.396 s on the preparation host, not a latency guarantee.

## 4. Coordinate systems and projections

### 4.1. Geographic, map and raster coordinates

Let latitude be $\phi$, east longitude $\lambda$, and reference radius $R=1{,}737{,}400\ \mathrm{m}$. The supported south-polar stereographic projection has true scale at the south pole, central meridian zero, unit origin scale and no false offsets:

$$\rho=2R\tan\left(\frac{\pi}{4}+\frac{\phi}{2}\right),\qquad x=\rho\sin\lambda,\qquad y=\rho\cos\lambda.$$

Positive map X points toward 90 degrees east; positive map Y points toward longitude zero. Raster rows increase toward decreasing Y. The inverse is

$$\phi=2\arctan\frac{\rho}{2R}-\frac{\pi}{2},\qquad \lambda=\operatorname{atan2}(x,y).$$

Map-to-ground scale is $k=1+\rho^2/(4R^2)$. A raster spacing $p$ therefore supplies local ground spacing $p/k$ to adaptive ray stepping.

For a pixel-is-area tie point $(i_t,j_t,x_t,y_t)$ and square spacing $p$, the first sample center is

$$x_0=x_t+(0.5-i_t)p,\qquad y_0=y_t-(0.5-j_t)p.$$

Fractional column and row are $(x-x_0)/p$ and $(y_0-y)/p$. The half-pixel shift is essential: tie points describe pixel corners, while interpolation uses sample centers.

### 4.2. Moon-fixed Cartesian and local ENU directions

The renderer uses a right-handed Moon-fixed basis: X toward 90 degrees east on the equator, Y toward the north pole, Z toward longitude zero on the equator. At a site,

$$\begin{aligned}
\mathbf U&=(\cos\phi\sin\lambda,\ \sin\phi,\ \cos\phi\cos\lambda),\\
\mathbf E&=(\cos\lambda,\ 0,\ -\sin\lambda),\\
\mathbf N&=(-\sin\phi\sin\lambda,\ \cos\phi,\ -\sin\phi\cos\lambda).
\end{aligned}$$

Horizons azimuth $A$ runs from geographic north toward east, and elevation $e$ is measured above the tangent plane. The body direction is

$$\mathbf d=\cos e\sin A\,\mathbf E+\cos e\cos A\,\mathbf N+\sin e\,\mathbf U.$$

Thus the zenith maps to $\mathbf U$, and zero azimuth at zero elevation maps to $\mathbf N$. Rays, selected-site lighting and the Earth-following camera use this same conversion. Angles are not reconstructed from visibility booleans. Invalid directions remain absent rather than creating a substitute light.

Sun parallax over one lunar radius is approximately 0.00067 degrees, negligible for globe lighting. Earth parallax is approximately 0.259 degrees. Using the selected site's topocentric Earth direction as a globe-wide light/camera direction accepts that latter error. Each site's scientific sky track remains separately topocentric.

### 4.3. Portable apparent ephemerides and finite-distance observers

The portable ephemeris retains finite ranges rather than transporting a topocentric direction unchanged across the Moon. Horizons' `Center pole/equ` diagnostic reports `MEAN_ME (high precision)`. A valid surface reference observer at latitude zero, longitude zero and altitude zero supplies AIRLESS apparent azimuth/elevation and apparent range using quantities `4,20`, degree angles and extra precision. Its returned spherical radius is checked against 1,737.4 km. The retained responses identify the target and center planetary ephemeris as DE441. This verifies the operational frame against the existing apparent tracks, not an independent equality of every ME and PA realization.

A requested geodetic observer at altitude -1,737.4 km is rejected by Horizons because its permitted altitude must exceed -20 km. No unverified inertial-to-body rotation or mean-Earth/principal-axis substitution is used. At the valid reference point, the observer lies on the positive Z axis, east is positive X and north is positive Y. The reconstructed apparent Moon-centered position is

$$\mathbf P=(\Delta\cos e\sin A,\ \Delta\cos e\cos A,\ R+\Delta\sin e),$$

where the apparent range $\Delta$ is converted from AU using $149{,}597{,}870{,}700\ \mathrm{m/AU}$. The binary stores $\mathbf q=\mathbf P/\lVert\mathbf P\rVert$ and $D=\lVert\mathbf P\rVert$ as four little-endian `f32` values per body per hour. After decoding, the direction is renormalized in double precision. These are apparent reference-observer-derived positions, not inertial geometric state vectors: observer-dependent light-time, aberration and deflection are not independently reevaluated for a new observer.

For arbitrary latitude, longitude and radial height $h$ above the reference sphere, including terrain plus instrument height, define

$$\mathbf s=D\frac{\mathbf q}{\lVert\mathbf q\rVert}-(R+h)\mathbf U,\qquad
A=\operatorname{atan2}(\mathbf s\cdot\mathbf E,\mathbf s\cdot\mathbf N)\pmod{2\pi},$$

$$e=\operatorname{atan2}\left(\mathbf s\cdot\mathbf U,\sqrt{(\mathbf s\cdot\mathbf E)^2+(\mathbf s\cdot\mathbf N)^2}\right).$$

Subtracting the observer retains Earth's finite-distance parallax. Invalid positions, non-unit/nonfinite compact directions, invalid observers and coincident body/observer positions return errors. At a zenith the azimuth is geometrically undefined; no scientific inference should depend on that azimuth.

The versioned binary begins with `MLVEPH01`, signed 64-bit start UTC, unsigned 32-bit cadence and unsigned 32-bit count. Each subsequent 32-byte record stores Sun then Earth, each as direction X/Y/Z followed by range in meters. Empty payloads, zero cadence, wrong length/magic, invalid bodies and timestamp overflow are rejected. There is no embedded checksum; structural validation is not protection against every possible bit corruption. The separate [portable ephemeris guide](14-any-point-ephemeris.md) contains the exact schema, reproducible command, source and API index.

## 5. DEM reading and multiresolution sampling

The native `tiff` decoder handles both TIFF and BigTIFF containers and compression. Site01 is strip-organized; the selected large regional files are tiled BigTIFFs. Only the first, full-resolution image directory is read; embedded COG overviews never replace scientific samples. Files are downloaded in full, then accessed through buffered local seekable readers, not an HTTP-range raster layer.

The reader requires a single IEEE float32 grayscale channel, top-left orientation and NaN nodata. It rejects a general transformation matrix, rotated or unsupported affine transforms, nonsquare spacing and incompatible metadata. GeoKey parsing checks the directory structure, scalar count, duplicate keys, inline SHORT values and DOUBLE-table offsets.

Numeric GeoKeys must specify a projected, pixel-is-area model; angular degrees; meter map units; equal 1,737,400 m sphere axes; zero prime-meridian offset; polar stereographic conversion; latitude of origin -90 degrees; central longitude zero; unit origin scale; and zero false easting/northing. Citation text alone is not used to infer these parameters.

Each raster maintains a FIFO cache of decoded strips or tiles, bounded to 512 MiB subject to retaining an individual decoded chunk. Cache hits do not reorder entries. Edge-chunk row strides come from actual decoded dimensions rather than assuming a padded 512-sample row.

For each requested geographic location, rasters are tried in increasing map-pixel spacing. Four finite neighboring sample centers are bilinearly interpolated within one grid. An exact edge uses the last valid cell with endpoint weights. Outside coverage or NaN causes a coarser-raster attempt; TIFF errors propagate and never masquerade as missing coverage. Interpolation does not blend different grids. Only failure of every selected raster returns absent terrain.

## 6. Terrain horizon algorithm

For $n=1{,}440$ azimuth bins, preparation follows independent great-circle rays. At surface arc distance $s$, central angle $\delta=s/R$ and initial bearing $A$, the destination is

$$\sin\phi_2=\sin\phi_1\cos\delta+\cos\phi_1\sin\delta\cos A,$$

$$\lambda_2=\lambda_1+\operatorname{atan2}\left(\sin A\sin\delta\cos\phi_1,\ \cos\delta-\sin\phi_1\sin\phi_2\right).$$

The DEM sampler projects that destination and returns radial surface height $z$ and local ground resolution $p_s$. Observer altitude is $z_0=z_{\mathrm{surface}}+h$, with instrument height $h=2\ \mathrm{m}$ for bundled data. Exact elevation above the observer tangent plane is

$$\epsilon=\operatorname{atan2}\left((z-z_0)-2(R+z)\sin^2\frac{\delta}{2},\ (R+z)\sin\delta\right).$$

This form avoids subtracting nearly equal radii at short distances. It includes spherical curvature and the target's actual radius, rather than a planar $s^2/(2R)$ correction. Equal observer and target radii give $\epsilon=-\delta/2$; a smooth sphere beneath a raised observer has tangent depression $-\arccos[R/(R+h)]$.

The first noncoincident distance is $s_0=p_0/2$, where $p_0$ is observer ground resolution. With bearing-bin width $\Delta A=2\pi/n$, stepping is

$$s_{j+1}=s_j+\min\left[\max\left(s_j\frac{\Delta A}{4},\ s_0\right),\ \frac{p_s}{2}\right].$$

The half-local-pixel upper bound limits skipping of resolved radial features. Each bin retains the largest sampled $\epsilon$. Production tracing has no arbitrary distance cutoff: it stops at the first position without valid coverage in any raster, with $\pi R$ as a nonwrapping safety guard. Missing observer coverage, invalid radii/resolutions, no noncoincident sample, decoding errors or loss of floating-point progress are errors.

The resulting cyclic profile $H(A)$ is linearly interpolated between adjacent bins, including across north. Finite azimuth and radial sampling remain approximations. The algorithm does not certify detection of subpixel obstacles or features falling between adjacent rays, and it does not continue across a complete coverage gap.

## 7. Sun, Earth and statistics

### 7.1. Visible solar area

The fixed apparent solar radius is $r_\odot=0.267$ degrees. Locally approximating the terrain edge as a straight horizontal chord across the disk, define

$$u=\operatorname{clamp}\left(\frac{H(A_\odot)-e_\odot}{r_\odot},-1,1\right),\qquad f_\odot=\frac{\arccos u-u\sqrt{1-u^2}}{\pi}.$$

The result is visible disk area, not visible diameter. Any-sun availability is $f_\odot>0$. It is not incident panel power: panel orientation, efficiency, eclipses, scattered light and thermal behavior are absent. The model uses one horizon value at the solar-center azimuth rather than integrating terrain variation across the entire disk.

### 7.2. Direct Earth clearance

For radio margin $m$, the implemented strict predicate is

$$e_\oplus>H(A_\oplus)+m.$$

The default margin is 0.5 degrees; the interface permits values through 1.0 degree. This is Earth-center angular clearance, not the visible area of the Earth's disk, not a full-Earth-clearance guarantee, and not a link-budget calculation. A portion of the drawn Earth may be above terrain while the radio predicate remains false. Relay links, antenna patterns, Earth stations and scheduling are not modeled.

### 7.3. Time weighting

For ordered samples $t_i$, state at $i$ is held over $[t_i,t_{i+1})$. Every gap contributes its full elapsed duration; the last sample has none. For any availability predicate $b_i$,

$$P_b=100\frac{\sum_{i=0}^{N-2}(t_{i+1}-t_i)b_i}{t_{N-1}-t_0}.$$

Joint availability uses the logical intersection of any-sun and Earth clearance. Mean solar area uses $f_{\odot,i}$ instead of $b_i$ in the weighted numerator. Longest night is the largest consecutive sum of intervals whose left endpoint has no visible Sun. Empty and single-sample inputs return zero-duration statistics; unequal arrays and nonincreasing times violate the core statistics contract.

The shared timeline selects a UTC start and end within 2027-01-01 through 2035-01-01. Hour, day and calendar-month steps control navigation and statistics sampling. Sampling starts at the chosen start, includes the exact end, and weights each held state by elapsed time, including a shortened final interval. Calendar-month steps clamp missing month-end dates. Coarse sampling can miss short events and alias visibility; it is not an event solver. The slider uses second-resolution time and normalized linear interpolation of local direction vectors between hourly samples, including north crossings. Faint paths are clipped to the selected period. Later dates lazily fetch the external compact ephemeris once; site tracks are extended in chunks of at most 512 epochs per frame using the original terrain plus observer altitude. Existing reference samples are retained. Missing sky data suppresses results for the unavailable period rather than relabeling old statistics. See [timeline notes](16-timeline.md).

Summary cards cover the applied start/end period, independent of the selected path window. Held-sample estimates can shift event durations by sampling error and miss short events entirely.

## 8. Globe renderer

### 8.1. Mesh, textures and normals

The unit sphere uses 256 longitude by 128 latitude cells: 33,153 vertices and 65,536 triangles. Its silhouette is spherical; relief is a normal map, not displaced geometry. Texture coordinates are equirectangular:

$$u=\frac{\operatorname{atan2}(x,z)}{2\pi}+\frac12,\qquad v=\frac{\arccos y}{\pi}.$$

Each tier is 2:1: 2,048 by 1,024; 4,096 by 2,048; or 8,192 by 4,096. Color is JPEG, sampled through an sRGB texture so filtering occurs in linear light. Relief normals are RGB8 object-space vectors. The bake differentiates the visualization heights using spherical east/north metric spacing, wraps longitude, applies no height exaggeration and tapers slopes through the final degree near the poles.

Encoding is $q_c=\operatorname{round}[255(n_c/2+1/2)]$. Decoding has maximum component quantization error $1/255$ before renormalization. The 8k normal texture resamples a 16-pixels-per-degree source and does not create additional independent terrain information.

The shader derives texture gradients from the continuous normalized 3D direction, rather than differentiating wrapped longitude. It uses explicit-gradient, trilinear mipmap sampling, with repeat in longitude and clamp at the poles. An anisotropic override is deliberately absent. Normals are renormalized after filtering and transformed with the same rigid matrix as positions and lights.

### 8.2. Tiers and progressive loading

Browser startup uses a 512 by 256 preview; native retains 2k first. Its albedo is downsampled in linear light and its averaged object-space normals are renormalized. The two preview files total 193,091 bytes, compared with 2,975,280 bytes for the browser's 2k pair. A real lit draw precedes the selected site JSON and higher selected tier. Desktop browser default is 4k, phone default is 2k, and native default is 8k, subject to GPU texture-size limits and explicit selection. Native decoding runs on worker threads. Browser decoding uses `HtmlImageElement.decode()` asynchronously and uploads the browser image directly through WebGL, rather than decoding it through the WASM image library.

The browser-only `web-release` profile uses size optimization, LTO, one codegen unit and aborting panics, with optimization level 3 retained for `lunar_core`. Trunk applies `wasm-opt -Oz` and strips debug/name and producer metadata after wasm-bindgen. Renamed Latin, punctuation and geometric-symbol font subsets preserve interface glyph metrics; original licenses accompany the assets. Native fonts and release settings are unchanged. Final WASM size is 3,823,377 bytes, or 1,292,561 bytes with local gzip level 6.

An existing renderer retains its old texture pair while an upgrade decodes. At most one replacement image uploads per frame; both replacements must succeed before the pair becomes active. Failed downloads, decoding or uploads retain the last successful Moon and expose a diagnostic. A failed initial picture releases the site-data gate instead of disabling the independent scientific graph. The first pair still requires initialization and upload before any Moon can be displayed. Mesh and shader objects are reused across upgrades.

Browser normals are lossless WebP copies of the native PNG values. Sizes are 2,250,260; 8,878,268; and 29,849,182 bytes, reducing the PNG transfer by 16.8%, 16.5% and 18.8%, respectively. Decoded RGB values and additional angular error match the PNGs exactly; this does not remove the preceding RGB8 quantization.

Cold HTTP-cache desktop Chrome measurements use software WebGL, 1440 by 1000 pixels and device scale 2. Fast 4G means 500,000 download bytes/s, 750,000 upload bytes/s and 20 ms latency. Readiness follows a real lit draw plus two animation frames. The baseline is deployed Pages; current results use a local gzip server, not a same-server before/after experiment:

| Network | Pages baseline first/full 4k, s | Current gzip first/full 4k, s |
|:--|--:|--:|
| Unthrottled | 1.919 / 5.020 | 0.944 / 3.010 |
| Fast 4G | 11.800 / 37.222 | 3.724 / 30.099 |
| 12.8 Mbps, 40 ms | 4.942 / 14.347 | 1.742 / 11.330 |

Resource Timing totals were 17,068,424 baseline bytes and 13,786,859 current local-gzip bytes. Both finish at 4k, but the new initial picture is coarser. Pages compression is verified by different encoded/decoded WASM and JSON lengths. Selected-site requests start after the first draw; full ephemeris and DEM requests remain zero. The strict first-Moon target below 3 s is not met: initial JS, WASM and preview images nearly exhaust the connection's three-second transfer budget before rendering. GPU upload and mipmap generation remain synchronous; maximum local-gzip frame gaps were 583.3 to 666.6 ms. Faster startup is not evidence of stall-free rendering.

### 8.3. Hapke shading and display transform

For surface normal $\mathbf n$, light direction $\mathbf l$ and view direction $\mathbf v$, let $\mu_0=\mathbf n\cdot\mathbf l$, $\mu=\mathbf n\cdot\mathbf v$, and phase angle $g=\arccos(\mathbf l\cdot\mathbf v)$. The double Henyey-Greenstein phase function is

$$P(g)=\frac{1+c}{2}\frac{1-b^2}{(1-2b\cos g+b^2)^{3/2}}+\frac{1-c}{2}\frac{1-b^2}{(1+2b\cos g+b^2)^{3/2}}.$$

Shadow hiding uses $B_S(g)=B_{S0}/[1+\tan(g/2)/h_S]$. With $\gamma=\sqrt{1-w}$ and $r_0=(1-\gamma)/(1+\gamma)$, the logarithmic multiple-scattering approximation is

$$H(x)=\left[1-wx\left(r_0+\frac{1-2r_0x}{2}\ln\frac{1+x}{x}\right)\right]^{-1}.$$

Using roughness-adjusted incidence/emission cosines $\mu_{0e},\mu_e$ and shadowing factor $S$ from the two USGS incidence/emission roughness branches, the radiance factor is

$$\frac IF=\frac w4\frac{\mu_{0e}}{\mu_{0e}+\mu_e}\left([1+B_S(g)]P(g)+H(\mu_{0e})H(\mu_e)-1\right)S.$$

The implementation returns zero for unlit or invisible facets and guards grazing denominators. It is a radiance factor, not a BRDF per steradian. The fixed map means are:

| Parameter | Value | Meaning |
|:--|--:|:--|
| $w$ | 0.3261142672 | Single-scattering albedo |
| $b$ | 0.2348928321 | HG lobe shape |
| $c$ | 0.3574639778 | HG lobe weighting; positive weights the backward lobe |
| $B_{S0}$ | 1.7707997700 | Shadow-hiding amplitude |
| $h_S$ | 0.0622193166 | Shadow-hiding width |
| $\bar\theta$ | 23.6566009521 degrees | Mean roughness angle |
| $B_{C0},h_C$ | 0, 1 | Coherent-backscatter amplitude and width |
| Porosity parameter | 0 | Porosity correction reduces to unity |

Coherent backscatter is unity because its amplitude is zero. Relative spatial modulation is linear color divided by reference luminance 0.4832, not a per-pixel photometric inversion. Exposure is 8 times relative linear radiance. The display applies Reinhard mapping $C'=C/(1+C)$ and then one sRGB encoding.

Mapped relief fades toward the smooth-sphere response between view cosines 0.65 and 0.35. A view-facing correction enforces a mapped-normal emission cosine of at least 0.01 before renormalization; a geometric day-side guard prevents light leaking across the sphere's terminator. These are display safeguards for unresolved normal-map relief, not terrain shadow calculations.

Until polar detail is ready, the globe uses a schematic spherical normal and reference mean color within 10 degrees of either pole, blending through 15 degrees. The northern fallback remains. At the south pole, lazy runtime Trek WMTS imagery and offline LOLA-derived stereographic normals replace that fallback through 75 to 77 degrees south. A 1,024 square atlas uses level 3 tiles on the default phone profile; a 2,048 square atlas uses level 4 on desktop. Their spacing is approximately 1,070 and 535 m respectively, not the source DEM's 30 m resolution. Three requests run concurrently, one image uploads per frame, and the previous complete material remains until every tile and normal succeeds. Nothing is requested before the first overview draw.

For unit Moon-fixed direction $\mathbf p$, the south-polar mapping is $X=2Rp_x/(1-p_y)$, $Y=2Rp_z/(1-p_y)$, with radius $R=1{,}737{,}400\ \mathrm{m}$. The atlas half-width is 547,965 m, $u=1/2+X/(2E)$ and $v=1/2-Y/(2E)$. Normals differentiate reconstructed spherical terrain positions and are normalized before RGB8 encoding. Photographic shadows are recorded, not date-dependent. The mosaic therefore supplies only a 25 percent bounded contrast adjustment, not calibrated albedo or live terrain shadowing. See [polar data, projection and loading details](15-polar-detail.md). The numerical panorama remains based on the detailed PGDA horizon.

### 8.4. Real illumination, earthshine and far-side fill

The selected site's transformed Sun and Earth directions set directional lights at the shared epoch. Missing tracks disable physical sunlight rather than restoring a fixed presentation Sun. There is no scale three-body orbital scene: changing directions, libration and the terminator are ephemeris driven, while camera distance and body diagrams remain illustrative.

For normalized Moon-to-Sun and Moon-to-Earth vectors $\mathbf s,\mathbf e$, Earth phase is approximated by $\alpha_\oplus=\pi-\arccos(\mathbf s\cdot\mathbf e)$. Reflected Earth irradiance relative to sunlight is

$$q=A_g\left(\frac{R_\oplus}{d}\right)^2\frac{\sin\alpha_\oplus+(\pi-\alpha_\oplus)\cos\alpha_\oplus}{\pi}.$$

NASA [Earth](https://nssdc.gsfc.nasa.gov/planetary/factsheet/earthfact.html), [Moon](https://nssdc.gsfc.nasa.gov/planetary/factsheet/moonfact.html) and [Sun](https://nssdc.gsfc.nasa.gov/planetary/factsheet/sunfact.html) fact-sheet values are $A_g=0.434$, $R_\oplus=6{,}371\ \mathrm{km}$ and $d=384{,}400\ \mathrm{km}$. Full-Earth $q$ is approximately 0.000119217, or 0.162 W per square meter for solar irradiance 1,361 W per square meter. This is an approximate Lambert-phase, mean-distance reflected-light model, not Earth weather or spatially resolved radiance. The explicitly labeled boosted setting multiplies only Earthshine by 1,000.

Far side fill is separate and nonphysical. For object-space unit position $\mathbf p$, its mask is a smooth transition in $-p_z/\sqrt{p_x^2+p_z^2}$, positive beyond 90 degrees from mean Earth longitude zero. It adds linear gray-blue color proportional to $(0.035,0.045,0.065)$ and relative albedo. It follows the mean far hemisphere, not the instantaneous terminator or Earth-facing boundary. Neither boost nor fill changes any availability calculation.

### 8.5. Rays, markers, picking and camera

Site rays start at the marker's unit radial vector and extend 0.6 lunar radii toward the site's own Sun and Earth directions. Yellow denotes Sun, blue Earth, and blocked directions use 35% brightness. Rays are unlit diagram overlays, not scale distances or terrain-occluded beams.

Screen-facing triangle ribbons replace implementation-dependent GL line widths. A 2.5-logical-pixel support width and transverse smooth alpha coverage give approximately 1.44 logical pixels of integrated line coverage. Endpoint disks have 2-logical-pixel support radius and radial smooth coverage. Device scale converts logical dimensions to framebuffer pixels. Alpha blending and disabled depth writes preserve the globe depth buffer; the selected-site diagrams are drawn without depth occlusion to retain blocked-direction legibility.

The optional whole-Moon Sun field samples 512 equal-area Fibonacci sphere candidates and retains those with $\mathbf p\cdot\mathbf s>0$, approximately 256 rays. Parallel segments run from $\mathbf p+(0.6/\mathrm{zoom})\mathbf s$ into surface endpoint $\mathbf p$. They use the real current Sun direction, one batched triangle draw, depth occlusion against the sphere and no endpoint disks. Lengths are illustrative; there is no global Earth-ray field. Invalid or absent light does not create a substitute field.

All published markers have contrasting outlines. Unselected radius is 6 logical pixels; selected radius is 9, with a name label. Back-hemisphere points are hidden. Picking chooses the nearest projected point within 14 logical pixels; the site list disambiguates close overlaps without displacing coordinates.

The shared model transform is $M=R_x(\theta)R_y(\alpha)R(t)$, where $R(t)$ maps Moon-fixed coordinates to J2000 equatorial space using all E1–E13 terms of the IAU lunar orientation model. Renderer axes are $(Y,Z,X)$, so inertial north is up. UTC approximates TDB for display only; this is not a DE-series attitude solution. See [dated Moon rotation](17-inertial-rotation.md) for the source, limitations and optional Earth-centered ecliptic orbit inset. The default camera follows Earth; dragging enters Free, which stays fixed in inertial space as time advances. The perspective camera remains at $(0,0,5)$ lunar radii; the same rigid transform applies to geometry, lights, normals, polar textures, markers, rays and tile overlays. Picking uses its transpose. Earth-following mode first transforms the Earth direction to $\mathbf e=R(t)\mathbf e_{fixed}$, then uses

$$\alpha=-\operatorname{atan2}(e_x,e_z),\qquad \theta=\arcsin(e_y).$$

Go to site applies the same camera-angle equations to the site's rotated radial vector, mapping it to the image center. It interpolates the shortest yaw difference, tilt and logarithmic zoom over 0.8 s with cubic easing. Interaction or a new selection cancels the transition. Wheel, pinch and zoom buttons change magnification; optional auto-orbit remains an artificial camera aid. These camera operations do not alter scientific time or observer coordinates.

## 9. Application state and interface

Earth view names the approximate phase and geometric illuminated fraction using the renderer's real apparent directions:

$$\gamma=\arccos(\hat{\mathbf s}\cdot\hat{\mathbf e}),\qquad f_{lit}=\frac{1+\cos\gamma}{2}.$$

The angular supplement is the Earth phase used for earthshine. The lunar phase estimate retains the existing small topocentric-direction approximation, and is not a brightness prediction or eclipse model. The visible cap of the illustrative finite-distance camera differs from the distant Earth-facing hemisphere. At camera distance $d$, no direct solar patch is visible when $\gamma\geq\pi/2+\arccos(1/d)$. Thus the previous $d=3.3$ camera could entirely hide a crescent with up to 2.351 percent geocentric illuminated area; at $d=5$ the threshold is 1.010 percent. This is explicitly near-new-Moon geometry, not an Earth object eclipsing the Sun. Below 10 percent illuminated area, a one-click shortcut enables the existing boosted earthshine aid.

The paint viewport has a 200-logical-pixel minimum height, preventing phase controls from pushing the instruction overlay across the globe on a phone; the surrounding layout scrolls instead.

Representative bundled C1 epochs independently give the following approximate phase geometry; dates are UTC in 2027:

| Epoch | Phase angle, degrees | Illuminated area, percent |
| :--- | :--- | :--- |
| January 5, 03:00 | 150.0226 | 6.6889 |
| January 7, 21:00 | 176.8867 | 0.0738 |
| January 9, 05:00 | 165.1818 | 1.6629 |
| January 22, 12:00 | 1.4023 | 99.9850 |
| January 29, 11:00 | 89.8861 | 50.0994 |

The compact menu exposes Site and Time, with specialist groups under collapsed More options and a separate shortened Explanations section. The site selector's help includes one short publication link instead of a Source section or raw citation string. Secondary coordinates cite the verified secondary table. The optional whole-Moon field is labeled Sun rays. Shared help rendering limits every question-mark explanation to 25 words, including linked captions, and 232 logical pixels of content, approximately 260 pixels including window chrome, for hover and tap alike. Longer help cannot widen a phone screen. The panorama labels both angular axes in degrees, names Sun/Earth/ground in a persistent legend, emphasizes zero height, and keeps the actual vertical stretch visible. Short green/red statuses indicate availability without promising electrical output or radio service; no scientific results change with this presentation.

The shared dark style uses a blue interaction accent, 10 pixel item spacing, 8 to 10 pixel corner radii and a 24/15/14/12 pixel heading/body/button/caption hierarchy. Ordinary controls reserve 36 pixel desktop and 44 pixel phone heights. Small circular help glyphs use the same full interaction target. Zero-height help explicitly identifies the observer's local horizontal plane: positive angles are above eye level and negative angles below it, independent of terrain altitude. Theme changes never alter scientific state.

Only the catalog is embedded. Selecting a site starts its native disk or browser HTTP load once and caches the validated result. Switching back reuses it. All-sites ray mode requests other tracks on demand and displays readiness and individual failures. Each completion slot belongs to its own model, preventing an older request from replacing another site's state.

Native resource directories can be overridden with `MOON_DATA_DIR` and `MOON_ASSET_DIR`; `MOON_TIER` selects a native texture target subject to hardware limits. Browser resources use relative `assets/data/` and `assets/globe/` paths. The page title is Karaya Explorer.

The application retains one UTC epoch and one applied statistics period and step across sites. Display directions interpolate normalized local vectors between adjacent hourly epochs; statistics use the selected sampling step with duration weighting. Out-of-range background epochs yield no ray. Radio margin is propagated to all-site predicates, including after pending data arrives. Track validation rejects empty/mismatched arrays, unordered or unrepresentable times, invalid angles, differing Sun/Earth epochs, and empty or invalid supplied horizons. Missing legacy terrain is explicitly labeled flat rather than confused with a computed profile.

Desktop controls occupy grouped, collapsible sidebar sections: Site, Time, View/camera, Lighting, Rays, Radio margin, Technical and Explanations. Site and Time start expanded. A phone drawer preserves the main stacked globe/panorama layout. A compact vector crescent and Karaya Explorer label remain visible. Question-mark controls provide hover and click explanations for controls, statuses and cards.

The default panorama is a black-sky 360-degree strip, north through east, south, west and north. Elevation is vertically exaggerated and labeled. Faint paths show the selected time window and split at north wrap. Current solar and Earth disks use fixed angular radii 0.267 and 0.95 degrees. Terrain is tessellated into adjacent trapezoids and drawn after the disks, hiding covered portions. Status text uses the same core horizon and predicates. Technical view substitutes a numerical azimuth/elevation plot of the same data, not a second physical model.

## 10. Validation and publication comparisons

### 10.1. Prepared results

All computed rows below use the same 2027 through 2030 hourly interval, 2 m observer, 1,440 bearings and unmodified terrain algorithm. Coordinates are preserved as printed:

| Site | Latitude, degrees | East longitude, degrees | Verification |
|:--|--:|--:|:--|
| Connecting Ridge C1 | -89.4398 | 222.83 | Gläser Table 2 |
| Connecting Ridge C2 | -89.3130 | 245.98 | Gläser Table 2 |
| Shackleton S1 | -89.6866 | 197.19 | Gläser Table 2, first row |
| Shackleton S2 | -89.7815 | 204.70 | Gläser Table 2, second row |
| Shackleton Mazarico | -89.78 | 204.27 | Gläser Table 1, Mazarico |
| Connecting Ridge Speyerer | -89.418 | 221.3 | Gläser Table 1, Speyerer |
| Shackleton Speyerer | -89.740 | 201.2 | Gläser Table 1, Speyerer |

| Site | Mean solar area, % | Any Sun, % | Earth 0.5 degrees, % | Both, % | Longest night, h |
|:--|--:|--:|--:|--:|--:|
| Connecting Ridge C1 | 86.7065 | 91.4462 | 52.6080 | 49.7567 | 111 |
| Connecting Ridge C2 | 78.7481 | 82.6680 | 53.0148 | 45.0882 | 164 |
| Shackleton S1 | 84.3611 | 89.1385 | 48.4831 | 44.7156 | 67 |
| Shackleton S2 | 81.6761 | 88.4466 | 51.2622 | 45.0654 | 65 |
| Shackleton Mazarico | 0.0000 | 0.0000 | 0.0000 | 0.0000 | 26,304 |
| Connecting Ridge Speyerer | 56.2341 | 57.5578 | 16.6781 | 9.4852 | 356 |
| Shackleton Speyerer | 74.2901 | 78.5850 | 36.8157 | 29.7787 | 140 |

The publications' comparison values are separate:

| Published site | Mean illumination, % | Solar visibility, % | Maximum darkness, h |
|:--|--:|--:|--:|
| C1, Gläser Table 2 | 88.0 | 92.1 | 112 |
| C2, Gläser Table 2 | 78.1 | 82.0 | 165 |
| S1, Gläser Table 2 | 85.5 | 89.4 | 66 |
| S2, Gläser Table 2 | 82.1 | 89.8 | 65 |
| Shackleton Mazarico, Table 1 | 86.7 | 90.5 | Not supplied |
| Connecting Ridge Speyerer, Table 1 | 64.2 | Not supplied | 308 |
| Shackleton Speyerer, Table 1 | 71.6 | Not supplied | 145 |

Gläser Table 2 uses a 2 m observer over 2017 through 2037. Table 1 compares surface-level calculations and observations: Mazarico used coarser LOLA terrain, while Speyerer used roughly a year of images. The original Mazarico simulation dates have not been verified. Earth and joint availability are not supplied by these publication tables. Period, height, resolution, coordinate precision and metric differences preclude treating numerical proximity as an exact reproduction.

### 10.2. Non-reproduction and sensitivity controls

At the printed Shackleton Mazarico coordinate, the computed minimum horizon is +2.294499 degrees. Maximum sampled solar-center elevation is +1.738698 degrees; even adding the 0.267-degree radius leaves the disk below every terrain bearing. This directly accounts for zero computed solar availability, but does not isolate why that terrain run differs from the published 86.7%. Coordinate precision, surface sampling, height and interval contributions remain unresolved. No coordinate or terrain parameter is adjusted to force agreement. Connecting Ridge Speyerer also retains a substantial reported discrepancy.

A distinct rounded Connecting Ridge baseline at -89.45 degrees, 222.69 degrees east gives 52.0027% mean solar area at 2 m and 92.4686% at 10 m. A DEM-elevation-only local crest control gives 89.2030% at 2 m; the published Table 2 C1 gives 86.7065%. These controls demonstrate sensitivity to topographic placement and observer height. They do not reproduce the older separately quoted 89.01% experiment or establish that coordinate rounding alone explains every disagreement. The baseline is retained unchanged and is not substituted into the catalog.

### 10.3. Automated and visual coverage

The decoded compact ephemeris is compared against all 26,305 original hourly epochs per site from 2027-01-01 through 2030-01-01, using each prepared surface height plus its 2 m observer height. Wrapped azimuth and ordinary elevation differences give:

| Site | Sun max azimuth, degrees | Sun max elevation, degrees | Earth max azimuth, degrees | Earth max elevation, degrees |
|:--|--:|--:|--:|--:|
| Connecting Ridge C1 | 0.00001791 | 0.00000073 | 0.00001302 | 0.00003013 |
| Connecting Ridge C2 | 0.00001717 | 0.00000068 | 0.00000943 | 0.00003032 |
| Shackleton S1 | 0.00001787 | 0.00000063 | 0.00001321 | 0.00003007 |
| Shackleton S2 | 0.00001747 | 0.00000071 | 0.00001297 | 0.00003013 |
| Shackleton Mazarico | 0.00001780 | 0.00000063 | 0.00001333 | 0.00003026 |
| Connecting Ridge Speyerer | 0.00001798 | 0.00000065 | 0.00001340 | 0.00003029 |
| Shackleton Speyerer | 0.00001774 | 0.00000065 | 0.00001327 | 0.00003019 |

All components meet the enforced limits of less than 0.01 degrees for Sun and less than 0.02 degrees for Earth. This is a transport/quantization consistency test against separately requested apparent tracks, not an independent test of the Horizons ephemeris model. The 2030 through 2035 extension and locations outside these seven south-polar references have no independent topocentric comparison here. Parser tests check frame, radius, observer coordinates, body, malformed data, bounds and cadence; binary and geometry tests cover normal, empty, corrupt and invalid inputs plus analytic finite-distance parallax.

The Rust workspace tests cover projection axes, pixel centers, bilinear interpolation, cyclic horizons, spherical elevation identities, great-circle directions, local walls and ridges, adaptive-sampler failure cases, solar area, strict Earth margins, duration weighting, Horizons optional markers/errors, malformed JSON and ephemeris alignment. TIFF fixtures cover strips, corrupt input, wrong sphere/GeoKeys, nodata and finest-valid fallback. Globe tests cover ENU conversion, camera/projection consistency, mesh winding/seams, ribbon dimensions, invalid/empty scene states, sunlit field distribution, loading failures and Earthshine phase/scale.

Bundled C1 directions also undergo a full-series physical-range regression. Measured subsolar latitude is -1.587370 through +1.559565 degrees; sub-Earth longitude is -8.043021 through +8.053838 degrees and latitude -6.609914 through +7.095109 degrees. These are dataset checks, not independent ephemeris accuracy measurements.

Python asset checks cover normal generation/quantization and photometric geometry, front-facing normal correction, exposure and trilinear sampling policy. Downloader tests exercise manifest verification and failure behavior. Native release and WASM builds, clippy, Trunk release, and browser shader linking complement the unit tests.

Pixel 7 emulation exercises the drawer, zoom controls, site selection and ray toggle. Refactoring preserved a matching default phone render pixel-for-pixel. Browser checks use software WebGL and emulated devices, not a physical-phone or all-driver qualification. Publication comparisons are regression evidence and sensitivity studies, not validation of landing-safety predictions.

## 11. Known limits

- The lunar reference is spherical, with radial instrument height. Terrain resolution, bilinear smoothing, inter-ray spacing and coverage boundaries limit horizon accuracy. Local boulders, new lander structures and unresolved obstacles are absent.
- Solar disk radius is fixed and its horizon crossing uses one local chord. Finite-time sampling, no eclipse model and no event-time solver limit short-window accuracy.
- Earth visibility is an angular center-clearance test, not a communications system simulation. No relay, antenna, station, atmospheric or operational availability is included.
- Printed published coordinates can identify different slopes or depressions from a study's internally selected maximum. Several comparisons remain non-reproduced. No parameters are tuned to published percentages.
- Globe normals cannot produce resolved terrain shadows, topographic silhouettes or scientific local horizons. The northern cap and unloaded southern cap are schematic. Loaded southern photographic contrast includes recorded shadows; normal-map spacing is 535 or 1,070 m. Overview 8k does not add independent relief beyond its source map.
- Globally averaged 566 nm Hapke parameters and relative RGB modulation are not a calibrated broadband spectral model. Normal safeguards, exposure, tone mapping, Earthshine boost and fill light affect appearance only.
- Earthshine uses a Lambert phase and mean separation, with no clouds or spatial Earth map. Globe-wide Earth lighting/camera accepts topocentric parallax of about 0.26 degrees.
- Progressive decoding does not eliminate synchronous GPU upload/mipmap stalls. Initial WASM, JSON and texture transfer remains network dependent; 8k remains an explicit large-memory/download choice in the browser.
- The current interface compares only the fixed catalog and its original dates. An eight-year arbitrary-observer ephemeris API and asset exist, but there is no arbitrary-point DEM workflow, live ephemeris service, route planner or scale Sun-Earth-Moon orbital model. Apparent-position transport is empirically checked only for the seven south-polar sites over the original three-year overlap.

## 12. Source and artifact index

The [source repository](https://github.com/Adriano4f/Moon-Landing-Visual) is authoritative for current code and data. The fixed-catalog implementation baseline is revision `5d0f4fa9c6bcef2a7ed1e2cf5df8400fc2593dff`; the portable ephemeris extension uses format `MLVEPH01` and is detailed in the [ephemeris guide](14-any-point-ephemeris.md).

The reproducibility records are `sites.json`, `prep/dem-manifest.json`, `app/assets/data/*.json`, `app/assets/data/moon-ephemeris.bin`, `globe/assets/manifest.json` and `globe/assets/README.md`. Scientific geometry and statistics are implemented in `core/src/lib.rs`; compact ephemerides in `core/src/ephemeris.rs`; DEM interpretation in `prep/src/dem.rs`; preparation orchestration in `prep/src/main.rs` and `prep/src/ephemeris.rs`. Renderer equations are in `globe/src/moon.frag`, transforms in `globe/src/math.rs`, and application state/presentation in the modules listed in Section 2.

Build and regression commands are:

```sh
cargo test --workspace
cargo clippy --workspace
cargo build -p app --release
cargo check -p app --target wasm32-unknown-unknown
python3 -m unittest discover -s globe/assets -p 'test_*.py'
python3 -m unittest discover -s prep -p 'test_download_dem.py'
```

Run `trunk build --release` from `app/` for the browser distribution. The rendered reference is supplied as `reference.pdf` with its LaTeX source `reference.tex`.
