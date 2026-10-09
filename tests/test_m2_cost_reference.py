"""Independent GH-209 checkpoint inventory and fail-closed runner controls."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import m2_cost_reference as reference


class M2CostReference(unittest.TestCase):
    def test_literal_inventory_and_first_divergence(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        expected = json.loads(reference.EXPECTED.read_text())
        reference.validate_fixture(fixture)
        self.assertEqual(len(expected), 6)
        reference.compare(expected)
        for case, points in expected.items():
            for index, point in enumerate(points):
                for field in point:
                    wrong = copy.deepcopy(expected)
                    wrong[case][index][field] = 'mutated'
                    with self.subTest(case=case, index=index, field=field):
                        with self.assertRaises(ValueError) as caught:
                            reference.compare(wrong)
                        difference = json.loads(str(caught.exception).split(': ', 1)[1])
                        self.assertEqual(difference['path'], f'$.{case}[{index}].{field}')
            wrong = copy.deepcopy(expected)
            del wrong[case]
            with self.assertRaises(ValueError):
                reference.compare(wrong)

    def test_unsupported_scripts_fail(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        for field, value in [('kind', 'automatic'), ('card', 'unsupported'),
                             ('payment', 'yes'), ('extra', True)]:
            wrong = copy.deepcopy(fixture)
            wrong['cases'][0][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                reference.validate_fixture(wrong)
        wrong = copy.deepcopy(fixture)
        wrong['cases'][1]['id'] = wrong['cases'][0]['id']
        with self.assertRaises(ValueError):
            reference.validate_fixture(wrong)

    def test_missing_build_never_emits_agreement(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'output').mkdir()
            (root / 'output/receipt.json').write_text('{"status":"agreed"}')
            result = subprocess.run([sys.executable, str(Path(reference.__file__)),
                                     '--cache', str(root / 'absent'), '--output', str(root / 'output')],
                                    stdin=subprocess.DEVNULL, capture_output=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((root / 'output/receipt.json').exists())
