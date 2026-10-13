"""L0-007 / CR 103.4–103.5: source chronology, physical copies and prefix scope.

The source game supplies choices, not expected rules. Literal counts follow
seven-card draws and cumulative London bottoming; every full deck has 40 cards.
Live Forge/native receipts separately certify executable integration.
"""
import copy
import gzip
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'scripts'))
import forge_opening as opening

TRACE = ROOT/'doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz'


class ForgeOpeningTests(unittest.TestCase):
    def test_real_opening_retains_chronology_and_explicit_remainder(self):
        result = opening.extract(TRACE)
        self.assertEqual(len(result['chance']), 4)  # two initial, two actual redraws
        self.assertEqual([c['selection'] for c in result['choices'] if c['kind']=='declare'],
                         ['keep', 'mulligan', 'mulligan', 'keep'])
        self.assertEqual([len(c['selection']) for c in result['choices'] if c['kind']=='bottom'], [1, 2])
        self.assertEqual(result['starter'], 1)
        self.assertFalse(result['complete_game'])
        self.assertEqual(result['endpoint']['method'], 'chooseSpellAbilityToPlay')
        self.assertGreater(result['remainder']['records'], 0)
        self.assertEqual(len(result['bindings']), 80)
        self.assertEqual([len(p['hand']) for p in result['checkpoints'][-2:]], [7, 5])

    def test_native_fixtures_preserve_both_immutable_source_observations(self):
        fixtures=json.loads((ROOT/'fixtures/reference/forge-opening.json').read_text())
        paths=[TRACE, ROOT/'doc/evidence/forge-opening/source-seed-1/result.json.jsonl.gz']
        for fixture,path in zip(fixtures,paths,strict=True):
            result=opening.extract(path)
            self.assertEqual(fixture['source_sha256'],result['source_sha256'])
            self.assertEqual(fixture['native'],result['native'])
            self.assertEqual(fixture['expected'],result['native_checkpoints'])
            self.assertEqual(fixture['source_choices'],result['choices'])
            self.assertEqual(fixture['source_checkpoints'],result['checkpoints'])
            self.assertEqual(fixture['remainder'],result['remainder'])

    def mutate(self, change, reason):
        with gzip.open(TRACE, 'rt') as stream:
            rows = []
            for s in stream:
                r = json.loads(s); rows.append(r)
                if r.get('method') == 'chooseSpellAbilityToPlay': break
        change(rows)
        with tempfile.TemporaryDirectory() as folder:
            p = Path(folder)/'trace.jsonl.gz'
            with gzip.open(p, 'wt') as stream:
                for r in rows: stream.write(json.dumps(r)+'\n')
            with self.assertRaisesRegex(ValueError, reason): opening.extract(p, verify_receipt=False)

    def test_missing_chance_record(self):
        self.mutate(lambda rows: rows.pop(next(i for i,r in enumerate(rows) if r['kind']=='random')),
                    'record index')

    def test_reordered_chance(self):
        def change(rows):
            a = next(i for i,r in enumerate(rows) if r['kind']=='random')
            rows[a], rows[a+1] = rows[a+1], rows[a]
        self.mutate(change, 'record index')

    def test_duplicate_occurrence(self):
        def change(rows):
            r = next(r for r in rows if r['kind']=='shuffle')
            r['library'][1] = copy.deepcopy(r['library'][0])
        self.mutate(change, 'duplicate physical')

    def test_unconsumed_opening_choice(self):
        def change(rows):
            r = next(r for r in rows if r.get('method')=='chooseSpellAbilityToPlay')
            r['method'] = 'mulliganKeepHand'
        self.mutate(change, 'callback|opening')
