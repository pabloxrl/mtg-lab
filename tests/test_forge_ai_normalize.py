"""L0-011: shared content identity keeps original source evidence separate.

CR103.5 and CR400.7 require actual shuffle order and stable physical copies.
The immutable Forge game, rather than native output, supplies checkpoints.
"""
import copy
import gzip
import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'scripts'))
import forge_ai_normalize as normalize
import xmage_ai_pilot as identity


class ForgeNormalizationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = ROOT/'doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz'
        cls.game = normalize.normalize(cls.source)

    def test_complete_game_has_shared_identity_and_separate_source_evidence(self):
        game = self.game
        self.assertTrue('identity' in game, 'normalization must bind the common content identity')
        self.assertEqual(game['identity'], identity.identity(game['transcript']))
        self.assertEqual(game['fixture']['operations'][-1]['checkpoint']['outcome'],
                         dict(winner=1, losses=['Life', None]))
        self.assertEqual(len(game['transcript']['cases'][0]['play']), 818)
        self.assertEqual(game['fixture'], json.loads((ROOT/'fixtures/reference/forge-combat.json').read_text())[0])
        self.assertEqual(game['provenance']['engine'], 'forge')
        self.assertNotIn('provenance', json.loads(game['canonical']))

    def test_normalized_artifact_retains_every_opening_checkpoint(self):
        self.assertTrue('opening_checkpoints' in self.game,
                        'full normalization must retain intermediate London checkpoints')
        self.assertEqual(len(self.game['opening_checkpoints']), 8)

    def test_cross_source_duplicate_retains_both_origins_and_first_attribution(self):
        # Synthetic cross-source duplicate, not a second AI capture or quota.
        transcript = copy.deepcopy(self.game['transcript'])
        forge = dict(engine='forge', receipt='original-forge-receipt')
        xmage = dict(engine='xmage', receipt='synthetic-duplicate-test')
        entries = normalize.merge_origin([], transcript, forge)
        transcript['provenance'] = xmage
        transcript['cases'][0]['id'] = 'different-source-name'
        entries = normalize.merge_origin(entries, transcript, xmage)
        self.assertEqual(len(entries), 1)
        self.assertEqual(entries[0]['origins'], [forge, xmage])
        self.assertEqual(entries[0]['first_origin'], forge)
        self.assertEqual(normalize.merge_origin(entries, transcript, xmage), entries)
        transcript['cases'][0]['play'][0]['actor'] ^= 1
        self.assertEqual(len(normalize.merge_origin(entries, transcript, xmage)), 2)

    def test_equal_card_renaming_keeps_identity_but_changed_choice_does_not(self):
        # Swap two physical Mountain labels everywhere, preserving relationships.
        original = self.game['transcript']
        def rename(value):
            if isinstance(value, dict):
                return {k: rename(v) for k,v in value.items()}
            if isinstance(value, list):
                return [rename(v) for v in value]
            if isinstance(value, str):
                return {'0/mountain/0':'0/mountain/1',
                        '0/mountain/1':'0/mountain/0'}.get(value, value)
            return value
        self.assertEqual(identity.identity(rename(original)), self.game['identity'])
        bad = copy.deepcopy(original)
        land = next(a for a in bad['cases'][0]['play'] if a['kind'] == 'play_land')
        land['incarnation'] += 1
        self.assertNotEqual(identity.identity(bad), self.game['identity'])

    def test_changed_checkpoint_rejected_against_original_source(self):
        bad = copy.deepcopy(self.game)
        bad['fixture']['operations'][0]['checkpoint']['life'][0] -= 1
        with self.assertRaisesRegex(ValueError, r'normalized artifact:.*operations\[0\].checkpoint.life\[0\]'):
            normalize.validate(bad, self.source)

    def test_actual_native_and_reference_openings_match_and_mutations_fail(self):
        # Executed witnesses from verified #328; source capture supplies expected
        # values. These are real consumer outputs, not fabricated sibling engines.
        folder = ROOT/'doc/evidence/forge-combat'
        index = json.loads((folder/'attempt-index.json').read_text())['files']
        for name in ['combat-live-integrated/native.json',
                     'combat-cross-integrated/forge-actual-combat.xmage-1.json',
                     'combat-cross-integrated/forge-actual-combat.xmage-2.json']:
            item = next(item for item in index if item['path'] == name)
            packed = (folder/item['storage']).read_bytes()
            self.assertEqual(hashlib.sha256(packed).hexdigest(), item['stored_sha256'])
            raw = gzip.decompress(packed)
            self.assertEqual(hashlib.sha256(raw).hexdigest(), item['sha256'])
            run = json.loads(raw)
            if isinstance(run, list):
                run = run[0]
            normalize.verify_opening(self.game, run['opening'])
            changed = copy.deepcopy(run['opening'])
            library = changed['points'][0]['library']
            library[0], library[1] = library[1], library[0]
            source = self.game['opening_checkpoints'][0]['source']
            with self.assertRaisesRegex(ValueError, f'/source/{source}/opening checkpoint/0'):
                normalize.verify_opening(self.game, changed)
            changed = copy.deepcopy(run['opening'])
            changed['points'].pop()
            with self.assertRaisesRegex(ValueError, 'opening checkpoint count'):
                normalize.verify_opening(self.game, changed)

    def test_collision_cannot_drop_an_origin(self):
        entries = normalize.merge_origin([], self.game['transcript'], dict(engine='forge'))
        entries[0]['canonical'] = '{}'
        with self.assertRaisesRegex(ValueError, 'canonical hash collision'):
            normalize.merge_origin(entries, self.game['transcript'], dict(engine='xmage'))

    def test_source_opening_faults_remain_available_in_complete_consumer(self):
        import forge_casting
        # Existing independent opening controls must run at full-game scope too.
        for fault, wanted in [('missing-chance', 'record kind random'),
                              ('checkpoint', 'state/players/0/life')]:
            rows = [dict(kind='start'), dict(kind='random'),
                    dict(kind='boundary', state=dict(players=[dict(life=20)]))]
            self.assertEqual(forge_casting.corrupt(dict(prefix=rows), fault), wanted)


if __name__ == '__main__':
    unittest.main()
