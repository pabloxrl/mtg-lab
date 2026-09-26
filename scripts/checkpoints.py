"""Compare neutral scenario assertions with supplied canonical checkpoints (no engine)."""
import argparse
import json
import re
from pathlib import Path

try:
    from . import scenario as s
except ImportError:
    import scenario as s


MISSING = object()


def difference(expected, actual, path=''):
    """First typed JSON difference: sorted object keys, ordered array indices."""
    if expected is MISSING or actual is MISSING:
        return dict(path=path, expected_present=expected is not MISSING,
                    actual_present=actual is not MISSING,
                    expected=None if expected is MISSING else expected,
                    actual=None if actual is MISSING else actual)
    if s.json_equal(expected, actual):
        return None
    if type(expected) is dict and type(actual) is dict:
        for key in sorted(expected.keys() | actual.keys()):
            escaped = key.replace('~', '~0').replace('/', '~1')
            result = difference(expected.get(key, MISSING), actual.get(key, MISSING), path+'/'+escaped)
            if result:
                return result
    if type(expected) is list and type(actual) is list:
        for i in range(max(len(expected), len(actual))):
            result = difference(expected[i] if i < len(expected) else MISSING,
                                actual[i] if i < len(actual) else MISSING, f'{path}/{i}')
            if result:
                return result
    return dict(path=path, expected_present=True, actual_present=True,
                expected=expected, actual=actual)


def validate_snapshot(snapshot):
    """Require captured evidence, never null/unavailable placeholders."""
    s.fields(snapshot, 'state rng decision private_information')
    # A complete neutral state, including zones, objects and hidden identities.
    schema = s.load(s.ROOT / 'schemas/neutral-scenario-v1.json')
    state_schema = schema['properties']['setup']['anyOf'][0]['properties']['state']
    s.shape(snapshot['state'], state_schema)
    s.validate_state(snapshot['state'], s.source_contract()[0])
    rng = snapshot['rng']
    s.fields(rng, 'algorithm state_hex')
    s.text(rng['algorithm'])
    s.require(type(rng['state_hex']) is str and
              re.fullmatch(r'(?:[0-9a-f]{2})+', rng['state_hex']), 'missing serialized RNG state')
    decision = snapshot['decision']
    s.fields(decision, 'id actor kind candidates')
    s.text(decision['id'])
    s.text(decision['kind'])
    s.require(type(decision['actor']) is int and decision['actor'] in (0, 1), 'missing decision actor')
    s.require(type(decision['candidates']) is list and decision['candidates'], 'missing decision candidates')
    for candidate in decision['candidates']:
        s.require((type(candidate) is str and candidate.strip()) or
                  (type(candidate) is dict and candidate), 'unavailable decision candidate')
    private = snapshot['private_information']
    s.fields(private, 'views')
    s.require(type(private['views']) is list and len(private['views']) == 2, 'both private views required')
    for seat, view in enumerate(private['views']):
        s.fields(view, 'seat observation')
        s.require(type(view['seat']) is int and view['seat'] == seat, 'private view seat mismatch')
        observation = view['observation']
        s.require(type(observation) is dict and
                  {'own_hand', 'opponent_hand_count', 'library_counts'} <= observation.keys(),
                  'missing private observation fields')
        s.require(type(observation['own_hand']) is list, 'missing own hand')
        for card in observation['own_hand']:
            s.text(card)
        s.require(type(observation['opponent_hand_count']) is int and observation['opponent_hand_count'] >= 0,
                  'missing opponent hand count')
        counts = observation['library_counts']
        s.require(type(counts) is list and len(counts) == 2 and
                  all(type(n) is int and n >= 0 for n in counts), 'missing library counts')


def compare(fixture, actual):
    """A pass certifies only supplied observations, never engine execution."""
    s.validate(fixture)
    s.digest(actual)
    s.fields(actual, 'checkpoint_version fixture_id fixture_revision initial consumed_script checkpoints invalid_results')
    s.require(type(actual['checkpoint_version']) is int and actual['checkpoint_version'] == 1,
              'unsupported checkpoint version')
    s.require(actual['fixture_id'] == fixture['fixture_id'] and
              actual['fixture_revision'] == fixture['provenance']['fixture_revision'], 'fixture identity mismatch')
    s.require(type(actual['initial']) is dict and type(actual['consumed_script']) is list,
              'invalid initial state or consumed script')
    points = actual['checkpoints']
    s.require(type(points) is list and len(points) == len(fixture['checkpoints']),
              'missing/extra checkpoints')
    for expected, point in zip(fixture['checkpoints'], points):
        s.fields(point, 'name after kind state')
        s.require(all(s.json_equal(point[k], expected[k]) for k in ('name', 'after', 'kind')),
                  'missing/reordered/misaligned checkpoint')
        s.require(type(point['state']) is dict, 'checkpoint state must be an object')
        if point['after'] == 'initial':
            s.require(s.json_equal(point['state'], actual['initial']), 'initial checkpoint conflicts with initial state')
    probes = actual['invalid_results']
    s.require(type(probes) is list and len(probes) == len(fixture['invalid_actions']),
              'missing/extra invalid-action results')
    for expected, probe in zip(fixture['invalid_actions'], probes):
        s.fields(probe, 'action at error before after')
        s.require(probe['at'] == expected['at'], 'misaligned invalid probe')
        for side in ('before', 'after'):
            validate_snapshot(probe[side])
    base = dict(fixture_id=fixture['fixture_id'], fixture_revision=actual['fixture_revision'])

    def mismatch(checkpoint, diff):
        return dict(base, status='mismatch', checkpoint=checkpoint, **diff)

    if fixture['setup']['kind'] == 'synthetic':
        expected_initial = fixture['setup']['state']
        observed = {k:v for k,v in actual['initial'].items() if k in expected_initial}
        diff = difference(expected_initial, observed)
        if diff:
            return mismatch('initial', diff)
    diff = difference(fixture['script'], actual['consumed_script'])
    if diff:
        return mismatch('script', diff)
    for checkpoint, point in zip(fixture['checkpoints'], points):
        for assertion in checkpoint['assertions']:
            try:
                value = s.pointer(point['state'], assertion['path'])
            except ValueError:
                value = MISSING
            diff = difference(assertion['expected'], value, assertion['path'])
            if diff:
                return dict(mismatch(checkpoint['name'], diff), field=assertion['field'], basis=assertion['basis'])
    for expected, probe in zip(fixture['invalid_actions'], probes):
        for field in ('action', 'error'):
            diff = difference(expected[field], probe[field], '/'+field)
            if diff:
                return mismatch('invalid/'+expected['action']['id'], diff)
        diff = difference(probe['before'], probe['after'])
        if diff:
            return mismatch('invalid/'+expected['action']['id'], diff)
    return dict(base, status='pass', scope='supplied-checkpoints-only')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('fixture', type=Path)
    parser.add_argument('actual', type=Path)
    parser.add_argument('--artifacts', type=Path, required=True,
                        help='new directory for privileged fixture, actual and diff artifacts')
    args = parser.parse_args()
    try:
        fixture, actual = s.load(args.fixture), s.load(args.actual)
        result = compare(fixture, actual)
        # Do not clobber an earlier failure or overwrite an input.
        args.artifacts.mkdir(parents=True, exist_ok=False)
        for name, value in (('fixture', fixture), ('actual', actual), ('diff', result)):
            (args.artifacts/(name+'.json')).write_text(json.dumps(value, indent=2, allow_nan=False)+'\n')
        result['artifacts'] = {name:str(args.artifacts/(name+'.json')) for name in ('fixture','actual','diff')}
        code = 0 if result['status'] == 'pass' else 1
    except (ValueError, OSError) as error:
        result, code = dict(status='error', message=str(error)), 2
    print(json.dumps(result, sort_keys=True, allow_nan=False))
    return code


if __name__ == '__main__':
    raise SystemExit(main())
