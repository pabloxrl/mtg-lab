# Initial terminal-game admission evidence

Executable acceptance **PASS** for the repaired initial native/XMage GR-040 subset.
[Validation receipt](validation.json) records the commands, counts and hashes.
The [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/277#issuecomment-6112473318)
records exact-candidate independent review, protected delivery and exact-main CI
status. Delivery is complete only after those gates pass. This report does not
declare M2 or the scenario audit complete.

The [version-10 contract](../../full-pool-reference.md#version-10-initial-terminal-game-admission)
and [frozen manifest](../../../fixtures/reference/full-pool-admission-manifest.json)
define sixteen distinct normal-reset, rules-terminal games: RG/GR/RR/GG, each
starting seat, one actual deterministic heuristic origin and one independently
authored path per row. Replays invoke no policy. All twenty nontoken card types
have consumed plays and observed resolved states. Both Surprise modes and the
payment, trigger, combat and cleanup interactions are retained alongside the
earlier detailed reference families.

| Check | Result |
| --- | --- |
| [Focused real-adapter discovery](focused.log) | 27 tests passed. |
| [Normal Python discovery](python-discovery.log) | 397 tests passed, including all 27 new regressions. |
| [Cargo workspace discovery (torture debug phase)](cargo-workspace.log) | 762 executions, zero failed/ignored; three new named admission regressions and the reused policy test discovered. |
| [Full torture after current-main integration](torture.log) | 397 Python tests and 1,524 Rust debug/release executions; zero failed/ignored; docs, program, catalog, formatting and Clippy passed. |
| [Reference run 1](run-1.json), [run 2](run-2.json) | Each: sixteen games twice per actual engine, 18 invalid tapes per engine, 42 native/48 XMage corruption controls, all eight retained families passed. |

Each sixteen-game replay set checks 14,863 action and 75 pre-SBA damage
checkpoints. Both complete reference invocations have identical hashes for all
168 source files. Their [first](final-run-1.json) and [second](final-run-2.json)
archive manifests bind every retained input, actual observation, consumed choice,
chance event, toolchain/dependency record, source snapshot and compatibility
receipt. The archives are [run 1](final-run-1.tar.gz) and
[run 2](final-run-2.tar.gz); their contents are privileged, with directory mode
0700 and file mode 0600. Extract into a private directory, never a policy-visible
artifact location.

The [oracle author](../../../fixtures/reference/author_admission.py) reads pinned
card definitions and inputs, never captured output. Inputs and expectations are
frozen before comparison. For directed paths, CR103.8a/121.4/104.3c fix the
nonstarter's turn-68 empty-draw loss; pinned Pyromancer text fixes life totals.
Explicit hand, library, three-spell, four-land and cleanup-turn ledgers are
authored independently. Heuristic paths have literal pre-first-cast checkpoints
derived from shuffled inputs and CR103.8a/121.2/305.2/400.7, plus CR704.5a
terminal predicates. Cross-engine intermediate agreement supplements these
expectations and the unchanged detailed earlier families. No captured outcome
or state becomes a golden checkpoint.

Behavioral tests were written first. [Initial red](behavioral-red.log) ran the
delivered real cleanup client and failed the sixteen-game floor (actual two).
[Nested observation reds](nested-fields-red.log) exposed missing payment and
forged effect/departure checks. [Runtime field red](runtime-field-red.log)
exposed a malformed action consumed before its missing field was rejected.
Further [identity](identity-fields-red.log) and
[opening/damage](intermediate-fields-red.log) reds required raw provenance and
actual pre-SBA observations. The [auxiliary-field red](auxiliary-fields-red.log)
reproduced omitted native cleanup birth and XMage pre-departure UUID fields
being accepted. A shared structural validator now checks every stream before
projection and binds auxiliary raw histories to the validated primary stream.
These regressions remain in ordinary discovery and the real reference controls.

[Retained-attempt notes](attempts.md) and the [complete archive index](retained-attempts.json)
distinguish generation variants, failed diagnostics and superseded candidates
from final acceptance. Standard recorder origins lacked pre-shuffle duplicate
bindings and remain unadmitted; admitted origins use actual creation hooks.
Their eight manifest provenance hashes match the occurrence-aware generation
records in the [generation archive](generation-attempts.tar.gz). Candidate-v1,
v2, v3 and review-rejected v4 results, first differences, source snapshots and the deliberately
interrupted torture pass remain retained. Repeated games never increase the
sixteen-game quota.

The conservative native compatibility fingerprint includes `lib.rs` test
declarations. Adding the actual-policy test import invalidated three profiling
fixture headers. [Compatibility metadata](profile-compatibility.json) records
the old/new fingerprints and unchanged nonheader hashes. Only those engine
headers changed; no authored checkpoint, upstream/card/rules pin or profiling
assertion changed. [The initial profile failure](profile-compatibility-red.log)
is retained. Full native measurement and privacy/capture equality checks passed
again in torture.

Limits: all sixteen initial tapes explicitly keep seven; the retained London
family separately verifies mulligans and bottoming. XMage evidence retains the
separately hashed `patched-combat-sba-v1` integration and its original upstream
mismatch evidence. Engine-specific cost and trigger staging are explicitly
verified before committed-state comparison. Selected-action acceptance, native
raw invalid/stale rejection and complete legal-set observability are distinct;
complete cross-engine legal-set comparison is not claimed. No Forge, reference-AI
generation, campaign/corpus, >=100-scenario audit or M2 verdict is supplied.

README impact: the root README adds the bounded admission command/evidence row;
its M2 status remains incomplete. Both documented admission commands were run.
Independent review must check README accuracy, retained tests and ownership.

[Independent review 1](review-1.json) and its [resolution](review-resolution.md)
record the required effect-presence/lifetime and
trigger-source/counter binding repairs. [Review regression reds](review-regressions-red.log)
retain three intended assertion failures. Five added ordinary-discovery tests
cover omitted/expired effects and missing/rebound trigger provenance. Expected
effect lifetimes come from the pinned card/effect classes and CR514.2; the
historical XMage payload is only a corruption witness, never a golden oracle.
