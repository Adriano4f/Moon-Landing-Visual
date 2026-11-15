# Moon Landing Visual

Compare lunar south-pole sites by sunlight for power and direct Earth visibility for communication.

**Live site:** https://adriano4f.github.io/Moon-Landing-Visual/

The app combines an interactive 3D Moon, published landing-site coordinates and a standing-at-site panorama with terrain horizons. Sun visible-disk percentage, Earth clearance, a timeline, summary statistics and a technical graph support site comparisons; custom points can compute terrain horizons in the browser. Globe lighting is illustrative, not a mission-design prediction.

## Repository layout

- `core/`: I/O-free geometry, terrain horizons, visibility, statistics and ephemeris conversion.
- `globe/`: independent OpenGL/WebGL2 Moon renderer, camera, markers, rays and streamed detail.
- `app/`: native/browser eframe/egui interface, data loading, panorama and timeline.
- `prep/`: offline Rust/Python tools for DEM downloads, Horizons tracks and prepared datasets.

## Data sources

- **Offline:** [NASA JPL Horizons](https://ssd.jpl.nasa.gov/api/horizons.api) Sun/Earth ephemerides; NASA PGDA LOLA DEMs (`prep/dem-manifest.json`); [NASA SVS CGI Moon Kit](https://svs.gsfc.nasa.gov/4720/) textures and relief; LROC WAC 566 nm Hapke parameters.
- **Runtime:** NASA Moon Trek imagery tiles and LOLA `ImageServer/exportImage` terrain requests for custom points.
- **Site coordinates:** Glaser et al. (2017), with Mazarico et al. (2011) and Speyerer and Robinson (2013) coordinates verified secondarily through Glaser's tables. Citations and precision are recorded in [`sites.json`](sites.json).

Asset sources, credits and regeneration commands: [`globe/assets/README.md`](globe/assets/README.md). Rendering conventions: [`docs/03-moon.md`](docs/03-moon.md).

## Build and run

Install Rust and [Trunk](https://trunk-rs.github.io/trunk/). Run from the repository root (native requires OpenGL; the browser requires WebGL2):

```sh
cargo test --workspace
cargo clippy --workspace
cargo run -p app
rustup target add wasm32-unknown-unknown
cd app
trunk serve
# Or build the static site:
trunk build --release
```

### Regenerate scientific data

Run from the repository root. Keep raw DEMs outside the repository; the downloader checks the versioned SHA-256 manifest. The final terrain argument is observer height in metres (default 2); times are UTC. These commands download external data and may take substantial time and disk space.

```sh
python3 prep/download_dem.py /home/ubuntu/data/lola
cargo run -p prep --release -- sites.json 2027-01-01 2030-01-01 1h app/assets/data /home/ubuntu/data/lola 2
cargo run -p prep --release -- --ephemeris app/assets/data/moon-ephemeris.bin /home/ubuntu/data/horizons-moon-cache app/assets/data
```

## Deployment and credits

[`.github/workflows/pages.yml`](.github/workflows/pages.yml) builds `app/` with Trunk and deploys `app/dist` to GitHub Pages.

Credits: NASA/JPL, PGDA/LOLA, NASA SVS (Ernie Wright), LROC/ASU and the cited site authors. No project-wide license file is currently provided; asset attribution and bundled font licenses remain in [`globe/assets/README.md`](globe/assets/README.md) and [`app/assets/fonts/`](app/assets/fonts/).
