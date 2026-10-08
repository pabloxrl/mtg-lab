"""Original literal CR ledgers and strict comparator mutation controls."""
import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import cast_trigger_reference as reference


class CastTriggerReference(unittest.TestCase):
    def test_inventory_and_every_checkpoint_field(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        expected = json.loads(reference.EXPECTED.read_text())
        self.assertEqual(len(fixture['cases']), 22)
        self.assertEqual(set(expected), {c['id'] for c in fixture['cases']})
        self.assertEqual(expected['mixed_growth'], [[[20, 20], [0, 4], 3, 0], [[20, 20], [3, 4], 2, 0], [[20, 19], [3, 4], 1, 0], [[20, 19], [6, 7], 0, 0]])
        reference.compare(expected)
        for name, points in expected.items():
            for row, point in enumerate(points):
                for column in range(len(point)):
                    wrong = copy.deepcopy(expected)
                    wrong[name][row][column] = 99
                    with self.subTest(name=name, row=row, column=column), self.assertRaises(ValueError):
                        reference.compare(wrong)
            wrong = copy.deepcopy(expected)
            del wrong[name]
            with self.assertRaises(ValueError):
                reference.compare(wrong)
