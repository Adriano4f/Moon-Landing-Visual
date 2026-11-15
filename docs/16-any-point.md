# Any-point terrain horizons

## Selection and grid

Click the Moon away from a published-site marker to select a custom point.
`lunar_core::anypoint::tile_of` snaps the click to the center of a global grid tile.
The grid has 90 latitude bands, each 2 degrees high.
Each band has `max(1, round(180 * cos(center latitude)))` longitude cells.
Longitude width is 360 degrees divided by that count: 2 degrees near the equator
and 120 degrees in the bands next to the poles. `tile_count()` returns 10,312.
A yellow outline shows the selected tile on the globe.
The name is `Custom point (lat, lon)`, with both coordinates to two decimals
and longitude displayed in -180..180 degrees.

## Picking and data flow

`globe/src/pick.rs` uses `View::unproject` to intersect a click with the Moon.
`take_pick` returns and clears the latitude and east longitude of a non-marker click.
`set_tile` supplies the tile boundary for the yellow outline.
`app/src/custom.rs` fetches the URLs from `terrain_requests` with `ehttp`,
both natively and in the browser, and loads the `moon-ephemeris.bin` asset.
The TIFF responses stay in request order. Core performs no HTTP or filesystem I/O.
`HorizonJob` computes 720 azimuth bins, advanced for up to 12 ms per app frame. Requests are 512² pixels (about 1 MB), or 1024² for polar fine windows; downloads time out after 30 s.
The observer's radial height is the tile-center surface height plus 2 m.
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
Global fine windows extend about 60,000 m, giving about 118 m north-south pixels.
Coarse windows remain unchanged: 300,000 m, polar at 80 degrees, global elsewhere.
Each request is 1,024 by 1,024 pixels, about 4.2 MB; normally two are needed.
Global longitude windows split at the -180/180 seam, so there can be up to four URLs.
The sampler tries fine terrain before coarse terrain and handles NoData.
Great-circle rays stop at 250 km or when neither raster covers the next sample.
Heights are meters above the 1,737.4 km lunar reference sphere.

## Validation

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
The result applies to the tile center, not the exact clicked position or every point in the tile.
There is no application-managed caching across reloads. Network failures are reported to the UI.
