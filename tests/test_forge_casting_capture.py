"""Additive matchup selection preserves original capture validation/pins."""
from pathlib import Path
import sys
import unittest
import json
import shutil
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'scripts'))
import forge_ai_capture
import forge_casting_capture
import forge_opening


class ForgeCastingCaptureTests(unittest.TestCase):
    def test_selected_source_keeps_both_frozen_deck_rows(self):
        path = ROOT / 'doc/evidence/forge-casting/source-rr-2/result.json.jsonl.gz'
        result = forge_opening.extract(path)
        self.assertFalse(result['complete_game'])
        self.assertEqual([d['deck'] for d in result['native']['cases'][0]['decks']], ['red', 'red'])

    def test_selected_manifest_cannot_change_cards_even_with_updated_hash(self):
        source = ROOT / 'doc/evidence/forge-casting/source-rr-2'
        with tempfile.TemporaryDirectory() as temp:
            dest = Path(temp)
            for name in ('receipt.json', 'selected-manifest.json', 'result.json.jsonl.gz'):
                shutil.copyfile(source / name, dest / name)
            manifest = json.loads((dest / 'selected-manifest.json').read_text())
            manifest['decks'][0]['cards'][0]['copies'] += 1
            (dest / 'selected-manifest.json').write_text(json.dumps(manifest))
            receipt = json.loads((dest / 'receipt.json').read_text())
            receipt['artifacts']['selected-manifest.json'] = forge_opening.xmage.sha(dest / 'selected-manifest.json')
            (dest / 'receipt.json').write_text(json.dumps(receipt))
            with self.assertRaisesRegex(ValueError, 'unchanged frozen deck selection'):
                forge_opening.extract(dest / 'result.json.jsonl.gz')

    def test_original_source_has_identical_structural_validation(self):
        path = ROOT / 'doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz'
        self.assertEqual(forge_casting_capture.validate_trace(path, ['red', 'green']),
                         forge_ai_capture.validate_trace(path))

    def test_wrong_matchup_is_rejected_by_actual_opening_cards(self):
        path = ROOT / 'doc/evidence/forge-ai-capture/acceptance/game/result.json.jsonl.gz'
        with self.assertRaisesRegex(ValueError, 'frozen 40-card opening'):
            forge_casting_capture.validate_trace(path, ['green', 'green'])
