"""Analytic controls for the spherical height-field solar visibility march."""
import json
import math
from pathlib import Path
import unittest

import numpy as np
from PIL import Image

R = 1737400.0
SUN_RADIUS = math.radians(0.265)


def visibility(height, elevation, steps=48):
    sun = np.array([math.cos(elevation), 0, math.sin(elevation)])
    origin = np.array([0, 0, R + height(np.array([0, 0, 1])) + 3])
    along = origin @ sun
    if along < 0 and origin @ origin - along * along < (R - 10000)**2:
        return 0
    distance = 267.56
    growth = (600000 / distance)**(1 / (steps - 1))
    angle = 1
    for _ in range(steps):
        ray = origin + distance * sun
        radius = np.linalg.norm(ray)
        if radius > R + 12000 and ray @ sun > 0:
            break
        angle = min(angle, (radius - R - height(ray / radius)) / distance)
        if angle <= -SUN_RADIUS:
            return 0
        distance *= growth
    x = np.clip(angle / SUN_RADIUS, -1, 1)
    return (math.acos(-x) + x * math.sqrt(max(0, 1-x*x))) / math.pi


class Shadows(unittest.TestCase):
    def test_flat_surface_and_deep_night(self):
        for steps in (32, 48):
            self.assertGreater(visibility(lambda _: 0, math.radians(10), steps), 0.999)
            self.assertEqual(visibility(lambda _: 0, math.radians(-20), steps), 0)
            self.assertEqual(visibility(lambda _: 0, math.radians(-2), steps), 0)

    def test_rim_and_soft_solar_disk(self):
        def ridge(p):
            return 1800 if 8000 < R * p[0] < 17000 else 0
        self.assertEqual(visibility(ridge, math.radians(1)), 0)
        self.assertGreater(visibility(ridge, math.radians(20)), 0.999)
        values = [visibility(ridge, math.radians(e)) for e in np.linspace(6, 14, 321)]
        self.assertTrue(any(0.05 < v < 0.95 for v in values))
        self.assertTrue(all(a <= b + 1e-6 for a, b in zip(values, values[1:])))

    def test_elevated_peak_beyond_reference_terminator(self):
        def peak(p):
            return 4000 if abs(R*p[0]) < 100 else 0
        self.assertGreater(visibility(peak, math.radians(-1)), 0.999)
        self.assertEqual(visibility(peak, math.radians(-10)), 0)

    def test_asset_integrity_encoding_and_height_bound(self):
        import hashlib
        root = Path(__file__).parent
        manifest = json.loads((root/'heights.manifest.json').read_text())
        self.assertEqual(len(manifest), 4)
        for name, entry in manifest.items():
            data = (root/name).read_bytes()
            self.assertEqual(hashlib.sha256(data).hexdigest(), entry['sha256'])
            self.assertEqual(len(data), entry['bytes'])
            image = Image.open(root/name)
            self.assertEqual(image.size, (entry['width'], entry['height']))
            self.assertFalse(any(k in image.info for k in ['icc_profile', 'srgb', 'gamma']))
            pixels = np.asarray(image)
            h = (pixels[..., 0].astype(float)*256 + pixels[..., 1])*entry['scale_m'] + entry['offset_m']
            self.assertLess(np.max(h[pixels[..., 2] == 255]), 12000)
