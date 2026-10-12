"""Verify retained raw-capture bytes; this does not perform replay admission."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
import xmage_ai_capture


def verify(full=False, current=False):
    manifest = json.loads((HERE / 'integrity.json').read_text())
    for name, expected in manifest['files'].items():
        path = HERE / name
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError('capture artifact checksum mismatch: ' + name)
    latest = HERE / manifest['latest']
    receipt = json.loads((latest / 'game/attempt.json').read_text())
    acceptance = json.loads((latest / 'acceptance.json').read_text())
    if receipt['status'] != 'complete' or acceptance['status'] != 'passed':
        raise ValueError('missing successful live-source acceptance')
    if current:
        for name, expected in receipt['pins'].items():
            if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != expected:
                raise ValueError('current execution source differs: ' + name)
    if full:
        observed = xmage_ai_capture.validate_trace(latest / 'game/trace.jsonl.xz')
        if observed != receipt['validation']:
            raise ValueError('raw trace accounting differs from receipt')
    return dict(files=len(manifest['files']), live_acceptance='passed', admitted_games=0)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--full', action='store_true')
    parser.add_argument('--current', action='store_true')
    args = parser.parse_args()
    print(json.dumps(verify(args.full, args.current)))
