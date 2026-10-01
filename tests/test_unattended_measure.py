"""Failed benchmark attempts remain explicit samples, never successful games."""
import importlib.util
import os
from pathlib import Path
import sys
import unittest

SPEC = importlib.util.spec_from_file_location(
    "measure", Path(__file__).resolve().parents[1]
    / "doc/evidence/unattended-integration/measure.py")
MEASURE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MEASURE)


class MeasurementFailures(unittest.TestCase):
    def test_timeout_retains_attempt_and_partial_output(self):
        row = MEASURE.attempt(
            [sys.executable, "-c", "import time; print('partial', flush=True); time.sleep(10)"],
            os.environ.copy(), timeout=0.5)
        self.assertEqual(row["failure"], "timeout")
        self.assertIsNone(row["exit_code"])
        self.assertIsNone(row["result"])
        self.assertIn("partial", row["stdout"])
        self.assertGreater(row["process_elapsed_ns"], 0)

    def test_malformed_and_missing_json_remain_explicit(self):
        for output in ("not-json", ""):
            row = MEASURE.attempt([sys.executable, "-c", f"print({output!r}, end='')"],
                                  os.environ.copy())
            self.assertEqual(row["failure"], "invalid_json")
            self.assertEqual(row["exit_code"], 0)
            self.assertEqual(row["stdout"], output)
            self.assertIsNone(row["result"])

    def test_nonzero_exit_keeps_structured_failure(self):
        row = MEASURE.attempt([sys.executable, "-c",
                               "import sys; print('{\"failed\":1}'); sys.exit(3)"],
                              os.environ.copy())
        self.assertEqual(row["exit_code"], 3)
        self.assertEqual(row["result"], {"failed": 1})


if __name__ == "__main__":
    unittest.main()
