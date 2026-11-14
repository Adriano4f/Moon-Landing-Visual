# Moon Landing Visual

Compare lunar south pole landing sites using Sun visibility for power and Earth visibility for direct communication.

- `core/` (`lunar_core`): pure geometry, terrain horizons, visibility, time-weighted statistics, and Horizons text parsing.
- `prep/`: downloads JPL Horizons observer tracks and writes one JSON per site in `app/assets/data/`. Terrain is currently `null`; LOLA DEM loading is a future step.
- `app/`: native and browser eframe placeholder. The panorama is coming.

```sh
cargo test -p lunar_core
cargo run -p prep -- sites.json 2026-12-01 2026-12-03 6h
cargo run -p app
rustup target add wasm32-unknown-unknown
cd app
trunk serve
```

Install [Trunk](https://trunkrs.dev/) for browser development. The prep arguments are the sites file, start date, stop date, step (`m`, `h`, or `d`), and optional output directory. Run prep from the workspace root. Horizons times are UTC; azimuth is north clockwise toward east.

Statistics hold each boolean sample until the next timestamp, including gaps. The last sample contributes no duration. Percentages are duration-weighted, not sample counts. Terrain sample coordinates and polar projection orientation must be checked against each DEM label before using real terrain data.
