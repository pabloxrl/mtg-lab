"""Original integrity cases derived from GH-9 and RFC 0002 §3, not game rules."""
import hashlib
import json
from pathlib import Path
import sys
import subprocess
import tempfile
from unittest.mock import patch
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
import rules_source as rules

# Synthetic transport payload: never an official rules document or verification.
PAYLOAD = (b'Magic: The Gathering Comprehensive Rules\n\n'
           b'These rules are effective as of September 25, 2026.\n'
           b'SYNTHETIC integrity fixture; no game rules.\n')
URL = 'https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt'


def manifest():
    return dict(schema_version=1, rules_version='cr-2026-09-25', source_url=URL,
                effective_date='2026-09-25', retrieval_date='2026-09-26',
                sha256=hashlib.sha256(PAYLOAD).hexdigest())


class IntegrityTests(unittest.TestCase):
    def test_valid(self):
        rules.validate_manifest(manifest())
        rules.verify_document(manifest(), PAYLOAD)

    def test_invalid_fields(self):
        bad_values = {
            'schema_version': [True, 2, '1', None],
            'rules_version': ['', 'cr-2025-09-25', None],
            'source_url': ['', 'http://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt',
                           URL + '?new=1', 'https://example.com/rules.txt', None],
            'effective_date': ['2026-09-24', '2026-02-30', '20260925', None],
            'retrieval_date': ['2026-09-24', '2026-02-30', '26-09-2026', None],
            'sha256': ['', 'z' * 64, 'a' * 63, 'A' * 64, None],
        }
        for key, values in bad_values.items():
            for value in values:
                with self.subTest(key=key, value=value):
                    candidate = manifest()
                    candidate[key] = value
                    with self.assertRaises(ValueError):
                        rules.validate_manifest(candidate)
        for key in manifest():
            candidate = manifest()
            del candidate[key]
            with self.subTest(missing=key), self.assertRaises(ValueError):
                rules.validate_manifest(candidate)
        for candidate in ([], None, dict(manifest(), typo=True)):
            with self.subTest(candidate=candidate), self.assertRaises(ValueError):
                rules.validate_manifest(candidate)

    def test_wrong_digest(self):
        with self.assertRaisesRegex(ValueError, 'SHA-256'):
            rules.verify_document(manifest(), PAYLOAD + b'changed')

    def test_wrong_effective_date_even_with_matching_digest(self):
        payload = PAYLOAD.replace(b'September 25', b'September 24')
        candidate = manifest()
        candidate['sha256'] = hashlib.sha256(payload).hexdigest()
        with self.assertRaisesRegex(ValueError, 'effective date'):
            rules.verify_document(candidate, payload)

    def test_not_a_rules_header(self):
        for payload in (b'', b'<html>error</html>', PAYLOAD.replace(b'Magic:', b'Not Magic:'),
                        PAYLOAD.replace(b'These rules', b'Old rules')):
            candidate = manifest()
            candidate['sha256'] = hashlib.sha256(payload).hexdigest()
            with self.subTest(payload=payload), self.assertRaises(ValueError):
                rules.verify_document(candidate, payload)


class TransportTests(unittest.TestCase):
    def transport(self, command, **kwargs):
        self.assertEqual(command[:2], ['curl', '--disable'])
        self.assertEqual(command[-1], URL)
        self.assertEqual(kwargs['stdin'], subprocess.DEVNULL)
        self.assertEqual(kwargs['timeout'], 35)
        self.assertEqual(command[command.index('--max-time') + 1], '30')
        self.assertEqual(command[command.index('--connect-timeout') + 1], '10')
        self.assertIn('--max-filesize', command)
        self.assertNotIn('--location', command)
        self.download = Path(command[command.index('--output') + 1])
        self.download.write_bytes(PAYLOAD)
        return subprocess.CompletedProcess(command, 0, b'200', b'')

    def test_synthetic_fetch_success_and_cleanup(self):
        with patch.object(rules.subprocess, 'run', side_effect=self.transport):
            receipt = rules.fetch_verify(manifest())
        self.assertEqual(receipt['sha256'], hashlib.sha256(PAYLOAD).hexdigest())
        self.assertEqual(receipt['byte_count'], len(PAYLOAD))
        self.assertFalse(self.download.exists())

    def test_unavailable_source(self):
        for result in (subprocess.CompletedProcess([], 22, b'404', b'not found'),
                       subprocess.CompletedProcess([], 28, b'000', b'timed out'),
                       subprocess.CompletedProcess([], 0, b'301', b'')):
            with self.subTest(result=result), patch.object(rules.subprocess, 'run', return_value=result):
                with self.assertRaisesRegex(ValueError, 'source unavailable'):
                    rules.fetch_verify(manifest())
        for error in (FileNotFoundError('curl'), subprocess.TimeoutExpired('curl', 35)):
            with self.subTest(error=error), patch.object(rules.subprocess, 'run', side_effect=error):
                with self.assertRaisesRegex(ValueError, 'source unavailable'):
                    rules.fetch_verify(manifest())

    def test_transport_wrong_digest_and_date(self):
        def response(payload):
            def run(command, **kwargs):
                result = self.transport(command, **kwargs)
                self.download.write_bytes(payload)
                return result
            return run
        payload = PAYLOAD.replace(b'September 25', b'September 24')
        for content, digest, message in (
            (PAYLOAD + b'corrupt', manifest()['sha256'], 'SHA-256'),
            (payload, hashlib.sha256(payload).hexdigest(), 'effective date'),
        ):
            candidate = manifest()
            candidate['sha256'] = digest
            with patch.object(rules.subprocess, 'run', side_effect=response(content)):
                with self.assertRaisesRegex(ValueError, message):
                    rules.fetch_verify(candidate)
            self.assertFalse(self.download.exists())

    def test_invalid_manifest_never_fetches(self):
        with patch.object(rules.subprocess, 'run') as run:
            with self.assertRaises(ValueError):
                rules.fetch_verify({})
            run.assert_not_called()

    def test_empty_and_oversize_transport(self):
        for payload in (b'', b'x' * (rules.MAX_BYTES + 1)):
            def run(command, **kwargs):
                result = self.transport(command, **kwargs)
                self.download.write_bytes(payload)
                return result
            with patch.object(rules.subprocess, 'run', side_effect=run):
                with self.assertRaisesRegex(ValueError, 'empty or exceeds'):
                    rules.fetch_verify(manifest())


class OfflineTests(unittest.TestCase):
    def test_committed_pin_and_receipt(self):
        with patch.object(rules.subprocess, 'run', side_effect=AssertionError('offline only')):
            pin = rules.load_manifest(rules.DEFAULT_MANIFEST)
        self.assertEqual(pin['effective_date'], '2026-09-25')
        self.assertEqual(pin['source_url'], URL)
        receipt = json.loads((rules.ROOT / 'data/rules/cr-2026-09-25.verification.json').read_text())
        for key, value in pin.items():
            self.assertEqual(receipt[key], value)
        self.assertEqual(receipt['byte_count'], 977752)
        self.assertEqual(receipt['verification'], 'official-source-fetch-sha256-and-header')

    def test_duplicate_fields_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'pin.json'
            path.write_text(json.dumps(manifest())[:-1] + ', "schema_version": 1}')
            with self.assertRaisesRegex(ValueError, 'duplicate'):
                rules.load_manifest(path)

    def test_cli_unattended_offline_and_missing_manifest(self):
        command = [sys.executable, str(rules.ROOT / 'scripts/rules_source.py'), 'validate']
        result = subprocess.run(command, stdin=subprocess.DEVNULL, capture_output=True,
                                timeout=5, env={'PATH': '', 'DISPLAY': ''})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)['validation'], 'offline-metadata-only')
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(command + ['--manifest', str(Path(directory) / 'missing')],
                                    stdin=subprocess.DEVNULL, capture_output=True, timeout=5)
        self.assertEqual(result.returncode, 1)
        self.assertIn(b'Rules source verification failed', result.stderr)


if __name__ == '__main__':
    unittest.main()
