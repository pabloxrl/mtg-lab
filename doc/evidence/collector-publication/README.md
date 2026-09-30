# GH-164 local publication acceptance

Related to #164; partial R0002-B036/B037 ownership only. #154/#117 retain all
aggregate collector clauses. No milestone or full-pool/reference claim.

[Normal-discovery tests](../../../crates/mtg-recorder/src/publication_tests.rs)
use real captured normal reset, seed **164**, ordinal **7**, default frozen config.
The independent ledger is P0 Keep, P1 Keep (CR 103.5), then P1 out-of-band concession
(CR 104.3a). Literal expected opening life [20,20], hands [7,7], libraries [33,33],
Keep/Mulligan domains and legal masks, two rows [(0,0),(1,1)], zero intermediate
rewards and terminal returns [1,-1] come from those rules and RFC reward semantics.
Canonical equality is an additional losslessness check, not the behavioral oracle.
Authorized disk replay must reconstruct the actual final full state/RNG, normalizing
only process-local namespace IDs. Both seat readers exclude privileged fields.

The [compiled scaffold red](behavioral-red.txt) has two intended behavioral failures
(normal and diagnostic publication), with the authorization rejection already
passing. Earlier fixture compilation errors were corrected and are not behavioral
red evidence. No existing test was removed, skipped or weakened.

Eleven tests cover the positive composition, actual decision-limit truncation,
record-capacity failure and unfinished capture; missing references/denied access
and actual ID/config mismatch; same ID/ordinal with a different real seed/history
binding (seed 165); no-clobber/retry and Linux private modes; corrupt or
missing disk data before publication and after publication; every write/sync/link
and manifest boundary; interruption before every file boundary; actual failing
no-replace hard links and retained dataset cleanup failure; explicit uncertain
manifest commit when both directory sync and withdrawal fail; run-directory
creation/parent-sync failure and whether reservation has consumed an ID. The
[directory-hook red](directory-red.txt) demonstrates that added fault boundary
before its hook wiring. Injected I/O hooks
and panic interruption are declared synthetic fault boundaries around the real
filesystem, not substitute games. A post-link interruption can only expose a
complete validated run; it cannot guarantee a returned acknowledgment or final
sync. Real no-replace collision tests exercise the hard-link syscall itself.

Reproduce: `cargo test -p mtg-recorder publication::tests`.
Mandatory full verification: `./scripts/torture.sh` (normal discovery in debug and
release, plus Python/docs/program/catalog/fmt/Clippy). [Final full log](torture-final.txt) passes 143 Python tests and **398 Rust
tests/doctests in each debug/release profile**, zero ignored; docs/program/catalog,
fmt and Clippy pass. [Focused green](focused-green.txt) covers all eleven tests.
[Verification receipt](verification.json) records source/log hashes. Fresh main
4115ebf7d402efd94bc346cc04a6eef7a07207e1 was integrated before the final run.
Only a test comment's seed typo was corrected during the final run; executable
implementation and assertions were unchanged. Preliminary full torture also
passed (396 debug / 398 release), but its last two tests were added during that
run, so it is not the final unchanged-implementation evidence. Separate review,
protected PR merge and exact-main CI are recorded in the PR/issue workpad. The small concession script qualifies this publication
component only; existing 27-turn capture acceptance and later integration audits
retain their broader obligations.

README now describes local publication, separate authorization, retained-result
replay reads and recovery limits. Quickstart commands and stage table are unchanged;
full verification covers their existing command checks. The public API and manual
recovery semantics are in [collector-publication.md](../../collector-publication.md).
