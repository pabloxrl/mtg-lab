"""B008/B014/B019/B020/B021 campaign arithmetic and actual native evidence."""
import copy
import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

from test_scalar_artifact import example

ROOT = Path(__file__).resolve().parents[1]
MODES = ('off', 'counters', 'sampled_trace', 'full_replay')
POLICIES = ('heuristic-activation-mana-v1', 'legal-random-activation-mana-v1')


def literal_reports():
    # Independent ledger: 60 natural completions + one unfinished per 30s.
    # Full replay requires every started recording complete; unfinished work
    # remains in the other modes' literal accounting controls.
    reports = []
    for policy in POLICIES:
        for encoded in (False, True):
            for mode in MODES:
                r = example()
                r.update(schema_version=2, workload='scalar-four-modes-v1')
                r['pins']['config'].update(schema_version=2, workload=r['workload'], policies=[policy]*2,
                    master_seed=42, policy_seed=42, instrumentation=mode, encoding=encoded)
                r['pins']['resolved_episode']['policies']=[policy]*2
                r['pins']['resolved_episode']['native']['instrumentation']=mode
                r['hardware'] = dict(binary_sha256='1'*64, source_sha256='2'*64, cpu_affinity='0',rustc='pinned',target='test',opt_level='3',rustflags='',debug_assertions=False)
                r['pins']['execution'] = dict(replay=dict(max_bytes=67108864, max_records=20000,
                    persistence='in_memory'), trace=dict(every=64, capacity=256) if mode=='sampled_trace' else None,
                    capture='none', sampled_timing='not_measured')
                r['collection'].update(before=dict(affinity=[0],cpu_quota='400000 100000',memory_limit='7516192768'),
                    after=dict(affinity=[0],cpu_quota='400000 100000',memory_limit='7516192768'))
                for i,w in enumerate([r['warmup'], *r['windows']]):
                    w['execution'] = dict(not_started=0,trace_selected=0,trace_retained=0,trace_dropped=0,
                        replay_verified=60 if mode=='full_replay' else 0,replay_bytes=600 if mode=='full_replay' else 0,
                        replay_failed=0,replay_incomplete=0,replay_ns=0,
                        publication_ns=0,published=0,publication_failed=0)
                    if mode=='full_replay':
                        w.update(unfinished=0,started=60,attempts=60,completed_decisions=600,
                                 mean_decisions_per_completed_game=10.0,first_episode=60*i)
                        w['row_attempts']=[sum(1 for n in range(60*i,60*(i+1)) if n%8==j) for j in range(8)]
                        w['row_completed']=w['row_attempts'].copy()
                    if encoded:
                        w['phases'].update(encoding_ns=1, encoded_bytes=600)
                    if mode!='off':
                        w['counters']=dict(overflowed=False,capacity_overflows=0,decisions=600,logical_actions=600)
                        w['logical_actions_per_second']=20.0
                reports.append(r)
    return reports


class MeasurementArithmetic(unittest.TestCase):
    def api(self):
        self.assertIsNotNone(importlib.util.find_spec('scripts.m2_measurement'),
                             'missing campaign validator for the delivered four-mode artifacts')
        from scripts import m2_measurement
        return m2_measurement

    def test_literal_matrix_and_denominators(self):
        summary=self.api().validate_campaign(literal_reports())
        self.assertEqual(summary['completed'], 4800)
        self.assertEqual(summary['elapsed_ns'], 2400_000_000_000)
        self.assertEqual(summary['unfinished'], 60)
        self.assertEqual(len(summary['configurations']), 16)
        for group in summary['configurations'].values():
            self.assertEqual(group['games_per_second']['samples'], [2.0]*5)
            self.assertEqual(group['overhead_percent']['samples'], [0.0]*25)

    def test_missing_contract_and_tampered_accounting_rejected(self):
        for mutation in ('mode','policy','row','window','horizon','failed','denominator','replay','trace',
                         'capacity','persistence','binary','extension','encoding','resources','flags','sampling','resolved_policy','resolved_mode','empty_encoding'):
            reports=literal_reports(); r=reports[-1]; w=r['windows'][0]
            if mutation=='mode': reports=[r for r in reports if r['pins']['config']['instrumentation']!='off']
            elif mutation=='policy': reports=reports[:8]
            elif mutation=='row': w['row_attempts']=[61]+[0]*7
            elif mutation=='window': r['windows'].pop()
            elif mutation=='horizon': r['pins']['resolved_episode']['max_decisions']=100
            elif mutation=='failed': w['failed']=1
            elif mutation=='denominator': w['elapsed_ns']=1
            elif mutation=='replay': w['execution']['replay_verified']=59
            elif mutation=='trace': reports[2]['pins']['execution']['trace']=None
            elif mutation=='capacity': del r['pins']['execution']['replay']['max_bytes']
            elif mutation=='persistence': del r['pins']['execution']['replay']['persistence']
            elif mutation=='binary': r['hardware']['binary_sha256']='3'*64
            elif mutation=='extension': reports.append(copy.deepcopy(r))
            elif mutation=='encoding': w['phases']['encoding_ns']=None
            elif mutation=='flags': del r['hardware']['rustflags']
            elif mutation=='sampling': del r['pins']['execution']['sampled_timing']
            elif mutation=='resolved_policy': r['pins']['resolved_episode']['policies']=[POLICIES[0]]*2
            elif mutation=='resolved_mode': r['pins']['resolved_episode']['native']['instrumentation']='off'
            elif mutation=='empty_encoding': w['phases'].update(encoding_ns=0,encoded_bytes=0)
            else: r['collection']['after']['memory_limit']='max'
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                self.api().validate_campaign(reports)

    def test_consistently_counted_incomplete_replay_still_rejects(self):
        reports=literal_reports(); r=reports[-1]; w=r['windows'][-1]
        # No count mismatch: replace one natural completion with one unfinished
        # replay and independently recompute every affected count/rate.
        w['completed']-=1; w['wins'][0]-=1; w['unfinished']+=1
        w['row_completed'][w['first_episode']%8]-=1
        w['execution']['replay_verified']-=1; w['execution']['replay_incomplete']+=1
        w['completed_decisions']=590; w['completed_games_per_second']=59/30
        w['mean_decisions_per_completed_game']=590/59
        samples=[2.0]*4+[59/30]
        r['completed_games_per_second']=dict(samples=samples,mean=sum(samples)/5,
                                           p50=2.0,p95=2.0,min=59/30,max=2.0)
        with self.assertRaisesRegex(ValueError,'incomplete'):
            self.api().validate_campaign(reports)

    def test_slow_warmup_does_not_invent_a_minimum_game_rate(self):
        reports=literal_reports(); r=reports[-1]; w=r['warmup']
        w.update(attempts=1,started=1,completed=1,wins=[1,0],decisions=10,completed_decisions=10)
        w['row_attempts']=[1]+[0]*7; w['row_completed']=[1]+[0]*7
        w['counters'].update(decisions=10,logical_actions=10)
        w['execution'].update(replay_verified=1,replay_bytes=10)
        for i,w in enumerate(r['windows']):
            w['first_episode']=1+60*i
            w['row_attempts']=[sum(1 for n in range(1+60*i,1+60*(i+1)) if n%8==j) for j in range(8)]
            w['row_completed']=w['row_attempts'].copy()
        try:
            summary=self.api().validate_campaign(reports)
        except ValueError as error:
            self.fail(f'B020 requires 10s warmup, not eight warmup completions: {error}')
        self.assertEqual(summary['completed'],4800)

    def test_off_resolved_mode_uses_declared_serde_omission(self):
        reports=literal_reports()
        for r in reports:
            if r['pins']['config']['instrumentation']=='off':
                del r['pins']['resolved_episode']['native']['instrumentation']
        try:
            summary=self.api().validate_campaign(reports)
        except ValueError as error:
            self.fail(f'native Config explicitly omits Off; this must remain valid: {error}')
        self.assertEqual(summary['completed'],4800)

    def test_variance_rule_is_literal_population_cv(self):
        api=self.api()
        self.assertFalse(api.needs_extension([2,2,2,2,2]))
        self.assertTrue(api.needs_extension([1,1,1,1,2]))
        self.assertEqual(api.distribution([1,2,3,4,5])['p95'],5)

    def test_literal_boundary_rollup_preserves_encoding_availability(self):
        import runpy
        function=runpy.run_path(str(ROOT/'doc/evidence/four-mode-measurement/summarize.py'))['window_costs']
        costs=function(literal_reports())
        native=costs[POLICIES[0]+'/native/off']
        self.assertEqual(native['elapsed_ns'],150_000_000_000)
        self.assertEqual(native['phases']['reset_ns'],5_000_000_000)
        self.assertEqual(native['policy_ns'],5_000_000_000)
        self.assertIsNone(native['phases']['encoding_ns'])
        encoded=costs[POLICIES[0]+'/encoded/off']
        self.assertEqual(encoded['phases']['encoding_ns'],5)
        self.assertEqual(encoded['phases']['encoded_bytes'],3000)
        self.assertNotIn('effect_dispatch_ns',encoded)

    def test_archive_rejects_unreported_failed_attempt(self):
        from scripts.m2_measurement import validate_archive
        import hashlib
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp); rows=[]
            for index,report in enumerate(literal_reports()):
                name=f'run-{index}'; folder=root/name; folder.mkdir()
                data=json.dumps(report).encode()
                (folder/'report.json').write_bytes(data)
                (folder/'raw.json').write_text(json.dumps({k:v for k,v in report.items() if k!='collection'}))
                (folder/'config.json').write_text(json.dumps(report['pins']['config']))
                (folder/'collection.json').write_text(json.dumps(report['collection']))
                rows.append(dict(name=name,status='collected',config=report['pins']['config'],
                                 report_sha256=hashlib.sha256(data).hexdigest()))
            manifest=dict(runs=rows,binary_sha256='1'*64)
            (root/'campaign.json').write_text(json.dumps(manifest))
            self.assertEqual(validate_archive(root)['completed'],4800)
            failed=root/'failed-attempt'; failed.mkdir()
            (failed/'collection.json').write_text('{"returncode":3,"wall_seconds":29}')
            with self.assertRaisesRegex(ValueError,'omitted'): validate_archive(root)
            manifest['runs'].append(dict(name='failed-attempt',status='failed'))
            (root/'campaign.json').write_text(json.dumps(manifest))
            with self.assertRaisesRegex(ValueError,'unsuccessful'): validate_archive(root)


class MeasurementArchivedEvidence(unittest.TestCase):
    def test_published_archives_recompute_and_reject_tampered_accounting(self):
        import hashlib
        import gzip
        import runpy
        evidence=ROOT/'doc/evidence/four-mode-measurement'
        binary=json.loads((evidence/'binary.json').read_text())
        compressed=(evidence/'measured-mtg.gz').read_bytes()
        self.assertEqual(hashlib.sha256(compressed).hexdigest(),binary['archive_sha256'])
        self.assertEqual(hashlib.sha256(gzip.decompress(compressed)).hexdigest(),binary['binary_sha256'])
        unpack=runpy.run_path(str(evidence/'archive.py'))['unpack_verified']
        summarize=runpy.run_path(str(evidence/'summarize.py'))['summarize']
        from scripts.m2_measurement import validate_archive
        with tempfile.TemporaryDirectory() as temporary:
            root=Path(temporary)
            unpack(evidence/'throughput.tar.gz',evidence/'throughput-index.json',root/'throughput')
            unpack(evidence/'privileged-diagnostics.tar.gz',evidence/'diagnostics-index.json',root/'diagnostics')
            actual=summarize(root/'throughput',root/'diagnostics')
            self.assertEqual(actual,json.loads((evidence/'summary.json').read_text()))
            self.assertEqual(len(actual['configurations']),16)
            self.assertGreater(actual['latency']['observations'],0)
            manifest_path=root/'throughput/campaign.json'
            manifest=json.loads(manifest_path.read_text())
            for field,path in (('plan_sha256',evidence/'plan.md'),
                               ('script_sha256',evidence/'campaign.py'),
                               ('validator_sha256',evidence/'validator-at-start.txt'),
                               ('collector_sha256',ROOT/'scripts/scalar_modes.py')):
                self.assertEqual(manifest[field],hashlib.sha256(path.read_bytes()).hexdigest())
            self.assertEqual(manifest['binary_sha256'],binary['binary_sha256'])
            row=manifest['runs'][0];folder=root/'throughput'/row['name']
            report=json.loads((folder/'report.json').read_text())
            report['windows'][0]['elapsed_ns']=1
            data=json.dumps(report).encode()
            (folder/'report.json').write_bytes(data)
            (folder/'raw.json').write_text(json.dumps({k:v for k,v in report.items() if k!='collection'}))
            row['report_sha256']=hashlib.sha256(data).hexdigest()
            manifest_path.write_text(json.dumps(manifest))
            # Re-hashing corruption must not turn a broken real denominator into
            # valid evidence. Earlier literal controls independently fix rates.
            with self.assertRaises(ValueError): validate_archive(root/'throughput')
            corrupted=root/'corrupted.tar.gz'
            corrupted.write_bytes((evidence/'throughput.tar.gz').read_bytes()+b'corrupt')
            with self.assertRaisesRegex(ValueError,'identity'):
                unpack(corrupted,evidence/'throughput-index.json',root/'unused')


class MeasurementRealAdapter(unittest.TestCase):
    def test_real_normal_reset_exports_and_negative_controls(self):
        # Invoke the delivered adapter, preserving its native semantics/capture
        # checks. This is supplemental finite evidence, never a timed campaign.
        from test_m2_repair_modes import FourModeContract
        from scripts.scalar_modes import validate_execution
        FourModeContract.setUpClass()
        try:
            receipt=FourModeContract.receipt
            self.assertGreater(validate_execution(receipt)['observations'],0)
            for key in ('attempts','elapsed_ns','completed'):
                bad=copy.deepcopy(receipt); bad['runs'][0]['window'][key]=0
                with self.subTest(key=key),self.assertRaises(ValueError): validate_execution(bad)
        finally:
            FourModeContract.tearDownClass()

    def test_real_fixed_episode_latency_matrix(self):
        from test_m2_repair_timing import validate
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'native.json'
            env=dict(os.environ, MTG_MEASUREMENT_NATIVE_OUTPUT=str(path))
            fd=env.get('MTG_SYMPHONY_LOCK_FD')
            command=['cargo','test','--locked','-p','mtg-cli','measurement_export_native_latency_and_equivalence','--','--nocapture']
            result=subprocess.run(command,cwd=ROOT,env=env,capture_output=True,text=True,
                pass_fds=() if fd is None else (int(fd),),timeout=1800)
            self.assertEqual(result.returncode,0,result.stdout[-4000:]+result.stderr[-4000:])
            self.assertTrue(path.is_file(),'real native export required')
            receipt=json.loads(path.read_text())
            if destination:=os.environ.get('MTG_MEASUREMENT_EVIDENCE'):
                Path(destination).write_bytes(path.read_bytes())
        rows=receipt['runs']
        self.assertEqual(len(rows),128)
        self.assertEqual({(r['policy'],r['encoded'],r['mode'],r['row']) for r in rows},
            {(p,e,m,row) for p in POLICIES for e in (False,True) for m in MODES for row in range(8)})
        baseline={}
        for r in rows:
            self.assertGreater(r['observations'],0)
            self.assertEqual(r['replay_contract'],dict(max_bytes=67108864,max_records=20000,persistence='in_memory'))
            self.assertEqual(r['replay_verified'],int(r['mode']=='full_replay'))
            self.assertEqual(r['config']['max_decisions'],20000)
            identity=[r[k] for k in ('observations','history_sha256','final_sha256','observations_sha256')]
            self.assertEqual(identity,baseline.setdefault((r['policy'],r['row']),identity))
            if r['mode']=='off': self.assertIsNone(r['latency'])
            else:
                validate(r['latency'])
                bad=copy.deepcopy(r['latency']); bad['phases']['policy']['skipped']+=1
                with self.assertRaises(ValueError): validate(bad)

    def test_real_profile_trace_and_cost_rejections(self):
        from test_m2_repair_profile import FixedTraceProfile, validate
        FixedTraceProfile.setUpClass()
        adapter=FixedTraceProfile('test_real_export_rejects_accounting_tampering')
        trace=adapter.fixture('effects')
        report=adapter.execute(trace)
        validate(report)
        self.assertGreater(report['observations'],0)
        self.assertEqual(report['effect_probe']['dispatches'],2)
        invalid=copy.deepcopy(trace); invalid['checkpoints'][0]['life'][0]=19
        self.assertIn('error',adapter.execute(invalid))
        for change in ('effect','encoding','denominator'):
            bad=copy.deepcopy(report)
            if change=='effect': bad['effect_probe']['scope']='inferred from transition difference'
            elif change=='encoding': bad['costs']['encoding']['ns']+=1
            else: bad['elapsed_ns']=0
            with self.subTest(change=change), self.assertRaises(ValueError): validate(bad)
