"""Independent integer oracle: doc/rng.md plus opening shuffle specification.
No engine imports. Frozen deck source order is top-first before shuffling.
"""
import json
from pathlib import Path

MOD = 2**64
STEP = 0x9E3779B97F4A7C15

def mix(x):
    x = ((x ^ (x // 2**30)) * 0xBF58476D1CE4E5B9) % MOD
    x = ((x ^ (x // 2**27)) * 0x94D049BB133111EB) % MOD
    return x ^ (x // 2**31)

def vectors():
    manifest = json.loads((Path(__file__).resolve().parents[3] / 'data/cards/foundations_micro_v1.json').read_text())
    result = []
    for master, episode in [(0, 0), (42, 9), (MOD-1, MOD-1)]:
        state = mix(mix(master) ^ mix((episode+STEP)%MOD) ^ mix(2*STEP%MOD))
        decks = []
        for deck in manifest['decks']:
            cards = [entry['card_id'] for entry in deck['cards'] for _ in range(entry['copies'])]
            for size in range(40, 1, -1):
                # Largest multiple of size no larger than 2^64; accept [0,limit).
                limit = (MOD // size) * size
                while True:
                    state = (state+STEP)%MOD
                    word = mix(state)
                    if word < limit:
                        break
                index = word % size
                cards[size-1], cards[index] = cards[index], cards[size-1]
            decks.append(cards)
        result.append(dict(master=master, episode=episode, decks=decks))
    return result

if __name__ == '__main__':
    print(json.dumps(vectors(), indent=2))
