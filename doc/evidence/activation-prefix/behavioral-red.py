"""Played activation acceptance; expected behavior follows CR 602/605 and Oracle."""
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import priority_reference as reference


class ActivationTests(unittest.TestCase):
    def test_real_played_activation_observations(self):
        with tempfile.TemporaryDirectory() as folder:
            actual = reference.native(Path(folder))
        points = [p for rows in actual['checkpoints'].values() for p in rows]
        self.assertGreater(len(points), 0)
        self.assertTrue(any(p.get('activation') is not None for p in points),
                        'played consumer must witness activation payment, not just creature casting')


if __name__ == '__main__':
    unittest.main()
