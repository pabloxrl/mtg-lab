"""L0-012 accounting controls; synthetic variants are never gameplay evidence."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import forge_ai_pilot as pilot
import forge_combat_reference
import forge_casting
import forge_casting_native


class ForgePilotAccounting(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        fixture = json.loads((ROOT / 'fixtures/reference/forge-combat.json').read_text())[0]
        cls.transcript = forge_combat_reference.transcript(fixture)

    def attempt(self, name='original'):
        return dict(id=name, source='immutable-source', status='qualified',
                    transcript=copy.deepcopy(self.transcript))

    def test_ten_replays_with_different_seeds_and_provenance_are_one_game(self):
        attempts = []
        for seed in range(10):
            attempt = self.attempt(str(seed))
            attempt['source'] = 'different-path-' + str(seed)
            attempt['transcript']['provenance'] = dict(engine='forge', seed=seed)
            attempt['transcript']['cases'][0]['id'] = 'renamed-' + str(seed)
            attempts.append(attempt)
        result = pilot.inventory(attempts)
        self.assertEqual(len(result['qualified']), 1)
        self.assertEqual(len(result['attempts']), 10)
        self.assertTrue(all(a['duplicate_of'] == '0' for a in result['attempts'][1:]))
        with self.assertRaisesRegex(ValueError, 'ten distinct'):
            pilot.require_pilot(attempts)

    def test_unsuccessful_attempts_remain_and_never_fill_quota(self):
        attempts = [self.attempt()]
        for status in ('failed', 'truncated', 'unsupported', 'divergent', 'duplicate'):
            attempt = self.attempt(status)
            attempt.update(status=status, earliest_difference='recorded first failure')
            attempts.append(attempt)
        result = pilot.inventory(attempts)
        self.assertEqual(len(result['qualified']), 1)
        self.assertEqual(len(result['attempts']), 6)
        attempts[-1].pop('earliest_difference')
        with self.assertRaisesRegex(ValueError, 'first difference'):
            pilot.inventory(attempts)

    def test_missing_terminal_and_duplicate_attempt_ids_reject(self):
        attempt = self.attempt()
        attempt['transcript']['cases'][0]['stop'] = 'first_upkeep'
        with self.assertRaisesRegex(ValueError, 'rules-terminal'):
            pilot.inventory([attempt])
        with self.assertRaisesRegex(ValueError, 'unique nonempty'):
            pilot.inventory([self.attempt(), self.attempt()])

    def test_distinctness_does_not_replace_eight_row_coverage(self):
        # Artificial content mutations test accounting only, not legal games.
        attempts = []
        for index in range(10):
            attempt = self.attempt(str(index))
            attempt['transcript']['cases'][0]['play'][0]['turn'] = index + 1
            attempts.append(attempt)
        self.assertEqual(len(pilot.inventory(attempts)['qualified']), 10)
        with self.assertRaisesRegex(ValueError, 'eight matchup'):
            pilot.require_pilot(attempts)

    def test_actual_truncated_capture_retains_failure_without_qualification(self):
        source = ROOT / 'doc/evidence/forge-ai-capture/acceptance/truncated/result.json.jsonl.gz'
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'attempt'
            result = pilot.execute(source, output, Path(temporary) / 'no-forge',
                                   Path(temporary) / 'no-xmage')
            self.assertEqual(result['status'], 'failed')
            self.assertIn('source completion', result['earliest_difference'])
            self.assertFalse(result['final_admission'])
            self.assertNotIn('identity', result)
            self.assertEqual(json.loads((output / 'receipt.json').read_text()), result)
            with self.assertRaises(FileExistsError):
                pilot.execute(source, output, 'unused', 'unused')


class RepeatedActivationPayment(unittest.TestCase):
    def test_previous_activation_payment_is_not_paid_again(self):
        # CR602.2b/601.2h: each Dragon activation costs one red mana. The
        # immutable source's second activation starts at 13726, taps physical
        # Mountain 56 at event 13741, and pays one red lot from ability 163.
        # Ability 160 / Mountain 49 was paid on the PREVIOUS activation.
        source = ROOT / 'doc/evidence/forge-casting/source-rr-2/result.json.jsonl.gz'
        converted = forge_casting.extract(source, abilities=True, full_game=True)
        fixture = forge_casting_native.fixture(converted, converted['opening'])
        actions = [op['action'] for op in fixture['operations']
                   if op['source_cursor'] == 13726 and 'action' in op]
        taps = [a['source'] for a in actions if a['kind'] == 'tap_mana']
        self.assertEqual(taps, [converted['bindings'][56]])
        self.assertEqual([a['color'] for a in actions if a['kind'] == 'activation_pay'], [3])
        # A history mismatch cannot be silently sliced away to obtain agreement.
        bad = dict(converted, prefix=list(converted['prefix']))
        end = converted['paired'][13726]
        bad['prefix'][end] = copy.deepcopy(converted['prefix'][end])
        bad['prefix'][end]['arguments'][0]['paying_mana_abilities'][0]['id'] += 1
        with self.assertRaisesRegex(ValueError, '/source/13726/mana payment history changed'):
            forge_casting_native.fixture(bad, bad['opening'])
        bad['prefix'][end] = copy.deepcopy(converted['prefix'][end])
        bad['prefix'][end]['arguments'][0]['paying_mana_abilities'].pop()
        with self.assertRaisesRegex(ValueError, '/source/13726/unsupported floating or multi-unit mana payment'):
            forge_casting_native.fixture(bad, bad['opening'])


class RetainedForgePilot(unittest.TestCase):
    def test_all_ten_source_bound_games_replay_twice_natively(self):
        import check_forge_pilot as check
        manifest = check.replay_native()
        self.assertEqual(len(manifest['games']), 10)
        self.assertEqual({(g['row'], g['starter']) for g in manifest['games']},
                         {(row, seat) for row in ('RG', 'GR', 'RR', 'GG') for seat in (0, 1)})

    def test_a_failed_attempt_cannot_disappear_from_the_published_ledger(self):
        import check_forge_pilot as check
        manifest = json.loads(check.DEFAULT.read_text())
        index = check.document(manifest, manifest['attempt_index'])
        archived = {entry['path']: entry for entry in index['files']}
        omitted = next(a for a in manifest['attempts']
                       if a['kind'] == 'execution' and a['status'] == 'failed')
        manifest['attempts'].remove(omitted)
        with self.assertRaisesRegex(ValueError, 'complete attempt ledger'):
            check.check_attempt_receipts(manifest, archived)

    def test_changed_artifact_or_path_cannot_reuse_execution_evidence(self):
        import check_forge_pilot as check
        manifest = json.loads(check.DEFAULT.read_text())
        name = manifest['games'][0]['receipt']
        changed = copy.deepcopy(manifest)
        changed['artifacts'][name] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'artifact checksum'):
            check.document(changed, name)
        for name in ('../outside.json', '/tmp/outside.json'):
            with self.assertRaisesRegex(ValueError, 'artifact path'):
                check.artifact(manifest, name)


if __name__ == '__main__':
    unittest.main()
