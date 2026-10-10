"""B020/B021 real native execution receipts, with independent arithmetic controls."""
import copy
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class FourModeContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.directory = tempfile.TemporaryDirectory()
        output = Path(cls.directory.name) / 'native.json'
        env = dict(os.environ, MTG_FOUR_MODE_TEST_OUTPUT=str(output))
        command = ['cargo', 'test', '--locked', '-p', 'mtg-cli',
                   'four_modes_export_native_contract', '--', '--nocapture']
        fd = env.get('MTG_SYMPHONY_LOCK_FD')
        if fd is None:
            control = Path(env.get('SYMPHONY_CONTROL_ROOT', ROOT))
            command = [sys.executable, str(control / 'scripts/symphony/resource_lock.py'),
                       'heavy', '--', *command]
        run = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True,
                             pass_fds=() if fd is None else (int(fd),), timeout=1800)
        if run.returncode:
            raise AssertionError('real native exporter failed: ' + run.stdout[-4000:] + run.stderr[-4000:])
        if not output.is_file():
            raise AssertionError('native exporter must produce nonempty execution evidence')
        cls.receipt = json.loads(output.read_text())
        if os.environ.get('MTG_FOUR_MODE_EVIDENCE'):
            Path(os.environ['MTG_FOUR_MODE_EVIDENCE']).write_bytes(output.read_bytes())

    @classmethod
    def tearDownClass(cls):
        cls.directory.cleanup()

    def validate(self, receipt):
        from scripts.scalar_modes import validate_execution
        return validate_execution(receipt)

    def test_real_normal_reset_four_modes_and_capture(self):
        result = self.validate(self.receipt)
        self.assertEqual(result['runs'], 64)
        self.assertGreater(result['observations'], 0)

    def test_missing_mode_row_version_or_observations_rejected(self):
        for change in ('mode', 'row', 'version', 'observations'):
            value = copy.deepcopy(self.receipt)
            if change == 'mode':
                value['runs'] = [r for r in value['runs'] if r['mode'] != 'full_replay']
            elif change == 'row':
                value['runs'].pop()
            elif change == 'version':
                value['schema_version'] = 1
            else:
                value['runs'][0]['observations'] = 0
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.validate(value)

    def test_tampered_denominators_and_incomplete_replay_rejected(self):
        for change in ('elapsed', 'attempts', 'failed', 'replay', 'drops', 'history'):
            value = copy.deepcopy(self.receipt)
            row = next(r for r in value['runs'] if r['mode'] == 'full_replay')
            if change == 'elapsed': row['window']['elapsed_ns'] = 0
            elif change == 'attempts': row['window']['attempts'] = 0
            elif change == 'failed': row['window']['failed'] = 1
            elif change == 'replay': row['window']['execution']['replay_verified'] = 0
            elif change == 'history': row['history_sha256'] = '0' * 64
            else:
                row = next(r for r in value['runs'] if r['mode'] == 'sampled_trace')
                row['window']['execution']['trace_dropped'] += 1
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.validate(value)


class FourModeArithmetic(unittest.TestCase):
    def test_new_version_preserves_independent_window_ledger(self):
        # B020/#216's independent 60 completions per 30s, five windows.
        from test_scalar_artifact import example
        from scripts.scalar_artifact import validate_report
        value = example()
        value['schema_version'] = 2
        value['workload'] = 'scalar-four-modes-v1'
        value['pins']['config'].update(schema_version=2, workload=value['workload'])
        value['pins']['execution'] = dict(replay=dict(max_bytes=67108864, max_records=20000,
            persistence='in_memory'), trace=None, capture='none', sampled_timing='not_measured')
        for row in [value['warmup'], *value['windows']]:
            row['execution'] = dict(not_started=0, trace_selected=0, trace_retained=0,
                trace_dropped=0, replay_verified=0, replay_bytes=0, replay_failed=0,
                replay_incomplete=0, replay_ns=0, publication_ns=0, published=0,
                publication_failed=0)
        try:
            result = validate_report(value)
        except ValueError as error:
            self.fail(f'new four-mode version must accept the independent ledger: {error}')
        self.assertEqual(result['completed'], 300)
