"""The literal token comparator must reject each observable corruption."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import token_reference


class TokenReferenceTests(unittest.TestCase):
    def test_missing_cases_and_mutated_token_checkpoints_fail(self):
        expected = json.loads((token_reference.ROOT / 'fixtures/reference/token-expectations.json').read_text())
        token_reference.compare(expected)
        for mode in expected:
            wrong = copy.deepcopy(expected)
            del wrong[mode]
            with self.assertRaisesRegex(ValueError, 'token checkpoint mismatch'):
                token_reference.compare(wrong)
            for field, value in [('tokens', 0), ('unique_created', 1), ('life', [0, 0]),
                                 ('pending_zero', False), ('one_one', False), ('grave_tokens', 1)]:
                wrong = copy.deepcopy(expected)
                wrong[mode][field] = value
                with self.assertRaisesRegex(ValueError, 'token checkpoint mismatch'):
                    token_reference.compare(wrong)
