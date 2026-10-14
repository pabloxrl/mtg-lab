"""Real normal-reset combat client, rules ledgers, and comparator controls."""
from pathlib import Path
import copy
import json
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
import played_combat_reference as combat

class CombatTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.folder=tempfile.TemporaryDirectory()
        cls.actual=combat.native(Path(cls.folder.name))
        cls.doc=json.loads(combat.FIXTURE.read_text())
    @classmethod
    def tearDownClass(cls):cls.folder.cleanup()
    def test_played_client_reaches_combat_damage(self):
        combat.check_run(self.actual,self.doc,'native')
        self.assertTrue(self.actual['checkpoints'])
        for points in self.actual['checkpoints'].values():
            self.assertTrue(points)
            self.assertEqual(points[-1]['step'],'combat_damage')
    def test_early_life_mutation(self):
        bad=copy.deepcopy(self.actual)
        bad['runs']['shivan-unblocked']['points'][0]['life'][0]=19
        bad['checkpoints']['shivan-unblocked'][0]['life'][0]=19
        with self.assertRaisesRegex(ValueError,'life'):
            combat.check_run(bad,self.doc,'native')
    def test_fixture_authoring_is_independent_and_reproducible(self):
        sys.path.insert(0,str(combat.ROOT/'fixtures/reference'))
        import author_combat
        self.assertEqual(author_combat.fixtures(),self.doc)
        self.assertEqual(author_combat.negatives(self.doc),combat.negative_inputs(self.doc))
    def test_comparator_controls(self):
        controls=combat.comparator_controls(self.actual,self.doc,'native')
        self.assertEqual(set(controls), {
            'early-life', 'final-life', 'simultaneous-damage', 'missing-simultaneous',
            'blocked-status', 'allocation', 'graveyard-order', 'zone-identity',
            'incarnation', 'library-identity', 'missing-consumption', 'legal-observation',
            'missing-stack', 'missing-permanents', 'missing-life', 'missing-combat',
            'missing-allocations'})
        # Selective copies must leave all shared views usable by the next case.
        combat.check_run(self.actual,self.doc,'native')
    def test_simultaneous_marks_and_source_incarnations(self):
        for run in self.actual['runs'].values():self.assertEqual(len(run['marked']),1)
    def test_ordered_graveyard_comparator_retains_real_disagreement(self):
        # Diagnostic regression, not an acceptance waiver: the real full-family
        # runner must still fail on this unexplained/uncontrolled ordered state.
        saved=json.loads((combat.ROOT/'doc/evidence/combat-prefix/ordered-graveyard-checkpoints.json').read_text())
        difference=combat.base.instant_reference.difference(combat.semantic(saved['native']),combat.semantic(saved['xmage']))
        self.assertEqual(difference,dict(path='$.graveyard[1][0]',expected='1/magnigoth-sentry/0',actual='1/thornweald-archer/0'))
    def test_opposite_orders_are_real_distinct_engine_states(self):
        for base,owner in [('shivan-multiple',1),('pathwarden-multiple',0),('both-owners',0)]:
            left=self.actual['checkpoints'][base][-1]['graveyard'][owner]
            right=self.actual['checkpoints'][base+'-reverse'][-1]['graveyard'][owner]
            self.assertEqual(left,list(reversed(right)))
            self.assertEqual(len(left),2)
            self.assertNotEqual(left,right)
    def test_swapped_arrival_occurrences_are_witnessed(self):
        p=self.actual['checkpoints']['shivan-multiple-identities'][-1]
        self.assertEqual(p['graveyard'][1],['1/magnigoth-sentry/1','1/thornweald-archer/1'])
        self.assertEqual(p['incarnations']['1/magnigoth-sentry/1'],4)
        self.assertEqual(p['incarnations']['1/magnigoth-sentry/0'],0)
    def test_multi_owner_choices_precede_every_death(self):
        c=next(c for c in self.doc['cases'] if c['id']=='both-owners')
        events=[e for e in c['play'] if e['kind']=='graveyard_order']
        self.assertEqual([e['actor'] for e in events],[0,1])
        for e in events:
            p=self.actual['checkpoints']['both-owners'][e['sequence']]
            self.assertEqual(p['graveyard'],[[],[]])
            for source in ['0/axgard-cavalry/0','0/swab-goblin/0','1/bear-cub/0','1/thornweald-archer/0']:
                self.assertEqual(p['permanents'][source]['damage'],2)
    def test_ordered_graveyard_mutation_is_rejected(self):
        bad=copy.deepcopy(self.actual)
        bad['runs']['shivan-multiple']['points'][-1]['graveyard'][1].reverse()
        bad['checkpoints']={k:v['points'] for k,v in bad['runs'].items()}
        with self.assertRaisesRegex(ValueError,'ordered graveyard'):
            combat.check_run(bad,self.doc,'native')
    def test_negative_controls_have_exact_consumption_boundaries(self):
        self.assertGreaterEqual(len(self.actual['rejections']),20)
        combat.check_rejections(self.actual,'native')

if __name__=='__main__':unittest.main()
