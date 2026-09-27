"""Validate test designs, independently of authored fixtures/execution evidence.

A complete design never establishes engine support. Scenario execution evidence
continues to be checked by scenario.py, including its require-passed gate.
"""
import json
from pathlib import Path
import re

try:
    from scripts import scenario
except ModuleNotFoundError:
    import scenario

ROOT = Path(__file__).resolve().parents[1]
PLAN = 'doc/testing/capability-test-plan.json'
CATEGORIES = {'positive', 'negative', 'interaction', 'regression'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def fields(value, names):
    require(type(value) is dict and set(value) == set(names.split()),
            'unexpected or missing test-plan fields')


def validate(plan, registry, program, root=ROOT):
    fields(plan, 'plan_version scope status rules cards cases authored_fixtures')
    require(type(plan['plan_version']) is int and plan['plan_version'] == 1,
            'unsupported test plan version')
    require(plan['scope'] == registry['scope'] == 'foundations_micro_v1', 'wrong plan scope')
    require(plan['status'] == 'design-only', 'test plan must not claim execution')
    _, rules, cards = scenario.source_contract()
    require(plan['rules'] == rules and plan['cards'] == cards, 'stale rules/card design pins')
    caps = scenario.validate_registry_structure(registry)
    tasks = {task['issue']: task for task in program['tasks']}
    require(type(plan['cases']) is list and plan['cases'], 'missing planned cases')
    ids, slots = set(), set()
    for case in plan['cases']:
        fields(case, 'id capability_id category setup actions expected basis owner_issue milestone')
        identifier = case['id']
        require(type(identifier) is str and re.fullmatch(r'[a-z0-9][a-z0-9/_.-]*', identifier),
                'invalid planned test ID')
        require(identifier not in ids, 'duplicate planned test ID')
        ids.add(identifier)
        cap, category = case['capability_id'], case['category']
        require(type(cap) is str and cap in caps, 'unresolved capability')
        require(type(category) is str and category in CATEGORIES, 'unknown test category')
        require((cap, category) not in slots, 'duplicate capability/category mapping')
        slots.add((cap, category))
        for name in ('setup', 'actions', 'expected', 'basis'):
            require(type(case[name]) is str and bool(case[name].strip()), f'missing case {name}')
        owner = case['owner_issue']
        require(type(owner) is int and owner in tasks, 'unresolved implementation owner')
        require(tasks[owner]['kind'] == 'implementation', 'test owner must implement behavior')
        require(case['milestone'] == tasks[owner]['milestone'], 'owner/stage mismatch')
    expected = {(cap, category) for cap in caps for category in CATEGORIES}
    require(slots == expected, 'missing capability/category mappings: ' + str(sorted(expected - slots)))

    # Related authored inputs are not assertions that a planned case has run.
    require(type(plan['authored_fixtures']) is list, 'invalid authored fixture index')
    linked = set()
    for link in plan['authored_fixtures']:
        fields(link, 'path fixture_id fixture_revision relation capability_ids')
        require(link['relation'] == 'related-authored-input-not-execution', 'fixture link claims execution')
        path = link['path']
        require(type(path) is str and path.startswith('fixtures/scenarios/')
                and '..' not in Path(path).parts and Path(path).suffix == '.json', 'invalid fixture path')
        require(path not in linked, 'duplicate authored fixture link')
        linked.add(path)
        require((root / path).is_file(), 'unresolved authored fixture')
        fixture = json.loads((root / path).read_text())
        scenario.validate(fixture, registry)
        require(fixture['provenance']['review']['status'] == 'accepted-for-m0', 'unadmitted authored fixture')
        require(link['fixture_id'] == fixture['fixture_id'], 'wrong authored fixture ID')
        require(link['fixture_revision'] == fixture['provenance']['fixture_revision'], 'stale authored fixture revision')
        require(link['capability_ids'] == fixture['required_capabilities'], 'wrong authored fixture capabilities')
    admitted = {str(p.relative_to(root)) for p in (root / 'fixtures/scenarios').rglob('*.json')
                if json.loads(p.read_text())['provenance']['review']['status'] == 'accepted-for-m0'}
    require(linked == admitted, 'missing admitted authored fixture links')
    return {'capabilities': len(caps), 'designed_slots': len(slots),
            'related_authored_fixtures': len(linked), 'execution_evidence_added': 0}


def main():
    try:
        result = validate(json.loads((ROOT / PLAN).read_text()),
                          json.loads((ROOT / 'data/capabilities-v1.json').read_text()),
                          json.loads((ROOT / 'doc/programs/rfc-0002.json').read_text()))
        print(json.dumps(result, sort_keys=True))
    except (ValueError, OSError) as error:
        raise SystemExit(str(error)) from error


if __name__ == '__main__':
    main()
