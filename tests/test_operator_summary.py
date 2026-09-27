"""Human progress reports are evidence-bound, fresh, and safe to display."""
from datetime import datetime, timezone
import unittest
from scripts.symphony.operator_summary import summarize


class SummaryTests(unittest.TestCase):
    def test_fresh_note_preserves_human_explanation_and_connects_to_goal(self):
        now = datetime(2026, 9, 27, 14, 0, tzinfo=timezone.utc)
        state = {'running': [{'issue_identifier': 'GH-70', 'started_at': '2026-09-27T13:00:00Z'}],
                 'retrying': [], 'blocked': []}
        report = dict(version=1, issue=70, updated_at='2026-09-27T13:59:00Z',
                      current='I am checking what happens when a spell’s target dies before it resolves.',
                      why='Games need to resolve these responses correctly before we can trust an AI match.',
                      next='I will test both spells together and keep those checks as regressions.', blocker=None)
        result = summarize(state, report, 'M1', now)
        self.assertEqual(result['current'], report['current'])
        self.assertEqual(result['why'], report['why'])
        self.assertEqual(result['freshness'], 'fresh')
        self.assertIn('complete', result['milestone_goal'])
        self.assertIn('AI', result['project_goal'])

    def setUp(self):
        self.now = datetime(2026, 9, 27, 14, 0, tzinfo=timezone.utc)
        self.row = dict(issue_identifier='GH-70', started_at='2026-09-27T13:00:00Z')
        self.state = dict(running=[self.row], retrying=[], blocked=[])
        self.report = dict(version=1, issue=70, updated_at='2026-09-27T13:59:00Z',
                           current='I am testing spell responses.', why='AI games need correct results.',
                           next='I will check the regression suite.', blocker=None)

    def test_missing_invalid_foreign_old_session_and_future_notes_are_not_current(self):
        for report in (None, {}, dict(self.report, issue=69),
                       dict(self.report, updated_at='2026-09-27T12:59:59Z'),
                       dict(self.report, updated_at='2026-09-27T14:00:01Z'),
                       dict(self.report, updated_at='2026-09-27T13:59:00'),
                       dict(self.report, version=True), dict(self.report, extra='private')):
            with self.subTest(report=report):
                result = summarize(self.state, report, 'M1', self.now)
                self.assertEqual(result['freshness'], 'missing')
                self.assertIsNone(result['report_updated_at'])
                self.assertNotEqual(result['current'], self.report['current'])

    def test_stale_note_is_retained_but_explicitly_marked(self):
        report = dict(self.report, updated_at='2026-09-27T13:54:59Z')
        result = summarize(self.state, report, 'M1', self.now)
        self.assertEqual(result['freshness'], 'stale')
        self.assertEqual(result['current'], report['current'])
        self.assertEqual(result['report_updated_at'], '2026-09-27T13:54:59+00:00')
        report['updated_at'] = '2026-09-27T13:55:00Z'
        self.assertEqual(summarize(self.state, report, 'M1', self.now)['freshness'], 'fresh')

    def test_idle_retry_block_and_unavailable_do_not_inherit_active_note(self):
        cases = [(None, 'unavailable'), ({}, 'unavailable'),
                 (dict(running=[], retrying=[], blocked=[]), 'idle'),
                 (dict(running=[], retrying=[self.row], blocked=[]), 'retrying'),
                 (dict(running=[], retrying=[], blocked=[self.row]), 'blocked')]
        for state, expected in cases:
            with self.subTest(expected=expected):
                result = summarize(state, self.report, 'M1', self.now)
                self.assertEqual(result['state'], expected)
                self.assertIsNone(result['report_updated_at'])
                self.assertNotEqual(result['current'], self.report['current'])
        self.assertIn('does not mean', summarize(cases[2][0], now=self.now)['next'])

    def test_untrusted_row_does_not_supply_links_or_raw_commands(self):
        self.row.update(issue_identifier='../../.codex/auth.json', last_message='secret command',
                        issue_url='javascript:alert(1)', workspace_path='/private')
        result = summarize(self.state, self.report, 'unknown', self.now)
        self.assertIsNone(result['issue'])
        self.assertIsNone(result['milestone'])
        self.assertNotIn('secret command', str(result))
        self.assertNotIn('javascript:', str(result))

    def test_blocker_clear_and_bounded_plain_fields(self):
        from scripts.symphony.operator_summary import validate_report
        self.assertEqual(summarize(self.state, dict(self.report, blocker='A required service is unavailable.'),
                                   'M1', self.now)['blocker'], 'A required service is unavailable.')
        self.assertIsNone(summarize(self.state, self.report, 'M1', self.now)['blocker'])
        for value in ['', 'x' * 701, 'raw\nlog', None, 5]:
            with self.subTest(value=value):
                with self.assertRaises(ValueError):
                    validate_report(dict(self.report, current=value))


class ReportFileTests(unittest.TestCase):
    def test_atomic_writer_and_safe_reader(self):
        import json
        from pathlib import Path
        import tempfile
        from scripts.symphony.operator_summary import bounded_json
        from scripts.symphony.report_progress import write_report
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            write_report(root, 70, 'I am testing responses.', 'Matches need correct results.', 'Run the suite.')
            target = root / '.agent-artifacts/operator-summary.json'
            first = bounded_json(target, 8192)
            self.assertEqual(first['issue'], 70)
            write_report(root, 71, 'I am testing combat.', 'Games need correct damage.', 'Check blockers.')
            self.assertEqual(bounded_json(target, 8192)['issue'], 71)
            self.assertEqual(len(list(target.parent.iterdir())), 1)
            target.unlink()
            (root / 'private.json').write_text(json.dumps(first))
            target.symlink_to(root / 'private.json')
            with self.assertRaises(ValueError):
                bounded_json(target, 8192)
            with self.assertRaises(ValueError):
                bounded_json(root / 'private.json', 1)

    def test_symlink_directory_cannot_escape_workspace(self):
        from pathlib import Path
        import tempfile
        from scripts.symphony.operator_summary import bounded_json
        from scripts.symphony.report_progress import write_report
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            (root / 'private').mkdir()
            (root / '.agent-artifacts').symlink_to(root / 'private', target_is_directory=True)
            with self.assertRaises(ValueError):
                write_report(root, 70, 'Working.', 'Why.', 'Next.')
            (root / 'private/operator-summary.json').write_text('{}')
            with self.assertRaises(ValueError):
                bounded_json(root / '.agent-artifacts/operator-summary.json', 8192)

    def test_collector_uses_allowlisted_workspace_report_and_current_program(self):
        import io
        import json
        from pathlib import Path
        import tempfile
        from unittest.mock import patch
        from scripts.symphony.operator_summary import collect
        from scripts.symphony.report_progress import write_report
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            workspace = root / 'GH-70'
            workspace.mkdir()
            (workspace / 'doc/programs').mkdir(parents=True)
            (workspace / 'doc/programs/rfc-0002.json').write_text(json.dumps(
                {'tasks': [dict(issue=70, milestone='M1')]}))
            write_report(workspace, 70, 'I am testing targets.', 'Games need correct responses.', 'Check invalid targets.')
            state = dict(running=[dict(issue_identifier='GH-70', started_at='2026-01-01T00:00:00Z',
                workspace_path='/unrelated/private', last_message='private command')], retrying=[], blocked=[])
            with patch('urllib.request.urlopen', return_value=io.BytesIO(json.dumps(state).encode())):
                result = collect(root, root / 'unused')
            self.assertEqual(result['current'], 'I am testing targets.')
            self.assertEqual(result['milestone'], 'M1')
            self.assertNotIn('private command', str(result))
            (workspace / '.agent-artifacts/operator-summary.json').write_text('broken JSON')
            with patch('urllib.request.urlopen', return_value=io.BytesIO(json.dumps(state).encode())):
                self.assertEqual(collect(root, root / 'unused')['freshness'], 'missing')
            with patch('urllib.request.urlopen', side_effect=OSError('private connection detail')):
                result = collect(root, root / 'unused')
            self.assertEqual(result['state'], 'unavailable')
            self.assertNotIn('private connection detail', str(result))

    def test_maximum_unicode_note_is_readable_after_successful_write(self):
        from pathlib import Path
        import tempfile
        from scripts.symphony.operator_summary import bounded_json, MAX_REPORT_BYTES
        from scripts.symphony.report_progress import write_report
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            sentence = '\U0001f0a1' * 700
            expected = write_report(root, 70, sentence, sentence, sentence, sentence)
            self.assertEqual(bounded_json(root / '.agent-artifacts/operator-summary.json',
                                          MAX_REPORT_BYTES), expected)
