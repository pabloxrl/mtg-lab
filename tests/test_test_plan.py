"""Correction GH-52: a design must be complete without certifying execution."""
import copy
import json
from pathlib import Path
import unittest
from scripts import test_plan

ROOT = Path(__file__).resolve().parents[1]


class TestPlanTests(unittest.TestCase):
    def setUp(self):
        self.plan = json.loads((ROOT / test_plan.PLAN).read_text())
        self.registry = json.loads((ROOT / 'data/capabilities-v1.json').read_text())
        self.program = json.loads((ROOT / 'doc/programs/rfc-0002.json').read_text())

    def check(self):
        return test_plan.validate(self.plan, self.registry, self.program)

    def reject(self, edit, reason):
        edit(self.plan)
        with self.assertRaisesRegex(ValueError, reason):
            self.check()

    def test_complete_design_is_not_execution_evidence(self):
        report = self.check()
        self.assertEqual(report['designed_slots'], 4 * len(self.registry['capabilities']))
        self.assertEqual(report['execution_evidence_added'], 0)
        # Even with complete design, existing coverage gate must still reject zero executions.
        unexecuted = copy.deepcopy(self.registry)
        for capability in unexecuted['capabilities']:
            capability['implementation'] = 'planned'
            capability['required_evidence'] = {category: [] for category in test_plan.CATEGORIES}
        with self.assertRaisesRegex(ValueError, 'missing executed/passed coverage'):
            test_plan.scenario.validate_registry(unexecuted, require_passed=True)

    def test_missing_and_duplicate_slot_are_rejected(self):
        self.reject(lambda p: p['cases'].pop(), 'missing capability/category')
        self.setUp()
        duplicate = copy.deepcopy(self.plan['cases'][0])
        duplicate['id'] += '-duplicate'
        self.reject(lambda p: p['cases'].append(duplicate), 'duplicate capability/category')

    def test_duplicate_id_is_rejected(self):
        self.reject(lambda p: p['cases'].append(copy.deepcopy(p['cases'][0])), 'duplicate planned test ID')

    def test_unresolvable_capability_owner_and_stage(self):
        for field, value, reason in [('capability_id', 'invented', 'unresolved capability'),
                                     ('owner_issue', 999999, 'unresolved implementation owner'),
                                     ('owner_issue', 54, 'must implement behavior'),
                                     ('owner_issue', True, 'unresolved implementation owner'),
                                     ('milestone', 'M9', 'owner/stage mismatch')]:
            with self.subTest(field=field, value=value):
                self.setUp()
                self.reject(lambda p: p['cases'][0].__setitem__(field, value), reason)

    def test_empty_design_and_fabricated_pass_are_rejected(self):
        for field in ('setup', 'actions', 'expected', 'basis'):
            with self.subTest(field=field):
                self.setUp()
                self.reject(lambda p: p['cases'][0].__setitem__(field, ' '), 'missing case')
        self.setUp()
        self.reject(lambda p: p.__setitem__('status', 'passed'), 'must not claim execution')
        self.setUp()
        self.reject(lambda p: p['cases'][0].__setitem__('passed', True), 'unexpected or missing')

    def test_stale_source_and_authored_fixture_links(self):
        self.reject(lambda p: p['rules'].__setitem__('sha256', '0' * 64), 'stale rules/card')
        self.setUp()
        self.reject(lambda p: p['authored_fixtures'][0].__setitem__('fixture_revision', '0' * 64), 'stale authored fixture')
        self.setUp()
        self.reject(lambda p: p['authored_fixtures'][0].__setitem__('path', 'fixtures/scenarios/missing.json'), 'unresolved authored fixture')
        self.setUp()
        self.reject(lambda p: p['authored_fixtures'].pop(), 'missing admitted')
        self.setUp()
        self.reject(lambda p: p['authored_fixtures'].append(copy.deepcopy(p['authored_fixtures'][0])), 'duplicate authored fixture')


if __name__ == '__main__':
    unittest.main()
