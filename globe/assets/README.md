# Moon assets

Credit: **NASA's Scientific Visualization Studio**. Visualization by Ernie Wright, with LROC and LOLA instrument-team source data.

The official [CGI Moon Kit, SVS ID 4720](https://svs.gsfc.nasa.gov/4720/) describes the product geometry, vertical units, and limitations. Its maps are equirectangular, centered on longitude zero, with north at the top. The left edge is longitude -180 degrees and east-positive longitude increases to the right.

## Committed files

`lroc_color_2k.jpg` is the December 2025 LROC color-map preview, downloaded without modification from:

https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/lroc_color_2k.jpg

It has 2,048 by 1,024 pixels, RGB8 channels, and 457,942 bytes. Its SHA-256 is `f7130a1822681fa7512d7dcfd40db8c10b9ba4f06777910348698260ed7a2170`. The unit is dimensionless display color, not measured absolute albedo. We interpret this display JPEG as sRGB. NASA explicitly calls these aesthetic rendering maps rather than scientific products. The corresponding 2025 TIFF family is explicitly sRGB encoded; the float EXR alternative is linear. The color bands represent 643 nm, 566 nm, and 415 nm, with exposure and white balance adjustments. The polar fill comes from lower-resolution monochromatic LOLA albedo.

`ldem_4_uint.png` is a lossless, single-channel unsigned 16-bit PNG conversion of:

https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_4_uint.tif

The original TIFF is 1,440 by 720 pixels, four pixels per degree, and 2,076,866 bytes. The committed PNG is 1,643,582 bytes, with SHA-256 `60298cf57a2ea076ae6c8e9f83b6ac2b9e71c8494f4135a195b71006f1f90a45`. No resampling, normalization, or bit-depth reduction is performed. PNG byte order is handled by the decoder.

The unsigned sample q is a radius offset in half-meters above 1,727,400 m. NASA obtained this encoding by adding 20,000 to signed half-meter heights relative to 1,737,400 m. Therefore the elevation used by the renderer is `h_m = 0.5 * q - 10,000`, relative to 1,737,400 m. This is not the floating-point kilometer TIFF product. The observed sample range is 2,243 to 41,008, giving elevations -8,878.5 m to 10,504 m. CPU normal generation uses an exaggeration of 2.0, but the sphere silhouette remains undisplaced.

## Reproduction

Download the JPEG from its URL. Run `python3 globe/assets/convert_height.py` from any working directory to regenerate the PNG with Python 3 and Pillow. The script downloads the TIFF into memory and preserves the unsigned 16-bit samples. PNG compression bytes can differ between Pillow versions, even when decoded samples agree. Neither Pillow nor network access is needed to build or run the Rust app. The generated normal map is computed at startup and is not committed.

Both committed assets are well below 100 MB. The existing `*.tif` ignore rule remains unchanged. These rendering assets are deliberately not connected to the terrain-horizon DEM reader, which is still unimplemented.
