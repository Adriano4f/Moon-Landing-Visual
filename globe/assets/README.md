# NASA Moon assets and offline reproduction

## South-polar detail

Runtime LROC WAC south-polar WMTS imagery is credited to NASA, Arizona State University and the LROC Team. [Product metadata](https://trek.nasa.gov/moon/TrekWS/rest/cat/metadata/fgdc/html?label=LRO_WAC_Mosaic_SPole60_100m_v02) lists no access or use constraints. The mosaic has recorded photographic shadows, not date-dependent lighting or calibrated albedo; the shader uses only bounded contrast from it.

`polar_normals.py /path/outside/repo/lola75-f32.tif` prepares the 1k and 2k object-space normal files from Trek's unrendered F32 LOLA south-polar 30 m ImageServer export. The exported grid spacing is 535.122 m, not 30 m; phone normals use twice that spacing. Both files use south-polar stereographic coordinates with radius 1,737,400 m and square half-width 547,965 m. `polar_normals.meta.json` records the source hash and interpretation. The raw TIFF is not deployed. PNG is used natively, lossless WebP in the browser. Tests are in `test_polar_normals.py`.

The files and runtime imagery are lazy and do not join the startup request chain. See [complete projection, loading and scientific limits](../../docs/15-polar-detail.md).

## Sources and credit

Credit: NASA's Scientific Visualization Studio. The CGI Moon Kit visualization is by Ernie Wright, using LROC color and LOLA relief data. Source page: https://svs.gsfc.nasa.gov/4720/. These are visualization assets, not terrain products ingested by lunar_core.

The page lists lroc_color_2k.jpg (2,048 by 1,024; about 447.2 KB), lroc_color_16bit_srgb_4k.tif (4,096 by 2,048; about 59.0 MB), and lroc_color_16bit_srgb_8k.tif (8,192 by 4,096; about 232.0 MB). All committed albedo tiers below are generated consistently from the 8k TIFF rather than mixing independently encoded files.

Downloads actually used:
- https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/lroc_color_16bit_srgb_8k.tif
- https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_16_uint.tif

The color TIFF downloaded as 243,289,792 bytes, SHA-256 db7808e878b6a55eb409bb231eab8deb477f84b5c9d7396d76ff73e5d54992d9.
The unsigned LOLA TIFF downloaded as 33,201,026 bytes, SHA-256 45a2b32d56e81ed30db07fead8abc842b249b6511219d9ca2c53f81bc2dc5d62.

LOLA resolution is 16 pixels per degree, 5,760 by 2,880. The unsigned q encoding is h=0.5*q-10,000 meters relative to radius 1,737,400 m, equivalent to half-meter samples with a 20,000-sample offset. Longitude increases eastward, the equator is the center row, and north is at the top. The color map is aesthetic RGB sRGB imagery, not measured spectral single-scattering albedo.

## Generated files

Each tier has the following width and half that height. File hashes and pre-JPEG, area-weighted linear luminance means are in manifest.json.

| Tier | File | Bytes |
|:--|:--|--:|
| 2k | `albedo_2k.jpg` | 725,020 |
| 2k | `normal_2k.png` | 2,703,533 |
| 4k | `albedo_4k.jpg` | 2,795,173 |
| 4k | `normal_4k.png` | 10,637,334 |
| 8k | `albedo_8k.jpg` | 10,490,203 |
| 8k | `normal_8k.png` | 36,756,786 |

Total tier payload: 64,108,049 bytes (64.11 MB). Every file is below 100 MB; the total is below 150 MB. Source TIFFs are cached in ~/plan and are not committed. The earlier embedded 2k JPEG/4-pixels-per-degree PNG and its conversion script have been replaced.

Browser normal maps additionally have lossless WebP equivalents: 2,250,260 bytes at 2k, 8,878,268 at 4k, and 29,849,182 at 8k. Run `python3 globe/assets/web_normals.py` after preparation. It verifies every decoded RGB byte against the PNG. Trunk deploys the WebP copies instead of PNG; native retains PNG. See [progressive loading](../../docs/12-progressive-loading.md) for browser decoding, quality defaults, memory and measurements.

Browser startup additionally uses a 512 by 256 preview pair: `albedo_preview.jpg`, 29,555 bytes, and `normal_preview.webp`, 163,536 bytes. Run `python3 globe/assets/preview.py` after preparing 2k assets. Albedo is averaged in linear light; object-space normal vectors are averaged and renormalized. This coarse first picture does not change the selected final tiers, the DEM horizon or scientific visibility results. Native startup remains 2k.

## Reproduction and quantization

Install Python 3, Pillow, and NumPy, then run:

```sh
python3 -m pip install pillow numpy
python3 globe/assets/prepare.py
python3 -m unittest discover -s globe/assets -p 'test_*.py'
python3 globe/assets/photometry.py
```

The recorded bake used Pillow 12.3.0 and NumPy 2.2.6. prepare.py creates ~/plan if needed, downloads missing cached source files, verifies dimensions and height type, resizes albedo with Lanczos, and resizes metric relief with bilinear interpolation. Pillow converts the source RGB TIFF to RGB8 before JPEG encoding at quality 94, chroma subsampling disabled. JPEG is lossy; PNG normals are lossless after RGB8 vector quantization.

For each tier, the script differentiates heights using the metric east/north spacing on a sphere, with the 1,737,400 m reference radius and local height. Exaggeration is 1.0. Longitude differences wrap and slopes taper through the last degree near each pole. Normals are object-space, encoded as round((n*0.5+0.5)*255) per component. Decoding yields a maximum component quantization error of 1/255; the shader renormalizes the vector after filtering. The 8k relief texture resamples the 16-pixels-per-degree source, so its independent height information remains limited by that source.

The manifest mean decodes RGB8 sRGB to linear, forms luminance with weights 0.2126/0.7152/0.0722, and weights rows by cos(latitude). It is computed before JPEG compression. The shader uses the common rounded reference 0.4832 for all tiers. It multiplies linear color/reference by a common 566 nm Hapke response. This is an explicit spatial-modulation choice, not per-pixel Hapke inversion.

See [the renderer guide](../../docs/03-moon.md) for the complete normal derivation, texture memory estimates, runtime native path rules, browser HTTP selection, and Hapke equations.

## Traceable photometry data

photometry.py reads the attached label and nine-band little-endian float payload of [WAC_HAPKEPARAMMAP_566NM.IMG v1.1](https://pds.lroc.im-ldi.com/data/LRO-L-LROC-5-RDR-V1.0/LROLRC_2001/DATA/SDP/WAC_HAPKEPARAMMAP/WAC_HAPKEPARAMMAP_566NM.IMG). The mapped domain is 70 degrees south to 70 degrees north at one degree per tile. The common parameters are cos(latitude)-weighted means of 50,400 valid tiles, not a global photometric inversion.

They are w=0.3261142672, b=0.2348928321, c=0.3574639778, Bs0=1.7707997700, hs=0.0622193166, theta=23.6566009521 degrees, Bc0=0, hc=1, phi=0, at 566 nm. The script reproduces these means and checks full-Moon limb response, opposition brightening, a dark facet, and 1,296 geometries.

Equation source actually read: USGS ISIS3 commit fadd004da2f733525209ee4f66ffdb3a9a9d193e, isis/src/lro/apps/lrowacphomap/HapkeLROC.cpp (HGfunc and photometry). HapkeLRO.cpp and lrowacphomap.cpp in that same directory were read to check conventions and dispatch. Positive c weights the backward lobe by (1+c)/2. The logarithmic H approximation, SHOE, both roughness branches, and the coherent-backscatter formula are documented in the guide. Bc0=0 and phi=0 make coherent backscatter and porosity multipliers identically one in this map set. The reference paper is Sato et al. 2014, DOI 10.1002/2013JE004580; direct Wiley access was denied, so verification used these open sources and the actual map rather than claiming to have read that appendix.
