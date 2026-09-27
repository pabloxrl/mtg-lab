# Core seat-filtered observation acceptance

Atomic #73, R0002-B016/B025/B033 core portion. The lossless RFC ledger and exact
catalog expectations are unchanged. #19 retains full component acceptance,
spell/provisional-choice privacy, snapshots and seat replay export; #27 retains
numeric tensors, padding and batch behavior. No M1 completion claim.

## Independent expectations and executable mapping

Original tests use CR 400.2 (public/hidden zones), 401.2/401.3 (library secrecy and
counts), 402.3 (own hand inspection), and RFC 0001 player knowledge / RFC 0002
B016/B033 information invariance. Exact draw fixtures explicitly place Forest
versus Mountain on top; expected viewer counts are two cards in the other hand
and one in its library after drawing. A seed is not the expected card oracle.

| Assigned case / contract | Normal-discovery test |
| --- | --- |
| `rules-setup-library-privacy-positive` | `views_library_privacy_positive`: own Mountain/Goblin, public Bear Cub, exact life/counts; neither library identity nor privileged metadata. |
| `rules-setup-library-privacy-negative` | `views_library_permutation_alone_preserves_bytes_candidates_and_errors`: independent normal resets, only one unknown library permutation changed, both libraries and both viewer seats. |
| `rules-setup-library-privacy-interaction` | `views_library_privacy_interaction_forest_versus_mountain_draw`: actual `draw_top`, explicit different top cards, equal other-seat bytes/counts, distinct authorized own-hand views. |
| SYS-PRIV-001 core | Independent synthetic and normal-reset pairs vary opponent hand, own/opponent unknown library order, RNG seeds, storage scope/slot allocation and hand insertion order. Complete JSON bytes, candidate rows and ordering agree. Tests include `views_library_privacy_negative_independent_states` and `views_only_unseen_library_permutation_and_hidden_allocation_order_are_invariant`. |
| SYS-PRIV-002 core / B016 | `views_opening_candidates_errors_and_bottoming_use_only_visible_rows`, `views_unavailable_boundaries_and_uniform_errors_do_not_mutate`, `views_error_serialization_is_literal_and_context_free`: wrong actor, stale generation, empty/multiple/out-of-range guessed rows, no decision and internal work. Rejections preserve full privileged Debug state including RNG/history; errors are literal context-free JSON. Real mulligan/redraw/bottom continuations map sorted rows back to the intended card. |
| Remembered revelations | `views_revelations_are_historical_seat_scoped_and_reset`, `views_remembered_facts_do_not_track_hidden_handles_or_order`: historical public and explicit private revelations, hidden moves/removal, invalid foreign handles, both seats, no live hidden location/order, reset clearing. |
| Public/schema boundaries | `views_public_zones_history_and_turn_boundary`, `views_public_terminal_result_preserves_private_filter`, `views_literal_empty_schema_and_owned_snapshot`: all public zones, exact creature damage/boost/tap/sickness, life/mana/actor/turn, terminal seat filtering, literal complete schema and owned snapshots. |
| Explicit integration limit | `views_private_spell_continuations_are_explicitly_unavailable_in_core_schema`: actual pending payment produces `Unavailable` for both viewers. This does not claim #19's spell privacy acceptance passed. |

Unit tests in [views_tests.rs](../../../crates/mtg-core/src/views_tests.rs) declare
synthetic construction; they do not claim reachability for a mixed-library
Forest/Mountain setup. [Integration tests](../../../crates/mtg-core/tests/views.rs)
use real supported red/green configurations, ordered reset and normal opening
choices; explicit replacement orders use the existing privileged chance hook.
No mock spell decisions or fake sibling implementations.

## Red, green and mutations

- [Initial red](red.txt): five compiled behavioral failures against an empty view
  stub, not build/import failure.
- [Opening red](red-opening.txt): two compiled failures for absent candidate and
  terminal records.
- [Boundary red](red-boundaries.txt): missing public actor and erroneously
  available private payment observation both fail before their implementations.
- [Named green](green.txt): `cargo test -p mtg-core views`, 14 tests (11 unit,
  3 integration), all normally discovered.
- [Mutation receipt](mutations.txt): three compiled mutants killed: replace own
  hand by opponent hand, expose internal store identity, retain knowledge across
  reset. Sources restored before final checks. No tests weakened or removed.

During fixture development, the initial paired setup put the complementary land
in each opponent hand; after drawing, both authorized hands were therefore the
same Forest/Mountain multiset. The fixture now uses distinct hidden Growth/Bite
cards so the retained `assert_ne!` genuinely checks different authorized hands.
The draw expectations and all assertions are unchanged. This is a fixture setup
correction, not adoption of an engine-derived expected outcome.

The full `./scripts/torture.sh` runs inside the existing managed Linux Docker
worker, without invoking Docker from the worker. It covers docs/program/catalog,
all Python tests, formatting, Clippy and all Rust debug/release tests; it is not
replaced by named acceptance. [Final complete receipt](torture.txt): 132 Python tests, 130 Rust tests in
each debug/release profile, zero failures or ignored tests; run after integrating
fresh origin/main `c1d8632e0a1c368f7f7eb4829b360314d2daf899`. README adds the usable core observation boundary and links this evidence;
existing quickstart commands are unchanged. Stage table remains unchanged.

No applicable cached reference bridge emits this custom player-view/candidate
schema. Existing XMage/Forge rules scenarios do not verify Rust storage IDs,
serialization or error privacy; no reference agreement is claimed here. The
independently specified information contract and paired executable tests are the
oracle for this delivery. Full reference and cross-feature integration remain
with their registered owners.

[Visibility schema and usage](../../views.md). Independent review, candidate SHA,
protected merge and exact-main CI receipts are preserved in the issue workpad and
PR; local acceptance alone does not constitute delivery.
