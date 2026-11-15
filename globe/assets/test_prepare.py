"""Small analytic tests of the production offline normal generator."""
import unittest

import numpy as np

from prepare import normals


class NormalTests(unittest.TestCase):
    def test_flat_and_metric_slope(self):
        w,h = 32,16
        height = np.zeros((h,w),dtype=np.float32)
        flat = normals(height).astype(float)/255*2-1
        lat = np.pi/2-(np.arange(h)+0.5)*np.pi/h
        lon = ((np.arange(w)+0.5)/w-0.5)*2*np.pi
        radial = np.stack(np.broadcast_arrays(np.cos(lat[:,None])*np.sin(lon),np.sin(lat[:,None]),np.cos(lat[:,None])*np.cos(lon)),axis=-1)
        self.assertLess(np.max(np.abs(flat-radial)),0.004)
        y,x = 8,16
        height[y,x+1] = 20_000
        tilted = normals(height).astype(float)/255*2-1
        slope = 20_000/(2*2*np.pi/w*1_737_400*np.cos(lat[y]))
        east = np.array([np.cos(lon[x]),0,-np.sin(lon[x])])
        expected = radial[y,x]-slope*east
        expected /= np.linalg.norm(expected)
        self.assertLess(np.max(np.abs(tilted[y,x]-expected)),0.004)
        height[y,0] = 20_000
        self.assertFalse(np.array_equal(normals(height)[y,-1],normals(np.zeros_like(height))[y,-1]))

    def test_empty_and_invalid(self):
        for height in [np.empty((0,0)),np.zeros((2,2)),np.full((3,3),np.nan)]:
            with self.assertRaises(ValueError):
                normals(height)


if __name__ == "__main__":
    unittest.main()
