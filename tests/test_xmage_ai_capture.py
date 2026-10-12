"""L0-001 capture acceptance, based on the registered capture contract.

The live check is also callable as the explicit acceptance command. Lightweight
tests in ordinary discovery protect attempt persistence and fail-closed behavior.
"""
import sys
import gzip
import json
import lzma
import hashlib
import copy
import subprocess
from unittest.mock import patch
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts"))
import xmage_ai_capture as capture

EVIDENCE = capture.ROOT / 'doc/evidence/xmage-ai-capture/acceptance-004/game/trace.jsonl.xz'


class CaptureTests(unittest.TestCase):
    def test_retained_live_evidence_checksums(self):
        subprocess.run([sys.executable, str(capture.ROOT / 'doc/evidence/xmage-ai-capture/check_evidence.py')],
                       check=True, capture_output=True, text=True)

    def test_actual_source_game_has_complete_capture(self):
        # Captured source inputs, not a gameplay golden: these floors come from
        # L0-001. Native/source rules agreement belongs to later admission.
        result = capture.validate_trace(EVIDENCE)
        self.assertGreater(result['played_spells'], 0)
        self.assertGreater(result['attacking_combats'], 0)
        self.assertGreaterEqual(result['mulligan_decisions'], 2)

    def test_first_missing_semantic_field_is_rejected(self):
        with lzma.open(EVIDENCE, 'rt') as source:
            first = json.loads(next(source))
        for field in ('effects', 'stack', 'combat', 'pending_triggers', 'exile'):
            bad = copy.deepcopy(first)
            del bad['state'][field]
            with self.subTest(field=field):
                self.reject_prefix([bad], 'semantic observation fields')

    def test_simulation_and_card_incarnation_omissions_are_rejected(self):
        with lzma.open(EVIDENCE, 'rt') as source:
            first = json.loads(next(source))
        bad = copy.deepcopy(first)
        bad['simulation'] = True
        self.reject_prefix([bad], 'simulation branch')
        bad = copy.deepcopy(first)
        del bad['state']['cards'][0]['zcc']
        self.reject_prefix([bad], 'characteristics/incarnation')

    def test_unknown_callback_cannot_become_supported_by_echoing_it(self):
        with lzma.open(EVIDENCE, 'rt') as source:
            first = json.loads(next(source))
        unknown = copy.deepcopy(first)
        unknown.update(sequence=1, kind='call/unknownFutureDecision')
        self.reject_prefix([first, unknown], 'unsupported callback')

    def test_missing_permanent_incarnation_is_rejected(self):
        with lzma.open(EVIDENCE, 'rt') as source:
            first = json.loads(next(source))
            point = next(p for p in map(json.loads, source) if p['state']['permanents'])
        point.update(sequence=1, kind='event/before', detail={'type': 'OBSERVATION_PROBE'})
        del point['state']['permanents'][0]['zcc']
        self.reject_prefix([first, point], 'permanent characteristics/incarnation')

    def test_source_modes_have_identity_to_effect_bindings(self):
        # L0-001 requires modes, not just unresolvable source-generated UUIDs.
        with lzma.open(EVIDENCE, 'rt') as source:
            first = json.loads(next(source))
        abilities = [a for c in first['state']['cards'] for a in c['abilities']]
        self.assertTrue(abilities)
        for ability in abilities:
            self.assertIn('available_modes', ability)
            modes = ability['available_modes']
            for mode in modes:
                self.assertEqual(mode['key'], mode['value']['id'])
                self.assertIn('effects', mode['value'])
        bad = copy.deepcopy(first)
        del bad['state']['cards'][0]['abilities'][0]['available_modes']
        self.reject_prefix([bad], 'ability choice fields')

    def test_missing_callback_payloads_are_rejected(self):
        # Source callback signatures require arguments and returned decisions;
        # the review reproduced acceptance of an entirely erased choice tape.
        with lzma.open(EVIDENCE, 'rt') as source:
            points = []
            for line in source:
                point = json.loads(line)
                points.append(point)
                if point['kind'].startswith('call/'):
                    point['detail'] = None
                    break
        self.reject_prefix(points, 'callback payload')

    def test_callback_results_targets_and_payments_cannot_disappear(self):
        wanted = {'return/chooseMulligan', 'return/chooseTarget', 'return/playMana'}
        samples = {}
        opening = []
        with lzma.open(EVIDENCE, 'rt') as source:
            first = json.loads(next(source))
            for point in map(json.loads, source):
                if 'return/chooseMulligan' in wanted and point['kind'] != 'return/chooseMulligan':
                    opening.append(point)
                if point['kind'] in wanted:
                    samples[point['kind']] = point
                    wanted.remove(point['kind'])
                if not wanted:
                    break
        self.assertFalse(wanted)
        for kind, point in samples.items():
            before = copy.deepcopy(point)
            before.update(sequence=1, kind='call/' + kind[7:], detail=point['detail']['arguments'])
            prefix = [first] + opening if kind == 'return/chooseMulligan' else [first, before]
            for change in ('erase', 'arguments', 'result', 'nested'):
                bad = copy.deepcopy(point)
                bad['sequence'] = len(prefix)
                if change == 'erase':
                    bad['detail'] = None
                elif change in ('arguments', 'result'):
                    del bad['detail'][change]
                elif kind == 'return/chooseMulligan':
                    bad['detail']['result'] = None
                elif kind == 'return/chooseTarget':
                    del bad['detail']['arguments'][1]['targets']
                else:
                    del bad['detail']['arguments'][1]['payment']
                with self.subTest(kind=kind, change=change):
                    self.reject_prefix(prefix + [bad], 'callback payload')

    def reject_prefix(self, points, reason):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'trace.jsonl.gz'
            with gzip.open(path, 'wt') as output:
                for point in points:
                    output.write(json.dumps(point) + '\n')
            with self.assertRaisesRegex(ValueError, reason):
                capture.validate_trace(path)

    def test_storage_preserves_original_source_bytes(self):
        original = b'{"choice":"pass","actor":0}\n{"choice":null}\n'
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'trace.jsonl.gz'
            with gzip.open(path, 'wb') as stream:
                stream.write(original)
            result = capture.pack_trace(path)
            with lzma.open(Path(temp) / result['path'], 'rb') as stream:
                self.assertEqual(stream.read(), original)
            self.assertEqual(result['jsonl_sha256'], hashlib.sha256(original).hexdigest())

    def test_acceptance_rejects_wrong_negative_reason(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def fake_capture(cache, output, seed, **options):
                output.mkdir()
                (output / 'attempt.json').write_text('{}')
                return dict(status='unsupported' if options.get('unsupported_probe') else
                            'truncated' if options.get('max_records') else 'complete',
                            error='unrelated failure')
            with patch.object(capture, 'capture', side_effect=fake_capture):
                result = capture.acceptance(root / 'cache', root / 'run')
            self.assertEqual(result['status'], 'failed')

    def test_empty_trace_cannot_certify_gameplay(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'trace.jsonl.gz'
            with gzip.open(path, 'wt') as stream:
                stream.write('')
            with self.assertRaisesRegex(ValueError, 'configuration'):
                capture.validate_trace(path)

    def test_outcome_only_is_not_complete_capture(self):
        with tempfile.TemporaryDirectory() as temp:
            path = Path(temp) / 'trace.jsonl.gz'
            with gzip.open(path, 'wt') as stream:
                stream.write(json.dumps(dict(sequence=0, kind='terminal', state={'ended': True})) + '\n')
            with self.assertRaisesRegex(ValueError, 'configuration'):
                capture.validate_trace(path)

    def test_missing_source_is_a_retained_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            result = capture.capture(root / "missing", root / "attempt", seed=319)
            self.assertEqual(result["status"], "failed")
            self.assertEqual(result["seed"], 319)
            self.assertTrue((root / "attempt" / "attempt.json").is_file())

    def test_attempt_cannot_be_overwritten(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            capture.capture(root / "missing", root / "attempt")
            original = (root / "attempt" / "attempt.json").read_bytes()
            with self.assertRaises(FileExistsError):
                capture.capture(root / "missing", root / "attempt", seed=2)
            self.assertEqual((root / "attempt" / "attempt.json").read_bytes(), original)
