"""Immutable AI inputs; expectations from source witnesses and CR404.3/704.3."""
import sys
import json
import lzma
import tempfile
import copy
import unittest
from unittest.mock import patch
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import xmage_ai_normalize as normalize


class CapturedCombatSettlement(unittest.TestCase):
    def test_two_owned_combat_deaths_preserve_recorded_arrival_order(self):
        # Original source 6053 records simultaneous damage; 6061 records SBA.
        # Cub 0 dies to deathtouch, Cub 1 to lethal damage. CR404.3 requires
        # their owner's order before priority, even though upstream automatically
        # selected an order (the preserved patched-combat-sba-v1 exception).
        converted = normalize.normalize(ROOT / 'doc/evidence/xmage-ai-pilot/'
            'baseline-v1/06-green-green-0/trace.jsonl.xz')
        orders = [e for e in converted['transcript']['cases'][0]['play']
                  if e['kind'] == 'graveyard_order' and e['turn'] == 24]
        self.assertEqual(len(orders), 1)
        self.assertEqual(orders[0]['actor'], 0)
        self.assertEqual(orders[0]['selection'], [
            {'source': '0/bear-cub/0', 'incarnation': 3},
            {'source': '0/bear-cub/1', 'incarnation': 5}])
        self.assertEqual(orders[0]['event'], 'combat-sba/24')


class CapturedTriggerPlacement(unittest.TestCase):
    def test_archer_cast_trigger_is_placed_before_priority(self):
        # Source SPELL_CAST 1534 creates the Archer instance; source 1540
        # places it. CR603.3/117.5 require placement before genuine priority.
        converted = normalize.normalize(ROOT / 'doc/evidence/xmage-ai-pilot/'
            'baseline-v1/00-red-green-0/trace.jsonl.xz')
        orders = [e for e in converted['transcript']['cases'][0]['play']
                  if e['kind'] == 'order_triggers']
        self.assertEqual(orders[0]['actor'], 0)
        self.assertEqual(orders[0]['order'], [{
            'source': '0/firebrand-archer/2', 'incarnation': 5,
            'ability': 'archer', 'event': 0}])

    def test_explicit_cyclops_archer_order_and_boost(self):
        # The first paired callback at source 3023 selects Cyclops first;
        # its source effect at 3056 carries +3/+0 until cleanup (CR611.2c).
        converted = normalize.normalize(ROOT / 'doc/evidence/xmage-ai-pilot/'
            'baseline-v1/02-green-red-0/trace.jsonl.xz')
        orders = [e for e in converted['transcript']['cases'][0]['play']
                  if e['kind'] == 'order_triggers' and len(e['order']) == 2]
        self.assertEqual([key['ability'] for key in orders[0]['order']], ['cyclops', 'archer'])
        self.assertTrue(any(m['power_boost']==3 and m['boost']==0
            for c in converted['source_checkpoints'] for m in c['state']['effects']))


class CapturedModes(unittest.TestCase):
    def test_surprise_boost_mode_preserves_affected_objects(self):
        # Source cast 1977 selects BoostControlledEffect, and 2068 records
        # the affected-object set and +2/+0 values. CR700.2/611.2c.
        converted = normalize.normalize(ROOT / 'doc/evidence/xmage-ai-pilot/'
            'baseline-v1/04-red-red-0/trace.jsonl.xz')
        modes = [e['mode'] for e in converted['transcript']['cases'][0]['play'] if e['kind']=='mode']
        self.assertEqual(modes[0], 0)
        self.assertTrue(any(m['power_boost']==2 and m['boost']==0
            for c in converted['source_checkpoints'] for m in c['state']['effects']))


class CapturedGrantLifetime(unittest.TestCase):
    def test_departed_pyromancer_has_no_active_haste_grant(self):
        # Source game RG/1: Cavalry grants haste to Pyromancer incarnation 3;
        # it dies in combat on turn 14. CR400.7/611.2c: that grant cannot
        # remain active on a nonexistent battlefield object or a new identity.
        source=ROOT/'doc/evidence/xmage-ai-pilot/regressions/rg1-converted.json.xz'
        converted=json.loads(lzma.decompress(source.read_bytes()))
        with tempfile.TemporaryDirectory() as temporary:
            folder=Path(temporary)
            tape=folder/'input.json';tape.write_text(json.dumps(converted['transcript']))
            with patch.dict('os.environ',{'MTG_AI_REPLAY_INPUT':str(tape)}):
                runs=normalize.run_native(folder)
            for run in runs:
                self.assertTrue(all(grant['object'] is not None
                    for point in run['points'] for grant in point['grants']))
                normalize.verify_native(converted,run)


class CapturedCleanup(unittest.TestCase):
    def test_only_final_round_bottoms_follow_final_shuffle(self):
        # CR103.5: the next mulligan reshuffles the earlier bottomed card too.
        folder=ROOT/'doc/evidence/xmage-ai-pilot/regressions'
        converted=json.loads(lzma.decompress((folder/'rr1-converted.json.xz').read_bytes()))
        native=json.loads(lzma.decompress((folder/'rr1-native.json.xz').read_bytes()))['run']
        normalize.verify_native(converted,native)
        bad=copy.deepcopy(converted)
        bad['opening_partials'][0]['state']['library'].reverse()
        with self.assertRaisesRegex(ValueError,'partial opening checkpoint'):
            normalize.verify_native(bad,native)

    def test_cleanup_uses_the_captured_discard(self):
        # CR514.1; source 1757-1762 explicitly selects this physical Dragon,
        # rather than a policy choosing a different excess card during replay.
        converted=normalize.normalize(ROOT/'doc/evidence/xmage-ai-pilot/'
            'baseline-v1/05-red-red-1/trace.jsonl.xz')
        choices=[e for e in converted['transcript']['cases'][0]['play'] if e['kind']=='cleanup_discard']
        self.assertEqual(choices[0]['actor'],0)
        self.assertEqual(choices[0]['selection'],[{'source':'0/shivan-dragon/0','incarnation':1}])
        bottoms=[e for e in converted['transcript']['cases'][0]['choices']
                 if e['kind']=='bottom' and e['round']==2]
        self.assertEqual(len(bottoms),1)
        self.assertEqual(bottoms[0]['selection'],['0/mountain/14','0/firebrand-archer/0'])


class CapturedTriggerProvenance(unittest.TestCase):
    def setUp(self):
        folder=ROOT/'doc/evidence/xmage-ai-pilot/regressions'
        self.reference=json.loads(lzma.decompress((folder/'rg0-reference.json.xz').read_bytes()))
        self.converted=json.loads(lzma.decompress((folder/'rg0-converted.json.xz').read_bytes()))
        self.native=json.loads(lzma.decompress((folder/'rg0-native.json.xz').read_bytes()))['run']

    def test_mulligan_return_binds_trigger_to_witnessed_counter(self):
        import admission_observations
        import admission_reference
        # Pinned LondonMulligan returns hand to library directly: CR400.7
        # still changes semantic incarnation, without incrementing raw ZCC.
        case=admission_reference.reference_case(self.converted['transcript']['cases'][0])[0]
        admission_observations.xmage_trigger_provenance(self.reference,case)
        bad=copy.deepcopy(self.reference)
        bad['trigger_provenance'][0]['raw_source_zcc']+=1
        with self.assertRaisesRegex(ValueError,'source incarnation binding'):
            admission_observations.xmage_trigger_provenance(bad,case)

    def test_full_trigger_replay_compares_staging_with_bound_provenance(self):
        normalize.verify_pair(self.converted,self.native,self.reference)

    def test_forged_counter_chain_and_pending_set_reject(self):
        import admission_observations
        import admission_reference
        case=admission_reference.reference_case(self.converted['transcript']['cases'][0])[0]
        bad=copy.deepcopy(self.reference)
        transition=next(t for t in bad['identity_transitions']
                        if t['source']=='0/firebrand-archer/2' and t['direct_mulligan_return'])
        transition['raw_after']+=1
        with self.assertRaisesRegex(ValueError,'transition counter'):
            admission_observations.xmage_trigger_provenance(bad,case)
        bad=copy.deepcopy(self.native)
        point=next(p for p in bad['points'] if p['pending_triggers'])
        point['pending_triggers'][0]['controller']=1
        with self.assertRaisesRegex(ValueError,'pending source/event set'):
            normalize.verify_native(self.converted,bad)


class CapturedPendingSet(unittest.TestCase):
    def test_pending_collection_order_is_not_the_chosen_stack_order(self):
        # CR603.3b: the controller chooses stack order for simultaneous
        # waiting triggers. Their pre-choice collection has no rules order.
        # Source 2546 lists Cyclops/Archer; native lists Archer/Cyclops.
        folder=ROOT/'doc/evidence/xmage-ai-pilot/regressions'
        converted=json.loads(lzma.decompress((folder/'rg-extra-converted.json.xz').read_bytes()))
        native=json.loads(lzma.decompress((folder/'rg-extra-native-1.json.xz').read_bytes()))['run']
        normalize.verify_native(converted,native)
        # Membership and event identity still matter; removing one cannot pass.
        bad=copy.deepcopy(native)
        next(p for p in bad['points'] if len(p['pending_triggers'])==2)['pending_triggers'].pop()
        with self.assertRaisesRegex(ValueError,'pending source/event set'):
            normalize.verify_native(converted,bad)
