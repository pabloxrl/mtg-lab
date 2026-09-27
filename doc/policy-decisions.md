# Structured turn and land decisions, schema 1

`Game::policy_observe(seat, capacity)` returns an owned `policy::Observation`:
policy schema version, the existing schema-1 `PlayerView`, an actor-only decision,
and explicit unsupported families. Use `Game::apply_policy(actor, &Submission,
capacity)` to submit semantic choices. Types live in `mtg_core::opening::policy`
and serialize through Serde; submissions also deserialize. The trusted caller
binds the game and authorized seat. Neither a seat nor a revision/generation pair is an
authentication token; transport routing/idempotency remains later work.

The supported subset is opening keep/mulligan and ordered bottoming, priority
passes, Forest/Mountain land plays and tap mana, and standalone payment choices
initiated by trusted rules code. No policy-supplied cost, object handle, RNG,
shuffle order, opponent hand or library reference is accepted. Existing
`observe` and `apply_opening_view` retain their schema and behavior (including
`observe` being unavailable during **any** payment). The new policy projection
also works during standalone payment and exposes only the committed public pool
to either seat, plus payer-only color choices. It does not expose the provisional
pool, remaining cost or choices to the opponent.

## Tables and references

Each actor decision contains `revision`, `generation`, persistent `actor`, `kind`, required
`count`, `candidates` and a parallel `legal_mask`. The other seat receives
`decision: null`, not the acting player's private rows. Tables are dynamic,
unpadded and never truncated. `capacity` is a caller-supplied maximum number of
candidate rows, including masked rows; exceeding it returns `capacity_exceeded`
before any rules mutation. Consumers must stop/report the episode or explicitly
retry with sufficient resources; treating overflow as pass is invalid. This is
not a fixed tensor bound or a guarantee that all observation allocations fit a
particular memory budget.

A `VisibleRef { zone, row }` addresses the current authorized `view.hand` (zone
`hand`) or the `battlefield` public-zone list. Own-hand rows sort by card key;
indistinguishable duplicates retain their existing tie order. Battlefield rows
retain public zone-entry order. These references are stable for a decision
and independent of private storage slots. They are **not persistent object IDs**:
refresh them and the revision/generation pair after every accepted action, reset
or restore. The destination-local revision starts at zero and increments on
every successful snapshot restore. It is not a storage scope or object handle,
is not saved/loaded from snapshots, and cannot rewind when an old save is loaded.
Revision exhaustion rejects restore without changing state. Tapping
does not reorder a battlefield row; moving a card changes its zone/reference.
References to an opponent hand or either library cannot be represented.

| Kind | Candidate order | Selection |
| --- | --- | --- |
| `keep_or_mulligan` | Existing opening choices: keep, then mulligan when offered | One choice |
| `bottom` | One `bottom { card }` per sorted own-hand row | Exactly `count` distinct cards, in intended bottom order |
| `priority` | Pass; one `play_land { card }` per own-hand row; one `tap_mana { card }` per public battlefield row | One unmasked choice |
| `payment` | `pay { color }` for W/U/B/R/G/C (0–5), finish, cancel | One unmasked choice |

Priority masks use the existing rules' land/source legality. Nonland hand cards,
second lands, tapped lands and opponent-controlled sources remain visible but
masked. Payment masks use the existing exact-symbol-first/generic continuation;
finish becomes legal only when payment is complete. Cancellation discards the
provisional spending, preserving previously floated mana. There is no opponent
priority between payment selections. Normal opening still reaches an explicit
opening-complete boundary; the trusted runner calls `start_turns`, not a policy
choice. A terminal result has no decision.

## Submissions and errors

Example semantic land command (the generation comes from the current decision):

```json
{"schema_version":1,"revision":0,"generation":5,"choices":[{"kind":"play_land","card":{"zone":"hand","row":0}}]}
```

The boundary validates version, readiness, actor, revision/generation, unsupported
families, capacity, cardinality, membership, mask and distinctness before
resolving any visible reference internally. It then calls the existing
transactional rules transition. Rejecting stale input, a guessed reference, a
masked choice, duplicate bottom rows, wrong actor, pre-restore command or resource exhaustion leaves
the complete state/RNG/history unchanged. Repeating an accepted command is stale,
not an idempotent transport acknowledgment.

Errors are context-free snake-case tags: `unsupported_version`, `unavailable`,
`wrong_actor`, `stale_decision`, `invalid_selection`, `capacity_exceeded`,
`unsupported_spell`, `unsupported_combat`, `unsupported_decision`. Finite engine
counter/mana exhaustion also maps to `capacity_exceeded`; no private identifiers
or data are included. Before reset and during internal work the interface is
unavailable. Unsupported requested spell/combat choices return their explicit
family error. Pending casting/targeting and combat decisions also return that
family error for both seats; cleanup discard decisions return
`unsupported_decision`. No empty/default action silently completes them.

Every successful observation explicitly advertises unsupported families
`spell`, `combat`, `cleanup_discard`. Priority candidates are only the supported
subset; their absence does not mean casting is illegal. Existing trusted rules
APIs can cast and declare combat, but this policy interface cannot yet do so.
Passing may advance to such an unsupported decision, which then stops policy
execution. A pass can resolve an already committed supported stack through the
existing engine; this does not provide policy spell selection or private cast
views. This API alone does not yet drive a complete played game. Spell/combat
siblings and #19 retain integration acceptance; no M1 completion claim until #22.
No Python, tensors, trainers, new rules or CLI protocol are added here.

Run `cargo test -p mtg-core policy`, then `./scripts/torture.sh` in the managed
container. [Acceptance evidence](evidence/policy/README.md) distinguishes normal
reset scripts, synthetic resource/storage/payment states and reference limits.
