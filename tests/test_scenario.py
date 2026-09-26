"""Independent GH-12 rejection criteria from RFC 0002 §3/§7 and GH-11 policy."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from scripts import scenario as s

ROOT = Path(__file__).resolve().parents[1]

class ScenarioTests(unittest.TestCase):
    def setUp(self):
        self.fixture = s.load(ROOT / 'fixtures/scenarios/priority-pass.json')
        self.registry = s.load(ROOT / 'data/capabilities-v1.json')

    def reject(self, edit):
        f = copy.deepcopy(self.fixture)
        edit(f)
        # Re-sign modified input so semantic rejection is tested independently of integrity.
        if 'provenance' in f:
            f['provenance']['reproduction']['trace_sha256'] = s.digest(f['script'])
            raw = copy.deepcopy(f)
            del raw['provenance']['fixture_revision']
            f['provenance']['fixture_revision'] = s.digest(raw)
        with self.assertRaises(ValueError):
            s.validate(f, self.registry)

    def test_valid_schema_example(self):
        s.validate(self.fixture, self.registry)

    def test_unknown_versions_and_pins(self):
        for field in ['scenario_version', 'rules', 'cards']:
            with self.subTest(field=field):
                self.reject(lambda f: f.__setitem__(field, 99 if field == 'scenario_version' else {'version':'unresolved','sha256':'0'*64}))
        self.reject(lambda f: f.__setitem__('scenario_version', True))

    def test_strict_fields_and_types(self):
        for field in ['description','setup','provenance','checkpoints','script']:
            with self.subTest(field=field):
                self.reject(lambda f: f.pop(field) if field != 'script' else f.__setitem__('script', []))
        self.reject(lambda f: f['setup']['state'].__setitem__('unknown',1))
        self.reject(lambda f: f['setup']['state']['players'][0].__setitem__('life',True))
        self.reject(lambda f: f['setup']['state']['players'][0]['mana'].__setitem__('G',-1))

    def test_capability_and_checkpoint_references(self):
        self.reject(lambda f: f.__setitem__('required_capabilities',['planeswalkers']))
        self.reject(lambda f: f['checkpoints'][0].__setitem__('after','missing-action'))
        self.reject(lambda f: f['checkpoints'][0]['assertions'][0].__setitem__('basis','missing-basis'))
        self.reject(lambda f: f['checkpoints'].append(copy.deepcopy(f['checkpoints'][0])))
        self.reject(lambda f: f['invalid_actions'][0].__setitem__('invariants',['state_unchanged']))

    def test_synthetic_assumptions_and_zones(self):
        self.reject(lambda f: f['setup'].__setitem__('assumptions',[]))
        self.reject(lambda f: f['setup']['assumptions'][0]['checks'][0].__setitem__('expected',['phantom']))
        self.reject(lambda f: f['setup']['state']['players'][0]['zones']['library'].append('unknown-object'))
        self.reject(lambda f: f['setup']['state'].__setitem__('players',[f['setup']['state']['players'][0]]*2))
        self.reject(lambda f: f['setup']['state'].__setitem__('step','combat_damage'))

    def test_incomplete_choice_scripts(self):
        self.reject(lambda f: f['script'][0].__setitem__('choices',[]))
        self.reject(lambda f: f['script'][0].__setitem__('kind','cast'))
        self.reject(lambda f: f['script'][0]['choices'][0].__setitem__('actor',1))
        self.reject(lambda f: f['script'].append(copy.deepcopy(f['script'][0])))
        self.reject(lambda f: f['script'][0]['choices'][0].__setitem__('values',['auto']))

    def test_runtime_choice_completeness(self):
        choices = s.ChoiceScript(self.fixture['script'])
        with self.assertRaises(ValueError): choices.finish()
        with self.assertRaises(ValueError): choices.consume(1,'pass')
        with self.assertRaises(ValueError): choices.consume(0,'targets')
        self.assertEqual(choices.consume(0,'pass'), [])
        choices.finish()
        with self.assertRaises(ValueError): choices.consume(0,'pass')

    def test_provenance_independence_and_integrity(self):
        self.reject(lambda f: f['provenance']['expected_result_basis'][0].__setitem__('method','engine-output'))
        self.reject(lambda f: f['provenance'].__setitem__('source_kind','adapted'))
        self.reject(lambda f: f['provenance'].__setitem__('source_kind','regression'))
        self.reject(lambda f: f['provenance'].__setitem__('source_kind','differential'))
        self.reject(lambda f: f['provenance']['rules_basis']['rules'].__setitem__('sha256','0'*64))
        self.fixture['description'] = 'tampered'
        with self.assertRaises(ValueError): s.validate(self.fixture,self.registry)

    def test_registry_full_rfc_coverage(self):
        s.validate_registry(self.registry)
        for field in ['families','supported_requirements','capabilities']:
            r = copy.deepcopy(self.registry)
            r[field].pop()
            with self.subTest(field=field), self.assertRaises(ValueError): s.validate_registry(r)
        r = copy.deepcopy(self.registry)
        r['families'][0]['requirements'].pop()
        with self.assertRaises(ValueError): s.validate_registry(r)
        r = copy.deepcopy(self.registry)
        del r['capabilities'][0]['required_evidence']['negative']
        with self.assertRaises(ValueError): s.validate_registry(r)

    def test_planned_is_not_passed(self):
        with self.assertRaises(ValueError): s.validate_registry(self.registry,require_passed=True)
        r = copy.deepcopy(self.registry)
        r['capabilities'][0]['implementation']='supported'
        with self.assertRaises(ValueError): s.validate_registry(r)
        r = copy.deepcopy(self.registry)
        r['capabilities'][0]['required_evidence']['positive']=[{'status':'passed'}]
        with self.assertRaises(ValueError): s.validate_registry(r)

    def test_duplicate_keys_and_nonfinite(self):
        with tempfile.TemporaryDirectory() as directory:
            p = Path(directory)/'input.json'
            for value in ['{"scenario_version":1,"scenario_version":1}', '{"value":NaN}', '{"value":Infinity}']:
                p.write_text(value)
                with self.subTest(value=value), self.assertRaises(ValueError): s.load(p)


class ExtendedScenarioTests(unittest.TestCase):
    """Exercise valid alternative constructors, evidence states and CLI boundaries."""
    setUp = ScenarioTests.setUp
    reject = ScenarioTests.reject

    def signed(self, f):
        p = f['provenance']
        p['reproduction']['trace_sha256'] = s.digest(f['script'])
        p['fixture_revision'] = '0' * 64
        raw = copy.deepcopy(f)
        del raw['provenance']['fixture_revision']
        p['fixture_revision'] = s.digest(raw)
        return f

    def reset_fixture(self):
        f = copy.deepcopy(self.fixture)
        cards = s.load(ROOT / 'data/cards/foundations_micro_v1.json')
        orders = [[entry['card_id'] for entry in deck['cards'] for _ in range(entry['copies'])] for deck in cards['decks']]
        f['setup'] = {'kind':'normal_reset','starting_seat':0,'decks':[d['id'] for d in cards['decks']],
                      'ordered_libraries':orders,'shuffle_results':[]}
        f['script'] = [{'id':f'keep-{p}','actor':p,'kind':'keep','source':None,
                        'choices':[{'id':f'keep-choice-{p}','actor':p,'kind':'keep','values':[]}]} for p in (0,1)]
        f['checkpoints'][0]['after']='keep-1'
        f['provenance']['reproduction']['setup_kind']='normal_reset'
        f['provenance']['rules_basis']['oracle_sources']=[{'card_id':c['id'],'sha256':c['oracle_text_sha256']} for c in cards['cards'] if c['kind']=='card']
        f['required_capabilities'] += [c['behavior_id'] for c in cards['cards'] if c['kind']=='card']
        return self.signed(f)

    def test_normal_reset_is_separate_and_checks_decks(self):
        f = self.reset_fixture()
        s.validate(f,self.registry)
        for edit in [lambda f:f['setup']['ordered_libraries'][0].pop(),
                     lambda f:f['setup']['ordered_libraries'][0].__setitem__(0,'bear-cub'),
                     lambda f:f['script'].pop(0),
                     lambda f:f['setup'].__setitem__('state',{})]:
            bad=copy.deepcopy(f);edit(bad)
            with self.assertRaises(ValueError): s.validate(self.signed(bad),self.registry)

    def test_assertion_types_and_pointer_syntax(self):
        self.reject(lambda f:f['checkpoints'][0]['assertions'][0].__setitem__('expected','next-player'))
        self.reject(lambda f:f['checkpoints'][0]['assertions'][0].__setitem__('path','/priority~5'))

    def test_evidence_requires_exact_reviewed_fixture(self):
        f=copy.deepcopy(self.fixture)
        f['provenance']['review']['status']='accepted-for-m0'
        f['provenance']['vintage_audit']['status']='accepted'
        self.signed(f)
        r=copy.deepcopy(self.registry)
        cap=next(c for c in r['capabilities'] if c['id']=='priority/passing')
        record={'fixture_id':f['fixture_id'],'fixture_revision':f['provenance']['fixture_revision'],
                'status':'planned','engine':'native','engine_revision':None,'artifact_sha256':None,'artifact_url':None}
        cap['required_evidence']['positive']=[record]
        report=s.validate_registry(r,[f])
        self.assertEqual((report['planned'],report['passed']),(1,0))
        record['status']='passed'
        with self.assertRaises(ValueError):s.validate_registry(r,[f])
        record.update(engine_revision='a'*40,artifact_sha256='b'*64,artifact_url='https://example.org/test-result')
        report=s.validate_registry(r,[f])
        self.assertEqual(report['passed'],1)
        record['fixture_revision']='c'*64
        with self.assertRaises(ValueError):s.validate_registry(r,[f])

    def test_cli_unattended_and_missing_coverage(self):
        import os
        import subprocess
        import sys
        env={k:v for k,v in os.environ.items() if k not in ('DISPLAY','WAYLAND_DISPLAY')}
        for args,code in [(['validate','fixtures/scenarios/priority-pass.json'],0),(['coverage','--require-passed'],2),(['validate'],2)]:
            result=subprocess.run([sys.executable,'scripts/scenario.py',*args],cwd=ROOT,env=env,
                                  stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=5)
            self.assertEqual(result.returncode,code,result.stdout+result.stderr)
            self.assertEqual(result.stderr,'')
            self.assertIn(json.loads(result.stdout)['status'],('valid','error'))

class SemanticChoiceTests(unittest.TestCase):
    def test_explicit_block_and_damage_pairs(self):
        for action,kind,values in [
            ('block','blockers',[{'blocker':'bear-0','attacker':'goblin-1'}]),
            ('assign_damage','damage',[{'source':'bear-0','target':'goblin-1','amount':2}]),
        ]:
            s.validate_action({'id':'combat-choice','actor':0,'kind':action,'source':None,
                               'choices':[{'id':'allocation','actor':0,'kind':kind,'values':values}]})

    def test_reject_malformed_target_and_payment(self):
        action={'id':'cast','actor':0,'kind':'cast','source':'growth',
                'choices':[{'id':k,'actor':0,'kind':k,'values':[]} for k in ['mode','targets','payment','discard']]}
        for index,value in [(1,42),(2,-1)]:
            bad=copy.deepcopy(action);bad['choices'][index]['values']=[value]
            with self.assertRaises(ValueError):s.validate_action(bad)

class ResetCompletenessTests(unittest.TestCase):
    setUp = ScenarioTests.setUp
    reset_fixture = ExtendedScenarioTests.reset_fixture
    signed = ExtendedScenarioTests.signed
    def test_mulligan_requires_bottom_choice(self):
        f=self.reset_fixture()
        f['script'].insert(0,{'id':'mull-0','actor':0,'kind':'mulligan','source':None,
                              'choices':[{'id':'mull-choice','actor':0,'kind':'mulligan','values':[]}]})
        f['setup']['shuffle_results']=[{'id':'reshuffle-0','seat':0,'order':f['setup']['ordered_libraries'][0]}]
        with self.assertRaises(ValueError):s.validate(self.signed(f),self.registry)

class SyntheticIdentityTests(unittest.TestCase):
    def test_dead_source_keeps_last_known_identity(self):
        state=s.load(ROOT/'fixtures/scenarios/priority-pass.json')['setup']['state']
        source={'id':'dragon-old','card_id':'shivan-dragon','generation':0,'owner':0,'controller':0,
                'status':{'tapped':False,'controlled_since_turn':0,'damage':0,'power':5,'toughness':5,
                          'keywords':['Flying'],'token':False}}
        state['stack']=[{'id':'boost','source':'dragon-old','controller':0,'kind':'activated',
                         'targets':[],'modes':[],'last_known_source':source}]
        cards=s.source_contract()[0]
        s.validate_state(state,cards)
        bad=copy.deepcopy(state);bad['stack'][0]['last_known_source']=None
        with self.assertRaises(ValueError):s.validate_state(bad,cards)
        bad=copy.deepcopy(state);bad['stack'][0]['last_known_source']['status']['keywords']=['First strike']
        with self.assertRaises(ValueError):s.validate_state(bad,cards)

if __name__ == '__main__': unittest.main()
