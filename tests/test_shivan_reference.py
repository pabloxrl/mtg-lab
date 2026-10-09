"""Independent literal fixture completeness and comparator controls."""
import copy
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import shivan_reference as reference


class ShivanReference(unittest.TestCase):
    def test_case_inventory_and_mutations(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        expected = json.loads(reference.EXPECTED.read_text())
        self.assertEqual(len([c for c in fixture['cases'] if not c['id'].startswith('exact_')]), 23)
        self.assertEqual(len([c for c in fixture['cases'] if c['id'].startswith('exact_')]), 8)
        self.assertEqual(set(expected), {c['id'] for c in fixture['cases']})
        reference.compare(expected)
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

    def test_exact_checkpoint_fields_and_choice_mutations(self):
        expected = json.loads(reference.EXPECTED.read_text())
        for name, result in expected.items():
            if not name.startswith('exact_'):
                continue
            for point, state in result['points'].items():
                for field in state:
                    wrong = copy.deepcopy(expected)
                    wrong[name]['points'][point][field] = None
                    with self.subTest(case=name, point=point, field=field):
                        with self.assertRaisesRegex(ValueError, 'first divergence'):
                            reference.compare(wrong)
                for obj, values in state['objects'].items():
                    for index in range(6):
                        wrong = copy.deepcopy(expected)
                        wrong[name]['points'][point]['objects'][obj][index] = 'mutated'
                        with self.assertRaisesRegex(ValueError, point):
                            reference.compare(wrong)
            for target in result['combat_damage']:
                wrong = copy.deepcopy(expected)
                wrong[name]['combat_damage'][target] += 1
                with self.assertRaisesRegex(ValueError, 'combat_damage'):
                    reference.compare(wrong)
            for mutation in ('omitted', 'extra', 'wrong_actor'):
                wrong = copy.deepcopy(expected)
                choices = wrong[name]['consumed']
                if mutation == 'omitted':
                    choices.pop()
                elif mutation == 'extra':
                    choices.append(choices[-1])
                else:
                    choices[0]['actor'] = 1 - choices[0]['actor']
                with self.assertRaisesRegex(ValueError, 'consumed'):
                    reference.compare(wrong)

    def test_strict_exact_inputs(self):
        fixture = json.loads(reference.FIXTURE.read_text())
        reference.validate_fixture(fixture)
        index = next(i for i, c in enumerate(fixture['cases']) if c['id'].startswith('exact_'))
        for field, value in [('active', True), ('active', 2), ('mode', 'unknown'), ('unknown', 1)]:
            bad = copy.deepcopy(fixture)
            bad['cases'][index][field] = value
            with self.assertRaisesRegex(ValueError, 'first divergence'):
                reference.validate_fixture(bad)
        for change in ('missing', 'extra', 'actor', 'unknown'):
            bad = copy.deepcopy(fixture)
            choices = bad['cases'][index]['choices']
            if change == 'missing':
                choices.pop()
            elif change == 'extra':
                choices.append(choices[-1])
            elif change == 'actor':
                choices[0]['actor'] = True
            else:
                choices[0]['unexpected'] = 0
            with self.assertRaisesRegex(ValueError, 'first divergence'):
                reference.validate_fixture(bad)
