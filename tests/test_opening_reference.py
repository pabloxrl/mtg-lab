"""Reference count-ledger rejection controls; no engine-generated oracle."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import opening_reference as reference


class OpeningReferenceTests(unittest.TestCase):
    def test_missing_extra_and_changed_fields(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        expected = {c['id']: c['expected'] for c in fixture['cases']}
        reference.compare(fixture, expected)
        for name in expected:
            bad = copy.deepcopy(expected)
            del bad[name]
            with self.assertRaisesRegex(ValueError, 'missing or extra'):
                reference.compare(fixture, bad)
            for field in expected[name]:
                bad = copy.deepcopy(expected)
                bad[name][field] = None
                with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
                    reference.compare(fixture, bad)
        bad = copy.deepcopy(expected)
        bad['extra'] = expected[next(iter(expected))]
        with self.assertRaisesRegex(ValueError, 'missing or extra'):
            reference.compare(fixture, bad)

    def test_boolean_count_is_not_numeric_evidence(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        observed = {c['id']: copy.deepcopy(c['expected']) for c in fixture['cases']}
        observed['starter-0-rounds-0']['bottom_choices'] = False
        with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
            reference.compare(fixture, observed)
