# Semantic action records v1

`mtg_core::opening::actions` encodes, resolves and applies standalone privileged
records for every currently implemented policy decision: keep/mulligan/ordered
bottom, pass, land, basic-land mana, supported creature/Growth/Bite casts,
ordered targets, all six payment colors, explicit finish/cancel, factored
attackers/blockers/damage, and cleanup discards. Out-of-band concession is also
supported for either seat. This is the GH-113 component of R0002-B016/B025/B033;
#19 retains integration acceptance. Full-game replay envelopes, compatibility
pins, consumed-stream validation and checkpoints remain #114. M1 is incomplete.

Use `encode(&game, actor, &policy_submission, capacity)` before applying a live
policy choice, or `encode_concession(&game, actor)`. Both return owned JSON bytes.
`decode(&game, bytes, capacity)` validates without mutating the game and returns
`Decoded::Decision { actor, submission }` or `Decoded::Concession { actor, episode }`
with fresh local tokens. `apply(&mut game, bytes, capacity)` validates and executes
through the existing core. A decoded submission expires under normal policy
revision/generation rules; never serialize those tokens. Input byte limits belong
to the caller; the capacity argument retains the policy decision-domain limit.

For example, a keep record is:

```json
{"version":1,"actor":"P0","decision":"keep_or_mulligan","choices":[{"kind":"keep"}]}
```

A land choice in a `priority` record looks like:

```json
{"kind":"play_land","card":{"birth":3,"card":"forest","owner":"P0","zone":{"Hand":"P0"},"incarnation":1}}
```

That particular birth describes the fourth P0 card in the hand-authored test
configuration, not an arbitrary game's fourth hand row. `birth` is the zero-based
creation ordinal within the episode: reset allocates P0's 40 cards in the resolved
initial library order, then P1's 40. Later births increase monotonically, even if
storage slots are reused. `incarnation` starts at zero and increases on every
actual zone change (CR 400.7); same-zone operations leave it unchanged. Drawing
makes incarnation 1; a land play makes 2; hand→stack→battlefield makes a creature
3. Birth survives moves, tapping and reordering. Reset restarts the birth sequence;
restoring a snapshot preserves it while rebasing local capability scopes.

Every object reference requires birth, frozen card key, owner, current zone and
incarnation. Resolution searches the current actor's hand or public battlefield;
it never substitutes a same-name copy or uses a saved row/handle. These fields
are intentionally privileged: initial creation order and cross-zone linkage can
reveal information. They must not become policy observation features or be sent
to opponent channels. Policy clients continue using seat-filtered visible rows.

Records require version, actor, decision kind and choices. Decision names match
`policy::Decision::kind`; choice names/structure match the policy schema with
object references replacing **every** visible row. `select_blockers.blocks` is an
array of `[blocker, attacker]` pairs; `assign_damage` has `attacker` and
`amounts: [[blocker, amount], ...]`. Ordered bottom/discard choices are an array
of `bottom`/`discard` objects. A concession requires decision `concession` and
exactly `[{"kind":"concede"}]`; it does not require priority.

Missing/unknown fields, incompatible action versions, wrong actor/kind,
ineligible/missing/stale objects, duplicate selections and illegal payments or
combat allocations fail without changing full game state, RNG or pending choices.
Encoding also rejects illegal or stale live submissions. Validation runs the
existing policy/core path on an isolated snapshot, so it adds copying and execution
cost; this persistence/debug API is not the throughput path or a second rules engine.
Snapshot compatibility fingerprints include the codec and birth metadata; old
incompatible snapshots are rejected explicitly.

Action v1 defines syntax and identity semantics, not a complete replay protocol.
A caller must recreate the same configuration, initial ordering/RNG and preceding
choices. Records intentionally carry no process scope, episode token, decision
counter, row index or state hash. Thus a legal action can be reapplied at another
compatible decision, and a structurally compatible record is not proof of the
same historical game. The future envelope must bind configuration, engine/rules/
cards versions, stream position and checkpoints. Existing opening replay v1 still
works with its own envelope and hand-occurrence schema; this module does not
rewrite it or extend the opening CLI to played games.

Run `cargo test -p mtg-core --lib opening::actions::tests`, then the full
`./scripts/torture.sh` in the managed container. See [acceptance evidence](evidence/actions/README.md).
