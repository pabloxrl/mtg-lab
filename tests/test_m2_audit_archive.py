"""Original unsuccessful audit artifacts must remain byte-for-byte evidence."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
ARCHIVE = ROOT / 'doc/evidence/m2-audit-repair-registration'


class AuditArchiveTests(unittest.TestCase):
    def test_supplied_originals_retain_sizes_and_hashes(self):
        rows = json.loads((ARCHIVE / 'archive-hashes.json').read_text())
        self.assertEqual(len(rows), 11)
        by_supplied = {row['supplied_path']: row for row in rows}
        original = json.loads((ARCHIVE / 'originals/manifest.json').read_text())
        self.assertEqual(original['audit_commit'],
                         'ff4d0d455209c6a3dda2907e011cb25c52b1cc43')
        for receipt in original['files']:
            row = by_supplied[receipt['supplied']]
            self.assertEqual(row['sha256'], receipt['sha256'])
            self.assertEqual(row['bytes'], receipt['bytes'])
        for row in rows:
            with self.subTest(path=row['path']):
                data = (ARCHIVE / 'originals' / row['path']).read_bytes()
                self.assertEqual(len(data), row['bytes'])
                self.assertEqual(hashlib.sha256(data).hexdigest(), row['sha256'])
                # A byte mutation must not match the retained evidence receipt.
                self.assertNotEqual(hashlib.sha256(data + b'\n').hexdigest(),
                                    row['sha256'])
