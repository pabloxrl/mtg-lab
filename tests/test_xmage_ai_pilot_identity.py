"""L0-003 content accounting: provenance cannot manufacture distinct games."""
import copy
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
import xmage_ai_pilot as pilot


class PilotIdentity(unittest.TestCase):
    def setUp(self):
        self.game=json.loads((ROOT/'fixtures/reference/xmage-ai-normalized.json').read_text())

    def test_altered_seed_uuid_engine_and_provenance_are_one_game(self):
        changed=copy.deepcopy(self.game)
        changed['provenance']={'source':'different engine label','seed':999,'uuid':'renamed'}
        changed['cases'][0]['id']='another-file-and-source-label'
        rows=pilot.distinct_attempts([
            {'attempt_id':'original','transcript':self.game},
            {'attempt_id':'altered-provenance','transcript':changed}])
        self.assertEqual(rows[0]['identity'],rows[1]['identity'])
        self.assertIsNone(rows[0]['duplicate_of'])
        self.assertEqual(rows[1]['duplicate_of'],'original')

    def test_indistinguishable_copy_labels_do_not_make_new_game(self):
        text=json.dumps(self.game)
        changed=json.loads(text.replace('0/mountain/0','COPY-TEMP').replace(
            '0/mountain/1"','0/mountain/0"').replace('COPY-TEMP','0/mountain/1'))
        self.assertEqual(pilot.canonical_game(self.game),pilot.canonical_game(changed))

    def test_changed_actual_chance_and_player_choice_are_distinct(self):
        changed=copy.deepcopy(self.game)
        shuffle=changed['cases'][0]['chance'][0]['after']
        other=next(i for i,v in enumerate(shuffle) if v.split('/')[1]!=shuffle[0].split('/')[1])
        shuffle[0],shuffle[other]=shuffle[other],shuffle[0]
        self.assertNotEqual(pilot.identity(self.game),pilot.identity(changed))
        changed=copy.deepcopy(self.game)
        changed['cases'][0]['play'][0]['actor']=1-changed['cases'][0]['play'][0]['actor']
        self.assertNotEqual(pilot.identity(self.game),pilot.identity(changed))

    def test_rule_definition_and_starter_are_part_of_identity(self):
        for mutate in (lambda g:g['pins'].update({'data/rules/cr-2026-09-25.json':'0'*64}),
                       lambda g:g['cases'][0].update(starter=1-g['cases'][0]['starter'])):
            changed=copy.deepcopy(self.game);mutate(changed)
            self.assertNotEqual(pilot.identity(self.game),pilot.identity(changed))
