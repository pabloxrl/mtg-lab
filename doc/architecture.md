# Architecture

## Planned card model

[RFC 0003](rfcs/0003-data-driven-cards.md) defines the target: a versioned runtime
card catalog compiled into typed ability programs for the shared Rust rules engine.
Adding a card made entirely of supported mechanics must require only definitions
and deck/pool data, not an engine or player-code change. The current implementation
below is still a hybrid with compiled identities and specialized ability paths;
the RFC includes their staged migration and an unchanged-binary acceptance test.

## Core module responsibilities

The canonical rules API is now rooted at [mtg_core::game](../crates/mtg-core/src/game.rs).
It owns game state and shared decision identities.
[Opening setup](../crates/mtg-core/src/opening.rs) handles reset, initial hands and
mulligans; the private [work executor](../crates/mtg-core/src/work.rs) resumes owned
opening, turn, spell and combat work. The existing mtg_core::opening API remains
a compatibility re-export of the same types and implementation.

This extraction does not change rules, serialized field schemas or supported
cards. The existing conservative engine fingerprint includes the moved sources,
so snapshots and replays produced before the refactor are explicitly incompatible.
The private [cast state boundary](../crates/mtg-core/src/cast_state.rs) now owns
pending target, cast and payment transitions. Rule entry points keep their
legality checks; policy views read the continuations through immutable accessors.
[Compatibility evidence](evidence/cast-boundary/README.md) covers restoration,
cancellation, rejection and unchanged normal-reset history/capture. Other Game
state remains shared by rule modules. The private
[typed card definitions](../crates/mtg-core/src/card_definitions.rs) provide all twenty
supported cards’ costs, creature characteristics and behavior tags to casting,
targets, mana, activations and combat. Frozen identities/hashes and unsupported-content errors
remain pinned. [Flying/reach and Sentry evidence](evidence/flying-reach/README.md)
covers real 3G Sentry casting, reach blocking, normal-reset replay/capture, and
pinned XMage checks. [Shivan acceptance](evidence/shivan/README.md) covers real 4RR 5/5 flying Shivan,
explicit R activations through the stack, additive power boosts and cleanup. [Invoker acceptance](evidence/invoker/README.md) covers eight-generic targeted +5/+5 and temporary trample. Tajuru vigilance/trample and Thornweald deathtouch are covered by the separate acceptance reports in the [capability reference](capabilities.md). [Compatibility evidence](evidence/card-definitions/README.md)
covers the manifest, rejection boundaries and unchanged played history/capture.
The recorder uses [explicit typed conversions](../crates/mtg-recorder/src/conversion.rs)
from authorized core records into its owned v1/v2 wire contracts. Wire validators
retain validation ownership; [compatibility evidence](evidence/recorder-conversion/README.md)
covers canonical bytes, optional statistics and failure handling.

## Repository map

| Path | Purpose |
| --- | --- |
| [crates/](../crates/) | Rust workspace: core RNG/storage/opening/turn/mana/casting/target/combat/terminal transitions and seat-filtered views, scalar JSONL recorder and executable fixture comparator. |
| [scripts/](../scripts/) | Manifest/scenario validators, verification and reference runners. |
| [data/](../data/) | Frozen rules/card metadata and scoped capability registry. |
| [fixtures/](../fixtures/) | Original scenario and comparator inputs. |
| [references/](../references/) | Reference bridge source, pins and minimal execution receipts. |
| [tests/](../tests/) | Python verification-tool tests. |
| [docker/](../docker/) | Container toolchain and runtime configuration. |
| [doc/](README.md) | RFCs, test plans, operating guides and audit evidence. |

Start with the [project charter](rfcs/0001-project-charter.md) for purpose,
the [current plan](programs/engine-validation.md) for delivery scope, and
[AGENTS.md](../AGENTS.md) for contribution rules. Repository licensing and exact
release/data redistribution decisions remain open release prerequisites, recorded
in the [provenance policy](provenance-policy.md); M0 is not release clearance.
