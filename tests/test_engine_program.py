"""GH-241: operator-scoped M3 cannot resume legacy work or bypass handoff gates."""
import copy
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
from check_engine_program import PATH, RFC, select_successor, validate


def documents():
    return (json.loads((ROOT / PATH).read_text()), (ROOT / RFC).read_bytes(),
            json.loads((ROOT / 'doc/programs/rfc-0002.json').read_text()))


def receipt():
    url = 'https://github.com/pabloxrl/mtg-lab/'
    return dict(merge_sha='a'*40, ci_sha='a'*40, ci_conclusion='success',
                review_verdict='pass', acceptance=url+'blob/a/report',
                review=url+'pull/1', pull_request=url+'pull/1',
                main_ci=url+'actions/runs/1', workpad=url+'issues/1#issuecomment-1')


def issue(n, state='open', names=()):
    return dict(number=n, state=state, state_reason='completed' if state == 'closed' else None,
                labels=[{'name': s} for s in names], milestone={'number': 7})


def snapshot(program):
    return dict(current=241, current_success=True,
                issues=[issue(7), issue(26, 'closed'), issue(241, names=['agent-ready'])]
                + [issue(t['issue']) for t in program['tasks']],
                open_claims=[issue(241, names=['agent-ready','agent-running'])],
                receipts={'26': receipt(), '241': receipt()},
                events={str(t['issue']): [] for t in program['tasks']})


class RegistryTests(unittest.TestCase):
    def setUp(self):
        self.p, self.source, self.legacy = documents()

    def reject(self, pattern):
        with self.assertRaisesRegex(ValueError, pattern):
            validate(self.p, self.source, self.legacy)

    def test_committed_registry_and_legacy_separation(self):
        before = copy.deepcopy(self.p)
        tasks = validate(self.p, self.source, self.legacy)
        self.assertEqual(len(tasks), 18)
        self.assertEqual(self.p, before)
        self.assertEqual(self.legacy['authorized_milestones'], ['M0','M1','M2'])

    def test_namespace_authority_schema_and_milestone(self):
        for field, value in [('program','rfc-0002'),('schema_version',True),
                             ('coordinator_issue',300),('github_milestone',3),
                             ('parent_issue',299),('authorized_milestones',['M3']),
                             ('prerequisites',[26]),('rfc_sha256','0'*64)]:
            with self.subTest(field=field):
                self.p, self.source, self.legacy = documents()
                self.p[field] = value
                self.reject('namespace|schema|authority|milestone|parent|prerequisites|pin')

    def test_unknown_and_missing_fields_fail_closed(self):
        self.p['fallback'] = True
        self.reject('fields')
        del self.p['fallback']; del self.p['requirements']
        self.reject('fields')

    def test_legacy_authorization_and_task_reuse_rejected(self):
        self.legacy['authorized_milestones'].append('M3')
        self.reject('legacy authorization')
        self.legacy['authorized_milestones'].pop()
        self.p['tasks'][0]['issue'] = 27
        self.reject('legacy task')

    def test_task_type_numbers_duplicates_and_controls(self):
        for field, value in [('issue',True),('issue',7),('issue',0),('kind','operations'),
                             ('milestone','E2'),('depends_on',[999]),
                             ('depends_on',[299]),('depends_on',[300,300]),
                             ('requirements',['EV3-999'])]:
            with self.subTest(field=field, value=value):
                self.p, self.source, self.legacy = documents()
                self.p['tasks'][0][field] = value
                self.reject('invalid|unknown|unauthorized|self|duplicate')
        self.p, self.source, self.legacy = documents()
        self.p['tasks'].append(copy.deepcopy(self.p['tasks'][0]))
        self.reject('duplicate task')

    def test_cycle_and_gate_missing_prerequisite(self):
        self.p['tasks'][0]['depends_on'] = [300]
        self.reject('cycle')
        self.p['tasks'][0]['depends_on'] = []
        self.p['tasks'][-1]['depends_on'] = [299]
        self.reject('gate omits')

    def test_requirement_owner_and_gate_coverage(self):
        self.p['requirements'][0]['owner'] = 316
        self.reject('owner')
        self.p['requirements'][0]['owner'] = 299
        self.p['tasks'][-1]['requirements'].pop()
        self.reject('gate omits requirements')


class HandoffTests(unittest.TestCase):
    def setUp(self):
        self.p, _, _ = documents()
        self.s = snapshot(self.p)

    def target(self,n):
        return next(i for i in self.s['issues'] if i['number'] == n)

    def choose(self):
        return select_successor(self.p,self.s)

    def complete(self,n):
        self.target(n).update(state='closed',state_reason='completed')
        self.s['receipts'][str(n)] = receipt()

    def test_first_activation_after_verified_registration_only(self):
        self.assertEqual(self.choose(), {'successor':299,'already_owned':False})
        self.s['receipts']['241']['ci_sha'] = 'b'*40
        with self.assertRaisesRegex(ValueError,'unverified registration'):
            self.choose()

    def test_parent_and_current_controls_win(self):
        for n, name in [(7,'program-paused'),(7,'program-cancelled'),
                        (241,'agent-held'),(241,'agent-blocked')]:
            with self.subTest(n=n,name=name):
                self.s = snapshot(self.p)
                self.target(n)['labels'].append({'name':name})
                with self.assertRaisesRegex(ValueError,'parent|authorization'):
                    self.choose()
        self.s = snapshot(self.p); self.target(241)['labels'] = []
        with self.assertRaisesRegex(ValueError,'authorization'):
            self.choose()

    def test_bare_closed_cancelled_and_missing_review_do_not_complete(self):
        for change in ('receipt','cancelled','review','ci'):
            with self.subTest(change=change):
                self.s = snapshot(self.p)
                if change == 'receipt': self.s['receipts'].pop('26')
                elif change == 'cancelled': self.target(26)['state_reason'] = 'not_planned'
                elif change == 'review': self.s['receipts']['26']['review_verdict'] = 'error'
                else: self.s['receipts']['26']['ci_conclusion'] = 'pending'
                with self.assertRaisesRegex(ValueError,'unverified program'):
                    self.choose()

    def test_successful_open_current_can_satisfy_dependency_but_failed_cannot(self):
        self.s['current'] = 299
        self.complete(241)
        self.target(299)['labels'] = [{'name':'agent-ready'}]
        self.s['receipts']['299'] = receipt()
        self.assertEqual(self.choose()['successor'],300)
        self.s['current_success'] = False
        self.assertIsNone(self.choose()['successor'])

    def test_independent_held_sibling_does_not_stop_next_eligible(self):
        self.complete(299); self.complete(300)
        self.target(301)['labels'] = [{'name':'agent-blocked'}]
        self.assertEqual(self.choose()['successor'],302)

    def test_withdrawn_ready_requires_existing_resume_grant(self):
        self.s['events']['299'] = [{'event':'labeled','label':{'name':'agent-ready'}},
                                   {'event':'unlabeled','label':{'name':'agent-ready'}}]
        self.assertIsNone(self.choose()['successor'])
        self.target(299)['labels'] = [{'name':'agent-resume-authorized'}]
        self.assertEqual(self.choose()['successor'],299)
        self.target(299)['labels'].append({'name':'agent-held'})
        self.assertIsNone(self.choose()['successor'])

    def test_capacity_includes_other_programs_and_counts_unique_claims(self):
        self.s['open_claims'] += [issue(n,names=['agent-ready','agent-running']) for n in (20,21,22)]
        self.assertEqual(self.choose(),{'successor':None,'reason':'capacity'})
        self.s['open_claims'].pop()
        self.assertEqual(self.choose()['successor'],299)

    def test_retry_reserved_successor_never_selects_another(self):
        self.s['reserved_successor'] = 299
        self.target(299)['labels'] = [{'name':'agent-ready'}]
        self.assertEqual(self.choose(),{'successor':299,'already_owned':True})
        self.complete(299)
        self.assertEqual(self.choose(),{'successor':299,'already_owned':True})
        self.target(299).update(state='open',labels=[{'name':'agent-held'}])
        with self.assertRaisesRegex(ValueError,'reconciliation'):
            self.choose()

    def test_missing_snapshot_or_event_pages_fail(self):
        self.s['events'].pop('299')
        with self.assertRaisesRegex(ValueError,'events'):
            self.choose()
        self.s = snapshot(self.p); self.s['issues'].pop()
        with self.assertRaisesRegex(ValueError,'incomplete'):
            self.choose()

    def test_gate_waits_for_every_prerequisite(self):
        for n in range(299,315): self.complete(n)
        self.target(315)['labels'] = [{'name':'agent-ready'}]
        self.assertIsNone(self.choose()['successor'])
        self.complete(315)
        self.assertEqual(self.choose()['successor'],316)

    def test_gate_requires_all_receipts_not_only_last_child(self):
        for n in range(299,316): self.complete(n)
        self.s['receipts'].pop('301')
        self.assertIsNone(self.choose()['successor'])

    def test_malformed_current_outcome_cannot_satisfy_dependency(self):
        self.s['current_success'] = 'yes'
        with self.assertRaisesRegex(ValueError,'outcome'):
            self.choose()

    def test_reserved_withdrawal_requires_reconciliation_even_if_running(self):
        for remaining in ([], ['agent-running']):
            with self.subTest(remaining=remaining):
                self.s = snapshot(self.p)
                self.s['reserved_successor'] = 299
                self.target(299)['labels'] = [{'name': n} for n in remaining]
                self.s['events']['299'] = [
                    {'event':'labeled','label':{'name':'agent-ready'}},
                    {'event':'unlabeled','label':{'name':'agent-ready'}}]
                with self.assertRaisesRegex(ValueError,'reconciliation'):
                    self.choose()
