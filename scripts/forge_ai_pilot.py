"""Forge pilot accounting: complete distinct games, never repeated executions.

The canonical identity is shared with both source lanes. Accounting is separate
from execution: callers must verify the source and actual replay receipts before
marking an attempt qualified. Failed attempts always remain in the ledger.
"""
import hashlib

from xmage_ai_pilot import canonical_game


def inventory(attempts):
    """Bind rows and duplicate accounting to content, preserving every attempt."""
    seen = {}
    ids = set()
    rows = set()
    ledger = []
    qualified = []
    for attempt in attempts:
        name = attempt['id']
        if not isinstance(name, str) or not name or name in ids:
            raise ValueError('unique nonempty attempt id required')
        ids.add(name)
        status = attempt['status']
        if status not in {'qualified', 'failed', 'truncated', 'unsupported', 'divergent', 'duplicate'}:
            raise ValueError('unknown attempt verdict')
        entry = dict(id=name, status=status, source=attempt['source'])
        if status != 'qualified':
            if not attempt.get('earliest_difference'):
                raise ValueError('unsuccessful attempt requires first difference')
            entry['earliest_difference'] = attempt['earliest_difference']
        transcript = attempt.get('transcript')
        if transcript is not None:
            canonical = canonical_game(transcript)
            digest = hashlib.sha256(canonical).hexdigest()
            previous = seen.get(digest)
            if previous and previous[1] != canonical:
                raise ValueError('canonical hash collision')
            entry['identity'] = digest
            if status == 'qualified':
                case = transcript['cases'][0]
                if case.get('stop') != 'terminal' or not case['play'] or case['play'][-1]['kind'] != 'finish':
                    raise ValueError('qualified game must be rules-terminal')
                row = (''.join(d['deck'][0].upper() for d in case['decks']), case['starter'])
                if row not in {(decks, seat) for decks in ('RG', 'GR', 'RR', 'GG') for seat in (0, 1)}:
                    raise ValueError('unknown frozen matchup row')
                entry.update(row=row[0], starter=row[1])
                if previous:
                    entry.update(status='duplicate', duplicate_of=previous[0],
                                 earliest_difference='identical canonical gameplay/chance')
                else:
                    seen[digest] = (name, canonical)
                    qualified.append(digest)
                    rows.add(row)
        elif status == 'qualified':
            raise ValueError('qualified attempt requires transcript')
        ledger.append(entry)
    return dict(attempts=ledger, qualified=qualified,
                rows=[dict(row=row, starter=seat) for row, seat in sorted(rows)])


def require_pilot(attempts):
    result = inventory(attempts)
    if len(result['qualified']) != 10:
        raise ValueError('ten distinct complete games required')
    if len(result['rows']) != 8:
        raise ValueError('all eight matchup/starter rows required')
    return result


def execute(source, output, forge_cache, xmage_cache):
    """Replay one immutable game twice in each reference and native consumer.

    This produces candidate evidence, not a ten-game verdict or final admission.
    Existing source-specific live controls remain separate required evidence.
    Every failed execution retains its inputs, outputs and first difference.
    """
    import argparse
    import json
    import time
    from pathlib import Path
    import forge_ai_normalize as normalize
    import forge_casting
    import forge_combat_reference as cross
    import xmage
    from instant_reference import difference

    source, output = Path(source).resolve(), Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    receipt = dict(schema_version=1, status='started', source=str(source),
                   source_sha256=xmage.sha(source), final_admission=False,
                   runner_sha256=xmage.sha(Path(__file__)))

    def write():
        (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')

    def need(ok, message):
        if not ok:
            raise ValueError(message)

    write()
    try:
        game = normalize.normalize(source)
        (output / 'normalized.json').write_text(json.dumps(game) + '\n')
        receipt['identity'] = game['identity']
        for repeat in (1, 2):
            folder = output / f'forge-{repeat}'
            result = forge_casting.run(argparse.Namespace(trace=source,
                cache=Path(forge_cache), output=folder, negative=None,
                empty_combat=False, stop_before=None, abilities=True, full_game=True))
            child = json.loads((folder / 'receipt.json').read_text())
            need(result == 0 and child['status'] == 'full-game-agreed',
                 f'Forge/native repetition {repeat}: {child.get("error")}')
            actual = json.loads((folder / 'native-fixture.json').read_text())
            need(difference([game['fixture']], actual) is None, 'executed source fixture changed')
            native = json.loads((folder / 'native.json').read_text())[0]
            normalize.verify_opening(game, native['opening'])
            forge = json.loads((folder / 'forge.json').read_text())
            need(forge['complete_game'], 'incomplete Forge replay')
        folder = output / 'cross'
        result = cross.run(argparse.Namespace(fixture=output / 'forge-1/native-fixture.json',
                                             cache=Path(xmage_cache), output=folder))
        child = json.loads((folder / 'receipt.json').read_text())
        need(result == 0 and child['status'] == 'native-xmage-agreed',
             'native/XMage: ' + str(child.get('error')))
        normalize.verify_opening(game, json.loads((folder / 'native.json').read_text())[0]['opening'])
        index = json.loads((folder / 'xmage-index.json').read_text())
        names = index[game['transcript']['cases'][0]['id']]
        need(len(names) == len(set(names)) == 2, 'two distinct XMage execution artifacts required')
        for name in names:
            normalize.verify_opening(game, json.loads((folder / name).read_text())['opening'])
        receipt.update(status='all-three-agreed', forge_repetitions=2,
                       native_invocations=3, native_repetitions=6, xmage_repetitions=2)
    except Exception as error:
        receipt.update(status='failed', earliest_difference=str(error))
    finally:
        receipt['elapsed_seconds'] = time.monotonic() - started
        receipt['artifacts'] = {str(p.relative_to(output)): xmage.sha(p)
                               for p in sorted(output.rglob('*'))
                               if p.is_file() and p != output / 'receipt.json'}
        write()
    return receipt


if __name__ == '__main__':
    import argparse
    import json
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--forge-cache', required=True)
    parser.add_argument('--xmage-cache', required=True)
    args = parser.parse_args()
    result = execute(args.source, args.output, args.forge_cache, args.xmage_cache)
    print(json.dumps({key: value for key, value in result.items() if key != 'artifacts'}))
    raise SystemExit(0 if result['status'] == 'all-three-agreed' else 2)
