"""Independent CR103.5 opening alignment and frozen-source regression coverage."""
import json
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[1]


class ForgeOpeningAlignmentTests(unittest.TestCase):
    def test_both_starters_and_every_source_choice_are_retained_once(self):
        fixtures=json.loads((ROOT/'fixtures/reference/forge-opening.json').read_text())
        self.assertEqual({f['native']['cases'][0]['starter'] for f in fixtures},{0,1})
        for f in fixtures:
            case=f['native']['cases'][0]
            self.assertEqual(sorted(f['alignment']),list(range(len(f['source_choices']))))
            for n,i in enumerate(f['alignment']):
                expected=dict(f['source_choices'][i],sequence=n)
                self.assertEqual(case['choices'][n],expected)
                self.assertEqual(f['expected'][n],f['source_checkpoints'][i])
            # CR103.4–103.5: full frozen decks; every declaration has 7 minus
            # previous London bottoms, each pre-bottom hand has seven cards.
            for point in f['expected']:
                self.assertEqual(point['life'],[20,20])
                self.assertEqual(len(point['hand'])+len(point['library']),40)
                size=7 if point['boundary']=='bottom' else 7-point['mulligans']
                self.assertEqual(len(point['hand']),size)
                self.assertEqual(len(set(point['hand']+point['library'])),40)
            self.assertFalse(f['complete_game'])
            self.assertEqual(f['endpoint'],f['remainder']['first_record'])
            self.assertGreater(f['remainder']['records'],0)

    def test_starter_zero_preserves_the_source_scheduling_difference(self):
        f=next(f for f in json.loads((ROOT/'fixtures/reference/forge-opening.json').read_text())
               if f['native']['cases'][0]['starter']==0)
        self.assertEqual([c['kind'] for c in f['source_choices']],['declare','bottom','declare','declare'])
        self.assertEqual([c['kind'] for c in f['native']['cases'][0]['choices']],['declare','declare','bottom','declare'])
        self.assertEqual(f['alignment'],[0,2,1,3])
