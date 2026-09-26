"""Offline rules metadata validation and explicit, bounded official-source verification."""
import argparse
from datetime import date, datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_MANIFEST = ROOT / 'data/rules/cr-2026-09-25.json'
SOURCE_URL = 'https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt'
EFFECTIVE_DATE = '2026-09-25'
MAX_BYTES = 4 * 1024 * 1024
FETCH_SECONDS = 30
FIELDS = {'schema_version', 'rules_version', 'source_url', 'effective_date',
          'retrieval_date', 'sha256'}


def validate_manifest(manifest):
    """Accept only this deliberately selected revision and a complete schema."""
    if not isinstance(manifest, dict) or set(manifest) != FIELDS:
        raise ValueError('manifest must contain exactly: ' + ', '.join(sorted(FIELDS)))
    if type(manifest['schema_version']) is not int or manifest['schema_version'] != 1:
        raise ValueError('unsupported schema_version; expected integer 1')
    if manifest['rules_version'] != 'cr-' + EFFECTIVE_DATE:
        raise ValueError('rules_version must be cr-' + EFFECTIVE_DATE)
    if manifest['source_url'] != SOURCE_URL:
        raise ValueError('source_url must match the exact official RFC URL: ' + SOURCE_URL)
    if manifest['effective_date'] != EFFECTIVE_DATE:
        raise ValueError('effective_date must match RFC 0002: ' + EFFECTIVE_DATE)
    retrieved = manifest['retrieval_date']
    if not isinstance(retrieved, str) or not re.fullmatch(r'\d{4}-\d{2}-\d{2}', retrieved):
        raise ValueError('retrieval_date must be an ISO YYYY-MM-DD date')
    if date.fromisoformat(retrieved) < date.fromisoformat(EFFECTIVE_DATE):
        raise ValueError('retrieval_date precedes the rules effective date')
    digest = manifest['sha256']
    if not isinstance(digest, str) or not re.fullmatch(r'[0-9a-f]{64}', digest):
        raise ValueError('sha256 must be 64 lowercase hexadecimal characters')


def load_manifest(path):
    def unique_fields(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('duplicate manifest field: ' + key)
            result[key] = value
        return result
    manifest = json.loads(path.read_text(encoding='utf-8'), object_pairs_hook=unique_fields)
    validate_manifest(manifest)
    return manifest


def verify_document(manifest, document):
    """Verify raw-byte hash and the publication header independently."""
    validate_manifest(manifest)
    if not document or len(document) > MAX_BYTES:
        raise ValueError('source document is empty or exceeds the 4 MiB limit')
    if hashlib.sha256(document).hexdigest() != manifest['sha256']:
        raise ValueError('SHA-256 mismatch; retain the pin and investigate source changes')
    try:
        header = document[:1024].decode('utf-8-sig').splitlines()
    except UnicodeDecodeError as error:
        raise ValueError('source header is not UTF-8') from error
    lines = [line.strip() for line in header if line.strip()]
    if not lines or lines[0] != 'Magic: The Gathering Comprehensive Rules':
        raise ValueError('source is not a Comprehensive Rules document')
    if len(lines) < 2 or lines[1] != 'These rules are effective as of September 25, 2026.':
        raise ValueError('source effective date does not match RFC 0002 (2026-09-25)')


def fetch_document(url):
    """Use curl with a total deadline (including DNS), no config or redirects."""
    if url != SOURCE_URL:
        raise ValueError('refusing a source other than the exact official RFC URL')
    with tempfile.TemporaryDirectory(prefix='mtg-rules-') as directory:
        output = Path(directory) / 'rules.txt'
        command = ['curl', '--disable', '--silent', '--show-error', '--fail',
                   '--proto', '=https', '--connect-timeout', '10',
                   '--max-time', str(FETCH_SECONDS), '--max-filesize', str(MAX_BYTES),
                   '--output', str(output), '--write-out', '%{http_code}', url]
        try:
            result = subprocess.run(command, stdin=subprocess.DEVNULL, capture_output=True,
                                    timeout=FETCH_SECONDS + 5, check=False)
        except (OSError, subprocess.TimeoutExpired) as error:
            raise ValueError('source unavailable: install curl/check connectivity and retry; '
                             'the rules pin was not changed') from error
        if result.returncode or result.stdout != b'200':
            raise ValueError(f'source unavailable: curl exit {result.returncode}, '
                             f'HTTP {result.stdout.decode(errors="replace")}; '
                             'check the official URL/connectivity and retry; do not substitute revisions')
        try:
            with output.open('rb') as stream:
                document = stream.read(MAX_BYTES + 1)
        except OSError as error:
            raise ValueError('source unavailable: curl produced no readable document') from error
        if not document or len(document) > MAX_BYTES:
            raise ValueError('source document is empty or exceeds the 4 MiB limit')
        return document


def fetch_verify(manifest):
    validate_manifest(manifest)
    document = fetch_document(manifest['source_url'])
    verify_document(manifest, document)
    return dict(manifest, verified_at=datetime.now(timezone.utc).isoformat(),
                byte_count=len(document), verification='official-source-fetch-sha256-and-header')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['validate', 'fetch-verify'])
    parser.add_argument('--manifest', type=Path, default=DEFAULT_MANIFEST)
    args = parser.parse_args()
    try:
        manifest = load_manifest(args.manifest)
        result = (fetch_verify(manifest) if args.command == 'fetch-verify'
                  else dict(validation='offline-metadata-only', **manifest))
    except (ValueError, OSError) as error:
        parser.exit(1, f'Rules source verification failed: {error}\n')
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == '__main__':
    main()
