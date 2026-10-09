"""RFC B008/B019/B020: literal counts and time are independent of engine output."""
import copy
import unittest
from scripts.scalar_artifact import validate_report


def example():
    # Five 30s windows: 60 terminal games each, 30 per winner; one unfinished.
    w = dict(elapsed_ns=30_000_000_000, started=61, attempts=61,
             completed=60, wins=[30, 30], draws=0, failed=0, unfinished=1,
             truncated=0, concessions=0, decisions=600, completed_decisions=590,
             stop_code=0, policy_ns=1_000_000_000, counters=None,
             phases=dict(reset_ns=1_000_000_000, transition_ns=4_000_000_000,
                         legality_and_view_ns=2_000_000_000, finalization_ns=1_000_000_000,
                         encoding_ns=None, encoded_bytes=0),
             completed_games_per_second=2.0, decisions_per_second=20.0,
             mean_decisions_per_completed_game=590/60,
             logical_actions_per_second=None, has_rules_completions=True)
    windows = [dict(copy.deepcopy(w), first_episode=61*(i+1)) for i in range(5)]
    warmup = dict(copy.deepcopy(w), elapsed_ns=10_000_000_000, first_episode=0)
    for w in [warmup, *windows]:
        w['row_attempts']=[sum(1 for n in range(w['first_episode'],w['first_episode']+w['attempts']) if n%8==row) for row in range(8)]
        w['row_completed']=w['row_attempts'].copy()
        w['row_completed'][(w['first_episode']+60)%8]-=1
    return dict(status="measured", workload="scalar-full-pool-v1", warmup=warmup,
                windows=windows, pins=dict(resolved_episode=dict(max_decisions=20000, native=dict(max_work_calls=100000,max_records=20000)), config=dict(warmup_seconds=10, window_seconds=30,
                windows=5, instrumentation="off", encoding=False)),
                completed_games_per_second=dict(samples=[2.0]*5, mean=2.0,p50=2.0,p95=2.0,min=2.0,max=2.0),
                collection=dict(contaminated=False, profiler=False, returncode=0, heavy_lock=True))


class ScalarArtifactTests(unittest.TestCase):
    def test_literal_accounting(self):
        result = validate_report(example())
        self.assertEqual(result["completed"], 300)

    def test_omitted_window_rejected(self):
        v=example(); v["windows"].pop()
        with self.assertRaises(ValueError): validate_report(v)

    def test_contamination_rejected(self):
        for key,value in [("contaminated",True),("profiler",True),("returncode",3),("heavy_lock",False)]:
            v=example(); v["collection"][key]=value
            with self.subTest(key=key), self.assertRaises(ValueError): validate_report(v)

    def test_horizon_duration_failure_overflow_and_accounting_rejected(self):
        for key,value in [("elapsed_ns",29_999_999_999),("failed",1),("completed",61),
                          ("stop_code",3),("first_episode",0),("decisions",-1),
                          ("completed_games_per_second",3.0),("counters",{"overflowed":True})]:
            v=example(); v["windows"][1][key]=value
            with self.subTest(key=key), self.assertRaises(ValueError): validate_report(v)

    def test_best_window_summary_rejected(self):
        v=example(); v["completed_games_per_second"]["samples"]=[2.0]
        with self.assertRaises(ValueError): validate_report(v)

    def test_warmup_omission_or_shortening_rejected(self):
        for n in [0,9_999_999_999]:
            v=example(); v["warmup"]["elapsed_ns"]=n
            with self.subTest(n=n), self.assertRaises(ValueError): validate_report(v)

    def test_shortened_game_horizon_rejected(self):
        v=example();v['pins']['resolved_episode']['max_decisions']=100
        with self.assertRaises(ValueError): validate_report(v)

    def test_full_pool_rows_cannot_be_omitted_or_fabricated(self):
        for key,value in [('row_attempts',[61]+[0]*7),('row_completed',[60]+[0]*7)]:
            v=example();v['windows'][1][key]=value
            with self.subTest(key=key), self.assertRaises(ValueError): validate_report(v)

    def test_deadline_before_reset_retains_an_unstarted_attempt(self):
        # Native control may expire between scheduling an attempt and reset.
        # It starts no game; the outer window still retains that attempt/time.
        v=example();w=v['windows'][-1]
        w['row_attempts'][(w['first_episode']+w['attempts'])%8]+=1
        w['attempts']+=1
        self.assertEqual(validate_report(v)['not_started'],1)
        w['attempts']=w['started']-1
        with self.assertRaises(ValueError):validate_report(v)


class ProcessReceiptTests(unittest.TestCase):
    def test_os_receipt_retains_failure_and_memory(self):
        import os
        import sys
        import tempfile
        from scripts.collect_scalar_baseline import measured_process
        with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
            receipt=measured_process([sys.executable,'-c','raise SystemExit(7)'],out,err,out.fileno(),5)
        self.assertEqual(receipt['returncode'],7)
        self.assertGreater(receipt['peak_rss_bytes'],0)
        self.assertFalse(receipt['timed_out'])
        self.assertGreater(receipt['wall_seconds'],0)

    def test_os_receipt_retains_timeout(self):
        import sys
        import tempfile
        from scripts.collect_scalar_baseline import measured_process
        with tempfile.TemporaryFile() as out, tempfile.TemporaryFile() as err:
            receipt=measured_process([sys.executable,'-c','import time; time.sleep(30)'],out,err,out.fileno(),.1)
        self.assertLess(receipt['returncode'],0)
        self.assertTrue(receipt['timed_out'])


class ScalarPairTests(unittest.TestCase):
    def suite(self):
        runs=[]
        for policy in ('heuristic-activation-mana-v1','legal-random-activation-mana-v1'):
            for mode in ('off','counters'):
                v=example();v['pins']['config'].update(policies=[policy]*2,instrumentation=mode,master_seed=42,policy_seed=42)
                v['hardware']=dict(binary_sha256='a'*64,source_sha256='b'*64,cpu_affinity='0')
                for w in [v['warmup'],*v['windows']]:
                    if mode=='counters':
                        w['counters']=dict(overflowed=False,capacity_overflows=0,decisions=w['decisions'],logical_actions=300)
                        w['logical_actions_per_second']=300*1e9/w['elapsed_ns']
                runs.append(v)
        return runs

    def test_identical_rates_have_zero_overhead(self):
        from scripts.scalar_artifact import validate_suite
        result=validate_suite(self.suite())
        self.assertEqual(result['heuristic-activation-mana-v1']['counters_overhead_percent'],0)

    def test_missing_pair_and_changed_seed_or_binary_rejected(self):
        from scripts.scalar_artifact import validate_suite
        for mutation in ('missing','seed','binary'):
            runs=self.suite()
            if mutation=='missing': runs.pop()
            elif mutation=='seed': runs[1]['pins']['config']['master_seed']=43
            else: runs[1]['hardware']['binary_sha256']='c'*64
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):validate_suite(runs)


class ResidentArtifactTests(unittest.TestCase):
    def example(self):
        counts=[0,1,32,128,512,1000,5000,10000]
        return dict(schema_version=1,workload='scalar-resident-v1',status='measured',specimen=dict(kind='token-heavy',score=8),
            samples=[dict(cycle=cycle,resident=n,rss_bytes=1_000_000+1024*n,high_water_bytes=20_000_000)
                     for cycle in range(3) for n in counts],
            released=[dict(cycle=i,rss_bytes=2_000_000,high_water_bytes=20_000_000) for i in range(3)],
            reset_churn=[dict(cycle=i,resident=10000,rss_bytes=11_240_000,high_water_bytes=20_000_000) for i in range(3)])

    def test_literal_os_slope(self):
        from scripts.scalar_artifact import validate_resident
        self.assertEqual(validate_resident(self.example())['first_cycle_bytes_per_state'],1024)

    def test_missing_capacity_point_or_stress_rejected(self):
        from scripts.scalar_artifact import validate_resident
        for mutation in ('point','class','rss'):
            v=self.example()
            if mutation=='point':v['samples'].pop()
            elif mutation=='class':v['specimen']['score']=0
            else:v['samples'][3]['rss_bytes']=-1
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):validate_resident(v)

    def test_reset_churn_must_not_be_omitted(self):
        from scripts.scalar_artifact import validate_resident
        v=self.example();del v['reset_churn']
        with self.assertRaises(ValueError):validate_resident(v)

    def test_resident_version_rejected(self):
        from scripts.scalar_artifact import validate_resident
        v=self.example();v['schema_version']=2
        with self.assertRaises(ValueError):validate_resident(v)
