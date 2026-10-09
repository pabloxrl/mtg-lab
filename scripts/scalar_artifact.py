"""Independent arithmetic and completeness validation for scalar baseline artifacts.

Input includes the unmodified benchmark JSON plus a collector's `collection`
receipt. This checks evidence consistency, not authenticity or rules correctness.
"""
import argparse
import json
import math
from pathlib import Path


def require(ok, message):
    if not ok:
        raise ValueError(message)


def integer(value):
    return type(value) is int and value >= 0


def close(a, b):
    return type(a) in (int, float) and math.isfinite(a) and math.isclose(a, b, rel_tol=1e-9, abs_tol=1e-9)


def validate_report(report):
    """Validate every raw interval before computing aggregate rates."""
    try:
        return _validate(report)
    except (KeyError, TypeError, IndexError, ZeroDivisionError) as error:
        raise ValueError(f"malformed scalar artifact: {error}") from error


def _validate(r):
    require(r['status'] == 'measured', 'unsuccessful benchmark')
    require(r['workload'] == 'scalar-full-pool-v1', 'not the full-pool workload')
    collection = r['collection']
    require(collection['contaminated'] is False and collection['profiler'] is False
            and collection['returncode'] == 0 and collection['heavy_lock'] is True,
            'contaminated, profiled, failed or unlocked throughput run')
    episode = r['pins']['resolved_episode']
    require(episode['max_decisions'] == 20_000 and episode['native']['max_work_calls'] == 100_000
            and episode['native']['max_records'] == 20_000, 'changed game horizon')
    c = r['pins']['config']
    require(integer(c['warmup_seconds']) and c['warmup_seconds'] >= 10, 'short warmup config')
    require(integer(c['window_seconds']) and c['window_seconds'] >= 30, 'short window config')
    require(integer(c['windows']) and c['windows'] >= 5, 'too few declared windows')
    require(len(r['windows']) == c['windows'], 'omitted windows')
    require(c['instrumentation'] in ('off', 'counters'), 'unsupported instrumentation')
    next_episode = 0
    for i, w in enumerate([r['warmup'], *r['windows']]):
        duration = c['window_seconds'] if i else c['warmup_seconds']
        require(integer(w['elapsed_ns']) and w['elapsed_ns'] >= duration*1_000_000_000, 'short interval')
        require(w['first_episode'] == next_episode, 'omitted or repeated episode interval')
        for k in ('attempts','started','completed','failed','unfinished','truncated','concessions',
                  'draws','decisions','completed_decisions','policy_ns'):
            require(integer(w[k]), f'invalid {k}')
        expected_rows=[w['attempts']//8]*8
        for offset in range(w['attempts']%8):
            expected_rows[(next_episode+offset)%8]+=1
        require(w['row_attempts']==expected_rows, 'omitted or fabricated matchup attempts')
        require(len(w['row_completed'])==8 and all(integer(n) and n<=a for n,a in zip(w['row_completed'],expected_rows))
                and sum(w['row_completed'])==w['completed'], 'matchup completion accounting')
        next_episode += w['attempts']
        require(0 <= w['attempts']-w['started'] <= 1, 'attempt accounting')
        require(w['stop_code'] == 0 and w['failed'] == 0 and not w.get('error'), 'failed interval')
        require(w['started'] == sum(w[k] for k in ('completed','failed','unfinished','truncated','concessions')), 'outcome accounting')
        require(len(w['wins']) == 2 and all(integer(n) for n in w['wins']), 'invalid wins')
        require(w['completed'] == sum(w['wins']) + w['draws'], 'terminal accounting')
        require(w['completed_decisions'] <= w['decisions'], 'decision accounting')
        require(w['completed'] > 0, 'no rules completions')
        phases = w['phases']
        require(all(integer(phases[k]) for k in ('reset_ns','transition_ns','legality_and_view_ns','finalization_ns','encoded_bytes')), 'invalid boundary times')
        require(integer(phases['encoding_ns']) if c['encoding'] else phases['encoding_ns'] is None, 'encoding availability')
        elapsed_phases = sum(phases[k] for k in ('reset_ns','transition_ns','legality_and_view_ns','finalization_ns'))
        require(elapsed_phases + (phases['encoding_ns'] or 0) + w['policy_ns'] <= w['elapsed_ns'], 'overlapping boundary times')
        counters = w['counters']
        if c['instrumentation'] == 'off':
            require(counters is None, 'off counters must be unavailable')
        else:
            require(isinstance(counters,dict), 'missing counters')
            require(counters['overflowed'] is False and counters['capacity_overflows'] == 0, 'counter/capacity overflow')
            require(counters['decisions'] == w['decisions'], 'counter decision mismatch')
        if i:
            require(close(w['completed_games_per_second'], w['completed']*1e9/w['elapsed_ns']), 'completion rate arithmetic')
            require(close(w['decisions_per_second'], w['decisions']*1e9/w['elapsed_ns']), 'decision rate arithmetic')
            require(close(w['mean_decisions_per_completed_game'],w['completed_decisions']/w['completed']), 'completed decision mean')
            require(w['has_rules_completions'] is True, 'completion flag')
            require(w['logical_actions_per_second'] is None if counters is None else close(w['logical_actions_per_second'],counters['logical_actions']*1e9/w['elapsed_ns']), 'logical rate arithmetic')
    rates = [w['completed']*1e9/w['elapsed_ns'] for w in r['windows']]
    d = r['completed_games_per_second']; ordered=sorted(rates)
    require(len(d['samples']) == len(rates) and all(close(a,b) for a,b in zip(d['samples'],rates)), 'selected or reordered distribution')
    expected = dict(mean=sum(rates)/len(rates),p50=ordered[math.ceil(len(rates)*.5)-1],
                    p95=ordered[math.ceil(len(rates)*.95)-1],min=min(rates),max=max(rates))
    require(all(close(d[k],v) for k,v in expected.items()), 'distribution arithmetic')
    total = {k:sum(w[k] for w in r['windows']) for k in ('completed','started','failed','truncated','unfinished','decisions','elapsed_ns')}
    total['not_started'] = sum(w['attempts']-w['started'] for w in r['windows'])
    total['completed_games_per_second'] = total['completed']*1e9/total['elapsed_ns']
    total['decisions_per_second'] = total['decisions']*1e9/total['elapsed_ns']
    return total


def validate_suite(reports):
    """Pool every retained valid run; never select a best window or retry."""
    groups={}
    reference=None
    for r in reports:
        summary=validate_report(r)
        config=dict(r['pins']['config']);policies=config.pop('policies')
        require(len(policies)==2 and policies[0]==policies[1]
                and policies[0] in ('heuristic-activation-mana-v1','legal-random-activation-mana-v1'), 'unsupported policy pair')
        mode=config.pop('instrumentation');config.pop('windows')
        identity=(config,r['pins']['resolved_episode'],r['hardware']['binary_sha256'],
                  r['hardware']['source_sha256'],r['hardware']['cpu_affinity'])
        # resolved_episode includes the policy/mode in real reports; compare
        # only fixed game/horizon fields, separately from declared policy below.
        episode=dict(identity[1]);episode.pop('policies',None)
        native=dict(episode['native']);native.pop('instrumentation',None);episode['native']=native
        identity=(identity[0],episode,*identity[2:])
        if reference is None: reference=identity
        require(identity==reference,'incomparable workload, seeds, binary or affinity')
        groups.setdefault((policies[0],mode),[]).append(summary)
    expected={(p,m) for p in ('heuristic-activation-mana-v1','legal-random-activation-mana-v1') for m in ('off','counters')}
    require(set(groups)==expected,'missing policy/mode pair')
    result={}
    for policy in sorted({p for p,_ in expected}):
        rates={}
        for mode in ('off','counters'):
            rows=groups[(policy,mode)];ns=sum(r['elapsed_ns'] for r in rows)
            rates[mode]={k:sum(r[k] for r in rows)*1e9/ns for k in ('decisions','completed')}
        overhead=100*(1-rates['counters']['decisions']/rates['off']['decisions'])
        result[policy]=dict(rates=rates,counters_overhead_percent=overhead,
                            provisional_5_percent_overhead_met=overhead<=5,
                            provisional_100k_decisions_per_second_met=rates['off']['decisions']>=100_000)
    return result


def validate_resident(report):
    try:
        require(report['schema_version']==1 and report['workload']=='scalar-resident-v1','unsupported resident artifact version')
        require(report['status']=='measured','failed resident probe')
        minima={'typical':1,'token-heavy':4,'target-rich':6,'stack-heavy':3}
        specimen=report['specimen']
        require(specimen['kind'] in minima and specimen['score']>=minima[specimen['kind']], 'missing stress position')
        expected=[(cycle,n) for cycle in range(3) for n in (0,1,32,128,512,1000,5000,10000)]
        samples=report['samples']
        require([(r['cycle'],r['resident']) for r in samples]==expected,'omitted or reordered resident sweep')
        require([r['cycle'] for r in report['released']]==[0,1,2],'omitted release cycle')
        require([(r['cycle'],r['resident']) for r in report['reset_churn']]==[(i,10000) for i in range(3)],'omitted reset churn')
        for row in [*samples,*report['released'],*report['reset_churn']]:
            require(integer(row['rss_bytes']) and row['rss_bytes']>0 and integer(row['high_water_bytes'])
                    and row['high_water_bytes']>=row['rss_bytes'],'invalid OS memory accounting')
        marginal=(samples[7]['rss_bytes']-samples[0]['rss_bytes'])/10000
        require(marginal>0,'resident probe did not touch/retain states')
        return dict(first_cycle_bytes_per_state=marginal,provisional_64kib_met=marginal<=65536,
                    total_rss_at_10000=[r['rss_bytes'] for r in samples if r['resident']==10000],
                    released_rss=[r['rss_bytes'] for r in report['released']],
                    limitation='three cycles expose allocator retention; not a proof of no long-running leak')
    except (KeyError,TypeError,IndexError) as error:
        raise ValueError(f'malformed resident report: {error}') from error


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reports',type=Path,nargs='+')
    args=parser.parse_args()
    try:
        for path in args.reports:
            print(json.dumps({'path':str(path), **validate_report(json.loads(path.read_text()))},sort_keys=True))
    except (ValueError,OSError) as error:
        parser.exit(1,f'Artifact validation failed: {error}\n')


if __name__ == '__main__':
    main()
