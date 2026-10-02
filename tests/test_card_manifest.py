"""Requirements: RFC 0002 §3 exact table/non-goals; GH-11 data boundary.

Expected deck counts are independently read from the pinned RFC table, not the
implementation's constants. Synthetic source tests do not claim live access.
"""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import card_manifest as cm

ROOT = Path(__file__).resolve().parents[1]


class ManifestTests(unittest.TestCase):
    def setUp(self):
        self.manifest = json.loads(cm.DEFAULT_MANIFEST.read_bytes())

    def reject(self, mutate):
        value = copy.deepcopy(self.manifest)
        mutate(value)
        # Re-seal records so structural rejection is not just a stale checksum.
        if isinstance(value, dict):
            for record in value.get('cards', []) + value.get('decks', []):
                if isinstance(record, dict):
                    record['content_sha256'] = cm.digest(
                        {k: v for k, v in record.items() if k != 'content_sha256'})
        with self.assertRaises(ValueError):
            cm.validate_manifest(value)

    def test_exact_rfc_decks_and_all_seats(self):
        cm.validate_manifest(self.manifest)
        table = (ROOT / 'doc/rfcs/0002-first-mvp.md').read_text().split('| Red deck |')[1]
        rows = table.split('| **Total**')[0].splitlines()[2:]
        expected = {'red': {}, 'green': {}}
        for row in rows:
            red, nr, green, ng = [x.strip() for x in row.split('|')[1:-1]]
            expected['red'][red.lower().replace(' ', '-')] = int(nr)
            expected['green'][green.lower().replace(' ', '-')] = int(ng)
        for deck in self.manifest['decks']:
            self.assertEqual({x['card_id']: x['copies'] for x in deck['cards']}, expected[deck['id']])
            self.assertEqual((deck['total_cards'], deck['lands'], deck['spells']), (40, 16, 24))
        self.assertEqual(len(self.manifest['cards']), 21)
        self.assertEqual({(tuple(m['seats']), m['starting_seat']) for m in self.manifest['matchups']},
                         {((a,b),s) for a in ['red','green'] for b in ['red','green'] for s in [0,1]})
        token = next(c for c in self.manifest['cards'] if c['kind'] == 'token')
        self.assertEqual(token['characteristics']['colors'], ['R'])
        self.assertEqual((token['characteristics']['power'], token['characteristics']['toughness']), ('1','1'))
        dragon = next(c for c in self.manifest['cards'] if c['name'] == 'Shivan Dragon')
        self.assertEqual(dragon['printing']['collector_number'], '763')

    def test_unsupported_and_malformed(self):
        for value in [None, [], 1, 'manifest', {}]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                cm.validate_manifest(value)
        for field, value in [('schema_version',2),('schema_version',True),('pool_id','standard'),
                             ('oracle_revision','latest'),('rules_version','latest'),('cards',[]),
                             ('decks',[]),('matchups',[])]:
            with self.subTest(field=field,value=value):
                self.reject(lambda m: m.update({field:value}))
        self.reject(lambda m: m.update(artwork='forbidden'))
        self.reject(lambda m: m['cards'][0].update(name='Black Lotus'))
        self.reject(lambda m: m['cards'][0].update(behavior_status='implemented'))
        self.reject(lambda m: m['cards'][0].update(behavior_id='arbitrary-script'))
        self.reject(lambda m: m['cards'][0].update(oracle_text='not admitted'))
        self.reject(lambda m: m['cards'][0]['printing'].update(set='lea'))
        self.reject(lambda m: m['cards'][0]['printing'].update(lang='de'))
        self.reject(lambda m: m['cards'][0]['printing'].update(layout='transform'))
        self.reject(lambda m: m['cards'][0]['characteristics'].update(power=2))
        self.reject(lambda m: m['cards'][0]['characteristics'].update(keywords=['First strike']))
        self.reject(lambda m: m['cards'][0].update(retrieval_date='2026-02-30'))
        self.reject(lambda m: m['cards'][0].update(source_url='https://example.com'))
        self.reject(lambda m: m['cards'][0].update(oracle_text_sha256='bad'))

    def test_duplicates_and_missing(self):
        for field in ['id','oracle_id','behavior_id']:
            with self.subTest(field=field):
                self.reject(lambda m: m['cards'][1].update({field:m['cards'][0][field]}))
        self.reject(lambda m: m['cards'][1]['printing'].update(id=m['cards'][0]['printing']['id']))
        self.reject(lambda m: m['cards'].pop())
        self.reject(lambda m: m['cards'].append(m['cards'][0]))
        self.reject(lambda m: m['decks'].append(m['decks'][0]))
        self.reject(lambda m: m['decks'][0]['cards'].append(m['decks'][0]['cards'][0]))
        self.reject(lambda m: m['matchups'].append(m['matchups'][0]))

    def test_wrong_totals_and_multiplicities(self):
        for field in ['total_cards','lands','spells']:
            self.reject(lambda m: m['decks'][0].update({field:99}))
        for value in [0,-1,True,4.0,'4',17]:
            self.reject(lambda m: m['decks'][0]['cards'][0].update(copies=value))
        def balanced_wrong(m):
            m['decks'][0]['cards'][1]['copies'] += 1
            m['decks'][0]['cards'][2]['copies'] -= 1
        self.reject(balanced_wrong)
        self.reject(lambda m: m['decks'][0]['cards'][0].update(card_id='goblin-token'))
        self.reject(lambda m: m['matchups'][0].update(starting_seat=True))
        self.reject(lambda m: m['matchups'][0].update(starting_seat=2))
        self.reject(lambda m: m['matchups'][0].update(seats=['red','blue']))

    def test_record_and_file_digest_mismatch(self):
        self.manifest['cards'][0]['characteristics']['power'] = '9'
        with self.assertRaisesRegex(ValueError, 'digest'):
            cm.validate_manifest(self.manifest)
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'manifest.json'
            path.write_bytes(cm.DEFAULT_MANIFEST.read_bytes() + b' ')
            path.with_suffix('.sha256').write_text(cm.DEFAULT_MANIFEST.with_suffix('.sha256').read_text())
            with self.assertRaisesRegex(ValueError, 'digest'):
                cm.load_manifest(path)

    def test_every_frozen_card_rejects_changed_source_and_content_hashes(self):
        # GH-132: keeping an identity does not permit stale source pins. Exercise
        # every record, including all six supported cards and the reserved pool.
        for index, card in enumerate(self.manifest['cards']):
            for field in ('content_sha256', 'source_projection_sha256',
                          'oracle_text_sha256', 'raw_response_sha256'):
                with self.subTest(card=card['id'], field=field):
                    changed = copy.deepcopy(self.manifest)
                    value = card[field]
                    changed['cards'][index][field] = ('0' if value[0] != '0' else '1') + value[1:]
                    with self.assertRaisesRegex(ValueError, 'content digest mismatch'):
                        cm.validate_manifest(changed)

    def test_duplicate_json_keys_and_nonfinite(self):
        for raw in [b'{"schema_version":1,"schema_version":1}', b'{"x":NaN}',b'{bad']:
            with tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / 'bad.json'
                path.write_bytes(raw)
                path.with_suffix('.sha256').write_text(hashlib.sha256(raw).hexdigest())
                with self.assertRaises(ValueError):
                    cm.load_manifest(path)

    def test_synthetic_source_integrity_and_identity(self):
        card = next(c for c in self.manifest['cards'] if c['name'] == 'Bear Cub')
        source = dict(card['printing'], oracle_id=card['oracle_id'], name='Bear Cub', oracle_text='',
                      **card['characteristics'])
        raw = json.dumps(source).encode()
        cm.verify_source(card,raw)
        for field, value in [('name','Other'),('id',self.manifest['cards'][0]['printing']['id']),
                             ('oracle_text','changed'),('power','8'),('set','lea')]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                cm.verify_source(card,json.dumps(dict(source,**{field:value})).encode())
        for raw in [b'bad',b'[]',b'{}']:
            with self.assertRaises(ValueError):
                cm.verify_source(card,raw)

    def test_offline_load(self):
        with patch('subprocess.run', side_effect=AssertionError('offline means no subprocess')):
            cm.load_manifest()

    def test_live_receipt_covers_exact_pins(self):
        # Historical metadata evidence only; this test does not re-fetch sources.
        receipt = json.loads(cm.DEFAULT_MANIFEST.with_suffix('.verification.json').read_bytes())
        self.assertEqual(receipt['pool_id'], self.manifest['pool_id'])
        self.assertEqual(receipt['revision'], self.manifest['revision'])
        self.assertEqual(len(receipt['cards']), 21)
        self.assertEqual({r['card_id'] for r in receipt['cards']},
                         {c['id'] for c in self.manifest['cards']})
        for record in receipt['cards']:
            card = next(c for c in self.manifest['cards'] if c['id'] == record['card_id'])
            self.assertEqual(record['source_url'], card['source_url'])
            self.assertEqual(record['source_projection_sha256'], card['source_projection_sha256'])
            self.assertGreater(record['byte_count'], 0)


if __name__ == '__main__':
    unittest.main()
