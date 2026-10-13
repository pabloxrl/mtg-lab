"""L0-008 prefix boundaries: real source records, never gameplay certification.

CR 117/601 require priority, cast choices and costs to remain ordered. These
parser tests supplement the mandatory executable Forge/native acceptance.
"""
import json
import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import forge_casting
import forge_casting_native
import forge_opening


class ForgeCastingPrefixTests(unittest.TestCase):
    def test_explicit_stop_at_settled_opening_has_no_gameplay_choices(self):
        path = ROOT / 'doc/evidence/forge-opening/source-seed-1/result.json.jsonl.gz'
        original = forge_opening.extract(path)
        endpoint = original['checkpoint_sources'][-1]
        converted = forge_casting.extract(path, stop_before=endpoint)
        self.assertIn('opening', converted, 'reuse the independently validated opening without rereading the entire source')
        self.assertEqual(converted['opening'], original['native'])
        expected = forge_casting_native.fixture(converted, original['native'])
        self.assertEqual(expected['actions'], 0)
        self.assertEqual(len(expected['operations']), 1)
        self.assertEqual(expected['operations'][0]['source_cursor'], endpoint)
        fixtures = json.loads((ROOT / 'fixtures/reference/forge-casting.json').read_text())
        self.assertEqual(len(fixtures), 5)
        self.assertEqual(fixtures[4], expected)

    def test_targeted_response_preserves_stack_and_boost(self):
        path = ROOT / 'doc/evidence/forge-casting/source-gg-11/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True)
        response = converted['prefix'][1004]
        self.assertEqual(response['arguments'][0]['host']['name'], 'Giant Growth')
        self.assertEqual([a['host']['name'] for a in response['state']['stack']], ['Bite Down'])
        expected = forge_casting_native.fixture(converted, converted['opening'])
        fixtures = json.loads((ROOT / 'fixtures/reference/forge-casting.json').read_text())
        self.assertEqual(fixtures[3], expected)
        points = [op['checkpoint'] for op in expected['operations'] if 'checkpoint' in op]
        self.assertTrue(any(len(p['stack']) == 2 for p in points))
        self.assertTrue(any([3, 3] in p['effects'].values() for p in points))

    def test_additional_discard_and_spell_created_tokens(self):
        path = ROOT / 'doc/evidence/forge-casting/source-rr-3/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True, stop_before=2984)
        expected = forge_casting_native.fixture(converted, converted['opening'])
        fixtures = json.loads((ROOT / 'fixtures/reference/forge-casting.json').read_text())
        self.assertEqual(fixtures[2], expected)
        actions = [op['action'] for op in expected['operations'] if 'action' in op]
        discards = [a for a in actions if a['kind'] == 'discard']
        self.assertEqual(len(discards), 1)
        self.assertTrue(all('/mountain/' in a['source'] for a in discards))
        self.assertTrue(any(k.startswith('token/') for op in expected['operations']
                            for k in op.get('checkpoint', {}).get('permanents', {})))

    def test_trigger_boundary_is_not_native_casting_acceptance(self):
        path = ROOT / 'doc/evidence/forge-casting/source-rr-2/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True)
        with self.assertRaisesRegex(ValueError, 'unsupported pending triggers'):
            forge_casting_native.fixture(converted, converted['opening'])

    def test_modal_source_selects_tokens_before_payment(self):
        path = ROOT / 'doc/evidence/forge-casting/source-rr-3/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True, stop_before=2984)
        expected = forge_casting_native.fixture(converted, converted['opening'])
        actions = [op['action'] for op in expected['operations'] if 'action' in op]
        cast = next(i for i, a in enumerate(actions) if a['kind'] == 'cast' and 'goblin-surprise' in a['source'])
        self.assertEqual(actions[cast + 1]['kind'], 'mode')
        self.assertEqual(actions[cast + 1]['mode'], 1)  # Frozen card: two Goblin tokens.
        self.assertEqual(actions[cast + 2]['kind'], 'tap_mana')

    def test_source_prefix_reaches_first_unsupported_combat_callback(self):
        result = forge_casting.extract(
            ROOT / 'doc/evidence/forge-opening/source-seed-1/result.json.jsonl.gz')
        self.assertEqual(result['endpoint']['method'], 'declareAttackers')
        self.assertEqual(result['endpoint']['index'], 1005)
        self.assertEqual(len(result['prefix']), 1005)
        self.assertFalse(result['complete_game'])
        self.assertGreater(result['remainder']['records'], 0)

    def test_creature_cast_and_nested_payment_are_retained(self):
        result = forge_casting.extract(
            ROOT / 'doc/evidence/forge-opening/source-seed-1/result.json.jsonl.gz')
        casts = [r for r in result['prefix'] if r.get('method') == 'playChosenSpellAbility']
        self.assertIn('Llanowar Elves', [r['arguments'][0]['host']['name'] for r in casts])
        elf = next(r for r in casts if r['arguments'][0]['host']['name'] == 'Llanowar Elves')
        self.assertEqual(result['paired'][elf['index']], 568)
        nested = result['prefix'][elf['index']:569]
        self.assertTrue(any(r.get('method') == 'payManaCost' for r in nested))
        self.assertTrue(any(r.get('event') == 'GameEventSpellAbilityCast' for r in nested))

    def test_native_fixture_is_exactly_derived_from_immutable_source(self):
        path = ROOT / 'doc/evidence/forge-opening/source-seed-1/result.json.jsonl.gz'
        converted = forge_casting.extract(path)
        expected = forge_casting_native.fixture(converted, converted['opening'])
        fixtures = json.loads((ROOT / 'fixtures/reference/forge-casting.json').read_text())
        self.assertEqual(fixtures[0], expected)
        self.assertEqual(expected['actions'], 75)
        self.assertEqual(sum('checkpoint' in op for op in expected['operations']), 65)

    def test_targeted_fixture_preserves_both_bite_down_targets(self):
        path = ROOT / 'doc/evidence/forge-casting/source-seed-3/result.json.jsonl.gz'
        converted = forge_casting.extract(path)
        opening = converted['opening']
        expected = forge_casting_native.fixture(converted, opening)
        fixtures = json.loads((ROOT / 'fixtures/reference/forge-casting.json').read_text())
        self.assertEqual(fixtures[1], expected)
        targets = [op['action'] for op in expected['operations'] if op.get('action', {}).get('kind') == 'target']
        self.assertEqual([t['role'] for t in targets], ['bite_source', 'bite_destination'])
        self.assertEqual([t['source'].split('/')[1] for t in targets], ['thornweald-archer', 'swab-goblin'])
        actions = [op['action'] for op in expected['operations'] if 'action' in op]
        cast = next(i for i, a in enumerate(actions) if a['kind'] == 'cast' and '/bite-down/' in a['source'])
        # Source mana lots name their producing abilities. Each basic generates
        # one unit, which Forge spends before activating the next source.
        self.assertEqual([a['kind'] for a in actions[cast + 4:cast + 9]],
                         ['tap_mana', 'pay', 'tap_mana', 'pay', 'finish_payment'])
        entry = next(r for r in converted['prefix']
                         if r.get('method') == 'playChosenSpellAbility' and
                         r['arguments'][0]['host']['name'] == 'Bite Down')
        committed = converted['prefix'][converted['paired'][entry['index']]]['arguments'][0]
        committed['paying_mana'].reverse()
        with self.assertRaisesRegex(ValueError, 'unsupported mana source/payment order'):
            forge_casting_native.fixture(converted, opening)
