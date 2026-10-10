"""Execute the real fixed-trace native diagnostic client; independently audit costs."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
CATEGORIES = {'reset', 'application', 'legality_and_view', 'encoding', 'finalization', 'shared'}


def validate(report):
    if report.get('schema_version') != 1 or report.get('status') != 'measured':
        raise ValueError('version/status')
    costs = report['costs']
    if set(costs) != CATEGORIES:
        raise ValueError('missing/extra cost category')
    for cost in costs.values():
        for field in ('ns', 'calls', 'allocations', 'allocated_bytes'):
            if type(cost[field]) is not int or cost[field] < 0:
                raise ValueError('invalid cost')
    if sum(c['ns'] for c in costs.values()) != report['elapsed_ns']:
        raise ValueError('overlapping or missing duration')
    for field in ('allocations', 'allocated_bytes'):
        if sum(c[field] for c in costs.values()) != report[field]:
            raise ValueError('overlapping or missing allocations')
    if report['policy_ns'] != 0 or report['observations'] <= 0 or report['encoding']['bytes'] <= 0:
        raise ValueError('script must generate observations/encoding without a policy')
    if report['legal_candidates'] <= 0 or not report['decision_kinds']:
        raise ValueError('empty legal observations')
    if costs['legality_and_view']['calls'] != report['observations']:
        raise ValueError('observation count')
    if costs['encoding']['calls'] != report['observations']:
        raise ValueError('encoding count')
    if report['validated_checkpoints'] != report['consumed'] or report['consumed'] <= 0:
        raise ValueError('trace completeness')
    for name in ('trace_sha256', 'source_sha256', 'binary_sha256', 'toolchain_sha256', 'history_sha256'):
        if len(report[name]) != 64 or any(c not in '0123456789abcdef' for c in report[name]):
            raise ValueError('missing pin')
    if report['effect_probe']['scope'] != 'isolated pending Work::Modify via Game::resume(1); separate experiment':
        raise ValueError('effect attribution boundary')
    effect = report['effect_probe']
    if effect['dispatches'] != len(effect['samples']) or effect['dispatches'] != effect['cost']['calls']:
        raise ValueError('effect dispatch count')
    if sum(s['ns'] for s in effect['samples']) != effect['cost']['ns']:
        raise ValueError('effect duration')
    if any(s['before_sha256'] == s['after_sha256'] for s in effect['samples']):
        raise ValueError('effect did not change witnessed state')


class FixedTraceProfile(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        # Invocation owns the mandatory heavy lock, just as for the other native adapters.
        result = subprocess.run(['cargo', 'test', '-p', 'mtg-cli', '--locked', '--no-run',
                                 '--message-format=json'], cwd=ROOT, capture_output=True, text=True, check=True)
        cls.binary = next(Path(row['executable']) for line in result.stdout.splitlines()
                          if (row := json.loads(line)).get('reason') == 'compiler-artifact'
                          and row.get('executable') and row['profile']['test']
                          and row['target']['name'] == 'mtg')

    def execute(self, trace):
        data = json.dumps(trace).encode()
        with tempfile.TemporaryDirectory() as temp:
            inp, out = Path(temp) / 'input.json', Path(temp) / 'output.json'
            inp.write_bytes(data)
            subprocess.run([str(self.binary), 'profile::profile_export', '--exact', '--test-threads=1'],
                           cwd=ROOT, env={**os.environ, 'MTG_PROFILE_INPUT': str(inp), 'MTG_PROFILE_OUTPUT': str(out)},
                           capture_output=True, text=True, check=True, timeout=120)
            report = json.loads(out.read_text())
        if destination := os.environ.get('MTG_PROFILE_ARTIFACT_DIR'):
            directory = Path(destination)
            directory.mkdir(parents=True, exist_ok=True)
            key = f'{self._testMethodName}-{hashlib.sha256(data).hexdigest()}'
            (directory / f'{key}.input.json').write_bytes(data)
            (directory / f'{key}.report.json').write_text(json.dumps(report, indent=2) + '\n')
        if 'error' not in report:
            self.assertEqual(report['trace_sha256'], hashlib.sha256(data).hexdigest())
            self.assertEqual(report['binary_sha256'], hashlib.sha256(self.binary.read_bytes()).hexdigest())
            for field, path in [('dependency_sha256', 'Cargo.lock'),
                                ('rules_sha256', 'data/rules/cr-2026-09-25.json'),
                                ('cards_sha256', 'data/cards/foundations_micro_v1.json')]:
                self.assertEqual(report[field], hashlib.sha256((ROOT / path).read_bytes()).hexdigest())
            self.assertEqual(report['toolchain_sha256'], hashlib.sha256(report['toolchain'].encode()).hexdigest())
        return report

    def fixture(self, name):
        return json.loads((ROOT / f'fixtures/profile/{name}.json').read_text())

    def test_real_native_traces_validate_and_measure(self):
        for name in ('control', 'combat', 'effects'):
            with self.subTest(trace=name):
                report = self.execute(self.fixture(name))
                validate(report)
                self.assertEqual(report['effect_probe']['dispatches'] >= 2, name == 'effects')
                self.assertGreater(report['costs']['application']['allocations'], 0)
                if name == 'effects':
                    self.assertEqual(report['decision_kinds']['trigger_order'], 2)
                if name == 'combat':
                    self.assertEqual(report['decision_kinds']['combat_damage'], 9)
                    self.assertEqual(report['decision_kinds']['cleanup_discard'], 13)

    def test_all_modes_and_capture_preserve_semantics_and_player_inputs(self):
        baseline = None
        capture_baseline = None
        for mode in ('off', 'counters', 'sampled_trace', 'full_replay'):
            for capture in (False, True):
                trace = self.fixture('effects')
                trace['config']['native']['instrumentation'] = mode
                trace['capture'] = capture
                report = self.execute(trace)
                validate(report)
                state = [report[k] for k in ('history_sha256', 'observations_sha256', 'final')]
                if baseline is None:
                    baseline = state
                self.assertEqual(state, baseline)
                self.assertEqual(report['captured_decisions'], 155 if capture else 0)
                self.assertIn('capture_sha256', report, 'capture equality requires actual canonical payload evidence')
                if capture:
                    self.assertIsNotNone(report['capture_sha256'])
                    if capture_baseline is None:
                        capture_baseline = report['capture_sha256']
                    self.assertEqual(report['capture_sha256'], capture_baseline)
                else:
                    self.assertIsNone(report['capture_sha256'])
                self.assertEqual(report['replay_bytes'] > 0, mode == 'full_replay')

    def test_real_client_rejects_mutated_input(self):
        original = self.fixture('combat')
        for mutation in ('checkpoint', 'pin', 'missing', 'extra', 'unfinished', 'unvalidated', 'target'):
            trace = copy.deepcopy(original)
            records = trace['config']['script']['records']
            if mutation == 'checkpoint': trace['checkpoints'][0]['life'][0] = 19
            elif mutation == 'pin': trace['engine'] = '0'*64
            elif mutation == 'missing': records.pop()
            elif mutation == 'extra': records.append(copy.deepcopy(records[-1]))
            elif mutation == 'unfinished':
                records.pop()
                trace['checkpoints'].pop()
            elif mutation == 'unvalidated': trace['validated'] = False
            else:
                target = json.loads(records[82]['record'])
                target['choices'][0]['card']['incarnation'] = 99
                records[82]['record'] = json.dumps(target)
            with self.subTest(mutation=mutation):
                self.assertIn('error', self.execute(trace))

    def test_real_export_rejects_accounting_tampering(self):
        report = self.execute(self.fixture('control'))
        validate(report)
        for mutation in ('overlap', 'missing', 'extra', 'policy', 'observation'):
            bad = copy.deepcopy(report)
            if mutation == 'overlap': bad['costs']['application']['ns'] += 1
            elif mutation == 'missing': del bad['costs']['encoding']
            elif mutation == 'extra': bad['costs']['effect'] = bad['costs']['application']
            elif mutation == 'policy': bad['policy_ns'] = 1
            else: bad['observations'] = 0
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                validate(bad)
