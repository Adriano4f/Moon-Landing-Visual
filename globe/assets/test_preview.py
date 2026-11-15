import unittest
import numpy as np
from PIL import Image
from preview import average, normal_preview


class PreviewTests(unittest.TestCase):
    def test_normal_shape_unit_length_and_constant(self):
        image = Image.new('RGB', (2048, 1024), (128, 128, 255))
        result = normal_preview(image)
        self.assertEqual(result.size, (512, 256))
        vectors = np.asarray(result, dtype=float)/127.5-1
        self.assertLess(np.abs(np.linalg.norm(vectors, axis=2)-1).max(), 0.007)
        self.assertEqual(result.getpixel((30, 30)), (128, 128, 255))

    def test_empty_invalid_and_cancelled_normals(self):
        for size in [(0, 0), (5, 5)]:
            with self.assertRaises(ValueError):
                average(Image.new('RGB', size))
        values = np.zeros((1024, 2048, 3), dtype=np.uint8)
        values[::2] = 255
        with self.assertRaises(ValueError):
            normal_preview(Image.fromarray(values))


if __name__ == '__main__':
    unittest.main()
