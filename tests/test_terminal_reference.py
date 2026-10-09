"""Independent comparator negative controls for terminal observations."""
import copy
import json
from pathlib import Path
import sys
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import terminal_reference


class TerminalReferenceTests(unittest.TestCase):
    def test_missing_extra_and_every_observed_field_rejected(self):
        fixture = json.loads(terminal_reference.FIXTURE.read_text())
        expected = {c['id']: c['expected'] for c in fixture['cases']}
        terminal_reference.compare(fixture, expected)
        for case in expected:
            wrong = copy.deepcopy(expected)
            del wrong[case]
            with self.assertRaisesRegex(ValueError, 'missing or extra'):
                terminal_reference.compare(fixture, wrong)
            for field in ['life', 'lost', 'library', 'hand']:
                for seat in [0, 1]:
                    wrong = copy.deepcopy(expected)
                    value = wrong[case][field][seat]
                    wrong[case][field][seat] = not value if field == 'lost' else value + 1
                    with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
                        terminal_reference.compare(fixture, wrong)
        wrong = copy.deepcopy(expected)
        wrong['extra'] = expected['zero-life']
        with self.assertRaisesRegex(ValueError, 'missing or extra'):
            terminal_reference.compare(fixture, wrong)

    def test_unknown_input_cannot_be_ignored(self):
        fixture = json.loads(terminal_reference.FIXTURE.read_text())
        fixture['cases'][0]['draw_seet'] = 1
        expected = {c['id']: c['expected'] for c in fixture['cases']}
        with self.assertRaisesRegex(ValueError, 'unknown'):
            terminal_reference.compare(fixture, expected)

    def test_boolean_is_not_integer_observation(self):
        fixture = json.loads(terminal_reference.FIXTURE.read_text())
        expected = {c['id']: c['expected'] for c in fixture['cases']}
        wrong = copy.deepcopy(expected)
        wrong['zero-life']['library'][0] = False
        with self.assertRaisesRegex(ValueError, 'checkpoint mismatch'):
            terminal_reference.compare(fixture, wrong)

    def test_v2_literal_outcomes_and_legacy_preservation(self):
        legacy = json.loads(terminal_reference.FIXTURE.read_text())
        current = json.loads(terminal_reference.V2_FIXTURE.read_text())
        terminal_reference.validate(current)
        by_id = {c['id']: c for c in current['cases']}
        for old in legacy['cases']:
            new = by_id[old['id']]
            for field in ['id', 'life', 'action', 'library_card']:
                self.assertEqual(old.get(field), new.get(field))
            for field, value in old['expected'].items():
                self.assertEqual(value, new['expected'][field])
        for seat in [0, 1]:
            for order in ['life_then_draw', 'draw_then_life']:
                c = by_id[f'mixed-life-p{1-seat}-draw-p{seat}-{order}']
                self.assertEqual(c['expected']['lost'], [True, True])
                self.assertEqual(c['expected']['outcome'], 'draw')
                self.assertIsNone(c['expected']['winner'])
                self.assertTrue(c['expected']['draw'])
                self.assertEqual(c['expected_pending']['outcome'], 'ongoing')

    def test_v2_strict_inputs(self):
        fixture = json.loads(terminal_reference.V2_FIXTURE.read_text())
        case = next(c for c in fixture['cases'] if c['id'] == 'empty-draw-p1')
        mutations = [dict(draw_seat=n) for n in [-1, 2, True, '1', None]]
        mutations += [dict(draw_seet=1), dict(action='unknown'), dict(action=['draw']),
                      dict(action='settle'), dict(concede_seat=0), dict(life=[20]),
                      dict(life=[True, 20]), dict(life_order=[0, 0]),
                      dict(injection_order='settle_between'), dict(library_card='Island')]
        for update in mutations:
            wrong = copy.deepcopy(fixture)
            wrong['cases'] = [dict(case, **update)]
            with self.subTest(update=update), self.assertRaises(ValueError):
                terminal_reference.validate(wrong)
        for field in case:
            wrong = copy.deepcopy(fixture)
            wrong['cases'] = [copy.deepcopy(case)]
            del wrong['cases'][0][field]
            with self.subTest(missing=field), self.assertRaises(ValueError):
                terminal_reference.validate(wrong)
        for version in [0, 3, True, '2']:
            with self.assertRaisesRegex(ValueError, 'version'):
                terminal_reference.validate(dict(fixture, version=version))

    def test_every_v2_observation_leaf_has_named_divergence(self):
        fixture = json.loads(terminal_reference.V2_FIXTURE.read_text())
        expected = terminal_reference.expected_results(fixture)
        terminal_reference.compare(fixture, expected)
        def leaves(obj, path=()):
            if isinstance(obj, dict):
                for k, v in obj.items():
                    yield from leaves(v, path + (k,))
            elif isinstance(obj, list):
                for k, v in enumerate(obj):
                    yield from leaves(v, path + (k,))
            else:
                yield path
        for path in leaves(expected):
            wrong = copy.deepcopy(expected)
            parent = wrong
            for key in path[:-1]:
                parent = parent[key]
            value = parent[path[-1]]
            parent[path[-1]] = (not value if type(value) is bool else
                                value + 1 if type(value) is int else
                                0 if value is None else 'CORRUPTED')
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, 'first divergence') as error:
                terminal_reference.compare(fixture, wrong)
            self.assertIn(path[0], str(error.exception))
        # An old report cannot silently acquire v2 meaning.
        old = json.loads(terminal_reference.FIXTURE.read_text())
        with self.assertRaises(ValueError):
            terminal_reference.compare(fixture, {c['id']: c['expected'] for c in old['cases']})

    def test_shared_invalid_corpus_is_rejected(self):
        corpus = json.loads((terminal_reference.ROOT / 'fixtures/reference/terminal-invalid.json').read_text())
        self.assertEqual(len(corpus), 27)
        for case in corpus:
            with self.subTest(case=case['id']), self.assertRaisesRegex(ValueError, 'terminal input'):
                document = terminal_reference.load_json(case['text']) if 'text' in case else case['fixture']
                terminal_reference.validate(document)

    def test_conflicting_duplicate_json_fields_rejected(self):
        fixture = terminal_reference.V2_FIXTURE.read_text()
        duplicate = fixture.replace('"draw_seat": 1', '"draw_seat": 0, "draw_seat": 1', 1)
        with self.assertRaisesRegex(ValueError, 'duplicate'):
            terminal_reference.validate(terminal_reference.load_json(duplicate))

    def test_recorded_legacy_artifact_requires_legacy_contract(self):
        fixture = json.loads(terminal_reference.FIXTURE.read_text())
        report = json.loads((terminal_reference.ROOT / 'doc/evidence/terminal/xmage-terminal.json').read_text())
        terminal_reference.compare(fixture, report['checkpoints'])
        current = json.loads(terminal_reference.V2_FIXTURE.read_text())
        with self.assertRaises(ValueError):
            terminal_reference.compare(current, report['checkpoints'])

    def test_fault_controls_require_complete_observations_and_named_semantic_failure(self):
        fixture = json.loads(terminal_reference.V2_FIXTURE.read_text())
        fixture['cases'] = [next(c for c in fixture['cases'] if c['id'] ==
                                'mixed-life-p0-draw-p1-life_then_draw')]
        wanted = terminal_reference.expected_results(fixture)
        key = fixture['cases'][0]['id']
        for fault, checkpoint, seat, value in [('wrong_draw_seat', 'settled', 1, False),
                                               ('premature_settlement', 'pending', 0, True)]:
            actual = copy.deepcopy(wanted)
            actual[key][checkpoint]['lost'][seat] = value
            self.assertIn(f'{checkpoint}.lost[{seat}]', terminal_reference.detect_fault(fixture, actual, fault))
            for malformed in [{}, {key: {}}, copy.deepcopy(actual)]:
                if malformed == actual:
                    del malformed[key]['settled']['winner']
                with self.subTest(fault=fault, malformed=malformed), self.assertRaisesRegex(ValueError, 'fault observations'):
                    terminal_reference.detect_fault(fixture, malformed, fault)
            unrelated = copy.deepcopy(wanted)
            unrelated[key]['pending']['hand'][0] = 1
            with self.assertRaisesRegex(ValueError, 'unexpected fault divergence'):
                terminal_reference.detect_fault(fixture, unrelated, fault)
            with self.assertRaisesRegex(ValueError, 'missed actual-engine fault'):
                terminal_reference.detect_fault(fixture, wanted, fault)
