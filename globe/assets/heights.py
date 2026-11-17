"""Lossless metric elevations: high/low bytes in RG, validity in B, no color profile."""
import hashlib
import json
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
SCALE = 0.5
OFFSET = -10000.0


def packed(heights):
    if heights.ndim != 2 or not heights.size:
        raise ValueError('Expected a nonempty height grid')
    valid = np.isfinite(heights) & (heights >= OFFSET) & (heights <= OFFSET + 65535 * SCALE)
    values = np.rint((np.where(valid, heights, OFFSET) - OFFSET) / SCALE).astype(np.uint16)
    return np.stack((values >> 8, values & 255, valid * 255), axis=-1).astype(np.uint8)


def save(heights, name, source, spacing):
    path = ROOT / name
    Image.fromarray(packed(heights)).save(path, optimize=True)
    manifest_path = ROOT / 'heights.manifest.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    manifest[name] = dict(width=heights.shape[1], height=heights.shape[0],
                          scale_m=SCALE, offset_m=OFFSET, spacing_m=spacing,
                          encoding='RGB8: R high byte, G low byte, B validity; linear, no mipmaps',
                          source=source, bytes=path.stat().st_size,
                          sha256=hashlib.sha256(path.read_bytes()).hexdigest())
    manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    print(name, path.stat().st_size, flush=True)


if __name__ == '__main__':
    import sys
    from prepare import source
    from polar_normals import load_heights, SERVICE, EXTENT

    dem = np.asarray(source('ldem_16_uint.tif'), dtype=np.float32) * SCALE + OFFSET
    for tier in (2, 4):
        size = (tier * 1024, tier * 512)
        grid = np.asarray(Image.fromarray(dem).resize(size, Image.Resampling.BILINEAR))
        save(grid, f'height_{tier}k.png', 'NASA CGI Moon Kit ldem_16_uint.tif',
             2 * np.pi * 1737400 / size[0])
    polar = load_heights(Path(sys.argv[1]))
    for tier, grid in [(2, polar), (1, polar.reshape(1024, 2, 1024, 2).mean(axis=(1, 3)))]:
        save(grid, f'polar_height_{tier}k.png', SERVICE, 2 * EXTENT / grid.shape[0])
