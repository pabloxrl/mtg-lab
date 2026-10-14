"""L0-005 accounting requirements, independent of any engine's game output."""
import sys
import io
import json
import tarfile
import tempfile
import unittest
from collections import Counter
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import xmage_campaign as campaign
import check_xmage_campaign as check
import xmage_ai_normalize as normalize


class CampaignAccounting(unittest.TestCase):
    def test_declared_hundred_is_balanced_and_baseline_precedes_concurrency(self):
        rows = campaign.schedule()
        self.assertEqual(len(rows), 100)
        self.assertEqual(len({r['seed'] for r in rows}), 100)
        self.assertEqual({tuple(r['decks']) + (r['starter'],) for r in rows}, {
            ('red', 'green', 0), ('red', 'green', 1),
            ('green', 'red', 0), ('green', 'red', 1),
            ('red', 'red', 0), ('red', 'red', 1),
            ('green', 'green', 0), ('green', 'green', 1)})
        self.assertEqual(sorted(Counter(tuple(r['decks']) + (r['starter'],) for r in rows).values()), [12]*4+[13]*4)
        self.assertEqual([r['workers'] for r in rows[:16]], [1]*16)
        self.assertEqual([r['workers'] for r in rows[16:]], [2]*84)
        self.assertEqual(rows, campaign.schedule())

    def test_projection_includes_every_failure_and_duplicate_cost(self):
        # Four attempts cost 80 seconds; only two distinct complete games count.
        rows = [dict(status=s, identity=i) for s,i in
                [('complete','a'),('failed',None),('complete','a'),('complete','b')]]
        result = campaign.account(rows, 80)
        self.assertEqual(result['source_complete'], 3)
        self.assertEqual(result['distinct_complete'], 2)
        self.assertEqual(result['duplicates'], 1)
        self.assertEqual(result['projected_1000_generation_seconds'], 40000)
        self.assertEqual(result['source_completion_rate'], .75)

    def test_no_success_cannot_yield_finite_projection(self):
        result = campaign.account([dict(status='truncated', identity=None)], 5)
        self.assertIsNone(result['projected_1000_generation_seconds'])
        with self.assertRaises(ValueError):
            campaign.account([dict(status='started', identity=None)], 5)


class MeasurementStatistics(unittest.TestCase):
    def test_nearest_rank_percentiles_retain_slow_tail(self):
        result = check.distribution(list(range(1, 101)))
        self.assertEqual(result['p50'], 50)
        self.assertEqual(result['p95'], 95)
        self.assertEqual(result['total'], 5050)
        for bad in [[], [float('nan')], [float('inf')], [-1]]:
            with self.assertRaises(ValueError):
                check.distribution(bad)


class RetainedCampaign(unittest.TestCase):
    def test_every_declared_attempt_has_verified_durable_accounting(self):
        result = check.check()
        self.assertEqual(result['attempts'], 100)
        self.assertEqual(sum(result['source_classes'].values()), 100)
        self.assertEqual(result['new_provisional_games'], 0)
        self.assertEqual(result['final_admitted_games'], 0)

    def test_archived_assignment_and_canonical_corruption_fail(self):
        # These are integrity mutations of a real game, never gameplay oracles.
        original = check.DEFAULT / 'attempts/000.tar.xz'
        with tarfile.open(original, 'r:xz') as archive:
            files = {m.name: archive.extractfile(m).read() for m in archive}
        for name, expected in [('measurement.json', 'seed binding'),
                               ('canonical.json', 'canonical bytes')]:
            altered = dict(files)
            if name == 'measurement.json':
                value = json.loads(altered[name])
                value['seed'] += 1
                altered[name] = json.dumps(value).encode()
            else:
                altered[name] = b'{}'
            with self.subTest(name=name), tempfile.TemporaryDirectory() as tmp:
                path = Path(tmp) / 'bad.tar.xz'
                with tarfile.open(path, 'w:xz') as archive:
                    for entry, data in altered.items():
                        member = tarfile.TarInfo(entry)
                        member.size = len(data)
                        archive.addfile(member, io.BytesIO(data))
                with self.assertRaisesRegex(ValueError, expected):
                    check.read_attempt(path)

    def test_unresolved_source_choices_and_effects_remain_fail_closed(self):
        # L0 requires effect lifetime and incarnation binding; an unexplained
        # source observation must not be waived or turned into admitted output.
        # These assert rejection, not that either engine's rules are incorrect.
        for attempt, field in [(37, '/source/4352/effect lifetime'),
                               (39, '/source/4775/departed effect target'),
                               (68, '/source/5168/committed ability')]:
            with self.subTest(attempt=attempt), tempfile.TemporaryDirectory() as tmp:
                archive_path = check.DEFAULT / f'attempts/{attempt:03d}.tar.xz'
                check.read_attempt(archive_path)
                with tarfile.open(archive_path, 'r:xz') as archive:
                    archive.extractall(tmp, filter='data')
                with self.assertRaisesRegex(ValueError, field):
                    normalize.normalize(Path(tmp) / 'trace.jsonl.xz')

    def test_retained_stack_target_difference_is_not_silently_accepted(self):
        with tarfile.open(check.DEFAULT/'attempts/006.tar.xz','r:xz') as archive:
            converted=json.load(archive.extractfile('converted.json'))
        with tarfile.open(check.DEFAULT/'native/006.tar.xz','r:xz') as archive:
            native=json.load(archive.extractfile('native-1.json'))['run']
        # Required identity/incarnation comparison rejects the actual mismatch;
        # this does not adjudicate which engine/adapter needs correction.
        with self.assertRaisesRegex(ValueError, r"targets\[0\].object"):
            normalize.verify_native(converted,native)
