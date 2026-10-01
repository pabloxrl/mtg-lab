"""GH-189: diagnostics must never revoke delivery authorization.

Run the real hook with isolated filesystem/process/API boundaries. Expectations
come from the operator's continuous-delivery policy, not observed hook output.
"""
import contextlib
import io
import json
import os
from pathlib import Path
import runpy
import subprocess
import tempfile
import unittest
from unittest.mock import patch


HOOK = Path(__file__).resolve().parents[1] / "scripts/symphony/before_run.py"


class BeforeRunTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.workspace = Path(self.temp.name) / "GH-189"
        self.workspace.mkdir()
        self.counter = self.workspace / ".symphony-attempts.json"
        self.program = {"execution": {"kind": "docker", "prerequisite_issue": 47},
                        "tasks": [{"issue": 189}]}
        self.gate = {"state": "closed", "state_reason": "completed"}
        self.calls = []

    def process(self, args, **kwargs):
        self.calls.append(args)
        if args[:2] == ["git", "show"]:
            return json.dumps(self.program)
        if args[:2] == ["gh", "api"]:
            self.assertEqual(args[2], "repos/pabloxrl/mtg-lab/issues/47")
            return json.dumps(self.gate)
        return subprocess.CompletedProcess(args, 0)

    def run_hook(self, container="1", marker=True):
        exists = Path.exists
        with contextlib.chdir(self.workspace), \
                patch.dict(os.environ, {"MTG_CONTAINER": container}), \
                patch.object(Path, "exists", lambda p: marker if str(p) == "/.dockerenv" else exists(p)), \
                patch("subprocess.run", side_effect=self.process), \
                patch("subprocess.check_output", side_effect=self.process), \
                patch("time.time", return_value=100_000), \
                contextlib.redirect_stdout(io.StringIO()):
            runpy.run_path(str(HOOK), run_name="__main__")

    def assert_no_github_mutations(self):
        self.assertFalse([c for c in self.calls if c[:2] == ["gh", "issue"]])

    def test_old_counter_is_diagnostic(self):
        self.counter.write_text(json.dumps({"started": 1, "attempts": 1, "note": "retain"}))
        self.run_hook()
        self.assertEqual(json.loads(self.counter.read_text()),
                         {"started": 1, "attempts": 2, "note": "retain"})
        self.assert_no_github_mutations()

    def test_many_dispatches_are_diagnostic(self):
        self.counter.write_text(json.dumps({"started": 99_999, "attempts": 80}))
        self.run_hook()
        self.assertEqual(json.loads(self.counter.read_text())["attempts"], 81)
        self.assert_no_github_mutations()

    def test_merged_awaiting_ci_repeated_continuation_has_no_comments(self):
        # Existing merged delivery evidence is retained; the hook must not need
        # PR state to permit a CI/final-handoff continuation.
        receipt = self.workspace / "merged-awaiting-ci.json"
        receipt.write_text('{"merged": true, "main_ci": "pending"}\n')
        self.counter.write_text('{"started": 1, "attempts": 8}')
        for _ in range(3):
            self.run_hook()
        self.assertEqual(json.loads(self.counter.read_text()), {"started": 1, "attempts": 11})
        self.assertEqual(receipt.read_text(), '{"merged": true, "main_ci": "pending"}\n')
        self.assert_no_github_mutations()

    def test_fresh_dispatch_keeps_identity_and_prerequisite_checks(self):
        self.run_hook()
        self.assertEqual(json.loads(self.counter.read_text()), {"started": 100_000, "attempts": 1})
        self.assertEqual(self.calls[:3], [
            ["git", "config", "--local", "user.name", "pablo ribalta"],
            ["git", "config", "--local", "user.email", "pabloxrl@gmail.com"],
            ["git", "fetch", "origin", "main"]])
        self.assert_no_github_mutations()

    def test_invalid_container_rejects_before_diagnostics(self):
        for container, marker in (("0", True), ("1", False)):
            with self.subTest(container=container, marker=marker):
                with self.assertRaisesRegex(SystemExit, "managed Docker"):
                    self.run_hook(container, marker)
                self.assertFalse(self.counter.exists())
        self.assert_no_github_mutations()

    def test_incomplete_prerequisite_rejects_even_with_old_counter(self):
        original = '{"started": 1, "attempts": 80}'
        self.counter.write_text(original)
        for gate in ({"state": "open"}, {"state": "closed", "state_reason": "not_planned"}):
            with self.subTest(gate=gate):
                self.gate = gate
                with self.assertRaisesRegex(SystemExit, "prerequisite #47 is incomplete"):
                    self.run_hook()
                self.assertEqual(self.counter.read_text(), original)
        self.assert_no_github_mutations()

    def test_malformed_diagnostic_fails_visibly_without_mutation(self):
        for original in ('{', '[]', '{}', '{"started": 1, "attempts": "bad"}',
                         '{"started": 1, "attempts": -1}',
                         '{"started": 1, "attempts": true}',
                         '{"started": NaN, "attempts": 2}',
                         '{"started": "bad", "attempts": 2}'):
            with self.subTest(original=original):
                self.counter.write_text(original)
                with self.assertRaises((SystemExit, ValueError, TypeError, KeyError)):
                    self.run_hook()
                self.assertEqual(self.counter.read_text(), original)
        self.assert_no_github_mutations()

    def test_invalid_execution_contract_and_missing_prerequisite_reject(self):
        for execution in ({"kind": "native", "prerequisite_issue": 47},
                          {"kind": "docker"}):
            with self.subTest(execution=execution):
                self.program["execution"] = execution
                with self.assertRaises((SystemExit, KeyError)):
                    self.run_hook()
                self.assertFalse(self.counter.exists())
        self.assert_no_github_mutations()

    def test_prerequisite_api_outage_retains_diagnostics_without_authorizing(self):
        original = '{"started": 1, "attempts": 80}'
        self.counter.write_text(original)
        process = self.process

        def unavailable(args, **kwargs):
            if args[:2] == ["gh", "api"]:
                raise subprocess.CalledProcessError(1, "gh api")
            return process(args, **kwargs)

        with patch.object(self, "process", side_effect=unavailable):
            with self.assertRaises(subprocess.CalledProcessError):
                self.run_hook()
        self.assertEqual(self.counter.read_text(), original)
        self.assert_no_github_mutations()

    def test_git_failure_is_not_silently_ignored(self):
        with patch.object(self, "process", side_effect=subprocess.CalledProcessError(1, "git")):
            with self.assertRaises(subprocess.CalledProcessError):
                self.run_hook()
        self.assertFalse(self.counter.exists())
