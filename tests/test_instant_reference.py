"""Original translator/comparator regressions; expectations are rules-derived."""
import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import instant_reference as instant

class InstantReferenceTests(unittest.TestCase):
    def setUp(self):
        self.fixture=json.loads(instant.FIXTURE.read_text())
        expected=json.loads(instant.EXPECTATIONS.read_text())
        self.results={c['id']: {'checkpoints':expected[c['id']], 'consumed':c['script']} for c in self.fixture['cases']}

    def test_target_power_stack_mutations_fail(self):
        instant.compare(self.fixture,self.results)
        for field in ['target','power','stack']:
            wrong=copy.deepcopy(self.results)
            state=wrong['destination']['checkpoints'][2]['state']
            if field=='target': state['stack'][1]['targets']=['source']
            elif field=='power': state['objects'][0]['power']=5
            else: state['stack'].reverse()
            with self.assertRaisesRegex(ValueError,'first divergence'):
                instant.compare(self.fixture,wrong)

    def test_every_consumed_choice_is_exact(self):
        for case in self.fixture['cases']:
            name=case['id'];n=len(case['script'])
            for i in range(n):
                for mutation in ['omit','extra','reorder','actor']:
                    wrong=copy.deepcopy(self.results);script=wrong[name]['consumed']
                    if mutation=='omit': script.pop(i)
                    elif mutation=='extra': script.insert(i,copy.deepcopy(script[i]))
                    elif mutation=='reorder': script[i],script[(i+1)%n]=script[(i+1)%n],script[i]
                    elif 'actor' in script[i]: script[i]['actor']=1-script[i]['actor']
                    else: continue
                    with self.assertRaisesRegex(ValueError,'first divergence'):
                        instant.compare(self.fixture,wrong)

    def test_missing_extra_or_supplied_success_fail(self):
        for change in ['missing','extra','checkpoint','expected']:
            wrong=copy.deepcopy(self.results)
            if change=='missing': del wrong['source']
            elif change=='extra': wrong['fake']={}
            elif change=='checkpoint': wrong['destination']['checkpoints'].pop()
            else: wrong['destination']['expected']=wrong['destination']['checkpoints']
            with self.assertRaises(ValueError): instant.compare(self.fixture,wrong)

    def test_input_translation_is_bound_to_original_script(self):
        for field in ['setup', 'script', 'expected']:
            wrong=copy.deepcopy(self.fixture)
            if field=='setup': wrong['cases'][0]['setup']['priority']=1
            elif field=='script': wrong['cases'][0]['script'][1:3]=reversed(wrong['cases'][0]['script'][1:3])
            else: wrong['cases'][0]['expected']=self.results['destination']['checkpoints']
            with self.assertRaisesRegex(ValueError,'first divergence'):
                instant.compare(wrong,self.results)

    def test_damage_zones_mana_priority_and_missing_fields(self):
        for field in ['amount','zone','mana','priority','missing']:
            wrong=copy.deepcopy(self.results)
            state=wrong['source']['checkpoints'][-1]['state']
            if field=='amount':state['damage_events'][0]['amount']=2
            elif field=='zone':state['objects'][1]['zone']='battlefield'
            elif field=='mana':state['mana'][0][4]=1
            elif field=='priority':state['priority']=1
            else:del state['stack']
            with self.assertRaisesRegex(ValueError,'first divergence'):
                instant.compare(self.fixture,wrong)

    def test_unavailable_reference_cannot_leave_a_success_receipt(self):
        import tempfile
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as temp:
            output=Path(temp)/'receipt';output.mkdir()
            (output/'acceptance.json').write_text('{"status":"agreed"}')
            with patch.object(sys,'argv',['instant_reference','--cache',temp,'--output',str(output)]), patch.object(instant.xmage,'verify_inputs',side_effect=ValueError('unavailable pinned reference')):
                self.assertEqual(instant.main(),1)
            self.assertFalse((output/'acceptance.json').exists())
            self.assertIn('unavailable pinned reference',json.loads((output/'first-divergence.json').read_text())['error'])

    def test_departed_identity_and_resolution_mutants_fail_at_first_checkpoint(self):
        # CR 400.7 / 608.2b: the old target persists on stack, and no illegal
        # source can deal damage. These mutations do not calculate expectations.
        for variant in ['growth-gone','bite-source-gone','bite-destination-gone','bite-both-gone']:
            for mutation in ['retarget','new-incarnation','missing-identity','illegal-damage','resolution','unconsumed']:
                wrong=copy.deepcopy(self.results)
                state=wrong[variant]['checkpoints'][3]['state']
                final=wrong[variant]['checkpoints'][-1]['state']
                if mutation=='retarget': state['stack'][0]['targets'][0]['id']='decoy0'
                elif mutation=='new-incarnation': state['stack'][0]['targets'][0]['incarnation']=1
                elif mutation=='missing-identity': del state['stack'][0]['targets'][0]['incarnation']
                elif mutation=='illegal-damage': final['damage_events'].append({'source':'source','target':'destination','amount':2})
                elif mutation=='resolution': final['last_resolution']['resolved']=not final['last_resolution']['resolved']
                else: wrong[variant]['consumed'].pop()
                with self.assertRaisesRegex(ValueError,'first divergence') as caught:
                    instant.compare(self.fixture,wrong)
                self.assertIn(variant,str(caught.exception))
                if mutation in ['retarget','new-incarnation','missing-identity']:
                    self.assertIn('checkpoints[3].state.stack[0].targets[0]',str(caught.exception))
