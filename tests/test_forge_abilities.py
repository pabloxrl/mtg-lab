"""L0-009 source boundaries; parser evidence supplements executable consumers.

CR 603.3 and the frozen Cyclops text require the recorded cast trigger to be
placed and resolved before the underlying Goblin Surprise can resolve.
"""
import copy
import gzip
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import forge_casting
import forge_casting_native
import forge_opening


class ForgeAbilitiesTests(unittest.TestCase):
    def test_actual_invoker_payment_target_and_temporary_trample(self):
        path = ROOT / 'doc/evidence/forge-abilities/source-rg-4/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True, abilities=True)
        fixture = forge_casting_native.fixture(converted, forge_opening.extract(path)['native'])
        activation = [op['action'] for op in fixture['operations']
                      if op['source_cursor'] == 8002 and 'action' in op]
        self.assertEqual(activation[0]['kind'], 'activate')
        self.assertIn('/wildheart-invoker/', activation[0]['source'])
        targets = [a for a in activation if a['kind'] == 'activation_target']
        self.assertEqual(len(targets), 1)
        self.assertIn('/druid-of-the-cowl/', targets[0]['source'])
        self.assertEqual(sum(a['kind'] == 'activation_pay' for a in activation), 8,
                         'frozen Invoker text requires eight generic mana')
        point = next(op['checkpoint'] for op in fixture['operations']
                     if op['source_cursor'] == 8159 and 'checkpoint' in op)
        druid = point['permanents'][targets[0]['source']]
        self.assertEqual((druid['power'], druid['toughness']), (6, 8),
                         'printed Druid 1/3 plus Invoker +5/+5')
        self.assertTrue(druid['trample'])
        self.assertEqual(point['effects'][targets[0]['source']], [5, 5])
        self.assertEqual(point['stack'], [])
        frozen = json.loads((ROOT / 'fixtures/reference/forge-abilities.json').read_text())
        self.assertEqual(frozen[3], fixture)
        bad = copy.deepcopy(converted)
        card = next(c for player in bad['prefix'][8159]['state']['players']
                    for c in player['zones']['Battlefield'] if c['keyword_changes'])
        card['keyword_changes'][0][2]['added'] = ['Flying']
        with self.assertRaisesRegex(ValueError, 'unsupported keyword grant'):
            forge_casting_native.fixture(bad, {})

    def test_actual_etb_target_and_multiple_cast_trigger_order(self):
        path = ROOT / 'doc/evidence/forge-abilities/source-rr-10/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True, abilities=True)
        fixture = forge_casting_native.fixture(converted, forge_opening.extract(path)['native'])
        actions = [op['action'] for op in fixture['operations'] if 'action' in op]
        targeted = [a for a in actions if a['kind'] == 'target_player']
        self.assertGreaterEqual(len(targeted), 3)
        self.assertEqual(targeted[0]['player'], 1)
        orders = [a['order'] for a in actions if a['kind'] == 'order_triggers']
        self.assertTrue(any(len(order) >= 4 for order in orders))
        # Pyromancer's printed ETB deals 2 to its chosen player, separately
        # from later non-targeting Archer cast triggers (CR 603/608/120).
        first = next(op['checkpoint'] for op in fixture['operations']
                     if op['source_cursor'] == 643 and 'checkpoint' in op)
        self.assertEqual(first['life'], [20, 18])
        self.assertEqual(first['stack'], [])

    def test_shivan_translation_excerpt_is_not_gameplay_acceptance(self):
        # Source rr-3 contains this activation after combat. This isolated
        # translation test cannot certify its earlier unsupported game prefix.
        path = ROOT / 'doc/evidence/forge-casting/source-rr-3/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True, abilities=True)
        with gzip.open(path, 'rt') as stream:
            rows = [json.loads(line) for line in stream]
        calls, paired, roots = [], {}, []
        for row in rows:
            if row['kind'] == 'enter':
                if not calls:
                    roots.append(row['index'])
                calls.append(row['index'])
            elif row['kind'] == 'leave':
                paired[calls.pop()] = row['index']
        play = 5341
        self.assertEqual(rows[play]['arguments'][0]['host']['name'], 'Shivan Dragon')
        before = max(i for i in roots if i < play and rows[i]['method'] == 'chooseSpellAbilityToPlay')
        after = min(i for i in roots if i > paired[play] and rows[i]['method'] == 'chooseSpellAbilityToPlay')
        converted.update(prefix=rows[:after], paired=paired, endpoint=rows[after], checkpoint_sources=[before])
        fixture = forge_casting_native.fixture(converted, forge_opening.extract(path)['native'])
        actions = [op['action'] for op in fixture['operations'] if 'action' in op]
        # Shivan's frozen {R}: +1/+0 ability uses the stack (CR 602/605).
        self.assertEqual([a['kind'] for a in actions],
                         ['activate', 'tap_mana', 'activation_pay', 'finish_activation'])
        self.assertEqual(actions[2]['color'], 3)
        self.assertEqual(fixture['operations'][-1]['checkpoint']['stack'][-1]['ability'], 'power')

        # Authored diagnostic continuation, not an AI game: CR 113.7a keeps
        # the activation's original source object after that card changes zone.
        departed = copy.deepcopy(converted)
        point = departed['endpoint']['state']
        source_id = rows[play]['arguments'][0]['host']['id']
        for player in point['players']:
            for card in list(player['zones']['Battlefield']):
                if card['id'] == source_id:
                    player['zones']['Battlefield'].remove(card)
                    card['zone'] = 'Graveyard'
                    player['zones']['Graveyard'].append(card)
        synthetic = forge_casting_native.fixture(departed, {})
        last = synthetic['operations'][-1]['checkpoint']
        source = last['stack'][-1]['source']
        self.assertEqual(last['stack'][-1]['incarnation'], 0,
                         'CR 113.7a retains the activated source incarnation')
        self.assertEqual(last['incarnations'][source], 1,
                         'CR 400.7 gives the graveyard card a new incarnation')

    def test_mana_creature_payment_retains_source_and_color(self):
        path = ROOT / 'doc/evidence/forge-abilities/source-gg-20/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True, abilities=True)
        fixture = forge_casting_native.fixture(converted, forge_opening.extract(path)['native'])
        actions = [op['action'] for op in fixture['operations'] if 'action' in op]
        taps = [i for i, a in enumerate(actions) if a['kind'] == 'tap_mana'
                and '/druid-of-the-cowl/' in a['source']]
        self.assertEqual(len(taps), 1)
        # Frozen Druid text / CR 605: tap adds one green without the stack.
        self.assertEqual(actions[taps[0] + 1]['kind'], 'pay')
        self.assertEqual(actions[taps[0] + 1]['color'], 4)
        frozen = json.loads((ROOT / 'fixtures/reference/forge-abilities.json').read_text())
        self.assertEqual(frozen[1], fixture)
        bad = copy.deepcopy(converted)
        cast = next(r for r in bad['prefix'] if r.get('method') == 'playChosenSpellAbility'
                    and r['index'] == 1328)
        paid = bad['prefix'][bad['paired'][cast['index']]]['arguments'][0]
        paid['paying_mana'][0]['source_ability'] += 10000
        with self.assertRaisesRegex(ValueError, 'mana source/payment order'):
            forge_casting_native.fixture(bad, forge_opening.extract(path)['native'])

    def test_cast_trigger_prefix_retains_placement_and_resolution(self):
        path = ROOT / 'doc/evidence/forge-casting/source-rr-2/result.json.jsonl.gz'
        result = forge_casting.extract(path, empty_combat=True, abilities=True)
        self.assertGreater(result['endpoint']['index'], 1990)
        methods = [r.get('method') for r in result['prefix']]
        self.assertIn('orderAndPlaySimultaneousSa', methods)
        self.assertIn('playSpellAbilityNoStack', methods)
        points = [r['state'] for r in result['prefix']
                  if r.get('method') == 'chooseSpellAbilityToPlay']
        self.assertTrue(any(c['name'] == 'Crackling Cyclops' and c['power'] >= 3
                            for s in points for p in s['players']
                            for c in p['zones']['Battlefield']))

    def test_native_choices_retain_cast_trigger_source_and_order(self):
        path = ROOT / 'doc/evidence/forge-casting/source-rr-2/result.json.jsonl.gz'
        converted = forge_casting.extract(path, empty_combat=True, abilities=True)
        fixture = forge_casting_native.fixture(converted, forge_opening.extract(path)['native'])
        orders = [op['action'] for op in fixture['operations']
                  if op.get('action', {}).get('kind') == 'order_triggers']
        self.assertEqual(len(orders), 1)
        self.assertEqual(orders[0]['order'][0]['ability'], 'cyclops')
        self.assertIn('/crackling-cyclops/', orders[0]['order'][0]['source'])
        points = [op['checkpoint'] for op in fixture['operations'] if 'checkpoint' in op]
        self.assertTrue(any(p['stack'][-1].get('ability') == 'trigger'
                            for p in points if p['stack']))
        # Frozen text: Cyclops starts 0/4, its cast trigger adds +3/+0,
        # then Goblin Surprise adds a separate +2/+0 on resolution.
        self.assertTrue(any(c['card'] == 'crackling-cyclops' and c['power'] == 3
                            and c['toughness'] == 4 and len(p['stack']) == 1
                            for p in points for c in p['permanents'].values()))
        self.assertTrue(any(c['card'] == 'crackling-cyclops' and c['power'] == 5
                            and c['toughness'] == 4 and not p['stack']
                            for p in points for c in p['permanents'].values()))
        frozen = json.loads((ROOT / 'fixtures/reference/forge-abilities.json').read_text())
        self.assertEqual(frozen[0], fixture)
        # Authored translation case only: source leaves between the cast event
        # and trigger placement. Hand -> stack -> battlefield is incarnation 2;
        # its new graveyard object is 3 (CR 400.7/113.7a).
        departed = copy.deepcopy(converted)
        root = next(r for r in departed['prefix']
                    if r.get('method') == 'orderAndPlaySimultaneousSa')
        host = root['arguments'][0][0]['host']
        initial = departed['prefix'][departed['checkpoint_sources'][-1]]['state']
        self.assertTrue(any(c['id'] == host['id'] for p in initial['players']
                            for c in p['zones']['Hand']))
        for player in root['state']['players']:
            for card in list(player['zones']['Battlefield']):
                if card['id'] == host['id']:
                    player['zones']['Battlefield'].remove(card)
                    card['zone'] = 'Graveyard'
                    card['timestamp'] += 100000
                    player['zones']['Graveyard'].append(card)
        synthetic = forge_casting_native.fixture(departed, {})
        order = next(o['action']['order'] for o in synthetic['operations']
                     if o.get('action', {}).get('kind') == 'order_triggers')
        self.assertEqual(order[0]['incarnation'], 2,
                         'the trigger retains its witnessed battlefield source')
        bad = copy.deepcopy(converted)
        forge_casting.corrupt(bad, 'trigger-order')
        with self.assertRaisesRegex(ValueError, 'trigger order permutation'):
            forge_casting_native.fixture(bad, forge_opening.extract(path)['native'])
