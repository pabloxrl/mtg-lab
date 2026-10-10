"""CR 103.5: strict occurrence-aware opening prefixes from real consumers."""
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import full_pool_reference as ref


class MulliganTests(unittest.TestCase):
    def test_real_opening_reaches_first_upkeep(self):
        # Baseline consumer really resets both complete frozen decks. CR 103.5
        # requires explicit declarations and bottoms before first upkeep.
        with tempfile.TemporaryDirectory() as folder:
            actual = ref.native(Path(folder))
        self.assertTrue(actual['checkpoints'])
        for point in actual['checkpoints'].values():
            self.assertEqual(point['boundary'], 'first_upkeep')
            self.assertTrue(point['consumed_choices'])


if __name__ == '__main__':
    unittest.main()
