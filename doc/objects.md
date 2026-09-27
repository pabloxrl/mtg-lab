# Object storage contract

`mtg_core::objects` implements the storage portion of R0002-B015 / SYS-CORE-004.
It is a storage primitive, not a playable engine or a legality checker. Card-driven
identity integration, token disappearance, stack behavior and full reset/opening
hands remain subsequent deliveries. [Acceptance evidence](evidence/objects/README.md)
uses literal storage ledgers, not spell simulations or reference-engine agreement.

```rust
use mtg_core::objects::{CardId, ObjectStore, Seat, Zone};
let mut objects = ObjectStore::new().unwrap();
let bear = CardId::from_key("bear-cub").unwrap();
let hand = objects.allocate(bear, Seat::P0, Zone::Hand(Seat::P0)).unwrap();
let battlefield = objects.move_to(hand, Zone::Battlefield).unwrap();
assert!(objects.get(hand).is_err());
assert_eq!(objects.get(battlefield).unwrap().card, bear);
objects.reset().unwrap();
assert!(objects.get(battlefield).is_err());
```

Card IDs are compact indices into one immutable static identity table. Its keys
and content hashes are an exact projection of the frozen
[manifest](../data/cards/foundations_micro_v1.json); tests check every entry.
A hash pins the complete card/token metadata. This table does not interpret text
or implement effects. Unknown keys return `None`. The full card definitions remain
in the manifest; per-object records carry the index, original owner, zone, controller and tapped status.
Allocation and zone changes initialize controller to owner and status to untapped;
same-zone movement preserves status. The [turn prefix](turns.md) uses controller
and tapped status for untap. Status mutation is internal to the rules core.
Numeric indices are internal to this frozen table, not a versioned interchange API.

Each store owns dense slots, a free list and nine ordered zone vectors: each seat's
library, hand and graveyard, plus shared battlefield, stack and exile. These are
the scoped pool's zones; command-zone mechanics are not implemented. Each live
slot occurs exactly once in its zone. Allocation and movement append; removing a
member preserves the order of all remaining members. No top-of-library convention,
shuffle, arbitrary reordering, placement legality or policy-view filtering is
implied. Callers must not expose these privileged storage views to a player.

`move_to` returns a replacement handle when the zone changes (CR 400.7). The
previous handle fails lookup, movement and removal. Moving to the current zone is
a no-op with the same identity and position. Removal invalidates the handle;
allocation reuses a vacant slot with a new generation. `reset` clears all slots
and zones, advances the epoch and retains vector capacities. Every earlier handle
then fails even after the same slot and generation numbers are used again.

Handles include a private store identity, epoch, slot and generation. A process-wide
checked atomic counter assigns store identities once on construction; there is no
atomic operation per storage transition. Foreign-store handles fail. These opaque
handles are process-local capabilities, not semantic replay/snapshot identifiers.
Store identity may differ with construction order; it must never influence game
rules, RNG or serialized semantic results. Snapshot/replay integration must define
its own semantic mapping. Stores cannot be cloned; shared immutable card identities
can be copied cheaply and referenced from any store.

All counters use checked increment. Identity exhaustion returns `IdentityExhausted`
without semantic mutation; it never wraps. Exhausted vacant slots currently cause
allocation to fail explicitly, rather than silently bypassing the slot. Slot-index
or vector reservation failure returns `CapacityExceeded`. Invalid handles return
`InvalidHandle` before mutation. A failed allocation may grow reserved capacity,
but does not change any live object or zone. There is no silent object truncation.
Memory sizes are tested (object ≤8 bytes, slot ≤24 bytes); this is not a whole-game
capacity benchmark or proof of the RFC performance targets.

Run `cargo test -p mtg-core objects` and `./scripts/torture.sh` inside the managed
toolchain container. Both discover all storage tests; the full suite also executes
them in release mode. The root Docker quickstart runs the same full suite.
