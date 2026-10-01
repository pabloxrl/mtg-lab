"""GH-189 acceptance runs in ordinary discovery, using the managed image."""
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]


class DeliveryContinuationTests(unittest.TestCase):
    def test_packaged_controller_continuation_and_bounds(self):
        result = subprocess.run(
            [sys.executable, str(ROOT / "scripts/symphony/continuation_smoke.py")],
            capture_output=True, text=True, timeout=100)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("credential-free-controller-continuation-ok", result.stdout)

    def test_workflow_retains_program_and_operator_authorization_gates(self):
        # These are agent instructions, not native scheduler logic. Lock their
        # preservation separately rather than claiming the controller enforces
        # parent labels or a full dependency handoff itself.
        workflow = (ROOT / "WORKFLOW.md").read_text()
        for gate in (
            "A closed parent or `program-paused`/`program-cancelled` label stops delivery",
            "`agent-held`/`agent-blocked`, closed, or missing `agent-ready` must stop without",
            "Never remove `agent-held` or `agent-blocked` during handoff.",
            "never automatically\n   restore `agent-ready`",
            "This exception\n   does not clear held/blocked controls or bypass dependencies.",
            "A program-wide pause/cancel always wins over handoff.",
        ):
            with self.subTest(gate=gate):
                self.assertIn(gate, workflow)
