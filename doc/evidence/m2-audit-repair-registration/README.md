# M2 audit repair registration evidence

Planning/registration only. The source-contract diagnoses remain unsuccessful
#24/#25 audit evidence; no reference repair or measurement is implemented here.
Base: `1df9b65e95b2976ef9eeb3e46b66877580e2e20c`, freshly fetched 2026-10-10.

- [Original #24 workpad](https://github.com/pabloxrl/mtg-lab/issues/24#issuecomment-6091093721), audit commit `ff4d0d455209c6a3dda2907e011cb25c52b1cc43`.
- [Original #25 workpad](https://github.com/pabloxrl/mtg-lab/issues/25#issuecomment-6090375546), base `71776215b7c2372757b37c4113ac1c0a28bb1362`.
- [Byte-preserved supplied artifacts and SHA-256/size manifest](archive-hashes.json). The supplied [original manifest](originals/manifest.json) pins all six GH-24 artifacts. The archived [audit README](originals/gh24/README.md.original) retains original bytes and original relative links, interpreted at its audit-commit path; `.original` marks archival text, not relocated live documentation. Inventory, dependency receipts, source hashes, diagnostic review and full raw torture log are preserved unchanged. Supplied GitHub issue/comment JSON for both audits is also retained byte-for-byte. There was no new GH-25 local diagnostic implementation artifact; its source/workpad evidence is preserved, not invented.
- [Re-fetched delivered prerequisite workpads](dependency-workpads.json), [successful CI receipts](dependency-ci.json), and [child registry](children.json).
- [Baseline preservation snapshot](preservation-baseline.json): all 129 original tasks, complete verbatim ledger and source hashes. Run the one-shot [registration checker](check_registration.py) on this registration candidate. Later feature changes may legitimately change source files; this retrospective checker is not a new permanent implementation gate.
- [Plan/graph and clause crosswalk](../../programs/m2-audit-repair-prerequisites.md), [complete child contracts](../../programs/m2-audit-repair-contracts.md).

Validation commands: `python3 scripts/check_program.py`,
`python3 doc/evidence/m2-audit-repair-registration/check_registration.py`,
`python3 scripts/check_docs.py`, `python3 scripts/run_tests.py`, followed by full
`./scripts/torture.sh` under the unchanged heavy lock. Final exact-candidate separate
review, protected PR and exact-main CI receipts belong in the
[#268 workpad](https://github.com/pabloxrl/mtg-lab/issues/268#issuecomment-6092397740).
Registration completion is conditional on those delivery receipts, not this file.

README impact: no usable command, setup, implemented behavior, architecture or
verified milestone status changes. New child commands are explicitly future delivery
interfaces; README already keeps M2 incomplete and describes existing mode/measurement
limits accurately. No root README edit or quickstart change is warranted; existing
quickstart checks remain in full torture and the separate reviewer must verify claims.
