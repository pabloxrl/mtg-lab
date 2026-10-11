"""Played combat acceptance through the delivered native reference client."""
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import priority_reference


class CombatTests(unittest.TestCase):
    def test_played_client_reaches_combat_damage(self):
        # CR 510.2: combat damage is dealt simultaneously, before priority.
        # The delivered baseline ends after casting creatures. This assertion
        # exposes the missing played combat consumer without an import failure.
        with tempfile.TemporaryDirectory() as folder:
            result = priority_reference.native(Path(folder))
        self.assertTrue(result['checkpoints'])
        for points in result['checkpoints'].values():
            self.assertTrue(points)
            self.assertEqual(points[-1]['step'], 'combat_damage')


if __name__ == '__main__':
    unittest.main()
