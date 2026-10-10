"""Real played spell consumer plus independent positive/negative controls."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
import spells_reference as reference


class SpellTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.folder=tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.folder.cleanup)
        cls.doc=json.loads(reference.FIXTURE.read_text())
        cls.actual=reference.native(Path(cls.folder.name))

    def test_real_normal_reset_played_prefixes(self):
        reference.check_run(self.actual,self.doc,'native')
        self.assertEqual(len(self.actual['checkpoints']),6)
        self.assertGreater(sum(map(len,self.actual['checkpoints'].values())),500)

    def test_literal_final_ledger(self):
        reference.compare(self.actual['checkpoints'])
        points=self.actual['checkpoints']
        self.assertEqual(points['growth-bite'][-1]['permanents']['1/bear-cub/0']['power'],5)
        self.assertEqual(points['departed-target'][-1]['permanents']['1/bear-cub/0']['power'],2)

    def test_rejections_cannot_be_later_exhaustion(self):
        self.assertEqual(len(self.actual['rejections']),18)
        self.assertEqual(self.actual['rejection_state_rng'],'unchanged')
        for name in self.actual['rejections']:
            if name=='truncated_tape':continue
            bad=dict(self.actual['rejections']);bad[name]='first divergence: /missing choice before named stop'
            with self.subTest(control=name),self.assertRaises(ValueError):reference.check_rejections(bad,'native')

    def test_intermediate_target_role_and_mode_mutations_fail(self):
        for case,field,value in [('growth-bite','targets',[]),('surprise-boost','mode',1)]:
            bad=copy.deepcopy(self.actual)
            for p in bad['runs'][case]['points']:
                if p['stack'] and p['stack'][-1]['source'] in ('1/bite-down/0','0/goblin-surprise/0'):
                    p['stack'][-1][field]=value
            bad['checkpoints'][case]=bad['runs'][case]['points']
            with self.subTest(field=field),self.assertRaises(ValueError):reference.check_run(bad,self.doc,'native')

    def test_missing_extra_and_empty_observations_fail(self):
        for mode in ('missing','extra','empty'):
            bad=copy.deepcopy(self.actual['checkpoints'])
            if mode=='missing':bad['fodder'][0].pop('effects')
            elif mode=='extra':bad['fodder'][0]['invented']=[]
            else:bad['fodder']=[]
            with self.subTest(mode=mode),self.assertRaises(ValueError):reference.compare(bad)

    def test_real_observation_comparator_mutations(self):
        controls=reference.comparator_controls(self.actual['checkpoints'])
        self.assertEqual(len(controls),14)
        for name in ('stack-order','target-order','target-incarnation','token-id','token-order','draw-order'):
            self.assertTrue(controls[name]['path'])

    def test_token_birth_carries_frozen_definition_identity(self):
        manifest=json.loads((reference.ROOT/'data/cards/foundations_micro_v1.json').read_text())
        token=next(c for c in manifest['cards'] if c['id']=='goblin-token')
        for birth in self.actual['checkpoints']['fodder'][-1]['creations']:
            self.assertEqual(birth.get('definition_hash'),token['content_sha256'])
            self.assertEqual(birth.get('card'),'goblin-token')

    def test_wrong_category_at_correct_sequence_is_rejected(self):
        specs=reference.negative_specs('native')
        for name,spec in specs.items():
            if name in ('truncated_tape','extra_tape'):continue
            bad=dict(self.actual['rejections'])
            bad[name]=f"first divergence: /play/{spec['sequence']} /unrelated failure"
            with self.subTest(control=name),self.assertRaises(ValueError):reference.check_rejections(bad,'native')

    def test_duplicate_creation_identity_is_rejected(self):
        for field in ('id','birth'):
            bad=copy.deepcopy(self.actual)
            births=bad['runs']['fodder']['points'][-1]['creations']
            births[1][field]=births[0][field]
            bad['checkpoints']['fodder']=bad['runs']['fodder']['points']
            with self.subTest(field=field),self.assertRaises(ValueError):reference.check_run(bad,self.doc,'native')

    def test_wrong_token_identity_or_effect_cannot_pass_oracle(self):
        for case,key,value in [('fodder','power',2),('growth-bite','toughness',2)]:
            bad=copy.deepcopy(self.actual['checkpoints'])
            identity='token/0/0/0' if case=='fodder' else '1/bear-cub/0'
            bad[case][-1]['permanents'][identity][key]=value
            with self.subTest(case=case),self.assertRaises(ValueError):reference.compare(bad)


if __name__=='__main__':unittest.main()
