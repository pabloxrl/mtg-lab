"""Independent arithmetic oracle; never imports or calls the Rust engine.

Uses the specified RNG/shuffle math and frozen manifest input. CR 103.5
supplies round order/cardinality. Hand remainder + old library is the specified
pre-shuffle sequence; selected bottoms append in the declared order.
"""
import json
from pathlib import Path
import runpy

ROOT = Path(__file__).resolve().parents[3]
arithmetic = runpy.run_path(str(ROOT / 'doc/evidence/opening/vectors.py'))
mix, MOD, STEP = (arithmetic[key] for key in ('mix', 'MOD', 'STEP'))


def vectors():
    manifest = json.loads((ROOT / 'data/cards/foundations_micro_v1.json').read_text())
    output = []
    for starter in [0, 1]:
        master, episode = 42, 9
        state = mix(mix(master) ^ mix((episode + STEP) % MOD) ^ mix(2 * STEP % MOD))

        def shuffle(cards):
            nonlocal state
            result = list(cards)
            for size in range(40, 1, -1):
                while True:
                    state = (state + STEP) % MOD
                    word = mix(state)
                    if word < (MOD // size) * size:
                        break
                index = word % size
                result[size - 1], result[index] = result[index], result[size - 1]
            return result

        decks = [shuffle([c['card_id'] for c in d['cards'] for _ in range(c['copies'])])
                 for d in manifest['decks']]
        hands, libraries = [], []
        for deck in decks:
            hands.append(deck[:7])
            libraries.append(deck[7:])
        # Both mulligan once, starting player first in replacement RNG order.
        for seat in [starter, 1 - starter]:
            deck = shuffle(hands[seat] + libraries[seat])
            hands[seat], libraries[seat] = deck[:6], deck[7:] + [deck[6]]
        first = dict(hands=[list(h) for h in hands], libraries=[list(l) for l in libraries])
        # Starter mulligans again, opponent keeps six. Bottom positions 5 then 1.
        deck = shuffle(hands[starter] + libraries[starter])
        hands[starter] = [c for i, c in enumerate(deck[:7]) if i not in [5, 1]]
        libraries[starter] = deck[7:] + [deck[5], deck[1]]
        output.append(dict(master=master, episode=episode, starter=starter,
                           first=first, second=dict(hands=hands, libraries=libraries)))
    return output


if __name__ == '__main__':
    print(json.dumps(vectors(), indent=2))
