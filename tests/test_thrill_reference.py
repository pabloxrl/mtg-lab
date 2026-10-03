"""Independent literal fixture completeness and comparator controls."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import thrill_reference as reference


class ThrillReference(unittest.TestCase):
    def test_case_inventory_and_mutations(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        expected = json.loads(reference.EXPECTED.read_text())
        self.assertEqual(len(fixture['cases']), 6)
        self.assertEqual(set(expected), {c['id'] for c in fixture['cases']})
        reference.compare(expected)
        def leaves(value, path=()):
            if isinstance(value, dict):
                for key, child in value.items():
                    yield from leaves(child, path + (key,))
            elif isinstance(value, list) and value:
                for index, child in enumerate(value):
                    yield from leaves(child, path + (index,))
            else:
                yield path
        for path in leaves(expected):
            wrong = copy.deepcopy(expected)
            node = wrong
            for key in path[:-1]:
                node = node[key]
            node[path[-1]] = "mutated checkpoint"
            with self.subTest(path=path), self.assertRaises(ValueError):
                reference.compare(wrong)
        for name, point in expected.items():
            for field, value in point.items():
                wrong = copy.deepcopy(expected)
                wrong[name][field] = not value if isinstance(value, bool) else 99
                with self.subTest(name=name, field=field), self.assertRaises(ValueError):
                    reference.compare(wrong)
            wrong = copy.deepcopy(expected)
            del wrong[name]
            with self.assertRaises(ValueError):
                reference.compare(wrong)
