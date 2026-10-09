"""CR 608.2b/h, 120.6, 400.7 and 514.2: original Sentry adapter controls."""
import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import instant_reference as instant


class SentryInstantTests(unittest.TestCase):
    def setUp(self):
        self.fixture = json.loads(instant.FIXTURE.read_text())
        self.expected = json.loads(instant.EXPECTATIONS.read_text())
        self.results = {c['id']: {'checkpoints': self.expected[c['id']], 'consumed': c['script']}
                        for c in self.fixture['cases']}

    def test_literal_rules_checkpoints(self):
        def state(case, name):
            return next(p['state'] for p in self.expected[case] if p['name'] == name)
        for case in ['rules-foundations_micro_v1-bite-down-positive', 'rules-objects-bookkeeping-positive']:
            s = state(case, 'bite-resolved')
            self.assertEqual([s['objects'][1][k] for k in ['power', 'toughness', 'damage']], [4, 4, 2])
            self.assertEqual(s['objects'][0]['damage'], 0)
        case = 'rules-objects-bookkeeping-interaction'
        for name, stats in [('bite-resolved', [4,4,2]), ('growth-resolved', [7,7,2]), ('next-upkeep', [4,4,0])]:
            s = state(case, name)
            self.assertEqual([s['objects'][1][k] for k in ['power', 'toughness', 'damage']], stats)
        case = 'rules-foundations_micro_v1-bite-down-negative'
        self.assertEqual(state(case, 'before-rejection'), state(case, 'after-rejection'))
        case = 'rules-foundations_micro_v1-bite-down-regression'
        s = state(case, 'lower-finished')
        self.assertEqual(s['objects'][1]['damage'], 0)
        self.assertEqual(s['objects'][0]['zone'], 'graveyard')
        self.assertEqual(s['last_resolution'], {'id':'lower','legal_targets':1,'resolved':True})

    def test_mutated_sentry_observations_preserve_first_divergence(self):
        controls = [
            ('rules-objects-bookkeeping-positive', 2, 'toughness', 2),
            ('rules-objects-bookkeeping-positive', 2, 'damage', 0),
            ('rules-objects-bookkeeping-interaction', 4, 'power', 4),
            ('rules-objects-bookkeeping-interaction', 12, 'damage', 2),
            ('rules-objects-bookkeeping-interaction', 12, 'toughness', 7),
            ('rules-foundations_micro_v1-bite-down-regression', 4, 'damage', 2),
        ]
        for case, point, field, value in controls:
            wrong = copy.deepcopy(self.results)
            wrong[case]['checkpoints'][point]['state']['objects'][1][field] = value
            with self.assertRaisesRegex(ValueError, 'first divergence') as caught:
                instant.compare(self.fixture, wrong)
            self.assertIn(f'{case}.checkpoints[{point}].state.objects[1].{field}', str(caught.exception))
        case = 'rules-foundations_micro_v1-bite-down-regression'
        for field, value in [('id', 'decoy0'), ('incarnation', 1)]:
            wrong = copy.deepcopy(self.results)
            wrong[case]['checkpoints'][3]['state']['stack'][0]['targets'][0][field] = value
            with self.assertRaisesRegex(ValueError, r'checkpoints\[3\].state.stack\[0\].targets\[0\]'):
                instant.compare(self.fixture, wrong)
        case = 'rules-continuous-resolution-power-positive'
        wrong = copy.deepcopy(self.results)
        wrong[case]['checkpoints'][2]['state']['stack'].reverse()
        with self.assertRaisesRegex(ValueError, r'checkpoints\[2\].state.stack\[0\]'):
            instant.compare(self.fixture, wrong)

    def test_rejected_boundary_mutations_are_detected(self):
        case = 'rules-foundations_micro_v1-bite-down-negative'
        for field in ['mana', 'zones', 'history', 'targets']:
            wrong = copy.deepcopy(self.results)
            s = wrong[case]['checkpoints'][2]['state']
            if field == 'mana': s['mana'][0][4] = 0
            elif field == 'zones': s['objects'][-1]['zone'] = 'graveyard'
            elif field == 'history': s['damage_events'].append({'source':'source','target':'destination','amount':2})
            else: s['stack'].append({'id':'bite','controller':0,'targets':[{'id':'destination','incarnation':0}]})
            with self.assertRaisesRegex(ValueError, r'checkpoints\[2\].state'):
                instant.compare(self.fixture, wrong)

    def test_live_observation_controls_use_the_normal_comparator(self):
        import sentry_instant_controls as controls
        mutations = list(controls.observation_mutants(self.results))
        self.assertEqual(len(mutations), 9)
        for name, wrong, path in mutations:
            with self.subTest(name=name):
                with self.assertRaisesRegex(ValueError, 'first divergence') as caught:
                    instant.compare(self.fixture, wrong)
                self.assertIn(path, str(caught.exception))

    def test_live_choice_controls_keep_the_exact_sentry_setup(self):
        import sentry_instant_controls as controls
        original = next(c for c in self.fixture['cases'] if c['id'] == controls.POSITIVE)
        mutations = list(controls.choice_mutants(self.fixture))
        self.assertEqual(len(mutations), 5)
        for name, wrong, native, reference in mutations:
            with self.subTest(name=name):
                self.assertEqual(wrong['cases'][0]['setup'], original['setup'])
                self.assertNotEqual(wrong['cases'][0]['script'], original['script'])
                self.assertTrue(native and reference)
