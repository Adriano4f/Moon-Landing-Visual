"""Derive a 512 by 256 first frame from the existing 2k scientific-display assets."""
from pathlib import Path
import numpy as np
from PIL import Image


def average(image):
    values = np.asarray(image, dtype=np.float64) / 255.0
    if values.shape != (1024, 2048, 3) or not np.isfinite(values).all():
        raise ValueError('Expected a finite 2k RGB image')
    return values.reshape(256, 4, 512, 4, 3).mean(axis=(1, 3))


def normal_preview(image):
    vectors = average(image) * 2.0 - 1.0
    lengths = np.linalg.norm(vectors, axis=2, keepdims=True)
    if np.any(lengths < 1e-6):
        raise ValueError('Degenerate mean normal')
    return Image.fromarray(np.rint((vectors / lengths + 1.0) * 127.5).astype(np.uint8))


def main():
    directory = Path(__file__).parent
    source = np.asarray(Image.open(directory/'albedo_2k.jpg'), dtype=np.float64)/255.0
    linear = np.where(source <= 0.04045, source/12.92, ((source+0.055)/1.055)**2.4)
    mean = linear.reshape(256, 4, 512, 4, 3).mean(axis=(1, 3))
    srgb = np.where(mean <= 0.0031308, mean*12.92, 1.055*mean**(1/2.4)-0.055)
    Image.fromarray(np.rint(srgb*255).astype(np.uint8)).save(directory/'albedo_preview.jpg', quality=87, subsampling=0, optimize=True)
    normal_preview(Image.open(directory/'normal_2k.png')).save(directory/'normal_preview.webp', lossless=True, method=6)
    for name in ['albedo_preview.jpg', 'normal_preview.webp']:
        print(name, (directory/name).stat().st_size)


if __name__ == '__main__':
    main()
