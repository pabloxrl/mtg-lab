# GH-63 object storage acceptance

Scope: R0002-B015 storage clauses and SYS-CORE-004 storage contract, under
[atomic delivery](../../programs/atomic-delivery.md). No full game catalog case is
assigned to GH-63. The rest of R0002-B015 and card-driven integration remain with
their registered owners. M1 is not complete.

Independent expectations come from the pinned RFC's compact/shared storage,
generation-tagged identity and allocation reuse requirements; SYS-CORE-004's
one-zone ownership and stale-handle requirements; and CR 400.7's new identity on
zone change. [Tests](../../../crates/mtg-core/tests/objects.rs) specify literal
ledgers before and after actual storage operations. They never derive an expected
ledger from the implementation. No spells or token effects are simulated.

The test file was written before implementation. A compileable
[initial API stub](initial-stub.rs.txt) rejects allocation: the
[red run](red.txt) compiles and fails all four tests on actual failed allocation,
not an import/build error. The [green run](green.txt) executes those same assertions
and two additional unit tests successfully. Expectations were not weakened.

Coverage:

- Literal allocation, movement and return ledgers, owner preservation, untouched
  neighboring objects, stable zone order and same-zone no-op.
- Removed-slot reuse and 32 repeated reset epochs. Every retained old handle is
  checked against lookup, movement and removal after new allocation, with the
  complete semantic ledger unchanged on every rejection.
- Every scoped zone, exactly one membership, cross-store rejection and final removal.
- Every frozen identity's key/content hash checked against the manifest, unknown
  key rejection and pointer equality for shared identities across stores.
- Internal boundary injection proves generation/epoch exhaustion rejects without
  state change; exact slot reuse and compact record sizes are checked directly.

Additional seeded adversaries prove the tests catch actual identity defects:
[unchanged zone generation](mutant-zone-generation.txt),
[unchanged reset epoch](mutant-reset-epoch.txt), and
[foreign-store acceptance](mutant-foreign-store.txt). Each mutant was applied alone,
failed assertions and was restored before the green/full-suite run. These are
mutation checks, not alternate production implementations.

Commands inside managed Linux Docker: `cargo test -p mtg-core objects` and
`./scripts/torture.sh`. See [full torture receipt](torture.txt). Full-suite validation
covers debug/release Rust tests, Python regressions, fmt/clippy and repository
metadata. No heavyweight reference execution is claimed by this storage delivery.
Protected PR review/CI and exact-main CI links are preserved in the
[issue workpad](https://github.com/pabloxrl/mtg-lab/issues/63#issuecomment-5855195573).

[Public API and limitations](../../objects.md). README now describes the implemented
storage primitive, its acceptance evidence and exclusions; no milestone verdict changed.
