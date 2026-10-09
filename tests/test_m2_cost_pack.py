"""The joined receipt cannot silently drop assigned cases or change their oracle."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import m2_cost_pack as pack


class M2CostPack(unittest.TestCase):
    def test_exact_original_catalog_and_native_mapping(self):
        rows = json.loads(pack.CATALOG.read_text())
        pack.validate_catalog(rows)
        for mutation in ('missing', 'duplicate', 'expected', 'owner', 'test'):
            wrong = copy.deepcopy(rows)
            if mutation == 'missing':
                wrong.pop()
            elif mutation == 'duplicate':
                wrong.append(wrong[0])
            elif mutation == 'expected':
                wrong[0]['catalog']['expected'] = 'trust engine output'
            elif mutation == 'owner':
                wrong[0]['catalog']['owner_issue'] = 209
            else:
                wrong[0]['native_tests'] = ['missing_test']
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                pack.validate_catalog(wrong)
