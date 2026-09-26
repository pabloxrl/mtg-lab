"""M0 corpus admission/contract tests, not execution of Magic rules."""
import copy
from pathlib import Path
import unittest
from scripts import scenario as s

ROOT=Path(__file__).resolve().parents[1]

def sign(f):
    f['provenance']['reproduction']['trace_sha256']=s.digest(f['script'])
    f['provenance'].pop('fixture_revision',None)
    f['provenance']['fixture_revision']=s.digest(f)
    return f

class CorpusTests(unittest.TestCase):
    def test_pinned_corpus(self):
        files=sorted((ROOT/'fixtures/scenarios/m0').glob('*.json'))
        self.assertEqual(len(files),6)
        for path in files:
            with self.subTest(path=path):
                f=s.load(path);s.validate(f)
                self.assertEqual(f['provenance']['reference_evidence'],[])
                self.assertEqual(f['provenance']['source_kind'],'original')

    def london(self):
        f=s.load(ROOT/'fixtures/scenarios/m0/mulligan.json')
        by_id={a['id']:a for a in f['script']}
        f['script']=[by_id[k] for k in ('mulligan-0','keep-1','bottom-0','keep-0','upkeep-pass-0','upkeep-pass-1')]
        # Intermediate bottom and keep checkpoints are independently defined in corpus.
        f['checkpoints']=[f['checkpoints'][0],f['checkpoints'][-1]]
        return sign(f)

    def test_bottom_before_next_mulligan_declaration(self):
        # CR 103.5: taking a mulligan includes bottoming before the next round.
        s.validate(self.london())

    def test_keep_before_bottom_rejected(self):
        f=self.london();f['script'][2],f['script'][3]=f['script'][3],f['script'][2]
        with self.assertRaises(ValueError):s.validate(sign(f))

    def test_no_play_before_other_seat_finishes(self):
        f=self.london();f['script'][3],f['script'][4]=f['script'][4],f['script'][3]
        with self.assertRaises(ValueError):s.validate(sign(f))

    def test_privacy_pair_has_identical_view_and_choices(self):
        a=s.load(ROOT/'fixtures/scenarios/m0/privacy-a.json')
        b=s.load(ROOT/'fixtures/scenarios/m0/privacy-b.json')
        self.assertNotEqual(a['setup']['state']['objects'],b['setup']['state']['objects'])
        self.assertEqual(a['checkpoints'],b['checkpoints'])
        self.assertEqual(a['invalid_actions'],b['invalid_actions'])

    def test_candidates_are_pinned_and_not_execution(self):
        import hashlib
        d=s.load(ROOT/'fixtures/xmage-candidates.json')
        self.assertEqual(len(d['selected']),2)
        for candidate in d['selected']:
            self.assertEqual(candidate['status'],'selected-not-adapted')
            self.assertFalse(candidate['same_engine_independent_corroboration'])
            self.assertTrue(candidate['combat_audit'])
        notice=(ROOT/d['retained_license']).read_bytes()
        self.assertEqual(hashlib.sha256(notice).hexdigest(),d['license_sha256'])
        self.assertEqual(notice.decode(),d['full_notice_text'])
        for source in d['sources']:
            self.assertIn(source['commit'],source['url'])
            self.assertEqual(len(source['sha256']),64)
            self.assertTrue(source['file_notices'])
