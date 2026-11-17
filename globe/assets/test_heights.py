import unittest
import numpy as np
from heights import packed, SCALE, OFFSET


class Heights(unittest.TestCase):
    def test_round_trip_and_byte_carries(self):
        grid = np.array([[-10000, -9872.5, -9872, -1.1, 0, 12345.6]])
        image = packed(grid)
        decoded = (image[..., 0].astype(float) * 256 + image[..., 1]) * SCALE + OFFSET
        self.assertLessEqual(np.max(np.abs(decoded - grid)), SCALE / 2)
        self.assertTrue(np.all(image[..., 2] == 255))

    def test_empty_and_missing(self):
        for grid in [np.empty((0, 0)), np.zeros(3)]:
            with self.assertRaises(ValueError): packed(grid)
        self.assertTrue(np.all(packed(np.array([[np.nan, np.inf, -20000]]))[..., 2] == 0))
