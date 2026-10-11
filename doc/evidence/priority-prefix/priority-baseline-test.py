"""Played prefixes must advance beyond the delivered London opening client."""
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import mulligan_reference as reference


class PriorityTests(unittest.TestCase):
    def test_real_client_reaches_played_priority(self):
        # CR 117.3a/117.4: explicit consecutive upkeep passes advance play.
        # This baseline demonstrates the existing consumer stops too early.
        with tempfile.TemporaryDirectory() as folder:
            actual = reference.native(Path(folder))
        self.assertTrue(actual['checkpoints'])
        for points in actual['checkpoints'].values():
            self.assertTrue(points)
            self.assertEqual(points[-1]['boundary'], 'second_creature_resolved')


if __name__ == '__main__':
    unittest.main()
