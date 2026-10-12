"""GH-318: L0 has its own launch authority and cannot wait for held M3."""
import copy
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from test_engine_program import documents, issue, receipt
from check_engine_program import select_successor


def l0_snapshot():
    program, _, _ = documents()
    program = copy.deepcopy(program)
    program.update(coordinator_issue=318, prerequisites=[26, 277, 318], github_milestone=8)
    # Model two independent concrete deliveries and an integrated gate.
    program['tasks'] = [dict(issue=401, kind='implementation', depends_on=[]),
                        dict(issue=402, kind='operations', depends_on=[401]),
                        dict(issue=403, kind='gate', depends_on=[402])]
    objects = [issue(7), issue(26, 'closed'), issue(277, 'closed'),
               issue(318, names=['agent-ready'])] + [issue(n) for n in (401,402,403)]
    for obj in objects:
        obj['milestone'] = {'number': 8}
    snap = dict(current=318, current_success=True, issues=objects,
                receipts={str(n): receipt() for n in (26,277,318)},
                open_claims=[issue(318,names=['agent-ready'])],
                events={str(n): [] for n in (401,402,403)})
    return program, snap


class L0HandoffTests(unittest.TestCase):
    def test_launch_uses_l0_registration_without_m3(self):
        p,s = l0_snapshot()
        self.assertEqual(select_successor(p,s), {'successor':401,'already_owned':False})

    def test_requires_initial_terminal_admission_receipt(self):
        p,s = l0_snapshot()
        s['receipts'].pop('277')
        with self.assertRaisesRegex(ValueError, 'unverified program prerequisite'):
            select_successor(p,s)

    def test_registration_ci_must_match(self):
        p,s = l0_snapshot()
        s['receipts']['318']['ci_sha'] = 'b'*40
        with self.assertRaisesRegex(ValueError, 'unverified registration'):
            select_successor(p,s)

    def test_operations_child_is_required_by_final_gate(self):
        p,s = l0_snapshot()
        for obj in s['issues']:
            if obj['number'] in (401,402):
                obj.update(state='closed',state_reason='completed')
                s['receipts'][str(obj['number'])] = receipt()
        s['receipts'].pop('402')
        self.assertIsNone(select_successor(p,s)['successor'])

    def test_m3_claims_count_toward_same_capacity(self):
        p,s = l0_snapshot()
        s['open_claims'] += [issue(n,names=['agent-running']) for n in (299,300,301)]
        self.assertEqual(select_successor(p,s), {'successor':None,'reason':'capacity'})


class L0RegistryTests(unittest.TestCase):
    def setUp(self):
        import json
        from check_l0_program import PATH, CONTRACT
        root = Path(__file__).resolve().parents[1]
        self.p = json.loads((root / PATH).read_text())
        self.source = (root / CONTRACT).read_bytes()
        _, _, self.legacy = documents()
        self.m3 = json.loads((root / 'doc/programs/engine-validation-m3.json').read_text())

    def validate(self):
        from check_l0_program import validate
        return validate(self.p, self.source, self.legacy, self.m3)

    def test_committed_registry_is_capture_first_with_operations_and_full_gate(self):
        before = copy.deepcopy(self.p)
        self.assertEqual(len(self.validate()),21)
        self.assertEqual(self.p,before)
        self.assertEqual(self.p['tasks'][0]['issue'],319)
        self.assertEqual(self.p['prerequisites'],[26,277,318])
        self.assertEqual(self.p['tasks'][-1]['depends_on'],list(range(319,337))+[338,339])
        self.assertEqual(next(t for t in self.p['tasks'] if t['issue']==332)['kind'],'operations')

    def test_source_hash_and_authority_are_pinned(self):
        for field,value in [('rfc_sha256','0'*64),('coordinator_issue',241),
                            ('github_milestone',7),('authorized_milestones',['EV-M3']),
                            ('prerequisites',[26,241,318]),('schema_version',True)]:
            with self.subTest(field=field):
                before = copy.deepcopy(self.p)
                self.p[field]=value
                with self.assertRaises(ValueError): self.validate()
                self.p=before

    def test_m3_task_cannot_enter_l0(self):
        self.p['tasks'][1]['issue']=299
        self.p['requirements'][1]['owner']=299
        for t in self.p['tasks']:
            t['depends_on']=[299 if n==320 else n for n in t['depends_on']]
        with self.assertRaisesRegex(ValueError,'M3'): self.validate()

    def test_gate_cannot_omit_operations(self):
        for t in self.p['tasks']:
            t['depends_on']=[n for n in t['depends_on'] if n!=332]
        with self.assertRaisesRegex(ValueError,'gate omits'): self.validate()

    def test_capture_cannot_have_architecture_prerequisite(self):
        self.p['tasks'][0]['key']='architecture'
        with self.assertRaisesRegex(ValueError,'concrete capture'): self.validate()

    def test_bad_dependency_cycle_and_owner_fail(self):
        for mutation in ('unknown','cycle','owner','coverage','legacy'):
            with self.subTest(mutation=mutation):
                before=copy.deepcopy(self.p)
                if mutation=='unknown': self.p['tasks'][0]['depends_on']=[9999]
                elif mutation=='cycle': self.p['tasks'][0]['depends_on']=[320]
                elif mutation=='owner': self.p['requirements'][0]['owner']=337
                elif mutation=='coverage': self.p['tasks'][-1]['requirements'].pop()
                else: self.p['tasks'][0]['issue']=27
                with self.assertRaises(ValueError): self.validate()
                self.p=before

    def test_real_registry_launch_and_independent_forge_successor(self):
        from check_engine_program import select_successor
        p=self.p
        objs=[issue(7),issue(26,'closed'),issue(277,'closed'),issue(318,names=['agent-ready'])]
        objs += [issue(t['issue']) for t in p['tasks']]
        for obj in objs: obj['milestone']={'number':8}
        snap=dict(current=318,current_success=True,issues=objs,open_claims=[],
                  receipts={str(n):receipt() for n in (26,277,318)},
                  events={str(t['issue']):[] for t in p['tasks']})
        self.assertEqual(select_successor(p,snap)['successor'],319)
        next(o for o in objs if o['number']==319).update(state='closed',state_reason='completed')
        snap['receipts']['319']=receipt()
        next(o for o in objs if o['number']==320)['labels']=[{'name':'agent-blocked'}]
        self.assertEqual(select_successor(p,snap)['successor'],324)
        snap['events']['324']=[{'event':'labeled','label':{'name':'agent-ready'}}]
        self.assertIsNone(select_successor(p,snap)['successor'])
        next(o for o in objs if o['number']==324)['labels']=[{'name':'agent-resume-authorized'}]
        self.assertEqual(select_successor(p,snap)['successor'],324)
        snap['reserved_successor']=324
        next(o for o in objs if o['number']==324)['labels']=[{'name':'agent-ready'}]
        self.assertEqual(select_successor(p,snap),{'successor':324,'already_owned':True})
        objs[0]['labels']=[{'name':'program-paused'}]
        with self.assertRaisesRegex(ValueError,'parent'): select_successor(p,snap)

    def test_continuous_monitoring_has_real_consumers_and_policy_dependency(self):
        tasks={t['key']:t for t in self.p['tasks']}
        self.assertIn('matched_benchmark',tasks)
        self.assertIn('performance_history',tasks)
        self.assertEqual(tasks['matched_benchmark']['depends_on'],[322,330])
        self.assertEqual(tasks['performance_history']['depends_on'],[338])
        self.assertIn(339,tasks['replay_policy']['depends_on'])
        self.assertTrue({'L0-019','L0-020'} <= set(tasks['integrated_gate']['requirements']))
        self.assertEqual(tasks['xmage_capture']['depends_on'],[])
