"""Real played native client with independently specified trigger acceptance."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
import triggers_reference as reference

class TriggerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.folder=tempfile.TemporaryDirectory();cls.addClassCleanup(cls.folder.cleanup)
        cls.doc=json.loads(reference.FIXTURE.read_text())
        cls.actual=reference.native(Path(cls.folder.name))

    def test_real_played_prefixes_and_all_negative_controls(self):
        reference.check_run(self.actual,self.doc,'native')
        self.assertEqual(len(self.actual['checkpoints']),3)
        self.assertGreater(sum(map(len,self.actual['checkpoints'].values())),450)
        self.assertEqual(len(self.actual['rejections']),16)

    def test_independent_damage_and_cyclops_oracle(self):
        reference.compare(self.actual['checkpoints'])
        self.assertEqual(self.actual['checkpoints']['archer-cyclops'][-1]['permanents']['0/crackling-cyclops/0']['power'],3)

    def test_all_semantic_mutations_are_detected(self):
        controls=reference.comparator_controls(self.actual,self.doc,'native')
        self.assertEqual(len(controls),16)
        self.assertTrue(all(v['path'] and 'first divergence' in v['rejection'] for v in controls.values()))

    def test_missing_extra_and_empty_observations_fail(self):
        for kind in ('missing','extra','empty'):
            bad=copy.deepcopy(self.actual)
            rows=bad['runs']['archer-cyclops']['points']
            if kind=='missing':rows[0].pop('pending_triggers')
            elif kind=='extra':rows[0]['invented']=[]
            else:rows.clear()
            bad['checkpoints']['archer-cyclops']=rows
            with self.subTest(kind=kind),self.assertRaises(ValueError):reference.check_run(bad,self.doc,'native')

    def test_late_pending_and_boundary_mutations_fail(self):
        for repeat in (False, True):
            for field, value in (
                ('trigger_boundary', 'order'),
                ('pending_triggers', [{'controller': 0, 'key': {
                    'source': '0/firebrand-archer/99', 'incarnation': 99,
                    'ability': 'archer', 'event': 99}}]),
            ):
                bad = copy.deepcopy(self.actual)
                run = 'repeat_runs' if repeat else 'runs'
                bad[run]['archer-cyclops']['points'][145][field] = value
                if not repeat:
                    bad['checkpoints']['archer-cyclops'] = bad[run]['archer-cyclops']['points']
                with self.subTest(repeat=repeat, field=field), self.assertRaises(ValueError):
                    reference.check_run(bad, self.doc, 'native')

    def test_no_empty_real_client_pass(self):
        with self.assertRaises(ValueError):reference.compare({})

    def test_wrong_rejection_category_is_not_coverage(self):
        for name in self.actual['rejections']:
            bad=copy.deepcopy(self.actual);bad['rejections'][name]='first divergence: /unrelated'
            with self.subTest(control=name),self.assertRaises(ValueError):reference.check_run(bad,self.doc,'native')

    def test_late_exhaustion_cannot_replace_intended_rejection(self):
        for name in self.actual['rejections']:
            if name=='truncated_tape':continue
            bad=copy.deepcopy(self.actual);bad['rejections'][name]='first divergence: /missing choice before named stop'
            with self.subTest(control=name),self.assertRaises(ValueError):reference.check_run(bad,self.doc,'native')

    def test_actual_choice_consumption_is_required(self):
        bad=copy.deepcopy(self.actual);bad['runs']['duplicate-archers']['consumed_play'].pop()
        with self.assertRaises(ValueError):reference.check_run(bad,self.doc,'native')

    def test_failed_cast_has_no_cast_trigger(self):
        self.assertEqual(self.actual['negative_runs']['failed_cast']['post_run_trigger_counts'],{'pending':0,'stack':0})
        p=self.actual['negative_runs']['failed_cast']['points'][-1]
        self.assertEqual(p['pending_triggers'],[])
        self.assertFalse(any(a.get('ability')=='trigger' for a in p['stack']))
        self.assertEqual(p['life'],[20,20])

    def test_source_death_retains_original_incarnation(self):
        rows=self.actual['checkpoints']['pyromancer-bite']
        witnessed=[p for p in rows if p['stack'] and '0/viashino-pyromancer/0' in p['graveyard'][0]]
        self.assertTrue(witnessed)
        for p in witnessed:
            self.assertEqual(p['stack'][0]['key']['incarnation'],3)
            self.assertEqual(p['incarnations']['0/viashino-pyromancer/0'],4)
        self.assertEqual(rows[-1]['life'],[20,18])

if __name__=='__main__':unittest.main()
