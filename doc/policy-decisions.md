# Structured policy decisions, schema 1

`Game::policy_observe(seat, capacity)` returns an owned `policy::Observation`:
policy schema version, the existing schema-1 `PlayerView`, an actor-only decision,
actor-only provisional spell state, committed stack target rows and combat relationships, and explicit unsupported families. Use `Game::apply_policy(actor, &Submission,
capacity)` to submit semantic choices. Types live in `mtg_core::opening::policy`
and serialize through Serde; submissions also deserialize. The trusted caller
binds the game and authorized seat. Neither a seat nor a revision/generation pair is an
authentication token; transport routing/idempotency remains later work.

The supported subset is opening keep/mulligan and ordered bottoming, priority
passes, Forest/Mountain land plays and tap mana, and standalone payment choices
initiated by trusted rules code, plus Bear Cub/Swab Goblin casting and Giant
Growth/Bite Down source/destination targeting, staged land mana, payment, finish
and cancel, vanilla combat declarations/damage allocation, and cleanup discards. No policy-supplied cost, object handle, RNG,
shuffle order, opponent hand or library reference is accepted. Existing
`observe` and `apply_opening_view` retain their schema and behavior (including
`observe` being unavailable during **any** payment). The new policy projection
also works during targeting, casting and standalone payment and exposes only the committed public pool
to either seat, plus payer-only color choices. It does not expose the provisional
pool, remaining cost or choices to the opponent.

## Tables and references

Each actor decision contains `revision`, `generation`, persistent `actor`, `kind`, required
`count`, `candidates`, a parallel `legal_mask`, and optional `factored` combat domains. The other seat receives
`decision: null`, not the acting player's private rows. Tables are dynamic,
unpadded and never truncated. `capacity` is a caller-supplied maximum number of
candidate rows, including masked rows and factored rows as defined below; exceeding it returns `capacity_exceeded`
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
| `cleanup_discard` | One `discard { card }` per sorted own-hand row | Exactly `count` distinct cards |
| `attackers`, `blockers`, `combat_damage` | `finish_combat`; structured `factored` domains below | One finish or structured replacement command |
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
family error. The fieldless legacy `combat` request is unsupported; use the
structured commands below. No empty/default submission silently finishes combat.
The legacy fieldless `spell` request remains an explicit unsupported request;
use `cast { card }` for supported spell choices.

The `unsupported_families` array is now empty for the implemented decision families;
this does not imply additional card or keyword support. Casting legality uses the existing rules: instant timing,
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

Complete played-game policy, replay and #19 integration acceptance remain pending;
no M1 completion claim until #22.
No Python, tensors, trainers, new rules or CLI protocol are added here.

Run `cargo test -p mtg-core policy`, then `./scripts/torture.sh` in the managed
container. [Acceptance evidence](evidence/policy/README.md) distinguishes normal
reset scripts, synthetic resource/storage/payment states and reference limits.

[Spell acceptance and privacy evidence](evidence/policy-spells/README.md) covers
normal-reset and explicitly synthetic response tests.

## Factored combat and cleanup

Combat supports the core's Bear Cub/Swab Goblin and existing Growth modifications;
no flying, trample, deathtouch or other combat keywords are added. `combat` is a
public list of `{ attacker, blocked, blockers }` using current battlefield rows.
Departed creatures are removed from those relationships; remembered `blocked`
remains true when all blockers leave. A returning object is not the old blocker.
Only committed declarations appear here. Priority windows remain the core's windows.

`decision.factored` is null outside combat and absent with the opponent's entire
decision. It contains legal `attackers`, legal `blockers`, actor-only provisional
`selected` and `blocks`, and `damage` entries. Domains use public battlefield
order (committed attacker/blocker order for damage). These are legal domains,
not masked illegal rows or an exponential list of combined actions:

- `attackers`: submit one `select_attackers { cards: [VisibleRef, ...] }` with any
  distinct subset of the attacker domain, including empty. This replaces the
  provisional selection. `finish_combat` commits it and taps attackers.
- `blockers`: submit one `select_blockers { blocks: [[blocker, attacker], ...] }`.
  Every reference must belong to its domain. Each blocker appears at most once;
  multiple blockers can choose the same attacker. Empty clears the provisional map.
  `finish_combat` commits it. There is no damage assignment order.
- `combat_damage`: each `damage` entry identifies an attacker, current `power`,
  legal `blockers`, and actor-only `amounts` (null until assigned). Submit one
  `assign_damage { attacker, amounts: [[blocker, nonnegative_u32], ...] }` for a
  multiply blocked attacker. Recipients must be distinct and amounts sum to its
  full power; omitted blockers get zero. Repeating this command replaces that
  attacker's allocation. Any division is legal, including 1+1 for a 2/2 against
  two 2/2 blockers. `finish_combat` is masked until all required allocations exist;
  the existing rules validate and commit simultaneous damage. Single/unblocked
  assignments are determined by the rules, without a discretionary default.

Every command carries the current revision/generation and exactly one structured
choice. Domain membership is validated before resolving internal handles; core
validation enforces subset/map distinctness and allocation sums transactionally.
No opponent sees provisional changes, including replacements with empty choices.

Capacity counts flat candidates plus each attacker/blocker/selected reference,
each provisional block pair, and for each damage entry one header plus each
blocker reference and each assigned amount pair. The whole current decision must
fit, including on submission; no list is clipped. A later decision can require
more space and fail observation explicitly, so callers must handle capacity errors
at every boundary. Capacity does not bound public observation size or allocator
memory usage. The representation grows with creatures and actual relationships,
not the number of possible subsets, maps or integer compositions.

Cleanup uses exactly `count` distinct `discard` choices from the sorted own hand.
The adapter maps those rows back to the core's hand order; it does not interpret
them as privileged storage indices. No provisional discard is revealed. Selection
commits through the core's existing cleanup and next-turn processing.

[Combat/discard acceptance](evidence/policy-combat/README.md) covers both-seat
reachable scripts, independent exhaustive small choices, public relationships,
privacy, capacity, and atomic rejection.

### Flying/reach pair restrictions

Combat domains optionally include `forbidden_blocks`: blocker/attacker pairs
excluded from the Cartesian product. An absent/empty list means no pair
restrictions (the earlier vanilla encoding). It is not a list of whole maps.
Scalar and policy submission enforce the same rule: a flying attacker requires
a flying or reach blocker; tapped creatures never enter the blocker domain.
Reach does not restrict which creatures may block its bearer when it attacks.
Each excluded pair counts as one capacity row; excess fails explicitly. The
acting player alone receives this domain. Semantic action kinds are unchanged;
typed trajectory v2 preserves and validates these optional restrictions. Prior
engine snapshots/replays remain incompatible by source fingerprint.
