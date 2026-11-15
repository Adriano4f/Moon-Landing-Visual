# Real terrain horizons from zero

## 1. The scientific question and the boundary of this phase

At a fixed lunar landing site, terrain can hide the Sun needed for power or the Earth needed for a direct radio link. A horizon profile records the highest terrain elevation angle in every horizontal direction. Once computed, it can be reused for many dates: compare the body's azimuth and elevation against the profile rather than tracing the terrain again at every hour.

This phase prepares the fixed sites in sites.json. It does not redesign the UI, add interactive point selection, or add region comparisons. The globe remains an illustrative rendering, not a scientific illumination simulation. Raw DEMs stay outside the repository. The browser receives only the prepared tracks and 1,440 horizon angles, not gigabytes of topography.

A digital elevation model, abbreviated DEM, is a rectangular array of surface heights. A pixel is an array cell with a spatial footprint; its stored height represents the surface at a sample location. A map projection converts spherical latitude and longitude into planar coordinates used to address the array. These are three different things: array indices, projected meters, and geographic angles.

The implementation boundary is deliberate. lunar_core contains coordinates, great-circle rays, exact spherical elevation, interpolation, solar area, visibility and statistics, with no filesystem or TIFF dependency. prep owns TIFF decoding, projection-metadata validation, buffered file access and caches. The app loads the resulting JSON through its existing native disk or web HTTP path. Read [Rust and the globe code](04-globe-code.md) for ownership and borrowing and [the panorama guide](05-panorama.md) for immediate-mode UI and data loading.

## 2. Sources and the selected inventory

The sources actually read are NASA PGDA [product 78](https://pgda.gsfc.nasa.gov/products/78), its [README](https://pgda.gsfc.nasa.gov/data/LOLA_5mpp/README), and [product 90](https://pgda.gsfc.nasa.gov/products/90). Product 78 describes locally adjusted 5 m/pixel LOLA surfaces; Site01 is the connecting ridge. Product 90 supplies wider adjusted surfaces at several resolutions. Both pages specify south polar stereographic X/Y coordinates in meters in the MOON_ME frame of DE421. Product 78 and 90 describe LDEM pixel values as surface height in meters, not absolute radius or encoded integer counts.

Cite Barker et al. (2021), Improved LOLA Elevation Maps for South Pole Landing Sites: Error Estimates and Their Impact on Illumination Conditions, Planetary and Space Science 203, 105119, DOI [10.1016/j.pss.2020.105119](https://doi.org/10.1016/j.pss.2020.105119), for PGDA 78. Cite Barker et al. (2023), The Planetary Science Journal 4, 183, DOI [10.3847/PSJ/acf3e1](https://doi.org/10.3847/PSJ/acf3e1), and data DOI [10.60903/gsfcpgda-lola-spole](https://doi.org/10.60903/gsfcpgda-lola-spole), for PGDA 90.

The selected inventory is one local surface covering the fixed example plus the finest published PGDA 90 surface for each successively wider coverage band. Redundant coarser versions, hillshades, count maps, error maps, roughness maps and point clouds are not required by this deterministic horizon calculation. This is not a download of the entire PGDA 78 catalog: only Site01 is selected for the current fixed-site list. Other published local tiles are not silently substituted or claimed to have been used.

| File | Pixel scale | Dimensions | Exact bytes |
|:--|:--|:--|:--|
| Site01_final_adj_5mpp_surf.tif | 5 m | 3,200 by 3,200 | 40,980,810 |
| LDEM_83S_10MPP_ADJ.TIF | 10 m | 43,280 by 43,280 | 5,116,709,568 |
| LDEM_80S_20MPP_ADJ.TIF | 20 m | 30,400 by 30,400 | 2,696,082,051 |
| LDEM_75S_30MPP_ADJ.TIF | 30 m | 30,496 by 30,496 | 2,802,850,273 |
| LDEM_60S_60MPP_ADJ.TIF | 60 m | 31,040 by 31,040 | 3,089,110,093 |

Total: 13,745,732,795 bytes, about 12.80 GiB, below the user's approximately 20 GB gate. Decimal GB is $10^9$ bytes; binary GiB is $2^{30}$ bytes. The table gives compressed file bytes, not decoded RAM consumption. The exact URLs and observed SHA-256 digests are in the [versioned manifest](../prep/dem-manifest.json), quoted below. Digests identify the bytes obtained over HTTPS; they are not claimed to be publisher-supplied checksums.

The widest nominal coverage is 60 through 90 degrees south, deliberately much larger than the local landing-site area. This includes nearby features at the finest selected resolution, intermediate terrain without jumping immediately to 60 m pixels, and far terrain well beyond plausible horizon-forming distances. There is no independently selected ray cutoff: every ray proceeds to the first point without valid coverage in any selected raster. The actual square bounds and NaN mask, not the filename's nominal latitude, decide availability.

## 3. TIFF and GeoTIFF from zero

TIFF is a binary container. Its header identifies byte order and a directory offset. An image file directory, abbreviated IFD, contains tagged entries. Each entry has a numeric tag identifier, a numeric value type, a count and either inline values or an offset to values elsewhere in the file. SHORT means an unsigned 16-bit integer; DOUBLE means an IEEE 64-bit floating-point number; ASCII is text. Classic TIFF uses smaller offsets; BigTIFF uses 64-bit offsets for files that may exceed 4 GiB. The tiff crate handles both binary container layouts and compression, so prep does not implement a second TIFF codec.

Images can be organized into strips spanning rows or rectangular tiles. The decoder calls either one a chunk. Product 78 Site01 is strip-organized; the selected product 90 surfaces are tiled BigTIFFs with 512 by 512 nominal tile dimensions. An edge tile's decoded dimensions exclude padding. Raster::pixel therefore asks chunk_data_dimensions for its actual row stride rather than assuming every decoded row has 512 samples.

A cloud-optimized GeoTIFF, abbreviated COG, organizes directories, tiles and lower-resolution overview images for efficient range access. An overview is a separate reduced image, not extra fine terrain. This implementation decodes the first full-resolution IFD only and never advances to overviews. It downloads the selected original files in full and then performs local Read + Seek; the Rust sampler is not an HTTP-range virtual filesystem.

GeoTIFF adds spatial tags to TIFF. The relevant tags are:

| Tag | Meaning | Interpretation here |
|:--|:--|:--|
| 256, 257 | ImageWidth, ImageLength | Raster sample counts |
| 339 | SampleFormat | IEEE floating point, value 3 |
| 33550 | ModelPixelScaleTag | Positive X/Y spacing in map meters |
| 33922 | ModelTiepointTag | Raster corner coordinate paired with map coordinate |
| 34264 | ModelTransformationTag | General affine matrix, rejected rather than ignored |
| 34735 | GeoKeyDirectoryTag | Compact projection and coordinate-system keys |
| 34736 | GeoDoubleParamsTag | Floating-point values referenced by GeoKeys |
| 34737 | GeoAsciiParamsTag | Human-readable coordinate-system citations |
| 42113 | GDAL_NODATA | NaN for these published surfaces |

The reader requires a single float32 gray channel, top-left row orientation and NaN nodata. It rejects unsupported transforms, a different projection, a different sphere, a corrupt decoder result or an incorrect file length. A decompression error is an error, not an excuse to use coarser terrain. Only valid missing coverage or nodata triggers a fallback.

Implementation: [prep/src/dem.rs:180](../prep/src/dem.rs#L180).

## 4. GeoKeys and verified projection parameters

GeoKeyDirectory begins with four SHORT values: directory version, revision, minor revision and number of keys. Each subsequent entry contains four SHORTs: key identifier, storage location, count and value/offset. Storage location 0 means the last SHORT is the value. Location 34736 means the last SHORT indexes GeoDoubleParams. The key reader requires a scalar, checks offsets and rejects duplicate keys. It does not assume that the eight doubles occur in any particular order.

All selected files encode the following parameters. In the downloaded files, GeoDoubleParams contains $[-90,0,1,0,0,1737400,1737400,0]$. Their meanings come from the keys pointing into that table, not from that displayed ordering.

| GeoKey | Decoded value | Meaning |
|:--|:--|:--|
| 1024 | 1 | Projected model |
| 1025 | 1 | RasterPixelIsArea |
| 2054 | 9102 | Angular unit is degree |
| 2057, 2058 | 1,737,400 m | Equal semimajor/semiminor radii: a sphere |
| 2061 | 0 degrees | Prime-meridian longitude |
| 3075 | 15 | Polar stereographic conversion |
| 3076 | 9001 | Linear unit is meter |
| 3081 | -90 degrees | Natural origin at the south pole |
| 3095 | 0 degrees | Straight vertical pole longitude |
| 3092 | 1 | Scale factor at the natural origin |
| 3082, 3083 | 0 m | False easting and false northing |

True scale at the pole follows from the encoded natural origin and unit scale factor. It is not a separately invented latitude of true scale. Easting increases toward longitude 90 east; northing increases toward longitude zero. Rows increase toward decreasing northing. The product 90 citation reads Moon (2015) - Sphere / Ocentric / South Polar. Site01's textual citation is generic, but its numeric GeoKeys encode the same sphere and conversion; the product page/README supplies the MOON_ME/DE421 frame identification.

The horizontal GeoKeys do not define a separate vertical datum. The product pages define surface-height units; the radius keys define the horizontal sphere. NASA's [LOLA polar illumination documentation](https://pgda.gsfc.nasa.gov/products/69) explicitly describes elevations relative to a 1,737.4 km sphere, consistent with these surface-height values and reference radius. We use $R+z$ as surface radius. No terrestrial geoid correction, integer rescaling, or guessed vertical offset is applied. This distinction matters: a horizontal semimajor-axis key alone would not prove a vertical datum.

The [Horizons manual](https://ssd.jpl.nasa.gov/horizons/manual.html#lunar-mean-earth-frame-of-de421) states that lunar site coordinates use the mean-Earth frame of DE421, with the high-precision lunar rotation model over 1550 through 2650. Our dates lie inside that interval. The preparation log retains the actual Center pole/equ label, MEAN_ME (high precision), and Center radii of 1,737.4 km. The manual describes that mean-Earth convention as the standard DE421 frame; the service's orbital ephemeris source DE441 does not change that site-coordinate convention. An additional frame rotation is therefore not applied.

Implementation: [prep/src/dem.rs:70](../prep/src/dem.rs#L70), [prep/src/dem.rs:101](../prep/src/dem.rs#L101).

## 5. From latitude/longitude to sample centers

Let $phi$ be latitude, $lambda$ east longitude, and $R=1{,}737{,}400$ m. Angles inside trigonometric functions are radians. Spherical south-polar stereographic projection with the verified parameters is

$$
\rho=2R\tan\left(\frac{\pi}{4}+\frac{\phi}{2}\right),\qquad
x=\rho\sin\lambda,\qquad y=\rho\cos\lambda.
$$

The pole maps to $(0,0)$. At longitude zero, going north increases $y$. At longitude 90 east, going north increases $x$ and geographic east initially points toward negative $y$. A map-grid bearing and geographic azimuth are therefore not interchangeable away from a special meridian. Core tests preserve the earlier axis/ridge tests and add the labeled pole, known-angle 60S coordinate, and a Site01 corner-coordinate round trip.

For a tie point $(I,J)\leftrightarrow(X,Y)$ and pixel scale $p$, RasterPixelIsArea means integer raster coordinates address corners. The first sample center is

$$x_0=X+(0.5-I)p,\qquad y_0=Y-(0.5-J)p.$$

Site01's actual tag is $(0,0)\leftrightarrow(-19{,}000,-4{,}000)$ m and $p=5$ m. Its first center is $(-18{,}997.5,-4{,}002.5)$ m, not the corner. This reconciles the TIFF corner transform with the PGDA README's pixel-registered center convention. The reader fixture asserts those actual numbers. Missing the half-pixel shift would move every sample by 2.5 m in each axis, significant for a high-resolution skyline.

| Surface | Corner X range, m | Corner Y range, m |
|:--|:--|:--|
| Site01 | -19,000 through -3,000 | -20,000 through -4,000 |
| 83S | -216,400 through 216,400 | -216,400 through 216,400 |
| 80S | -304,000 through 304,000 | -304,000 through 304,000 |
| 75S | -457,440 through 457,440 | -457,440 through 457,440 |
| 60S | -931,200 through 931,200 | -931,200 through 931,200 |

Usable interpolation-center bounds are inset by half a pixel on every edge, and actual valid data may be reduced by NaNs. The four surrounding centers define bilinear interpolation. For fractions $a=c-\lfloor c\rfloor$ and $b=r-\lfloor r\rfloor$, weights are $(1-a)(1-b),a(1-b),(1-a)b,ab$. Contributing NaN corners invalidate that resolution. A zero-weight NaN must not invalidate an exact valid center, so the implementation excludes zero-weight terms before multiplying.

Implementation: [core/src/lib.rs:83](../core/src/lib.rs#L83), [prep/src/dem.rs:283](../prep/src/dem.rs#L283).

## 6. Multiresolution lookup and bounded memory

Terrain::open reads the small manifest, verifies each file's byte length, opens a buffered decoder, validates tags and sorts the rasters by increasing projected pixel size. The downloader performs the SHA-256 verification; prep intentionally does not rehash 13.75 GB on every invocation. Use --verify-only for an independent repeat verification before an analysis run.

For a query, Terrain::sample projects latitude/longitude once. It attempts 5 m Site01 first, then 10 m 83S, 20 m 80S, 30 m 75S, and 60 m 60S. It returns the first valid bilinear height. A missing fine raster location or nodata falls back to the next source. It does not average unrelated resolutions, blend source seams, or silently treat zero as missing terrain. The source products are separately adjusted surfaces, so small boundary discontinuities remain a limitation.

The projected stereographic scale is $k=1+\rho^2/(4R^2)$. A projected pixel size $p$ corresponds locally to ground spacing $p/k$. The returned TerrainSample includes this corrected ground spacing so the scientific integrator does not mistake map meters for surface arc meters at large distances.

Each raster uses a HashMap from chunk index to decoded float32 samples and a VecDeque of insertion order. In C++, these correspond to unordered_map and deque. On a miss, read_chunk decodes just one strip/tile; if the new payload would exceed 512 MiB, oldest chunks are evicted until it fits. Cache hits do not reorder the queue. This is FIFO, not a falsely labeled least-recently-used cache. Five raster budgets total at most about 2.5 GiB of retained decoded payload, plus decoder buffers and bookkeeping. No entire regional DEM is loaded into RAM.

Raster is generic over Read + Seek. A trait bound states the operations its reader must support, analogous to a constrained C++ template parameter. Production uses BufReader<File>; tests use Cursor<Vec<u8>> as an in-memory binary stream. Ownership keeps each decoder and cache together. Mutable borrowing gives the integrator exclusive access to sampling state while preventing another simultaneous decoder seek.

Implementation: [prep/src/dem.rs:344](../prep/src/dem.rs#L344), [prep/src/dem.rs:390](../prep/src/dem.rs#L390).

## 7. Great-circle rays and exact spherical elevation

A great circle is the intersection of the sphere with a plane through its center. Following a constant initial geographic azimuth means following that great circle, not drawing a straight line in a stereographic image. Let the initial latitude/longitude define unit radial vector $\mathbf r$, geographic north $\mathbf n$ and east $\mathbf e$:

$$
\begin{aligned}
\mathbf r&=(\cos\phi\cos\lambda,\cos\phi\sin\lambda,\sin\phi),\\
\mathbf n&=(-\sin\phi\cos\lambda,-\sin\phi\sin\lambda,\cos\phi),\\
\mathbf e&=(-\sin\lambda,\cos\lambda,0).
\end{aligned}
$$

These are conventional body-fixed Cartesian axes, not the renderer's separate object-axis arrangement. For azimuth $A$ north clockwise and surface arc distance $s$, set $\beta=s/R$ and $\mathbf t=\mathbf n\cos A+\mathbf e\sin A$. The great-circle destination vector is

$$\mathbf q=\mathbf r\cos\beta+\mathbf t\sin\beta.$$

Because $\mathbf r$ and $\mathbf t$ are perpendicular unit vectors, this stays on the unit sphere and in one center-crossing plane. Recover latitude with $\operatorname{atan2}(q_z,\sqrt{q_x^2+q_y^2})$ and longitude with $\operatorname{atan2}(q_y,q_x)$, then apply to_xy. atan2 is stable near the pole where asin would amplify rounding. At the exact pole, geographic north has no unique physical longitude; the supplied longitude defines the limiting north/east tangent basis, consistently with the existing pole tests.

Let $z$ be sampled radial height and $z_0$ the observer's surface height plus instrument height. Relative to the observer's local radial/tangent basis, the terrain displacement has components

$$d_v=(R+z)\cos\beta-(R+z_0),\qquad d_h=(R+z)\sin\beta.$$

The exact elevation is

$$\theta=\operatorname{atan2}(d_v,d_h).$$

At meter-scale distances, subtracting two nearly equal radii loses precision. The implemented algebraically identical expression uses $\cos\beta-1=-2\sin^2(\beta/2)$:

$$d_v=z-z_0-2(R+z)\sin^2(\beta/2).$$

The previous $s^2/(2R)$ approximation is no longer used. For equal observer/surface radii, the exact elevation is $-\beta/2$. For a height $h$ above a smooth sphere, tangent depression is $-\arccos(R/(R+h))$. Tests check both identities, a direct unrearranged formula, a northward meridian at the example longitude, and pole/east directions. The raised-wall test now expects the exact formula: raised terrain changes the horizontal component as well as the vertical one. Legacy geographic-ridge and cyclic interpolation tests remain valid.

Implementation: [core/src/lib.rs:170](../core/src/lib.rs#L170), [core/src/lib.rs:186](../core/src/lib.rs#L186).

## 8. Sampling accuracy, observer height and ray termination

There are 1,440 uniform initial bearings, so bearing spacing is $\Delta A=2\pi/1440$, or 0.25 degrees. The observer is placed 2 m above the bilinearly interpolated local surface by default. This is an explicitly chosen instrument/mast height, not a surveyed property. A final prep argument changes it. The same total radial altitude is passed to Horizons in kilometers; terrain and body directions therefore refer to the same observer.

The first sample is half the observer's local pixel spacing: 2.5 m for a 5 m ground pixel, with the small map-scale correction included. Define $d_0=p_0/2$ and angular target $\alpha=\Delta A/4$. At distance $s$ with local ground pixel spacing $p(s)$, the radial step is

$$\Delta s=\min\left(\frac{p(s)}{2},\max(d_0,s\alpha)\right).$$

The half-pixel cap is intentionally conservative: distance alone never permits skipping a resolved radial feature. The distance term grows gently when moving into coarser sources, while keeping $\Delta s/s\leq\Delta A/4$ once $s\alpha\geq d_0$. Very near the observer the half-pixel floor takes precedence; a relative angular criterion cannot be satisfied at $s=0$, where bearing is undefined. This is a radial-sampling criterion, not a claim that radial motion changes azimuth. Azimuth itself remains the fixed great-circle initial bearing.

Each sample evaluates the exact spherical angle and retains the maximum. Production supplies no distance cutoff: the first query for which every source is unavailable ends that ray. A callback error aborts preparation, rather than turning a corrupt TIFF into a horizon edge. Missing observer coverage is an error. Nonfinite/invalid radii and spacings are rejected, and a floating-point progress guard prevents an infinite loop. The half-circumference safety guard prevents wrapping around the Moon; the selected south-polar coverage ends much earlier. horizon_from_dem keeps an explicit maximum-distance argument only for compatibility with prior in-memory-grid callers.

This is not a claim of an exact continuous physical skyline. Half-pixel sampling and 0.25-degree bearing bins can still miss subpixel or sub-bin features, and a bilinear surface is an approximation to sampled topography. PGDA 78 states that roughly 90 percent of its 5 m pixels are interpolated and reports median RMS height uncertainty around 0.30 to 0.50 m within its regions of interest. A nominal 5 m grid does not mean a 5 m independent measurement everywhere. No uncertainty clones, sub-hour event solver, refraction, scattered sunlight, lunar shape departures from the adopted sphere, or finite-Earth/DSN antenna model is included.

Implementation: [core/src/lib.rs:198](../core/src/lib.rs#L198).

## 9. Visible solar area and Earth clearance

Take the Sun as a uniform circular disk of angular radius $r=0.267$ degrees, with center elevation $e$. Approximate the local skyline by a horizontal chord at $H=\operatorname{Horizon.at}(A_S)$. Normalize its offset as $u=\operatorname{clamp}((H-e)/r,-1,1)$. Integrating the unit circle above the chord gives

$$F=\frac{2}{\pi}\int_u^1\sqrt{1-y^2}\,dy
=\frac{\arccos u-u\sqrt{1-u^2}}{\pi}.$$

This is visible area, not visible diameter. At $u=1$ the fraction is zero; at $u=0$ it is one half; at $u=-1$ it is one. Half a radius above the skyline gives $2/3+\sqrt3/(4\pi)\simeq0.804499$, not 0.75. The core tests check all these cases and the complementary lower segment. Evaluation uses f64 internally and returns f32. The radius remains the existing fixed apparent-radius approximation; varying Sun distance and skyline changes across the disk are not integrated.

For Earth, the user-selected margin $m$ applies to the center: visible means $e_E>H_E+m$. Equality is blocked. The default is 0.5 degrees and one minimal slider permits 0.5 through 1.0 degrees. The core function retains its explicit margin parameter; no scientific constant is hidden in visible. The UI recomputes the cached visibility vector only on a margin edit. This does not claim that the entire Earth or any particular ground station is visible.

Implementation: [core/src/lib.rs:291](../core/src/lib.rs#L291), [core/src/lib.rs:299](../core/src/lib.rs#L299), [app/src/data.rs:108](../app/src/data.rs#L108).

## 10. Preparation, statistics and reproducibility

Run from the workspace root:

```sh
python3 prep/download_dem.py /home/ubuntu/data/lola
python3 prep/download_dem.py /home/ubuntu/data/lola --verify-only
cargo run -p prep --release -- sites.json 2027-01-01 2030-01-01 1h app/assets/data /home/ubuntu/data/lola 2
```

The downloader validates the manifest, refuses a destination inside the repository, checks the total before requests and verifies sizes plus SHA-256. Incomplete streams use a .part file and resume only if HTTP returns the expected Content-Range. A completed file with a wrong digest is an error, not silently overwritten. At most four asset requests run concurrently. TLS and hashing use Python's standard library; the DEM reader itself is pure Rust. No GDAL installation is required.

Prep retains the earlier CLI without a DEM argument for explicitly flat legacy datasets. Reproducing this phase requires the DEM-directory argument. No raw TIFF, partial download, source point cloud or external PDF is committed. The generated JSON includes observer height, interpolated surface height, reference radius, exact asset URLs, byte sizes and digests under terrain, in addition to the existing site, sun, earth and horizon fields. Older app decoders can ignore the provenance field.

The inclusive interval spans 1,096 days: 365 in 2027, 366 in 2028 and 365 in 2029. Each body request produces $1{,}096\cdot24+1=26{,}305$ rows, comfortably below the 90,000-row per-request limit. Prep checks count, start/end timestamps, hourly spacing, and exact Sun/Earth alignment before writing. All times are UTC; APPARENT=AIRLESS avoids introducing atmospheric refraction on the Moon.

lunar_core::stats operates on boolean availability held over $[t_i,t_{i+1})$. Gaps count in full and the last sample has no duration. For its Sun percentage, prep defines availability as $F>0$, meaning any visible solar area. It separately prints the duration-weighted mean of $F$, a more useful geometric power proxy. Neither number includes panel orientation, efficiency, terrain albedo, scattered light or eclipse occultations. The Earth percentage uses the chosen center margin. Hourly samples do not resolve exact rise/set instants.

The completed release run for Example 89.45S 222.69E reported:

| Measurement | Result |
|:--|:--|
| Surface height above reference sphere | 1,954.598173 m |
| Observer height above surface | 2 m |
| Observer radial altitude used for Horizons | 1,956.598173 m |
| Horizon bins and angular separation | 1,440 and 0.25 degrees |
| Minimum horizon | -2.845430 degrees at azimuth 284.5 |
| Maximum horizon | 2.032920 degrees at azimuth 169.5 |
| Horizon tracing elapsed time | 115.078 s |
| Mean visible solar area | 52.002668 percent |
| Any solar area visible | 56.668187 percent |
| Longest held-sample darkness | 551 h |
| Earth visible, margin 0.5 degrees | 37.948601 percent |
| Both available, margin 0.5 degrees | 21.437804 percent |
| Earth visible, margin 1.0 degrees | 35.260797 percent |
| Both available, margin 1.0 degrees | 20.023571 percent |
| Samples per track | 26,305 |
| Generated JSON bytes | 2,718,450 |

The benchmark is a single Linux release run with empty decoded-chunk caches, not a statistical performance study. Files had just been hashed, so operating-system file caches may be warm. Network request time is excluded from the horizon timer. Raw terrain files are on local storage at /home/ubuntu/data/lola. The Sun elevation range is -2.075228 through 2.061621 degrees; values are not clipped to an expected visual range.

Implementation: [prep/src/main.rs:105](../prep/src/main.rs#L105), [core/src/lib.rs:329](../core/src/lib.rs#L329).

## 11. Published comparison and its limits

Mazarico et al. (2011), Illumination conditions of the lunar polar regions using LOLA topography, Icarus 211, 1066 to 1081, DOI [10.1016/j.icarus.2010.10.030](https://doi.org/10.1016/j.icarus.2010.10.030), is available as an [NTRS record](https://ntrs.nasa.gov/citations/20120010094). Its abstract and accessible article text describe 240 m topography and long multi-precession-cycle simulations. The [NASA polar illumination page](https://pgda.gsfc.nasa.gov/products/69) explains the related horizon-profile approach and warns that finite Earth disk visibility is optimistic for radio links.

An accessible published coordinate-specific comparison is Volker Maiwald (2013), Being Selene's Guest: Analysis of the Lunar Environment and Its Impact on Base Location Selection, IAC-13-A5.1.1, [DLR-hosted PDF](https://elib.dlr.de/84844/1/IAC_GHM_final_v1.pdf). Table 3 reproduces a Mazarico-derived average solar illumination of 89.01 percent at longitude 222.69 east, latitude -89.45, exactly the rounded coordinates in sites.json. The table states averaging over four precession cycles. This is a secondary quotation of the older result, not a new validation dataset.

Our result does not reproduce that 89.01 percent: mean solar area is lower by 37.007332 percentage points, and any-area time is lower by 32.341813 percentage points. This is a substantial discrepancy, not evidence of successful validation. Our interval is three specified future years, our observer is 2 m above a much finer adjusted DEM, our boolean Sun statistic means any visible area, and our area statistic uses circular-segment area. The rounded coordinates alone do not reconstruct the earlier paper's exact 240 m cell or height.

A separate diagnostic tried to disprove the idea that the difference comes only from far terrain: a truncated 10 km, 360-bearing control with the same 5 m source already gives 52.642622 percent mean solar area and 57.116788 percent any-area time. Disabling only Site01 in that control gives 64.197032 percent mean area and 69.822080 percent any-area time. Both controls use the same actual Sun track and 2 m observer height; each recomputes surface height from its selected DEM. Therefore nearby terrain and resolution materially affect this result, while the far-terrain cutoff alone is not sufficient to explain the discrepancy.

For a direct numerical cross-check, 50 m due east of the observer the 5 m surface rises by 2.760868 m. After subtracting the 2 m instrument height and exact curvature, its elevation is +0.870019 degrees. The independently selected 10 m surface rises by 2.249820 m relative to its own observer surface, giving +0.285124 degrees. Meter-scale changes matter when the Sun grazes the horizon. These controls demonstrate sensitivity; they do not establish that resolution explains the entire published difference. The earlier full horizon curve and four-cycle experiment remain unreproduced. No coordinates, exposure, observer height or DEM values were tuned to force agreement.

## 12. Reading the Rust and Python implementations

The existing Rust guide introduces ownership, references, Option, Result, traits, closures and ?. Here are the terrain-specific uses, with C++ equivalents:

| Construct | Why it appears | C++ equivalent |
|:--|:--|:--|
| Raster<R: Read + Seek> | Decoder accepts disk or memory streams | Constrained class template |
| Terrain<R = BufReader<File>> | Production reader type has a default | Default template argument |
| impl<R: Read + Seek> | Methods share generic reader constraints | Template member definitions |
| impl FnMut(...) -> Result<...> | Stateful fallible sampling callback | Generic mutable callable returning expected |
| &mut self, &mut terrain | Exclusive access to file cursor/cache | Non-const reference with checked exclusivity |
| Result<Option<TerrainSample>, String> | Error differs from valid no-coverage | expected<optional<Sample>, string> |
| let Some(v) = expression else | Extract or end the ray | Optional test plus early branch |
| let DecodingResult::F32(v) = ... else | Require exact decoded sample type | Checked variant alternative |
| map(...).transpose()? | Propagate an error inside an optional value | Convert optional<expected> to expected<optional> |
| as_chunks::<4>() | Safely view complete four-SHORT entries | Span of fixed-size arrays |
| checked index and get | Reject invalid offsets before dereference | Bounds-checked access returning optional |
| include_str! | Embed only the tiny manifest text | Compile-time small text resource |
| #[cfg(test)], #[path] | Compile separate test fixture modules only in tests | Conditional test translation units |

File, BufReader, Decoder, HashMap and VecDeque own resources and release them when dropped, analogous to C++ RAII. The cache stores decoded vectors by value; no global mutable state or unsafe block is needed. A closure borrows the prep sampler mutably only while core traces the profile. Numeric geometry uses f64, even though source samples and serialized angles are f32.

The Python helper uses with blocks to close streams, analogous to RAII scopes; a ThreadPoolExecutor joins workers and propagates exceptions when results are consumed. A Path represents a filesystem path rather than concatenated command text. hashlib.sha256 maintains a streaming digest. Request supplies the exact byte-range header; urlopen returns a context-managed binary stream. Unit tests replace only HTTP transport with BytesIO, so resume, hashing, protocol errors and final-file publication run without network requests.

## 13. Terrain API index

The alphabetical callable index below links each detected Rust method, qualified function and macro to the exact annotated source quotation. The concept sections above explain the algorithms and C++ contracts; the source comments explain individual statements. Standard arithmetic and control-flow syntax follow the Rust guide.

- `abs`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `acos`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `all`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `and_then`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `and_utc`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `any`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `as_chunks`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `as_str`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `assert!`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `assert_eq!`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Asset`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `assets`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `at`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `atan`: [annotated core/src/terrain_tests.rs](#144-coresrcterrain_testsrs)
- `atan2`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `BufReader::new`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `bytes`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `chain`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `checked_mul`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `chunk_data_dimensions`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `chunk_dimensions`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `clamp`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `collect`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `colortype`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `contains`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `contains_key`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `copied`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `cos`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Cursor::new`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `decoded_strips_centers_interpolation_and_nodata`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `Decoder::new`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `DecodingResult::F32`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `Dem`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `destination`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `dimensions`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `display`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `div_ceil`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `download`: [annotated prep/src/main.rs](#142-prepsrcmainrs)
- `EARTH_MARGIN_DEG`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `encoder`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `enumerate`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `eq_ignore_ascii_case`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `exact_elevation_and_great_circle`: [annotated core/src/terrain_tests.rs](#144-coresrcterrain_testsrs)
- `f64::from`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `File::open`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `file_name`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `filter`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `filter_map`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `find_tag`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `finest_valid_surface_wins_and_nodata_falls_back`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `fixture`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `floor`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `format!`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `fract`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `get`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `get_tag_ascii_string`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `get_tag_f64_vec`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `get_tag_u16_vec`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `grid`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `HashMap::new`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `height`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `height_at`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Horizon`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `horizon_from_dem`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `horizon_from_terrain`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `hypot`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `include_str!`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `insert`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `into`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `into_u16`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `is_ascii_hexdigit`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `is_empty`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `is_err`: [annotated core/src/terrain_tests.rs](#144-coresrcterrain_testsrs)
- `is_finite`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `is_none`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `is_some`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `is_some_and`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `iter`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `iter_mut`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `join`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `key`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `keys`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `last`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `len`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `lines`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `map`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `map_err`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `max`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `metadata`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `min`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `MOON_RADIUS_M`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `NaiveDateTime::parse_from_str`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `new`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `new_image`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `next`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `ok`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `ok_or`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `ok_or_else`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `open`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `or_else`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `panic!`: [annotated core/src/terrain_tests.rs](#144-coresrcterrain_testsrs)
- `parse`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `parse_horizons`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Path::new`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `pgda_projection_matches_labeled_axes`: [annotated core/src/terrain_tests.rs](#144-coresrcterrain_testsrs)
- `pixel`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `pop_front`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `powi`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `println!`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `push`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `push_back`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `Raster::new`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `read_chunk`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `rejects_wrong_sphere_bad_keys_and_corrupt_tiff`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `rem_euclid`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `remove`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `rev`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `rows_per_strip`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `sample`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `serde_json::from_str`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `set_position`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `sin`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `sin_cos`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Site`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `sort_by`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `spherical_elevation`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `split_once`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `split_whitespace`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `sqrt`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Stats`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `stats`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Stats::default`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `sum`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `sun_fraction`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `SUN_RADIUS_DEG`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `tan`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Terrain`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `terrain_sampler_height_edges_and_errors`: [annotated core/src/terrain_tests.rs](#144-coresrcterrain_testsrs)
- `TerrainSample`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `then_some`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `tiff::ColorType::Gray`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `TiffEncoder::new`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `timestamp`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `to_degrees`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `to_radians`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `to_str`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `to_string`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `to_xy`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `total_cmp`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `Track`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Track::default`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `trim`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `trim_end_matches`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `unwrap`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `unwrap_err`: [annotated core/src/terrain_tests.rs](#144-coresrcterrain_testsrs)
- `unwrap_or`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `unwrap_or_else`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `usize::from`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `vec!`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `Vec::new`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `VecDeque::new`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)
- `visible`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `windows`: [annotated core/src/lib.rs](#143-coresrclibrs)
- `write_data`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `write_tag`: [annotated prep/src/dem_tests.rs](#145-prepsrcdem_testsrs)
- `zip`: [annotated prep/src/dem.rs](#141-prepsrcdemrs)

## 14. Exact terrain source quotations

These are mechanically refreshed complete files, not shortened illustrative substitutes. Tabs are preserved. The app's complete updated source remains quoted in the panorama guide.

### 14.1. prep/src/dem.rs

```rust
//! Native GeoTIFF decoding and a bounded multiresolution tile cache; core remains I/O free.

// Import only pure geometry and sample values; C++ includes the scientific header.
use lunar_core::{MOON_RADIUS_M, TerrainSample, to_xy};
// Deserialize the reproducible asset manifest; C++ JSON-to-struct adapter.
use serde::Deserialize;
// Keep filesystem, buffering and cache ownership here, not in core; C++ standard library headers.
use std::{
	collections::{HashMap, VecDeque},
	fs::File,
	io::{BufReader, Read, Seek},
	path::Path,
};
// Decode TIFF containers, not a system GDAL library; C++ a native TIFF decoder API.
use tiff::{
	decoder::{Decoder, DecodingResult},
	tags::Tag,
};

// A published DEM asset and its reproducibility digest; C++ aggregate descriptor.
#[derive(Deserialize, serde::Serialize)]
pub struct Asset
{
	// Local basename; C++ std::string.
	pub file: String,
	// Source URL remains in serialized provenance; C++ std::string.
	pub url: String,
	// Expected byte length from HTTP headers; C++ uint64_t.
	pub bytes: u64,
	// SHA-256 digest recorded after the initial download; C++ std::string.
	pub sha256: String,
}

// Read the versioned manifest at compile time, not the DEMs; C++ embedded small text resource.
pub fn assets() -> Result<Vec<Asset>, String>
{
	// Fail on a malformed developer manifest; C++ expected-returning JSON parse.
	let assets: Vec<Asset> =
		serde_json::from_str(include_str!("../dem-manifest.json")).map_err(|e| e.to_string())?;
	// Validate trusted basenames and recorded digests before opening files; C++ all_of over descriptors.
	if assets.is_empty()
		|| assets.iter().any(|a| {
			Path::new(&a.file).file_name().and_then(|p| p.to_str()) != Some(a.file.as_str())
				|| a.bytes == 0
				|| a.sha256.len() != 64
				|| !a.sha256.bytes().all(|c| c.is_ascii_hexdigit())
		})
	{
		return Err("Invalid DEM asset manifest".into());
	}
	// Return owned validated descriptors; C++ vector move.
	Ok(assets)
}

// GeoTIFF's supported affine transform, expressed at sample centers; C++ plain struct.
#[derive(Debug)]
struct Grid
{
	// Raster dimensions count samples, not map meters; C++ uint32_t members.
	w: u32,
	h: u32,
	// Square projected pixel spacing; C++ double.
	res: f64,
	// First sample's map coordinates; C++ double members.
	x: f64,
	y: f64,
}

// Parse numeric GeoKeys without outsourcing projection interpretation; C++ free function.
fn key(keys: &[u16], doubles: &[f64], id: u16) -> Result<f64, String>
{
	// Require the GeoTIFF 1.x key directory header and complete four-word entries; C++ span bounds checks.
	if keys.len() < 4 || keys[0] != 1 || keys.len() != 4 + usize::from(keys[3]) * 4
	{
		return Err("Malformed GeoKeyDirectory".into());
	}
	// Find exactly one definition, rejecting ambiguous duplicate keys; C++ filtered iterator.
	let mut matches = keys[4..].as_chunks::<4>().0.iter().filter(|k| k[0] == id);
	// A required key may not be silently assumed; C++ optional extraction.
	let k = matches
		.next()
		.ok_or_else(|| format!("Missing GeoKey {id}"))?;
	// Scalar numeric keys use either inline SHORT or a DOUBLE-table offset; C++ checked tagged representation.
	if matches.next().is_some() || k[2] != 1
	{
		return Err(format!("Invalid GeoKey {id}"));
	}
	// Resolve the TIFF-defined storage location, never a guessed ordering; C++ switch.
	match k[1]
	{
		0 => Ok(f64::from(k[3])),
		34736 => doubles
			.get(usize::from(k[3]))
			.copied()
			.ok_or_else(|| format!("GeoKey {id} offset out of bounds")),
		_ => Err(format!("Unsupported numeric GeoKey storage for {id}")),
	}
}

// Decode the supported unrotated, pixel-is-area affine transform; C++ constructor helper.
fn grid(
	w: u32,
	h: u32,
	scale: &[f64],
	tie: &[f64],
	keys: &[u16],
	doubles: &[f64],
) -> Result<Grid, String>
{
	// Require the precise projection encoded in both PGDA products; C++ loop over expected key/value pairs.
	for (id, expected) in [
		(1024, 1.0),
		(1025, 1.0),
		(2054, 9102.0),
		(2057, MOON_RADIUS_M),
		(2058, MOON_RADIUS_M),
		(2061, 0.0),
		(3075, 15.0),
		(3076, 9001.0),
		(3081, -90.0),
		(3082, 0.0),
		(3083, 0.0),
		(3092, 1.0),
		(3095, 0.0),
	]
	{
		// Do not accept Earth ellipsoids or a different polar convention; C++ tolerance comparison.
		let actual = key(keys, doubles, id)?;
		// NaN must not bypass a tolerance comparison; C++ std::isfinite plus absolute difference.
		if !actual.is_finite() || (actual - expected).abs() > 1e-9
		{
			return Err(format!("Unsupported projection GeoKey {id}"));
		}
	}
	// Validate dimensions, scalar scale, and the one supported tie point; C++ geometric preconditions.
	if w < 2
		|| h < 2
		|| scale.len() != 3
		|| tie.len() != 6
		|| scale.iter().chain(tie).any(|v| !v.is_finite())
		|| scale[0] <= 0.0
		|| scale[0] != scale[1]
		|| scale[2] != 0.0
		|| tie[2] != 0.0
		|| tie[5] != 0.0
	{
		return Err("Unsupported GeoTIFF raster transform".into());
	}
	// RasterPixelIsArea tie points refer to corners; shift half a pixel to the first center; C++ affine transform.
	Ok(Grid {
		w,
		h,
		res: scale[0],
		x: tie[3] + (0.5 - tie[0]) * scale[0],
		y: tie[4] - (0.5 - tie[1]) * scale[1],
	})
}

// A TIFF decoder plus bounded FIFO cache; generic Read+Seek permits in-memory test files; C++ template class.
struct Raster<R: Read + Seek>
{
	// Own the decoder and its underlying seekable file; C++ RAII decoder.
	decoder: Decoder<R>,
	// Validated spatial interpretation; C++ value member.
	grid: Grid,
	// Native strip or tile dimensions; C++ pair<uint32_t,uint32_t>.
	chunk: (u32, u32),
	// Decoded chunks keyed by TIFF chunk index; C++ unordered_map.
	cache: HashMap<u32, Vec<f32>>,
	// FIFO ordering changes only on cache insertion, keeping hits cheap; C++ deque.
	order: VecDeque<u32>,
	// Account actual decoded bytes rather than guessing compression ratios; C++ size_t.
	bytes: usize,
}

// Define decoder operations over either disk files or test cursors; C++ template member definitions.
impl<R: Read + Seek> Raster<R>
{
	// Open only the first/full-resolution IFD; COG overviews must not replace scientific samples; C++ factory.
	fn new(reader: R) -> Result<Self, String>
	{
		// Decode TIFF/BigTIFF directory and compression information; C++ checked constructor.
		let mut decoder = Decoder::new(reader).map_err(|e| e.to_string())?;
		// PGDA DEMs are scalar IEEE float32, not encoded normals or integer radii; C++ type check.
		if decoder.colortype().map_err(|e| e.to_string())? != tiff::ColorType::Gray(32)
			|| decoder
				.get_tag_u16_vec(Tag::SampleFormat)
				.map_err(|e| e.to_string())?
				!= [3]
		{
			return Err("Expected a single float32 DEM channel".into());
		}
		// Reject rotated affine transforms rather than ignoring them; C++ optional metadata guard.
		if decoder
			.find_tag(Tag::ModelTransformationTag)
			.map_err(|e| e.to_string())?
			.is_some()
		{
			return Err("ModelTransformationTag is not supported".into());
		}
		// Top-left row orientation is required; absence means TIFF's default top-left; C++ optional default.
		if decoder
			.find_tag(Tag::Orientation)
			.map_err(|e| e.to_string())?
			.is_some_and(|v| v.into_u16().ok() != Some(1))
		{
			return Err("Unsupported raster orientation".into());
		}
		// NaN is the published nodata representation; fail on an unimplemented numeric sentinel; C++ metadata validation.
		let nodata = decoder
			.get_tag_ascii_string(Tag::GdalNodata)
			.map_err(|e| e.to_string())?;
		if !nodata.trim_end_matches('\0').eq_ignore_ascii_case("nan")
		{
			return Err("Expected NaN nodata".into());
		}
		// Read named TIFF tags instead of hardcoded raster dimensions; C++ typed tag accessors.
		let (w, h) = decoder.dimensions().map_err(|e| e.to_string())?;
		let scale = decoder
			.get_tag_f64_vec(Tag::ModelPixelScaleTag)
			.map_err(|e| e.to_string())?;
		let tie = decoder
			.get_tag_f64_vec(Tag::ModelTiepointTag)
			.map_err(|e| e.to_string())?;
		let keys = decoder
			.get_tag_u16_vec(Tag::GeoKeyDirectoryTag)
			.map_err(|e| e.to_string())?;
		let doubles = decoder
			.get_tag_f64_vec(Tag::GeoDoubleParamsTag)
			.map_err(|e| e.to_string())?;
		// Validate projection and pixel semantics before allowing any samples; C++ checked factory.
		let grid = grid(w, h, &scale, &tie, &keys, &doubles)?;
		// TIFF abstracts both strips and tiles as chunks; C++ pair of dimensions.
		let chunk = decoder.chunk_dimensions();
		// Start with no decoded elevation payload; C++ aggregate initialization.
		Ok(Self {
			decoder,
			grid,
			chunk,
			cache: HashMap::new(),
			order: VecDeque::new(),
			bytes: 0,
		})
	}

	// Fetch one scalar, decoding a chunk only on a cache miss; C++ checked member function.
	fn pixel(&mut self, c: u32, r: u32) -> Result<f32, String>
	{
		// Raster bounds were checked by bilinear sampling; convert row/column into a chunk index; C++ integer arithmetic.
		let index = (r / self.chunk.1) * self.grid.w.div_ceil(self.chunk.0) + c / self.chunk.0;
		// Edge tiles have cropped output dimensions, unlike padded on-disk storage; C++ get decoded stride.
		let (width, _) = self.decoder.chunk_data_dimensions(index);
		// Avoid repeatedly decoding a neighboring tile during interpolation; C++ unordered_map membership.
		if !self.cache.contains_key(&index)
		{
			// Compression errors are propagated, not reclassified as missing terrain; C++ expected extraction.
			let DecodingResult::F32(values) =
				self.decoder.read_chunk(index).map_err(|e| e.to_string())?
			else
			{
				return Err("Decoder did not return float32 DEM samples".into());
			};
			// Bound each raster cache to 512 MiB; C++ FIFO eviction loop.
			while self.bytes + values.len() * 4 > 512 * 1024 * 1024 && !self.order.is_empty()
			{
				// Every queued key belongs to the cache; C++ pop_front and erase.
				let old = self.order.pop_front().unwrap();
				self.bytes -= self.cache.remove(&old).unwrap().len() * 4;
			}
			// Account and retain this decoded chunk; C++ move into an unordered_map.
			self.bytes += values.len() * 4;
			self.cache.insert(index, values);
			self.order.push_back(index);
		}
		// Convert the in-chunk coordinate using its actual decoded width; C++ vector::at.
		self.cache[&index]
			.get(((r % self.chunk.1) * width + c % self.chunk.0) as usize)
			.copied()
			.ok_or_else(|| "Invalid decoded chunk dimensions".into())
	}

	// Bilinearly interpolate four valid sample centers; C++ optional-returning sampler.
	fn height(&mut self, x: f64, y: f64) -> Result<Option<f64>, String>
	{
		// Invert the unrotated center-coordinate affine transform; C++ two doubles.
		let c = (x - self.grid.x) / self.grid.res;
		let r = (self.grid.y - y) / self.grid.res;
		// Never extrapolate a border sample beyond its center; C++ range guards.
		if !c.is_finite()
			|| !r.is_finite()
			|| c < 0.0
			|| r < 0.0
			|| c > f64::from(self.grid.w - 1)
			|| r > f64::from(self.grid.h - 1)
		{
			return Ok(None);
		}
		// Choose the surrounding sample indices, clamping only the last center; C++ integer indices.
		let (c0, r0) = (c.floor() as u32, r.floor() as u32);
		let (c1, r1) = ((c0 + 1).min(self.grid.w - 1), (r0 + 1).min(self.grid.h - 1));
		// Decode all four neighbors before evaluating the interpolation; C++ small fixed array.
		let v = [
			self.pixel(c0, r0)?,
			self.pixel(c1, r0)?,
			self.pixel(c0, r1)?,
			self.pixel(c1, r1)?,
		];
		// Zero-weight corners must not turn a valid center into nodata; C++ array of bilinear weights.
		let weights = [
			(1.0 - c.fract()) * (1.0 - r.fract()),
			c.fract() * (1.0 - r.fract()),
			(1.0 - c.fract()) * r.fract(),
			c.fract() * r.fract(),
		];
		// A contributing nodata neighbor makes this resolution unavailable; C++ any_of zipped values.
		if v.iter()
			.zip(weights)
			.any(|(v, w)| w > 0.0 && !v.is_finite())
		{
			return Ok(None);
		}
		// Evaluate only contributing weights in double precision; C++ filtered inner product.
		Ok(Some(
			v.iter()
				.zip(weights)
				.filter(|(_, w)| *w > 0.0)
				.map(|(&v, w)| f64::from(v) * w)
				.sum(),
		))
	}
}

// Own only the selected published rasters, ordered finest first; C++ vector of RAII decoders.
pub struct Terrain<R: Read + Seek = BufReader<File>>
{
	// Each file maintains a separate bounded cache; C++ vector<Raster<ifstream>>.
	rasters: Vec<Raster<R>>,
}

// Define the preparation-only multiresolution lookup; C++ class member definitions.
impl Terrain
{
	// Require the manifest's exact file inventory and sizes; C++ filesystem validation and factories.
	pub fn open(directory: &Path) -> Result<Self, String>
	{
		// Empty inventory must not become a silently flat Moon; C++ vector construction.
		let mut rasters = Vec::new();
		// Visit all finest nonredundant resolution levels selected for this site set; C++ range-for.
		for asset in assets()?
		{
			// Resolve only a trusted manifest basename under the supplied directory; C++ path concatenation.
			let path = directory.join(&asset.file);
			// Reject partial downloads before touching TIFF offsets; C++ RAII file and size checks.
			let file = File::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
			if file.metadata().map_err(|e| e.to_string())?.len() != asset.bytes
			{
				return Err(format!("Wrong DEM byte length: {}", path.display()));
			}
			// Buffer small tag reads and seek over compressed tile payloads; C++ buffered ifstream.
			let raster = Raster::new(BufReader::new(file))
				.map_err(|e| format!("{}: {e}", path.display()))?;
			// Record the exact on-disk bounds, independent of filename latitude limits; C++ formatted diagnostic.
			println!(
				"DEM {}: {}x{}, {} m/pixel, center bounds x=[{},{}] y=[{},{}], source={}, sha256={}",
				asset.file,
				raster.grid.w,
				raster.grid.h,
				raster.grid.res,
				raster.grid.x,
				raster.grid.x + f64::from(raster.grid.w - 1) * raster.grid.res,
				raster.grid.y - f64::from(raster.grid.h - 1) * raster.grid.res,
				raster.grid.y,
				asset.url,
				asset.sha256
			);
			// Move decoder ownership into the hierarchy; C++ vector::push_back(move).
			rasters.push(raster);
		}
		// Prefer resolution, not filename lexicographic ordering; C++ stable_sort with comparator.
		rasters.sort_by(|a, b| a.grid.res.total_cmp(&b.grid.res));
		// Construct the scientific sampler with no GPU/UI coupling; C++ aggregate return.
		Ok(Self { rasters })
	}
}

// Sampling does not depend on the reader's concrete type; C++ template member definitions.
impl<R: Read + Seek> Terrain<R>
{
	// Query a spherical location using highest-resolution valid coverage; C++ expected<optional<Sample>>.
	pub fn sample(&mut self, lat: f64, lon: f64) -> Result<Option<TerrainSample>, String>
	{
		// All selected files share this validated projection; C++ one coordinate conversion.
		let (x, y) = to_xy(lat, lon);
		// Convert projected spacing to ground spacing using stereographic scale; C++ double.
		let scale = 1.0 + (x * x + y * y) / (4.0 * MOON_RADIUS_M * MOON_RADIUS_M);
		// Missing high-resolution coverage or nodata falls back, but decoding errors propagate; C++ ordered search.
		for raster in &mut self.rasters
		{
			// Bilinear interpolation is never performed across two incompatible grids; C++ checked optional.
			if let Some(z_m) = raster.height(x, y)?
			{
				return Ok(Some(TerrainSample {
					z_m,
					resolution_m: raster.grid.res / scale,
				}));
			}
		}
		// None means the continuous available terrain has ended; C++ nullopt.
		Ok(None)
	}
}

// Compile fixtures only into the preparation test binary; C++ conditional test module.
#[cfg(test)]
#[path = "dem_tests.rs"]
mod tests;
```

### 14.2. prep/src/main.rs

```rust
//! Offline Horizons and PGDA terrain preparation for fixed landing sites.

// Keep native TIFF decoding out of core and the web app; C++ implementation translation unit.
mod dem;
// Parse date-only UTC endpoints; C++ chrono::year_month_day.
use chrono::NaiveDate;
// Reuse geometry and statistics, not duplicated formulas; C++ scientific header.
use lunar_core::{
	EARTH_MARGIN_DEG, Horizon, Site, Track, horizon_from_terrain, parse_horizons, stats,
	sun_fraction, visible,
};
// Generate JSON encoders; C++ struct-to-JSON adapters.
use serde::Serialize;
// Native I/O and elapsed clocks belong in preparation; C++ standard headers.
use std::{
	error::Error,
	fs,
	path::Path,
	time::{Duration, Instant},
};

// Serialize provenance beside terrain bins; C++ JSON-adapted aggregate.
#[derive(Serialize)]
struct TerrainInfo
{
	// Instrument height above the DEM surface; C++ double.
	observer_height_m: f64,
	// Radial surface height above the reference sphere; C++ double.
	surface_height_m: f64,
	// Radius validated against the GeoKeys; C++ double.
	reference_radius_m: f64,
	// Exact files, URLs, sizes and digests; C++ vector<Asset>.
	assets: Vec<dem::Asset>,
}

// Keep the legacy track/profile schema with additional provenance; C++ JSON aggregate.
#[derive(Serialize)]
struct SiteData
{
	// Geographic site definition; C++ Site value.
	site: Site,
	// Aligned directions for each body; C++ Track members.
	sun: Track,
	earth: Track,
	// None explicitly means legacy flat preparation; C++ optional<Horizon>.
	horizon: Option<Horizon>,
	// Older app versions may ignore this new field; C++ optional<TerrainInfo>.
	terrain: Option<TerrainInfo>,
}

// Query one body at the instrument's radial altitude; C++ expected-returning HTTP helper.
fn download(
	client: &reqwest::blocking::Client,
	site: &Site,
	observer_z_m: f64,
	body: &str,
	start: &str,
	stop: &str,
	step: &str,
) -> Result<Track, Box<dyn Error>>
{
	// Horizons altitude is kilometers, unlike DEM meters; C++ formatted coordinate string.
	let coords = format!(
		"'{},{},{}'",
		site.lon_deg,
		site.lat_deg,
		observer_z_m / 1000.0
	);
	// Request airless apparent azimuth/elevation at a fixed lunar site; C++ HTTP builder.
	let text = client
		.get("https://ssd.jpl.nasa.gov/api/horizons.api")
		.query(&[
			("format", "text"),
			("COMMAND", body),
			("CENTER", "'coord@301'"),
			("COORD_TYPE", "'GEODETIC'"),
			("SITE_COORD", &coords),
			("MAKE_EPHEM", "'YES'"),
			("EPHEM_TYPE", "'OBSERVER'"),
			("QUANTITIES", "'4'"),
			("ANG_FORMAT", "'DEG'"),
			("APPARENT", "'AIRLESS'"),
			("START_TIME", &format!("'{start}'")),
			("STOP_TIME", &format!("'{stop}'")),
			("STEP_SIZE", &format!("'{step}'")),
		])
		.send()?
		.error_for_status()?
		.text()?;
	// Retain reference radii and frame diagnostics in the preparation log; C++ filtered stream output.
	for line in text.lines().filter(|line| {
		line.contains("Center radii")
			|| line.contains("Center-site")
			|| line.contains("Center pole")
	})
	{
		println!("Horizons {body}: {line}");
	}
	// Parsing errors include the actual remote response; C++ expected error mapping.
	parse_horizons(&text)
		.map_err(|e| format!("Horizons for {} ({body}): {e}\n{text}", site.name).into())
}

// Native command entry point; C++ main with an error-returning implementation.
fn main() -> Result<(), Box<dyn Error>>
{
	// Own arguments after the executable name; C++ vector<string> from argv.
	let args: Vec<_> = std::env::args().skip(1).collect();
	// Append DEM directory and height without breaking the prior CLI; C++ argument-count guard.
	if !(4..=7).contains(&args.len())
	{
		return Err("Usage: prep <sites.json> <start YYYY-MM-DD> <stop YYYY-MM-DD> <step, e.g. 1h> [output-dir] [dem-dir] [observer-height-m, default 2]".into());
	}
	// Parse inclusive UTC dates before making requests; C++ checked calendar parsing.
	let start = NaiveDate::parse_from_str(&args[1], "%Y-%m-%d")?;
	let stop = NaiveDate::parse_from_str(&args[2], "%Y-%m-%d")?;
	// Empty/reversed intervals are errors; C++ early return.
	if stop <= start
	{
		return Err("Stop date must be after start date".into());
	}
	// Validate the supported Horizons cadence syntax; C++ suffix check and integer parsing.
	let step = &args[3];
	if !step.ends_with(['m', 'h', 'd']) || step[..step.len() - 1].parse::<u32>().unwrap_or(0) == 0
	{
		return Err("Step must be a positive integer followed by m, h, or d".into());
	}
	// Convert cadence to elapsed seconds; C++ switch expression.
	let seconds = i64::from(step[..step.len() - 1].parse::<u32>()?)
		* match step.as_bytes()[step.len() - 1]
		{
			b'm' => 60,
			b'h' => 3600,
			_ => 86400,
		};
	// Count inclusive endpoints against the per-query service limit; C++ integer arithmetic.
	let expected = ((stop - start).num_seconds() / seconds + 1) as usize;
	if expected > 90000
	{
		return Err(format!("{expected} rows exceeds Horizons' 90,000-row limit").into());
	}
	// Instrument height is a nonnegative physical input; C++ optional argument parsing.
	let height = args.get(6).map_or(Ok(2.0), |s| s.parse::<f64>())?;
	if !height.is_finite() || height < 0.0
	{
		return Err("Observer height must be finite and nonnegative".into());
	}
	// Decode fixed sites before opening large rasters; C++ JSON parser.
	let sites: Vec<Site> = serde_json::from_str(&fs::read_to_string(&args[0])?)?;
	if sites.is_empty()
	{
		return Err("At least one fixed site is required".into());
	}
	// Reject filename collisions as well as invalid coordinates; C++ unordered_set.
	let mut slugs = std::collections::HashSet::new();
	for site in &sites
	{
		if !site.lat_deg.is_finite()
			|| !(-90.0..=90.0).contains(&site.lat_deg)
			|| !site.lon_deg.is_finite()
			|| slug(&site.name).is_empty()
			|| !slugs.insert(slug(&site.name))
		{
			return Err(format!(
				"Invalid coordinates or duplicate/empty slug for {}",
				site.name
			)
			.into());
		}
	}
	// Reuse caches across sites; transpose converts optional failure into operation failure; C++ optional/expected conversion.
	let mut terrain = args
		.get(5)
		.map(|p| dem::Terrain::open(Path::new(p)))
		.transpose()?;
	// Explicitly identify legacy runs without a DEM; C++ formatted diagnostic.
	println!(
		"Preparation: {expected} rows/body, observer height {height} m, terrain={}",
		terrain.is_some()
	);
	// Preserve the previous output default; C++ path from optional argument.
	let output = Path::new(args.get(4).map(String::as_str).unwrap_or("app/assets/data"));
	// Bound remote request time; C++ HTTP client timeout.
	let client = reqwest::blocking::Client::builder()
		.timeout(Duration::from_secs(180))
		.build()?;
	// Create only the selected output directory; C++ filesystem::create_directories.
	fs::create_dir_all(output)?;
	// Process fixed sites rather than interactive map positions; C++ range-for over owned values.
	for site in sites
	{
		// Legacy no-DEM preparation uses the reference sphere; C++ initialized local.
		let mut surface_height = 0.0;
		// Optional mutable borrowing gives the sampler exclusive cache access; C++ optional reference.
		let horizon = if let Some(dem) = &mut terrain
		{
			// Missing observer data must not silently become flat terrain; C++ checked optional extraction.
			surface_height = dem
				.sample(site.lat_deg, site.lon_deg)?
				.ok_or("Site outside DEM coverage")?
				.z_m;
			// Include tile decoding and cold-cache work in wall time; C++ steady_clock.
			let timer = Instant::now();
			// Trace 1,440 bearings to available coverage edges, with no fixed production distance; C++ mutable lambda.
			let h = horizon_from_terrain(
				site.lat_deg,
				site.lon_deg,
				1440,
				height,
				None,
				|lat, lon| dem.sample(lat, lon),
			)?;
			// Report exact extrema and a one-site elapsed benchmark; C++ reductions and stream output.
			println!(
				"Horizon {}: {} azimuths, surface {:.6} m, min {:.6} deg, max {:.6} deg, benchmark {:.3} s",
				site.name,
				h.h_deg.len(),
				surface_height,
				h.h_deg.iter().copied().fold(f32::INFINITY, f32::min),
				h.h_deg.iter().copied().fold(f32::NEG_INFINITY, f32::max),
				timer.elapsed().as_secs_f64()
			);
			Some(h)
		}
		else
		{
			None
		};
		// Query both bodies at the actual surface plus instrument altitude; C++ common scalar parameter.
		let sun = download(
			&client,
			&site,
			surface_height + height,
			"'10'",
			&args[1],
			&args[2],
			step,
		)?;
		let earth = download(
			&client,
			&site,
			surface_height + height,
			"'399'",
			&args[1],
			&args[2],
			step,
		)?;
		// A UTC date endpoint is midnight in the service's sample grid; C++ chrono conversion.
		let first = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
		// Verify count, endpoints, cadence and exact inter-body UTC alignment; C++ vector predicates.
		if sun.t != earth.t
			|| sun.t.len() != expected
			|| sun.t[0] != first
			|| *sun.t.last().unwrap() != first + (expected as i64 - 1) * seconds
			|| sun.t.windows(2).any(|t| t[1] - t[0] != seconds)
		{
			return Err("Unexpected Horizons row counts or mismatched timestamps".into());
		}
		// Only the legacy no-DEM path has a flat fallback; C++ local profile.
		let flat = Horizon { h_deg: vec![0.0] };
		let h = horizon.as_ref().unwrap_or(&flat);
		// Evaluate visible solar area at every sample; C++ zipped transform.
		let fractions: Vec<_> = sun
			.el_deg
			.iter()
			.zip(&sun.az_deg)
			.map(|(&el, &az)| sun_fraction(el, h.at(az)))
			.collect();
		// stats requires booleans: report any visible area, not average power; C++ predicate transform.
		let any_sun: Vec<_> = fractions.iter().map(|&f| f > 0.0).collect();
		// Report both allowed communication-margin endpoints; C++ range-for.
		for margin in [EARTH_MARGIN_DEG, 1.0]
		{
			println!(
				"Stats (Sun area > 0, Earth margin {margin} deg): {}",
				serde_json::to_string(&stats(&sun.t, &any_sun, &visible(&earth, h, margin)))?
			);
		}
		// Match stats' left-held weighting and durationless final sample; C++ weighted sum.
		let mean_area: f64 = if sun.t.len() < 2
		{
			0.0
		}
		else
		{
			sun.t
				.windows(2)
				.zip(&fractions)
				.map(|(t, &f)| (t[1] - t[0]) as f64 * f64::from(f))
				.sum::<f64>()
				/ (sun.t.last().unwrap() - sun.t[0]) as f64
		};
		println!("Mean visible solar area: {:.6}%", mean_area * 100.0);
		// Add provenance only for real terrain; C++ optional factory with error propagation.
		let provenance = horizon
			.as_ref()
			.map(|_| {
				Ok::<_, String>(TerrainInfo {
					observer_height_m: height,
					surface_height_m: surface_height,
					reference_radius_m: lunar_core::MOON_RADIUS_M,
					assets: dem::assets()?,
				})
			})
			.transpose()?;
		// Derive the existing stable ASCII basename; C++ path concatenation.
		let path = output.join(format!("{}.json", slug(&site.name)));
		// Transfer owned arrays into the serialized record; C++ aggregate moves.
		let data = SiteData {
			site,
			sun,
			earth,
			horizon,
			terrain: provenance,
		};
		// Pretty JSON is shared unchanged by native and web; C++ encoded file write.
		fs::write(&path, serde_json::to_string_pretty(&data)?)?;
		println!(
			"Wrote {} ({} samples per track, {} bytes)",
			path.display(),
			data.sun.t.len(),
			fs::metadata(&path)?.len()
		);
	}
	// Signal success to the shell; C++ return success.
	Ok(())
}

// Convert a label to the existing filename convention; C++ string utility.
fn slug(name: &str) -> String
{
	// Split punctuation runs, lowercase nonempty ASCII words, then join; C++ ranges pipeline.
	name.split(|c: char| !c.is_ascii_alphanumeric())
		.filter(|s| !s.is_empty())
		.map(str::to_ascii_lowercase)
		.collect::<Vec<_>>()
		.join("-")
}
```

### 14.3. core/src/lib.rs

```rust
// Parse UTC table timestamps without accessing a clock; C++ chrono calendar parser.
use chrono::NaiveDateTime;
// Generate portable JSON adapters without performing I/O here; C++ serialization traits.
use serde::{Deserialize, Serialize};

// PGDA's spherical reference radius in meters; C++ constexpr double.
pub const MOON_RADIUS_M: f64 = 1_737_400.0;
// Fixed apparent solar radius used for the inexpensive disk-area approximation; C++ constexpr float.
pub const SUN_RADIUS_DEG: f32 = 0.267;
// Default angular clearance for direct-to-Earth links; C++ constexpr float.
pub const EARTH_MARGIN_DEG: f32 = 0.5;

// Generate diagnostic formatting and JSON conversion; C++ generated adapters.
#[derive(Debug, Serialize, Deserialize)]
pub struct Site
{
	// Human-readable fixed-site label; C++ std::string.
	pub name: String,
	// Planetocentric latitude, north positive; C++ double.
	pub lat_deg: f64,
	// East-positive longitude; C++ double.
	pub lon_deg: f64,
}

/// UTC Unix seconds and angles in degrees. Azimuth is north, clockwise to east.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Track
{
	// UTC Unix seconds, monotonically increasing; C++ vector<int64_t>.
	pub t: Vec<i64>,
	// Geographic azimuth north clockwise; C++ vector<float>.
	pub az_deg: Vec<f32>,
	// Local tangent-plane elevation; C++ vector<float>.
	pub el_deg: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Horizon
{
	// Uniform cyclic bearing bins, beginning at north; C++ vector<float>.
	pub h_deg: Vec<f32>,
}

impl Horizon
{
	/// An empty profile represents an unobstructed horizon below all elevations.
	pub fn at(&self, az_deg: f32) -> f32
	{
		// Preserve the unobstructed sentinel for legacy scientific callers; C++ vector::empty.
		if self.h_deg.is_empty()
		{
			// No surface reaches this elevation; C++ return a float literal.
			return -90.0;
		}
		// Euclidean remainder wraps negative and greater-than-360 bearings; C++ positive modulo.
		let p = az_deg.rem_euclid(360.0) / 360.0 * self.h_deg.len() as f32;
		// Locate the lower bin, protecting the rounded 360 endpoint; C++ size_t cast and modulo.
		let i = p.floor() as usize % self.h_deg.len();
		// Read neighboring cyclic elevations; C++ indexed vector values.
		let a = self.h_deg[i];
		let b = self.h_deg[(i + 1) % self.h_deg.len()];
		// Piecewise-linear interpolation includes the last-to-first wrap; C++ affine blend.
		a + (b - a) * p.fract()
	}
}

pub struct Dem
{
	// Number of sample columns and rows; C++ size_t members.
	pub w: usize,
	pub h: usize,
	// Projected spacing in meters; C++ double.
	pub res_m: f64,
	// First sample center, not the corner of a GeoTIFF pixel; C++ double members.
	pub x_min_m: f64,
	pub y_max_m: f64,
	// Row-major radial heights above the reference sphere; C++ vector<float>.
	pub z_m: Vec<f32>,
}

/// PGDA south polar stereographic: spherical radius R, true scale at -90,
/// longitude origin zero, no false offsets; +x toward 90E, +y toward 0E.
pub fn to_xy(lat_deg: f64, lon_deg: f64) -> (f64, f64)
{
	// South-pole true scale gives rho=2R tan(pi/4+latitude/2); C++ std::tan.
	let rho =
		2.0 * MOON_RADIUS_M * (std::f64::consts::FRAC_PI_4 + lat_deg.to_radians() / 2.0).tan();
	// Trigonometric functions require radians; C++ degrees*pi/180.
	let lon = lon_deg.to_radians();
	// Preserve PGDA's easting and northing axis signs; C++ pair<double,double>.
	(rho * lon.sin(), rho * lon.cos())
}

impl Dem
{
	/// Grid coordinates refer to sample centers, with rows running toward decreasing y.
	pub fn height_at(&self, x: f64, y: f64) -> Option<f32>
	{
		// Guard malformed dimensions, invalid spacing, and row-major length overflow; C++ checked preconditions.
		if self.w == 0
			|| self.h == 0
			|| !self.res_m.is_finite()
			|| self.res_m <= 0.0
			|| self.w.checked_mul(self.h)? != self.z_m.len()
		{
			// Invalid memory grids have no sample; C++ nullopt.
			return None;
		}
		// Invert the center-coordinate affine transform; C++ double arithmetic.
		let col = (x - self.x_min_m) / self.res_m;
		let row = (self.y_max_m - y) / self.res_m;
		// Do not extrapolate or cast invalid coordinates into indices; C++ finite range checks.
		if !col.is_finite()
			|| !row.is_finite()
			|| col < 0.0
			|| row < 0.0
			|| col > (self.w - 1) as f64
			|| row > (self.h - 1) as f64
		{
			// The point lies outside valid center coverage; C++ nullopt.
			return None;
		}
		// Lower integer sample indices; C++ floor and size_t conversion.
		let c = col.floor() as usize;
		let r = row.floor() as usize;
		// Clamp the neighbor at the last exact center; C++ std::min.
		let c1 = (c + 1).min(self.w - 1);
		let r1 = (r + 1).min(self.h - 1);
		// Fractional positions become bilinear interpolation weights; C++ fractional parts.
		let fx = col.fract() as f32;
		let fy = row.fract() as f32;
		// Interpolate each row and then between rows; C++ weighted sums.
		let top = self.z_m[r * self.w + c] * (1.0 - fx) + self.z_m[r * self.w + c1] * fx;
		let bottom = self.z_m[r1 * self.w + c] * (1.0 - fx) + self.z_m[r1 * self.w + c1] * fx;
		let z = top * (1.0 - fy) + bottom * fy;
		// NaN elevations propagate as missing data; C++ finite ? optional(z) : nullopt.
		z.is_finite().then_some(z)
	}
}

/// Compatibility wrapper: zero observer height and an explicit distance limit.
pub fn horizon_from_dem(dem: &Dem, lat: f64, lon: f64, n_az: usize, max_dist_m: f64) -> Horizon
{
	// Adapt a memory grid to the fallible sampler without introducing I/O; C++ lambda adapter.
	horizon_from_terrain(lat, lon, n_az, 0.0, Some(max_dist_m), |lat, lon| {
		// Project each spherical destination, not a straight map-grid ray; C++ structured binding.
		let (x, y) = to_xy(lat, lon);
		// Preserve the legacy projected spacing for existing grid callers; C++ optional transform.
		Ok(dem.height_at(x, y).map(|z| TerrainSample {
			z_m: f64::from(z),
			resolution_m: dem.res_m,
		}))
	})
	// Preserve the old empty/missing-data convention only in this compatibility API; C++ expected fallback.
	.unwrap_or_else(|_| Horizon {
		h_deg: vec![-90.0; n_az],
	})
}

// A sampled surface height and local ground pixel spacing; C++ plain value struct.
pub struct TerrainSample
{
	// Radial elevation above the reference sphere, not absolute radius; C++ double.
	pub z_m: f64,
	// Ground spacing allows the integrator to retain half-pixel radial sampling; C++ double.
	pub resolution_m: f64,
}

/// Great-circle destination with a geographic north/east tangent basis, also at the pole.
pub fn destination(lat: f64, lon: f64, az: f64, distance_m: f64) -> (f64, f64)
{
	// Form latitude, longitude, bearing and arc trigonometry once; C++ sin/cos pairs.
	let (sp, cp) = lat.to_radians().sin_cos();
	let (sl, cl) = lon.to_radians().sin_cos();
	let (sa, ca) = az.to_radians().sin_cos();
	let (sb, cb) = (distance_m / MOON_RADIUS_M).sin_cos();
	// r*cos(beta) + (north*cos(A)+east*sin(A))*sin(beta); C++ Cartesian components.
	let x = cp * cl * cb + (-sp * cl * ca - sl * sa) * sb;
	let y = cp * sl * cb + (-sp * sl * ca + cl * sa) * sb;
	let z = sp * cb + cp * ca * sb;
	// atan2 avoids asin's near-pole conditioning and enforces finite longitude; C++ std::atan2/hypot.
	(z.atan2(x.hypot(y)).to_degrees(), y.atan2(x).to_degrees())
}

/// Exact radial-tangent elevation in degrees; observer_z includes instrument height.
pub fn spherical_elevation(z: f64, observer_z: f64, beta: f64) -> f64
{
	// Rewrite cos(beta)-1 to avoid cancellation for meter-scale baselines; C++ double expression.
	let vertical = z - observer_z - 2.0 * (MOON_RADIUS_M + z) * (beta / 2.0).sin().powi(2);
	// atan2 retains the correct quadrant unlike atan(vertical/horizontal); C++ std::atan2.
	vertical
		.atan2((MOON_RADIUS_M + z) * beta.sin())
		.to_degrees()
}

/// Sample 0..360 geographic bearings to the first unavailable point on each ray.
/// The sampler can live in prep and perform I/O; this crate only evaluates geometry.
pub fn horizon_from_terrain(
	lat: f64,
	lon: f64,
	n_az: usize,
	observer_height_m: f64,
	max_distance_m: Option<f64>,
	mut sample: impl FnMut(f64, f64) -> Result<Option<TerrainSample>, String>,
) -> Result<Horizon, String>
{
	// Reject invalid integration inputs instead of producing plausible empty terrain; C++ guard with error return.
	if !lat.is_finite()
		|| !(-90.0..=90.0).contains(&lat)
		|| !lon.is_finite()
		|| n_az == 0
		|| !observer_height_m.is_finite()
		|| observer_height_m < 0.0
		|| max_distance_m.is_some_and(|v| !v.is_finite() || v <= 0.0)
	{
		return Err("Invalid horizon sampling parameters".into());
	}
	// A missing observer invalidates the whole calculation; C++ checked optional extraction.
	let origin = sample(lat, lon)?.ok_or("Observer is outside valid DEM coverage")?;
	// Validate the sampler contract before using spacing in a loop; C++ floating-point guards.
	if !origin.z_m.is_finite()
		|| origin.z_m <= -MOON_RADIUS_M
		|| !origin.resolution_m.is_finite()
		|| origin.resolution_m <= 0.0
	{
		return Err("Invalid observer terrain sample".into());
	}
	// The instrument is above the local radial surface, not above a flat zero plane; C++ double.
	let z0 = origin.z_m + observer_height_m;
	// Use half the finest starting ground pixel near the observer; C++ double.
	let near = origin.resolution_m / 2.0;
	// Refuse overflow or a spacing rounded to zero before entering the integration loop; C++ floating-point guard.
	if !z0.is_finite() || near <= 0.0
	{
		return Err("Invalid observer height or spacing".into());
	}
	// A quarter bearing-bin width is the adaptive angular target; C++ radians constant.
	let angular = std::f64::consts::TAU / n_az as f64 / 4.0;
	// A ray never wraps around the Moon; normal production termination is coverage, not this guard; C++ min.
	let limit = max_distance_m.unwrap_or(std::f64::consts::PI * MOON_RADIUS_M);
	// Allocate one maximum elevation per uniformly spaced bearing; C++ vector<float>.
	let mut h_deg = vec![-90.0_f32; n_az];
	// Evaluate each independent geographic ray; C++ indexed range-for.
	for (i, h) in h_deg.iter_mut().enumerate()
	{
		// Convert the bearing index into degrees; C++ double arithmetic.
		let az = i as f64 * 360.0 / n_az as f64;
		// Start off the observer so atan2 never evaluates the coincident point; C++ loop variable.
		let mut s = near;
		// Trace until the requested compatibility limit or continuous DEM edge; C++ while.
		while s <= limit
		{
			// Advance on the sphere before projecting in the sampler; C++ structured binding.
			let (p, l) = destination(lat, lon, az, s);
			// Propagate decoding errors; only absent data terminates a ray; C++ expected/optional extraction.
			let Some(point) = sample(p, l)?
			else
			{
				break;
			};
			// A malformed sample is an error, never an artificial horizon gap; C++ guard.
			if !point.z_m.is_finite()
				|| point.z_m <= -MOON_RADIUS_M
				|| !point.resolution_m.is_finite()
				|| point.resolution_m <= 0.0
			{
				return Err("Invalid terrain sample".into());
			}
			// Retain the largest exact spherical elevation seen on this ray; C++ std::max.
			*h = h.max(spherical_elevation(point.z_m, z0, s / MOON_RADIUS_M) as f32);
			// Grow only up to half a local pixel: do not skip resolved narrow radial features; C++ min/max.
			let next = s + (s * angular).max(near).min(point.resolution_m / 2.0);
			// A broken callback must not stall a finite ray forever; C++ monotonic-progress guard.
			if next <= s
			{
				return Err("Terrain spacing is below floating-point resolution".into());
			}
			// Advance only after validating floating-point progress; C++ assignment.
			s = next;
		}
		// Fail closed if a ray has no noncoincident sample; C++ sentinel check.
		if *h == -90.0
		{
			return Err(format!("No terrain samples at azimuth {az}"));
		}
	}
	// Move the cyclic profile to the caller; C++ expected<Horizon>.
	Ok(Horizon { h_deg })
}

pub fn sun_fraction(el: f32, h: f32) -> f32
{
	// Normalize the horizon's chord offset by the apparent solar radius; C++ clamped double.
	let u = ((f64::from(h) - f64::from(el)) / f64::from(SUN_RADIUS_DEG)).clamp(-1.0, 1.0);
	// Integrate the visible circular segment, not the visible diameter; C++ std::acos/sqrt.
	((u.acos() - u * (1.0 - u * u).sqrt()) / std::f64::consts::PI).clamp(0.0, 1.0) as f32
}

pub fn visible(track: &Track, horizon: &Horizon, margin_deg: f32) -> Vec<bool>
{
	// Parallel arrays are an explicit scientific precondition; C++ assertions.
	assert_eq!(track.t.len(), track.az_deg.len());
	assert_eq!(track.t.len(), track.el_deg.len());
	// Compare each center to the interpolated terrain plus adjustable clearance; C++ zipped transform.
	track
		.az_deg
		.iter()
		.zip(&track.el_deg)
		.map(|(&az, &el)| el > horizon.at(az) + margin_deg)
		.collect()
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stats
{
	// Time with any caller-defined solar availability; C++ double percentage.
	pub pct_sun: f64,
	// Longest consecutive unavailable solar interval; C++ double hours.
	pub longest_night_h: f64,
	// Time with the chosen Earth clearance; C++ double percentage.
	pub pct_earth: f64,
	// Time when both availability predicates hold; C++ double percentage.
	pub pct_both: f64,
}

/// Time-weighted percentages using boolean visibility held over [t[i], t[i+1]).
/// Gaps count in full; the final sample has no duration. Empty/single series return zeros.
/// Panics for unequal lengths or timestamps that are not strictly increasing.
pub fn stats(t: &[i64], sun_vis: &[bool], earth_vis: &[bool]) -> Stats
{
	// Reject incompatible parallel arrays; C++ assertions.
	assert_eq!(t.len(), sun_vis.len());
	assert_eq!(t.len(), earth_vis.len());
	// Default-zero accumulators also define empty/single-sample results; C++ value initialization.
	let mut result = Stats::default();
	let mut night: f64 = 0.0;
	let mut total = 0.0;
	// Integrate half-open intervals, never invent a duration for the last point; C++ adjacent-pair loop.
	for (i, pair) in t.windows(2).enumerate()
	{
		// Nonincreasing timestamps invalidate elapsed time; C++ assertion.
		assert!(pair[1] > pair[0]);
		// Cast before subtraction to avoid signed-integer overflow; C++ double conversion.
		let dt = (pair[1] as f64 - pair[0] as f64) / 3600.0;
		// Every interval contributes elapsed time even across gaps; C++ accumulation.
		total += dt;
		// Visibility at the left endpoint is held across this interval; C++ boolean branch.
		if sun_vis[i]
		{
			// Sunlight ends a consecutive night; C++ counters.
			result.pct_sun += dt;
			night = 0.0;
		}
		else
		{
			// Extend darkness and retain its maximum; C++ std::max.
			night += dt;
			result.longest_night_h = result.longest_night_h.max(night);
		}
		// Communication availability is independent of illumination; C++ conditional accumulation.
		if earth_vis[i]
		{
			result.pct_earth += dt;
		}
		// Joint availability uses a logical intersection; C++ &&.
		if sun_vis[i] && earth_vis[i]
		{
			result.pct_both += dt;
		}
	}
	// Only positive-duration series can be normalized; C++ guard against division by zero.
	if total > 0.0
	{
		// Convert accumulated hours into percentages; C++ scalar multiplications.
		result.pct_sun *= 100.0 / total;
		result.pct_earth *= 100.0 / total;
		result.pct_both *= 100.0 / total;
	}
	// Return the accumulated scientific result by value; C++ return-value optimization.
	result
}

pub fn parse_horizons(text: &str) -> Result<Track, String>
{
	// Isolate only the service's marked numeric ephemeris block; C++ checked delimiter searches.
	let (_, rest) = text.split_once("$$SOE").ok_or("Missing $$SOE")?;
	let (body, _) = rest.split_once("$$EOE").ok_or("Missing $$EOE")?;
	// Own output arrays, initially empty; C++ Track{}.
	let mut track = Track::default();
	// Skip blank separator lines inside the data block; C++ filtered line iteration.
	for line in body.lines().filter(|line| !line.trim().is_empty())
	{
		// Whitespace tokenization also accepts optional printed event flags; C++ vector<string_view>.
		let fields: Vec<_> = line.split_whitespace().collect();
		// Need a date, time, azimuth and elevation; C++ minimum-length guard.
		if fields.len() < 4
		{
			// Preserve the failing row in diagnostics; C++ formatted error alternative.
			return Err(format!("Invalid ephemeris row: {line}"));
		}
		// Accept both minute and second precision in Horizons UTC output; C++ alternate chrono parse formats.
		let time = NaiveDateTime::parse_from_str(
			&format!("{} {}", fields[0], fields[1]),
			"%Y-%b-%d %H:%M",
		)
		.or_else(|_| {
			NaiveDateTime::parse_from_str(
				&format!("{} {}", fields[0], fields[1]),
				"%Y-%b-%d %H:%M:%S",
			)
		})
		.map_err(|e| format!("Invalid date: {e}"))?
		.and_utc()
		.timestamp();
		// The requested two angles are the last two numeric columns, after any event flags; C++ reverse numeric-token iterator.
		let mut angles = fields[2..]
			.iter()
			.rev()
			.filter_map(|s| s.parse::<f32>().ok());
		// Reverse order yields elevation first, then azimuth; C++ optional<float> extraction.
		let el = angles.next();
		let az = angles.next();
		// A successful row must contain both values; C++ checked optional destructuring.
		let (Some(az), Some(el)) = (az, el)
		else
		{
			return Err(format!("Expected azimuth and elevation: {line}"));
		};
		// JSON and downstream geometry require finite directions; C++ std::isfinite.
		if !az.is_finite() || !el.is_finite()
		{
			return Err(format!("Expected finite azimuth and elevation: {line}"));
		}
		// Reject duplicate or reversed sample times before appending; C++ optional-last comparison.
		if track.t.last().is_some_and(|&last| time <= last)
		{
			return Err("Timestamps are not increasing".into());
		}
		// Keep all three arrays aligned on every successful row; C++ vector::push_back.
		track.t.push(time);
		track.az_deg.push(az);
		track.el_deg.push(el);
	}
	// Empty marked output is an error rather than an apparently successful download; C++ vector::empty.
	if track.t.is_empty()
	{
		return Err("Empty ephemeris".into());
	}
	// Move the valid track to the caller; C++ expected containing the result.
	Ok(track)
}

#[cfg(test)]
mod tests;

// Keep geometry-only terrain fixtures independent of TIFF I/O; C++ conditional test translation unit.
#[cfg(test)]
mod terrain_tests;
```

### 14.4. core/src/terrain_tests.rs

```rust
// Import only scientific values and geometry; C++ test includes the core header.
use super::*;

// Register independent projection metadata and axis checks; C++ TEST macro.
#[test]
fn pgda_projection_matches_labeled_axes()
{
	// PGDA's spherical pole has no false offsets; C++ exact origin assertion.
	assert_eq!(to_xy(-90.0, 0.0), (0.0, 0.0));
	// At the labeled 60S latitude, tan(15 degrees)=2-sqrt(3); C++ independent exact-angle reference.
	let northing_60s = 2.0 * MOON_RADIUS_M * (2.0 - 3.0_f64.sqrt());
	assert!((to_xy(-60.0, 0.0).1 - northing_60s).abs() < 1e-8);
	// The Site01 label gives the upper-left corner (-19000,-4000) meters; invert its radius; C++ independent inverse formula.
	let (x, y) = (-19000.0_f64, -4000.0_f64);
	let lat = -90.0 + (2.0 * (x.hypot(y) / (2.0 * MOON_RADIUS_M)).atan()).to_degrees();
	let lon = x.atan2(y).to_degrees();
	// A known map coordinate must round-trip, including both axis signs; C++ tolerance assertions.
	let projected = to_xy(lat, lon);
	assert!((projected.0 - x).abs() < 1e-8 && (projected.1 - y).abs() < 1e-8);
	// Longitude 90E maps toward positive x, and longitude zero toward positive y; C++ coordinate convention checks.
	assert!(to_xy(-89.0, 90.0).0 > 30000.0);
	assert!(to_xy(-89.0, 0.0).1 > 30000.0);
}

// Register exact geometry checks; C++ TEST macro.
#[test]
fn exact_elevation_and_great_circle()
{
	// Equal surface radii give the exact half-central-angle depression; C++ closed-form identity.
	for beta in [1e-10, 1e-5, 0.1, 1.0]
	{
		assert!((spherical_elevation(500.0, 500.0, beta) + beta.to_degrees() / 2.0).abs() < 1e-10);
	}
	// Compare against the requested unrearranged formula at a non-cancelling distance; C++ direct independent equation.
	let beta = 0.01_f64;
	let expected = ((MOON_RADIUS_M + 1200.0) * beta.cos() - (MOON_RADIUS_M + 802.0))
		.atan2((MOON_RADIUS_M + 1200.0) * beta.sin())
		.to_degrees();
	assert!((spherical_elevation(1200.0, 802.0, beta) - expected).abs() < 1e-10);
	// A northward meridian keeps longitude and increases latitude by the central angle; C++ expected destination.
	let d = MOON_RADIUS_M * 0.25_f64.to_radians();
	let (lat, lon) = destination(-89.45, 222.69, 0.0, d);
	assert!((lat + 89.2).abs() < 1e-10);
	assert!((lon.rem_euclid(360.0) - 222.69).abs() < 1e-10);
	// At the south pole the longitude convention still fixes the local north direction; C++ pole regression.
	let (lat, lon) = destination(-90.0, 90.0, 90.0, d);
	assert!((lat + 89.75).abs() < 1e-10 && (lon - 180.0).abs() < 1e-10);
	// At 90E, geographic east points toward negative projected y; C++ short-ray axis test.
	assert!(
		to_xy(
			destination(-89.0, 90.0, 90.0, 100.0).0,
			destination(-89.0, 90.0, 90.0, 100.0).1
		)
		.1 < 0.0
	);
}

// Register adaptive integration, height, and error checks; C++ TEST macro.
#[test]
fn terrain_sampler_height_edges_and_errors()
{
	// A flat sphere with a raised observer has a known tangent depression; C++ test lambda.
	let h = horizon_from_terrain(-90.0, 0.0, 8, 2.0, Some(20000.0), |_, _| {
		Ok(Some(TerrainSample {
			z_m: 0.0,
			resolution_m: 10.0,
		}))
	})
	.unwrap();
	let expected = -(MOON_RADIUS_M / (MOON_RADIUS_M + 2.0)).acos().to_degrees() as f32;
	assert!(h.h_deg.iter().all(|v| (*v - expected).abs() < 1e-5));
	// No observer data cannot produce a scientifically usable profile; C++ optional-empty callback.
	assert!(horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |_, _| Ok(None)).is_err());
	// Decoding errors must propagate instead of truncating a ray; C++ expected-error callback.
	assert_eq!(
		horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |_, _| Err("bad TIFF".into())).unwrap_err(),
		"bad TIFF"
	);
	// Reject nonsensical inputs before using the callback; C++ validation table.
	for (lat, n, height) in [(91.0, 8, 2.0), (-90.0, 0, 2.0), (-90.0, 8, -1.0)]
	{
		assert!(
			horizon_from_terrain(lat, 0.0, n, height, None, |_, _| panic!("must not sample"))
				.is_err()
		);
	}
	// Corrupt elevation or resolution is not a missing-data edge; C++ invalid-sample table.
	for (z, res) in [(f64::NAN, 5.0), (0.0, 0.0), (-MOON_RADIUS_M, 5.0)]
	{
		assert!(
			horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |_, _| Ok(Some(TerrainSample {
				z_m: z,
				resolution_m: res
			})))
			.is_err()
		);
	}
	// A finite coverage disk ends each ray without imposing an arbitrary production distance; C++ closure with a geographic coverage predicate.
	let edge = horizon_from_terrain(-90.0, 0.0, 8, 2.0, None, |lat, lon| {
		let (x, y) = to_xy(lat, lon);
		Ok((x.hypot(y) < 1000.0).then_some(TerrainSample {
			z_m: 0.0,
			resolution_m: 10.0,
		}))
	})
	.unwrap();
	assert_eq!(edge.h_deg.len(), 8);
	assert!(edge.h_deg.iter().all(|v| v.is_finite() && *v < 0.0));
}
```

### 14.5. prep/src/dem_tests.rs

```rust
// Access private decoder and projection helpers; C++ implementation test namespace.
use super::*;
// Build lossless synthetic float TIFFs entirely in memory; C++ byte-stream fixture encoder.
use std::io::Cursor;
use tiff::encoder::{TiffEncoder, colortype};

// Encode the exact scalar key conventions checked by the reader; C++ fixture factory.
fn keys() -> Vec<u16>
{
	// Radius keys reference the one DOUBLE value; all other values are inline SHORTs; C++ vector initialization.
	vec![
		1, 1, 0, 13, 1024, 0, 1, 1, 1025, 0, 1, 1, 2054, 0, 1, 9102, 2057, 34736, 1, 0, 2058,
		34736, 1, 0, 2061, 0, 1, 0, 3075, 0, 1, 15, 3076, 0, 1, 9001, 3081, 34736, 1, 1, 3082, 0,
		1, 0, 3083, 0, 1, 0, 3092, 0, 1, 1, 3095, 0, 1, 0,
	]
}

// Construct a two-by-two float32 raster with north-to-south rows; C++ binary fixture factory.
fn fixture(values: &[f32], res: f64, invalid_projection: bool) -> Cursor<Vec<u8>>
{
	// A cursor implements both Read and Seek without filesystem I/O; C++ stringstream.
	let mut bytes = Cursor::new(Vec::new());
	// Limit the encoder borrow's lifetime before rewinding the stream; C++ lexical RAII scope.
	{
		// Use ordinary TIFF to cover strip decoding, while real PGDA90 files also exercise BigTIFF; C++ TIFF encoder.
		let mut encoder = TiffEncoder::new(&mut bytes).unwrap();
		let mut image = encoder.new_image::<colortype::Gray32Float>(2, 2).unwrap();
		// Split rows into distinct strips to exercise cache indexing across chunks; C++ encoder setting.
		image.rows_per_strip(1).unwrap();
		// Pixel-is-area corner gives sample centers (-res/2,+res/2) and (+res/2,-res/2); C++ tag writes.
		image
			.encoder()
			.write_tag(Tag::ModelPixelScaleTag, &[res, res, 0.0][..])
			.unwrap();
		image
			.encoder()
			.write_tag(Tag::ModelTiepointTag, &[0.0, 0.0, 0.0, -res, res, 0.0][..])
			.unwrap();
		image
			.encoder()
			.write_tag(Tag::GeoKeyDirectoryTag, &keys()[..])
			.unwrap();
		image
			.encoder()
			.write_tag(
				Tag::GeoDoubleParamsTag,
				&[
					if invalid_projection
					{
						6378137.0
					}
					else
					{
						MOON_RADIUS_M
					},
					-90.0,
				][..],
			)
			.unwrap();
		image.encoder().write_tag(Tag::GdalNodata, "nan").unwrap();
		// Float32 preserves NaN as the actual published nodata representation; C++ write typed sample array.
		image.write_data(values).unwrap();
	}
	// Rewind to the TIFF header for the decoder; C++ seekg(0).
	bytes.set_position(0);
	bytes
}

// Register raster interpolation and coverage checks; C++ TEST macro.
#[test]
fn decoded_strips_centers_interpolation_and_nodata()
{
	// These are the downloaded Site01's actual dimensions, scale and tie point; C++ label-derived transform fixture.
	let real = grid(
		3200,
		3200,
		&[5.0, 5.0, 0.0],
		&[0.0, 0.0, 0.0, -19000.0, -4000.0, 0.0],
		&keys(),
		&[MOON_RADIUS_M, -90.0],
	)
	.unwrap();
	assert_eq!((real.x, real.y), (-18997.5, -4002.5));
	// Decode actual TIFF bytes rather than mocking the TIFF crate; C++ checked factory.
	let mut r = Raster::new(fixture(&[0.0, 2.0, 4.0, 6.0], 5.0, false)).unwrap();
	// The upper-left sample is half a pixel inside the corner tie point; C++ exact center assertion.
	assert_eq!(r.height(-2.5, 2.5).unwrap(), Some(0.0));
	assert_eq!(r.height(2.5, -2.5).unwrap(), Some(6.0));
	// Equal bilinear weights at the pole average all four elevations; C++ interpolation assertion.
	assert_eq!(r.height(0.0, 0.0).unwrap(), Some(3.0));
	// Repeated reads hit the two retained strips, without accumulating duplicate cache entries; C++ cache invariant.
	assert_eq!(r.cache.len(), 2);
	assert_eq!(r.height(0.0, 0.0).unwrap(), Some(3.0));
	assert_eq!(r.order.len(), 2);
	// Do not extrapolate beyond the last sample center or accept NaN coordinates; C++ empty optional checks.
	assert_eq!(r.height(2.5001, 0.0).unwrap(), None);
	assert_eq!(r.height(f64::NAN, 0.0).unwrap(), None);
	// Any invalid interpolation corner causes a fallback, not a fabricated zero height; C++ nodata fixture.
	let mut nodata = Raster::new(fixture(&[0.0, f32::NAN, 4.0, 6.0], 5.0, false)).unwrap();
	assert_eq!(nodata.height(0.0, 0.0).unwrap(), None);
	// At an exact valid center the NaN neighbor has zero weight; C++ boundary interpolation assertion.
	assert_eq!(nodata.height(-2.5, 2.5).unwrap(), Some(0.0));
}

// Register resolution preference and fallback checks; C++ TEST macro.
#[test]
fn finest_valid_surface_wins_and_nodata_falls_back()
{
	// Synthetic tiers intentionally disagree, proving that finest-first selection executes; C++ vector of movable decoders.
	let mut terrain = Terrain {
		rasters: vec![
			Raster::new(fixture(&[10.0; 4], 5.0, false)).unwrap(),
			Raster::new(fixture(&[20.0; 4], 20.0, false)).unwrap(),
		],
	};
	assert_eq!(terrain.sample(-90.0, 0.0).unwrap().unwrap().z_m, 10.0);
	// A location outside the fine raster but inside the coarse raster must use the latter; C++ optional fallback.
	let lat = -90.0 + (2.0 * (5.0 / (2.0 * MOON_RADIUS_M)).atan()).to_degrees();
	assert_eq!(terrain.sample(lat, 90.0).unwrap().unwrap().z_m, 20.0);
	// Source NaN also triggers a valid coarse fallback; C++ replace first decoder fixture.
	terrain.rasters[0] = Raster::new(fixture(&[f32::NAN; 4], 5.0, false)).unwrap();
	assert_eq!(terrain.sample(-90.0, 0.0).unwrap().unwrap().z_m, 20.0);
	// No tier covers a distant point; C++ nullopt assertion.
	assert!(terrain.sample(-80.0, 0.0).unwrap().is_none());
}

// Register projection and corrupt-file rejection checks; C++ TEST macro.
#[test]
fn rejects_wrong_sphere_bad_keys_and_corrupt_tiff()
{
	// An Earth radius may not enter lunar ray geometry; C++ factory failure assertion.
	assert!(Raster::new(fixture(&[0.0; 4], 5.0, true)).is_err());
	// A NaN radius must not pass through floating-point comparison semantics; C++ invalid-key regression.
	assert!(
		grid(
			2,
			2,
			&[5.0, 5.0, 0.0],
			&[0.0; 6],
			&keys(),
			&[f64::NAN, -90.0]
		)
		.is_err()
	);
	// Invalid bytes and missing key-directory headers are errors, not empty images; C++ checked parser assertions.
	assert!(Raster::new(Cursor::new(vec![])).is_err());
	assert!(key(&[], &[], 1024).is_err());
	// A DOUBLE offset outside its backing table is rejected; C++ bounds-check regression.
	assert!(key(&[1, 1, 0, 1, 2057, 34736, 1, 2], &[MOON_RADIUS_M], 2057).is_err());
	// Duplicated key definitions must not depend on entry order; C++ ambiguity rejection.
	assert!(key(&[1, 1, 0, 2, 1024, 0, 1, 1, 1024, 0, 1, 2], &[], 1024).is_err());
	// The compiled scientific inventory has checked digests and stays under the download gate; C++ manifest regression.
	let inventory = assets().unwrap();
	assert_eq!(inventory.len(), 5);
	assert!(inventory.iter().map(|a| a.bytes).sum::<u64>() < 20_000_000_000);
}
```

### 14.6. prep/download_dem.py

```python
#!/usr/bin/env python3
"""Download only the fixed-site manifest, outside the repository, and verify SHA-256."""

# Native standard-library modules avoid an additional package environment; C++ standard headers.
import argparse
import concurrent.futures
import hashlib
import json
from pathlib import Path
import urllib.request


# Hash large rasters with bounded RAM; C++ function reading a buffered ifstream.
def digest(path):
    # SHA-256 identifies the published bytes, not just a similarly sized TIFF; C++ digest context.
    value = hashlib.sha256()
    # A context manager closes the file on success and exceptions; C++ RAII ifstream.
    with path.open("rb") as source:
        # Stream 8 MiB blocks rather than allocating gigabytes; C++ while(read(...)).
        while block := source.read(8 * 1024 * 1024):
            # Feed exactly the downloaded bytes into the digest; C++ hash.update.
            value.update(block)
    # Hexadecimal text is directly comparable to the manifest; C++ digest-to-hex utility.
    return value.hexdigest()


# Support resumable downloads without exposing an incomplete final filename; C++ asset-download function.
def download(asset, directory, verify_only=False):
    # Join a trusted manifest basename under the explicitly selected data directory; C++ filesystem::path.
    target = directory / asset["file"]
    # A final filename must already match both size and checksum; C++ existence/verification branch.
    if target.exists():
        # Do not silently overwrite a corrupt or different published version; C++ checked precondition.
        if target.stat().st_size != asset["bytes"] or digest(target) != asset["sha256"]:
            raise ValueError(f"Existing DEM does not match manifest: {target}")
    # Verification-only mode performs no network I/O; C++ read-only operation branch.
    elif verify_only:
        raise FileNotFoundError(target)
    else:
        # Incomplete bytes remain resumable in a visibly separate file; C++ staging path.
        partial = target.with_name(target.name + ".part")
        # File length is the contiguous successfully written prefix; C++ filesystem::file_size.
        offset = partial.stat().st_size if partial.exists() else 0
        # A too-large prefix cannot be a valid resume of this asset; C++ bounds guard.
        if offset > asset["bytes"]:
            raise ValueError(f"Oversized partial DEM: {partial}")
        # An already complete prefix needs only checksum verification; C++ conditional HTTP operation.
        if offset < asset["bytes"]:
            # Range resumes explicitly require the correct response offset; C++ request headers.
            request = urllib.request.Request(asset["url"], headers={"Range": f"bytes={offset}-"})
            # Bound socket inactivity and append only after validating the response; C++ RAII response.
            with urllib.request.urlopen(request, timeout=180) as response:
                # Reject caches/servers ignoring a resume request; C++ status/header validation.
                if response.status != 206 or not response.headers.get("Content-Range", "").startswith(f"bytes {offset}-"):
                    raise ValueError(f"Server did not honor byte range: {asset['url']}")
                # Append exact response bytes without decompressing/re-encoding the TIFF; C++ binary append stream.
                with partial.open("ab") as output:
                    # Limit transient memory per worker; C++ buffered-copy loop.
                    while block := response.read(8 * 1024 * 1024):
                        output.write(block)
        # Verify before publishing the final path; C++ size and SHA-256 preconditions.
        if partial.stat().st_size != asset["bytes"] or digest(partial) != asset["sha256"]:
            raise ValueError(f"Downloaded DEM does not match manifest: {partial}")
        # Rename on the same filesystem publishes a completed file atomically; C++ filesystem::rename.
        partial.rename(target)
    # Emit an auditable per-file result; C++ formatted stream output.
    print(f"Verified {asset['file']}: {asset['bytes']:,} bytes, SHA-256 {asset['sha256']}", flush=True)


# Parse the small command interface before any network request; C++ program entry function.
def main():
    # The output directory is explicit to avoid accidentally vendoring DEMs; C++ CLI parser.
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--verify-only", action="store_true")
    arguments = parser.parse_args()
    # Read the versioned manifest next to this script, not a remote mutable file; C++ JSON file parsing.
    assets = json.loads(Path(__file__).with_name("dem-manifest.json").read_text())
    # Validate all paths, sizes and digests before creating files; C++ all_of predicate.
    if not assets or any(Path(a["file"]).name != a["file"] or a["bytes"] <= 0 or len(a["sha256"]) != 64 for a in assets):
        raise ValueError("Invalid DEM manifest")
    # Enforce the user's approximate 20 GB ceiling before starting downloads; C++ total-size reduction.
    total = sum(a["bytes"] for a in assets)
    if total > 20_000_000_000:
        raise ValueError(f"{total:,} bytes exceeds the 20 GB download gate")
    # Resolve symbolic paths before rejecting any directory inside the repository; C++ canonical paths.
    directory = arguments.directory.expanduser().resolve()
    repository = Path(__file__).resolve().parents[1]
    if directory == repository or repository in directory.parents:
        raise ValueError("DEM directory must be outside the repository")
    # Create only the caller's explicit external directory; C++ create_directories.
    directory.mkdir(parents=True, exist_ok=True)
    # Announce total disk usage before requests; C++ formatted diagnostic.
    print(f"Manifest: {len(assets)} DEMs, {total:,} bytes; directory={directory}", flush=True)
    # Four concurrent files are enough without issuing many requests to the public archive; C++ bounded worker pool.
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        # Materialization propagates every worker's failure to the command exit status; C++ join/get futures.
        list(pool.map(lambda asset: download(asset, directory, arguments.verify_only), assets))


# Importing the downloader for tests performs no network work; C++ main separated from library helpers.
if __name__ == "__main__":
    main()
```

### 14.7. prep/dem-manifest.json

```json
[
  {
    "url": "https://pgda.gsfc.nasa.gov/data/LOLA_5mpp/Site01/Site01_final_adj_5mpp_surf.tif",
    "bytes": 40980810,
    "file": "Site01_final_adj_5mpp_surf.tif",
    "sha256": "3ba7b97cb00a2bcf21189c3aeb535f65afc21207154ab9f0d43c5bdc1f7e747e"
  },
  {
    "url": "https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/LDEM_83S_10MPP_ADJ.TIF",
    "bytes": 5116709568,
    "file": "LDEM_83S_10MPP_ADJ.TIF",
    "sha256": "6c548d47896e238616149cfe336a1174eb9c070e34e8b605425fc40f768bde51"
  },
  {
    "url": "https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/LDEM_80S_20MPP_ADJ.TIF",
    "bytes": 2696082051,
    "file": "LDEM_80S_20MPP_ADJ.TIF",
    "sha256": "09b7ca80f9e6a146f970225d18af72fc02787669b3ef51b888e347d2b6845649"
  },
  {
    "url": "https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/LDEM_75S_30MPP_ADJ.TIF",
    "bytes": 2802850273,
    "file": "LDEM_75S_30MPP_ADJ.TIF",
    "sha256": "56dd4290379a4f94ff1e22fed232a87781158107bf20cd27568955f8bfdaaf9c"
  },
  {
    "url": "https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/LDEM_60S_60MPP_ADJ.TIF",
    "bytes": 3089110093,
    "file": "LDEM_60S_60MPP_ADJ.TIF",
    "sha256": "b03b5b3b3bf6a0eb8e871be7a7c04bcfa1f7d9c01c885db10655c9f26afc1f6a"
  }
]
```

### 14.8. prep/test_download_dem.py

```python
"""Offline downloader tests: no request reaches NASA or another shared service."""

# Standard-library fixtures and mocks have no extra environment dependencies; C++ test headers.
import hashlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import download_dem


# Isolate file/network behavior from real multi-gigabyte assets; C++ test fixture class.
class DownloadTests(unittest.TestCase):
    # Run the normal download and resumable-prefix paths against an in-memory response; C++ parameterized test.
    def test_download_and_resume(self):
        # Cover empty and partial prefixes without duplicating setup; C++ range-for.
        for offset in [0, 5]:
            # Automatically remove test-only temporary bytes; C++ scoped temporary directory.
            with self.subTest(offset=offset), tempfile.TemporaryDirectory() as name:
                # A known tiny binary payload gives an independent expected digest; C++ byte array.
                payload = b"synthetic DEM bytes"
                asset = {"file": "test.tif", "url": "https://example.invalid/test.tif", "bytes": len(payload), "sha256": hashlib.sha256(payload).hexdigest()}
                # Use a path object for the tested binary writer; C++ filesystem::path.
                directory = Path(name)
                # Only the resume fixture begins with saved bytes; C++ conditional prefix write.
                if offset:
                    (directory / "test.tif.part").write_bytes(payload[:offset])
                # BytesIO provides the same context-manager/read protocol as an HTTP response; C++ mock input stream.
                response = io.BytesIO(payload[offset:])
                response.status = 206
                response.headers = {"Content-Range": f"bytes {offset}-{len(payload)-1}/{len(payload)}"}
                # Replace only transport, not the downloader's file or digest logic; C++ injected transport stub.
                with patch("urllib.request.urlopen", return_value=response) as request:
                    download_dem.download(asset, directory)
                    # The requested starting byte must match the existing prefix; C++ mock argument assertion.
                    self.assertEqual(request.call_args.args[0].headers["Range"], f"bytes={offset}-")
                # Final bytes are exact and staging is no longer visible; C++ file-content assertions.
                self.assertEqual((directory / "test.tif").read_bytes(), payload)
                self.assertFalse((directory / "test.tif.part").exists())
                # Reverification of a completed asset must not contact the network; C++ mock expectation.
                with patch("urllib.request.urlopen", side_effect=AssertionError("unexpected HTTP")):
                    download_dem.download(asset, directory, verify_only=True)

    # Missing or altered complete files must fail without a request; C++ error-path test.
    def test_verify_missing_corrupt_and_empty_digest(self):
        # Keep all fixture files isolated; C++ scoped temporary directory.
        with tempfile.TemporaryDirectory() as name:
            # One byte is enough to test size and digest mismatches; C++ descriptor aggregate.
            directory = Path(name)
            asset = {"file": "test.tif", "url": "https://example.invalid/test.tif", "bytes": 1, "sha256": "0" * 64}
            # Verification-only cannot manufacture a missing asset; C++ expected exception.
            with self.assertRaises(FileNotFoundError):
                download_dem.download(asset, directory, verify_only=True)
            # Equal length does not imply equal contents; C++ corrupt-payload fixture.
            (directory / "test.tif").write_bytes(b"x")
            with self.assertRaises(ValueError):
                download_dem.download(asset, directory, verify_only=True)
            # Hashing an empty file is defined independently from downloading an empty DEM; C++ empty-stream digest.
            (directory / "empty").write_bytes(b"")
            self.assertEqual(download_dem.digest(directory / "empty"), hashlib.sha256(b"").hexdigest())

    # A server ignoring Range must not append unrelated bytes; C++ protocol-error test.
    def test_rejects_wrong_range(self):
        # Preserve a known prefix while rejecting the mock response; C++ scoped fixture.
        with tempfile.TemporaryDirectory() as name:
            directory = Path(name)
            (directory / "test.tif.part").write_bytes(b"a")
            asset = {"file": "test.tif", "url": "https://example.invalid/test.tif", "bytes": 2, "sha256": "0" * 64}
            # A 200 response means the resume contract was not honored; C++ mock status.
            response = io.BytesIO(b"ab")
            response.status = 200
            response.headers = {}
            with patch("urllib.request.urlopen", return_value=response), self.assertRaises(ValueError):
                download_dem.download(asset, directory)
            # The error occurs before opening the append stream; C++ unchanged-content assertion.
            self.assertEqual((directory / "test.tif.part").read_bytes(), b"a")


# Permit direct execution as well as unittest discovery; C++ test runner entry.
if __name__ == "__main__":
    unittest.main()
```
