"""A compiler error, wrong test, or unrelated panic is never a mutant kill."""
import unittest
from scripts.m2_mutations import verdict

TEST = "game::m2_mutation_tests::priority"
MARKER = "M2-MUT priority: opponent receives priority"


def output(test=TEST, marker=MARKER):
    return (f"running 1 test\nthread '{test}' panicked at src/check.rs:1:1:\n"
            f"assertion `left == right` failed: {marker}\n  left: 0\n right: 1\n"
            f"test {test} ... FAILED\n"
            "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s\n")


class MutationVerdicts(unittest.TestCase):
    def test_exact_behavioral_assertion_detects_mutant(self):
        self.assertEqual(verdict(TEST, MARKER, 101, output()), "detected")

    def test_surviving_mutant_is_not_detected(self):
        self.assertEqual(verdict(TEST, MARKER, 0, "test result: ok."), "survived")

    def test_missing_test_build_failure_wrong_scenario_and_panic_fail(self):
        for code, text in [
            (101, "error[E0308]: mismatched types"),
            (101, output(test="different_scenario")),
            (101, output(marker="unrelated assertion")),
            (101, output().replace("assertion `left == right` failed", "unwrap failed")),
            (101, output().replace("running 1 test", "running 0 tests")),
            (1, output()),
            (-9, output()),
            (101, output().replace("0 ignored", "1 ignored")),
        ]:
            with self.subTest(code=code, text=text):
                self.assertEqual(verdict(TEST, MARKER, code, text), "invalid")


class MutationExecution(unittest.TestCase):
    def test_source_patch_rejects_absent_duplicate_and_noop_anchors(self):
        from scripts.m2_mutations import patch
        self.assertEqual(patch('before good after', 'good', 'bad'), 'before bad after')
        for source, old, new in [('none', 'good', 'bad'), ('good good', 'good', 'bad'),
                                 ('good', 'good', 'good'), ('good', '', 'bad')]:
            with self.assertRaises(ValueError):
                patch(source, old, new)

    def test_matrix_has_twelve_distinct_named_assertions_and_real_anchors(self):
        from scripts.m2_mutations import ROOT, load_matrix, patch
        cases = load_matrix()
        self.assertEqual({c['id'] for c in cases}, {
            'priority', 'mana_stack', 'sick_tap', 'cast_trigger', 'boost_expiry',
            'cleanup_damage', 'stale_action', 'private_hand', 'terminal_reward',
            'truncation', 'clipped_choices', 'failed_denominator'})
        for c in cases:
            self.assertIn(c['assertion'], (ROOT / c['test_path']).read_text())
            source = (ROOT / c['path']).read_text()
            self.assertNotEqual(patch(source, c['old'], c['new']), source)

    def test_build_failure_and_missing_test_cannot_reach_execution(self):
        from pathlib import Path
        from unittest.mock import patch
        from scripts.m2_mutations import execute, load_matrix
        case = load_matrix()[0]
        with patch('scripts.m2_mutations.command', return_value=(101, 'compiler failure')) as run:
            self.assertEqual(execute(case, Path('.'), {}, Path('.'), 'mutant')['verdict'], 'build_failure')
            self.assertEqual(run.call_count, 1)
        artifact = '{"reason":"compiler-artifact","executable":"/tmp/test","profile":{"test":true}}'
        with patch('scripts.m2_mutations.command', side_effect=[(0, artifact), (0, '0 tests, 0 benchmarks')]) as run:
            self.assertEqual(execute(case, Path('.'), {}, Path('.'), 'mutant')['verdict'], 'missing_test')
            self.assertEqual(run.call_count, 2)

    def test_baseline_requires_one_pass_with_no_ignored_tests(self):
        from scripts.m2_mutations import passed
        green = (f'running 1 test\ntest {TEST} ... ok\n'
                 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 9 filtered out;')
        self.assertTrue(passed(TEST, 0, green))
        for text in [green.replace('1 passed', '0 passed'), green.replace('0 ignored', '1 ignored'),
                     green.replace(TEST, 'wrong_test'), '', output()]:
            self.assertFalse(passed(TEST, 0, text))
        self.assertFalse(passed(TEST, 101, green))

    def test_real_subprocess_launch_failure_and_timeout_are_not_assertions(self):
        import os
        import sys
        import tempfile
        from pathlib import Path
        from scripts.m2_mutations import command
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            code, text = command([str(root / 'absent')], root, os.environ.copy(), root / 'launch.log')
            self.assertEqual(code, -1)
            self.assertEqual(verdict(TEST, MARKER, code, text), 'invalid')
            code, text = command([sys.executable, '-c', 'import time; time.sleep(10)'],
                                 root, os.environ.copy(), root / 'timeout.log', timeout=0.05)
            self.assertEqual(code, -1)
            self.assertIn('timed out', (root / 'timeout.log').read_text())
