"""Real scalar client acceptance; parser probes alone cannot pass this module."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import priority_reference as reference


class PriorityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.doc = json.loads(reference.FIXTURE.read_text())
        cls.folder = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.folder.cleanup)
        cls.actual = reference.native(Path(cls.folder.name))

    def test_real_client_reaches_played_priority(self):
        reference.check_run(self.actual, self.doc, 'native')
        self.assertEqual(len(self.actual['checkpoints']), 6)
        self.assertGreater(sum(map(len, self.actual['checkpoints'].values())), 700)
        for points in self.actual['checkpoints'].values():
            self.assertTrue(points)
            self.assertEqual(points[-1]['boundary'], 'second_creature_resolved')

    def test_independent_final_zone_ledger_and_distinct_copies(self):
        final = json.loads((reference.ROOT / 'fixtures/reference/full-pool-priority-final.json').read_text())
        for name, points in self.actual['checkpoints'].items():
            self.assertEqual(points[-1], final[name])
            stack_sources = {e['source'] for p in points for e in p['stack']}
            self.assertEqual(len(stack_sources), 4)
            self.assertTrue(stack_sources.issubset(points[-1]['battlefield']))

    def test_runtime_negative_controls_preserve_state(self):
        self.assertEqual(len(self.actual['rejections']), 20)
        self.assertEqual(self.actual['rejection_state_rng'], 'unchanged')
        for result in self.actual['runs'].values():
            self.assertGreater(result['stale_candidates_rejected'], 100)
            self.assertEqual(result['stale_candidates_rejected'], len(result['records']))

    def test_first_field_comparator_controls(self):
        controls = reference.comparator_controls(self.actual['checkpoints'])
        self.assertGreaterEqual(len(controls), 20)
        for name in ('first-draw', 'first-priority', 'stack-action', 'stack-incarnation',
                     'source-incarnation', 'stack-order-synthetic-from-witnessed-entries'):
            self.assertIn(name, controls)

    def test_missing_and_extra_observation_fields_fail(self):
        for mutate in (lambda p: p.pop('mana'), lambda p: p.__setitem__('guessed', 0)):
            bad = copy.deepcopy(self.actual['checkpoints'])
            mutate(next(iter(bad.values()))[0])
            with self.assertRaises(ValueError):
                reference.compare(bad)


if __name__ == '__main__':
    unittest.main()
