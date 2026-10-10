"""Real adapter/client tests, CR514/104/121 independent ledgers."""
import copy
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
import played_cleanup_reference as reference

class CleanupTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.folder=tempfile.TemporaryDirectory();cls.addClassCleanup(cls.folder.cleanup)
        cls.actual=reference.native(Path(cls.folder.name))
    def test_real_both_starter_rules_endings(self):
        reference.check_run(self.actual)
        self.assertEqual(len(self.actual['runs']),5)
        self.assertGreater(sum(map(len,self.actual['checkpoints'].values())),2000)
    def test_empty_observations_rejected(self):
        bad=copy.deepcopy(self.actual['checkpoints']);bad['empty-library-0']=[]
        with self.assertRaises(ValueError):reference.compare(bad)
    def test_wrong_terminal_outcome_rejected(self):
        bad=copy.deepcopy(self.actual['checkpoints']);bad['empty-library-0'][-1]['outcome']['winner']=1
        with self.assertRaises(ValueError):reference.compare(bad)
    def test_omitted_final_checkpoint_rejected(self):
        bad=copy.deepcopy(self.actual);bad['runs']['empty-library-0']['points'].pop()
        with self.assertRaises(ValueError):reference.check_run(bad)
    def test_native_real_invalid_tapes(self):
        reference.check_rejections(self.actual,'native')
        self.assertEqual(len(self.actual['rejections']),14)
        self.assertEqual(self.actual['rejection_state_rng'],'unchanged')
    def test_negative_cannot_fail_late(self):
        bad=copy.deepcopy(self.actual)
        bad['rejections']['wrong_incarnation']='first divergence: /omitted final checkpoint'
        with self.assertRaises(ValueError):reference.check_rejections(bad,'native')
    def test_missing_cleanup_observation_fails(self):
        bad=copy.deepcopy(self.actual);bad['runs']['stacked-expiry']['cleanup_checkpoints'].pop()
        with self.assertRaises(ValueError):reference.check_run(bad)
    def test_atomic_expiry_and_damage_mutations_fail(self):
        for field,value in [('damage',2),('power',9),('toughness',7)]:
            bad=copy.deepcopy(self.actual)
            bad['runs']['stacked-expiry']['cleanup_checkpoints'][-1]['permanents']['token/0/0/0'][field]=value
            with self.subTest(field=field),self.assertRaises(ValueError):reference.check_run(bad)
    def test_terminal_reward_mapping_is_outcome_derived(self):
        bad=copy.deepcopy(self.actual);bad['runs']['empty-library-0']['policy_capture']['footer']['returns']=[-1,1]
        with self.assertRaises(ValueError):reference.check_run(bad)
    def test_unused_consumed_suffix_fails(self):
        bad=copy.deepcopy(self.actual);bad['runs']['empty-library-0']['consumed_play'].append({'kind':'pass'})
        with self.assertRaises(ValueError):reference.check_run(bad)
    def test_premature_terminal_fails(self):
        bad=copy.deepcopy(self.actual);r=bad['runs']['empty-library-0'];r['points'][0]['outcome']=r['points'][-1]['outcome'];bad['checkpoints']['empty-library-0']=r['points']
        with self.assertRaises(ValueError):reference.check_run(bad)
    def test_intermediate_lethal_mutation_fails(self):
        bad=copy.deepcopy(self.actual);del bad['runs']['stacked-expiry']['cleanup_checkpoints'][-1]['permanents']['token/0/0/0']
        with self.assertRaises(ValueError):reference.check_run(bad)
    def test_boolean_is_not_winner_seat(self):
        bad=copy.deepcopy(self.actual['checkpoints']);bad['empty-library-1'][-1]['outcome']['winner']=True
        with self.assertRaises(ValueError):reference.compare(bad)
    def test_missing_additional_cleanup_fails(self):
        import json
        specs=json.loads((reference.ROOT/'fixtures/reference/cleanup.json').read_text())['cases']
        observations={c['id']:[dict(ordinal=i,turn=1,step='CLEANUP') for i in range(1,3 if c['trigger'] else 2)] for c in specs}
        reference.check_additional_cleanup(observations)
        observations['rules-continuous-hand-size-cleanup-interaction'].pop()
        with self.assertRaisesRegex(ValueError,'missing additional cleanup'):reference.check_additional_cleanup(observations)
    def test_concession_cannot_be_natural_empty_draw(self):
        bad=copy.deepcopy(self.actual['checkpoints']);bad['concession-only'][-1]['outcome']['losses'][0]='EmptyDraw'
        with self.assertRaises(ValueError):reference.compare(bad)
