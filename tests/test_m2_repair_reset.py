"""CR 103: real reset observations, occurrence swaps and strict negative controls."""
import copy
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
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


class LauncherTests(unittest.TestCase):
    def test_standalone_native_uses_repository_shared_lock(self):
        # CI's toolchain image declares a controller root without copying the
        # controller checkout. Probe the real launch path, not an engine stub.
        with tempfile.TemporaryDirectory() as folder, patch.dict(
                os.environ, {'SYMPHONY_CONTROL_ROOT': '/absent-ci-controller'}):
            os.environ.pop('MTG_SYMPHONY_LOCK_FD', None)
            with patch.object(ref.subprocess, 'run',
                              side_effect=RuntimeError('launch probe')) as launch:
                with self.assertRaisesRegex(RuntimeError, 'launch probe'):
                    ref.native(Path(folder))
            command = launch.call_args.args[0]
        self.assertEqual(command[1], str(ref.ROOT / 'scripts/symphony/resource_lock.py'))
        self.assertEqual(command[2:5], ['heavy', '--', 'cargo'])
