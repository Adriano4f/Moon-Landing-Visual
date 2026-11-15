import unittest
import numpy as np
from polar_normals import normals


class PolarNormals(unittest.TestCase):
    def test_flat_sphere_pole_and_orientation(self):
        n = normals(np.zeros((33, 33))).astype(float)/127.5-1
        self.assertLess(n[16, 16, 1], -0.99)
        self.assertGreater(n[16, -1, 0], 0)
        self.assertGreater(n[0, 16, 2], 0)
        self.assertTrue(np.all(np.abs(np.linalg.norm(n, axis=-1)-1) < 0.007))

    def test_missing_cells_use_radial_not_invalid_normals(self):
        n = normals(np.full((5, 5), np.nan))
        self.assertEqual(n[2, 2].tolist(), [128, 0, 128])

    def test_bad_dimensions_and_extent(self):
        for grid in [np.empty((0, 0)), np.zeros(4), np.zeros((2, 2))]:
            with self.assertRaises(ValueError): normals(grid)
        with self.assertRaises(ValueError): normals(np.zeros((5, 5)), -1)


if __name__ == '__main__':
    unittest.main()
