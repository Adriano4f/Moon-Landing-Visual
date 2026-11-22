# Any-point terrain horizons

## Selection and grid

For current main through `5e4d5b7`, clicking the Moon selects the continuous
intersection coordinate, not a tile center. The old `tile_of` helper remains in
core but is not used to place this observer. The grid is decorative: at zoom
at least 3× it appears after a zoom change, holds for one second and fades over
half a second. There is no selected-tile outline. Completed custom points remain
as clickable dots colored by duration-weighted sunlight availability for the
chosen period. The name uses signed-degree coordinates to two decimals; the
underlying point is not rounded to that label. Read the full derivation and
code walkthrough in [chapter 18](18-exact-points-and-feedback.md).

## Picking and data flow

`globe/src/pick.rs` uses `View::unproject` to intersect a click with the Moon.
`take_pick` returns and clears the latitude and east longitude of a non-marker click.
Existing computed dots are hit-tested in screen space and reopen their exact
cached coordinate. Marker and double-click handling avoid unintended new jobs.
`app/src/custom.rs` fetches the URLs from `terrain_requests` with `ehttp`,
both natively and in the browser, and loads the `moon-ephemeris.bin` asset.
The TIFF responses stay in request order. Core performs no HTTP or filesystem I/O.
`HorizonJob` computes 720 azimuth bins in four-bin chunks under a 12 ms per-frame
work budget. Requests are 512² pixels (about 1.05 MB of F32 plus metadata), or
1024² for polar fine windows (about 4.19 MB plus metadata). The job times out
after 90 seconds without another terrain response while terrain is outstanding;
this is an inactivity deadline, not a total-download deadline.
The observer's radial height is the exact selected point's DEM height plus 2 m.
`lunar_core::ephemeris::topocentric` computes the Sun and Earth tracks hourly
from 2027-01-01 through 2030-01-01 UTC, inclusive, matching the published sites.
The resulting site data feeds the existing panorama, visibility calculations and stats.

## Terrain sources and sampling

NASA Moon Trek ImageServer `exportImage` requests return uncompressed F32 TIFFs.
At absolute latitude at least 75 degrees, fine coverage tries polar LOLA DEMs.
`LRO_LOLA_DEM_SPole875_5mp_v04` and `LRO_LOLA_DEM_NPole875_5mp_v04`
provide 5 m pixels in windows extending 2,560 m from the center, including C1.
Outside their full-window extent, the south tries `ldem_85s_10m_Spole_float_SP`
at 10 m with a 5,120 m half-window. Both poles then try the corresponding
`LRO_LOLA_DEM_SPole75_30mp_v04` or `LRO_LOLA_DEM_NPole75_30mp_v04` at 30 m over 15,360 m.
Otherwise fine coverage uses geographic `LRO_LOLA_DEM_Global_256ppd_v06`.
Global fine windows have a 60,000 m half-width north/south before clipping,
giving about 234.375 m north-south output spacing at 512 samples. The source's
approximately 118 m native spacing must not be confused with this export spacing.
Coarse windows remain unchanged: 300,000 m, polar at 80 degrees, global elsewhere.
Coarse exports are 512², approximately 1,171.875 m north-south spacing for the
unclipped global 600 km span. Polar fine exports alone are 1024²; normally a
fine and coarse window are needed, with possible additional seam requests.
Global longitude windows split at the -180/180 seam, so there can be up to four URLs.
The sampler tries fine terrain before coarse terrain and handles NoData.
Great-circle rays stop at 250 km or when neither raster covers the next sample.
Heights are meters above the 1,737.4 km lunar reference sphere.

## Validation

The following are historical measurements retained from the earlier any-point
implementation, not newly rerun benchmarks of main `5e4d5b7`. Current regression
tests also cover continuous picking, dot identity, signed wrapping and timeout
progress; see chapter 18. Run network checks from a current-main checkout if
you need current service availability and fresh timing evidence.

| Check | Result |
|:--|:--|
| C1, -89.4398 latitude, 222.83 east longitude, 2 m observer | Max difference 1.005 degrees; RMS difference 0.338 degrees versus the 1,440-bin 5 m reference |
| C1 surface height | 1,958.289 m; reference 1,958.503 m |
| Equator, 0.0 latitude, 23.5 east longitude | Horizon minimum -0.502 degrees; maximum 1.152 degrees |
| Native release, 720 bins | C1 0.226 s; equator 0.261 s |
| Native release, worst 20-bin batch | C1 6.414 ms; equator 20.991 ms |
| Prior 30 m version, optimized wasm in Node/V8 | C1 0.141 to 0.149 s, worst 20 bins 9.389 ms; equator 0.262 to 0.264 s, worst 20 bins 8.809 ms |

Timings exclude downloads and are machine-specific, not browser frame guarantees.
To rerun the network check from the repository root, with `curl` available:

```sh
cargo test -p lunar_core --release network_check -- --ignored --nocapture
```

## Limitations

The finer source does not improve C1 agreement: the prior max/RMS was 0.716/0.254 degrees.
Downloads depend on `trek.nasa.gov` availability and its CORS policy in the browser.
The result now applies to the exact selected coordinate, not an entire tile or
every nearby point. The smooth globe is not displaced by this scientific DEM.
There is no application-managed caching across reloads. Network failures are reported to the UI.
