"""Fail-closed integrity and replay checks for the provisional XMage pilot."""
import argparse
import hashlib
import json
import lzma
import os
from pathlib import Path
import tempfile
from unittest.mock import patch

import xmage_ai_normalize as normalize
import xmage_ai_pilot as pilot

ROOT=Path(__file__).resolve().parents[1]
DEFAULT=ROOT/'doc/evidence/xmage-ai-pilot/pilot.json'


def require(ok,message):
    if not ok:
        raise ValueError('pilot: '+message)


def artifact(base,manifest,name):
    path=Path(name)
    require(not path.is_absolute() and '..' not in path.parts,'artifact path')
    require(name in manifest['artifacts'],'unlisted artifact '+name)
    data=(base/path).read_bytes()
    require(hashlib.sha256(data).hexdigest()==manifest['artifacts'][name],'artifact checksum '+name)
    return data


def document(base,manifest,name):
    data=artifact(base,manifest,name)
    return json.loads(lzma.decompress(data) if name.endswith('.xz') else data)


def validate(path=DEFAULT):
    path=Path(path);base=path.parent;manifest=json.loads(path.read_text())
    require(manifest['schema_version']==1,'schema version')
    require(manifest['status']=='provisional-native-patched-xmage','qualification scope')
    require(manifest['final_admission']=={'status':'not-admitted','overlap_ids':[]},'final admission claim')
    require(len(manifest['games'])==10,'ten complete distinct games')
    for name in manifest['artifacts']:
        artifact(base,manifest,name)
    rows=set();seen={};attempt_ids=set()
    for game in manifest['games']:
        require(game['attempt'] not in attempt_ids,'duplicate attempt');attempt_ids.add(game['attempt'])
        converted=document(base,manifest,game['converted'])
        transcript=converted['transcript'];case=transcript['cases'][0]
        canonical=pilot.canonical_game(transcript);identity=hashlib.sha256(canonical).hexdigest()
        require(identity==game['id'],'game content identity')
        if identity in seen:
            require(seen[identity]==canonical,'hash collision')
            raise ValueError('pilot: duplicate canonical game')
        seen[identity]=canonical
        row=(''.join(d['deck'][0].upper() for d in case['decks']),case['starter'])
        require(row==(game['row'],game['starter']),'row binding');rows.add(row)
        require(case['stop']=='terminal' and case['play'][-1]['kind']=='finish','terminal transcript')
        require(len(game['native'])==len(game['reference'])==2
                and len(set(game['native']))==len(set(game['reference']))==2,'twice per engine')
        require(len(game['negative_results'])==2 and
                {Path(name).name for name in game['negative_results']}==
                {'native-negatives.json.xz','xmage-negatives.json.xz'},'both negative result streams')
        receipt=document(base,manifest,game['receipt'])
        require(receipt['status']=='passed' and receipt['reference']=='patched-combat-sba-v1','qualification receipt')
        require(receipt['game_identity_sha256']==identity,'receipt identity')
        require(len(receipt['repeats'])==2,'repeat receipt')
        require(receipt['source_sha256']==manifest['artifacts'][game['source']],'source receipt binding')
        normalize.validate_provenance(base/game['source'],converted['source_receipt'])
    require(rows=={(row,seat) for row in ('RG','GR','RR','GG') for seat in (0,1)},'all eight rows')
    attempts=manifest['attempts']
    require(len({a['id'] for a in attempts})==len(attempts),'unique attempt inventory')
    require({a['id'] for a in attempts if a['status']=='qualified'}==attempt_ids,
            'complete qualified attempt accounting')
    for attempt in attempts:
        require(attempt['status'] in {'qualified','failed','truncated','unsupported','divergent','duplicate'},
                'attempt verdict')
        artifact(base,manifest,attempt['source'])
        if attempt['status']!='qualified':
            require(bool(attempt['earliest_difference']),'retained first difference')
    probes=manifest['duplicate_probes']
    require(bool(probes),'duplicate controls')
    import copy
    for probe in probes:
        game=next(g for g in manifest['games'] if g['id']==probe['original'])
        transcript=document(base,manifest,game['converted'])['transcript']
        altered=copy.deepcopy(transcript)
        altered['provenance']=probe['altered_provenance']
        altered['cases'][0]['id']=probe['altered_case_id']
        require(pilot.canonical_game(transcript)==pilot.canonical_game(altered),
                'altered provenance must not change content')
        require(probe['counted']==0,'duplicate quota')
    return base,manifest


def check_retained(path=DEFAULT):
    base,manifest=validate(path)
    for game in manifest['games']:
        converted=document(base,manifest,game['converted'])
        negatives=document(base,manifest,game['negatives'])
        original=json.loads((ROOT/'fixtures/reference/xmage-ai-negative.json').read_text())
        require(all(negatives.get(k)==v for k,v in original.items()),'preserved original negative controls')
        expected=dict(original,**pilot.source_negative_inputs(converted))
        require(negatives==expected,'source-specific negative controls')
        for name in game['negative_results']:
            rejected=document(base,manifest,name)
            for key,spec in negatives.items():
                require(key in rejected and bool(rejected[key].get('error')),'negative rejection '+key)
                if spec['sequence'] is not None:
                    require(len(rejected[key]['run']['consumed_play'])==spec['sequence'],
                            'negative earliest cursor '+key)
        for native,reference in zip(game['native'],game['reference']):
            normalize.verify_pair(converted,document(base,manifest,native)['run'],
                                  document(base,manifest,reference))
    return manifest


def replay_native(path=DEFAULT,output=None):
    """Every provisional game, twice, with full source/semantic assertions."""
    base,manifest=validate(path)
    with tempfile.TemporaryDirectory() as temporary:
        root=Path(output) if output else Path(temporary)
        root.mkdir(parents=True,exist_ok=True)
        for game in manifest['games']:
            folder=root/game['attempt'];folder.mkdir(exist_ok=False)
            converted=document(base,manifest,game['converted'])
            tape=folder/'input.json';tape.write_text(json.dumps(converted['transcript']))
            negatives=folder/'negatives.json'
            negatives.write_text(json.dumps(document(base,manifest,game['negatives'])))
            with patch.dict(os.environ,{'MTG_AI_REPLAY_INPUT':str(tape.resolve()),
                    'MTG_AI_REPLAY_NEGATIVES':str(negatives.resolve())}):
                for run in normalize.run_native(folder):
                    normalize.verify_native(converted,run)
    return manifest


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--manifest',type=Path,default=DEFAULT)
    parser.add_argument('--native-output',type=Path)
    args=parser.parse_args()
    manifest=replay_native(args.manifest,args.native_output) if args.native_output else check_retained(args.manifest)
    print(f"Verified {len(manifest['games'])} provisional games; no all-three-engine quota claim.")


if __name__=='__main__':
    main()
