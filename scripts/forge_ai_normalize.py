"""Normalize immutable actual Forge AI games for strict three-engine replay.

The common gameplay/chance identity excludes source diagnostics. Source-derived
checkpoints and the original capture receipt remain mandatory replay evidence.
Normalization alone never certifies a game or admits it to a corpus.
"""
import copy
import hashlib
import json
from pathlib import Path

import forge_casting
import forge_casting_native
import forge_combat_reference
import forge_opening as opening
import xmage_ai_pilot as identities
from instant_reference import difference


def normalize(path):
    path = Path(path)
    converted = forge_casting.extract(path, abilities=True, full_game=True)
    fixture = forge_casting_native.fixture(converted, converted['opening'])
    transcript = forge_combat_reference.transcript(fixture)
    canonical = identities.canonical_game(transcript).decode()
    receipt = json.loads(path.with_name('receipt.json').read_text())
    aligned = converted['native_alignment'] + list(range(len(converted['choices']), len(converted['checkpoints'])))
    points = [dict(source=converted['checkpoint_sources'][i], state=converted['checkpoints'][i])
              for i in aligned]
    return dict(schema_version=1, identity=hashlib.sha256(canonical.encode()).hexdigest(),
                canonical=canonical, transcript=transcript, fixture=fixture, opening_checkpoints=points,
                provenance=dict(engine='forge', source_sha256=converted['source_sha256'],
                                receipt_sha256=opening.xmage.sha(path.with_name('receipt.json')),
                                capture_receipt=receipt))


def verify_opening(game, run):
    """Compare every source-observed declaration, bottom and first-upkeep point."""
    expected = game['opening_checkpoints']
    opening.need(len(run['points']) == len(expected), 0, 'opening checkpoint count')
    case = game['transcript']['cases'][0]
    for kind in ('chance', 'choices'):
        opening.need(run['consumed_'+kind] == case[kind], 0, 'opening exact '+kind)
    for index, (point, actual) in enumerate(zip(expected, run['points'])):
        delta = difference(point['state'], actual)
        opening.need(delta is None, point['source'], f'opening checkpoint/{index}: {delta}')
    return len(expected)


def validate(game, source):
    """Rebind all executable inputs and observations to the immutable source.

    A digest alone cannot establish that supplied checkpoints came from capture.
    Compare against fresh source translation, before invoking any consumer.
    """
    expected = normalize(source)
    delta = difference(expected, game)
    opening.need(delta is None, 0, 'normalized artifact: '+str(delta))
    return expected


def merge_origin(entries, transcript, origin):
    """One global content identity, all origins, deterministic first attribution.

    This is an identity index, never evidence that either origin was admitted.
    Callers keep capture/replay receipts in each separate origin record.
    """
    canonical = identities.canonical_game(transcript).decode()
    digest = hashlib.sha256(canonical.encode()).hexdigest()
    result = copy.deepcopy(entries)
    matches = [entry for entry in result if entry['identity'] == digest]
    opening.need(len(matches) <= 1, 0, 'duplicate identity index entry')
    if matches:
        entry = matches[0]
        opening.need(entry['canonical'] == canonical, 0, 'canonical hash collision')
        if origin not in entry['origins']:
            entry['origins'].append(copy.deepcopy(origin))
    else:
        result.append(dict(identity=digest, canonical=canonical,
                           first_origin=copy.deepcopy(origin), origins=[copy.deepcopy(origin)]))
    return result


def run(args):
    """Execute source-bound inputs twice in each actual delivered consumer."""
    import argparse
    import time
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    paths = [Path(__file__), opening.ROOT/'scripts/xmage_ai_pilot.py']
    implementation = {str(p.relative_to(opening.ROOT)):opening.xmage.sha(p) for p in paths}
    receipt = dict(status='started', admitted_games=0, repeats=[], implementation=implementation)
    def write(name, value):
        (output/name).write_text(json.dumps(value, indent=2)+'\n')
    write('receipt.json', receipt)
    try:
        game = normalize(args.source)
        write('normalized.json', game)
        write('canonical-game.json', json.loads(game['canonical']))
        write('native-fixture.json', [game['fixture']])
        write('transcript.json', game['transcript'])
        receipt.update(identity=game['identity'], source_sha256=game['provenance']['source_sha256'])
        for repeat in (1, 2):
            forge_dir = output/f'forge-{repeat}'
            options = argparse.Namespace(trace=args.source, cache=args.forge_cache, output=forge_dir,
                negative=None, empty_combat=False, stop_before=None, abilities=True, full_game=True)
            opening.need(forge_casting.run(options) == 0, 0, f'Forge repetition {repeat}')
            actual_fixture = json.loads((forge_dir/'native-fixture.json').read_text())
            delta = difference([game['fixture']], actual_fixture)
            opening.need(delta is None, 0, 'executed normalized fixture: '+str(delta))
            verify_opening(game, json.loads((forge_dir/'native.json').read_text())[0]['opening'])
            forge = json.loads((forge_dir/'forge.json').read_text())
            opening.need(forge['complete_game'], 0, 'Forge complete game')
            cross_dir = output/f'cross-{repeat}'
            opening.need(forge_combat_reference.run(argparse.Namespace(
                fixture=output/'native-fixture.json', cache=args.xmage_cache, output=cross_dir)) == 0,
                0, f'native/XMage repetition {repeat}')
            verify_opening(game, json.loads((cross_dir/'native.json').read_text())[0]['opening'])
            index = json.loads((cross_dir/'xmage-index.json').read_text())
            for name in index[game['transcript']['cases'][0]['id']]:
                verify_opening(game, json.loads((cross_dir/name).read_text())['opening'])
            cross = json.loads((cross_dir/'receipt.json').read_text())
            receipt['repeats'].append(dict(forge_records=forge['consumed_records'],
                forge_random=forge['consumed_random'], comparisons=cross['runs']))
        receipt['negative_controls'] = {}
        for fault in ('duplicate-occurrence', 'mode', 'target-incarnation', 'missing-chance', 'payment', 'checkpoint'):
            folder = output/('negative-'+fault)
            options = argparse.Namespace(trace=args.source, cache=args.forge_cache, output=folder,
                negative=fault, empty_combat=False, stop_before=None, abilities=True, full_game=True)
            opening.need(forge_casting.run(options) == 0, 0, 'live negative '+fault)
            negative = json.loads((folder/'receipt.json').read_text())
            opening.need(negative['status'] == 'negative-detected', 0, 'negative verdict '+fault)
            receipt['negative_controls'][fault] = json.loads((folder/'forge.json').read_text())
        # Record origin separately; this command does not publish an admitted corpus.
        write('identity-index.json', merge_origin([], game['transcript'], game['provenance']))
        opening.need(implementation == {str(p.relative_to(opening.ROOT)):opening.xmage.sha(p) for p in paths},
                     0, 'normalization implementation changed during execution')
        receipt['status'] = 'all-three-agreed'
    except Exception as error:
        receipt.update(status='failed', error=str(error))
        raise
    finally:
        receipt['elapsed_seconds'] = time.monotonic()-started
        receipt['artifacts'] = {str(p.relative_to(output)):opening.xmage.sha(p)
            for p in sorted(output.rglob('*')) if p.is_file() and p != output/'receipt.json'}
        write('receipt.json', receipt)
    return 0


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--forge-cache', type=Path, required=True)
    parser.add_argument('--xmage-cache', type=Path, required=True)
    raise SystemExit(run(parser.parse_args()))
