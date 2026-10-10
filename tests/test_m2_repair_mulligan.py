"""CR 103.5: strict occurrence-aware opening prefixes from real consumers."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import mulligan_reference as ref


class MulliganTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(ref.FIXTURE.read_text())
        with tempfile.TemporaryDirectory() as folder:
            cls.actual = ref.native(Path(folder))

    def test_real_opening_reaches_first_upkeep(self):
        self.assertEqual(len(self.actual['checkpoints']), 32)
        for points in self.actual['checkpoints'].values():
            self.assertTrue(points)
            self.assertEqual(points[-1]['boundary'], 'first_upkeep')
            self.assertEqual({p['actor'] for p in points[-2:]}, {0, 1})
        ref.compare(self.actual['checkpoints'])

    def test_every_chance_and_choice_consumed_once(self):
        self.assertEqual(self.actual['runs'], self.actual['repeat_runs'])
        for case in self.fixture['cases']:
            run = self.actual['runs'][case['id']]
            self.assertTrue(run['consumed_choices'])
            self.assertEqual(run['consumed_choices'], case['choices'])
            self.assertEqual(run['consumed_chance'], case['chance'])
            self.assertEqual(run['native_rng'], 'unchanged')

    def test_runtime_rejections_leave_state_and_rng_unchanged(self):
        self.assertEqual(set(self.actual['rejections']), set(ref.negative_inputs(self.fixture)))
        self.assertEqual(len(self.actual['rejections']), 32)
        self.assertEqual(self.actual['rejection_state_rng'], 'unchanged')
        self.assertTrue(all(v.startswith('first divergence:') for v in self.actual['rejections'].values()))

    def test_each_checkpoint_field_is_required(self):
        for field in next(iter(self.actual['checkpoints'].values()))[0]:
            bad = copy.deepcopy(self.actual['checkpoints'])
            del next(iter(bad.values()))[0][field]
            with self.assertRaisesRegex(ValueError, 'first divergence:'):
                ref.compare(bad)

    def test_reordered_bottom_copies_are_detected(self):
        bad = copy.deepcopy(self.actual['checkpoints'])
        last = bad['red-green-0-both'][-2]['library']
        self.assertEqual(last[-1].split('/')[1], last[-2].split('/')[1])
        last[-1], last[-2] = last[-2], last[-1]
        with self.assertRaisesRegex(ValueError, 'first divergence:'):
            ref.compare(bad)

    def test_missing_and_reordered_boundaries_reject(self):
        for reorder in (False, True):
            bad = copy.deepcopy(self.actual['checkpoints'])
            points = bad['red-green-0-unequal']
            if reorder:
                points[1], points[2] = points[2], points[1]
            else:
                points.pop(2)
            with self.assertRaisesRegex(ValueError, 'first divergence:'):
                ref.compare(bad)

    def test_positive_envelope_is_versioned(self):
        ref.validate(self.fixture)
        bad = copy.deepcopy(self.fixture)
        bad['schema_version'] = 1
        with self.assertRaisesRegex(ValueError, 'schema_version'):
            ref.validate(bad)


if __name__ == '__main__':
    unittest.main()
