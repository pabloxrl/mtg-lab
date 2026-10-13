"""Strict opening prefixes from immutable complete Forge AI source games."""
from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path

import forge_ai_capture as capture
import xmage

ROOT = Path(__file__).resolve().parents[1]
OPENING = {'cheatShuffle', 'complainCardsCantPlayWell', 'revealUnsupported',
           'awaitNextInput', 'chooseStartingPlayer', 'mulliganKeepHand',
           'tuckCardsViaMulligan', 'willPutCardOnTop'}


def need(ok, cursor, field):
    if not ok:
        raise ValueError(f'first divergence: /source/{cursor}/{field}')


def extract(path, verify_receipt=True):
    path = Path(path)
    selected_decks = ['red', 'green']
    selected_profile = False
    if verify_receipt:
        receipt = json.loads(path.with_name('receipt.json').read_text())
        need(receipt['status']=='complete' and receipt['started_game'], 0, 'source completion')
        need(receipt['artifacts'][path.name]==xmage.sha(path), 0, 'source tape hash')
        for key, file in [('pins_sha256','references/forge/pins.json'),
                          ('dependencies_sha256','references/forge/dependencies.json'),
                          ('card_manifest_sha256','data/cards/foundations_micro_v1.json')]:
            need(receipt[key]==xmage.sha(ROOT/file),0,'source pin '+key)
        need(receipt['upstream_commit']=='95dc682bf92460f49cebd7a9578f06ccf60d5569',0,'upstream pin')
        selected_profile = receipt.get('source_profile') == 'frozen-deck-selection-v1'
        need(receipt.get('source_profile') in (None, 'frozen-deck-selection-v1'), 0, 'unknown source profile')
        runner = 'scripts/forge_casting_capture.py' if selected_profile else 'scripts/forge_ai_capture.py'
        if selected_profile:
            selected_decks = receipt['decks']
            need(len(selected_decks) == 2 and all(d in ('red', 'green') for d in selected_decks), 0, 'selected decks')
            need(receipt['capture_base_sha256'] == xmage.sha(ROOT/'scripts/forge_ai_capture.py'), 0, 'capture base pin')
            selected = path.with_name('selected-manifest.json')
            need(receipt['artifacts'][selected.name] == xmage.sha(selected), 0, 'selected manifest hash')
            expected = json.loads((ROOT/'data/cards/foundations_micro_v1.json').read_text())
            original = expected['decks']
            expected['decks'] = [next(d for d in original if d['id'] == key) for key in selected_decks]
            need(json.loads(selected.read_text()) == expected, 0, 'unchanged frozen deck selection')
        need(receipt['runner_sha256']==xmage.sha(ROOT/runner) and
             receipt['patch_runner_sha256']==xmage.sha(ROOT/'scripts/forge_capture_patch.py'),0,'capture implementation pin')
        for target,local in [(capture.RECORDER,'MtgRawCapture.java'),(capture.TEST,'RawAiCaptureTest.java')]:
            need(receipt['patches'][target]['modified']==xmage.sha(ROOT/'references/forge'/local),0,'capture bridge pin')
        need(receipt['mode']=='record' and receipt['rules_version']=='cr-2026-09-25' and
             receipt['decks']==selected_decks,0,'capture mode/rules/decks')
    manifest = json.loads((ROOT/'data/cards/foundations_micro_v1.json').read_text())
    names = {c['name']:c['id'] for c in manifest['cards']}
    decks = [dict(deck=d['id'], occurrences=[f'{a}/{c["card_id"]}/{i}'
             for c in d['cards'] for i in range(c['copies'])]) for a,d in enumerate(
                 next(d for d in manifest['decks'] if d['id'] == key) for key in selected_decks)]
    bindings, prefix, chance, choices, checkpoints = {}, [], [], [], []
    calls, shuffles, rounds, instances = [], {}, [0,0], {}
    paired, starter, endpoint = {}, None, None
    remainder_digest = hashlib.sha256(); remainder_count = 0; draws = 0
    inventory = {m['id']:m['name'] for ms in json.loads((ROOT/'references/forge/capture-callbacks.json').read_text()).values() for m in ms}

    def point(r, actor, boundary):
        ps = r['state']['players']; p = ps[actor]
        return dict(source=r['index'], state=dict(actor=actor, boundary=boundary,
            life=[q['life'] for q in ps], hand=sorted(bindings[c['id']] for c in p['zones']['Hand']),
            library=[bindings[c['id']] for c in p['zones']['Library']], mulligans=rounds[actor]))

    with gzip.open(path, 'rt') as stream:
        for index, line in enumerate(stream):
            if endpoint is not None:
                remainder_digest.update(line.encode()); remainder_count += 1
                continue
            r = json.loads(line); k = r['kind']
            need(r['index']==index,index,'record index')
            if k == 'enter' and r['method']=='chooseSpellAbilityToPlay':
                need(not calls and not shuffles and starter is not None,index,'unfinished opening')
                endpoint = r
                remainder_digest.update(line.encode()); remainder_count += 1
                for a in (starter,1-starter): checkpoints.append(point(r,a,'first_upkeep'))
                continue
            prefix.append(r)
            if k == 'random':
                draws += 1
                need(r['draw']==draws and 1<=r['bits']<=32,index,'random chronology')
            elif k == 'shuffle':
                a = r['actor']['player_id']; cards = r['library']; ids = [c['id'] for c in cards]
                need(len(ids)==40 and len(set(ids))==40,index,'duplicate physical or incomplete deck')
                if r['stage']=='before':
                    need(a not in shuffles,index,'nested shuffle')
                    if not any(v.startswith(str(a)+'/') for v in bindings.values()):
                        copies = Counter()
                        for c in cards:
                            name = names[c['name']]; key=f'{a}/{name}/{copies[name]}'
                            need(c['id'] not in bindings,index,'duplicate physical binding')
                            bindings[c['id']]=key; copies[name]+=1
                        need(set(bindings[i] for i in ids)==set(decks[a]['occurrences']),index,'frozen deck')
                    shuffles[a]=set(ids)
                else:
                    need(r['stage']=='after' and shuffles.pop(a,None)==set(ids),index,'shuffle permutation')
                    chance.append(dict(sequence=len(chance),kind='initial_shuffle' if len(chance)<2 else 'mulligan_shuffle',
                        actor=a,source=None,before=decks[a]['occurrences'],after=[bindings[i] for i in ids]))
            elif k == 'enter':
                need(inventory.get(r['method_id'])==r['method'] and r['method'] in OPENING,index,'unsupported opening callback')
                calls.append(r)
                if r['method'] in ('mulliganKeepHand','tuckCardsViaMulligan'):
                    checkpoints.append(point(r,r['actor']['player_id'],'declaration' if r['method']=='mulliganKeepHand' else 'bottom'))
            elif k == 'leave':
                need(calls and calls[-1]['call']==r['call'],index,'callback pairing')
                entry = calls.pop(); paired[entry['index']]=index
                a = entry['actor']['player_id']; method = entry['method']
                if method=='chooseStartingPlayer': starter=r['result']['player_id']
                if method in ('mulliganKeepHand','tuckCardsViaMulligan'):
                    value = ('keep' if r['result'] else 'mulligan') if method=='mulliganKeepHand' else [bindings[c['id']] for c in r['result']]
                    if method=='tuckCardsViaMulligan':
                        need(len(value)==rounds[a] and len(set(value))==len(value),index,'bottom occurrences/cardinality')
                    choices.append(dict(sequence=len(choices),kind='declare' if method=='mulliganKeepHand' else 'bottom',actor=a,source=None,round=rounds[a],selection=value))
                    if value=='mulligan': rounds[a]+=1
            elif k not in ('start','event','boundary'):
                need(False,index,'unexpected opening record')
            if 'state' in r:
                seen = set()
                for p in r['state']['players']:
                    for cards in p['zones'].values():
                        for c in cards:
                            need(c['id'] not in seen,index,'duplicate physical zone'); seen.add(c['id'])
                            # A source allocation is diagnostic, while timestamp
                            # identifies a zone incarnation of a physical copy.
                            instances.setdefault(str(c['instance']),dict(physical=c['id'],timestamp=c['timestamp']))
    need(endpoint is not None, len(prefix), 'missing opening endpoint')
    if verify_receipt:
        if selected_profile:
            import forge_casting_capture
            forge_casting_capture.validate_trace(path, selected_decks)
        else:
            capture.validate_trace(path)
    points = [p['state'] for p in checkpoints]
    # Forge redraws/bottoms each player immediately, whereas the native CR103.5
    # interface collects a round's private declarations first. Preserve source
    # chronology above; align ONLY actor-local opening observations by round.
    # No opponent hand or computed state is copied into either engine.
    aligned=sorted(range(len(choices)),key=lambda i:(2*choices[i]['round']-(choices[i]['kind']=='bottom'),
                                                    choices[i]['actor']!=starter))
    native_choices=[dict(choices[i],sequence=n) for n,i in enumerate(aligned)]
    native_points=[points[i] for i in aligned]+points[len(choices):]
    case = dict(id='forge-opening',starter=starter,decks=decks,chance=chance,choices=native_choices,stop='first_upkeep')
    pins = {p:xmage.sha(ROOT/p) for p in ('data/cards/foundations_micro_v1.json','data/rules/cr-2026-09-25.json','references/xmage/pins.json')}
    return dict(schema_version=1,complete_game=False,source_sha256=xmage.sha(path),starter=starter,
                bindings=bindings,source_instances=instances,prefix=prefix,paired=paired,
                endpoint=endpoint,remainder=dict(first_record=endpoint['index'],records=remainder_count,sha256=remainder_digest.hexdigest()),
                chance=chance,choices=choices,checkpoints=points,native_checkpoints=native_points,native_alignment=aligned,checkpoint_sources=[p['source'] for p in checkpoints],
                native=dict(schema_version=2,family='mulligan',pins=pins,cases=[case]))


def install(cache):
    import tarfile
    import forge
    import forge_opening_patch
    source=cache/forge.SOURCE
    hashes,_=capture.install(cache)
    with tarfile.open(cache/'forge.tar.gz') as archive:
        original=archive.extractfile(forge.SOURCE+'/'+capture.patch.CONTROLLER).read().decode()
    modified=forge_opening_patch.controller(original).encode()
    (source/capture.patch.CONTROLLER).write_bytes(modified)
    hashes[capture.patch.CONTROLLER]['modified']=capture.sha_bytes(modified)
    # Preserve the capture implementation in the repository. The replay cache
    # adds forwarding hooks and a privileged value observer to its private copy.
    recorder=(ROOT/'references/forge/MtgRawCapture.java').read_text()
    for signature,call in [
        ('public static void event(Game game, Event event) {','MtgOpeningReplay.event(game,event);'),
        ('public static void boundary(Game game, String boundary) {','MtgOpeningReplay.boundary(game,boundary);'),
        ('public static void shuffle(Player player, String stage) {','MtgOpeningReplay.shuffle(player,stage);')]:
        need(recorder.count(signature)==1,0,'recorder hook')
        recorder=recorder.replace(signature,signature+'\n        '+call)
    (source/capture.RECORDER).write_text(recorder)
    hashes[capture.RECORDER]['modified']=capture.sha_bytes(recorder.encode())
    for local,target in [('MtgOpeningReplay.java','forge-game/src/main/java/forge/game/MtgOpeningReplay.java'),
                         ('OpeningReplayTest.java','forge-gui-desktop/src/test/java/forge/gamesimulationtests/mtglab/OpeningReplayTest.java')]:
        (source/target).write_bytes((ROOT/'references/forge'/local).read_bytes())
        hashes[target]=dict(original=None,modified=xmage.sha(source/target))
    return hashes



NEGATIVES = ('missing-chance','reordered-chance','duplicate-occurrence','unconsumed-choice','checkpoint')


def corrupt(converted, fault):
    """Explicit consumer fault injection after immutable source validation."""
    import copy
    rows=converted['prefix']
    random=next(i for i,r in enumerate(rows) if r['kind']=='random')
    if fault=='missing-chance':
        rows.pop(random); return 'record kind random'
    if fault=='reordered-chance':
        rows[random],rows[random+1]=rows[random+1],rows[random]; return 'record kind random'
    if fault=='duplicate-occurrence':
        entry=next(r for r in rows if r.get('method')=='cheatShuffle')
        leave=rows[converted['paired'][entry['index']]]
        leave['result'][1]=copy.deepcopy(leave['result'][0]); return 'duplicate occurrence'
    if fault=='unconsumed-choice':
        converted['endpoint']['method']='mulliganKeepHand'; return 'unconsumed opening choice/endpoint'
    if fault=='checkpoint':
        rows[2]['state']['players'][0]['life']=19; return 'state/players/0/life'
    raise ValueError('unknown negative control')

def run(args):
    import forge
    import os
    import time
    import xml.etree.ElementTree as ET
    output=args.output.resolve(); output.mkdir(parents=True,exist_ok=False)
    receipt=dict(status='started',complete_game=False,source_sha256=xmage.sha(args.trace),
                 implementation={p:xmage.sha(ROOT/p) for p in ('scripts/forge_opening.py','scripts/forge_opening_patch.py',
                     'references/forge/MtgOpeningReplay.java','references/forge/OpeningReplayTest.java',
                     'crates/mtg-core/src/mulligan_reference_tests.rs')})
    receipt['native_sources']={str(p.relative_to(ROOT)):xmage.sha(p) for p in sorted((ROOT/'crates/mtg-core/src').glob('*.rs'))}
    receipt['native_sources']['Cargo.lock']=xmage.sha(ROOT/'Cargo.lock')
    started=time.monotonic()
    try:
        converted=extract(args.trace)
        expected=corrupt(converted,args.negative) if args.negative else None
        receipt['negative_control']=args.negative
        (output/'opening.json').write_text(json.dumps(converted)+'\n')
        (output/'native-input.json').write_text(json.dumps(converted['native'])+'\n')
        cache=args.cache.resolve()
        need(not cache.is_relative_to(ROOT),0,'cache must be external')
        receipt['toolchain']=capture.verify_toolchain(cache)
        receipt['patches']=install(cache)
        source_receipt=json.loads(args.trace.with_name('receipt.json').read_text())
        replay_manifest=ROOT/'data/cards/foundations_micro_v1.json'
        if source_receipt.get('source_profile') == 'frozen-deck-selection-v1':
            replay_manifest=output/'selected-manifest.json'
            replay_manifest.write_bytes(args.trace.with_name('selected-manifest.json').read_bytes())
        receipt['source_receipt_sha256']=xmage.sha(args.trace.with_name('receipt.json'))
        receipt['deck_selection']=source_receipt['decks']
        command=xmage.maven(cache)+['-o','-pl','forge-gui-desktop','-am','test',
            '-Dtest=forge.gamesimulationtests.mtglab.OpeningReplayTest','-Dsurefire.failIfNoSpecifiedTests=false',
            '-Dmtglab.assets='+str(cache/forge.SOURCE/'forge-gui')+'/',
            '-Dmtglab.manifest='+str(replay_manifest),
            '-Dmtglab.input='+str(output/'opening.json'),'-Dmtglab.output='+str(output/'forge.json')]
        if expected: command.append('-Dmtglab.expectedRejection='+expected)
        env=forge.environment(cache); env['MAVEN_OPTS']='-Xmx3g -Djava.awt.headless=true'
        report=cache/forge.SOURCE/'forge-gui-desktop/target/surefire-reports/TEST-forge.gamesimulationtests.mtglab.OpeningReplayTest.xml'
        report.unlink(missing_ok=True)
        xmage.bounded(command,cache/forge.SOURCE,env,output/'forge.log',300)
        (output/'test-report.xml').write_bytes(report.read_bytes())
        suite=ET.parse(report).getroot()
        need(suite.get('tests')=='1' and all(suite.get(k)=='0' for k in ('failures','errors','skipped')) and
             [c.get('name') for c in suite.findall('testcase')]==['openingReplay'],0,'actual unskipped Forge test')
        need(capture.source_diagnostic((output/'forge.log').read_text()) is None,0,'source diagnostic')
        need(xmage.dependencies(cache)==json.loads((ROOT/'references/forge/dependencies.json').read_text()),0,'dependency inventory')
        if expected:
            actual=json.loads((output/'forge.json').read_text())
            need(actual['status']=='negative-detected' and expected in actual['reason'],0,'live negative result')
            receipt['status']='negative-detected'
        else:
            fixture=dict(source_sha256=converted['source_sha256'],native=converted['native'],
                         expected=converted['native_checkpoints'],endpoint=converted['endpoint']['index'],
                         remainder=converted['remainder'],complete_game=False)
            (output/'native-fixture.json').write_text(json.dumps([fixture])+'\n')
            native_env=dict(os.environ,MTG_FORGE_OPENING_INPUT=str(output/'native-fixture.json'),
                            MTG_FORGE_OPENING_OUTPUT=str(output/'native.json'))
            xmage.bounded(['cargo','test','-p','mtg-core','--locked','--lib','captured_forge_opening_strict_replay'],
                          ROOT,native_env,output/'native.log',300)
            native=json.loads((output/'native.json').read_text())
            need(len(native)==1 and native[0]['source_sha256']==converted['source_sha256'] and
                 native[0]['run']['points']==converted['native_checkpoints'],0,'native opening result')
            receipt['status']='opening-agreed'
    except (ValueError,OSError,KeyError,TypeError) as e:
        receipt.update(status='failed',error=str(e))
    receipt['elapsed_seconds']=time.monotonic()-started
    receipt['artifacts']={p.name:xmage.sha(p) for p in output.iterdir() if p.is_file()}
    (output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps(dict(status=receipt['status'],error=receipt.get('error'),output=str(output))))
    return 0 if receipt['status'] in ('opening-agreed','negative-detected') else 2


if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cache',type=Path,required=True)
    parser.add_argument('--trace',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--negative',choices=NEGATIVES,help='Explicit live consumer negative control; never a successful replay')
    raise SystemExit(run(parser.parse_args()))
