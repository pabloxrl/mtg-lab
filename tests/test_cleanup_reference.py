"""Original literal Oracle/CR ledgers and complete comparator controls."""
import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import cleanup_reference as reference


class CleanupReference(unittest.TestCase):
    def test_inventory_and_every_checkpoint_field(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        expected = json.loads(reference.EXPECTED.read_text())
        self.assertEqual(len(fixture['cases']), 4)
        self.assertEqual(set(expected), {c['id'] for c in fixture['cases']})
        self.assertEqual(expected['rules-continuous-hand-size-cleanup-regression'][-1],
                         [7, 8, 3, 18, 0, 2, 1, 0])
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
