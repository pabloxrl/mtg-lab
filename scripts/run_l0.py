"""Run a prebuilt native L0 command and persist a complete cold-process receipt.

No build, installation, generation, or network is performed here. Output paths
must be new: historical receipts are never overwritten. The second complete pass runs in the same native process and is timed separately.
"""
import argparse
import hashlib
import json
import math
import os
import signal
import threading
import platform
import resource
import subprocess
import time
from pathlib import Path


def comparable(native, cold, expected_hash):
    """Missing timing/accounting can never certify a full comparable run."""
    try:
        if not math.isfinite(cold) or cold <= 0 or not native['complete']:
            return False
        if native['corpus_sha256'] != expected_hash or len(native['runner_sha256']) != 64:
            return False
        for run in (native, native['warm']):
            if (type(run['expected']) is not int or run['expected'] <= 0
                    or run['successful'] != run['expected'] or run['failed'] != 0 or run['errors']
                    or len(run['games']) != run['expected']
                    or not all(game['passed'] for game in run['games'])):
                return False
        elapsed = native['warm']['elapsed_seconds']
        return math.isfinite(elapsed) and elapsed > 0
    except (KeyError, TypeError, ValueError):
        return False


def execute_runner(command, start, timeout_seconds):
    """Drain both pipes continuously; timestamp the cold marker on arrival."""
    captured = {'stdout': bytearray(), 'stderr': bytearray()}
    counts = {'stdout': 0, 'stderr': 0}
    timing = {}
    limits = {'stdout': 32 * 1024 * 1024, 'stderr': 1024 * 1024}
    def drain(pipe, name):
        while chunk := os.read(pipe.fileno(), 65536):
            counts[name] += len(chunk)
            room = limits[name] - len(captured[name])
            captured[name].extend(chunk[:max(0, room)])
            if name == 'stdout' and 'cold' not in timing and b'\n' in captured[name]:
                timing['cold'] = time.perf_counter() - start
    with subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                          stdin=subprocess.DEVNULL, start_new_session=True) as process:
        readers = [threading.Thread(target=drain, args=(getattr(process, name), name), daemon=True)
                   for name in ('stdout', 'stderr')]
        for reader in readers:
            reader.start()
        timed_out = False
        try:
            process.wait(timeout=max(0, timeout_seconds - (time.perf_counter() - start)))
        except subprocess.TimeoutExpired:
            timed_out = True
            # The native CLI defers SIGTERM; kill and reap this owned process so
            # an unresponsive engine cannot outlive the verification deadline.
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
        for reader in readers:
            reader.join(timeout=max(0, timeout_seconds - (time.perf_counter() - start)))
        if any(reader.is_alive() for reader in readers):
            timed_out = True
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            for reader in readers:
                reader.join(timeout=1)
    return dict(stdout=bytes(captured['stdout']), stderr=bytes(captured['stderr']).decode(errors='replace'),
                stderr_bytes=counts['stderr'], stderr_truncated=counts['stderr'] > limits['stderr'],
                stdout_overflow=counts['stdout'] > limits['stdout'], cold_seconds=timing.get('cold'),
                timed_out=timed_out, returncode=process.returncode)


def run(runner, manifest, manifest_sha256, output, timeout_seconds=600.0):
    receipt = dict(launcher_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), schema_version=1, kind='native-l0-full-path', full_comparable_run=False,
                   successful=0, failed=0, compilation_seconds=None, warm_throughput=None,
                   cold_boundary='before runner launch through first full-corpus completion receipt (before warm pass)',
                   hardware=dict(platform=platform.platform(), machine=platform.machine(),
                                 processor=platform.processor()), errors=[])
    # Reserve the receipt before execution; never overwrite prior run history.
    with Path(output).open('x') as stream:
        start = time.perf_counter()
        code = 1
        try:
            if not math.isfinite(timeout_seconds) or timeout_seconds <= 0:
                raise ValueError('timeout must be a finite positive number of seconds')
            receipt['timeout_seconds'] = timeout_seconds
            result = execute_runner([str(runner), 'l0', str(manifest), manifest_sha256, '--warm'],
                                    start, timeout_seconds)
            receipt.update({key: result[key] for key in ('stderr', 'stderr_bytes', 'stderr_truncated')})
            code = result['returncode']
            if result['timed_out']:
                raise ValueError('runner deadline exceeded; process killed and reaped')
            if result['stdout_overflow']:
                raise ValueError('runner receipt exceeds 32 MiB; full run cannot be certified')
            first, rest = result['stdout'].split(b'\n', 1)
            marker = json.loads(first)
            if marker.get('phase') != 'cold_complete' or result['cold_seconds'] is None:
                raise ValueError('missing cold timing boundary')
            cold_seconds = result['cold_seconds']
            native = json.loads(rest)
            if any(marker['receipt'][key] != native[key] for key in
                   ('corpus_sha256', 'runner_sha256', 'expected', 'successful', 'failed', 'errors', 'games')):
                raise ValueError('cold/final corpus or accounting mismatch')
            receipt['cold_elapsed_seconds'] = cold_seconds
            receipt['warm_throughput'] = native['warm']
            receipt['native'] = native
            receipt['successful'] = native['successful']
            receipt['failed'] = native['failed']
            receipt['runner_sha256'] = native['runner_sha256']
            receipt['corpus_sha256'] = native['corpus_sha256']
            receipt['errors'] = native['errors']
            receipt['full_comparable_run'] = code == 0 and marker['receipt']['complete'] and comparable(native, cold_seconds, manifest_sha256)
        except (OSError, ValueError, KeyError, TypeError) as error:
            receipt['errors'].append(str(error))
        receipt.setdefault('cold_elapsed_seconds', time.perf_counter() - start)
        receipt['total_invocation_seconds'] = time.perf_counter() - start
        receipt['peak_rss_kib'] = resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss
        receipt['resource_scope'] = 'child process high water; Linux KiB; launcher invokes one runner'
        receipt['runner_exit_code'] = code
        receipt['exit_code'] = 0 if receipt['full_comparable_run'] else 1
        json.dump(receipt, stream, sort_keys=True, indent=2)
        stream.write('\n')
    return 0 if receipt['full_comparable_run'] else 1


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runner', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, required=True)
    parser.add_argument('--manifest-sha256', required=True)
    parser.add_argument('--receipt', type=Path, required=True)
    parser.add_argument('--timeout-seconds', type=float, default=600.0)
    args = parser.parse_args()
    raise SystemExit(run(args.runner.resolve(), args.manifest, args.manifest_sha256, args.receipt, args.timeout_seconds))
