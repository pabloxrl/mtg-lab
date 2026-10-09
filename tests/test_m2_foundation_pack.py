"""Coverage cannot be certified by dropping cases or ignoring corrupt checkpoints."""
import copy
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import m2_foundation_pack as pack


class M2FoundationPack(unittest.TestCase):
    def test_exact_assignment_rejects_missing_changed_and_unexecuted_cases(self):
        rows = pack.load(pack.ASSIGNMENT)
        pack.validate_catalog(rows)
        self.assertEqual(len(rows), 31)
        for mutation in ('missing', 'duplicate', 'oracle', 'owner', 'native', 'reference', 'omit_reference'):
            wrong = copy.deepcopy(rows)
            if mutation == 'missing':
                wrong.pop()
            elif mutation == 'duplicate':
                wrong.append(wrong[0])
            elif mutation == 'oracle':
                wrong[0]['catalog']['expected'] = 'accept native output'
            elif mutation == 'owner':
                wrong[0]['catalog']['owner_issue'] = 212
            elif mutation == 'native':
                wrong[0]['native_tests'] = ['absent_test']
            elif mutation == 'reference':
                wrong[0]['reference_cases'] = ['absent_reference_case']
            else:
                wrong[0]['pack'] = 'native'
                wrong[0]['reference_cases'] = []
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                pack.validate_catalog(wrong)

    def test_holdout_literal_oracle_and_corrupted_checkpoint(self):
        original = pack.expected_holdout()
        pack.compare_holdout(original)
        case = original['holdout-sentry-bite-grown-sentry']
        final = case['checkpoints'][-1]['state']
        self.assertEqual([final['objects'][0][k] for k in ('power','toughness','damage')], [4,4,0])
        self.assertEqual([final['objects'][1][k] for k in ('power','toughness','damage')], [7,7,4])
        self.assertEqual(final['damage_events'], [{'source':'source','target':'destination','amount':4}])
        for field, value in [('power',4), ('damage',2), ('owner',0)]:
            wrong = copy.deepcopy(original)
            wrong['holdout-sentry-bite-grown-sentry']['checkpoints'][-1]['state']['objects'][1][field] = value
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, field):
                pack.compare_holdout(wrong)
        for part in ('checkpoints','consumed'):
            wrong = copy.deepcopy(original)
            wrong['holdout-sentry-bite-grown-sentry'][part].pop()
            with self.subTest(part=part), self.assertRaises(ValueError):
                pack.compare_holdout(wrong)
