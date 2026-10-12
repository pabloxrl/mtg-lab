"""Recheck inherited M2 evidence bytes and accounting; does not run game engines.

Run from the repository root. Semantic review and fresh execution are separate
gate obligations; hashes or recomputed statistics alone do not satisfy them.
"""
import collections
import hashlib
import json
from pathlib import Path
import re
import runpy
import subprocess
import tarfile
import tempfile

ROOT = Path(__file__).resolve().parents[3]


def read(path):
    return json.loads((ROOT / path).read_text())


def sha(path):
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def check_hashes(mapping):
    for path, digest in mapping.items():
        assert sha(path) == digest, path
    return len(mapping)


def check_archive(path, expected, digest):
    assert sha(path) == digest, path
    with tarfile.open(ROOT / path, 'r:gz') as archive:
        all_members = archive.getmembers()
        assert len({m.name for m in all_members}) == len(all_members), path
        for member in all_members:
            p = Path(member.name)
            assert (member.isfile() or member.isdir()) and not p.is_absolute() and '..' not in p.parts
        members = [m for m in all_members if m.isfile()]
        assert len(members) == len(expected), path
        assert {m.name for m in members} == set(expected), path
        for member in members:
            p = Path(member.name)
            assert member.isfile() and not p.is_absolute() and '..' not in p.parts
            row = expected[member.name]
            assert member.size == row['bytes'], member.name
            assert hashlib.sha256(archive.extractfile(member).read()).hexdigest() == row['sha256'], member.name
    return len(members)


def main():
    prefix = 'doc/evidence/m2-coverage-audit/'
    scenarios = read(prefix + 'reviewed-scenarios.json')['scenarios']
    assert len(scenarios) == len({s['id'] for s in scenarios}) == 123
    families = collections.Counter()
    for row in scenarios:
        assert row['count'] == 1 and row['basis'] and row['review']
        assert 'fn ' + row['native_test'] + '(' in (ROOT / row['source']).read_text()
        for key in ('source', 'fixture', 'oracle', 'reference_receipt'):
            if key in row:
                assert sha(row[key]) == row[key + '_sha256'], row['id']
        families.update(row['families'])
    assert dict(families) == read(prefix + 'reviewed-scenarios.json')['family_counts']
    assert len(families) == 12
    input_hashes = check_hashes(read(prefix + 'audit-inputs.json')['files'])
    inventory = read(prefix + 'catalog-inventory.json')['rows']
    current = {r['id']: r for r in read('doc/testing/capability-test-plan.json')['cases'] if r['milestone'] == 'M2'}
    assert len(inventory) == len(current) == 175
    assert {r['catalog']['id']: r['catalog'] for r in inventory} == current
    capabilities = read(prefix + 'capability-accounting.json')['capabilities']
    assert len(capabilities) == 58
    assert sum(r['native_executed'] for r in capabilities) == 175
    assert sum(r['xmage_executed'] for r in capabilities) == 173
    assert sum(r['xmage_unsupported'] for r in capabilities) == 2
    assert {i for r in capabilities for i in r['catalog_ids']} == set(current)
    registry = {c['id'] for c in read('data/capabilities-v1.json')['capabilities']}
    supplement = (ROOT / prefix / 'registry-supplement.md').read_text()
    supplemental_ids = set(re.findall(r'^\| `([^`]+)` \|', supplement, re.M))
    mapped_ids = {c['capability'] for c in capabilities}
    assert len(registry) == 80 and len(supplemental_ids) == 22
    assert not (mapped_ids & supplemental_ids) and mapped_ids | supplemental_ids == registry

    reference = read(prefix + 'fresh-reference.json')
    reference_sources = check_hashes(reference['execution']['source_sha256'])
    reference_files = check_archive(prefix + 'reference-runs.tar.gz', reference['archive_members'], reference['archive_sha256'])
    with tarfile.open(ROOT / prefix / 'reference-runs.tar.gz', 'r:gz') as archive:
        for row in scenarios:
            if 'fresh_reference' in row:
                ref = row['fresh_reference']
                assert hashlib.sha256(archive.extractfile(ref['receipt_member']).read()).hexdigest() == ref['receipt_sha256']

    admission = []
    cards = {c['id'] for c in read('data/cards/foundations_micro_v1.json')['cards'] if c['kind'] != 'token'}
    for n in (1, 2):
        base = 'doc/evidence/admission/'
        receipt = read(base + f'run-{n}.json')
        index = read(base + f'final-run-{n}.json')
        files = check_archive(base + f'final-run-{n}.tar.gz', {r['path']: r for r in index['files']}, index['sha256'])
        sources = check_hashes(receipt['sources'])
        assert receipt['status'] == 'agreed' and receipt['cases'] == 16 and receipt['runs_per_engine'] == 32
        assert set(receipt['played_cards']) == cards
        assert receipt['reference_semantics'] == 'patched-combat-sba-v1'
        admission.append(dict(run=n, archived_files=files, matching_sources=sources, distinct_games=16))

    measurement = ROOT / 'doc/evidence/four-mode-measurement'
    unpack = runpy.run_path(str(measurement / 'archive.py'))['unpack_verified']
    summarize = runpy.run_path(str(measurement / 'summarize.py'))['summarize']
    groups = {}
    windows = 0
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        unpack(measurement / 'throughput.tar.gz', measurement / 'throughput-index.json', root / 'throughput')
        unpack(measurement / 'privileged-diagnostics.tar.gz', measurement / 'diagnostics-index.json', root / 'diagnostics')
        summary = summarize(root / 'throughput', root / 'diagnostics')
        assert summary == read('doc/evidence/four-mode-measurement/summary.json')
        campaign = json.loads((root / 'throughput/campaign.json').read_text())
        for run in campaign['runs']:
            report = json.loads((root / 'throughput' / run['name'] / 'report.json').read_text())
            config = report['pins']['config']
            key = '/'.join((config['policies'][0], 'encoded' if config['encoding'] else 'native', config['instrumentation']))
            off = config['instrumentation'] == 'off'
            group = groups.setdefault(key, dict(elapsed_ns=0, begun=None if off else 0, committed=None if off else 0, cancelled=None if off else 0))
            for window in report['windows']:
                windows += 1
                assert window['elapsed_ns'] >= 30_000_000_000
                group['elapsed_ns'] += window['elapsed_ns']
                if not off:
                    for to, field in [('begun', 'logical_actions'), ('committed', 'committed_actions'), ('cancelled', 'cancelled_actions')]:
                        group[to] += window['counters'][field]
        for group in groups.values():
            group['committed_per_second'] = None if group['committed'] is None else group['committed'] * 1e9 / group['elapsed_ns']
        assert groups == read('doc/evidence/m2-instrumentation-audit/committed-actions.json')['groups']
    assert windows == 100 and summary['completed'] == summary['attempts'] == 7204

    resident_summary = read('doc/evidence/full-pool-baseline/resident/summary.json')
    for name, expected in resident_summary.items():
        report = read('doc/evidence/full-pool-baseline/resident/' + name + '.json')
        assert len(report['samples']) == 24 and len(report['released']) == 3
        assert report['specimen']['prefix_verified']
        first = [s for s in report['samples'] if s['cycle'] == 0]
        zero = next(s for s in first if s['resident'] == 0)
        full = next(s for s in first if s['resident'] == 10000)
        assert (full['rss_bytes'] - zero['rss_bytes']) / 10000 == expected['first_cycle_bytes_per_state']
        assert [s['rss_bytes'] for s in report['samples'] if s['resident'] == 10000] == expected['total_rss_at_10000']
        assert [s['rss_bytes'] for s in report['released']] == expected['released_rss']

    mutations = read(prefix + 'mutations.json')
    assert mutations['passed'] and len(mutations['cases']) == 12
    for case in mutations['cases']:
        assert case['baseline']['verdict'] == 'passed' and case['mutant']['verdict'] == 'detected'
    # Later report-only files can differ; the executed production/tests/fixtures,
    # reference inputs and mutation runner must still match their exact receipt.
    mutation_sources = {p: d for p, d in mutations['source_sha256'].items() if p.startswith(('crates/', 'fixtures/', 'references/', 'scripts/')) or p in ('Cargo.toml', 'Cargo.lock')}
    check_hashes(mutation_sources)
    result = dict(status='passed', audit_head=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  selected_scenarios=123, families=dict(families), input_hashes=input_hashes,
                  catalog_mappings=175, registry_ids=80, native_mappings=175, bounded_xmage_mappings=173,
                  reference_sources=reference_sources, reference_archive_files=reference_files,
                  admission_archives=admission, measurement_windows=windows,
                  measured_attempts=summary['attempts'], measured_completions=summary['completed'],
                  measured_decisions=summary['decisions'], measured_elapsed_ns=summary['elapsed_ns'],
                  committed_rate_groups=len(groups), inherited_detected_mutants=12,
                  resident_classes=len(resident_summary), resident_points=96,
                  matching_mutation_sources=len(mutation_sources),
                  limits='Integrity/accounting check of inherited execution, not new game/reference/mutation execution or semantic review.')
    # Fresh gate receipts remain distinct from inherited accounting above.
    gate = 'doc/evidence/m2-gate/'
    index = read(gate + 'execution-index.json')
    result['fresh_execution_files'] = check_archive(
        gate + 'execution.tar.gz', index['files'], index['archive_sha256'])
    execution = read(gate + 'execution.json')
    assert execution['status'] == 'passed'
    assert [r['name'] for r in execution['runs']] == [
        'build', 'cost', 'combat', 'trigger', 'foundation', 'admission',
        'mutations', 'quickstart-pass', 'quickstart-script', 'quickstart-native']
    assert all(r['exit_code'] == 0 for r in execution['runs'])
    result['fresh_matching_sources'] = check_hashes(execution['sources'])
    with tarfile.open(ROOT / gate / 'execution.tar.gz', 'r:gz') as archive:
        def archived(name):
            return json.load(archive.extractfile(name))
        assert archived('execution.json') == execution
        for row in execution['runs']:
            assert hashlib.sha256(archive.extractfile(row['name'] + '.log').read()).hexdigest() == row['log_sha256']
        admission = archived('admission.json')
        assert admission['status'] == 'agreed' and admission['cases'] == 16
        assert admission['runs_per_engine'] == 32 and set(admission['played_cards']) == cards
        mutants = archived('mutations/receipt.json')
        assert mutants['passed'] and len(mutants['cases']) == 12
        assert all(c['baseline']['verdict'] == 'passed' and c['mutant']['verdict'] == 'detected' for c in mutants['cases'])
    result['fresh_commands_passed'] = len(execution['runs'])
    result['fresh_terminal_games'] = 16
    result['fresh_detected_mutants'] = 12
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
