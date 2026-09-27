"""Run standard unittest discovery; empty, skipped or expected-failure runs fail."""
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]


def complete_success(result):
    return (result.testsRun > 0 and result.wasSuccessful()
            and not result.skipped and not result.expectedFailures)


def main():
    sys.path.insert(0, str(ROOT))
    suite = unittest.defaultTestLoader.discover(str(ROOT / 'tests'), pattern='test_*.py')
    result = unittest.TextTestRunner(verbosity=1).run(suite)
    if not complete_success(result):
        print('Regression suite requires nonempty execution with no failures, skips or expected failures.',
              file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
