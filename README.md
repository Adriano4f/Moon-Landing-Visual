# Moon Landing Visual

## Learning archive and current-main continuation

This branch preserves the long-form learning guides and the older teaching
source snapshot. Chapters 18–23 explain main through **`5e4d5b7`** without
merging application code into `learn`. Their source links are pinned to that
commit; relative source links and annotated listings in earlier chapters refer
to the historical code on this branch. Run current-feature experiments from a
separate checkout of that main revision, not from this archived source tree.

Continue after the original 01–17 material:

| Chapter | What you will learn |
|---|---|
| [18. Exact points and feedback](docs/18-exact-points-and-feedback.md) | Continuous inverse picking, cached colored dots, signed degrees, grid fading and progress-based timeout |
| [19. Far-field photometry](docs/19-far-field-photometry.md) | Why the close inspection camera created a moving opposition hotspot, and why geometry and photometric observers differ |
| [20. Terrain shadows](docs/20-terrain-shadows.md) | Packed metric heights, spherical rays, solar-disk penumbra, cached masks and fine-window shadows |
| [21. Atomic loading and flicker](docs/21-atomic-loading-and-flicker.md) | Compatible resource transactions, published/staged atlas pages, protected layers and hysteresis |
| [22. Quality and antialiasing](docs/22-quality-and-antialiasing.md) | Shader variants, implicit-limb coverage, filtering, projected quad and honestly scoped SwiftShader measurements |
| [23. Deep-zoom windows](docs/23-deep-zoom-windows.md) | WAC/NAC coverage, bounded photo/DEM requests, normal construction, resolutions and 2048× navigation |

The historical [any-point entry](docs/16-any-point.md) and
[deep-zoom entry](docs/17-deep-zoom.md) below have also been refreshed to avoid
teaching tile snapping or the obsolete atlas limits as current behavior.

## Project overview

Compare lunar south pole landing sites using Sun visibility for power and Earth visibility for direct communication.

Read the standalone [technical reference](docs/13-technical-reference.md) for architecture, provenance, scientific algorithms, rendering, validation and limitations.

The [portable ephemeris guide](docs/14-any-point-ephemeris.md) documents the 2,244,152-byte hourly Sun/Earth position file for 2027 through 2035 and arbitrary-location topocentric conversion. Current main uses that foundation for exact custom points as well as the seven prepared sites. The full binary is not fetched merely to draw the initial Moon; see chapter 18 for the custom job's 2027–2030 track construction and the timeline guides for shared date handling.

Start with [the skeleton guide for C++ programmers](docs/01-skeleton.md) for Rust basics, the math, every crate, and Windows/Linux setup. Continue with [the full code walkthrough](docs/02-code.md) for every source file and Rust API used.

Read [the from-zero rendering guide](docs/03-moon.md) for vectors, coordinate spaces, matrices, the GPU pipeline, NASA assets, normals, Hapke photometry, and color. Continue with [Rust and the complete globe code](docs/04-globe-code.md) for C++ comparisons, statement-level comments, exact source quotations, and the API index. The first two guides describe the original scientific workspace and placeholder app; the renderer guide documents its replacement.

Continue with [the panorama teaching guide](docs/05-panorama.md) for immediate-mode egui, UTC sample selection, egui_plot, north-wrap handling, asynchronous data loading, visibility readouts, and the rotating site marker. The bundled example covers 2027-01-01 through 2030-01-01 at one-hour cadence with a real 1,440-bearing terrain horizon. [The terrain teaching guide](docs/06-terrain.md) explains PGDA GeoTIFFs, their verified projection, pure-Rust decoding, great-circle rays, exact spherical elevation, solar area fractions, provenance, and reproducibility.

Open the folder in Zed with `zed .` or File > Open. Zed's Rust support is built in and downloads rust-analyzer automatically. There is no extension to install.

The default [standing-at-site view](docs/07-ui.md) shows finite Sun/Earth disks behind grey terrain, plain-language sunlight/radio statuses, whole-period summary cards and a question-mark help panel. Technical view retains the numerical plot. Vertical stretching is labeled, so the small angular disks are not silently enlarged. The [published-site catalog and globe controls](docs/09-sites-and-globe-controls.md) provide a site list, clickable outlined markers, drag rotation, zoom, go-to-site animation and lazy per-site loading. Connecting Ridge C1 remains the default. The [globe direction-ray guide](docs/08-rays.md) derives the local east/north/up frame used for thin, antialiased Sun/Earth ribbons and endpoint disks, with selected-only or all-sites display. An optional whole-Moon field shows incoming rays from the real Sun direction. Ray lengths are not distances; dim rays fail the same sunlight/radio tests as the panorama. The rounded baseline dataset is retained unchanged.

- `core/` (`lunar_core`): pure geometry, terrain horizons, visibility, time-weighted statistics, Horizons text parsing, and portable finite-distance ephemeris conversion/encoding.
- `prep/`: reads external LOLA PGDA 78/90 GeoTIFFs with the pure-Rust `tiff` crate, computes fixed-site terrain horizons through core's I/O-free sampler interface, downloads JPL Horizons observer tracks, and writes one JSON per site in `app/assets/data/`.
- `globe/`: rotating Moon renderer with runtime-loaded 2k/4k/8k NASA color and offline LOLA-derived normal maps. It has no dependency on `lunar_core` and does not compute landing-site visibility.
- `app/`: native and browser eframe shell displaying the globe against black beside a responsive Sun/Earth panorama. Narrow screens stack the globe above the plot. Native OpenGL or browser WebGL2 is required. Earth clearance is adjustable from 0.5 to 1.0 degrees, default 0.5. Current main downloads bounded Trek DEM exports for scientific custom points and Realistic rendering detail; the multi-gigabyte prep DEM inventory remains external.

Current main additionally provides Quality: Fast / Realistic. Fast uses recorded photographic contrast without rendering-height downloads; Realistic uses normals and date-driven DEM cast shadows. Both share analytic limb coverage and consistent loading. These rendering shadows do not replace the numerical scientific horizon.

```sh
cargo test -p lunar_core
python3 prep/download_dem.py /home/ubuntu/data/lola
cargo run -p prep --release -- sites.json 2027-01-01 2030-01-01 1h app/assets/data /home/ubuntu/data/lola 2
cargo run -p prep --release -- --ephemeris app/assets/data/moon-ephemeris.bin /home/ubuntu/data/horizons-moon-cache app/assets/data
cargo run -p app
rustup target add wasm32-unknown-unknown
cd app
trunk serve
```

Install [Trunk](https://trunkrs.dev/) for browser development. The prep arguments are the sites file, start date, stop date, step (`m`, `h`, or `d`), optional output directory, optional external DEM directory, and optional observer height in meters (default 2). Run prep from the workspace root. Omitting the DEM directory retains the explicitly flat legacy preparation path. Horizons times are UTC; azimuth is north clockwise toward east. Raw DEMs total 13,745,732,795 bytes and must stay outside the repository; the downloader verifies the versioned SHA-256 manifest.

Statistics hold each boolean sample until the next timestamp, including gaps. The last sample contributes no duration. Percentages are duration-weighted, not sample counts. Prep reports any visible solar area separately from the time-weighted mean visible area. Its default observer is 2 m above the interpolated DEM surface, a modeling choice, not a measured mast height. The decoder validates the sphere, axes, true-scale pole, units, pixel registration, and false offsets against each file's GeoKeys.

Reproduce the committed example with the terrain preparation command above. Native data loading checks `MOON_DATA_DIR`, then `assets/data` beside the executable, then the workspace app assets. The expected filename is `connecting-ridge-c1.json`. The browser requests `assets/data/connecting-ridge-c1.json` over HTTP; Trunk copies it with the other app assets. No scientific site JSON is embedded in the release app or WASM. The globe uses the selected epoch's Sun/Earth directions and its default Earth-following view. Drag for free orbit, or use Go to site; optional Auto-rotate is a presentation spin. Earthshine is phase-scaled with an explicit boosted exposure option. See [real lighting and camera](docs/10-real-lighting-and-orbit.md) for the derivation and [chapter 22](docs/22-quality-and-antialiasing.md) for current quality paths.
