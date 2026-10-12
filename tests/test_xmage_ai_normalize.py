"""L0-002: source-derived choices, never native-generated expected states."""
import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import xmage_ai_normalize as normalize


class ActualCaptureNormalization(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.converted = normalize.normalize(ROOT / 'doc/evidence/xmage-ai-capture/acceptance-005/game/trace.jsonl.xz')
        cls.folder = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.folder.cleanup)
        cls.runs = normalize.run_native(Path(cls.folder.name))

    def test_complete_capture_preserves_opening_and_physical_choice(self):
        # Independently inspected immutable source records 40, 44, 64, 68,
        # 105 and 285. CR103.5 preserves the actual London choices; CR400.7
        # distinguishes each zone incarnation and each physical duplicate.
        result = self.converted
        case = result['transcript']['cases'][0]
        self.assertEqual([e['actor'] for e in case['chance']], [0, 1, 0])
        self.assertEqual([e['selection'] for e in result['source_opening_choices']],
                         ['mulligan', 'keep', ['0/mountain/14'], 'keep'])
        lands = [e for e in case['play'] if e['kind'] == 'play_land']
        self.assertEqual(lands[0]['source'], '0/mountain/8')
        self.assertEqual(case['play'][-1]['kind'], 'finish')
        self.assertEqual(len([e for e in case['play'] if e['kind'] == 'cast']), 15)
        self.assertEqual(result['transcript'], json.loads((ROOT / 'fixtures/reference/xmage-ai-normalized.json').read_text()))
        self.assertEqual(normalize.negative_inputs(result), json.loads((ROOT / 'fixtures/reference/xmage-ai-negative.json').read_text()))

    def test_required_source_semantics_are_not_omitted(self):
        for checkpoint in self.converted['source_checkpoints']:
            self.assertIn('exile', checkpoint['state'])
            self.assertIn('combat', checkpoint['state'])
        self.assertEqual(self.converted['outcome'], {'winner': 1, 'losses': ['Life', None]})
        self.assertEqual(len(self.converted['opening_checkpoints']), 4)

    def test_replay_twice_matches_source_checkpoints(self):
        for run in self.runs:
            normalize.verify_native(self.converted, run)

    def test_checkpoint_mutation_fails_at_first_source_field(self):
        run = copy.deepcopy(self.runs[0])
        run['points'][2]['life'][0] -= 1
        source = self.converted['source_cursors'][2]
        with self.assertRaisesRegex(ValueError, f'/source/{source}/checkpoint/action/2/life'):
            normalize.verify_native(self.converted, run)

    def test_missing_required_field_fails(self):
        run = copy.deepcopy(self.runs[0])
        del run['points'][0]['exile']
        with self.assertRaisesRegex(ValueError, 'checkpoint/action/0/exile missing'):
            normalize.verify_native(self.converted, run)

    def test_opening_shuffle_checkpoint_mutation_fails(self):
        run = copy.deepcopy(self.runs[0])
        library = run['opening']['points'][0]['library']
        library[0], library[1] = library[1], library[0]
        with self.assertRaisesRegex(ValueError, 'opening checkpoint'):
            normalize.verify_native(self.converted, run)

    def test_actual_illegal_inputs_fail_at_their_cursors(self):
        actual = json.loads((Path(self.folder.name) / 'native-negatives.json').read_text())
        for name, spec in normalize.negative_inputs(self.converted).items():
            self.assertIn(name, actual)
            if spec['sequence'] is not None:
                self.assertEqual(len(actual[name]['run']['consumed_play']), spec['sequence'])
                self.assertIn(f"/play/{spec['sequence']} ", actual[name]['error'])

    def test_capture_receipt_pins_and_tape_are_required(self):
        source = ROOT / 'doc/evidence/xmage-ai-capture/acceptance-005/game/trace.jsonl.xz'
        receipt = json.loads(source.with_name('attempt.json').read_text())
        receipt['artifacts']['trace.jsonl.xz'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'source tape hash'):
            normalize.validate_provenance(source, receipt)
        receipt = json.loads(source.with_name('attempt.json').read_text())
        receipt['pins']['data/cards/foundations_micro_v1.json'] = '0' * 64
        with self.assertRaisesRegex(ValueError, 'source pin'):
            normalize.validate_provenance(source, receipt)

    def reference_witness(self):
        import hashlib
        import lzma
        folder = ROOT / 'doc/evidence/xmage-ai-normalize'
        meta = json.loads((folder / 'witness.json').read_text())
        raw = (folder / meta['path']).read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(), meta['sha256'])
        decoded = lzma.decompress(raw)
        self.assertEqual(hashlib.sha256(decoded).hexdigest(), meta['json_sha256'])
        return json.loads(decoded)

    def test_actual_reference_witness_agrees_with_fresh_native(self):
        normalize.verify_pair(self.converted, self.runs[0], self.reference_witness())

    def test_departure_observer_binds_the_old_incarnation(self):
        witness = self.reference_witness()
        departure = next(d for d in witness['points'][-1]['departures']
                         if d['source'] == '0/crackling-cyclops/2')
        # CR400.7: hand -> stack -> battlefield gave incarnation 3; the
        # recorded terminal death creates 4, which cannot be its own source.
        self.assertEqual(departure['departing_incarnation'], 3)
        departure['departing_incarnation'] = 4
        with self.assertRaisesRegex(ValueError, 'checkpoint/450/departure identity'):
            normalize.verify_pair(self.converted, self.runs[0], witness)


class TerminalReasonNormalization(unittest.TestCase):
    def test_empty_draw_uses_existing_replay_loss_reason(self):
        # CR104.3c and the independently authored admission deck-out cases:
        # this is a loss, distinct from a natural rules draw.
        self.assertEqual(normalize.loss_reason({'life_loss': False, 'empty_draw': True}, 0), 'EmptyDraw')
