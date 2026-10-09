"""Rules-authored source ordering and strict checkpoint negative controls."""
import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import m2_trigger_reference as reference
import cleanup_reference
import m2_trigger_pack
import tempfile


class M2TriggerReference(unittest.TestCase):
    def test_literal_source_order_and_holdout(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        expected = json.loads(reference.EXPECTED.read_text())
        self.assertEqual(set(expected), {c['id'] for c in fixture['cases']})
        self.assertTrue(all(c['setup'] == 'synthetic' for c in fixture['cases']))
        self.assertEqual(expected['apnap_p0'][0]['stack'], [
            'trigger:p0-archer-1', 'trigger:p0-archer-0',
            'trigger:p1-archer-0', 'trigger:p1-archer-1'])
        holdout = expected['holdout_departed_cyclops']
        self.assertEqual([p['life'] for p in holdout],
                         [[20, 20], [20, 19], [20, 19], [20, 18], [20, 18]])
        self.assertTrue(all(p['creatures']['cyclops'] is None for p in holdout))
        self.assertEqual(holdout[-1]['stack'], [])
        reference.compare(expected)
        for name, points in expected.items():
            for row, point in enumerate(points):
                for field in point:
                    wrong = copy.deepcopy(expected)
                    wrong[name][row][field] = 99
                    with self.subTest(name=name, row=row, field=field), self.assertRaises(ValueError):
                        reference.compare(wrong)
            wrong = copy.deepcopy(expected)
            del wrong[name]
            with self.assertRaises(ValueError):
                reference.compare(wrong)
        for wrong in reference.mutations(expected).values():
            with self.assertRaises(ValueError):
                reference.compare(wrong)

    def test_skipped_cleanup_checkpoint_is_detected(self):
        actual = json.loads(cleanup_reference.EXPECTED.read_text())
        # CR 514.3a: trigger resolution requires another cleanup before upkeep.
        del actual['rules-continuous-hand-size-cleanup-interaction'][-1]
        with self.assertRaisesRegex(ValueError, 'first divergence'):
            cleanup_reference.compare(actual)

    def test_exact_assignment_and_minimized_controls(self):
        rows = m2_trigger_pack.assignments()
        self.assertEqual(len(rows), 38)
        with tempfile.TemporaryDirectory() as folder:
            controls = m2_trigger_pack.minimized_controls(Path(folder))
            self.assertEqual({c['name'] for c in controls}, {'source-order', 'skipped-cleanup'})
            for control in controls:
                artifact = json.loads((Path(folder) / control['artifact']).read_text())
                self.assertTrue(artifact['first_divergence'])
