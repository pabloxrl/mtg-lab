"""RFC B008/B021: execute the real native client and reject bad accounting."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
LABELS = {"reset", "transition", "legality_and_view", "policy", "encoding", "finalization"}
EDGES = [0, 10, 100, 1000, 10000, 100000, 1000000]


def native_binary():
    return ROOT / os.environ.get("CARGO_TARGET_DIR", "target") / "debug/mtg"


def validate(report):
    """Independent fixed schema/arithmetic oracle, never repaired from output."""
    if report.get("schema_version") != 1 or report.get("interval") != 64:
        raise ValueError("version/schedule")
    if report.get("bucket_upper_ns") != EDGES or set(report.get("phases", {})) != LABELS:
        raise ValueError("unbounded labels/buckets")
    for label, phase in report["phases"].items():
        if phase["status"] not in ("measured", "not_measured", "not_applicable", "unavailable"):
            raise ValueError("availability")
        fields = ("attempts", "count", "skipped", "clock_errors", "errors", "sum_ns")
        if any(type(phase[f]) is not int or phase[f] < 0 for f in fields):
            raise ValueError("unsigned counts")
        if phase["overflow"] or phase["clock_errors"]:
            raise ValueError("invalid measurement")
        if phase["attempts"] != phase["count"] + phase["skipped"]:
            raise ValueError("denominator")
        if phase["errors"] > phase["attempts"]:
            raise ValueError("error attempts")
        buckets = phase["buckets"]
        if len(buckets) != 8 or any(type(n) is not int or n < 0 for n in buckets):
            raise ValueError("buckets")
        if sum(buckets) != phase["count"]:
            raise ValueError("sample count")
        if phase["count"] != (phase["attempts"] + 63) // 64:
            raise ValueError("sampling schedule")
        if phase["status"] != ("measured" if phase["count"] else "not_measured"):
            raise ValueError("availability does not match samples")
        low = sum(n * (0 if i == 0 else EDGES[i-1] + 1) for i, n in enumerate(buckets))
        high = None if buckets[-1] else sum(n * edge for n, edge in zip(buckets, EDGES))
        if phase["sum_ns"] < low or (high is not None and phase["sum_ns"] > high):
            raise ValueError("sum outside bucket bounds")
        for p in (50, 95, 99):
            expected = None
            if phase["count"]:
                rank = (phase["count"] * p + 99) // 100
                seen = 0
                for i, n in enumerate(buckets):
                    seen += n
                    if seen >= rank:
                        expected = {"lower_ns": 0 if i == 0 else EDGES[i-1]+1,
                                    "upper_ns": EDGES[i] if i < 7 else None}
                        break
            if phase[f"p{p}"] != expected:
                raise ValueError("quantile bounds")
        if label != "encoding" and phase["count"] == 0:
            raise ValueError("empty real execution")


class ScalarSampledTiming(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Resource ownership belongs to the invoking environment. Symphony
        # wraps the whole check in its existing heavy lock; CI has an isolated
        # verification partition. The test must not infer controller presence
        # from an image's default environment variable.
        subprocess.run(["cargo", "build", "-p", "mtg-cli", "--locked"], cwd=ROOT, check=True)

    def run_client(self, mode):
        config = json.loads((ROOT / "fixtures/bench/scalar-workload-v1.json").read_text())
        config["max_decisions"] = 8
        config["native"]["instrumentation"] = mode
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "input.json"
            path.write_text(json.dumps(config))
            result = subprocess.run([str(native_binary()), "simulate", "--config", str(path)],
                                    cwd=ROOT, capture_output=True, text=True, timeout=60, check=True)
        return [json.loads(line) for line in result.stdout.splitlines()]

    def test_real_native_modes_have_nonempty_samples_and_unchanged_history(self):
        baseline = self.run_client("off")
        self.assertNotIn("latency", baseline[-1])
        self.assertEqual(baseline[1]["decisions"], 8)
        for mode in ("counters", "sampled_trace", "full_replay"):
            rows = self.run_client(mode)
            self.assertIn("latency", rows[-1], "missing real-client sampled observations")
            validate(rows[-1]["latency"])
            for key in ("history_sha256", "life", "public_zones", "decisions", "status"):
                self.assertEqual(rows[1][key], baseline[1][key])

    def test_real_export_rejects_tampered_accounting_and_private_labels(self):
        report = self.run_client("counters")[-1]
        self.assertIn("latency", report, "missing real-client artifact")
        report = report["latency"]
        validate(report)
        changes = [("count", 0), ("sum_ns", -1), ("skipped", 999),
                   ("p95", {"lower_ns": -1, "upper_ns": -1}), ("errors", 999),
                   ("buckets", [0]*8), ("overflow", True), ("clock_errors", 1),
                   ("status", "not_measured")]
        for key, value in changes:
            bad = copy.deepcopy(report)
            bad["phases"]["reset"][key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                validate(bad)
        bad = copy.deepcopy(report)
        bad["phases"]["hidden-hand-seed-42"] = bad["phases"].pop("policy")
        with self.assertRaises(ValueError):
            validate(bad)


class NativeBuildInterface(unittest.TestCase):
    def test_build_command_does_not_depend_on_controller_environment(self):
        # The pinned CI image declares a controller path absent from the
        # read-only verification checkout. Scheduling belongs to the caller.
        with patch.dict(os.environ, {"SYMPHONY_CONTROL_ROOT": "/unavailable-controller"}):
            with patch("subprocess.run") as run:
                os.environ.pop("MTG_SYMPHONY_LOCK_FD", None)
                ScalarSampledTiming.setUpClass()
        self.assertEqual(run.call_args.args[0], ["cargo", "build", "-p", "mtg-cli", "--locked"])

    def test_binary_path_respects_cargo_target_directory(self):
        # scripts/verify-docker.sh supplies an absolute target outside source;
        # Cargo also accepts a target path relative to its working directory.
        for value, expected in [("/tmp/native-build-output", Path("/tmp/native-build-output/debug/mtg")),
                                ("other-target", ROOT / "other-target/debug/mtg")]:
            with self.subTest(value=value), patch.dict(os.environ, {"CARGO_TARGET_DIR": value}):
                self.assertEqual(native_binary(), expected)
