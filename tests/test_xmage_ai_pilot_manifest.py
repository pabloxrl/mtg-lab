"""The complete provisional pilot is ordinary regression coverage, not a sample."""
import copy
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'scripts'))
import check_xmage_pilot as check


class CompletePilot(unittest.TestCase):
    def test_every_retained_pair_and_required_checkpoint(self):
        manifest=check.check_retained()
        self.assertEqual(len(manifest['games']),10)

    def test_every_game_executes_twice_in_native(self):
        check.replay_native()


class PilotInventory(unittest.TestCase):
    def setUp(self):
        self.manifest=json.loads(check.DEFAULT.read_text())

    def validate_mutation(self,edit,message):
        altered=copy.deepcopy(self.manifest);edit(altered)
        original_read=Path.read_text
        def read(path,*args,**kwargs):
            if path==check.DEFAULT:
                return json.dumps(altered)
            return original_read(path,*args,**kwargs)
        with patch.object(Path,'read_text',read):
            with self.assertRaisesRegex(ValueError,message):
                check.validate()

    def test_missing_game_cannot_pass_as_complete(self):
        self.validate_mutation(lambda m:m['games'].pop(),'ten complete distinct games')

    def test_same_content_cannot_count_twice_under_another_attempt(self):
        def edit(m):
            repeated=copy.deepcopy(m['games'][0]);repeated['attempt']='renamed-duplicate'
            m['games'][-1]=repeated
        self.validate_mutation(edit,'duplicate canonical game')

    def test_missing_repeat_is_not_twice_per_engine(self):
        self.validate_mutation(lambda m:m['games'][0]['reference'].pop(),'twice per engine')

    def test_missing_negative_execution_cannot_pass(self):
        self.validate_mutation(lambda m:m['games'][0]['negative_results'].pop(),'both negative result streams')

    def test_duplicate_probe_has_zero_quota(self):
        self.validate_mutation(lambda m:m['duplicate_probes'][0].update(counted=1),'duplicate quota')

    def test_corrupted_artifact_is_rejected_before_replay(self):
        name=self.manifest['games'][0]['source']
        self.validate_mutation(lambda m:m['artifacts'].update({name:'0'*64}),'artifact checksum')

    def test_provisional_pilot_cannot_claim_final_admission(self):
        self.validate_mutation(lambda m:m['final_admission'].update(status='admitted'),'final admission claim')
