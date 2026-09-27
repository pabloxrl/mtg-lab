"""Executable verifier torture cases; these do not simulate a game engine."""
import copy
import io
import random
import unittest

from scripts.checkpoints import difference
from scripts.run_tests import complete_success


class RegressionRunnerTests(unittest.TestCase):
    def run_case(self, method, decorator=lambda f: f):
        case = type('Probe', (unittest.TestCase,), {'test_probe': decorator(method)})
        return unittest.TextTestRunner(stream=io.StringIO()).run(
            unittest.defaultTestLoader.loadTestsFromTestCase(case))

    def test_only_executed_passes_are_success(self):
        self.assertTrue(complete_success(self.run_case(lambda t: t.assertEqual(2 + 2, 4))))
        self.assertFalse(complete_success(self.run_case(lambda t: t.fail('literal negative control'))))
        self.assertFalse(complete_success(self.run_case(lambda t: None, unittest.skip('unimplemented'))))
        self.assertFalse(complete_success(self.run_case(lambda t: t.fail(), unittest.expectedFailure)))
        self.assertFalse(complete_success(self.run_case(lambda t: None, unittest.expectedFailure)))
        self.assertFalse(complete_success(unittest.TestResult()))


class VerifierTortureTests(unittest.TestCase):
    def test_seeded_nested_change_is_detected_without_mutation(self):
        # Independent contract: one changed leaf must be reported at its exact
        # JSON pointer, regardless of its depth or irrelevant surrounding data.
        for seed in range(256):
            with self.subTest(seed=seed):
                rng = random.Random(seed)
                expected, actual, path = 17, 18, ''
                for _ in range(rng.randint(1, 12)):
                    if rng.choice([False, True]):
                        key, escaped = rng.choice([('~/', '~0~1'), ('/', '~1'), ('~', '~0'), ('', ''), ('x', 'x')])
                        expected, actual = {key: expected}, {key: actual}
                        path = '/' + escaped + path
                    else:
                        offset = rng.randrange(5)
                        prefix = [None] * offset
                        expected, actual = prefix + [expected], prefix + [actual]
                        path = '/' + str(offset) + path
                before = copy.deepcopy((expected, actual))
                self.assertEqual(difference(expected, actual), dict(path=path, expected=17, actual=18,
                    expected_present=True, actual_present=True))
                self.assertEqual((expected, actual), before)

    def test_object_order_is_irrelevant_but_array_order_matters(self):
        for seed in range(128):
            with self.subTest(seed=seed):
                rng = random.Random(seed)
                keys = list(range(12))
                rng.shuffle(keys)
                expected = {str(k): {'items': [k, k + 1]} for k in keys}
                actual = dict(reversed(list(expected.items())))
                self.assertIsNone(difference(expected, actual))
                actual = copy.deepcopy(actual)
                actual['5']['items'].reverse()
                self.assertEqual(difference(expected, actual)['path'], '/5/items/0')

    def test_literal_type_and_presence_counterexamples(self):
        for expected, actual in [(True, 1), (False, 0), (None, ''), ([], {}), ('1', 1)]:
            with self.subTest(expected=expected, actual=actual):
                self.assertIsNotNone(difference(expected, actual))
        self.assertIsNone(difference(1, 1.0))  # JSON number equivalence is intentional.
        self.assertEqual(difference({'secret': None}, {}), dict(path='/secret', expected=None,
            actual=None, expected_present=True, actual_present=False))
        self.assertEqual(difference([], [None]), dict(path='/0', expected=None,
            actual=None, expected_present=False, actual_present=True))

    def test_first_divergence_is_stable_across_insertion_order(self):
        for seed in range(128):
            with self.subTest(seed=seed):
                keys = ['z', 'a', 'm', '~', '/']
                random.Random(seed).shuffle(keys)
                expected = {k: [10, 20] for k in keys}
                actual = {k: [11, 21] for k in reversed(keys)}
                self.assertEqual(difference(expected, actual)['path'], '/~1/0')
