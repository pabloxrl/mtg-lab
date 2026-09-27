"""Reference comparator must detect each observed combat checkpoint defect."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import combat_reference


class CombatReferenceTests(unittest.TestCase):
    def test_combat_reference_rejects_missing_and_changed_checkpoints(self):
        fixture = json.loads(combat_reference.FIXTURE.read_text())
        expected = {c['id']: c['expected'] for c in fixture['cases']}
        combat_reference.compare(fixture, expected)
        for case in expected:
            wrong = copy.deepcopy(expected)
            del wrong[case]
            with self.assertRaisesRegex(ValueError, 'missing or extra'):
                combat_reference.compare(fixture, wrong)
            for field, value in [('life', [20, 19]), ('battlefield', []),
                                 ('graveyard', [['Swab Goblin'], []])]:
                wrong = copy.deepcopy(expected)
                wrong[case][0][field] = value
                with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
                    combat_reference.compare(fixture, wrong)
        for field, value in [('blocked', False), ('blockers', [])]:
            wrong = copy.deepcopy(expected)
            wrong['modern-split'][2][field] = value
            with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
                combat_reference.compare(fixture, wrong)
        wrong = copy.deepcopy(expected)
        wrong['modern-split'][3]['battlefield'][0]['damage'] = 2
        with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
            combat_reference.compare(fixture, wrong)

        wrong = copy.deepcopy(expected)
        wrong['modern-split'][1]['battlefield'][0]['tapped'] = False
        with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
            combat_reference.compare(fixture, wrong)
