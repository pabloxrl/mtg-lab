"""Serialize foreground Symphony work using shared, crash-released POSIX locks.

Usage: python3 scripts/symphony/resource_lock.py [--timeout SECONDS] heavy -- COMMAND ...
       python3 scripts/symphony/resource_lock.py handoff -- COMMAND ...

All workers use ~/.local/share/mtg-lab-symphony/locks on their shared managed
volume. --lock-dir or MTG_SYMPHONY_LOCK_DIR explicitly overrides that directory
(for tests); it must be absolute with no symlink components. Never delete lock
files: flock ownership, not file existence, determines whether work is running.
Only the fixed names heavy and handoff are accepted. The two locks are independent.

Commands run without a shell unless a shell is explicitly requested. The wrapper
forwards INT/TERM/HUP to the command's new process group, waits, and preserves its
exit code or terminating signal. A wait timeout exits 124 without starting work;
helper errors exit 125, missing executables 127, and unexecutable commands 126.
Waiting notices repeat every 30 seconds. Command arguments are never logged.

The command inherits the held lock descriptor (MTG_SYMPHONY_LOCK_FD), so killing
the wrapper does not release the lock while its direct child still runs. Normal
fork/exec descendants also retain the lock unless they close that descriptor.
Commands MUST remain in the foreground and wait for their descendants. Python
subprocess users that need inherited protection must include that descriptor in
pass_fds. Deliberately detached descendants that close inherited descriptors are
outside this process-lock contract; do not launch detached heavy or handoff work.
Do not nest the same lock or unlink/replace its file while workers are running.
"""

import argparse
import errno
import fcntl
import math
import os
from pathlib import Path
import signal
import stat
import subprocess
import sys
import time


NAMES = ('heavy', 'handoff')
POLL_SECONDS = 0.1
NOTICE_SECONDS = 30.0
DIRECTORY_ENV = 'MTG_SYMPHONY_LOCK_DIR'
FD_ENV = 'MTG_SYMPHONY_LOCK_FD'
FORWARDED_SIGNALS = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)


class LockError(Exception):
    """A diagnosed lock error; messages never contain command arguments."""


def _notice(message):
    print(f'Symphony resource lock: {message}', file=sys.stderr, flush=True)


def _directory_fd(directory):
    """Traverse with directory descriptors, refusing symlinks at every level."""
    path = Path(directory)
    if not path.is_absolute() or '..' in path.parts:
        raise LockError('lock directory must be absolute without parent traversal')
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC
    fd = os.open('/', flags)
    try:
        for part in path.parts[1:]:
            try:
                os.mkdir(part, mode=0o700, dir_fd=fd)
            except FileExistsError:
                pass
            next_fd = os.open(part, flags, dir_fd=fd)
            os.close(fd)
            fd = next_fd
        info = os.fstat(fd)
        if info.st_uid != os.geteuid() or stat.S_IMODE(info.st_mode) & 0o022:
            raise LockError('lock directory must be owned by this user and not writable by others')
        return fd
    except BaseException:
        os.close(fd)
        raise


def _lock_fd(directory_fd, name):
    # O_NONBLOCK prevents a substituted special file from making open hang.
    flags = os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK
    fd = os.open(name + '.lock', flags, mode=0o600, dir_fd=directory_fd)
    try:
        info = os.fstat(fd)
        if (not stat.S_ISREG(info.st_mode) or info.st_uid != os.geteuid()
                or info.st_nlink != 1 or stat.S_IMODE(info.st_mode) & 0o077):
            raise LockError('lock file must be a private, owned, single-link regular file')
        return fd
    except BaseException:
        os.close(fd)
        raise


def _same_file(directory_fd, name, fd):
    held = os.fstat(fd)
    current = os.stat(name + '.lock', dir_fd=directory_fd, follow_symlinks=False)
    if (held.st_dev, held.st_ino) != (current.st_dev, current.st_ino):
        raise LockError('lock file was replaced; work was not started')


def run(name, command, directory, timeout=None):
    """Return a subprocess-style status; negative values are terminating signals."""
    if name not in NAMES:
        raise LockError('unknown resource name')
    if not command:
        raise LockError('a foreground command is required')
    if timeout is not None and (not math.isfinite(timeout) or timeout < 0):
        raise LockError('timeout must be a finite, nonnegative number')

    child = None
    received_signal = None
    directory_fd = None
    lock_fd = None

    def forward(signum, _frame):
        nonlocal received_signal
        received_signal = signum
        if child is not None:
            try:
                os.killpg(child.pid, signum)
            except ProcessLookupError:
                pass

    previous = {sig: signal.signal(sig, forward) for sig in FORWARDED_SIGNALS}
    try:
        directory_fd = _directory_fd(directory)
        lock_fd = _lock_fd(directory_fd, name)
        started = time.monotonic()
        next_notice = started
        waiting = False
        while True:
            if received_signal is not None:
                return -received_signal
            if waiting and timeout is not None and time.monotonic() - started >= timeout:
                _notice(f'timed out waiting for {name}; command was not started')
                return 124
            _same_file(directory_fd, name, lock_fd)
            try:
                fcntl.flock(lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except OSError as error:
                if error.errno not in (errno.EACCES, errno.EAGAIN):
                    raise
            now = time.monotonic()
            if now >= next_notice:
                _notice(f'waiting for {name} ({now - started:.0f}s elapsed)')
                next_notice = now + NOTICE_SECONDS
            waiting = True
            if timeout is not None and now - started >= timeout:
                _notice(f'timed out waiting for {name}; command was not started')
                return 124
            remaining = POLL_SECONDS if timeout is None else min(
                POLL_SECONDS, max(0, timeout - (now - started)))
            time.sleep(remaining)
        _same_file(directory_fd, name, lock_fd)
        if received_signal is not None:
            return -received_signal
        _notice(f'acquired {name}' + (' after waiting' if waiting else ''))
        env = dict(os.environ)
        env[FD_ENV] = str(lock_fd)
        # pass_fds preserves the same open file description, including flock,
        # across exec. Never explicitly LOCK_UN: a surviving child may own it.
        try:
            child = subprocess.Popen(command, env=env, pass_fds=(lock_fd,),
                                     start_new_session=True)
        except FileNotFoundError:
            _notice('command executable was not found')
            return 127
        except OSError as error:
            _notice(f'command could not be started ({errno.errorcode.get(error.errno, "OS_ERROR")})')
            return 126
        # Handle a signal received during Popen, before assigning child.
        if received_signal is not None:
            forward(received_signal, None)
        return child.wait()
    finally:
        # Closing only our reference is essential: explicit unlock would also
        # unlock the child's inherited reference after a wrapper interruption.
        if lock_fd is not None:
            os.close(lock_fd)
        if directory_fd is not None:
            os.close(directory_fd)
        for sig, handler in previous.items():
            signal.signal(sig, handler)


def _timeout(value):
    try:
        number = float(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError('timeout must be a number') from error
    if not math.isfinite(number) or number < 0:
        raise argparse.ArgumentTypeError('timeout must be finite and nonnegative')
    return number


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--lock-dir', default=os.environ.get(
        DIRECTORY_ENV, str(Path.home() / '.local/share/mtg-lab-symphony/locks')),
        help='explicit absolute shared directory; default: managed user home')
    parser.add_argument('--timeout', type=_timeout,
                        help='maximum lock wait in seconds; default: wait with periodic notices')
    parser.add_argument('name', choices=NAMES)
    parser.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args(argv)
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    try:
        return run(args.name, command, args.lock_dir, args.timeout)
    except LockError as error:
        _notice(str(error))
        return 125
    except OSError as error:
        # Do not stringify OSError: its filename might contain command secrets.
        _notice(f'cannot use shared lock ({errno.errorcode.get(error.errno, "OS_ERROR")})')
        return 125


if __name__ == '__main__':
    status = main()
    if status < 0:
        signum = -status
        if signum not in (signal.SIGKILL, signal.SIGSTOP):
            signal.signal(signum, signal.SIG_DFL)
        os.kill(os.getpid(), signum)
        # Defensive fallback on platforms delaying delivery; never report success.
        raise SystemExit(128 + signum)
    raise SystemExit(status)
