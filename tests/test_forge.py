"""Forge bridge boundaries; expected priority comes from CR 117.3d."""
import copy
import sys
import unittest
import tempfile
import subprocess
import os
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import forge
import scenario
import xmage


class ForgeContract(unittest.TestCase):
    def setUp(self):
        self.fixture = scenario.load(xmage.FIXTURE)
        initial = copy.deepcopy(self.fixture['setup']['state'])
        initial.pop('objects')
        initial.pop('effects')
        for player in initial['players']:
            player.pop('land_plays_used')
        after = copy.deepcopy(initial)
        after['priority'] = 1  # CR 117.3d, independently specified.
        self.output = dict(bridge_version=1, checkpoints=[
            dict(name='initial', state=initial), dict(name='priority-p1', state=after)],
            unsupported={k: 'not exported' for k in (
                'object_characteristics_status_damage', 'legal_choices', 'outcome',
                'effects_and_private_views', 'land_plays_used')},
            observed_cards={o['id']: o['card_id'] for o in self.fixture['setup']['state']['objects']})

    def test_same_ordered_card_identities_required(self):
        self.output['observed_cards']['p0-library-0'] = 'mountain'
        with self.assertRaisesRegex(ValueError, 'card identity'):
            forge.compare(self.fixture, self.output)

    def test_missing_identity_evidence_rejected(self):
        del self.output['observed_cards']
        with self.assertRaisesRegex(ValueError, 'card identity'):
            forge.compare(self.fixture, self.output)

    def test_independent_expected_pass(self):
        forge.compare(self.fixture, self.output)

    def test_wrong_state_and_order_are_rejected(self):
        # Same independent CR 117.3d expectation, at both synchronization points.
        for point in (0, 1):
            for mutation in ('life', 'priority', 'library', 'mana', 'stack'):
                with self.subTest(point=point, mutation=mutation):
                    output = copy.deepcopy(self.output)
                    state = output['checkpoints'][point]['state']
                    if mutation == 'life':
                        state['players'][0]['life'] = 19
                    elif mutation == 'priority':
                        state['priority'] = 1 - state['priority']
                    elif mutation == 'library':
                        state['players'][0]['zones']['library'].reverse()
                    elif mutation == 'mana':
                        state['players'][0]['mana']['G'] = 1
                    else:
                        state['stack'] = ['unexpected']
                    with self.assertRaises(ValueError):
                        forge.compare(self.fixture, output)

    def test_observability_and_checkpoint_presence_required(self):
        for key in ('unsupported', 'observed_cards'):
            output = copy.deepcopy(self.output)
            del output[key]
            with self.assertRaises(ValueError):
                forge.compare(self.fixture, output)
        self.output['checkpoints'].pop()
        with self.assertRaises(ValueError):
            forge.compare(self.fixture, self.output)

    def test_skipped_or_wrong_test_is_not_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'report.xml'
            valid = '<testsuite tests="1" failures="0" errors="0" skipped="0"><testcase name="neutralSmoke"/></testsuite>'
            path.write_text(valid)
            forge.require_executed(path)
            for old, new in [('tests="1"', 'tests="0"'), ('skipped="0"', 'skipped="1"'),
                             ('errors="0"', 'errors="1"'), ('failures="0"', 'failures="1"'),
                             ('neutralSmoke', 'someOtherTest')]:
                path.write_text(valid.replace(old, new))
                with self.assertRaises(ValueError):
                    forge.require_executed(path)

    def test_unpinned_toolchain_rejected(self):
        with mock.patch.object(forge.subprocess, 'run', return_value=mock.Mock(
                stderr='openjdk version "17"', stdout='Apache Maven 3.8.1')), \
                mock.patch.object(forge.platform, 'system', return_value='Linux'), \
                mock.patch.object(forge.platform, 'machine', return_value='aarch64'), \
                mock.patch.dict(os.environ, {'JAVA_HOME': '/unused'}):
            with self.assertRaisesRegex(ValueError, 'toolchain'):
                forge.verify_inputs(Path('/unused'))

    def test_cli_missing_reference_fails_without_input_or_display(self):
        with tempfile.TemporaryDirectory() as directory:
            env = dict(os.environ)
            env.pop('DISPLAY', None)
            env.pop('WAYLAND_DISPLAY', None)
            result = subprocess.run([sys.executable, str(forge.ROOT/'scripts/forge.py'),
                                     'run', '--cache', directory], env=env,
                                    stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=10)
            self.assertEqual(result.returncode, 2)
            self.assertIn('missing externally installed pinned Forge source', result.stdout)


if __name__ == '__main__':
    unittest.main()
