"""Verify immutable Forge pilot evidence and replay every selected game natively."""
import argparse
import hashlib
import gzip
import json
import lzma
import os
from pathlib import Path
import subprocess
import tempfile

import forge_ai_normalize as normalize
import forge_ai_pilot as pilot
import forge_combat_reference as cross

ROOT = Path(__file__).resolve().parents[1]
DEFAULT = ROOT / 'doc/evidence/forge-ai-pilot/pilot.json'
CONSUMERS = {
    'scripts/forge_ai_pilot.py', 'scripts/forge_ai_normalize.py', 'scripts/xmage_ai_pilot.py',
    'scripts/forge_casting_capture.py', 'scripts/forge_ai_capture.py',
    'references/forge/pins.json', 'references/forge/dependencies.json',
    'data/cards/foundations_micro_v1.json',
    'scripts/forge_casting.py', 'scripts/forge_casting_native.py',
    'scripts/forge_combat_reference.py',
    'references/forge/MtgCastingReplay.java', 'references/forge/CastingReplayTest.java',
    'references/xmage/FullPoolAdmissionTest.java', 'references/xmage/CombatSbaGame.java',
}

EXPECTED_CONTROLS = {
    'duplicate-occurrence': '/forge/44/duplicate occurrence',
    'mode': '/forge/5303/mode clone selection',
    'target-incarnation': '/forge/1458/stale occurrence/incarnation',
    'missing-chance': '/forge/3/record kind random',
    'payment': '/forge/772/execution arguments',
    'checkpoint': '/forge/2/state/players/0/life expected=19 actual=20',
}


def need(condition, message):
    if not condition:
        raise ValueError('Forge pilot: ' + message)


def artifact(manifest, name):
    path = Path(name)
    need(not path.is_absolute() and '..' not in path.parts, 'artifact path')
    need((ROOT / path).resolve().is_relative_to(ROOT), 'artifact outside repository')
    need(name in manifest['artifacts'], 'unlisted artifact ' + name)
    data = (ROOT / path).read_bytes()
    need(hashlib.sha256(data).hexdigest() == manifest['artifacts'][name],
         'artifact checksum ' + name)
    return data


def document(manifest, name):
    data = artifact(manifest, name)
    return json.loads(lzma.decompress(data) if name.endswith('.xz') else data)


def check_attempt_receipts(manifest, archived):
    """Every original capture/execution receipt must have one ledger outcome."""
    by_receipt = {}
    for attempt in manifest['attempts']:
        by_receipt.setdefault(attempt['original_receipt'], []).append(attempt)
    for name, entry in archived.items():
        if not name.endswith('/receipt.json'):
            continue
        data = gzip.decompress(artifact(manifest, entry['storage']))
        need(len(data) == entry['bytes'] and hashlib.sha256(data).hexdigest() == entry['sha256'],
             'original receipt bytes')
        receipt = json.loads(data)
        if 'runner_sha256' not in receipt and 'started_game' not in receipt:
            continue
        need(len(by_receipt.get(name, [])) == 1, 'complete attempt ledger: ' + name)
        attempt = by_receipt[name][0]
        if 'started_game' not in receipt:
            allowed = {'qualified', 'not-selected'} if receipt['status'] == 'all-three-agreed' else {'failed'}
            need(attempt['status'] in allowed, 'original execution verdict retained')
        else:
            allowed = {'not-selected', 'unsupported'} if receipt['status'] == 'complete' else {receipt['status']}
            need(attempt['status'] in allowed, 'original capture verdict retained')
            need(receipt['artifacts']['result.json.jsonl.gz']
                 == manifest['artifacts'][attempt['source']], 'original capture trace retained')


def check_retained(path=DEFAULT):
    manifest = json.loads(Path(path).read_text())
    need(manifest['schema_version'] == 1 and manifest['version'] == 'forge-ai-pilot-v1'
         and manifest['status'] == 'all-three-pilot',
         'pilot scope/version')
    need(manifest['final_admission'] == {'status': 'not-admitted', 'overlap_ids': []},
         'final admission claim')
    for name in manifest['artifacts']:
        artifact(manifest, name)
    index = document(manifest, manifest['attempt_index'])
    need(index['schema_version'] == 1, 'attempt archive version')
    archived = {}
    for entry in index['files']:
        name = entry['path']
        need(not Path(name).is_absolute() and '..' not in Path(name).parts
             and name not in archived, 'unique original archive path')
        need(entry['stored_sha256'] == manifest['artifacts'].get(entry['storage']),
             'archive storage binding')
        need(entry['bytes'] >= 0 and len(entry['sha256']) == 64, 'raw archive metadata')
        archived[name] = entry
    # These versioned consumers produced the retained live reference receipts.
    # Changing a consumer requires fresh acceptance, not blessing old outputs.
    need(CONSUMERS <= manifest['consumer_pins'].keys(), 'complete consumer pins')
    for name, digest in manifest['consumer_pins'].items():
        need(not Path(name).is_absolute() and '..' not in Path(name).parts
             and (ROOT / name).resolve().is_relative_to(ROOT), 'consumer path')
        need(hashlib.sha256((ROOT / name).read_bytes()).hexdigest() == digest,
             'changed reference consumer requires qualification: ' + name)
    selected = []
    fixtures = []
    for entry in manifest['games']:
        game = document(manifest, entry['normalized'])
        artifact(manifest, entry['source'])
        source = ROOT / entry['source']
        normalize.validate(game, source)
        receipt = document(manifest, entry['receipt'])
        need(receipt['status'] == 'all-three-agreed' and not receipt['final_admission'],
             'actual three-engine receipt')
        need(receipt['identity'] == game['identity'] == entry['id'], 'identity binding')
        for name, digest in receipt['artifacts'].items():
            original = entry['attempt'] + '/' + name
            need(original in archived and archived[original]['sha256'] == digest,
                 'complete original execution archive: ' + original)
        need(receipt['runner_sha256'] == manifest['consumer_pins']['scripts/forge_ai_pilot.py'],
             'executed pilot runner binding')
        need(receipt['source_sha256'] == manifest['artifacts'][entry['source']], 'original trace binding')
        need(receipt['forge_repetitions'] == receipt['xmage_repetitions'] == 2
             and receipt['native_invocations'] == 3 and receipt['native_repetitions'] == 6,
             'required repeated executions')
        for names in (entry['forge'], entry['native'], entry['references']):
            need(len(names) == len(set(names)) == 2, 'distinct execution artifacts')
        need(len(entry['forge_receipts']) == len(set(entry['forge_receipts'])) == 2,
             'two Forge execution receipts')
        for name in entry['forge_receipts'] + [entry['cross_receipt']]:
            child = document(manifest, name)
            need(child['status'] in {'full-game-agreed', 'native-xmage-agreed'},
                 'successful actual consumer invocation')
            required = ({'scripts/forge_casting.py', 'scripts/forge_casting_native.py',
                         'references/forge/MtgCastingReplay.java', 'references/forge/CastingReplayTest.java'}
                        if child['status'] == 'full-game-agreed' else
                        {'scripts/forge_combat_reference.py', 'references/xmage/FullPoolAdmissionTest.java',
                         'references/xmage/CombatSbaGame.java'})
            need(required <= child['implementation'].keys(), 'complete executed consumer pins')
            if child['status'] == 'full-game-agreed':
                need(child['complete_game'] and child['deck_selection']
                     == [d['deck'] for d in game['transcript']['cases'][0]['decks']],
                     'complete selected-deck execution')
            for implementation, digest in child['implementation'].items():
                if implementation in manifest['consumer_pins']:
                    need(digest == manifest['consumer_pins'][implementation],
                         'executed consumer binding ' + implementation)
        # Bind semantic outputs to the exact bytes hashed by each live run.
        for index, name in enumerate(entry['forge_receipts']):
            child = document(manifest, name)
            for output_name, original_name in ((entry['forge'][index], 'forge.json'),
                                               (entry['native'][index], 'native.json')):
                data = artifact(manifest, output_name)
                if output_name.endswith('.xz'):
                    data = lzma.decompress(data)
                need(hashlib.sha256(data).hexdigest() == child['artifacts'][original_name],
                     'live output receipt binding')
        child = document(manifest, entry['cross_receipt'])
        for output_name in entry['references']:
            data = artifact(manifest, output_name)
            if output_name.endswith('.xz'):
                data = lzma.decompress(data)
            original_name = Path(output_name).name.removesuffix('.xz')
            need(hashlib.sha256(data).hexdigest() == child['artifacts'][original_name],
                 'live reference output receipt binding')
        for index, name in enumerate(entry['forge_receipts']):
            child = document(manifest, name)
            need(child['source_sha256'] == receipt['source_sha256']
                 and child['source_receipt_sha256'] == game['provenance']['receipt_sha256'],
                 'Forge execution source binding')
            data = artifact(manifest, name)
            if name.endswith('.xz'):
                data = lzma.decompress(data)
            need(hashlib.sha256(data).hexdigest() == receipt['artifacts'][f'forge-{index+1}/receipt.json'],
                 'parent Forge receipt binding')
        data = artifact(manifest, entry['cross_receipt'])
        if entry['cross_receipt'].endswith('.xz'):
            data = lzma.decompress(data)
        need(hashlib.sha256(data).hexdigest() == receipt['artifacts']['cross/receipt.json'],
             'parent cross-reference receipt binding')
        fixture_bytes = (json.dumps([game['fixture']]) + '\n').encode()
        need(document(manifest, entry['cross_receipt'])['fixture_sha256']
             == hashlib.sha256(fixture_bytes).hexdigest(), 'executed shared fixture binding')
        captured = game['provenance']['capture_receipt']['validation']
        for name in entry['forge']:
            run = document(manifest, name)
            need(run['status'] == 'casting-replayed' and run['complete_game']
                 and run['consumed_records'] == captured['records']
                 and run['consumed_random'] == captured['kinds']['random'], 'Forge exact consumption')
            need(run['authored_ai_quota'] == 0, 'authored cases never fill quota')
        for native_name, reference_name in zip(entry['native'], entry['references'], strict=True):
            native = document(manifest, native_name)[0]
            reference = document(manifest, reference_name)
            normalize.verify_opening(game, native['opening'])
            normalize.verify_opening(game, reference['opening'])
            need(native['gameplay']['complete_game'], 'native complete game')
            cross.compare(game['fixture'], native['gameplay']['admission'], reference)
        selected.append(dict(id=entry['attempt'], source=entry['source'], status='qualified',
                             transcript=game['transcript']))
        fixtures.append(game['fixture'])
    counted = pilot.require_pilot(selected)
    need(counted['qualified'] == [g['id'] for g in manifest['games']], 'canonical roster')
    for row, entry in zip(counted['attempts'], manifest['games'], strict=True):
        need((row['row'], row['starter']) == (entry['row'], entry['starter']), 'reported row binding')
    import check_xmage_pilot
    prior = json.loads(check_xmage_pilot.DEFAULT.read_text())
    for entry in prior['games']:
        if entry['id'] in counted['qualified']:
            earlier = check_xmage_pilot.document(check_xmage_pilot.DEFAULT.parent, prior, entry['converted'])
            current = next(a['transcript'] for a, g in zip(selected, manifest['games'], strict=True)
                           if g['id'] == entry['id'])
            need(pilot.canonical_game(current) == pilot.canonical_game(earlier['transcript']),
                 'canonical hash collision')
            raise ValueError('Forge pilot: earlier XMage origin owns duplicate source quota')
    prior_forge = json.loads((ROOT / 'doc/evidence/forge-ai-normalize/execution.json').read_text())['identity']
    need(manifest['prior_all_three_overlap'] == sorted(set(counted['qualified']) & {prior_forge}),
         'explicit earlier Forge qualification overlap')
    ledger = manifest['attempts']
    need(len({a['id'] for a in ledger}) == len(ledger), 'unique attempt ledger')
    need({a['id'] for a in ledger if a['status'] == 'qualified'} == {a['id'] for a in selected},
         'qualified attempt accounting')
    selected_by_identity = {g['id']: g['attempt'] for g in manifest['games']}
    for attempt in ledger:
        need(attempt['kind'] in {'capture', 'normalization', 'execution'}, 'attempt kind')
        if attempt.get('same_game_as'):
            need(selected_by_identity.get(attempt.get('identity')) == attempt['same_game_as'],
                 'explicit repeated-game accounting')
        need(attempt['status'] in {'qualified', 'failed', 'truncated', 'unsupported',
                                  'divergent', 'duplicate', 'not-selected'}, 'attempt verdict')
        artifact(manifest, attempt['source'])
        need(attempt['original_receipt'] in archived, 'retained attempt receipt')
        if attempt.get('failure_log'):
            need(attempt['failure_log'] in archived, 'retained first difference log')
        if attempt['status'] != 'qualified':
            need(bool(attempt['earliest_difference']), 'retained first difference/reason')
    check_attempt_receipts(manifest, archived)
    need(set(manifest['negative_controls']) == set(EXPECTED_CONTROLS),
         'preserved live source controls')
    for fault, control in manifest['negative_controls'].items():
        receipt = document(manifest, control['receipt'])
        result = document(manifest, control['result'])
        need(receipt['status'] == result['status'] == 'negative-detected'
             and receipt['negative_control'] == fault
             and control['expected'] == EXPECTED_CONTROLS[fault]
             and EXPECTED_CONTROLS[fault] in result['reason'], 'live negative ' + fault)
        data = artifact(manifest, control['result'])
        if control['result'].endswith('.xz'):
            data = lzma.decompress(data)
        need(hashlib.sha256(data).hexdigest() == receipt['artifacts']['forge.json'],
             'live negative result binding')
        for implementation, digest in receipt['implementation'].items():
            need(manifest['consumer_pins'][implementation] == digest,
                 'live negative consumer binding')
    return manifest, fixtures


def replay_native(path=DEFAULT):
    manifest, fixtures = check_retained(path)
    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        source = root / 'fixtures.json'
        source.write_text(json.dumps(fixtures) + '\n')
        result = subprocess.run(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib',
                                 'captured_forge_combat_strict_replay'], cwd=ROOT,
            env=dict(os.environ, MTG_FORGE_COMBAT_INPUT=str(source),
                     MTG_FORGE_COMBAT_OUTPUT=str(root / 'native.json')),
            stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=300)
        need(result.returncode == 0, 'full native replay:\n' + result.stdout + result.stderr)
        runs = json.loads((root / 'native.json').read_text())
        need(len(runs) == len(fixtures) == 10, 'complete native roster')
        for entry, fixture, run in zip(manifest['games'], fixtures, runs, strict=True):
            game = document(manifest, entry['normalized'])
            normalize.verify_opening(game, run['opening'])
            reference = document(manifest, entry['references'][0])
            cross.compare(fixture, run['gameplay']['admission'], reference)
    return manifest


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest', type=Path, default=DEFAULT)
    parser.add_argument('--native', action='store_true')
    args = parser.parse_args()
    if args.native:
        replay_native(args.manifest)
    else:
        check_retained(args.manifest)
    print('Verified ten distinct Forge pilot games; no final corpus admission claim.')
