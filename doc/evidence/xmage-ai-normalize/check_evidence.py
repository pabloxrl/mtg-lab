"""Validate retained actual consumers and source bindings; never regenerate goldens."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import tarfile

ROOT=Path(__file__).resolve().parents[3]
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'scripts'))
import xmage_ai_normalize as normalize


def check(current=False):
    for name,expected in json.loads((HERE/'integrity.json').read_text()).items():
        if hashlib.sha256((HERE/name).read_bytes()).hexdigest()!=expected:
            raise ValueError('evidence checksum: '+name)
    receipt=json.loads((HERE/'acceptance.json').read_text())
    if receipt['status']!='passed' or len(receipt['repeats'])!=2:
        raise ValueError('missing actual repeated acceptance')
    with tarfile.open(HERE/'execution-sources.tar.xz') as archive:
        for name,expected in receipt['sources'].items():
            if hashlib.sha256(archive.extractfile(name).read()).hexdigest()!=expected:
                raise ValueError('execution source checksum: '+name)
            if current and hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=expected:
                raise ValueError('current execution source changed: '+name)
    with tarfile.open(HERE/'runs.tar.xz') as archive:
        def read(name):return json.load(archive.extractfile('acceptance-4/'+name))
        if read('receipt.json')!=receipt:
            raise ValueError('receipt mismatch')
        for name,expected in receipt['artifacts'].items():
            if not name.endswith('.json'):
                continue  # unchecked raw Maven/compiler logs remain private
            if hashlib.sha256(archive.extractfile('acceptance-4/'+name).read()).hexdigest()!=expected:
                raise ValueError('run checksum: '+name)
        converted=read('converted.json')
        source=ROOT/'doc/evidence/xmage-ai-capture/acceptance-005/game/trace.jsonl.xz'
        normalize.validate_provenance(source,converted['source_receipt'])
        for repeat in (1,2):
            normalize.verify_pair(converted,read(f'native-{repeat}.json')['run'],
                                  read(f'L0-XMage-captured.xmage-{repeat}.json'))
    print('Actual source, two repeated engine pairs, intermediate ledgers and artifact hashes passed.')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--current',action='store_true')
    check(parser.parse_args().current)
