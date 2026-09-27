# GH-78: scalar simulation acceptance

Scope: atomic unattended simulation portion of R0002-B020/B039/B041, not whole
shared RFC blocks or integration #21. No full game-catalog cases were assigned.
No game rules changed. Passive games use real reset/shuffle, opening choices,
turns, cleanup and terminal outcomes; they are not fake sibling implementations.
M1 remains incomplete. No new mature-engine agreement or performance claim.

Original independent expectations: RFC 0002 §9 requires explicit headless
configuration, bounded execution, machine output and termination accounting;
§5 separates completed games from failures/limits. CR 103.8a skips the starting
player's first draw; CR 504.1 draws one card; CR 704.5b loses on an attempted empty
draw. Forty minus seven is 33: with no mulligans, spells or damage, the
nonstarting seat's 34th draw is turn 68. Both libraries becoming empty does not
end the game. The starting seat wins with both life totals still 20. Budget and
signal expectations follow the experiment contract, not observed engine output.

Before implementation, five compiled subprocess tests failed against a CLI
placeholder: [behavioral red](red.txt). No compile/import error was counted as
red. The dependency cache initially required a narrow approved Cargo cache write;
that setup failure was not counted as a test result. The first implementation
compile exposed a reset return-type mismatch, corrected without assertions changing.

[Named green checks](green.txt) cover both starters, mixed/mirror decks, stable IDs,
seed extremes and deterministic output, explicit truncation, malformed/unsupported
configuration, no human fallback, output paths, real `/dev/full`, SIGTERM and wall
deadline. All subprocesses close stdin, unset display variables, pipe output, and
have an external deadline. Unit seams inject engine errors, writer failures and
precise interruption timing to assert failed/unfinished/not-started conservation.
These seams test accounting, not game semantics. All tests remain in normal Cargo
workspace discovery and debug/release torture; no existing test is weakened.

README impact: added actual runnable command, explicit passive limits, contract
and durable acceptance links; corrected obsolete no-simulation claims. Stage table
unchanged. Container quickstart's inner Cargo command is exercised separately.
The host Docker wrapper is documented but never invoked from the managed worker.

[Three compiled accounting mutations](mutations.json) were caught by unchanged
assertions: truncations counted as completions, unfinished counted as truncated,
and engine failure returning success. Each mutation was reverted.

[Full torture log](torture.txt) records normal discovery in debug and release
inside the managed Docker container after fetching/integrating current main.
Independent review, protected merge and exact-main CI receipts are retained in
the issue workpad and PR. No full-stage verification is inferred from these checks.
