# GH-62 RNG acceptance

Scope: R0002-B015 RNG clause and SYS-CORE-002 only. No complete RFC block,
game-catalog case, playable engine or M1 completion is claimed.

Before implementing the Rust arithmetic, `vectors.py` calculated the fixed
`vectors.json` using the equations in [the versioned specification](../../rng.md).
Rust tests retain independent literal expected outputs (3 primitive seeds × 4
words; 6 master/episode tuples × 3 domains × 4 words). Re-running the oracle
reproduces the JSON byte-for-byte.

The initial compiling stub returned zero words and accepted every version.
[Primitive red](red-primitive.txt) and [API red](red-integration.txt) show actual
assertion failures, not import/compile errors. The metamorphic order/isolation
assertion passed on that stub: it is deliberately complemented by fixed known
answers, which prevent constant or wrong streams from passing acceptance.
[Green](green.txt) records all four RNG tests passing after implementation.

[Full torture](torture.txt) passed inside the managed Linux Docker toolchain after
fetching/integrating main `49c5656af9f150591c09c54a68f6d048a69e83aa`: documentation,
program/design validation, 120 Python tests, fmt/clippy, and all 12 Rust tests in
each debug/release profile. A prior full run found a Clippy range-loop warning in
the new test; iteration was refactored without changing coverage or expectations.
No tests removed, skipped, weakened or replaced. No reference game run applies to
a PRNG primitive; future shuffle/game integration remains separately assigned.

Commands: `cargo test -p mtg-core rng`, `./scripts/torture.sh`,
`python3 doc/evidence/rng-v1/vectors.py` (compare stdout to `vectors.json`).
README impact: new delivered primitive/API and limitations documented; stage table
unchanged. Review and exact candidate/merge CI evidence are linked in
[the workpad](https://github.com/pabloxrl/mtg-lab/issues/62#issuecomment-5855096475).
