"""Publish a short operator-facing note atomically from the issue checkout."""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import tempfile

try:
    from .operator_summary import validate_report
except ImportError:
    from operator_summary import validate_report


def write_report(root, issue, current, why, next_step, blocker=None):
    report = validate_report(dict(version=1, issue=issue,
        updated_at=datetime.now(timezone.utc).isoformat(), current=current, why=why,
        next=next_step, blocker=blocker))
    directory = root / '.agent-artifacts'
    directory.mkdir(exist_ok=True)
    if directory.resolve() != directory.absolute():
        raise ValueError('artifact directory must not be a symlink')
    fd, temporary = tempfile.mkstemp(prefix='operator-summary-', dir=directory)
    try:
        with os.fdopen(fd, 'w') as stream:
            json.dump(report, stream, ensure_ascii=False)
            stream.write('\n')
        os.replace(temporary, directory / 'operator-summary.json')
    finally:
        Path(temporary).unlink(missing_ok=True)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--issue', type=int, required=True)
    for name in ('current', 'why', 'next'):
        parser.add_argument('--' + name, required=True)
    parser.add_argument('--blocker')
    args = parser.parse_args()
    write_report(Path.cwd(), args.issue, args.current, args.why, args.next, args.blocker)
    print('Operator progress note updated.')


if __name__ == '__main__':
    main()
