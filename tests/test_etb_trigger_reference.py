"""Original literal Oracle/CR ledgers and complete comparator controls."""
import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import etb_trigger_reference as reference


class EtbTriggerReference(unittest.TestCase):
    def test_inventory_and_every_checkpoint_field(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        expected = json.loads(reference.EXPECTED.read_text())
        self.assertEqual(len(fixture['cases']), 14)
        self.assertEqual(set(expected), {c['id'] for c in fixture['cases']})
        self.assertEqual(expected['self_nonlethal'], [
            [[20, 20], None, 1, [], [False, False]],
            [[20, 20], [2, 1], 1, [0], [False, False]],
            [[18, 20], [2, 1], 0, [], [False, False]],
        ])
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
