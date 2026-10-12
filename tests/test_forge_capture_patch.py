"""Patch construction guards; these parser tests do not certify a source game."""
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import forge_capture_patch as patch


class ForgeCapturePatchTests(unittest.TestCase):
    def test_source_drift_rejected_before_instrumentation(self):
        with self.assertRaisesRegex(ValueError, 'pinned PlayerControllerAi source mismatch'):
            patch.instrument_controller('class PlayerControllerAi {}')

    def test_nested_generic_arguments_remain_single_parameters(self):
        self.assertEqual(patch.split_parameters(
            'Map<Card, List<Integer>> candidates, final int amount, String prompt /* hint */'),
            ['Map<Card, List<Integer>> candidates', 'final int amount', 'String prompt'])

    def test_java_literals_and_comments_do_not_end_method(self):
        method = '''{ String s = "}"; char c = '}'; /* } */
        // }
        if (true) { return; }
        }'''
        self.assertEqual(patch.matching_brace(method, 0), len(method) - 1)

    def test_missing_closing_brace_rejected(self):
        with self.assertRaisesRegex(ValueError, 'unterminated Java method'):
            patch.matching_brace('{ /* } */', 0)

    def test_event_hook_precedes_dispatch_without_replacing_it(self):
        source = '    public void fireEvent(final Event event) {\n        events.post(event);\n    }'
        result = patch.instrument_game(source)
        self.assertIn('MtgRawCapture.event(this, event);\n        events.post(event);', result)
        self.assertEqual(result.count('events.post(event);'), 1)
        with self.assertRaisesRegex(ValueError, 'event seam changed'):
            patch.instrument_game('')

    def test_shuffle_hook_preserves_original_shuffle_and_ai_choice(self):
        source = '''    public final void shuffle(final SpellAbility sa) {
        Collections.shuffle(list, MyRandom.getRandom());
        getZone(ZoneType.Library).setCards(getController().cheatShuffle(list));
    }'''
        result = patch.instrument_shuffle(source)
        self.assertLess(result.index('"before"'), result.index('Collections.shuffle'))
        self.assertGreater(result.index('"after"'), result.index('cheatShuffle(list)'))
        self.assertEqual(result.count('Collections.shuffle(list, MyRandom.getRandom());'), 1)
        self.assertEqual(result.count('getController().cheatShuffle(list)'), 1)
        with self.assertRaisesRegex(ValueError, 'shuffle seam changed'):
            patch.instrument_shuffle('')
