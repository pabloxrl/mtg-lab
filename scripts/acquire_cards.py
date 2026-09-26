"""Explicit, bounded Scryfall acquisition into an external cache; never runtime I/O."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import time

from card_manifest import DEFAULT_MANIFEST, MAX_BYTES, load_manifest, require, verify_source

ROOT = Path(__file__).resolve().parents[1]
FETCH_SECONDS = 30


def fetch_card(card):
    # URL comes only from an already validated card record.
    url = 'https://api.scryfall.com/cards/' + card['printing']['id']
    require(card['source_url'] == url, 'unexpected source URL')
    with tempfile.TemporaryDirectory(prefix='mtg-card-') as directory:
        output = Path(directory) / 'response.json'
        command = ['curl','--disable','--silent','--show-error','--fail','--proto','=https',
                   '--connect-timeout','10','--max-time',str(FETCH_SECONDS),
                   '--max-filesize',str(MAX_BYTES),'--header',
                   'User-Agent: mtg-lab/0.1 (research; https://github.com/pabloxrl/mtg-lab)',
                   '--header','Accept: application/json','--output',str(output),
                   '--write-out','%{http_code}',url]
        try:
            result = subprocess.run(command,stdin=subprocess.DEVNULL,capture_output=True,
                                    timeout=FETCH_SECONDS+5,check=False)
        except (OSError,subprocess.TimeoutExpired) as error:
            raise ValueError('source unavailable: curl missing or timed out; pins unchanged') from error
        require(result.returncode == 0 and result.stdout == b'200',
                f'source unavailable: curl exit {result.returncode}, HTTP {result.stdout!r}; pins unchanged')
        with output.open('rb') as stream:
            raw = stream.read(MAX_BYTES+1)
        verify_source(card,raw)
        return raw


def acquire(manifest, cache):
    from card_manifest import validate_manifest
    validate_manifest(manifest)
    cache = Path(cache).resolve()
    require(cache != ROOT and ROOT not in cache.parents, 'raw source cache must be outside the repository')
    # New directory only: an interrupted/failed attempt cannot look like a
    # complete cache and an existing experiment's source bytes stay untouched.
    cache.mkdir(parents=True,exist_ok=False)
    receipts = []
    for card in manifest['cards']:
        raw = fetch_card(card)
        filename = card['printing']['id'] + '.json'
        with (cache / filename).open('xb') as stream:
            stream.write(raw)
        receipts.append({'card_id':card['id'],'source_url':card['source_url'],
                         'raw_response_sha256':hashlib.sha256(raw).hexdigest(),
                         'byte_count':len(raw),'source_projection_sha256':card['source_projection_sha256']})
        time.sleep(.1)  # At most ten requests/second; no automatic retries.
    receipt = {'schema_version':1,'pool_id':manifest['pool_id'],'revision':manifest['revision'],
               'verified_at':datetime.now(timezone.utc).isoformat(),'cards':receipts}
    (cache / 'verified.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest',type=Path,default=DEFAULT_MANIFEST)
    parser.add_argument('--cache',type=Path,required=True,help='new directory outside the repository')
    args = parser.parse_args()
    try:
        receipt = acquire(load_manifest(args.manifest),args.cache)
    except (ValueError,OSError) as error:
        parser.exit(1,f'Card acquisition failed: {error}\n')
    print(json.dumps(receipt,indent=2))


if __name__ == '__main__':
    main()
