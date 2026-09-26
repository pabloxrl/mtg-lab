"""Synthetic acquisition failures are never counted as live source evidence."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1] / 'scripts'))
import acquire_cards as ac
import card_manifest as cm


class AcquisitionTests(unittest.TestCase):
    def setUp(self):
        self.manifest = cm.load_manifest()
        self.card = next(c for c in self.manifest['cards'] if c['name'] == 'Bear Cub')
        self.raw = json.dumps(dict(self.card['printing'],oracle_id=self.card['oracle_id'],
                                   name='Bear Cub',oracle_text='',**self.card['characteristics'])).encode()

    def response(self, command, **kwargs):
        self.assertEqual(kwargs['stdin'],subprocess.DEVNULL)
        self.assertLessEqual(kwargs['timeout'],35)
        self.assertEqual(command[1],'--disable')
        self.assertNotIn('--location',command)
        self.assertIn('--max-filesize',command)
        self.assertIn('Accept: application/json',command)
        self.assertEqual(command[-1],self.card['source_url'])
        Path(command[command.index('--output')+1]).write_bytes(self.raw)
        return subprocess.CompletedProcess(command,0,b'200',b'')

    def test_bounded_fetch(self):
        with patch('subprocess.run',side_effect=self.response):
            self.assertEqual(ac.fetch_card(self.card),self.raw)

    def test_failures_never_fallback(self):
        for code,status in [(22,b'404'),(0,b'302'),(0,b'429'),(0,b'500')]:
            with self.subTest(code=code,status=status), patch('subprocess.run',return_value=
                    subprocess.CompletedProcess([],code,status,b'')), self.assertRaises(ValueError):
                ac.fetch_card(self.card)
        for error in [FileNotFoundError(),subprocess.TimeoutExpired('curl',35)]:
            with patch('subprocess.run',side_effect=error), self.assertRaises(ValueError):
                ac.fetch_card(self.card)
        self.raw = b'{}'
        with patch('subprocess.run',side_effect=self.response), self.assertRaises(ValueError):
            ac.fetch_card(self.card)

    def test_external_cache_and_no_success_receipt_after_failure(self):
        with self.assertRaises(ValueError):
            ac.acquire(self.manifest,ac.ROOT/'raw-cache')
        with tempfile.TemporaryDirectory() as directory:
            cache = Path(directory)/'new'
            with patch.object(ac,'fetch_card',side_effect=ValueError('unavailable')):
                with self.assertRaises(ValueError):
                    ac.acquire(self.manifest,cache)
            self.assertFalse((cache/'verified.json').exists())
            with self.assertRaises(FileExistsError):
                ac.acquire(self.manifest,cache)

    def test_cli_offline_and_missing_cache_unattended(self):
        script = str(ac.ROOT/'scripts/card_manifest.py')
        for args,code in [([],0),(['--cache','/nonexistent/mtg-card-cache'],1)]:
            result = subprocess.run([sys.executable,script]+args,stdin=subprocess.DEVNULL,
                                    capture_output=True,timeout=10)
            self.assertEqual(result.returncode,code,result.stderr)
            if code:
                self.assertIn(b'validation failed',result.stderr)
            else:
                self.assertEqual(json.loads(result.stdout)['validation'],'offline-metadata-only')


if __name__ == '__main__':
    unittest.main()
