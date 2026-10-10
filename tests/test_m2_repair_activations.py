"""Played activation acceptance; expected behavior follows CR 602/605 and Oracle."""
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import activations_reference as reference
import copy
import json


class ActivationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.folder = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.folder.cleanup)
        cls.actual = reference.native(Path(cls.folder.name))
        cls.doc = json.loads(reference.FIXTURE.read_text())

    def test_real_played_activation_observations(self):
        reference.check_run(self.actual, self.doc, 'native')
        points = [p for rows in self.actual['checkpoints'].values() for p in rows]
        self.assertGreater(len(points), 0)
        self.assertTrue(any(p.get('activation') is not None for p in points),
                        'played consumer must witness activation payment, not just creature casting')

    def test_literal_boosts_and_haste(self):
        reference.compare(self.actual['checkpoints'])

    def test_missing_observation_is_not_agreement(self):
        for field in self.actual['checkpoints']['shivan-single'][0]:
            bad = copy.deepcopy(self.actual['checkpoints'])
            del bad['shivan-single'][0][field]
            with self.subTest(field=field), self.assertRaises(ValueError):
                reference.compare(bad)

    def mutable_result(self):
        bad = dict(self.actual)
        bad["runs"] = copy.deepcopy(self.actual["runs"])
        bad["checkpoints"] = {k:r["points"] for k,r in bad["runs"].items()}
        return bad

    def test_wrong_reserved_source_cannot_pass(self):
        bad = self.mutable_result()
        for p in bad['runs']['invoker-empty']['points']:
            if p['activation'] and p['activation']['sources']:
                p['activation']['sources'][0]['source'] = '1/forest/5'
        bad['checkpoints']['invoker-empty'] = bad['runs']['invoker-empty']['points']
        with self.assertRaises(ValueError):
            reference.check_run(bad, self.doc, 'native')

    def test_identical_activation_stack_actions_are_detected(self):
        bad = self.mutable_result()
        for p in bad['runs']['shivan-repeated']['points']:
            if len(p['stack']) == 2:
                p['stack'][1]['action'] = p['stack'][0]['action']
        bad['checkpoints']['shivan-repeated'] = bad['runs']['shivan-repeated']['points']
        with self.assertRaises(ValueError):
            reference.check_run(bad, self.doc, 'native')

    def test_late_exhaustion_cannot_replace_rejection(self):
        for name in self.actual['rejections']:
            bad = dict(self.actual)
            bad['rejections'] = dict(self.actual['rejections'])
            bad['rejections'][name] = 'first divergence: /missing choice before named stop'
            with self.subTest(control=name), self.assertRaises(ValueError):
                reference.check_run(bad, self.doc, 'native')

    def test_extra_priority_after_mana_is_detected(self):
        bad = self.mutable_result()
        case = next(c for c in self.doc['cases'] if c['id'] == 'invoker-floating')
        index = next(e['sequence'] for e in case['play'] if e['kind'] == 'tap_mana' and e['turn'] == 12)
        bad['runs']['invoker-floating']['points'][index+1]['actor'] = 0
        bad['checkpoints']['invoker-floating'] = bad['runs']['invoker-floating']['points']
        with self.assertRaises(ValueError):
            reference.check_run(bad, self.doc, 'native')

    def test_distinct_raw_stack_identity_is_required(self):
        bad = self.mutable_result()
        for point in bad['runs']['shivan-repeated']['points']:
            if len(point['stack']) == 2:
                point['stack'][1]['raw_birth'] = point['stack'][0]['raw_birth']
        with self.assertRaises(ValueError):
            reference.check_run(bad, self.doc, 'native')

    def test_all_cancellation_stages_are_exercised(self):
        specs = json.loads((reference.ROOT / 'fixtures/reference/full-pool-activations-cancellations.json').read_text())
        self.assertEqual(set(self.actual['cancelled_runs']), set(specs))
        self.assertGreater(len(specs), 20)

    def test_receipt_covers_nested_recording_interface(self):
        self.assertIn(reference.ROOT / 'crates/mtg-core/src/trajectory/v2.rs', reference.source_files())

    def test_paid_flag_is_checked_not_merely_recorded(self):
        bad = self.mutable_result()
        point = next(p for p in bad['runs']['shivan-single']['points']
                     if p['activation'] is not None)
        point['activation']['paid'] = not point['activation']['paid']
        with self.assertRaises(ValueError):
            reference.check_run(bad, self.doc, 'native')

    def test_enemy_source_rejection_is_not_confounded_by_tapping(self):
        spec = reference.negative_inputs(self.doc)['enemy_untapped_mana']
        event = spec['input']['cases'][0]['play'][-1]
        point = self.actual['negative_runs']['enemy_untapped_mana']['points'][-1]
        source = point['permanents'][event['source']]
        self.assertFalse(source['tapped'])
        self.assertFalse(source['sick'])
        self.assertNotEqual(source['controller'], event['actor'])


if __name__ == '__main__':
    unittest.main()
