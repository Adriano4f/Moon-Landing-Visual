# Moon Landing Visual

Compare lunar south pole landing sites using Sun visibility for power and Earth visibility for direct communication.

Start with [the skeleton guide for C++ programmers](docs/01-skeleton.md) for Rust basics, the math, every crate, and Windows/Linux setup. Continue with [the full code walkthrough](docs/02-code.md) for every source file and Rust API used.

Read [the from-zero rendering guide](docs/03-moon.md) for vectors, coordinate spaces, matrices, the GPU pipeline, NASA assets, normals, Hapke photometry, and color. Continue with [Rust and the complete globe code](docs/04-globe-code.md) for C++ comparisons, statement-level comments, exact source quotations, and the API index. The first two guides describe the original scientific workspace and placeholder app; the renderer guide documents its replacement.

Continue with [the panorama teaching guide](docs/05-panorama.md) for immediate-mode egui, UTC sample selection, egui_plot, north-wrap handling, asynchronous data loading, visibility readouts, and the rotating site marker. The bundled example covers 2027-01-01 through 2030-01-01 at one-hour cadence with a real 1,440-bearing terrain horizon. [The terrain teaching guide](docs/06-terrain.md) explains PGDA GeoTIFFs, their verified projection, pure-Rust decoding, great-circle rays, exact spherical elevation, solar area fractions, provenance, and reproducibility.

Open the folder in Zed with `zed .` or File > Open. Zed's Rust support is built in and downloads rust-analyzer automatically. There is no extension to install.

The default [standing-at-site view](docs/07-ui.md) shows finite Sun/Earth disks behind grey terrain, plain-language sunlight/radio statuses, whole-period summary cards and a question-mark help panel. Technical view retains the numerical plot. Vertical stretching is labeled, so the small angular disks are not silently enlarged. The fixed site is the published 2 m Connecting Ridge C1 position; the terrain guide explains why it replaces the older rounded-coordinate baseline. The [globe direction-ray guide](docs/08-rays.md) derives the local east/north/up frame used to attach the selected Sun and Earth directions to the rotating marker. Ray lengths are not distances; dim rays fail the same sunlight/radio tests as the panorama.

- `core/` (`lunar_core`): pure geometry, terrain horizons, visibility, time-weighted statistics, and Horizons text parsing.
- `prep/`: reads external LOLA PGDA 78/90 GeoTIFFs with the pure-Rust `tiff` crate, computes fixed-site terrain horizons through core's I/O-free sampler interface, downloads JPL Horizons observer tracks, and writes one JSON per site in `app/assets/data/`.
- `globe/`: rotating Moon renderer with runtime-loaded 2k/4k/8k NASA color and offline LOLA-derived normal maps. It has no dependency on `lunar_core` and does not compute landing-site visibility.
- `app/`: native and browser eframe shell displaying the globe against black beside a responsive Sun/Earth panorama. Narrow screens stack the globe above the plot. Native OpenGL or browser WebGL2 is required. Earth clearance is adjustable from 0.5 to 1.0 degrees, default 0.5. Raw DEMs are never loaded by the app.

```sh
cargo test -p lunar_core
python3 prep/download_dem.py /home/ubuntu/data/lola
cargo run -p prep --release -- sites.json 2027-01-01 2030-01-01 1h app/assets/data /home/ubuntu/data/lola 2
cargo run -p app
rustup target add wasm32-unknown-unknown
cd app
trunk serve
```

Install [Trunk](https://trunkrs.dev/) for browser development. The prep arguments are the sites file, start date, stop date, step (`m`, `h`, or `d`), optional output directory, optional external DEM directory, and optional observer height in meters (default 2). Run prep from the workspace root. Omitting the DEM directory retains the explicitly flat legacy preparation path. Horizons times are UTC; azimuth is north clockwise toward east. Raw DEMs total 13,745,732,795 bytes and must stay outside the repository; the downloader verifies the versioned SHA-256 manifest.

Statistics hold each boolean sample until the next timestamp, including gaps. The last sample contributes no duration. Percentages are duration-weighted, not sample counts. Prep reports any visible solar area separately from the time-weighted mean visible area. Its default observer is 2 m above the interpolated DEM surface, a modeling choice, not a measured mast height. The decoder validates the sphere, axes, true-scale pole, units, pixel registration, and false offsets against each file's GeoKeys.

Reproduce the committed example with the terrain preparation command above. Native data loading checks `MOON_DATA_DIR`, then `assets/data` beside the executable, then the workspace app assets. The expected filename is `connecting-ridge-c1.json`. The browser requests `assets/data/connecting-ridge-c1.json` over HTTP; Trunk copies it with the other app assets. No JSON is embedded in the release app or WASM. The globe's 60-second rotation, fixed light, and 30-degree southward presentation tilt are illustrative, not driven by the UTC slider. Only the panorama/readouts use the scientific Horizons tracks.
