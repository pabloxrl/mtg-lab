"""Run a prebuilt native L0 command and persist a complete cold-process receipt.

No build, installation, generation, or network is performed here. Output paths
must be new: historical receipts are never overwritten. The second complete pass runs in the same native process and is timed separately.
"""
import argparse
import json
import math
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


def run(runner, manifest, manifest_sha256, output):
    receipt = dict(schema_version=1, kind='native-l0-full-path', full_comparable_run=False,
                   successful=0, failed=0, compilation_seconds=None, warm_throughput=None,
                   cold_boundary='before runner launch through first full-corpus completion receipt (before warm pass)',
                   hardware=dict(platform=platform.platform(), machine=platform.machine(),
                                 processor=platform.processor()), errors=[])
    # Reserve the receipt before execution; never overwrite prior run history.
    with Path(output).open('x') as stream:
        start = time.perf_counter()
        code = 1
        try:
            with subprocess.Popen([str(runner), 'l0', str(manifest), manifest_sha256, '--warm'],
                                  stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                  stdin=subprocess.DEVNULL) as process:
                first = process.stdout.readline()
                cold_seconds = time.perf_counter() - start
                rest, stderr = process.communicate()
                marker = json.loads(first)
                if marker.get('phase') != 'cold_complete':
                    raise ValueError('missing cold timing boundary')
                native = json.loads(rest)
                if marker['receipt']['corpus_sha256'] != native['corpus_sha256']:
                    raise ValueError('cold/final corpus mismatch')
                code = process.returncode
            receipt['cold_elapsed_seconds'] = cold_seconds
            receipt['warm_throughput'] = native['warm']
            receipt['native'] = native
            receipt['successful'] = native['successful']
            receipt['failed'] = native['failed']
            receipt['runner_sha256'] = native['runner_sha256']
            receipt['corpus_sha256'] = native['corpus_sha256']
            receipt['errors'] = native['errors']
            receipt['stderr'] = stderr
            receipt['full_comparable_run'] = code == 0 and comparable(native, cold_seconds, manifest_sha256)
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
    args = parser.parse_args()
    raise SystemExit(run(args.runner.resolve(), args.manifest, args.manifest_sha256, args.receipt))
