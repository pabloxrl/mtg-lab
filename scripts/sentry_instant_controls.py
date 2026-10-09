"""Original CR/Oracle-derived negative inputs for the bounded Sentry bridge."""
import copy
import json

POSITIVE = 'rules-foundations_micro_v1-bite-down-positive'
REGRESSION = 'rules-foundations_micro_v1-bite-down-regression'
BOOKKEEPING = 'rules-objects-bookkeeping-interaction'


def choice_mutants(document):
    """One-case inputs; expected rejection reasons belong to each real adapter."""
    base = next(c for c in document['cases'] if c['id'] == POSITIVE)
    for name, native, reference in [
        ('omitted-target', 'IllegalTarget', 'illegal target'),
        ('extra-target', 'unexpected target choice', 'choice order'),
        ('reordered-payment', 'payment requested before targets', 'choice order'),
        ('wrong-actor', 'WrongActor', 'wrong actor'),
        ('friendly-destination', 'IllegalTarget', 'illegal target'),
    ]:
        case = copy.deepcopy(base)
        script = case['script']
        if name == 'omitted-target': script.pop(4)
        elif name == 'extra-target': script.insert(6, copy.deepcopy(script[5]))
        elif name == 'reordered-payment': script[4], script[6] = script[6], script[4]
        elif name == 'wrong-actor': script[3]['actor'] = 1
        else: script[5]['target'] = 'source'
        yield name, {'version': 1, 'cases': [case]}, native, reference


def observation_mutants(observed):
    """Faults alter actual observations; independent expected states stay untouched."""
    for name, case, point, field, value in [
        ('toughness-as-health', POSITIVE, 2, 'toughness', 2),
        ('missing-damage', POSITIVE, 2, 'damage', 0),
        ('missing-growth', BOOKKEEPING, 4, 'power', 4),
        ('retained-damage', BOOKKEEPING, 12, 'damage', 2),
        ('retained-growth', BOOKKEEPING, 12, 'toughness', 7),
        ('cached-source-damage', REGRESSION, 4, 'damage', 2),
    ]:
        wrong = copy.deepcopy(observed)
        wrong[case]['checkpoints'][point]['state']['objects'][1][field] = value
        yield name, wrong, f'{case}.checkpoints[{point}].state.objects[1].{field}'
    for field, value in [('id', 'decoy0'), ('incarnation', 1)]:
        wrong = copy.deepcopy(observed)
        wrong[REGRESSION]['checkpoints'][3]['state']['stack'][0]['targets'][0][field] = value
        yield 'source-' + field, wrong, f'{REGRESSION}.checkpoints[3].state.stack[0].targets[0].{field}'
    case = 'rules-continuous-resolution-power-positive'
    wrong = copy.deepcopy(observed)
    wrong[case]['checkpoints'][2]['state']['stack'].reverse()
    yield 'stack-order', wrong, f'{case}.checkpoints[2].state.stack[0]'


def run(instant, cache, output, document, observed):
    receipts = {}
    for name, mutant, native, reference in choice_mutants(document):
        path = output / ('sentry-' + name + '-input.json')
        path.write_text(json.dumps(mutant, indent=2) + '\n')
        receipts[name] = {}
        for engine, reason in [('native', native), ('xmage', reference)]:
            stem = 'sentry-' + name + '-' + engine
            log = output / (stem + '.log')
            try:
                if engine == 'native': instant.run_native(output / (stem + '.json'), log, path)
                else: instant.run_xmage(cache, output / stem, log, path)
            except ValueError:
                instant.xmage.scenario.require(reason in log.read_text(), 'unrelated Sentry control failure: ' + stem)
                receipts[name][engine] = {'status': 'rejected', 'reason': reason,
                    'input_sha256': instant.xmage.sha(path), 'log_sha256': instant.xmage.sha(log)}
            else:
                raise ValueError('Sentry choice mutant survived: ' + stem)
    # Killing a different same-name Cub really executes, but cannot satisfy the
    # selected-source departure checkpoint. No original expectation is edited.
    case = copy.deepcopy(next(c for c in document['cases'] if c['id'] == REGRESSION))
    next(a for a in reversed(case['script']) if a['kind'] == 'target')['target'] = 'decoy0'
    path = output / 'sentry-retarget-input.json'
    path.write_text(json.dumps({'version': 1, 'cases': [case]}, indent=2) + '\n')
    expected = json.loads(instant.EXPECTATIONS.read_text())[REGRESSION]
    receipts['retarget'] = {}
    for engine in ('native', 'xmage'):
        stem = 'sentry-retarget-' + engine
        log = output / (stem + '.log')
        result = (instant.run_native(output / (stem + '.json'), log, path) if engine == 'native'
                  else instant.run_xmage(cache, output / stem, log, path))
        diff = instant.difference(expected, result[REGRESSION]['checkpoints'])
        instant.xmage.scenario.require(diff is not None and diff['path'] == '$[2].state.stack[1].targets[1].id',
                                      'Sentry retarget lost its first divergence')
        (output / (stem + '-divergence.json')).write_text(json.dumps(diff, indent=2) + '\n')
        receipts['retarget'][engine] = {'status': 'detected', 'difference': diff,
            'input_sha256': instant.xmage.sha(path), 'log_sha256': instant.xmage.sha(log)}
    faults = {}
    for name, wrong, required_path in observation_mutants(observed):
        try:
            instant.compare(document, wrong)
        except ValueError as error:
            instant.xmage.scenario.require(required_path in str(error), 'unrelated Sentry observation divergence')
            faults[name] = str(error)
        else:
            raise ValueError('Sentry observation mutant survived: ' + name)
    (output / 'sentry-observation-divergences.json').write_text(json.dumps(faults, indent=2) + '\n')
    receipts['observations'] = faults
    return receipts
