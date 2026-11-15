"""Download NASA's original unsigned LOLA TIFF and losslessly encode its samples as PNG.

Run with Python 3 and Pillow: python3 globe/assets/convert_height.py
The source TIFF stays in memory, so it is never accidentally committed.
"""
import io
import pathlib
import urllib.request

from PIL import Image

URL = "https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/ldem_4_uint.tif"
with urllib.request.urlopen(URL, timeout=120) as response:
    source = Image.open(io.BytesIO(response.read()))
    assert source.size == (1440, 720) and source.mode == "I;16"
    source.save(pathlib.Path(__file__).with_name("ldem_4_uint.png"))
