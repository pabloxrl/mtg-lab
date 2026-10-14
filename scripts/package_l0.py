"""Materialize the immutable provisional L0 package from independently qualified evidence.

Packaging verifies the original paired source/native assertions. It never executes
native code to generate expected states. Ordinary L0 uses only the resulting files.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

import check_xmage_pilot as pilot

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'fixtures/l0/xmage-pilot-v1.json'


def encoded(value):
    return (json.dumps(value, sort_keys=True, separators=(',', ':')) + '\n').encode()


def semantic_report(report):
    # ObjectStore identity is a process-local arena nonce, not physical identity.
    # Keep slot/generation and the stable occurrence/birth/incarnation bindings.
    import copy
    report = copy.deepcopy(report)
    for key in ('policy_capture', 'stale_candidates_rejected', 'continuation_rejections', 'stops'):
        del report[key]
    stores = {}
    def canonical_handle(item):
        handle = item.get('raw_source_handle')
        if isinstance(handle, dict):
            handle['store'] = stores.setdefault(handle['store'], len(stores))
    for point in report['points']:
        for item in point['stack']:
            canonical_handle(item)
    for item in report['trigger_provenance']:
        canonical_handle(item)
    return report


def contents():
    manifest = pilot.check_retained()
    base = pilot.DEFAULT.parent
    files = {'qualification.json': pilot.DEFAULT.read_bytes()}
    games = []
    for game in manifest['games']:
        converted = pilot.document(base, manifest, game['converted'])
        expected = semantic_report(pilot.document(base, manifest, game['native'][0])['run'])
        other = semantic_report(pilot.document(base, manifest, game['native'][1])['run'])
        if expected != other:
            raise ValueError('qualified scalar repeats differ')
        prefix = game['attempt']
        tape = prefix + '/input.json'
        oracle = prefix + '/expected.json'
        trace = prefix + '/trace.jsonl.xz'
        files[tape] = encoded(converted['transcript'])
        files[oracle] = encoded(expected)
        files[trace] = pilot.artifact(base, manifest, game['source'])
        files[prefix + '/converted.json'] = encoded(converted)
        files[prefix + '/reference.json'] = encoded(pilot.document(base, manifest, game['reference'][0]))
        games.append(dict(id=game['id'], seed=converted['source_receipt']['seed'],
                          input=tape, expected=oracle, trace=trace,
                          source_cursors=converted['source_cursors'],
                          actions=len(converted['transcript']['cases'][0]['play']),
                          checkpoints=len(expected['points'])))
    result = dict(schema_version=1, version='xmage-pilot-l0-v1',
                  status='provisional-native-patched-xmage', final_admitted_games=0,
                  semantic_schema='admission-v10-extended-v1', games=games,
                  game_count=len(games),
                  qualification_sha256=hashlib.sha256(files['qualification.json']).hexdigest(),
                  artifacts={name:hashlib.sha256(data).hexdigest() for name,data in sorted(files.items())})
    return result, files


def package(output):
    manifest, files = contents()
    if encoded(manifest) != MANIFEST.read_bytes():
        raise ValueError('package differs from immutable release manifest')
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    for name, data in files.items():
        path = output / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    shutil.copyfile(MANIFEST, output / 'manifest.json')
    return output / 'manifest.json'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    print(package(args.output))
