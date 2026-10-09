"""Exercise the real verification entry points without recompiling the game."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import textwrap
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class VerificationPhases(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'scripts/symphony').mkdir(parents=True)
        for name in ('verify.sh', 'torture.sh', 'verify-phase.sh', 'verify-docker.sh'):
            shutil.copy2(ROOT / 'scripts' / name, self.root / 'scripts' / name)
        self.bin = self.root / 'bin'
        self.bin.mkdir()
        self.log = self.root / 'calls'
        stub = '''#!/bin/sh
printf '%s %s\\n' "${0##*/}" "$*" >> "$CALLS"
if [ -n "$FAIL_MATCH" ] && [ "$*" = "$FAIL_MATCH" ]; then exit 17; fi
'''
        for name in ('python3', 'cargo', 'docker'):
            path = self.bin / name
            path.write_text(stub)
            path.chmod(0o755)
        smoke = self.root / 'scripts/symphony/runtime-smoke.sh'
        smoke.write_text('#!/bin/sh\nprintf "runtime-smoke\\n" >> "$CALLS"\n')
        smoke.chmod(0o755)
        self.env = dict(os.environ, PATH=str(self.bin) + os.pathsep + os.environ['PATH'],
                        CALLS=str(self.log), FAIL_MATCH='', MTG_VERIFY_CACHE='')

    def run_script(self, name, *args):
        return subprocess.run([str(self.root / 'scripts' / name), *args],
                              env=self.env, capture_output=True, text=True)

    def calls(self):
        return self.log.read_text().splitlines() if self.log.exists() else []

    def test_checked_profile_preserves_debug_assertions_and_overflow_checks(self):
        manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
        profile = manifest['profile']['test']
        self.assertTrue(profile['debug-assertions'])
        self.assertTrue(profile['overflow-checks'])
        self.assertTrue(profile['debug'])
        self.assertNotIn('release', manifest['profile'])

    def test_full_contract_retains_all_checks_and_both_unfiltered_modes(self):
        result = self.run_script('torture.sh')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls(), [
            'python3 scripts/check_docs.py', 'python3 scripts/check_program.py',
            'python3 scripts/test_plan.py', 'python3 scripts/run_tests.py',
            'cargo fmt --all -- --check',
            'cargo clippy --workspace --all-targets --locked -- -D warnings',
            'cargo test --workspace --locked',
            'cargo test --workspace --release --locked',
        ])

    def test_partitions_exactly_cover_the_full_local_contract(self):
        self.assertEqual(self.run_script('torture.sh').returncode, 0)
        complete = self.calls()
        self.log.unlink()
        for phase in ('checks', 'debug', 'release'):
            self.assertEqual(self.run_script('verify-phase.sh', phase).returncode, 0)
        self.assertEqual(self.calls(), complete)

    def test_failure_stops_full_verification_and_preserves_exit_status(self):
        self.env['FAIL_MATCH'] = 'scripts/run_tests.py'
        self.assertEqual(self.run_script('torture.sh').returncode, 17)
        self.assertFalse(any(line.startswith('cargo ') for line in self.calls()))

    def test_unknown_phase_cannot_silently_pass_or_start_docker(self):
        for script in ('verify-phase.sh', 'verify-docker.sh'):
            self.assertEqual(self.run_script(script, 'typo').returncode, 2)
        self.assertEqual(self.calls(), [])

    def test_default_docker_entrypoint_retains_full_suite_and_runtime(self):
        self.assertEqual(self.run_script('verify-docker.sh').returncode, 0)
        calls = self.calls()
        self.assertIn('target=/workspace,readonly', calls[1])
        self.assertTrue(calls[1].endswith('./scripts/torture.sh'))
        self.assertEqual(calls[-1], 'runtime-smoke')

    def test_runtime_partition_does_not_invoke_other_suites(self):
        self.assertEqual(self.run_script('verify-docker.sh', 'runtime').returncode, 0)
        self.assertEqual(self.calls(), ['runtime-smoke'])

    def test_cache_mounts_only_build_and_dependency_artifacts(self):
        self.env['MTG_VERIFY_CACHE'] = str(self.root / 'cache')
        self.assertEqual(self.run_script('verify-docker.sh', 'debug').returncode, 0)
        invocation = self.calls()[-1]
        self.assertIn('target=/tmp/target', invocation)
        self.assertIn('target=/home/agent/.cargo/registry', invocation)
        self.assertIn('target=/home/agent/.cargo/git', invocation)
        self.assertNotIn('/.codex', invocation)
        self.assertTrue(invocation.endswith('./scripts/verify-phase.sh debug'))
        self.assertNotIn('runtime-smoke', self.calls())

    def test_required_ci_gate_fails_on_failed_cancelled_skipped_or_missing_jobs(self):
        # Execute the actual aggregate step, not a second implementation of its gate.
        workflow = (ROOT / '.github/workflows/ci.yml').read_text()
        gate = textwrap.dedent(workflow.rsplit('        run: |\n', 1)[1])
        for status in ('success', 'failure', 'cancelled', 'skipped', ''):
            with self.subTest(status=status):
                result = subprocess.run(['bash', '-c', gate], capture_output=True,
                                        env=dict(self.env, VERIFICATION_RESULT=status))
                self.assertEqual(result.returncode == 0, status == 'success')


if __name__ == '__main__':
    unittest.main()
