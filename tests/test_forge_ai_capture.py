"""Reject incomplete raw evidence; only the retained JVM receipts certify execution.

Expectations follow L0-006's complete choices/chance/provenance contract and
CR 103's frozen deck/opening obligations, not source-generated golden outcomes.
"""
import copy
import hashlib
import gzip
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'scripts'))
import forge_ai_capture as capture

TRACE = ROOT/'doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz'


class ForgeRawCaptureTests(unittest.TestCase):
    def mutate(self, change, expected, select=lambda r: True):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory)/'bad.jsonl.gz'
            changed = False
            with gzip.open(TRACE, 'rt') as source, gzip.open(target, 'wt', compresslevel=1) as out:
                for line in source:
                    row = json.loads(line)
                    if not changed and select(row):
                        changed = True
                        change(row)
                        out.write(json.dumps(row, separators=(',', ':'))+'\n')
                        # Keep the entire unchanged prefix and stop after the
                        # defect. The required specific diagnostic means generic
                        # missing-footer rejection cannot satisfy this test.
                        break
                    out.write(line)
            self.assertTrue(changed, 'mutation must reach its semantic target')
            with self.assertRaisesRegex(ValueError, expected):
                capture.validate_trace(target)

    def test_actual_game_has_normal_opening_mulligans_and_played_witnesses(self):
        result = capture.validate_trace(TRACE)
        self.assertGreater(result['callbacks']['tuckCardsViaMulligan'], 0)
        self.assertGreater(result['callbacks']['cost.visit'], 0)
        self.assertGreater(result['kinds']['GameEventSpellResolved'], 0)
        self.assertGreater(result['kinds']['GameEventBlockersDeclared'], 0)
        self.assertGreater(result['kinds']['random'], 0)

    def test_receipt_binds_the_actual_source_game_and_implementation(self):
        receipt = json.loads((TRACE.parent/'receipt.json').read_text())
        self.assertEqual(receipt['status'], 'complete')
        self.assertTrue(receipt['started_game'])
        self.assertEqual(hashlib.sha256(TRACE.read_bytes()).hexdigest(),
                         receipt['artifacts'][TRACE.name])
        for relative, local in [(capture.RECORDER, 'MtgRawCapture.java'),
                                (capture.TEST, 'RawAiCaptureTest.java')]:
            self.assertEqual(hashlib.sha256((ROOT/'references/forge'/local).read_bytes()).hexdigest(),
                             receipt['patches'][relative]['modified'])

    def test_unknown_schema_is_rejected(self):
        self.mutate(lambda r: r.update(schema_version=999), 'unsupported schema')

    def test_missing_pending_trigger_queue_is_rejected(self):
        self.mutate(lambda r: r['state'].pop('pending_triggers'), 'state fields')

    def test_unknown_sba_boundary_is_rejected(self):
        self.mutate(lambda r: r.update(boundary='invented'), 'unknown rules boundary',
                    lambda r: r['kind']=='boundary')

    def test_unknown_callback_is_not_an_implicit_default(self):
        self.mutate(lambda r: r.update(method='silentlyChooseSomething'),
                    'unknown callback', lambda r: r['kind']=='enter')

    def test_missing_callback_return_is_rejected(self):
        self.mutate(lambda r: r.pop('result'), 'callback return', lambda r: r['kind']=='leave')

    def test_wrong_callback_pairing_is_rejected(self):
        self.mutate(lambda r: r.update(call=999999), 'unmatched callback', lambda r: r['kind']=='leave')

    def test_missing_ordered_zone_is_rejected(self):
        self.mutate(lambda r: r['state']['players'][0]['zones'].pop('Library'), 'ordered zones')

    def test_changed_deck_multiset_is_rejected(self):
        self.mutate(lambda r: r['library'].pop(), 'frozen 40-card opening',
                    lambda r: r['kind']=='shuffle' and r['stage']=='before')

    def test_duplicate_shuffle_physical_identity_is_rejected(self):
        def duplicate(row):
            row['library'][1] = copy.deepcopy(row['library'][0])
        self.mutate(duplicate, 'duplicate physical', lambda r: r['kind']=='shuffle')

    def test_missing_event_payload_is_rejected(self):
        self.mutate(lambda r: r.pop('payload'), 'event payload', lambda r: r['kind']=='event')

    def test_missing_chance_draw_is_rejected(self):
        self.mutate(lambda r: r.update(draw=999999), 'random draw', lambda r: r['kind']=='random')

    def test_missing_payment_provenance_is_rejected(self):
        def change(row):
            row['arguments'][0].pop('paying_mana')
        self.mutate(change, 'ability choices/payment',
                    lambda r: r['kind']=='enter' and r['method']=='playChosenSpellAbility')

    def test_missing_temporary_effects_is_rejected(self):
        def change(row):
            row['library'][0].pop('pt_boosts')
        self.mutate(change, 'card fields/effects', lambda r: r['kind']=='shuffle')

    def test_terminal_without_rules_reason_is_rejected(self):
        self.mutate(lambda r: r['state']['players'][0].update(outcome=None), 'missing terminal reasons',
                    lambda r: r['kind']=='finish')

    def test_concession_does_not_count_as_normal_game(self):
        def change(row):
            loser = next(p for p in row['state']['players'] if not p['outcome']['won'])
            loser['outcome']['loss'] = 'Conceded'
        self.mutate(change, 'nonnormal termination', lambda r: r['kind']=='finish')

    def test_truncation_cannot_be_certified_as_complete(self):
        self.mutate(lambda r: r.update(status='truncated'), 'noncomplete footer',
                    lambda r: r['kind']=='finish')

    def test_record_count_detects_dropped_records(self):
        self.mutate(lambda r: r.update(record_count=1), 'footer completeness counts',
                    lambda r: r['kind']=='finish')


class ForgeSourceDiagnosticTests(unittest.TestCase):
    def test_swallowed_timed_ai_failure_is_not_a_complete_game(self):
        self.assertIsNotNone(capture.source_diagnostic(
            'java.util.concurrent.TimeoutException\n    at java.util.concurrent.FutureTask.get(FutureTask.java:204)\n'))

    def test_swallowed_worker_exception_is_not_a_complete_game(self):
        self.assertIsNotNone(capture.source_diagnostic(
            'java.util.concurrent.ExecutionException: java.lang.IllegalStateException\n'))

    def test_normal_maven_summary_is_not_a_source_failure(self):
        self.assertIsNone(capture.source_diagnostic('[INFO] No tests to run.\n[INFO] BUILD SUCCESS\n'))
