"""Independent comparator negative controls for terminal observations."""
import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import terminal_reference


class TerminalReferenceTests(unittest.TestCase):
    def test_missing_extra_and_every_observed_field_rejected(self):
        fixture = json.loads(terminal_reference.FIXTURE.read_text())
        expected = {c['id']: c['expected'] for c in fixture['cases']}
        terminal_reference.compare(fixture, expected)
        for case in expected:
            wrong = copy.deepcopy(expected)
            del wrong[case]
            with self.assertRaisesRegex(ValueError, 'missing or extra'):
                terminal_reference.compare(fixture, wrong)
            for field in ['life', 'lost', 'library', 'hand']:
                for seat in [0, 1]:
                    wrong = copy.deepcopy(expected)
                    value = wrong[case][field][seat]
                    wrong[case][field][seat] = not value if field == 'lost' else value + 1
                    with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
                        terminal_reference.compare(fixture, wrong)
        wrong = copy.deepcopy(expected)
        wrong['extra'] = expected['zero-life']
        with self.assertRaisesRegex(ValueError, 'missing or extra'):
            terminal_reference.compare(fixture, wrong)
