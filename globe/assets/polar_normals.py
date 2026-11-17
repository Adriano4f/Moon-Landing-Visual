"""Bake object-space normals from the unrendered LOLA south-polar height service."""
from pathlib import Path
from io import BytesIO
import json
import hashlib
import numpy as np
from PIL import Image
import requests
from heights import save as save_heights

RADIUS = 1737400.0
EXTENT = 547965.0
SERVICE = 'https://trek.nasa.gov/moon/trekarcgis/rest/services/LRO_LOLA_DEM_SPole75_30mp_v04/ImageServer'


def normals(heights, extent=EXTENT):
    if heights.ndim != 2 or min(heights.shape) < 3 or not np.isfinite(extent) or extent <= 0:
        raise ValueError('Expected a height grid at least three pixels across and positive extent')
    height, width = heights.shape
    x, y = np.meshgrid((np.arange(width)+0.5)*2*extent/width-extent,
                       extent-(np.arange(height)+0.5)*2*extent/height)
    rho = np.hypot(x, y)
    latitude = -np.pi/2 + 2*np.arctan(rho/(2*RADIUS))
    longitude = np.arctan2(x, y)
    up = np.stack((np.cos(latitude)*np.sin(longitude), np.sin(latitude),
                   np.cos(latitude)*np.cos(longitude)), axis=-1)
    valid = np.isfinite(heights) & (np.abs(heights) < 15000)
    position = up*(RADIUS+np.where(valid, heights, 0))[..., None]
    dy, dx = np.gradient(position, axis=(0, 1))
    normal = np.cross(dy, dx)
    normal /= np.linalg.norm(normal, axis=-1)[..., None]
    normal *= np.where(np.sum(normal*up, axis=-1) < 0, -1, 1)[..., None]
    normal[~valid] = up[~valid]
    return np.round((normal+1)*127.5).clip(0, 255).astype(np.uint8)


def load_heights(cache):
    if not cache.exists():
        params = dict(f='image', bbox=f'{-EXTENT},{-EXTENT},{EXTENT},{EXTENT}',
                      size='2048,2048', format='tiff', pixelType='F32', compression='none',
                      renderingRule='{"rasterFunction":"None"}', interpolation='RSP_BilinearInterpolation')
        response = requests.get(SERVICE+'/exportImage', params=params, timeout=180)
        response.raise_for_status()
        image = Image.open(BytesIO(response.content))
        if image.mode != 'F' or image.size != (2048, 2048):
            raise ValueError('Service did not return the requested float height grid')
        cache.write_bytes(response.content)
        cache.with_suffix('.json').write_text(json.dumps(dict(url=response.url, bytes=len(response.content)), indent=2))
    return np.asarray(Image.open(cache), dtype=np.float64)


def prepare(cache, output):
    heights = load_heights(cache)
    for tier, grid in [(2, heights), (1, heights.reshape(1024, 2, 1024, 2).mean(axis=(1, 3)))]:
        save_heights(grid, f'polar_height_{tier}k.png', SERVICE, 2 * EXTENT / grid.shape[0])
        image = Image.fromarray(normals(grid))
        image.save(output/f'polar_normal_{tier}k.png', optimize=True)
        image.save(output/f'polar_normal_{tier}k.webp', lossless=True, method=6)
        print(tier, 'normal bytes', (output/f'polar_normal_{tier}k.webp').stat().st_size)
    metadata = dict(source=SERVICE, source_sha256=hashlib.sha256(cache.read_bytes()).hexdigest(),
                    radius_m=RADIUS, extent_m=EXTENT, projection='IAU2000:30120',
                    source_spacing_m=30, exported_spacing_m=2*EXTENT/2048,
                    pixel_centers=True, height_units='meters',
                    missing_height_policy='radial normal',
                    normal_encoding='RGB8 object-space unit vector; round((n+1)*127.5)')
    (output/'polar_normals.meta.json').write_text(json.dumps(metadata, indent=2)+'\n')


if __name__ == '__main__':
    import sys
    prepare(Path(sys.argv[1]), Path(__file__).parent)
