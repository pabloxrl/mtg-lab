"""Independent CR 117.3d expectations for the narrow smoke comparator."""
import copy
import sys
import unittest
import tempfile
import os
from unittest import mock
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import scenario
import xmage


class SmokeContract(unittest.TestCase):
    def setUp(self):
        self.fixture = scenario.load(xmage.FIXTURE)
        initial = copy.deepcopy(self.fixture['setup']['state'])
        initial.pop('objects'); initial.pop('effects')
        for p in initial['players']:
            p.pop('land_plays_used')
        after = copy.deepcopy(initial)
        after['priority'] = 1  # CR 117.3d; no life/zone/mana changes from passing.
        self.output = dict(bridge_version=1, unsupported={
            "object_characteristics_status_damage": "not exported",
            "legal_choices": "not exported", "outcome": "not exported",
            "effects_and_private_views": "not exported",
            "land_plays_used": "not exported"}, checkpoints=[
            dict(name='initial', state=initial), dict(name='priority-p1', state=after)])

    def test_unobservable_fields_must_be_declared(self):
        del self.output['unsupported']
        with self.assertRaisesRegex(ValueError, 'observability'):
            xmage.compare(self.fixture, self.output)

    def test_wrong_installed_toolchain_is_rejected(self):
        pins = dict(archives={}, jdk_runtime='21.0.9+10-LTS', maven='3.9.11')
        with mock.patch.object(scenario, 'load', return_value=pins), \
             mock.patch.object(xmage.platform, 'system', return_value='Linux'), \
             mock.patch.object(xmage.platform, 'machine', return_value='aarch64'), \
             mock.patch.object(xmage.subprocess, 'run', return_value=mock.Mock(
                 stdout='openjdk version "17"', stderr='', returncode=0)):
            with self.assertRaisesRegex(ValueError, 'toolchain'):
                xmage.verify_inputs(Path('/tmp'))

    def test_incomplete_or_extra_choices_rejected(self):
        for choices in ([], self.fixture['script'][0]['choices'] * 2):
            fixture = copy.deepcopy(self.fixture)
            fixture['script'][0]['choices'] = choices
            fixture['provenance']['reproduction']['trace_sha256'] = scenario.digest(fixture['script'])
            del fixture['provenance']['fixture_revision']
            fixture['provenance']['fixture_revision'] = scenario.digest(fixture)
            with self.assertRaises(ValueError):
                xmage.scope(fixture)

    def test_independent_pass_expectation(self):
        xmage.scope(self.fixture)
        xmage.compare(self.fixture, self.output)

    def test_changed_life_and_priority_detected(self):
        for point, path in [(0, 'life'), (1, 'life'), (0, 'priority'), (1, 'priority')]:
            with self.subTest(point=point, field=path):
                output = copy.deepcopy(self.output)
                state = output['checkpoints'][point]['state']
                if path == 'life': state['players'][0]['life'] = 19
                else: state['priority'] = 1 - state['priority']
                with self.assertRaises(ValueError): xmage.compare(self.fixture, output)

    def test_library_order_detected(self):
        for point in (0, 1):
            output = copy.deepcopy(self.output)
            output['checkpoints'][point]['state']['players'][0]['zones']['library'].reverse()
            with self.assertRaises(ValueError): xmage.compare(self.fixture, output)

    def test_closed_stdin_unset_displays_and_failure_propagation(self):
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory)/'run.log'
            code = "import os,sys; assert sys.stdin.read()==''; assert 'DISPLAY' not in os.environ; assert 'WAYLAND_DISPLAY' not in os.environ"
            xmage.bounded([sys.executable, '-c', code], directory,
                          dict(os.environ, DISPLAY='bad', WAYLAND_DISPLAY='bad'), log, 5)
            with self.assertRaisesRegex(ValueError, 'exited 3'):
                xmage.bounded([sys.executable, '-c', 'raise SystemExit(3)'], directory,
                              os.environ, log, 5)
            with self.assertRaisesRegex(ValueError, 'timeout'):
                xmage.bounded([sys.executable, '-c', 'import time; time.sleep(10)'],
                              directory, os.environ, log, .1)

    def test_missing_checkpoint_detected(self):
        self.output['checkpoints'].pop()
        with self.assertRaises(ValueError): xmage.compare(self.fixture, self.output)

    def test_unasserted_mana_or_stack_changes_detected(self):
        for field in ('mana', 'stack'):
            with self.subTest(field=field):
                output = copy.deepcopy(self.output)
                state = output['checkpoints'][1]['state']
                if field == 'mana': state['players'][0]['mana']['G'] = 1
                else: state['stack'] = ['unexpected']
                with self.assertRaises(ValueError): xmage.compare(self.fixture, output)


if __name__ == '__main__': unittest.main()
