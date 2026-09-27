# Structured policy decisions, schema 1

`Game::policy_observe(seat, capacity)` returns an owned `policy::Observation`:
policy schema version, the existing schema-1 `PlayerView`, an actor-only decision,
actor-only provisional spell state, committed stack target rows, and explicit unsupported families. Use `Game::apply_policy(actor, &Submission,
capacity)` to submit semantic choices. Types live in `mtg_core::opening::policy`
and serialize through Serde; submissions also deserialize. The trusted caller
binds the game and authorized seat. Neither a seat nor a revision/generation pair is an
authentication token; transport routing/idempotency remains later work.

The supported subset is opening keep/mulligan and ordered bottoming, priority
passes, Forest/Mountain land plays and tap mana, and standalone payment choices
initiated by trusted rules code, plus Bear Cub/Swab Goblin casting and Giant
Growth/Bite Down source/destination targeting, staged land mana, payment, finish
and cancel. No policy-supplied cost, object handle, RNG,
shuffle order, opponent hand or library reference is accepted. Existing
`observe` and `apply_opening_view` retain their schema and behavior (including
`observe` being unavailable during **any** payment). The new policy projection
also works during targeting, casting and standalone payment and exposes only the committed public pool
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
| `priority` | Pass; one `play_land { card }` per own-hand row; `cast { card }` for each supported spell in own hand; one `tap_mana { card }` per public battlefield row | One unmasked choice |
| `growth_target`, `bite_source`, `bite_destination`, `targets_complete` | `target { card }` per battlefield row; `finish_targets`; `cancel_targets` | One unmasked choice; Bite source precedes destination |
| `payment` | `pay { color }` for W/U/B/R/G/C (0–5), finish, cast-only battlefield tap rows, cancel | One unmasked choice |

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
family error. Combat decisions return that family error for both seats; cleanup discard
decisions return `unsupported_decision`. No empty/default action silently completes them.
The legacy fieldless `spell` request remains an explicit unsupported request;
use `cast { card }` for supported spell choices.

Every successful observation advertises unsupported families `combat` and
`cleanup_discard`. Casting legality uses the existing rules: instant timing,
required legal targets, sorcery timing for creatures, and sufficient resources.
`pending` is null for the other seat. For the acting seat it identifies the own-hand
spell, selected targets, staged mana sources, provisional pool and remaining cost
(the last two are null during targeting). All references use the accompanying
view rows. Required targets must be selected explicitly, followed by
`finish_targets`; no response window appears during a cast.

`finish_payment` joins the spell, targets, taps and spending in one commit and
leaves priority with its caster. It is masked while any cost remains, even when
no color is currently payable. `cancel_targets` or `cancel_payment` restores the
committed position, including staged taps; already floated mana remains real.
Only decision generation advances. Opponent observations are identical throughout
provisional choices, including cancellation. `stack` is bottom-to-top, with `row`
indexing the public stack cards and ordered `targets` indexing the battlefield.
A departed target is null, never rebound to a new object or exposed in a hidden
zone. The older `observe` API remains unavailable for private continuations;
use `policy_observe` for this supported projection.

Passing may reach a combat/cleanup decision outside this API's supported subset.
Complete played-game policy, replay and #19 integration acceptance remain pending;
no M1 completion claim until #22.
No Python, tensors, trainers, new rules or CLI protocol are added here.

Run `cargo test -p mtg-core policy`, then `./scripts/torture.sh` in the managed
container. [Acceptance evidence](evidence/policy/README.md) distinguishes normal
reset scripts, synthetic resource/storage/payment states and reference limits.

[Spell acceptance and privacy evidence](evidence/policy-spells/README.md) covers
normal-reset and explicitly synthetic response tests.
