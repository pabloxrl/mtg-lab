"""CR 103: real reset observations, occurrence swaps and strict negative controls."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import full_pool_reference as ref


class ResetTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = json.loads(ref.FIXTURE.read_text())
        cls.expected = json.loads(ref.EXPECTED.read_text())
        with tempfile.TemporaryDirectory() as folder:
            cls.actual = ref.native(Path(folder))

    def test_real_reset_all_rows_and_repeat(self):
        self.assertEqual(len(self.actual['checkpoints']), 16)
        ref.compare(self.actual['checkpoints'])
        self.assertEqual(self.actual['repeat_checkpoints'], self.actual['checkpoints'])
        self.assertEqual(self.actual['stale_handles_rejected'], 1280)

    def test_same_name_swaps_are_observed(self):
        for name, point in self.actual['checkpoints'].items():
            if name.endswith('-swap'):
                self.assertNotEqual(point['library'], self.actual['checkpoints'][name[:-5]]['library'])
                self.assertEqual(point['life'], [20, 20])
                self.assertEqual(point['completion'], 'prefix')

    def test_native_negative_controls_executed(self):
        self.assertEqual(set(self.actual['rejections']), set(ref.negative_inputs(self.fixture)))
        self.assertEqual(len(self.actual['rejections']), 19)
        self.assertTrue(all('first divergence:' in v for v in self.actual['rejections'].values()))

    def test_swapped_checkpoint_is_rejected(self):
        bad = copy.deepcopy(self.actual['checkpoints'])
        row = bad['red-green-0']['library'][0]
        row[-1], row[-2] = row[-2], row[-1]  # two Mountains, unchanged counts
        with self.assertRaisesRegex(ValueError, 'first divergence:'):
            ref.compare(bad)

    def test_every_observed_field_is_required(self):
        for field in self.actual['checkpoints']['red-green-0']:
            bad = copy.deepcopy(self.actual['checkpoints'])
            del bad['red-green-0'][field]
            with self.assertRaisesRegex(ValueError, 'first divergence:'):
                ref.compare(bad)

    def test_input_controls_have_independent_rejections(self):
        for name, doc in ref.negative_inputs(self.fixture).items():
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'first divergence:'):
                ref.validate(doc)
