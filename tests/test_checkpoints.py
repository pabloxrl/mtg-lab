"""Verifier requirements: exact typed fields, strict scripts, first divergence."""
import copy
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from scripts import checkpoints as c, scenario as s

ROOT = Path(__file__).resolve().parents[1]


def sign(f):
    f['provenance']['reproduction']['trace_sha256'] = s.digest(f['script'])
    f['provenance'].pop('fixture_revision', None)
    f['provenance']['fixture_revision'] = s.digest(f)
    return f


class ComparisonTests(unittest.TestCase):
    def setUp(self):
        self.f = s.load(ROOT/'fixtures/scenarios/priority-pass.json')
        self.f['invalid_actions'] = []
        self.f['checkpoints'][0]['assertions'] += [
            {'field':'stack','path':'/stack','expected':[{'targets':['bear-1']}],'basis':'pass-rule'},
            {'field':'player_visible_information','path':'/view','expected':{'hand_count':1},'basis':'boundary-spec'},
        ]
        sign(self.f)
        state = copy.deepcopy(self.f['setup']['state'])
        self.a = {'checkpoint_version':1,'fixture_id':self.f['fixture_id'],
                  'fixture_revision':self.f['provenance']['fixture_revision'],
                  'initial':state, 'consumed_script':copy.deepcopy(self.f['script']),
                  'checkpoints':[{'name':'priority-p1','after':'pass-0','kind':'decision',
                                  'state':{'priority':1,'stack':[{'targets':['bear-1']}], 'view':{'hand_count':1}}}],
                  'invalid_results':[]}

    def test_equal_is_only_comparison(self):
        self.assertEqual(c.compare(self.f,self.a)['status'],'pass')

    def mismatch(self, edit, checkpoint, path):
        edit(self.a)
        r = c.compare(self.f,self.a)
        self.assertEqual(r['status'],'mismatch')
        self.assertEqual(r['checkpoint'],checkpoint)
        self.assertEqual(r['path'],path)
        return r

    def test_wrong_starting_life(self):
        self.mismatch(lambda a:a['initial']['players'][0].update(life=19),'initial','/players/0/life')

    def test_wrong_target(self):
        self.mismatch(lambda a:a['checkpoints'][0]['state']['stack'][0].update(targets=['bear-0']),
                      'priority-p1','/stack/0/targets/0')

    def test_wrong_choice(self):
        self.mismatch(lambda a:a['consumed_script'][0]['choices'][0].update(actor=1),
                      'script','/0/choices/0/actor')

    def test_checkpoint_priority(self):
        self.mismatch(lambda a:a['checkpoints'][0]['state'].update(priority=0),'priority-p1','/priority')

    def test_first_divergence(self):
        self.a['checkpoints'][0]['state']['priority']=0
        self.mismatch(lambda a:a['initial']['players'][0].update(life=19),'initial','/players/0/life')

    def test_boolean_is_not_number(self):
        self.mismatch(lambda a:a['checkpoints'][0]['state']['view'].update(hand_count=True),
                      'priority-p1','/view/hand_count')

    def test_private_extra_fields_fail(self):
        self.mismatch(lambda a:a['checkpoints'][0]['state']['view'].update(opponent_hand=['giant-growth']),
                      'priority-p1','/view/opponent_hand')

    def test_missing_is_not_null(self):
        self.f['checkpoints'][0]['assertions'][0]['expected']=None
        sign(self.f);self.a['fixture_revision']=self.f['provenance']['fixture_revision']
        r=self.mismatch(lambda a:a['checkpoints'][0]['state'].pop('priority'),'priority-p1','/priority')
        self.assertFalse(r['actual_present'])
        self.assertTrue(r['expected_present'])

    def test_envelope_and_order_rejected(self):
        for edit in [lambda a:a.update(checkpoint_version=True), lambda a:a.update(extra=1),
                     lambda a:a.update(fixture_revision='0'*64),lambda a:a.update(checkpoints=[]),
                     lambda a:a['checkpoints'][0].update(after='other'),
                     lambda a:a.update(invalid_results=[{}])]:
            with self.subTest(edit=edit):
                a=copy.deepcopy(self.a);edit(a)
                with self.assertRaises(ValueError):c.compare(self.f,a)

    def test_unused_choice_fails(self):
        self.mismatch(lambda a:a['consumed_script'].append(copy.deepcopy(a['consumed_script'][0])),
                      'script','/1')

    def test_all_declared_fields_are_checked(self):
        # Each assertion uses a literal independent sentinel. No engine produces expectations.
        fields={'zones':{},'object_identity':{},'life':20,'marked_damage':2,
                'power_toughness':{'power':5,'toughness':5},'stack':[], 'priority':1,
                'legal_choices':[], 'player_visible_information':{},'mana':{},
                'active_player':0,'status':{},'outcome':{},'rewards':{},'invariant':False}
        for field,value in fields.items():
            with self.subTest(field=field):
                f=copy.deepcopy(self.f);a=copy.deepcopy(self.a)
                f['checkpoints'][0]['assertions']=[dict(field=field,path='/sentinel',expected=value,basis='pass-rule')]
                sign(f);a['fixture_revision']=f['provenance']['fixture_revision']
                a['checkpoints'][0]['state']={'sentinel':value}
                self.assertEqual(c.compare(f,a)['status'],'pass')
                a['checkpoints'][0]['state']['sentinel']='wrong'
                self.assertEqual(c.compare(f,a)['status'],'mismatch')

    def test_cli_artifacts(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d);(p/'fixture.json').write_text(json.dumps(self.f))
            self.a['initial']['players'][0]['life']=19
            (p/'actual.json').write_text(json.dumps(self.a))
            env={k:v for k,v in os.environ.items() if k not in ('DISPLAY','WAYLAND_DISPLAY')}
            r=subprocess.run(['python3','scripts/checkpoints.py',str(p/'fixture.json'),str(p/'actual.json'),
                              '--artifacts',str(p/'failure')],cwd=ROOT,env=env,stdin=subprocess.DEVNULL,
                             capture_output=True,text=True,timeout=10)
            self.assertEqual(r.returncode,1,r.stderr)
            report=json.loads(r.stdout)
            self.assertEqual(report['path'],'/players/0/life')
            self.assertEqual(json.loads((p/'failure/fixture.json').read_text()),self.f)
            self.assertEqual(json.loads((p/'failure/actual.json').read_text()),self.a)
            self.assertEqual(json.loads((p/'failure/diff.json').read_text())['status'],'mismatch')

    def test_initial_extensions_are_compared_at_named_assertions(self):
        self.a['initial']['views']={'0':{'private':False}}
        self.assertEqual(c.compare(self.f,self.a)['status'],'pass')

    def test_invalid_action_checks_all_four_invariants(self):
        inv=s.load(ROOT/'fixtures/scenarios/priority-pass.json')['invalid_actions'][0]
        self.f['invalid_actions']=[inv];sign(self.f)
        self.a['fixture_revision']=self.f['provenance']['fixture_revision']
        before=dict(state=copy.deepcopy(self.f['setup']['state']),
                    rng={'algorithm':'test-rng-v1','state_hex':'0123'},
                    decision={'id':'d1','actor':0,'kind':'priority','candidates':['pass','concede']},
                    private_information={'views':[{'seat':0,'observation':{'own_hand':[],'opponent_hand_count':0,'library_counts':[0,0]}},
                                                  {'seat':1,'observation':{'own_hand':[],'opponent_hand_count':0,'library_counts':[0,0]}}]})
        probe=dict(action=copy.deepcopy(inv['action']),at='initial',error=inv['error'],before=before,after=copy.deepcopy(before))
        self.a['invalid_results']=[probe]
        self.assertEqual(c.compare(self.f,self.a)['status'],'pass')
        for field in before:
            with self.subTest(field=field):
                probe['after']=copy.deepcopy(before)
                if field=='state':probe['after'][field]['players'][0]['life']=19
                elif field=='rng':probe['after'][field]['state_hex']='4567'
                elif field=='decision':probe['after'][field]['id']='d2'
                else:probe['after'][field]['views'][0]['observation']['own_hand']=['forest']
                self.assertEqual(c.compare(self.f,self.a)['status'],'mismatch')
        for field in before:
            for unavailable in (None, {}, [], 'unavailable'):
                with self.subTest(field=field, unavailable=unavailable):
                    probe['before']=copy.deepcopy(before)
                    probe['before'][field]=unavailable
                    probe['after']=copy.deepcopy(probe['before'])
                    with self.assertRaises(ValueError):c.compare(self.f,self.a)
