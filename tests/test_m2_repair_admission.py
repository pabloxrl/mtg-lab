"""GR-040 real normal-reset games and fail-closed admission controls."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import admission_reference as reference


class AdmissionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.folder = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.folder.cleanup)
        cls.index = reference.native(Path(cls.folder.name))
        cls.doc = json.loads(reference.FIXTURE.read_text())

    def actual(self, case):
        return json.loads((Path(self.folder.name) / self.index[case['id']][0]).read_text())

    def test_initial_terminal_game_floor(self):
        reference.validate(self.doc)
        self.assertEqual(len(self.index), 16)
        for case in self.doc['cases']:
            with self.subTest(case=case['id']):
                actual = self.actual(case)
                reference.check_game(actual, case, 'native')
                self.assertTrue(actual['records'])
                self.assertTrue(actual['points'])

    def test_missing_payment_field_blocks_matching_winner(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        point = next(p for p in actual['points'] if p['payment'])
        del point['payment']['pool']
        with self.assertRaisesRegex(ValueError, 'payment'):
            reference.check_game(actual, case, 'native')

    def test_forged_effect_blocks_matching_winner(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        point = next(p for p in actual['points'] if p['effects'])
        point['effects'][0]['boost'] += 1
        with self.assertRaisesRegex(ValueError, 'effect'):
            reference.check_game(actual, case, 'native')

    def test_departure_identity_blocks_matching_winner(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        point = next(p for p in actual['points'] if p['departures'])
        point['departures'][0]['object']['incarnation'] += 1
        with self.assertRaisesRegex(ValueError, 'departure'):
            reference.check_game(actual, case, 'native')

    def test_frozen_inputs_and_independent_oracle(self):
        manifest = reference.check_manifest(self.doc)
        self.assertEqual(len(manifest['played_cards']), 20)

    def test_real_invalid_tapes_reject_at_first_divergence(self):
        rejections = reference.check_negatives(self.folder.name, 'native')
        self.assertGreaterEqual(len(rejections), 16)

    def test_missing_every_checkpoint_field(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        for field in reference.FIELDS:
            bad = dict(actual, points=list(actual['points']))
            bad['points'][-1] = dict(bad['points'][-1])
            del bad['points'][-1][field]
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, 'fields'):
                reference.check_game(bad, case, 'native')

    def test_missing_extra_reordered_checkpoint(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        for kind, points in [('missing', actual['points'][:-1]),
                             ('extra', actual['points'] + actual['points'][-1:]),
                             ('reordered', [actual['points'][1], actual['points'][0]] + actual['points'][2:])]:
            with self.subTest(kind=kind), self.assertRaisesRegex(ValueError, 'checkpoint'):
                reference.check_game(dict(actual, points=points), case, 'native')

    def test_forged_outcome_and_nonrules_quota(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        for reason in ('Concession', 'Timeout', 'Life'):
            bad = copy.deepcopy(actual)
            final = bad['points'][-1]
            if reason == 'Life':
                winner = final['outcome']['winner']
                final['outcome'] = dict(winner=1-winner, losses=['Life' if s == winner else None for s in (0, 1)])
            else:
                final['outcome']['losses'][1-final['outcome']['winner']] = reason
            with self.subTest(reason=reason), self.assertRaises(ValueError):
                reference.check_game(bad, case, 'native')

    def test_altered_consumed_action_and_chance(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        for field in ('play', 'chance', 'choices'):
            bad = copy.deepcopy(actual)
            target = bad if field == 'play' else bad['opening']
            target['consumed_'+field].pop()
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, 'consumed'):
                reference.check_game(bad, case, 'native')

    def test_duplicate_identity_and_mismatched_pin(self):
        for field in ('duplicate_identity', 'mismatched_pin', 'missing_chance', 'extra_chance', 'reordered_chance'):
            bad = copy.deepcopy(self.doc)
            if field == 'mismatched_pin':
                bad['pins']['references/xmage/pins.json'] = '0' * 64
            elif field == 'duplicate_identity':
                row = bad['cases'][0]['chance'][0]['after']; row[1] = row[0]
            elif field == 'missing_chance':bad['cases'][0]['chance'].pop()
            elif field == 'extra_chance':bad['cases'][0]['chance'].append(bad['cases'][0]['chance'][0])
            else:bad['cases'][0]['chance'].reverse()
            with self.subTest(field=field), self.assertRaises(ValueError):reference.validate(bad)

    def test_duplicate_game_cannot_fill_missing_row(self):
        bad = copy.deepcopy(self.doc)
        bad['cases'][-1] = copy.deepcopy(bad['cases'][0])
        bad['cases'][-1]['id'] = 'duplicate-directed'
        with self.assertRaises(ValueError):reference.validate(bad)

    def test_forged_transaction_stack_is_rejected(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        point = next(p for p in actual['points'] if p['payment'])
        point['life'][0] += 1
        with self.assertRaisesRegex(ValueError, 'transaction footprint'):
            reference.check_game(actual, case, 'native')

    def test_required_creation_and_stack_provenance(self):
        case = next(c for c in self.doc['cases'] if c['id'] == 'GR-040-RG-S1-heuristic')
        for field in ('birth', 'definition_hash'):
            actual = self.actual(case)
            for point in actual['points']:
                for creation in point['creations']:creation.pop(field)
            with self.subTest(field=field), self.assertRaisesRegex(ValueError, 'creation'):
                reference.check_game(actual, case, 'native')
        actual = self.actual(case)
        point = next(p for p in actual['points'] if any('raw_birth' in s for s in p['stack']))
        next(s for s in point['stack'] if 'raw_birth' in s).pop('raw_birth')
        with self.assertRaisesRegex(ValueError, 'stack'):
            reference.check_game(actual, case, 'native')

    def test_actual_pre_sba_damage_is_required(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        self.assertTrue(actual['marked'], 'The actual quantum observer must witness marks before SBA')
        actual['marked'].pop()
        with self.assertRaisesRegex(ValueError, 'simultaneous damage'):
            reference.check_game(actual, case, 'native')

    def test_missing_opening_checkpoint_blocks_admission(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        actual['opening']['points'].pop()
        with self.assertRaisesRegex(ValueError, 'opening'):
            reference.check_game(actual, case, 'native')

    def test_altered_pre_sba_damage_blocks_matching_winner(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        point = actual['marked'][0]
        creature = next(v for v in point['permanents'].values() if v['damage'] is not None)
        creature['damage'] += 1
        with self.assertRaisesRegex(ValueError, 'simultaneous damage'):
            reference.check_game(actual, case, 'native')

    def test_same_name_copy_swap_is_rejected(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        row = actual['points'][0]['library'][0]
        a, b = next((a, b) for a in range(len(row)) for b in range(a+1, len(row))
                    if row[a].split('/')[1] == row[b].split('/')[1])
        row[a], row[b] = row[b], row[a]
        with self.assertRaisesRegex(ValueError, 'occurrence order'):
            reference.check_game(actual, case, 'native')

    def test_cleanup_requires_nested_creation_provenance(self):
        case = next(c for c in self.doc['cases'] if c['id'] == 'GR-040-RG-S1-heuristic')
        actual = self.actual(case)
        point = next(p for p in actual['cleanup_checkpoints'] if p['creations'])
        point['creations'][0].pop('birth')
        with self.assertRaisesRegex(ValueError, 'creation'):
            reference.check_game(actual, case, 'native')

    def test_damage_requires_complete_permanent_observation(self):
        case = self.doc['cases'][0]
        actual = self.actual(case)
        point = actual['marked'][0]
        next(iter(point['permanents'].values())).pop('flying')
        with self.assertRaisesRegex(ValueError, 'permanent fields'):
            reference.check_game(actual, case, 'native')

    def test_terminal_auxiliary_audit_requires_creation_identity(self):
        case = next(c for c in self.doc['cases'] if c['id'] == 'GR-040-RG-S1-heuristic')
        actual = self.actual(case)
        point = copy.deepcopy(actual['points'][-1])
        point['creations'][0].pop('birth')
        with self.assertRaisesRegex(ValueError, 'creation'):
            reference.auxiliary(point, actual, case, 'native', len(case['play'])-1, '/pre-departure')

    def test_auxiliary_raw_history_cannot_change_identity(self):
        case = next(c for c in self.doc['cases'] if c['id'] == 'GR-040-RG-S1-heuristic')
        actual = self.actual(case)
        point = next(p for p in actual['cleanup_checkpoints'] if p['creations'])
        point['creations'][0]['birth'] += 100000
        with self.assertRaisesRegex(ValueError, 'raw history'):
            reference.check_game(actual, case, 'native')
