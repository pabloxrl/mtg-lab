# GH-64 opening reset acceptance

This evidence covers only R0002-B015/B016 reset prefix, SYS-CORE-001 prefix and
SYS-CORE-009 core configuration. M1 and integration #17 remain incomplete.
The exact assigned catalog case `rules-setup-two-player-opening-negative` is
executable in `opening_invalid_config_preserves_complete_state_and_rng`:
third-seat and sideboard/game-two initialization must reject without an
approximate game. Both empty and already initialized games are tested.

Independent expectations: RFC 0002 §3 fixes two players, 20 life, seven cards,
both starters and red/green mirrors; CR 103.3/103.5 establish life/opening draw
and starting-player mulligan choice order. Frozen manifest card/copy counts
supply input permutations, not production output. Explicit top-first orders
are split at index seven by test assertions. Full mulligan actions and first
draw are deliberately not claimed; those cases stay with #65/#67.

[red.txt](red.txt) retains four compiled behavioral failures against the initial
no-op reset stub (life remained zero, third seat was accepted, no opening cards,
shuffle output empty). No build/import failure is counted as red. These same
assertions remain in normal Rust test discovery.
[green.txt](green.txt) records the final named acceptance run.

[vectors.py](vectors.py) independently computes the specified shuffle using
Python arbitrary-precision arithmetic and manifest inputs, without importing or
executing the engine. [vectors.json](vectors.json) was fixed before the Rust
shuffle implementation. Three seed/episode pairs cover zero, ordinary and
maximum unsigned inputs, with exact full orders for both decks. A separate
injected-word unit test tests the rejection boundary that ordinary seeds would
almost never hit. Expectations were not copied from Rust output.

Seven tests cover exact orders, both starters/all four matchups, strict invalid
configs (complete Debug state includes private RNG/counters), 32 intervening
resets and same-seed restarts, stale/foreign handles, unchanged second game,
retained storage capacities, sampler boundaries, decision exhaustion and explicit
orders with no RNG consumption. Every regression runs in the full suite.

[Full torture result](torture.txt) includes docs/program/catalog checks, Python
regressions and all Rust debug/release tests, fmt and Clippy in the managed Linux
Docker worker. No Docker invocation or host software installation is needed
inside the worker. README updated for usable API and limits, stage verdict unchanged.
Separate review, protected merge and exact-main CI receipts are preserved in
[the issue workpad](https://github.com/pabloxrl/mtg-lab/issues/64#issuecomment-5855275482)
and its linked PR. This report does not substitute for those delivery gates.
