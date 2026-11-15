"""Reproduce tier assets. Requires Python, Pillow and numpy; no runtime Python dependency."""
import hashlib
import json
import pathlib
import urllib.request

import numpy as np
from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parent
BASE = "https://svs.gsfc.nasa.gov/vis/a000000/a004700/a004720/"
Image.MAX_IMAGE_PIXELS = None


def source(name):
    cache = pathlib.Path.home() / "plan" / name
    cache.parent.mkdir(parents=True, exist_ok=True)
    if not cache.exists():
        urllib.request.urlretrieve(BASE + name, cache)
    print(name, cache.stat().st_size, flush=True)
    return Image.open(cache)


def normals(height):
    if height.ndim != 2 or min(height.shape) < 3 or not np.all(np.isfinite(height)):
        raise ValueError("Expected a finite height grid at least 3 by 3")
    h, w = height.shape
    lat = (np.pi / 2 - (np.arange(h) + 0.5) * np.pi / h)[:, None]
    lon = ((np.arange(w) + 0.5) / w - 0.5)[None, :] * 2 * np.pi
    radius = 1_737_400.0 + height
    east = (np.roll(height, -1, 1) - np.roll(height, 1, 1)) / (4 * np.pi / w * radius * np.cos(lat))
    north = -np.gradient(height, axis=0) / (np.pi / h * radius)
    # Taper the last degree to a radial polar cap; longitude is singular at the pole.
    taper = np.minimum(1.0, np.cos(lat) / np.sin(np.pi / 180))
    east *= taper
    north *= taper
    x = np.cos(lat) * np.sin(lon) - east * np.cos(lon) + north * np.sin(lat) * np.sin(lon)
    y = np.broadcast_to(np.sin(lat), (h, w)) - north * np.cos(lat)
    z = np.cos(lat) * np.cos(lon) + east * np.sin(lon) + north * np.sin(lat) * np.cos(lon)
    length = np.sqrt(x*x + y*y + z*z)
    return np.stack([np.rint((a / length * 0.5 + 0.5) * 255) for a in (x, y, z)], axis=-1).astype(np.uint8)


if __name__ == "__main__":
    color = source("lroc_color_16bit_srgb_8k.tif").convert("RGB")
    height_source = source("ldem_16_uint.tif")
    assert color.size == (8192, 4096) and height_source.size == (5760, 2880)
    assert height_source.mode == "I;16"
    dem = np.asarray(height_source, dtype=np.float32) * 0.5 - 10_000.0
    manifest = {}
    for tier, width in [(2, 2048), (4, 4096), (8, 8192)]:
        size = (width, width // 2)
        image = color.resize(size, Image.Resampling.LANCZOS)
        image.save(ROOT / f"albedo_{tier}k.jpg", quality=94, subsampling=0)
        heights = np.asarray(Image.fromarray(dem).resize(size, Image.Resampling.BILINEAR))
        Image.fromarray(normals(heights)).save(ROOT / f"normal_{tier}k.png", compress_level=9)
        # Solid-angle weighting gives each surface area equal importance, unlike image averaging.
        a = np.asarray(image, dtype=np.float64) / 255
        linear = np.where(a <= 0.04045, a / 12.92, ((a + 0.055) / 1.055)**2.4)
        weights = np.cos((np.arange(size[1]) + 0.5) * np.pi / size[1] - np.pi/2)
        mean = float(np.sum((linear @ [0.2126, 0.7152, 0.0722]) * weights[:, None]) / (weights.sum() * width))
        records = {}
        for name in [f"albedo_{tier}k.jpg", f"normal_{tier}k.png"]:
            data = (ROOT / name).read_bytes()
            records[name] = dict(bytes=len(data), sha256=hashlib.sha256(data).hexdigest())
        manifest[f"{tier}k"] = dict(width=width, height=width//2, linear_mean=mean, files=records)
        print(tier, manifest[f"{tier}k"], flush=True)
    (ROOT / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
