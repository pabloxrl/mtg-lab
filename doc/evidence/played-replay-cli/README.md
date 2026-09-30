# Played replay CLI acceptance (#180)

Partial R0002-B020/B039/B041 delivery only. #120 retains every composed CLI
clause, #21 integration/catalog acceptance, and #22 the M1 gate. No rules,
recorder, replay format, policy algorithm or enforcement changed.

`cargo test --locked -p mtg-cli --test played_replay_commands` is in ordinary
workspace discovery. Four subprocess contracts run with stdin closed, displays
unset, JSON stdout/stderr and a 15-second external deadline. The retained
[compiled behavioral red](behavioral-red.txt) precedes implementation and shows
valid played input rejected as malformed and required redacted categories absent.
[Focused green](behavioral-green.txt) records all four passing afterward.
An initial test-authoring error used numeric seats in the semantic wire format;
it was corrected to the existing P0/P1 enum encoding before the retained red.
It is not implementation-failure evidence or a changed game expectation.

The minimized seed 180/episode 0 and complete hand-authored six-record script live
in `crates/mtg-cli/tests/played_replay_commands.rs`. Both full frozen green decks
use exact land-first orders and normal reset. CR 103 permits both keeps; CR 117
upkeep passes and CR 103.8a skipped first draw reach main; CR 305 permits P0's
land play. P0 has six cards afterward; life remains [20,20]. P1's concession
(CR 104.3a) produces winner P0 and losses [null,Concession]. These literal
checkpoints and the complete public CLI summary are independent expectations,
not a golden file generated from the verifier. The production library recorder
creates the actual artifact. Retained opening artifacts use production record
with two keeps; their existing summary and seat-inspection tests remain intact.
No future simulate/capture CLI dependency or synthetic game state is used.

Declared synthetic corruption controls supplement the valid recording:
unsupported format/version; missing/duplicate discriminators; a swapped format
with the wrong body (no fallback); malformed/truncated bytes; over-16-MiB input;
missing/nonregular file; missing concession (nonterminal); extra post-terminal
choice; stale land incarnation; life changes despite no damage/cost; added private
checkpoint field; corrupted opponent opening hand. Each asserts its independent
rejection category and structured exit 2. Success exits 0; no-replace output
routing is exercised. Played inspection and an invented privilege flag fail.
Private payload/hand/field/path sentinels cannot appear in ordinary errors.

Verification reuses existing core APIs, accepts at most 16 MiB, and drains core
work per semantic record. Synchronous filesystem stalls still need external job
deadlines. No replay-ID authorization, persisted grant, capture, broad inspection,
cryptographic provenance or performance qualification is claimed. README and
command/replay documentation distinguish supported behavior from pending M1.

The first complete managed-container `./scripts/torture.sh` passed: 143 Python
tests, debug/release Rust tests and doctests, docs/program/catalog, formatting and
Clippy. [Receipt and source/log hashes](verification.json) tie that execution to
the implementation. All 97 program tasks and pinned requirements validate; no
catalog, RFC ownership or existing test changed. No new rules/reference agreement
is claimed. Both README quickstarts passed: passive benchmark completed two
games; the handwritten JSONL fixture validated one episode and one decision.
The new replay examples are exercised with actual temporary library-produced
files by the subprocess tests, including exact summaries and output routing.

Fresh main integration, final full-suite rerun, clean-candidate separate Codex
review (including README), protected PR and exact-main CI receipts are retained
in the [single issue workpad](https://github.com/pabloxrl/mtg-lab/issues/180).
Completion remains conditional on that delivery evidence; component acceptance
does not complete #120, #21 or M1.
