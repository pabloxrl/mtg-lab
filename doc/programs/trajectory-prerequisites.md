# Lossless played-policy trajectory prerequisites

## Superseding collector registration

Operations [#158](https://github.com/pabloxrl/mtg-lab/issues/158) supersedes ONLY
the oversized C boundary and its delivery sequence below with the
[six collector contracts](collector-prerequisites.md). A (#152) and B (#153)
are delivered; C (#154) is now the final collector integration audit after
#164, followed by the unchanged #117 aggregate audit. The original A/B/C plan
below remains historical evidence, including the unsuccessful obligation to
implement all collector contracts in C. It no longer assigns that combined
implementation to a prerequisite. The new clause crosswalk preserves all its
acceptance; no child must implement a future sibling to pass its component tests.

Operations [#151](https://github.com/pabloxrl/mtg-lab/issues/151) registers exactly
three M1 prerequisites for the preserved unsuccessful #117 audit. This is a plan,
not delivered trajectory capability or an M1 completion verdict.

## Inspected contract and dependency evidence

Inspected main `af6da9b201777ebe00d5c342417984f3b384730d`:

- `crates/mtg-core/src/trajectory.rs`: schema v1 `Frame::capture` uses
  `Game::observe` and `PlayerView`; `Choice.selected` names one flat candidate.
  `views.rs` intentionally rejects private cast/target/payment continuations.
- `crates/mtg-core/src/policy.rs`: `Observation` owns pending spell/targets/payment,
  committed stack targets and combat relationships; `Decision` carries actual
  flat candidates/masks and optional factored combat domains. `Submission.choices`
  carries ordered multiple choices and parameterized combat declarations.
- `crates/mtg-recorder/src/schema.rs`, `lib.rs` and `manifest.rs`: strict durable
  fields and version validation mirror the legacy record. Bypassing unavailable
  views or fabricating a selected row would lose actual policy inputs/actions.

These are missing integration contracts, not grounds to weaken the completed
v1 components. Existing #76/#77/#111/#112/#114/#116 completion workpads retain
acceptance, review and merge evidence; their exact-main CI was independently
rechecked during registration. #106 is the registration operation's prerequisite.
No correction to the operator's proposed A/B/C edges is necessary.

| Task | Deliverable | Direct prerequisites |
| --- | --- | --- |
| [#151](https://github.com/pabloxrl/mtg-lab/issues/151) | Register this plan and perform gated recovery; operations, no RFC ownership | #106 |
| A [#152](https://github.com/pabloxrl/mtg-lab/issues/152) | Lossless canonical in-memory contract and per-seat readers | #151, #76, #111, #112 |
| B [#153](https://github.com/pabloxrl/mtg-lab/issues/153) | Existing strict JSONL and run manifest support for A | #152, #77, #116 |
| C [#154](https://github.com/pabloxrl/mtg-lab/issues/154) | Owned scalar collector and authorized replay linkage | #153, #114 |
| [#117](https://github.com/pabloxrl/mtg-lab/issues/117) | Aggregate collector audit, every original acceptance clause retained | #154 |

#117's original #114/#116/#76 edges are all preserved transitively through C,
B and A. None has a separate independent reason to remain direct. #20 retains
full trajectory integration; #120 CLI integration; #21 full CLI/baseline audit;
#22 the M1 gate. The validated graph makes #22 depend on every M1 task, including
this operation and all three children. All later gates and full-pool/reference
requirements remain unchanged. Native links group #151 under program #7 and
A/B/C under #117; blocked-by links mirror the table. #117's native replacement
is deferred until protected registration merge and successful exact-main CI.

## Independently testable boundaries

**A:** Extend canonical owned records/readers with exact authorized structured
observations, actual flat/factored domains and complete semantic submissions.
Define explicit version/compatibility and logical-action/microchoice/cancellation
semantics. Preserve v1 behavior and the unavailable legacy view regression.
Positive and negative tests cover real pending cast/target/payment, multiple
bottom/discard choices, nontrivial attackers/blockers/damage allocations,
actor-only pending privacy, same-seat next/final observations, owned-buffer/reset
lifetime, and once-only terminal rewards including nonacting/zero-decision seats.
Use literal requirements/rules-derived expectations and independently enumerated
small domains. Reject invalid/stale/discontinuous records and incompatible
versions visibly. No game-driving collector, durable sink or CLI.

**B:** Extend the existing JSONL writer/strict reader and run manifest for A,
with an explicit new schema version and v1 fixtures/tests preserved. Hand-authored
complete canonical records round-trip exactly. Missing/corrupt action-time fields,
unknown/invalid versions, incomplete fragments, backpressure and writer errors
fail visibly. Retain privacy, immutable ownership, checksums/counts and publication
integrity; absent statistics stay absent. No parallel recorder, CLI or game loop.

**C:** Own reset/advance/submission lifecycle; record every accepted decision and
logical timing, supplied-only policy statistics, budget/truncation/failure
accounting, actual configuration/provenance/checksums, separately authorized
opaque replay linkage and sink-failure propagation. Run ALL original #117
acceptance: normal-reset played games against an independent decision ledger,
JSONL and authorized replay reload, same-seed recording on/off state/RNG equality,
both/nonacting/zero-decision seats, missing decisions, pending choices/privacy,
reset/buffer lifetime, truncation and writer failures. No CLI or synthetic
substitute siblings. #117 repeats aggregate acceptance after C; child success
alone is never aggregate completion.

All children add only partial R0002-B036/B037 ownership. Every old owner, RFC
block, source pin and catalog setup/action/expectation/owner remains intact.
Each issue requires test-first compiled behavioral red, independently justified
positive/negative expectations, normal-discovery regressions with minimized
inputs/seeds, full torture after main integration, separate Codex review,
protected merge, successful exact-main CI, README assessment and normal bounded
handoff. No policy/enforcement changes or unrelated #131–#133 refactors.
If inspection during implementation shows another split is essential, report
it for coordinator replanning; do not silently broaden or register new tasks.

## Delivery and recovery sequence

Create/reuse exactly A/B/C, record numbers immediately in #151's single workpad,
and leave them unready before delivery. Validate manifest/ledger, byte-preserved
RFC/catalog and native relationships, then run full managed-container torture,
integrate current main, obtain separate review and protected merge/exact-main CI.

Only afterward append a coordinator amendment to #117 without replacing its
original body or unsuccessful audit. Record the operator's explicit deferred
resumption authorization in its existing workpad. Atomically remove only the
resolved sizing `agent-blocked` label and add `agent-resume-authorized`, preserving
other labels. Do not add ready before C and all normal controls are satisfied.
Replace #117's native prerequisites with C after delivery. No retry-counter
reset or other workspace access is part of this operation.

Refresh all task states, parent controls and prerequisite completion evidence;
upsert #151's final evidence into the single Program workpad. With no other
ready/running successor, select only A using this verified operation's final
report under the pre-closure exception. Recheck controls/events, record selection,
add ready and confirm it before atomically closing #151 and removing its dispatch
labels. Any obstacle remains explicit; no prerequisite is waived. Ordinary
handoff then follows A → B → C → #117 and later eligible tasks. Neither #117
nor M1 is completed by this operation.

README assessment: no root README edit is needed. Registration changes no usable
command, setup, supported behavior, architecture or verified milestone; existing
trajectory rows correctly disclose the missing collector/pending integration.
Quickstart commands are unchanged. Queue status belongs in the Program workpad.
