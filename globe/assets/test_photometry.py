"""Regression checks for the display-normal correction and exposure choice."""
import math
from pathlib import Path
import unittest

import numpy as np

from photometry import front_facing_normal, reflectance


class ShadingTests(unittest.TestCase):
    def test_front_facing_unchanged_and_grazing_grid(self):
        view = np.array([0.,0.,1.])
        front = np.array([0.5,0.,math.sqrt(0.75)])
        np.testing.assert_allclose(front_facing_normal(front,view),front,atol=1e-12)
        for mu in np.linspace(-1,1,201):
            normal = np.array([math.sqrt(max(0,1-mu*mu)),0,mu])
            fixed = front_facing_normal(normal,view)
            self.assertTrue(np.all(np.isfinite(fixed)))
            self.assertAlmostEqual(np.linalg.norm(fixed),1)
            self.assertGreater(float(fixed@view),0.00999)
        shader = Path(__file__).resolve().parents[1]/"src"/"moon.frag"
        source = shader.read_text()
        self.assertIn("const float MIN_EMISSION_COSINE = 0.01;",source)
        self.assertIn("n = normalize(n + max(0.0, MIN_EMISSION_COSINE - dot(n, view)) * view);",source)

    def test_lit_back_facing_facet_and_exposure(self):
        view = np.array([0.,0.,1.])
        light = np.array([1.,0.2,0.6]);light /= np.linalg.norm(light)
        normal = np.array([1.,0.,-0.02]);normal /= np.linalg.norm(normal)
        phase = math.degrees(math.acos(float(light@view)))
        self.assertEqual(reflectance(float(normal@light),float(normal@view),phase),0)
        fixed = front_facing_normal(normal,view)
        self.assertGreater(reflectance(float(fixed@light),float(fixed@view),phase),0)
        def display(exposure):
            x = exposure*0.6/0.4832*0.08
            return 1.055*(x/(1+x))**(1/2.4)-0.055
        self.assertGreater(display(8),display(4))
        self.assertTrue(0.69 < display(8) < 0.71)
        self.assertIn("const float EXPOSURE = 8.0;",(Path(__file__).resolve().parents[1]/"src"/"moon.frag").read_text())

    def test_empty_invalid_and_opposite(self):
        for normal in [[],[0,0,0],[math.nan,0,1]]:
            with self.assertRaises(ValueError):
                front_facing_normal(normal,[0,0,1])
        with self.assertRaises(ValueError):
            front_facing_normal([0,0,1],[0,0,1],0)
        np.testing.assert_allclose(front_facing_normal([0,0,-1],[0,0,1]),[0,0,1])

    def test_grazing_response_crossfade(self):
        def fade(mu):
            t = max(0,min(1,(mu-0.35)/0.30))
            return t*t*(3-2*t)
        self.assertEqual(fade(0),0)
        self.assertEqual(fade(0.35),0)
        self.assertAlmostEqual(fade(0.5),0.5)
        self.assertEqual(fade(0.65),1)
        self.assertGreater((1-fade(0.5))*0.08+fade(0.5)*0,0)
        shader = (Path(__file__).resolve().parents[1]/"src"/"moon.frag").read_text()
        self.assertIn("smoothstep(0.35, 0.65, max(dot(geometric, view), 0.0))",shader)
        self.assertIn("mix(geometric_reflectance, mapped_reflectance, normal_strength)",shader)


if __name__ == "__main__":
    unittest.main()
