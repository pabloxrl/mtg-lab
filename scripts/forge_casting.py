"""Strict, nonterminal Forge casting prefixes from immutable source captures."""
import gzip
import hashlib
import json

import forge_opening as opening

# Later choice families remain fail-closed endpoints; no automatic pass.
SUPPORTED = opening.OPENING | {
    'chooseSpellAbilityToPlay', 'playChosenSpellAbility', 'payManaCost',
    'resetAtEndOfTurn', 'autoPassCancel', 'cost.visit',
    'cost.paysRightAfterDecision', 'getCostDecisionMaker', 'chooseTargetsFor',
    'chooseModeForAbility', 'chooseOptionalCosts', 'orderCosts',
    'chooseCardsForCost', 'chooseCardsToDiscardFrom', 'getAbilityToPlay',
    'confirmAction',
}


def extract(path, verify_receipt=True, empty_combat=False, stop_before=None):
    result = opening.extract(path, verify_receipt=verify_receipt)
    opening.need(stop_before is None or stop_before >= result['checkpoint_sources'][-1],
                 0, 'casting endpoint before settled opening')
    prefix, paired, calls = [], {}, []
    remainder = hashlib.sha256()
    count = 0
    endpoint = None
    root_lines = []
    with gzip.open(path, 'rt') as stream:
        for index, line in enumerate(stream):
            if endpoint is not None:
                remainder.update(line.encode())
                count += 1
                continue
            row = json.loads(line)
            opening.need(row['index'] == index, index, 'record index')
            if calls or row['kind'] == 'enter':
                root_lines.append(line)
            if row['kind'] == 'enter':
                if index == stop_before or (row['method'] not in SUPPORTED and not (empty_combat and row['method'] == 'declareAttackers')):
                    opening.need(not calls, index, 'unsupported nested casting callback: ' + row['method'])
                    endpoint = row
                    remainder.update(line.encode())
                    count += 1
                    continue
                calls.append(row)
            elif row['kind'] == 'leave':
                opening.need(calls and calls[-1]['call'] == row['call'], index, 'callback pairing')
                entry = calls.pop()
                paired[entry['index']] = index
                if not calls:
                    unsupported = entry['method'] == 'declareAttackers' and bool(row['state']['combat'])
                    if entry['method'] == 'chooseSpellAbilityToPlay' and row['result']:
                        unsupported |= any(not a['spell'] and 'Land' not in a['host']['types'] for a in row['result'])
                    if unsupported:
                        endpoint = entry
                        prefix = prefix[:entry['index']]
                        paired = {a: b for a, b in paired.items() if a < entry['index']}
                        for raw in root_lines:
                            remainder.update(raw.encode())
                        count = len(root_lines)
                        continue
                    root_lines = []
            prefix.append(row)
    opening.need(endpoint is not None, len(prefix), 'missing bounded casting endpoint')
    if stop_before is not None:
        opening.need(endpoint['index'] == stop_before, len(prefix), 'requested boundary is not a supported root callback')
    result.update(prefix=prefix, paired=paired, endpoint=endpoint,
                  remainder=dict(first_record=endpoint['index'], records=count,
                                 sha256=remainder.hexdigest()))
    # The opening-only native input cannot certify the extended gameplay prefix.
    result.pop('native')
    result['scope'] = 'casting-prefix'
    return result


def install(cache):
    import tarfile
    import forge
    import forge_ai_capture as capture
    import forge_opening_patch
    source = cache / forge.SOURCE
    hashes, _ = capture.install(cache)
    with tarfile.open(cache / 'forge.tar.gz') as archive:
        original = archive.extractfile(forge.SOURCE + '/' + capture.patch.CONTROLLER).read().decode()
    modified = forge_opening_patch.controller(original).replace('MtgOpeningReplay', 'MtgCastingReplay')
    (source / capture.patch.CONTROLLER).write_text(modified)
    hashes[capture.patch.CONTROLLER]['modified'] = capture.sha_bytes(modified.encode())
    with tarfile.open(cache / 'forge.tar.gz') as archive:
        original = archive.extractfile(forge.SOURCE + '/' + capture.COSTS).read().decode()
    modified, inventory = capture.patch.instrument_cost_decisions(original)
    replaced = set()
    for method in inventory:
        parameters = method['signature'].split('(', 1)[1][:-1]
        args = ', '.join(p.split()[-1] for p in capture.patch.split_parameters(parameters))
        before = f"mtgRawOriginal_{method['name']}({args});"
        if before in replaced:
            continue
        replaced.add(before)
        opening.need(before in modified, 0, 'cost invocation seam')
        observed = args + ', ability' if args else 'ability'
        after = f'forge.game.MtgCastingReplay.choose(player, "cost.{method["name"]}", new Object[] {{{observed}}});'
        modified = modified.replace(before, after)
    (source / capture.COSTS).write_text(modified)
    hashes[capture.COSTS]['modified'] = capture.sha_bytes(modified.encode())
    recorder = (opening.ROOT / 'references/forge/MtgRawCapture.java').read_text()
    for signature, call in [
        ('public static void event(Game game, Event event) {', 'MtgCastingReplay.event(game,event);'),
        ('public static void boundary(Game game, String boundary) {', 'MtgCastingReplay.boundary(game,boundary);'),
        ('public static void shuffle(Player player, String stage) {', 'MtgCastingReplay.shuffle(player,stage);')]:
        opening.need(recorder.count(signature) == 1, 0, 'recorder hook')
        recorder = recorder.replace(signature, signature + '\n        ' + call)
    (source / capture.RECORDER).write_text(recorder)
    hashes[capture.RECORDER]['modified'] = capture.sha_bytes(recorder.encode())
    for local, target in [('MtgCastingReplay.java', 'forge-game/src/main/java/forge/game/MtgCastingReplay.java'),
                          ('CastingReplayTest.java', 'forge-gui-desktop/src/test/java/forge/gamesimulationtests/mtglab/CastingReplayTest.java')]:
        (source / target).write_bytes((opening.ROOT / 'references/forge' / local).read_bytes())
        hashes[target] = dict(original=None, modified=opening.xmage.sha(source / target))
    return hashes


NEGATIVES = ('target-incarnation', 'payment', 'pass-actor', 'callback-order',
             'response-order', 'token-power', 'effect-amount', 'mode', 'discard')


def corrupt(converted, fault):
    """Mutate only the validated consumer input; retain original capture bytes."""
    rows = converted['prefix']
    if fault == 'response-order':
        play = next(r for r in rows if r.get('method') == 'playChosenSpellAbility'
                    and r['arguments'][0]['spell'] and r['state']['stack'])
        choose = next(r for r in reversed(rows[:play['index']]) if r.get('method') == 'chooseSpellAbilityToPlay')
        choose['method'], play['method'] = play['method'], choose['method']
        return 'callback actor/method'
    if fault in ('token-power', 'effect-amount'):
        for row in rows:
            for player in row.get('state', {}).get('players', []):
                for card in player['zones']['Battlefield']:
                    if fault == 'token-power' and card['token']:
                        card['power'] += 1
                        return '/power'
                    if fault == 'effect-amount' and card['pt_boosts']:
                        card['pt_boosts'][0][2][0] += 1
                        return '/pt_boosts/0/2/0'
        raise ValueError('negative control has no matching source observation')
    if fault == 'mode':
        import copy
        entry = next(r for r in rows if r.get('method') == 'chooseModeForAbility')
        leave = rows[converted['paired'][entry['index']]]
        other = next(a for a in entry['arguments'][1] if a['api'] != leave['result'][0]['api'])
        leave['result'] = [copy.deepcopy(other)]
        return 'mode clone selection'
    if fault == 'discard':
        import copy
        play = next(r for r in rows if r.get('method') == 'playChosenSpellAbility'
                    and r['arguments'][0]['host']['name'] == 'Thrill of Possibility')
        entry = next(r for r in rows[play['index']:converted['paired'][play['index']]]
                     if r.get('method') == 'cost.visit' and r['arguments'][0].get('class') == 'forge.game.cost.CostDiscard')
        leave = rows[converted['paired'][entry['index']]]
        hand = entry['state']['players'][entry['actor']['player_id']]['zones']['Hand']
        other = next(c for c in hand if c['id'] != leave['result']['cards'][0]['id'])
        leave['result']['cards'] = [copy.deepcopy(other)]
        return 'state/players'
    if fault == 'target-incarnation':
        entry = next(r for r in rows if r.get('method') == 'chooseSpellAbilityToPlay'
                     and rows[converted['paired'][r['index']]]['result']
                     and rows[converted['paired'][r['index']]]['result'][0]['targets'])
        result = rows[converted['paired'][entry['index']]]['result'][0]
        result['targets'][0]['target']['timestamp'] += 100000
        return 'stale occurrence/incarnation'
    if fault == 'payment':
        entry = next(r for r in rows if r.get('method') == 'payManaCost'
                     and rows[converted['paired'][r['index']]]['arguments'][2]['paying_mana'])
        result = rows[converted['paired'][entry['index']]]['arguments'][2]
        result['paying_mana'][0]['color'] = 1
        return 'execution arguments'
    if fault == 'pass-actor':
        entry = next(r for r in rows if r.get('method') == 'chooseSpellAbilityToPlay'
                     and rows[converted['paired'][r['index']]]['result'] is None)
        entry['actor']['player_id'] = 1 - entry['actor']['player_id']
        return 'callback actor/method'
    if fault == 'callback-order':
        i = next(i for i, r in enumerate(rows) if r.get('method') == 'playChosenSpellAbility')
        rows[i], rows[i + 1] = rows[i + 1], rows[i]
        return 'record kind enter'
    raise ValueError('unknown negative control')


def run(args):
    import forge
    import forge_ai_capture as capture
    import forge_casting_native
    import os
    import time
    import xml.etree.ElementTree as ET
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    receipt = dict(status='started', complete_game=False, source_sha256=opening.xmage.sha(args.trace))
    started = time.monotonic()
    try:
        converted = extract(args.trace, empty_combat=args.empty_combat, stop_before=args.stop_before)
        expected = corrupt(converted, args.negative) if args.negative else None
        receipt['negative_control'] = args.negative
        (output / 'casting.json').write_text(json.dumps(converted) + '\n')
        cache = args.cache.resolve()
        opening.need(not cache.is_relative_to(opening.ROOT), 0, 'cache must be external')
        receipt['toolchain'] = capture.verify_toolchain(cache)
        receipt['patches'] = install(cache)
        receipt['implementation'] = {p: opening.xmage.sha(opening.ROOT / p) for p in (
            'scripts/forge_casting.py', 'scripts/forge_casting_native.py', 'scripts/forge_opening.py',
            'scripts/forge_opening_patch.py', 'references/forge/MtgCastingReplay.java', 'references/forge/CastingReplayTest.java')}
        receipt['native_sources'] = {str(p.relative_to(opening.ROOT)): opening.xmage.sha(p) for p in sorted((opening.ROOT / 'crates/mtg-core/src').glob('*.rs'))}
        receipt['native_sources']['Cargo.lock'] = opening.xmage.sha(opening.ROOT / 'Cargo.lock')
        source_receipt = json.loads(args.trace.with_name('receipt.json').read_text())
        replay_manifest = opening.ROOT / 'data/cards/foundations_micro_v1.json'
        if source_receipt.get('source_profile') == 'frozen-deck-selection-v1':
            replay_manifest = output / 'selected-manifest.json'
            replay_manifest.write_bytes(args.trace.with_name('selected-manifest.json').read_bytes())
        receipt['source_receipt_sha256'] = opening.xmage.sha(args.trace.with_name('receipt.json'))
        receipt['deck_selection'] = source_receipt['decks']
        command = opening.xmage.maven(cache) + ['-o', '-pl', 'forge-gui-desktop', '-am', 'test',
            '-Dtest=forge.gamesimulationtests.mtglab.CastingReplayTest', '-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.assets=' + str(cache / forge.SOURCE / 'forge-gui') + '/',
            '-Dmtglab.manifest=' + str(replay_manifest),
            '-Dmtglab.input=' + str(output / 'casting.json'), '-Dmtglab.output=' + str(output / 'forge.json')]
        if expected:
            command.append('-Dmtglab.expectedRejection=' + expected)
        receipt['command'] = command
        env = forge.environment(cache)
        env['MAVEN_OPTS'] = '-Xmx3g -Djava.awt.headless=true'
        report = cache / forge.SOURCE / 'forge-gui-desktop/target/surefire-reports/TEST-forge.gamesimulationtests.mtglab.CastingReplayTest.xml'
        report.unlink(missing_ok=True)
        opening.xmage.bounded(command, cache / forge.SOURCE, env, output / 'forge.log', 300)
        (output / 'test-report.xml').write_bytes(report.read_bytes())
        suite = ET.parse(report).getroot()
        opening.need(suite.get('tests') == '1' and all(suite.get(k) == '0' for k in ('failures', 'errors', 'skipped'))
                     and [c.get('name') for c in suite.findall('testcase')] == ['castingReplay'], 0, 'actual unskipped Forge test')
        opening.need(capture.source_diagnostic((output / 'forge.log').read_text()) is None, 0, 'source diagnostic')
        opening.need(opening.xmage.dependencies(cache) == json.loads((opening.ROOT / 'references/forge/dependencies.json').read_text()), 0, 'dependency inventory')
        if expected:
            actual = json.loads((output / 'forge.json').read_text())
            opening.need(actual['status'] == 'negative-detected' and expected in actual['reason'], 0, 'live negative result')
            receipt['status'] = 'negative-detected'
        else:
            native = forge_casting_native.fixture(converted, opening.extract(args.trace)['native'])
            (output / 'native-fixture.json').write_text(json.dumps([native]) + '\n')
            native_env = dict(os.environ, MTG_FORGE_CASTING_INPUT=str(output / 'native-fixture.json'),
                              MTG_FORGE_CASTING_OUTPUT=str(output / 'native.json'))
            opening.xmage.bounded(['cargo', 'test', '-p', 'mtg-core', '--locked', '--lib', 'captured_forge_casting_strict_replay'],
                                 opening.ROOT, native_env, output / 'native.log', 300)
            native_result = json.loads((output / 'native.json').read_text())
            opening.need(len(native_result) == 1 and native_result[0]['gameplay']['complete_game'] is False
                         and len(native_result[0]['gameplay']['consumed']) == native['actions'], 0, 'native exact consumption')
            receipt['status'] = 'casting-prefix-agreed'
    except (ValueError, OSError, KeyError, TypeError) as error:
        receipt.update(status='failed', error=str(error))
    receipt['elapsed_seconds'] = time.monotonic() - started
    receipt['artifacts'] = {p.name: opening.xmage.sha(p) for p in output.iterdir() if p.is_file()}
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(dict(status=receipt['status'], error=receipt.get('error'), output=str(output))))
    return 0 if receipt['status'] in ('casting-prefix-agreed', 'negative-detected') else 2


if __name__ == '__main__':
    import argparse
    from pathlib import Path
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache', type=Path, required=True)
    parser.add_argument('--trace', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--negative', choices=NEGATIVES)
    parser.add_argument('--empty-combat', action='store_true', help='Replay only explicit empty attacker declarations; nonempty combat remains unsupported')
    parser.add_argument('--stop-before', type=int, help='End at this exact source root callback; cannot cross an unsupported callback')
    raise SystemExit(run(parser.parse_args()))
