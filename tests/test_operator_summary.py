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

    def test_every_worker_has_its_own_note_and_state_precedence(self):
        other = dict(issue_identifier='GH-71', started_at='2026-09-27T13:58:00Z')
        state = dict(running=[self.row, other], retrying=[self.row, dict(issue_identifier='GH-72')],
                     blocked=[other, dict(issue_identifier='GH-73')])
        reports = {70: self.report, 71: dict(self.report, issue=71, current='Checking cleanup.')}
        result = summarize(state, reports=reports, milestones={70: 'M1', 71: 'M2'}, now=self.now,
                           queue={'status': 'available', 'issues': [70, 71, 72, 73, 74], 'truncated': False})
        self.assertEqual([(t['issue'], t['state']) for t in result['tasks']],
                         [(70, 'running'), (71, 'running'), (72, 'retrying'), (73, 'blocked'), (74, 'queued')])
        self.assertEqual(result['current'], self.report['current'])
        self.assertEqual(result['tasks'][1]['current'], 'Checking cleanup.')
        self.assertEqual(result['tasks'][1]['milestone'], 'M2')
        self.assertIsNone(result['tasks'][2]['report_updated_at'])
        self.assertEqual(result['queue']['status'], 'available')

    def test_foreign_or_previous_session_report_cannot_bleed_between_workers(self):
        state = dict(running=[self.row, dict(issue_identifier='GH-71',
                     started_at='2026-09-27T14:00:00Z')], retrying=[], blocked=[])
        for report in (self.report, dict(self.report, issue=71)):
            result = summarize(state, reports={70: self.report, 71: report}, now=self.now)
            self.assertEqual(result['tasks'][0]['freshness'], 'fresh')
            self.assertEqual(result['tasks'][1]['freshness'], 'missing')
            self.assertNotEqual(result['tasks'][1]['current'], self.report['current'])

    def test_unavailable_state_and_queue_do_not_claim_an_empty_queue(self):
        result = summarize(None, now=self.now,
                           queue={'status': 'available', 'issues': [74], 'truncated': False})
        self.assertEqual(result['state'], 'unavailable')
        self.assertEqual(result['tasks'][0]['state'], 'ready')
        self.assertIn('unavailable', result['tasks'][0]['current'])
        unavailable = summarize(dict(running=[], retrying=[], blocked=[]), now=self.now)
        self.assertEqual(unavailable['queue']['status'], 'unavailable')
        self.assertIsNone(unavailable['queue']['count'])
        self.assertNotIn('learn from', unavailable['project_goal'])
        self.assertIn('Forge', unavailable['project_goal'])

    def test_queue_only_accepts_bounded_numeric_issue_ids(self):
        result = summarize(self.state, self.report, now=self.now,
            queue={'status': 'available', 'issues': [74, 74, True, 'javascript:alert(1)', -1],
                   'truncated': False, 'error': 'secret command'})
        self.assertEqual([t['issue'] for t in result['tasks']], [70, 74])
        self.assertNotIn('secret command', str(result))
        self.assertNotIn('javascript:', str(result))


class ReportFileTests(unittest.TestCase):
    def setUp(self):
        from unittest.mock import patch
        queue = patch('scripts.symphony.operator_summary.fetch_queue',
                      return_value=dict(status='available', issues=[], truncated=False))
        labels = patch('scripts.symphony.operator_summary.fetch_label',
                       return_value=dict(status='available', issues=[], truncated=False))
        self.labels = labels.start()
        self.addCleanup(labels.stop)
        self.queue = queue.start()
        self.addCleanup(queue.stop)

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

    def test_collector_reads_each_own_report_and_does_not_read_stopped_notes(self):
        import io
        import json
        from pathlib import Path
        import tempfile
        from unittest.mock import patch
        from scripts.symphony.operator_summary import collect
        from scripts.symphony.report_progress import write_report
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for issue in (70, 71, 72):
                workspace = root / f'GH-{issue}'
                workspace.mkdir()
                write_report(workspace, issue, f'Working on {issue}.', 'Correct games.', 'Run checks.')
            state = dict(running=[dict(issue_identifier=f'GH-{n}', started_at='2026-01-01T00:00:00Z')
                                  for n in (70, 71)], retrying=[], blocked=[dict(issue_identifier='GH-72')])
            self.queue.return_value = dict(status='available', issues=[70, 73], truncated=False)
            with patch('urllib.request.urlopen', return_value=io.BytesIO(json.dumps(state).encode())):
                result = collect(root, root / 'unused')
            self.assertEqual([t['issue'] for t in result['tasks']], [70, 71, 72, 73])
            self.assertEqual(result['tasks'][0]['current'], 'Working on 70.')
            self.assertEqual(result['tasks'][1]['current'], 'Working on 71.')
            self.assertNotIn('Working on 72.', str(result))
            self.assertEqual(result['tasks'][3]['state'], 'queued')

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


class QueueTests(unittest.TestCase):
    def test_bounded_repository_scoped_queue_and_truncation(self):
        import json
        import subprocess
        from unittest.mock import patch
        from scripts.symphony.operator_summary import fetch_queue, MAX_QUEUE_ISSUES
        rows = [{'number': n} for n in range(1, MAX_QUEUE_ISSUES + 2)]
        with patch('scripts.symphony.operator_summary.subprocess.run',
                   return_value=subprocess.CompletedProcess([], 0, stdout=json.dumps(rows).encode())) as run:
            queue = fetch_queue()
        self.assertEqual(queue['status'], 'available')
        self.assertTrue(queue['truncated'])
        self.assertEqual(len(queue['issues']), MAX_QUEUE_ISSUES)
        command = run.call_args.args[0]
        self.assertEqual(command[:3], ['gh', 'issue', 'list'])
        self.assertEqual(command[command.index('--repo') + 1], 'pabloxrl/mtg-lab')
        self.assertEqual(command[command.index('--label') + 1], 'agent-ready')
        self.assertEqual(command[command.index('--json') + 1], 'number')
        self.assertLessEqual(run.call_args.kwargs['timeout'], 3)
        self.assertEqual(run.call_args.kwargs['stderr'], subprocess.DEVNULL)

    def test_failures_malformed_output_and_timeouts_are_unavailable_without_details(self):
        import subprocess
        from unittest.mock import patch
        from scripts.symphony.operator_summary import fetch_queue
        for value in (subprocess.CompletedProcess([], 1, stdout=b'private credential'),
                      subprocess.CompletedProcess([], 0, stdout=b'not json'),
                      subprocess.CompletedProcess([], 0, stdout=b'[{"number":true}]'),
                      subprocess.CompletedProcess([], 0, stdout=b'[{"number":7,"title":"private"}]'),
                      subprocess.CompletedProcess([], 0, stdout=b'x' * 16385),
                      OSError('private credential'), subprocess.TimeoutExpired('private command', 3)):
            options = {'side_effect': value} if isinstance(value, Exception) else {'return_value': value}
            with self.subTest(value=type(value)), patch('scripts.symphony.operator_summary.subprocess.run', **options):
                result = fetch_queue()
            self.assertEqual(result, dict(status='unavailable', issues=[], truncated=False))


class GithubControlTests(unittest.TestCase):
    """GH-253 requires controls independent of controller activity/readiness."""
    def test_idle_with_blocked_issues_and_no_note(self):
        from unittest.mock import patch
        from scripts.symphony.operator_summary import collect
        with patch('scripts.symphony.operator_summary.bounded_json', side_effect=OSError), \
             patch('scripts.symphony.operator_summary.urllib.request.urlopen', side_effect=OSError), \
             patch('scripts.symphony.operator_summary.fetch_queue', return_value=dict(status='available', issues=[])), \
             patch('scripts.symphony.operator_summary.subprocess.run') as run:
            import subprocess
            run.return_value = subprocess.CompletedProcess([], 0, stdout=b'[{"number":23}]')
            result = collect()
        self.assertEqual([t['issue'] for t in result['tasks']], [23])

    def test_overlap_preserves_worker_and_both_controls(self):
        result = summarize(dict(running=[dict(issue_identifier='GH-70')], retrying=[], blocked=[]),
            queue=dict(status='available', issues=[70, 71]),
            controls={'blocked': dict(status='available', issues=[70, 72]),
                      'held': dict(status='available', issues=[70, 71])})
        self.assertEqual([(t['issue'], t['state']) for t in result['tasks']],
                         [(70, 'running'), (71, 'controlled'), (72, 'controlled')])
        self.assertEqual(result['tasks'][0]['github_controls'], ['blocked', 'held'])
        self.assertEqual(result['queue']['count'], 2)
        self.assertEqual(result['tracker']['blocked']['count'], 2)
        self.assertEqual(result['tasks'][2]['freshness'], 'missing')
        self.assertIn('reason', result['tasks'][2]['current'])

    def test_idle_controls_keep_ready_count_separate_and_missing_reason_explicit(self):
        result = summarize(dict(running=[], retrying=[], blocked=[]),
            queue=dict(status='available', issues=[]),
            controls={'blocked': dict(status='available', issues=[23, 210]),
                      'held': dict(status='available', issues=[])})
        self.assertEqual(result['state'], 'idle')
        self.assertEqual(result['queue']['count'], 0)
        self.assertEqual(result['tracker']['blocked']['count'], 2)
        self.assertEqual([t['issue'] for t in result['tasks']], [23, 210])
        self.assertTrue(all(t['freshness'] == 'missing' and 'reason' in t['current'] for t in result['tasks']))

    def test_partial_tracker_failure_truncation_and_numeric_filter(self):
        result = summarize(None, controls={
            'blocked': dict(status='unavailable', issues=[], error='private'),
            'held': dict(status='available', issues=[71, 71, True, '../private', -1], truncated=True)})
        self.assertEqual(result['tracker']['blocked'], dict(status='unavailable', count=None))
        self.assertEqual(result['tracker']['held'], dict(status='truncated', count=1))
        self.assertEqual([t['issue'] for t in result['tasks']], [71])
        self.assertFalse(result['tasks'][0]['github_controls_complete'])
        self.assertNotIn('private', str(result))

    def test_old_note_is_historical_and_foreign_future_invalid_notes_are_rejected(self):
        now = datetime(2026, 10, 9, 14, tzinfo=timezone.utc)
        note = dict(version=1, issue=23, updated_at='2026-10-09T13:00:00Z',
                    current='Checking tests.', why='Trustworthy games.', next='Run checks.', blocker=None)
        for report, expected in ((note, 'stale'), (dict(note, issue=24), 'missing'),
                                 (dict(note, updated_at='2026-10-10T00:00:00Z'), 'missing'),
                                 (dict(note, extra='private'), 'missing')):
            with self.subTest(expected=expected, report=report):
                task = summarize(None, now=now, reports={23: report}, controls={
                    'blocked': dict(status='available', issues=[23])})['tasks'][0]
                self.assertEqual(task['freshness'], expected)
                self.assertIn('blocked or held', task['current'])
                self.assertIsNone(task['blocker'])
                if expected == 'stale':
                    self.assertEqual(task['previous_note']['current'], note['current'])
                    self.assertIn('predate', task['current'])
                else:
                    self.assertNotIn('previous_note', task)

    def test_control_fetch_is_bounded_read_only_and_timeout_is_unknown(self):
        import json
        import subprocess
        from unittest.mock import patch
        from scripts.symphony.operator_summary import fetch_label, MAX_QUEUE_ISSUES
        for label in ('agent-blocked', 'agent-held'):
            with patch('scripts.symphony.operator_summary.subprocess.run', return_value=
                       subprocess.CompletedProcess([], 0, stdout=json.dumps(
                           [{'number': n} for n in range(1, MAX_QUEUE_ISSUES + 2)]).encode())) as run:
                result = fetch_label(label)
            self.assertTrue(result['truncated'])
            self.assertEqual(len(result['issues']), MAX_QUEUE_ISSUES)
            self.assertEqual(run.call_args.args[0], ['gh', 'issue', 'list', '--repo', 'pabloxrl/mtg-lab',
                '--state', 'open', '--label', label, '--limit', '201', '--json', 'number'])
            self.assertEqual(run.call_args.kwargs['timeout'], 3)
            with patch('scripts.symphony.operator_summary.subprocess.run',
                       side_effect=subprocess.TimeoutExpired('private command', 3)):
                self.assertEqual(fetch_label(label), dict(status='unavailable', issues=[], truncated=False))

    def test_collection_retains_only_each_controlled_issues_safe_note(self):
        import io
        import json
        import tempfile
        from pathlib import Path
        from unittest.mock import patch
        from scripts.symphony.operator_summary import collect
        from scripts.symphony.report_progress import write_report
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            for number in (23, 24):
                workspace = root / f'GH-{number}'
                workspace.mkdir()
                write_report(workspace, number, f'Checking task {number}.', 'Trustworthy games.', 'Run checks.')
            state = dict(running=[], retrying=[], blocked=[])
            with patch('scripts.symphony.operator_summary.urllib.request.urlopen',
                       return_value=io.BytesIO(json.dumps(state).encode())), \
                 patch('scripts.symphony.operator_summary.fetch_queue',
                       return_value=dict(status='available', issues=[])), \
                 patch('scripts.symphony.operator_summary.fetch_label',
                       side_effect=lambda label: dict(status='available', issues=[23] if label == 'agent-blocked' else [24])):
                result = collect(root, root / 'unused')
            self.assertEqual([t['previous_note']['current'] for t in result['tasks']],
                             ['Checking task 23.', 'Checking task 24.'])
            self.assertEqual([t['github_controls'] for t in result['tasks']], [['blocked'], ['held']])
            self.assertEqual(result['queue']['count'], 0)
