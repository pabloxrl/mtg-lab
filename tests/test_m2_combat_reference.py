"""GH-210 coverage accounting rejects missing, stale and altered evidence."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import m2_combat_reference as pack


class M2CombatReference(unittest.TestCase):
    def test_missing_catalog_slots_are_failure(self):
        with self.assertRaisesRegex(ValueError, '50|catalog|manifest'):
            pack.validate_manifest({'issue': 210, 'cases': []})

    def test_stale_or_unexecuted_receipts_are_failure(self):
        expected = {'runner_sha256': 'a' * 64, 'bridge_sha256': 'b' * 64}
        for receipt in ({}, {'status': 'unsupported'},
                        {'status': 'agreed', 'runner_sha256': 'c' * 64},
                        {'status': 'agreed', **expected, 'repetitions': 0}):
            with self.subTest(receipt=receipt), self.assertRaises(ValueError):
                pack.require_current_receipt(receipt, expected)

    def test_exact_manifest_and_original_expectations(self):
        document = json.loads((pack.ROOT / 'fixtures/reference/m2-combat-pack.json').read_text())
        self.assertEqual(len(pack.validate_manifest(document)), 50)
        for mutation in ('omit', 'duplicate', 'expectation', 'unexecuted'):
            wrong = copy.deepcopy(document)
            if mutation == 'omit':
                wrong['cases'].pop()
            elif mutation == 'duplicate':
                wrong['cases'][1] = wrong['cases'][0]
            elif mutation == 'expectation':
                wrong['cases'][0]['catalog']['expected'] = 'approve wrong result'
            else:
                wrong['cases'][0]['executions'][0]['cases'] = ['absent']
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                pack.validate_manifest(wrong)

    def test_named_checkpoint_mutants(self):
        canonical = json.loads(pack.EXPECTED.read_text())
        pack.compare(canonical)
        expected_paths = {'damage-split': 'after-damage',
                          'retained-temporary-keyword': 'after-cleanup'}
        for name, wrong in pack.mutations(canonical).items():
            with self.subTest(name=name), self.assertRaisesRegex(ValueError, expected_paths[name]):
                pack.compare(wrong)

    def test_complete_current_receipt(self):
        expected = {'runner_sha256': 'a' * 64, 'bridge_sha256': 'b' * 64}
        pack.require_current_receipt({'status': 'agreed', 'repetitions': 2, **expected}, expected)
        for field in expected:
            stale = dict(status='agreed', repetitions=2, **expected)
            stale[field] = 'c' * 64
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, field):
                pack.require_current_receipt(stale, expected)

    def test_fixture_rejects_ignored_fields_and_missing_scripts(self):
        fixture = json.loads(pack.FIXTURE.read_text())
        pack.validate_fixture(fixture)
        for mutation in ('unknown', 'missing', 'duplicate', 'unbounded', 'bad_type'):
            wrong = copy.deepcopy(fixture)
            if mutation == 'unknown':
                wrong['cases'][0]['ignored_choice'] = 'attack another target'
            elif mutation == 'missing':
                wrong['cases'].pop()
            elif mutation == 'duplicate':
                wrong['cases'][1] = wrong['cases'][0]
            elif mutation == 'unbounded':
                wrong['cases'][0]['boosts'] = 100000
            else:
                wrong['cases'][0]['boosts'] = True
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                pack.validate_fixture(wrong)
