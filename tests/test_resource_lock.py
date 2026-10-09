"""Real subprocess contention, interruption and fail-closed lock contracts."""

import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'scripts/symphony/resource_lock.py'
PYTHON = sys.executable
HOLDER = '''
import os, pathlib, sys, time
fd = int(os.environ['MTG_SYMPHONY_LOCK_FD'])
assert os.get_inheritable(fd)
pathlib.Path(sys.argv[1]).write_text(str(os.getpid()))
while not pathlib.Path(sys.argv[2]).exists():
    time.sleep(0.02)
'''


class ResourceLockTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        # macOS /var and /tmp have system symlinks: give the helper a canonical
        # explicit directory, while the rejection tests exercise unsafe aliases.
        self.root = Path(self.temporary.name).resolve()
        self.locks = self.root / 'locks'
        self.processes = []
        self.groups = set()
        self.streams = []

    def tearDown(self):
        for group in self.groups:
            try:
                os.killpg(group, signal.SIGKILL)
            except ProcessLookupError:
                pass
        for process in self.processes:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=5)
        for stream in self.streams:
            stream.close()
        self.temporary.cleanup()

    def argv(self, name='heavy', command=None, timeout=None, directory=None):
        args = [PYTHON, str(SCRIPT), '--lock-dir', str(directory or self.locks)]
        if timeout is not None:
            args += ['--timeout', str(timeout)]
        return args + [name, '--'] + (command or [PYTHON, '-c', 'pass'])

    def start(self, name='heavy', command=None, timeout=None):
        log = self.root / f'output-{len(self.processes)}'
        stream = log.open('w')
        self.streams.append(stream)
        process = subprocess.Popen(self.argv(name, command, timeout),
                                   stdout=stream, stderr=stream)
        self.processes.append(process)
        return process, log

    def wait_for(self, predicate):
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            if predicate():
                return
            time.sleep(0.02)
        self.fail('subprocess did not reach the required checkpoint within five seconds')

    def holder(self):
        ready, release = self.root / 'ready', self.root / 'release'
        process, log = self.start(command=[PYTHON, '-c', HOLDER, str(ready), str(release)])
        self.wait_for(ready.exists)
        self.groups.add(int(ready.read_text()))
        return process, log, release, int(ready.read_text())

    def test_same_name_serializes_and_other_resource_remains_independent(self):
        first, _, release, _ = self.holder()
        second_marker = self.root / 'second'
        second, second_log = self.start(command=[PYTHON, '-c',
            'import pathlib,sys; pathlib.Path(sys.argv[1]).touch(); sys.exit(7)', str(second_marker)])
        self.wait_for(lambda: 'waiting for heavy' in second_log.read_text())
        self.assertFalse(second_marker.exists())
        handoff = subprocess.run(self.argv('handoff'), capture_output=True, text=True, timeout=5)
        self.assertEqual(handoff.returncode, 0, handoff.stderr)
        self.assertIsNone(first.poll())
        release.touch()
        self.assertEqual(first.wait(timeout=5), 0)
        self.assertEqual(second.wait(timeout=5), 7)
        self.assertTrue(second_marker.exists())
        self.assertIn('acquired heavy after waiting', second_log.read_text())
        self.assertTrue((self.locks / 'heavy.lock').exists(), 'lock file must never be unlinked')

    def test_wait_timeout_does_not_start_command_or_print_arguments(self):
        self.holder()
        sentinel = self.root / 'NEVER_PRINT_SECRET_COMMAND_ARGUMENT'
        result = subprocess.run(self.argv(command=[PYTHON, '-c',
            'import pathlib,sys; pathlib.Path(sys.argv[1]).touch()', str(sentinel)], timeout=0.15),
            capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 124, result.stderr)
        self.assertIn('waiting for heavy', result.stderr)
        self.assertIn('command was not started', result.stderr)
        self.assertFalse(sentinel.exists())
        self.assertNotIn(sentinel.name, result.stderr + result.stdout)

    def test_exit_and_signal_status_are_preserved_and_lock_released(self):
        for command, status in [([PYTHON, '-c', 'raise SystemExit(23)'], 23),
                ([PYTHON, '-c', 'import os,signal; os.kill(os.getpid(),signal.SIGTERM)'], -signal.SIGTERM),
                ([PYTHON, '-c', 'import os,signal; os.kill(os.getpid(),signal.SIGKILL)'], -signal.SIGKILL)]:
            with self.subTest(status=status):
                result = subprocess.run(self.argv(command=command), capture_output=True, timeout=5)
                self.assertEqual(result.returncode, status)
                again = subprocess.run(self.argv(timeout=0), capture_output=True, timeout=5)
                self.assertEqual(again.returncode, 0)

    def test_signal_to_wrapper_reaches_running_child(self):
        holder, _, _, child = self.holder()
        holder.send_signal(signal.SIGTERM)
        self.assertEqual(holder.wait(timeout=5), -signal.SIGTERM)
        result = subprocess.run(self.argv(timeout=1), capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        # Successful reacquisition also proves the direct child closed its
        # inherited descriptor, rather than continuing after wrapper termination.
        self.assertNotEqual(child, holder.pid)

    def test_interrupted_waiter_never_starts_its_command(self):
        self.holder()
        sentinel = self.root / 'waiter-started'
        waiter, log = self.start(command=[PYTHON, '-c',
            'import pathlib,sys; pathlib.Path(sys.argv[1]).touch()', str(sentinel)])
        self.wait_for(lambda: 'waiting for heavy' in log.read_text())
        waiter.send_signal(signal.SIGTERM)
        self.assertEqual(waiter.wait(timeout=5), -signal.SIGTERM)
        self.assertFalse(sentinel.exists())

    def test_wrapper_crash_keeps_lock_until_inheriting_child_exits(self):
        holder, _, release, _ = self.holder()
        holder.kill()
        self.assertEqual(holder.wait(timeout=5), -signal.SIGKILL)
        blocked = subprocess.run(self.argv(timeout=0), capture_output=True, timeout=5)
        self.assertEqual(blocked.returncode, 124, blocked.stderr)
        release.touch()
        resumed = subprocess.run(self.argv(timeout=3), capture_output=True, timeout=5)
        self.assertEqual(resumed.returncode, 0, resumed.stderr)

    def test_forked_descendant_retains_lock_after_command_and_wrapper_exit(self):
        ready, release = self.root / 'descendant', self.root / 'release-descendant'
        code = '''
import os, pathlib, sys, time
child = os.fork()
if child:
    pathlib.Path(sys.argv[1]).write_text(str(os.getpgrp()))
    raise SystemExit(0)
while not pathlib.Path(sys.argv[2]).exists():
    time.sleep(0.02)
os._exit(0)
'''
        holder, _ = self.start(command=[PYTHON, '-c', code, str(ready), str(release)])
        self.wait_for(ready.exists)
        self.groups.add(int(ready.read_text()))
        self.assertEqual(holder.wait(timeout=5), 0)
        blocked = subprocess.run(self.argv(timeout=0), capture_output=True, timeout=5)
        self.assertEqual(blocked.returncode, 124, blocked.stderr)
        release.touch()
        resumed = subprocess.run(self.argv(timeout=3), capture_output=True, timeout=5)
        self.assertEqual(resumed.returncode, 0, resumed.stderr)

    def test_symlinked_directory_or_file_and_hardlink_fail_without_touching_target(self):
        self.locks.mkdir(mode=0o700)
        target = self.root / 'credential-sentinel'
        target.write_text('must remain unchanged')
        file = self.locks / 'heavy.lock'
        file.symlink_to(target)
        result = subprocess.run(self.argv(), capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 125)
        file.unlink()
        os.link(target, file)
        result = subprocess.run(self.argv(), capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 125)
        file.unlink()
        alias = self.root / 'alias'
        alias.symlink_to(self.locks, target_is_directory=True)
        result = subprocess.run(self.argv(directory=alias / 'nested'), capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 125)
        self.assertFalse((self.locks / 'nested').exists())
        self.assertEqual(target.read_text(), 'must remain unchanged')

    def test_unsafe_permissions_special_file_names_and_timeouts_fail_closed(self):
        self.locks.mkdir(mode=0o700)
        self.locks.chmod(0o777)
        result = subprocess.run(self.argv(), capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 125)
        self.locks.chmod(0o700)
        fifo = self.locks / 'heavy.lock'
        os.mkfifo(fifo, 0o600)
        result = subprocess.run(self.argv(), capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 125)
        fifo.unlink()
        for name in ['../credential', 'unknown']:
            result = subprocess.run(self.argv(name), capture_output=True, timeout=5)
            self.assertEqual(result.returncode, 2)
        for timeout in ['-1', 'nan', 'inf']:
            result = subprocess.run(self.argv(timeout=timeout), capture_output=True, timeout=5)
            self.assertEqual(result.returncode, 2)

    def test_explicit_environment_directory_and_launch_failure(self):
        env = dict(os.environ, MTG_SYMPHONY_LOCK_DIR=str(self.locks))
        result = subprocess.run([PYTHON, str(SCRIPT), 'heavy', '--', PYTHON, '-c', 'pass'],
                                env=env, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        missing = self.root / 'SECRET_MISSING_EXECUTABLE'
        result = subprocess.run(self.argv(command=[str(missing)]), capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 127)
        self.assertNotIn(missing.name, result.stderr)
        result = subprocess.run(self.argv(timeout=0), capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 0)


if __name__ == '__main__':
    unittest.main()
