"""Schema-2 four-mode execution/measurement receipt validation (no rules oracle)."""
if __package__:
    from .scalar_artifact import require, integer
else:
    from scalar_artifact import require, integer

MODES = ('off', 'counters', 'sampled_trace', 'full_replay')
WORKLOAD = 'scalar-four-modes-v1'


def validate_pins(r):
    require(r['schema_version'] == 2, 'four-mode schema version')
    c = r['pins']['config']; e = r['pins']['execution']
    require(c['schema_version'] == 2 and c['workload'] == WORKLOAD, 'config version mismatch')
    require(c['instrumentation'] in MODES, 'unknown mode')
    replay = e['replay']
    require(replay['persistence'] == 'in_memory', 'unsupported replay persistence')
    require(integer(replay['max_bytes']) and 0 < replay['max_bytes'] <= 67108864
            and integer(replay['max_records']) and 0 < replay['max_records'] <= 20000,
            'invalid replay capacities')
    require(r['pins']['resolved_episode']['native']['max_records'] == replay['max_records'],
            'replay record capacity mismatch')
    require(c.get('replay', replay) == replay, 'replay configuration mismatch')
    trace = e['trace']
    if c['instrumentation'] == 'sampled_trace':
        require(isinstance(trace, dict) and integer(trace['every']) and trace['every'] > 0
                and integer(trace['capacity']) and trace['capacity'] <= 65536, 'invalid trace config')
        require(c.get('trace', trace) == trace, 'trace configuration mismatch')
    else:
        require(trace is None and c.get('trace') is None, 'incompatible trace config')
    require(e['capture'] == ('canonical-v2-durable' if c.get('capture') else 'none'), 'capture mismatch')
    require(e['sampled_timing'] == 'not_measured', 'unimplemented timing claim')


def validate_window(w, mode, pins):
    require(mode in MODES, 'missing or unknown mode')
    e = w['execution']
    fields = ('not_started', 'trace_selected', 'trace_retained', 'trace_dropped',
              'replay_verified', 'replay_bytes', 'replay_failed', 'replay_incomplete',
              'replay_ns', 'publication_ns', 'published', 'publication_failed')
    require(all(integer(e[k]) for k in fields), 'invalid execution accounting')
    require(e['not_started'] == w['attempts'] - w['started'], 'unaccounted pre-reset attempt')
    require(e['replay_failed'] == 0 and e['publication_failed'] == 0, 'failed persistence')
    if mode == 'full_replay':
        require(e['replay_verified'] == w['completed'] + w['concessions'], 'incomplete complete replay')
        require(e['replay_incomplete'] == w['unfinished'] + w['failed'] + w['truncated'], 'unaccounted incomplete replay')
        require(e['replay_verified'] <= e['replay_bytes'] <= e['replay_verified'] * pins['replay']['max_bytes'], 'replay byte capacity')
    else:
        require(all(e[k] == 0 for k in ('replay_verified', 'replay_bytes', 'replay_failed', 'replay_incomplete', 'replay_ns')), 'unexpected replay')
    if mode == 'sampled_trace':
        trace = pins['trace']; every = trace['every']; capacity = trace['capacity']
        require(e['trace_selected'] == e['trace_retained'] + e['trace_dropped'], 'unaccounted diagnostic drops')
        # Sampling restarts each episode: floor(sum/every) can exceed sum(floor).
        require(max(0, w['decisions']//every - max(0, w['started']-1)) <= e['trace_selected'] <= w['decisions']//every,
                'trace selection accounting')
        require(e['trace_retained'] <= w['started'] * capacity, 'diagnostic capacity')
        if w['started'] == 1:
            require(e['trace_retained'] == min(w['decisions']//every, capacity), 'trace retention')
    else:
        require(e['trace_selected'] == e['trace_retained'] == e['trace_dropped'] == 0, 'unexpected diagnostics')
    if pins['capture'] == 'none':
        require(e['published'] == e['publication_ns'] == 0, 'unexpected publication')
    else:
        require(e['published'] == w['started'], 'missing canonical publication')
    phases = w['phases']
    ns = sum(phases[k] or 0 for k in ('reset_ns', 'transition_ns', 'legality_and_view_ns', 'finalization_ns', 'encoding_ns'))
    require(ns + w['policy_ns'] + e['replay_ns'] + e['publication_ns'] <= w['elapsed_ns'], 'omitted recording denominator')


def validate_execution(receipt):
    """Finite real-native matrix, separately named from production throughput."""
    try:
        require(receipt['schema_version'] == 2 and receipt['workload'] == WORKLOAD, 'execution version')
        runs = receipt['runs']
        expected = {(m, c, row) for m in MODES for c in (False, True) for row in range(8)}
        require(len(runs) == len(expected) and {(r['mode'], r['capture'], r['row']) for r in runs} == expected,
                'missing/duplicate mode, capture setting or matchup row')
        baseline = {}
        for r in runs:
            w = r['window']
            require(r['observations'] > 0 and r['observations'] == w['decisions'], 'missing real observations')
            require(integer(w['elapsed_ns']) and w['elapsed_ns'] > 0, 'missing denominator')
            require(w['attempts'] == w['started'] == w['completed'] == 1, 'attempt ledger')
            require(w['failed'] == w['unfinished'] == w['truncated'] == w['concessions'] == 0, 'failed native execution')
            require(w['stop_code'] == 0 and w['completed_decisions'] == w['decisions'], 'unsuccessful execution')
            require(w['row_attempts'] == w['row_completed'] == [int(i == r['row']) for i in range(8)], 'row ledger')
            require(w['first_episode'] == r['row'], 'seed ordinal')
            require(sum(w['wins']) + w['draws'] == 1, 'terminal ledger')
            require(r['capture_loaded'] == r['capture'], 'canonical capture not loaded')
            validate_window(w, r['mode'], r['execution'])
            identity = (r['history_sha256'], r['observations_sha256'], r['final_sha256'], w['decisions'], w['wins'], w['draws'])
            require(identity == baseline.setdefault(r['row'], identity), 'semantic/capture inequality')
        return dict(runs=len(runs), observations=sum(r['observations'] for r in runs))
    except (KeyError, TypeError, IndexError, ZeroDivisionError) as error:
        raise ValueError(f'malformed native execution artifact: {error}') from error


def collect_report(binary, config, directory, timeout=3700):
    """Collect one declared run under the existing lock; retain failures, no retries.

    Caller owns campaign order, affinity and correctness gates. This function adds
    no CLI surface and makes no throughput qualification or reference claim.
    """
    import json
    import os
    from pathlib import Path
    from scripts.collect_scalar_baseline import digest, measured_process, system
    from scripts.scalar_artifact import validate_report
    require('MTG_SYMPHONY_LOCK_FD' in os.environ, 'caller must hold the shared heavy lock')
    fd = int(os.environ['MTG_SYMPHONY_LOCK_FD']); os.fstat(fd)
    require(config.get('schema_version') == 2 and config.get('workload') == WORKLOAD, 'collector version')
    binary = Path(binary).resolve(); directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=False)
    source = directory / 'config.json'; source.write_text(json.dumps(config, indent=2)+'\n')
    raw = directory / 'raw.json'; binary_hash = digest(binary); before = system()
    command = [str(binary), 'bench', '--workload', WORKLOAD, '--config', str(source), '--output', str(raw)]
    with (directory/'stdout.log').open('w') as out, (directory/'stderr.log').open('w') as err:
        receipt = measured_process(command, out, err, fd, timeout)
    after = system()
    receipt.update(heavy_lock=True, profiler=any(os.environ.get(k) for k in ('LD_PRELOAD','LD_AUDIT','GPROFNG_COLLECTOR_EXPNAME')),
                   contaminated=digest(binary)!=binary_hash or any(before[k]!=after[k] for k in ('cpu_quota','memory_limit','affinity')),
                   before=before, after=after, command=command)
    (directory/'collection.json').write_text(json.dumps(receipt, indent=2)+'\n')
    require(raw.is_file(), 'missing native report; retained process receipt')
    report = json.loads(raw.read_text()); report['collection'] = receipt
    receipt['contaminated'] |= report.get('hardware',{}).get('binary_sha256') != binary_hash
    (directory/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    validate_report(report)
    return report
