"""Independent arbitrary-precision arithmetic oracle for the RNG v1 specification.

Run before implementing Rust; no production code or dependencies are imported.
XOR/shift expressions are written as division so this also checks Rust shift semantics.
"""
import json

MODULUS = 2**64
STEP = 0x9E3779B97F4A7C15


def mix(value):
    value = ((value ^ (value // 2**30)) * 0xBF58476D1CE4E5B9) % MODULUS
    value = ((value ^ (value // 2**27)) * 0x94D049BB133111EB) % MODULUS
    return value ^ (value // 2**31)


def words(seed):
    return [f'{mix((seed + n * STEP) % MODULUS):016x}' for n in range(1, 5)]


def vectors():
    primitive = [{'seed': f'{s:016x}', 'words': words(s)} for s in (0, 1, MODULUS - 1)]
    episodes = []
    for master, episode in ((0, 0), (0, 1), (1, 0), (42, 7), (MODULUS - 1, MODULUS - 1), (2**63, 2**63)):
        for domain in (0, 1, 2):
            seed = mix(mix(master) ^ mix((episode + STEP) % MODULUS) ^ mix((domain + 2 * STEP) % MODULUS))
            episodes.append(dict(master=f'{master:016x}', episode=f'{episode:016x}', domain=domain, seed=f'{seed:016x}', words=words(seed)))
    return dict(primitive=primitive, episodes=episodes)


if __name__ == '__main__':
    print(json.dumps(vectors(), indent=2))
