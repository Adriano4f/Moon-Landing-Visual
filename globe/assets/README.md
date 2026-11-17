# Moon assets: sources and reproduction

## Attribution and provenance

- **Color and relief:** NASA Scientific Visualization Studio [CGI Moon Kit](https://svs.gsfc.nasa.gov/4720/), visualization by Ernie Wright using LROC color and LOLA relief. Offline inputs are [`lroc_color_16bit_srgb_8k.tif`](https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/lroc_color_16bit_srgb_8k.tif) and [`ldem_16_uint.tif`](https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_16_uint.tif). These are visualization assets, not the scientific horizon DEMs.
  - Color source: 243,289,792 bytes; SHA-256 `db7808e878b6a55eb409bb231eab8deb477f84b5c9d7396d76ff73e5d54992d9`.
  - Relief source: 33,201,026 bytes; SHA-256 `45a2b32d56e81ed30db07fead8abc842b249b6511219d9ca2c53f81bc2dc5d62`.
- **Photometry:** [LROC WAC Hapke parameter map, 566 nm, v1.1](https://pds.lroc.im-ldi.com/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/SDP/WAC_HAPKEPARAMMAP/WAC_HAPKEPARAMMAP_566NM.IMG), Sato et al. (2014), DOI [10.1002/2013JE004580](https://doi.org/10.1002/2013JE004580). Common parameters are cosine-latitude-weighted means over 50,400 valid tiles between 70°S and 70°N, not a polar-specific fit. Equation verification uses USGS ISIS3 `HapkeLROC.cpp` (`HGfunc`, `photometry`), commit `fadd004da2f733525209ee4f66ffdb3a9a9d193e`.
- **Runtime imagery:** NASA Moon Trek LROC WAC south-polar tiles; credit NASA, Arizona State University and the LROC Team. [Product metadata](https://trek.nasa.gov/moon/TrekWS/rest/cat/metadata/fgdc/html?label=LRO_WAC_Mosaic_SPole60_100m_v02) lists no access/use constraints. Recorded photographic shadows are not date-dependent illumination or calibrated albedo.
- **Polar normals:** offline NASA Moon Trek F32 LOLA south-polar export, interpreted in `polar_normals.meta.json`. Export spacing is 535.122 m (twice that for phone normals), not the source service's nominal 30 m.

`manifest.json` records generated texture sizes, hashes and linear luminance means. `polar_normals.meta.json` records the polar source hash and projection. Keep both records with the assets. Raw source TIFFs are not deployed; normal maps use PNG natively and lossless WebP in the browser. Overview albedo/normal tiers are 2k/4k/8k, with a 512×256 browser preview.

## Offline reproduction

From the repository root, with Python 3, Pillow and NumPy:

```sh
python3 -m pip install pillow numpy
python3 globe/assets/prepare.py
python3 globe/assets/web_normals.py
python3 globe/assets/preview.py
python3 globe/assets/photometry.py
python3 -m unittest discover -s globe/assets -p 'test_*.py'
```

`prepare.py` downloads missing CGI Moon Kit TIFFs to `~/plan` and writes the texture tiers and manifest. `photometry.py` downloads the Hapke map and checks the parameter means and shader equations. Polar normals can be regenerated separately (also requires Python `requests`):

```sh
python3 globe/assets/polar_normals.py /path/outside/repo/lola75-f32.tif
```

The rendering conventions are in [`docs/03-moon.md`](../../docs/03-moon.md); detailed learning material is preserved on [`learn`](https://github.com/Adriano4f/Moon-Landing-Visual/tree/learn).

## Packed height maps

`prepare.py` also writes global 2k/4k packed heights; `polar_normals.py` writes south-polar 1k/2k heights. To regenerate just heights without changing existing color/normal files:

```sh
python3 globe/assets/heights.py /path/outside/repo/lola75-f32.tif
```

`heights.manifest.json` records lossless PNG hashes, metric scale/offset and valid height bounds. Decode RG as `(R * 256 + G) * 0.5 - 10000` metres; B=255 marks valid samples. There is no sRGB/gamma profile. Upload as linear UNORM8, with no mipmap sampling; linear byte interpolation commutes with this affine height decoding. Missing polar data falls back to global heights.

Added downloads are 4,726,424 bytes for global 2k + polar 1k, or 17,524,390 bytes for global 4k + polar 2k. Only the selected pair loads. All four files total 22,250,814 bytes in the distribution.

HTTPS verification must remain enabled for source downloads. `SSL_CERT_FILE` (urllib) or `REQUESTS_CA_BUNDLE` (requests) may select a trusted local CA bundle if required by the environment.
