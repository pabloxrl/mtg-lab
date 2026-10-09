#!/usr/bin/env python3
"""Run the designated M2 semantic mutants in a disposable source copy.

Run under the Symphony heavy lock when using the managed worker container.
No source edits are made to the caller's checkout. Tests never read this matrix.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
MATRIX = ROOT / 'fixtures/mutations/m2.json'


def verdict(test, marker, returncode, output):
    """Only the exact assertion in exactly one executed test is a detection."""
    if returncode == 0:
        return 'survived'
    thread = re.escape(test)
    panic = (rf"thread '{thread}'(?: \([0-9]+\))? panicked at [^\n]+:\n"
             rf"assertion (?:`left == right`|`left != right`|failed: [^\n]*)"
             rf"[^\n]*{re.escape(marker)}(?:\n|$)")
    if (returncode == 101 and re.search(r'^running 1 test$', output, re.M)
            and re.search(panic, output)
            and re.search(rf'^test {thread} \.\.\. FAILED$', output, re.M)
            and re.search(r'^test result: FAILED\. 0 passed; 1 failed; 0 ignored; '
                          r'0 measured; [0-9]+ filtered out;', output, re.M)):
        return 'detected'
    return 'invalid'


def passed(test, returncode, output):
    return (returncode == 0 and re.search(r'^running 1 test$', output, re.M) is not None
            and re.search(rf'^test {re.escape(test)} \.\.\. ok$', output, re.M) is not None
            and re.search(r'^test result: ok\. 1 passed; 0 failed; 0 ignored; '
                          r'0 measured; [0-9]+ filtered out;', output, re.M) is not None)


def patch(source, old, new):
    if not old or old == new or source.count(old) != 1:
        raise ValueError('mutation anchor must match exactly once and change behavior')
    return source.replace(old, new, 1)


def load_matrix(path=MATRIX):
    data = json.loads(path.read_text())
    if data.get('schema_version') != 1 or len(data.get('cases', [])) != 12:
        raise ValueError('expected the twelve-case M2 matrix, schema 1')
    ids, tests, markers = set(), set(), set()
    for c in data['cases']:
        for key in ('id', 'test', 'assertion', 'oracle', 'input', 'path', 'old', 'new', 'test_path', 'package'):
            if not isinstance(c.get(key), str) or not c[key]:
                raise ValueError(f'missing {key}')
        for key, seen in [('id', ids), ('test', tests), ('assertion', markers)]:
            if c[key] in seen:
                raise ValueError(f'duplicate {key}')
            seen.add(c[key])
        for key in ('path', 'test_path'):
            p = Path(c[key])
            if p.is_absolute() or '..' in p.parts or p.parts[0] != 'crates':
                raise ValueError('source paths must stay inside crates')
        if c['path'] == c['test_path'] or 'tests' in Path(c['path']).parts or c['path'].endswith('_tests.rs'):
            raise ValueError('mutants must not modify assertions')
        if c.get('target') != ['--lib'] and not (
                isinstance(c.get('target'), list) and len(c['target']) == 2
                and c['target'][0] == '--test' and re.fullmatch(r'[a-z_]+', c['target'][1])):
            raise ValueError('expected one Cargo test target')
    return data['cases']


def command(args, cwd, env, log, timeout=300):
    lock_fds = ((int(env["MTG_SYMPHONY_LOCK_FD"]),)
                if "MTG_SYMPHONY_LOCK_FD" in env else ())
    process = None
    handlers = {}
    def interrupted(signum, _frame):
        raise SystemExit(128 + signum)
    try:
        process = subprocess.Popen(args, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                   stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                   text=True, start_new_session=True, pass_fds=lock_fds)
        for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP):
            handlers[sig] = signal.signal(sig, interrupted)
        try:
            output, _ = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            output, _ = process.communicate()
            log.write_text(output + "\nmutation command timed out\n")
            return -1, output
        log.write_text(output)
        return process.returncode, output
    except OSError as e:
        log.write_text(str(e))
        return -1, str(e)
    finally:
        if process is not None:
            # Also reap descendants on cancellation or failed builds. Cargo and
            # libtest children must not outlive this command or the heavy lock.
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                pass
            process.wait()
        for sig, handler in handlers.items():
            signal.signal(sig, handler)


def execute(case, root, env, out, phase):
    """Build separately, require discovery, then run the actual test executable."""
    prefix = out / f"{case['id']}-{phase}"
    args = ['cargo', 'test', '--locked', '-p', case['package'], *case['target'],
            '--no-run', '--message-format=json']
    code, output = command(args, root, env, prefix.with_suffix('.build.log'))
    if code != 0:
        return {'verdict': 'build_failure', 'exit': code}
    executables = []
    for line in output.splitlines():
        try:
            row = json.loads(line)
        except ValueError:
            continue
        if (row.get('reason') == 'compiler-artifact' and row.get('executable')
                and row.get('profile', {}).get('test')):
            executables.append(row['executable'])
    if len(executables) != 1:
        return {'verdict': 'missing_executable'}
    binary = executables[0]
    code, listing = command([binary, '--exact', case['test'], '--list'], root, env,
                            prefix.with_suffix('.list.log'))
    if code != 0 or listing.splitlines() != [f"{case['test']}: test", '', '1 test, 0 benchmarks']:
        return {'verdict': 'missing_test', 'exit': code}
    code, output = command([binary, '--exact', case['test'], '--test-threads=1'],
                           root, env, prefix.with_suffix('.test.log'), timeout=60)
    result = ('passed' if passed(case['test'], code, output) else 'invalid') if phase == 'baseline' else verdict(case['test'], case['assertion'], code, output)
    return {'verdict': result, 'exit': code,
            'output_sha256': hashlib.sha256(output.encode()).hexdigest()}


def run(cases, source, output):
    output.mkdir(parents=True, exist_ok=False)
    # Copy source files only; never share writable caches with other issues.
    files = subprocess.check_output(['git', 'ls-files', '-z', '--cached', '--others',
                                     '--exclude-standard'], cwd=source).split(b'\0')
    files = sorted(set(os.fsdecode(p) for p in files if p))
    receipt = {'schema_version': 1, 'head': subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=source, text=True).strip(),
        'source_sha256': {}, 'cases': [], 'passed': False}
    with tempfile.TemporaryDirectory(prefix='m2-mutants-', dir=source / '.agent-artifacts') as tmp:
        root = Path(tmp)
        for name in files:
            src = source / name
            if not src.is_file() or src.is_symlink():
                raise ValueError(f'not a regular source file: {name}')
            dest = root / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(src, dest)
            receipt['source_sha256'][name] = hashlib.sha256(src.read_bytes()).hexdigest()
        env = os.environ.copy()
        env['CARGO_TARGET_DIR'] = str(source / 'target/m2-mutations')
        env['RUST_BACKTRACE'] = '0'
        # libtest output must be deterministic enough for strict classification.
        env.pop('RUST_TEST_NOCAPTURE', None)
        for c in cases:
            record = {'id': c['id'], 'test': c['test'], 'assertion': c['assertion']}
            receipt['cases'].append(record)
            path = root / c['path']
            original = path.read_text()
            if c['assertion'] not in (root / c['test_path']).read_text():
                raise ValueError(f"missing named assertion: {c['id']}")
            changed = patch(original, c['old'], c['new'])
            record['baseline'] = execute(c, root, env, output, 'baseline')
            if record['baseline']['verdict'] == 'passed':
                try:
                    path.write_text(changed)
                    record['mutant'] = execute(c, root, env, output, 'mutant')
                finally:
                    path.write_text(original)
            print(f"{c['id']}: {record}", flush=True)
            (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
        receipt['passed'] = all(r['baseline']['verdict'] == 'passed'
                                and r.get('mutant', {}).get('verdict') == 'detected'
                                for r in receipt['cases'])
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    return receipt['passed']


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True, help='new directory for logs and receipt')
    args = p.parse_args()
    try:
        (ROOT / '.agent-artifacts').mkdir(exist_ok=True)
        return 0 if run(load_matrix(), ROOT, args.output.resolve()) else 1
    except (ValueError, OSError, subprocess.SubprocessError) as e:
        p.exit(1, f'mutation run failed: {e}\n')


if __name__ == '__main__':
    raise SystemExit(main())
