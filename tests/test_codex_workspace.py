"""Managed workers cannot share mutable game/reference build outputs."""
import os
from pathlib import Path
import runpy
import tempfile
import unittest
from unittest.mock import patch


SCRIPT = Path(__file__).resolve().parents[1] / 'scripts/symphony/codex_server.py'


class WorkspaceIsolationTests(unittest.TestCase):
    def launch(self, name, home):
        captured = {}

        def execvp(executable, args):
            captured.update(reference=os.environ['MTG_REFERENCE_CACHE'],
                            target=os.environ['CARGO_TARGET_DIR'], args=args)

        with patch.dict(os.environ, {'CODEX_HOME': str(home)}, clear=True), \
                patch.object(Path, 'home', return_value=home), \
                patch.object(Path, 'cwd', return_value=home / 'workspaces' / name), \
                patch('os.execvp', side_effect=execvp):
            runpy.run_path(str(SCRIPT), run_name='__main__')
        return captured

    def test_two_workers_have_disjoint_mutable_cache_and_build_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            home = Path(directory)
            first, second = [self.launch(name, home) for name in ('GH-209', 'GH-210')]
            self.assertNotEqual(first['reference'], second['reference'])
            self.assertNotEqual(first['target'], second['target'])
            for result in (first, second):
                self.assertTrue(Path(result['reference']).is_relative_to(home / '.cache'))
                self.assertTrue(Path(result['target']).is_relative_to(home / 'workspaces'))
                self.assertEqual(result['args'][-1], 'app-server')
                self.assertIn('approvals_reviewer="auto_review"', result['args'])

    def test_unidentified_workspace_fails_before_starting_worker(self):
        with tempfile.TemporaryDirectory() as directory:
            for name in ('unknown', 'GH-0', 'GH-1-extra'):
                with self.subTest(name=name), self.assertRaises(SystemExit):
                    self.launch(name, Path(directory))
