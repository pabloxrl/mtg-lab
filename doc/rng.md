# Episode RNG v1

This implements the RNG clause of R0002-B015 / SYS-CORE-002. The exact version
identifier is `splitmix64-v1`; it versions both the generator and derivation.
Unknown identifiers fail explicitly (no aliases, default or normalization).
Record this identifier alongside the master seed and stable episode ID in future
restricted replay/run metadata. These seeds must not become policy observations.

All inputs and state are unsigned 64-bit integers. Arithmetic below wraps modulo
2^64; `>>` is logical right shift and `xor` is bitwise exclusive-or. No native
byte order, platform hash, wall clock, global counter or worker ID participates.

Define `G = 0x9e3779b97f4a7c15` and the SplitMix64 finalizer:

```text
mix(x):
    x = (x xor (x >> 30)) * 0xbf58476d1ce4e5b9
    x = (x xor (x >> 27)) * 0x94d049bb133111eb
    return x xor (x >> 31)
```

A generator initialized with state `s` emits a word by first setting `s = s + G`,
then returning `mix(s)`. Zero is a valid state. This is the fixed-increment
SplitMix64 algorithm described by Sebastiano Vigna
(<https://prng.di.unimi.it/splitmix64.c>), using the Stafford Mix13 finalizer.
The equations above are the complete normative specification; implementation is
original Rust, with no copied upstream source. This is a noncryptographic PRNG,
not suitable for secrets or adversarial hidden-information security.

For master seed `m`, stable episode ID `e` and stream domain `d`, initial state is:

```text
mix(mix(m) xor mix(e + G) xor mix(d + 2*G))
```

Domains are closed and versioned: environment = 0, policy seat 0 = 1, policy
seat 1 = 2. Scheduling must own a separate RNG and never borrow the environment
instance. The API creates an owned stream without consuming any other stream.
Repeated inputs restart exactly the same sequence; callers assign stable unique
numeric episode IDs within a run, never creation-order slot numbers. Future
string/UUID identifiers require a separately specified mapping, not a Rust hash.

For fixed master/domain, distinct episode IDs map bijectively to initial states.
This does not promise disjoint sequences, cryptographic independence, different
shuffled decks for every pair, or collision freedom across all 192-bit tuples.
Policy draw *consumption* cannot advance an independently owned environment RNG;
policy actions can still change which chance events occur in a future game.

`mtg_core::rng::EpisodeRng::new(version, master, episode, Stream)` returns a
validated stream, `version()` identifies it, and `next_u64()` advances it.
The [opening reset](opening.md) now supplies versioned bounded sampling and
shuffling using the environment stream. Snapshots/serialization, policies and
worker scheduling remain subsequent deliveries. This primitive alone implies no game behavior.

## Independent acceptance evidence

[Python integer-arithmetic oracle](evidence/rng-v1/vectors.py) calculates
[fixed vectors](evidence/rng-v1/vectors.json) from these equations independently
of Rust. It uses unbounded arithmetic, explicit modulus and division instead of
machine overflow/right shifts. Primitive zero-state outputs begin
`e220a8397b1dcdaf`, `6e789e6aa1b965f4`, `06c45d188009454f`, `f88bb8a8724c81ec`.
Vectors include zero, one, high-bit and maximum inputs, all three domains and
state wraparound. Tests retain literals and never ask production to generate
expectations. Run `cargo test -p mtg-core rng` and `./scripts/torture.sh` inside
the managed toolchain container. Full red/green and review evidence is recorded
in [GH-62's workpad](https://github.com/pabloxrl/mtg-lab/issues/62#issuecomment-5855096475).
