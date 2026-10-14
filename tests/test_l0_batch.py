"""L0-004: actual qualified games, exact comparisons, fail-closed accounting."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import package_l0


class BatchReplay(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        build = subprocess.run(['cargo', 'build', '--locked', '--release', '-p', 'mtg-cli',
                                '--message-format=json'], cwd=ROOT, check=True,
                               capture_output=True, text=True)
        cls.runner = next(Path(row['executable']) for line in build.stdout.splitlines()
                          if (row := json.loads(line)).get('reason') == 'compiler-artifact'
                          and row.get('executable') and row['target']['name'] == 'mtg')
        cls.temp = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.temp.cleanup)
        cls.manifest = package_l0.package(Path(cls.temp.name) / 'corpus')

    def invoke(self, manifest=None):
        path = manifest or self.manifest
        result = subprocess.run([str(self.runner), 'l0', str(path), hashlib.sha256(path.read_bytes()).hexdigest()],
                                cwd=ROOT, capture_output=True, text=True, stdin=subprocess.DEVNULL)
        self.assertTrue(result.stdout.strip(), result.stderr)
        return result.returncode, json.loads(result.stdout)

    def test_all_ten_real_games_twice(self):
        import run_l0
        output = Path(self.temp.name) / 'receipt.json'
        code = run_l0.run(self.runner, self.manifest,
                          hashlib.sha256(self.manifest.read_bytes()).hexdigest(), output)
        receipt = json.loads(output.read_text())
        self.assertEqual(code, 0, receipt)
        self.assertTrue(receipt['full_comparable_run'])
        self.assertGreater(receipt['cold_elapsed_seconds'], 0)
        self.assertEqual(receipt['successful'], 10)
        self.assertEqual(receipt['failed'], 0)
        self.assertEqual(receipt['native']['actions'], 4886)
        self.assertFalse(receipt['native']['full_comparable_run'])
        self.assertEqual(receipt['warm_throughput']['successful'], 10)
        self.assertEqual(receipt['warm_throughput']['failed'], 0)
        self.assertGreater(receipt['warm_throughput']['elapsed_seconds'], 0)
        self.assertEqual(len(receipt['runner_sha256']), 64)

    def test_missing_artifact_and_omitted_entry_fail(self):
        original = json.loads(self.manifest.read_text())
        for kind in ('missing', 'omitted', 'pin'):
            changed = copy.deepcopy(original)
            if kind == 'missing':
                changed['games'][0]['input'] = 'missing.json'
            elif kind == 'omitted':
                changed['games'].pop()
            else:
                changed['semantic_schema'] = 'unknown'
            path = self.manifest.parent / 'bad-manifest.json'
            path.write_text(json.dumps(changed))
            code, receipt = self.invoke(path)
            self.assertNotEqual(code, 0)
            self.assertFalse(receipt['full_comparable_run'])
            self.assertTrue(receipt['errors'])

    def test_actual_checkpoint_mutation_and_unconsumed_choice(self):
        original = json.loads(self.manifest.read_text())
        for kind in ('life', 'opening', 'choice', 'chance', 'pin', 'incomplete', 'checksum'):
            changed = copy.deepcopy(original)
            game = changed['games'][0]
            name = game['expected'] if kind in ('life', 'opening') else game['input']
            path = self.manifest.parent / name
            saved = path.read_bytes()
            try:
                value = json.loads(saved)
                if kind == 'life':
                    value['points'][0]['life'][0] += 1
                elif kind == 'opening':
                    value['opening']['points'][0]['library'].reverse()
                elif kind == 'chance':
                    value['cases'][0]['chance'].append(value['cases'][0]['chance'][-1])
                elif kind == 'checksum':
                    value['cases'][0]['play'].pop()
                elif kind == 'choice':
                    value['cases'][0]['play'].append(value['cases'][0]['play'][-1])
                elif kind == 'incomplete':
                    value['cases'][0]['play'].pop()
                else:
                    value['pins']['data/cards/foundations_micro_v1.json'] = '0' * 64
                path.write_bytes(package_l0.encoded(value))
                if kind != 'checksum':
                    changed['artifacts'][name] = hashlib.sha256(path.read_bytes()).hexdigest()
                m = self.manifest.parent / 'mutation.json'
                m.write_text(json.dumps(changed))
                code, receipt = self.invoke(m)
                self.assertNotEqual(code, 0)
                if kind == 'checksum':
                    self.assertEqual(receipt['successful'], 0)
                    self.assertIn('checksum', receipt['errors'][0]['first_field'])
                    continue
                self.assertEqual(receipt['failed'], 1, receipt)
                self.assertEqual(receipt['successful'], 9)
                self.assertEqual(receipt['errors'][0]['seed'], 32100)
                self.assertTrue(receipt['errors'][0]['first_field'])
                if kind in ('choice', 'incomplete'):
                    wanted = len(value['cases'][0]['play']) - (1 if kind == 'choice' else 0)
                    self.assertEqual(receipt['errors'][0]['cursor'], wanted)
                if kind == 'opening':
                    self.assertEqual(receipt['errors'][0]['cursor_kind'], 'opening_checkpoint')
                    self.assertEqual(receipt['errors'][0]['cursor'], 0)
                    self.assertIsNone(receipt['errors'][0]['source_cursor'])
            finally:
                path.write_bytes(saved)


class ReceiptBoundary(unittest.TestCase):
    def test_unavailable_release_runner_retains_failure_receipt(self):
        import run_l0
        with tempfile.TemporaryDirectory() as folder:
            output = Path(folder) / 'failure.json'
            code = run_l0.run(Path(folder) / 'missing-runner', Path(folder) / 'manifest.json', '0'*64, output)
            receipt = json.loads(output.read_text())
            self.assertEqual(code, 1)
            self.assertFalse(receipt['full_comparable_run'])
            self.assertGreater(receipt['cold_elapsed_seconds'], 0)
            self.assertTrue(receipt['errors'])
            with self.assertRaises(FileExistsError):
                run_l0.run(Path(folder) / 'missing-runner', Path(folder) / 'manifest.json', '0'*64, output)

    def test_missing_timing_or_incomplete_counts_never_certify_run(self):
        import run_l0
        # Receipt-only fault injection, not gameplay evidence.
        native = dict(complete=True, expected=1, successful=1, failed=0, errors=[],
                      games=[dict(passed=True)], corpus_sha256='a'*64, runner_sha256='b'*64)
        native['warm'] = dict(native, elapsed_seconds=1.0)
        self.assertTrue(run_l0.comparable(native, 1.0, 'a'*64))
        for edit in (lambda n:n['warm'].pop('elapsed_seconds'),
                     lambda n:n['warm'].update(successful=0),
                     lambda n:n.update(complete=False),
                     lambda n:n.update(games=[])):
            altered=copy.deepcopy(native);edit(altered)
            self.assertFalse(run_l0.comparable(altered, 1.0, 'a'*64))
        self.assertFalse(run_l0.comparable(native, float('nan'), 'a'*64))

    def launcher_probe(self, body, timeout):
        """Transport fault probes cannot serve as game replay acceptance."""
        import run_l0
        with tempfile.TemporaryDirectory() as folder:
            runner = Path(folder) / 'probe'
            runner.write_text('#!' + sys.executable + '\n' + body)
            runner.chmod(0o700)
            output = Path(folder) / 'receipt.json'
            code = run_l0.run(runner, Path(folder) / 'unused', '0'*64, output, timeout)
            return code, json.loads(output.read_text())

    def test_stalled_child_is_reaped_and_failure_receipt_is_written(self):
        code, receipt = self.launcher_probe('import time\ntime.sleep(60)\n', 0.1)
        self.assertEqual(code, 1)
        self.assertFalse(receipt['full_comparable_run'])
        self.assertIn('deadline', receipt['errors'][0])
        self.assertLess(receipt['total_invocation_seconds'], 5)
        self.assertLess(receipt['runner_exit_code'], 0)

    def test_stderr_flood_is_drained_without_waiting_for_cold_marker(self):
        code, receipt = self.launcher_probe(
            'import os\nos.write(2, b"x" * (2 * 1024 * 1024))\nprint("invalid receipt")\n', 5)
        self.assertEqual(code, 1)
        self.assertFalse(receipt['full_comparable_run'])
        self.assertEqual(receipt['stderr_bytes'], 2 * 1024 * 1024)
        self.assertTrue(receipt['stderr_truncated'])
        self.assertNotIn('deadline', receipt['errors'][0])
        self.assertEqual(receipt['runner_exit_code'], 0)
