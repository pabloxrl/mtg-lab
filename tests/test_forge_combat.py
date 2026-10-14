"""L0-010: immutable actual-AI game consumption, not gameplay certification.

CR 508/509/510 require declarations and exact damage choices; CR 704.5a
requires the observed life-loss ending. Live consumers supply execution proof.
"""
import copy
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import forge_casting
import forge_casting_native
import admission_observations


class ForgeCombatTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        source = ROOT / 'doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz'
        cls.converted = forge_casting.extract(source, abilities=True, full_game=True)

    @classmethod
    def tearDownClass(cls):
        cls.converted = None

    def test_complete_actual_game_has_no_unconsumed_suffix(self):
        converted = self.converted
        self.assertTrue(converted['complete_game'], 'combat must reach the actual rules ending')
        self.assertEqual(converted['remainder']['records'], 0)
        self.assertTrue(converted['prefix'][-1]['state']['terminal'])

    def test_actual_deathtouch_trample_and_ordered_deaths(self):
        fixture = forge_casting_native.fixture(self.converted, self.converted['opening'])
        actions = [op['action'] for op in fixture['operations'] if 'action' in op]
        self.assertTrue({'attackers', 'blockers', 'damage', 'graveyard_order', 'finish'} <=
                        {a['kind'] for a in actions})
        # CR 510.1 and the printed Pathwarden/Cyclops/token characteristics:
        # five power is assigned as four to Cyclops and one to the Goblin.
        damage = next(op['action'] for op in fixture['operations']
                      if op['source_cursor'] == 5535 and 'action' in op)
        group = damage['selection'][0]
        self.assertIn('/tajuru-pathwarden/', group['attacker']['source'])
        self.assertEqual(sorted(a['amount'] for a in group['amounts']), [1, 4])
        self.assertTrue(group['amounts'][0]['target']['source'].startswith('token/'))
        self.assertIn('/crackling-cyclops/', group['amounts'][1]['target']['source'],
                      'canonical recipient slots follow the recorded blocker list, not map iteration')
        # CR 111.6/404.3: tokens do not occupy ordered graveyard positions.
        for action in actions:
            if action['kind'] == 'graveyard_order':
                self.assertGreater(len(action['selection']), 1)
                self.assertTrue(all(not c['source'].startswith('token/') for c in action['selection']))
        terminal = fixture['operations'][-1]['checkpoint']
        self.assertEqual(terminal['outcome'], dict(winner=1, losses=['Life', None]))
        self.assertLessEqual(terminal['life'][0], 0, 'CR 704.5a life loss')
        self.assertEqual(actions[-1]['kind'], 'finish')
        cleanups = fixture['auxiliary']['cleanup_checkpoints']
        self.assertEqual([p['checkpoint']['turn'] for p in cleanups], list(range(1, 29)))
        for point in cleanups:
            self.assertEqual(point['checkpoint']['effects'], {}, 'CR 514.2 expires temporary boosts')
            self.assertTrue(all(not c['damage'] for c in point['checkpoint']['permanents'].values()),
                            'CR 514.2 removes marked damage at the same cleanup boundary')
        self.assertEqual(len(fixture['auxiliary']['marked']), 13)
        self.assertEqual(json.loads((ROOT / 'fixtures/reference/forge-combat.json').read_text()), [fixture])

    def test_changed_allocation_total_fails_at_its_source_choice(self):
        converted = dict(self.converted, prefix=list(self.converted['prefix']))
        index = converted['paired'][2171]
        converted['prefix'][index] = copy.deepcopy(converted['prefix'][index])
        converted['prefix'][index]['result'][0][1] += 1
        with self.assertRaisesRegex(ValueError, '/source/2171/exact damage total'):
            forge_casting_native.fixture(converted, converted['opening'])

    def test_full_game_cannot_select_a_prefix_or_omit_abilities(self):
        for options in [dict(full_game=True), dict(full_game=True, abilities=True, stop_before=10)]:
            with self.assertRaisesRegex(ValueError, 'full game requires abilities and no prefix endpoint'):
                forge_casting.extract(Path('unused'), **options)


class AuthoredCleanupChoiceTests(unittest.TestCase):
    def test_exact_excess_distinct_current_hand_occurrences(self):
        # Authored controller-boundary regression, not a captured AI game.
        hand = [dict(id=i, timestamp=100+i) for i in range(9)]
        row = dict(index=0, actor=dict(player_id=0), arguments=[2], state=dict(
            phase='CLEANUP', active=dict(player_id=0),
            players=[dict(zones=dict(Hand=hand)), dict(zones=dict(Hand=[]))]))
        self.assertEqual(forge_casting_native.cleanup_selection(row, hand[:2]), hand[:2])
        for selected in [hand[:1], hand[:3], [hand[0], hand[0]],
                         [hand[0], dict(hand[1], timestamp=999)]]:
            with self.assertRaisesRegex(ValueError, 'cleanup discard occurrences'):
                forge_casting_native.cleanup_selection(row, selected)
        for changed in [dict(row, arguments=[1]),
                        dict(row, state=dict(row['state'], phase='END_OF_TURN')),
                        dict(row, actor=dict(player_id=1))]:
            with self.assertRaisesRegex(ValueError, 'cleanup discard context/count'):
                forge_casting_native.cleanup_selection(changed, hand[:2])


class MulliganObservationTests(unittest.TestCase):
    def test_later_shuffle_replaces_earlier_bottom_order(self):
        # CR 103.5: a further mulligan shuffles the entire hand back. Only
        # bottoming after the final shuffle determines the surviving suffix.
        fixture = json.loads((ROOT / 'fixtures/reference/forge-combat.json').read_text())[0]
        case = fixture['opening']['cases'][0]
        point = copy.deepcopy(fixture['operations'][0]['checkpoint'])
        point.update(creations=[], effects=[])
        self.assertEqual(point['library'][0][-2:], ['0/mountain/11', '0/shivan-dragon/1'])
        self.assertNotEqual(point['library'][0][-3], '0/swab-goblin/1',
                            'first-mulligan bottom was reshuffled')
        admission_observations.structure(point, case, 'native', 0, '/opening')
        point['library'][0][-2:] = reversed(point['library'][0][-2:])
        with self.assertRaisesRegex(ValueError, 'library occurrence order'):
            admission_observations.structure(point, case, 'native', 0, '/opening')


if __name__ == '__main__':
    unittest.main()
