"""Supervise the pinned controller, read-only summary endpoint and nginx proxy."""
from pathlib import Path
import signal
import subprocess
import sys
import time


def main():
    control = Path(__file__).resolve().parents[2]
    stopping = False

    def stop(_signum, _frame):
        nonlocal stopping
        stopping = True

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    children = []
    try:
        for command in [
            ['symphony', '--i-understand-that-this-will-be-running-without-the-usual-guardrails', *sys.argv[1:]],
            [sys.executable, str(control / 'scripts/symphony/operator_summary.py')],
            ['nginx', '-c', str(control / 'docker/dashboard-nginx.conf'), '-g', 'daemon off;'],
        ]:
            children.append(subprocess.Popen(command))
        while not stopping and all(child.poll() is None for child in children):
            time.sleep(0.2)
        return 0 if stopping else 1
    finally:
        for child in children:
            if child.poll() is None:
                child.terminate()
        deadline = time.monotonic() + 20
        for child in children:
            try:
                child.wait(timeout=max(0, deadline - time.monotonic()))
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()


if __name__ == '__main__':
    raise SystemExit(main())
