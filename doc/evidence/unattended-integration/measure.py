#!/usr/bin/env python3
"""Initial M1 diagnostic, NOT the RFC's dedicated-host qualification protocol.

Build first: cargo build --locked --release -p mtg-cli
Run from repo root: python3 doc/evidence/unattended-integration/measure.py OUTPUT
Five repetitions of the same 16 single-episode cases; no warmup or best selection.
"""
import copy
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile
import time


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    binary = Path("target/release/mtg").resolve()
    config = json.loads(Path("fixtures/simulate/native-v2.json").read_text())
    config["episodes"] = 1
    sources = sorted(Path("crates").rglob("*.rs")) + [Path("Cargo.toml"), Path("Cargo.lock")]
    env = os.environ.copy()
    for key in ("DISPLAY", "WAYLAND_DISPLAY"):
        env.pop(key, None)
    receipt = {
        "schema_version": 1, "qualification": "initial-baseline-only",
        "protocol": "five repeats; no warmup; 16 cases per repeat; one episode per invocation",
        "limitations": "shared container, unpinned affinity, short windows; no hardware/performance qualification",
        "created_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "base_commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
        "source_sha256": {str(p): sha(p) for p in sources}, "binary_sha256": sha(binary),
        "build": "cargo build --locked --release -p mtg-cli",
        "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
        "rustflags": os.environ.get("RUSTFLAGS", ""),
        "cargo_encoded_rustflags": os.environ.get("CARGO_ENCODED_RUSTFLAGS", ""),
        "python": platform.python_version(), "platform": platform.platform(),
        "cpu": subprocess.check_output(["lscpu", "--json"], text=True),
        "affinity": sorted(os.sched_getaffinity(0)),
        "mem_total": next(line.strip() for line in Path("/proc/meminfo").read_text().splitlines() if line.startswith("MemTotal:")),
        "samples": [],
    }
    # Exclusive output publication: never overwrite a prior receipt.
    with open(sys.argv[1], "x") as output, tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "config.json"
        for repeat in range(5):
            for policy in ("heuristic-m1-v1", "legal-random-m1-v1"):
                for decks in (("red", "green"), ("green", "red"), ("red", "red"), ("green", "green")):
                    for starter in (0, 1):
                        c = copy.deepcopy(config)
                        c["policies"] = [policy, policy]
                        c["game"]["starting_seat"] = starter
                        for seat in (0, 1):
                            c["game"]["seats"][seat]["deck"] = decks[seat]
                        path.write_text(json.dumps(c))
                        before = time.perf_counter_ns()
                        p = subprocess.run([str(binary), "bench", "--workload", "native-rollout-v1", "--config", str(path)], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=30, env=env)
                        sample = {"repeat": repeat, "policy": policy, "decks": decks, "starter": starter,
                                  "process_elapsed_ns": time.perf_counter_ns()-before, "exit_code": p.returncode,
                                  "stderr": p.stderr, "result": json.loads(p.stdout) if p.stdout else None}
                        receipt["samples"].append(sample)
                        # Keep every attempt, including failures; refresh the raw artifact.
                        output.seek(0)
                        json.dump(receipt, output, indent=2)
                        output.truncate()
                        output.flush()
        print(f"Recorded {len(receipt['samples'])} attempts; inspect outcomes before drawing conclusions.")


if __name__ == "__main__":
    main()
