"""Author literal test inputs from a fixed CR-derived six-turn schedule.

No engine output is read. This is fixture-authoring evidence, not a rules runner.
"""
import hashlib
import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = json.loads((ROOT / 'data/cards/foundations_micro_v1.json').read_text())


def author(starter, mulligan, mana_response=False):
    decks, chance = [], []
    for seat, name in enumerate(('red', 'green')):
        deck = next(d for d in MANIFEST['decks'] if d['id'] == name)
        ids = [f'{seat}/{c["card_id"]}/{i}' for c in deck['cards'] for i in range(c['copies'])]
        land, creature = ('mountain', 'swab-goblin') if seat == 0 else ('forest', 'bear-cub')
        top = [f'{seat}/{land}/{i}' for i in range(3)]
        top += [f'{seat}/{creature}/{i}' for i in range(2)]
        top += [f'{seat}/{land}/{i}' for i in range(3, 8)]
        order = top + [i for i in ids if i not in top]
        decks.append(dict(deck=name, occurrences=ids))
        chance.append(dict(sequence=seat, kind='initial_shuffle', actor=seat,
                           source=None, before=ids, after=order))
    choices = []
    def opening(actor, kind, round_, selection):
        choices.append(dict(sequence=len(choices), kind=kind, actor=actor,
                            source=None, round=round_, selection=selection))
    for s in (starter, 1-starter):
        opening(s, 'declare', 0, 'mulligan' if mulligan and s == starter else 'keep')
    if mulligan:
        event = dict(chance[starter], sequence=2, kind='mulligan_shuffle')
        chance.append(event)
        opening(starter, 'bottom', 1, [event['after'][6]])
        opening(starter, 'declare', 1, 'keep')
    play = []
    def action(turn, step, actor, kind, source=None, color=None):
        incarnation = None
        if source is not None:
            incarnation = (2 if kind == 'tap_mana' else 1) + 2 * int(mulligan and actor == starter)
        play.append(dict(sequence=len(play), turn=turn, step=step, actor=actor,
                         kind=kind, source=source, incarnation=incarnation, color=color))
    def passes(turn, step, active):
        action(turn, step, active, 'pass')
        action(turn, step, 1-active, 'pass')
    # Explicit fixed schedule: neither policies nor engine output choose actions.
    for turn in range(1, 7):
        active = starter if turn % 2 else 1-starter
        own_turn = (turn+1)//2
        land, creature, color = ('mountain', 'swab-goblin', 3) if active == 0 else ('forest', 'bear-cub', 4)
        passes(turn, 'upkeep', active)
        if turn != 1:  # CR103.8a skips the ENTIRE first draw step.
            passes(turn, 'draw', active)
        action(turn, 'precombat_main', active, 'play_land', f'{active}/{land}/{own_turn-1}')
        if own_turn >= 2:
            if own_turn == 2:
                for copy in (0, 1):
                    action(turn, 'precombat_main', active, 'tap_mana', f'{active}/{land}/{copy}')
            action(turn, 'precombat_main', active, 'cast', f'{active}/{creature}/{own_turn-2}')
            if own_turn == 3:
                for copy in (0, 1):
                    action(turn, 'precombat_main', active, 'tap_mana', f'{active}/{land}/{copy}')
            action(turn, 'precombat_main', active, 'pay', color=color)
            action(turn, 'precombat_main', active, 'pay', color=color)
            action(turn, 'precombat_main', active, 'finish_payment')
            if mana_response and turn == 3:
                other = 1-active
                other_land = 'mountain' if other == 0 else 'forest'
                action(turn, 'precombat_main', active, 'pass')
                action(turn, 'precombat_main', other, 'tap_mana', f'{other}/{other_land}/0')
                action(turn, 'precombat_main', other, 'pass')
                action(turn, 'precombat_main', active, 'pass')
            else:
                passes(turn, 'precombat_main', active)
        if turn == 6:
            break
        passes(turn, 'precombat_main', active)
        passes(turn, 'begin_combat', active)
        action(turn, 'declare_attackers', active, 'empty_attackers')
        passes(turn, 'declare_attackers', active)
        passes(turn, 'end_combat', active)
        passes(turn, 'postcombat_main', active)
        passes(turn, 'end_turn', active)
    return dict(id=f'red-green-{starter}-'+('mulligan' if mulligan else 'keep')+('-mana-response' if mana_response else ''),
                starter=starter, decks=decks, chance=chance, choices=choices,
                play=play, stop='second_creature_resolved')


def final_oracle(case):
    """Literal end-of-sixth-turn-main ledger; no engine or action interpreter."""
    starter = case['starter']
    mulligan = len(case['chance']) == 3
    hand, library, incarnations = [], [], {}
    for s in (0, 1):
        order = case['chance'][s]['after']
        did_mulligan = mulligan and s == starter
        # Initial seven, optionally return/re-draw/bottom seventh (CR103.5).
        for occurrence in order:
            incarnations[occurrence] = 0
        for occurrence in order[:7]:
            incarnations[occurrence] = 3 if did_mulligan else 1
        if did_mulligan:
            incarnations[order[6]] = 4
        # Three land plays and two vanilla creature casts/resolutions.
        for occurrence in order[:3]:
            incarnations[occurrence] += 1
        for occurrence in order[3:5]:
            incarnations[occurrence] += 2
        # Starter draws on turns 3/5; other seat draws on turns 2/4/6.
        count = 2 if s == starter else 3
        drawn = order[7:7+count]
        for occurrence in drawn:
            incarnations[occurrence] = 1
        hand.append(order[5:6 if did_mulligan else 7] + drawn)
        library.append(order[7+count:] + ([order[6]] if did_mulligan else []))
    battlefield = []
    for turn in range(1, 7):
        s = starter if turn % 2 else 1-starter
        n = (turn+1)//2
        land, creature = ('mountain','swab-goblin') if s == 0 else ('forest','bear-cub')
        battlefield.append(f'{s}/{land}/{n-1}')
        if n >= 2:
            battlefield.append(f'{s}/{creature}/{n-2}')
    permanents = {}
    for s in (0, 1):
        land, creature = ('mountain','swab-goblin') if s == 0 else ('forest','bear-cub')
        for n in range(3):
            permanents[f'{s}/{land}/{n}'] = dict(tapped=n < 2, sick=False, power=None, toughness=None)
        for n in range(2):
            permanents[f'{s}/{creature}/{n}'] = dict(tapped=False, sick=n == 1, power=2, toughness=2)
    return dict(boundary='second_creature_resolved', turn=6, step='precombat_main',
                active=1-starter, actor=1-starter, life=[20,20], mana=[[0]*6,[0]*6],
                land_plays=1, hand=hand, library=library, graveyard=[[],[]], exile=[],
                battlefield=battlefield, stack=[], incarnations=incarnations, permanents=permanents,
                payment=None)


def authored_checkpoints(case):
    """Expand only this fixed vanilla script into literal native assertions.

    This does not select actions, test legality, resolve arbitrary cards, or run
    games. Cost/stat constants are independently pinned above. The two clients
    must execute real engines; neither may call this fixture-authoring function.
    Native private payment staging is deliberately represented as staging.
    """
    state = dict(boundary='', turn=1, step='upkeep', active=case['starter'],
                 actor=case['starter'], life=[20,20], mana=[[0]*6,[0]*6],
                 land_plays=0, hand=[], library=[], graveyard=[[],[]], exile=[],
                 battlefield=[], stack=[], incarnations={}, permanents={}, payment=None)
    for s in (0, 1):
        order = case['chance'][s]['after']
        m = len(case['chance']) == 3 and s == case['starter']
        state['hand'].append(order[:6 if m else 7])
        state['library'].append(order[7:] + ([order[6]] if m else []))
        state['incarnations'].update({o:0 for o in order})
        state['incarnations'].update({o:3 if m else 1 for o in order[:7]})
        if m:
            state['incarnations'][order[6]] = 4
    points = []
    pass_count = 0
    for event in case['play']:
        actor = event['actor']
        turn, step = event['turn'], event['step']
        new_boundary = (turn, step) != (state['turn'], state['step'])
        if turn != state['turn']:
            state['turn'] = turn
            state['active'] = case['starter'] if turn % 2 else 1-case['starter']
            state['land_plays'] = 0
            for occurrence, permanent in state['permanents'].items():
                if occurrence.startswith(str(state['active'])+'/'):
                    permanent.update(tapped=False, sick=False)
        if new_boundary:
            state['mana'] = [[0]*6,[0]*6]
            pass_count = 0
            if step == 'draw':
                occurrence = state['library'][actor].pop(0)
                state['hand'][actor].append(occurrence)
                state['incarnations'][occurrence] += 1
        state.update(step=step, actor=actor, boundary=f'before/{event["sequence"]}')
        points.append(copy.deepcopy(state))
        kind, source = event['kind'], event['source']
        if kind == 'play_land':
            state['hand'][actor].remove(source)
            state['battlefield'].append(source)
            state['incarnations'][source] += 1
            state['permanents'][source] = dict(tapped=False, sick=False, power=None, toughness=None)
            state['land_plays'] = 1
            pass_count = 0
        elif kind == 'cast':
            color = 3 if actor == 0 else 4
            colored = [0]*6
            colored[color] = 1
            state['payment'] = dict(actor=actor, source=source,
                                    incarnation=state['incarnations'][source],
                                    action=event['sequence'], pool=state['mana'][actor].copy(),
                                    colored=colored, generic=1, sources=[])
            pass_count = 0
        elif kind == 'tap_mana':
            color = 3 if actor == 0 else 4
            if state['payment'] is not None:
                state['payment']['pool'][color] += 1
                state['payment']['sources'].append(source)
            else:
                state['mana'][actor][color] += 1
                state['permanents'][source]['tapped'] = True
            pass_count = 0
        elif kind == 'pay':
            p, color = state['payment'], event['color']
            p['pool'][color] -= 1
            if p['colored'][color]:
                p['colored'][color] -= 1
            else:
                p['generic'] -= 1
        elif kind == 'finish_payment':
            p = state['payment']
            assert p['colored'] == [0]*6 and p['generic'] == 0
            for occurrence in p['sources']:
                state['permanents'][occurrence]['tapped'] = True
            state['mana'][actor] = p['pool'].copy()
            state['hand'][actor].remove(p['source'])
            state['incarnations'][p['source']] += 1
            state['stack'].append(dict(source=p['source'], incarnation=state['incarnations'][p['source']], action=p['action']))
            state['payment'] = None
            pass_count = 0
        elif kind == 'pass':
            pass_count += 1
            if pass_count == 2 and state['stack']:
                spell = state['stack'].pop()
                occurrence = spell['source']
                state['incarnations'][occurrence] += 1
                state['battlefield'].append(occurrence)
                state['permanents'][occurrence] = dict(tapped=False,sick=True,power=2,toughness=2)
                state['actor'] = state['active']
                pass_count = 0
        elif kind != 'empty_attackers':
            raise AssertionError('unexpected fixture-authoring instruction')
    state['boundary'] = case['stop']
    assert state == final_oracle(case), 'script arithmetic must match independent final ledger'
    points.append(state)
    return points


def negative_fixtures(doc):
    out = {}
    def add(name, change):
        bad = copy.deepcopy(doc)
        bad['cases'] = [bad['cases'][0]]
        original = copy.deepcopy(bad['cases'][0]['play'])
        change(bad, bad['cases'][0])
        # Structural sequence checks must not hide runtime chronology defects.
        for n, event in enumerate(bad['cases'][0]['play']):
            event['sequence'] = n
        changed = bad['cases'][0]['play']
        first = next((i for i, e in enumerate(changed)
                      if i == len(original) or e != original[i]), None)
        if first is not None and name != 'extra_pass':
            # Retain only the prefix needed to witness the offending choice
            # and, for omissions, the immediately following callback.
            del changed[first+2:]
        out[name] = bad
    add('missing_pass', lambda d,c:c['play'].pop(0))
    add('extra_pass', lambda d,c:c['play'].append(copy.deepcopy(c['play'][-1])))
    def swap(c):
        c['play'][0], c['play'][1] = c['play'][1], c['play'][0]
    add('reordered_pass', lambda d,c:swap(c))
    add('wrong_actor', lambda d,c:c['play'][0].__setitem__('actor',1))
    add('unsupported_callback', lambda d,c:c['play'][0].__setitem__('kind','choose_trigger'))
    add('wrong_color', lambda d,c:next(e for e in c['play'] if e['kind']=='pay').__setitem__('color',4))
    add('insufficient_payment', lambda d,c:c['play'].remove(next(e for e in c['play'] if e['kind']=='pay')))
    add('stale_source', lambda d,c:next(e for e in c['play'] if e['kind']=='cast').__setitem__('source','0/swab-goblin/99'))
    add('stale_incarnation', lambda d,c:next(e for e in c['play'] if e['kind']=='cast').__setitem__('incarnation',99))
    add('reuse_old_cast_source', lambda d,c:next(e for e in c['play'] if e['kind']=='cast' and e['source']=='0/swab-goblin/1').__setitem__('source','0/swab-goblin/0'))
    def replace_pass(c, turn, actor, kind, source):
        event = next(e for e in c['play'] if e['turn']==turn and e['step']=='precombat_main' and e['kind']=='pass' and e['actor']==actor)
        event.update(kind=kind, source=source, incarnation=1)
    add('second_land', lambda d,c:replace_pass(c,1,0,'play_land','0/mountain/1'))
    add('insufficient_mana', lambda d,c:replace_pass(c,1,0,'cast','0/swab-goblin/0'))
    def off_turn(c):
        # Green has an untapped Forest from turn 2. Elves is affordable, so
        # this isolates sorcery timing rather than a second insufficient cost.
        order = c['chance'][1]['after']
        index = order.index('1/llanowar-elves/0')
        order[5], order[index] = order[index], order[5]
        replace_pass(c,3,1,'cast','1/llanowar-elves/0')
    add('off_turn_creature', lambda d,c:off_turn(c))
    add('unscripted_mana', lambda d,c:replace_pass(c,1,0,'tap_mana','0/mountain/1'))
    add('missing_empty_declaration', lambda d,c:c['play'].remove(next(e for e in c['play'] if e['kind']=='empty_attackers')))
    add('wrong_turn', lambda d,c:c['play'][0].__setitem__('turn',2))
    add('wrong_step', lambda d,c:c['play'][0].__setitem__('step','draw'))
    add('unsupported_stop', lambda d,c:c.__setitem__('stop','complete_game'))
    add('schema', lambda d,c:d.__setitem__('schema_version',2))
    add('extra_field', lambda d,c:c['play'][0].__setitem__('automatic',True))
    return out


def xmage_checkpoints(native_points):
    """CR601.2a puts the announced card on stack before costs are paid.

    Native's private transaction stages those same changes until commit. Both
    raw representations are asserted separately; no runtime state is rewritten.
    """
    points = copy.deepcopy(native_points)
    # Pinned Mulligan.drawHand applies MulliganDefaultHandSorter: lands first,
    # then vanilla creatures, preserving equal-card order. Later draws append.
    opening = [sorted(hand, key=lambda o: o.split('/')[1] not in ('mountain', 'forest'))
               for hand in native_points[0]['hand']]
    for point in points:
        for seat in (0, 1):
            hand = point['hand'][seat]
            point['hand'][seat] = ([o for o in opening[seat] if o in hand] +
                                   [o for o in hand if o not in opening[seat]])
        payment = point['payment']
        if payment is None:
            continue
        source, actor = payment['source'], payment['actor']
        point['hand'][actor].remove(source)
        point['incarnations'][source] += 1
        point['stack'].append(dict(source=source, incarnation=point['incarnations'][source], action=payment['action']))
        point['mana'][actor] = payment['pool'].copy()
        for occurrence in payment['sources']:
            point['permanents'][occurrence]['tapped'] = True
    return points


if __name__ == '__main__':
    pins = {p: hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in
            ('data/cards/foundations_micro_v1.json', 'data/rules/cr-2026-09-25.json', 'references/xmage/pins.json')}
    doc = dict(schema_version=3, family='priority', pins=pins,
               cases=[author(s, m) for s in (0, 1) for m in (False, True)] +
                     [author(s, False, True) for s in (0, 1)])
    (ROOT/'fixtures/reference/full-pool-priority.json').write_text(json.dumps(doc, indent=2)+'\n')
    expected = {c['id']: final_oracle(c) for c in doc['cases']}
    (ROOT/'fixtures/reference/full-pool-priority-final.json').write_text(json.dumps(expected, indent=2)+'\n')
    (ROOT/'fixtures/reference/full-pool-priority-negative-inputs.json').write_text(json.dumps(negative_fixtures(doc), indent=2)+'\n')
    points = {c['id']:authored_checkpoints(c) for c in doc['cases']}
    (ROOT/'fixtures/reference/full-pool-priority-native.json').write_text(json.dumps(points, separators=(',',':'))+'\n')
    reference = {name:xmage_checkpoints(rows) for name,rows in points.items()}
    (ROOT/'fixtures/reference/full-pool-priority-xmage.json').write_text(json.dumps(reference, separators=(',',':'))+'\n')
