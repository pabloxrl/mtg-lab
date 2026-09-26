# M0 provenance and distribution decision

Recorded 2026-09-26 for [GH-11](https://github.com/pabloxrl/mtg-lab/issues/11),
covering the provenance/distribution aspects of RFC 0002 `R0002-B024`.
This is a conservative project decision and source inspection, not a legal
assurance. No repository license has been selected. Public release remains
blocked on an explicit owner selection and the artifact review below.

## Distribution boundary

M0 repository material is limited to original project code/documentation and
reviewed metadata/fixtures. Review is per artifact, not a blanket approval for
anything called metadata. Source URLs, revision identifiers, checksums, rule
numbers, original explanations, and independently justified synthetic scenarios
are the preferred records. Raw Oracle text, full rules/release-note documents,
card databases, artwork, symbols, and upstream reference code/data are fetched
separately, excluded from repository and release bundles. No live API belongs in
the production simulation path. Later card manifests must distinguish reviewed
identifiers/characteristics from separately retrieved text and retain hashes.

Keep Forge, its code, tests, card scripts and data separately installed and
versioned. Do not copy or translate GPL tests into this corpus under M0. Run
reference bridges out of process, but do not treat that boundary as a licensing
exemption. Review bridge dependencies and patches before distribution; external
installation does not settle their obligations. Neither reference engine is
vendored wholesale. XMage adaptations require the exact source and notice review
below before admission; none are introduced by this change.

This boundary applies to packaged artifacts, CI uploads, examples and published
replay/differential traces as well as tracked files. A log containing copied card
text is still data redistribution. Review output contents; publish minimal
reviewed semantic results and metadata, not unchecked external payloads.

## Primary-source findings

[Inspection receipts](provenance-sources.json) record each fetched URL, raw-byte
SHA-256, byte count and retrieval date. GitHub URLs in the receipts use the exact
survey commits, not moving branches. These are inspection pins, not validated
reference-build selections. Dynamic HTML hashes identify the response inspected;
future responses may differ even without substantive terms changes. Original
responses remain external; these receipts do not archive their contents.

| Source / receipt IDs | Inspected finding | Consequence and unresolved boundary |
| --- | --- | --- |
| XMage `000d8a7abc0ac31cc24af08691423e0c24dc59e7`: `xmage-license` | `LICENSE.txt` says MIT, copyright (c) 2010 betasteward@gmail.com; requires inclusion of copyright and permission notices in copies or substantial portions, with warranty disclaimer. | Preserve the complete MIT notice with any adaptation, plus all applicable file notices. MIT code permission is not a grant over Wizards card/rules content. |
| XMage `xmage-damage`, `xmage-base` | `AssignDamageTest.java` has an `@author JayDi85` notice and three Thorn Elemental methods, including an AI-driven case. `CardTestPlayerBase.java` has `@author ayratn`, default RB Aggro decks and a 60-card game constructor. Neither inspected file has an additional license header. | Retain author notices for adapted material; inspect every additional source file. Thorn Elemental is outside the pool: this is a harness example, not an accepted fixture. Replace hidden setup and AI choices with explicit neutral scripts, without calling the result original. |
| Forge `95dc682bf92460f49cebd7a9578f06ccf60d5569`: `forge-license`, `forge-readme`, `forge-base` | `LICENSE` contains GNU GPL version 3, 29 June 2007; README labels it GPL-3.0. `BaseGameSimulationTest.java` has no separate license header in the inspected file. GPL sections 4–6 address conveying source, modified source and non-source forms. | Do not infer an “or later” grant from the generic license appendix. This is not a dependency/card-data-wide license audit. Keep all Forge material external; resolve exact terms for any future bridge/patch distribution. |
| Official rules `cr` | Effective September 25, 2026; raw SHA matches the [rules manifest](../data/rules/cr-2026-09-25.json). | Use rule-number citations and original explanations; fetch full text separately using the [rules-source procedure](rules-source.md). Availability does not establish redistribution permission. |
| Official Foundations release notes `foundations` | Article displays Nov 1, 2024 and “Document last modified August 23, 2024”; official card-specific rulings and mechanic notes are scenario research material. Preserve both dates rather than assume they agree. | Record publication/version and applicability for each selected ruling. Do not copy the article or accept out-of-pool examples as scoped tests. |
| Scryfall `scryfall-api`, `scryfall-terms` | API “Use of Scryfall Data and Images” describes free data for Magic software/research/community content, disallows paywalling, new games, misleading endorsement and simple repackaging/republishing/proxying; requires added end-user value. API requires accurate User-Agent and Accept headers. Terms prohibit undue automated burden; footer attributes literal/graphical Magic information to Wizards. | Free API access is not an unrestricted data license. Review the exact manifest fields/use before redistribution. Initial API-doc request returned 403; an accurate application User-Agent and Accept header retry returned 200. No unresolved access failure for this inspection. |
| Wizards `wizards-fan` | Fan Content Policy, last updated November 15, 2017: free access and unofficial notice requirements, preservation of legal notices, restrictions on use in games, trademarks/logos and verbatim reposting; permission may be withdrawn. | Do not assert the policy permits this engine or its data distribution merely because it is research or free. Applicability is an unresolved release question. |
| Wizards `wizards-terms` | General Terms, “License” and “Wizards Ownership”: limited personal/non-commercial service-use license and reservation of IP/content rights; no implied rights. Fan policy says Wizards Terms prevail in a conflict. | Do not treat downloading rules or receiving MIT/GPL engine source as permission over all underlying content. Review intended use and exact release payload. |

For reproduction, retrieve a receipt URL into an untracked external cache with a
bounded timeout and compare its raw SHA-256. Inspect license, headers and terms,
not just HTTP success. A mismatch requires a new dated inspection; never overwrite
an old finding as though the bytes were unchanged. Access failures must record
URL, time, status/error, attempts and impacted artifacts, without credentials.

## Provenance record format (version 1)

Use one review record per fixture, in a sidecar or the future fixture schema.
This is a documentation format, not a claim of schema/runner enforcement. The
existing synthetic comparator schema remains unchanged; GH-12 owns integration.
Required fields are below; empty values must have a specific applicability reason,
not be silently omitted. A record starts `pending`, never implicitly approved.

| Field | Required contents |
| --- | --- |
| `provenance_version`, `fixture_id`, `fixture_revision` | `1`, stable ID, immutable fixture content digest/revision. |
| `source_kind`, `author`, `created_at` | `original`, `adapted`, `regression`, or `differential`; responsible author and date. |
| `origins` | All contributing sources, including nested adaptation ancestry: URL, exact commit or publication/version, retrieval date, raw hash, file path and method/section. Use an explicit “no upstream code adapted” statement where applicable. |
| `license_review` | Exact license/terms source and digest, inspected file-level notices, retained notice paths and applicable full notice text, reviewer/date, distribution decision and unresolved blocker IDs. Original project material records “repository license unselected”; do not assign MIT merely because XMage was consulted. |
| `modifications` | What was copied/translated, omitted or rewritten; setup/default changes, choice scripting and expectation changes. Retain upstream identity even after extensive rewriting. |
| `rules_basis` | Rules version and document hash, numbered rules, card manifest version/hash, Oracle source/version/hash, ruling publication date and section where used. |
| `expected_result_basis` | Independent derivation for each meaningful checkpoint, citations and reviewer; identify which engine output was observed versus independently justified. |
| `vintage_audit` | Upstream rules/card vintage (or explicitly unknown), target vintage, semantic differences, current-rule adjudication, reviewer/date and status. |
| `reproduction` | Synthetic versus reachable setup, fixture/action trace digest, ordered randomness, complete choices, command/tool versions and relevant failure artifacts. |
| `reference_evidence` | Engine/source/card/bridge pins and local patches; planned/executed/agreed/disputed/unsupported/unavailable result, artifact digests and observable fields per engine. “Not run” is not agreement. |
| `review` | `pending`, `accepted-for-m0`, or `blocked`; reviewer/date, evidence links, admission scope and blocker IDs. M0 admission is not public release authorization. |

Origin-specific requirements:

- **Original:** write the setup, script and expectations independently from numbered
  rules and applicable rulings; disclose consulted sources. Do not label a
  translation of upstream code original or describe an adapted corpus as clean-room.
- **Adapted:** pin repository commit, exact file and method for every source;
  retain full MIT copyright/permission/disclaimer and file notices beside the
  adaptation, linked from the record. List all modifications. Reject Forge/GPL
  copies or translations under this M0 decision. A run in the source engine does
  not provide independent corroboration of its own adapted test.
- **Regression:** link defect report, failing engine revision, original trace/seed,
  minimization steps and minimized replay hash. Retain the underlying origins and
  notices. Derive expected results from rules, not the fixed implementation;
  record behavioral red/green assertions.
- **Differential:** retain each engine's pins/results, aligned inputs and first
  divergent checkpoint; record rules-based adjudication and independent review.
  Retain any original/adapted ancestry. Neither majority vote nor rewriting
  expectations from engine output is adjudication. Preserve known upstream defects
  and vintage exceptions as disputed evidence, never passing agreement.

## Admission and rules-vintage checklist

1. Check pool/capability applicability and distinguish harness examples from
   accepted scenarios. Audit every contributing file and data field, not just
   the repository license. Complete origins, notices and distribution review.
2. Pin CR `cr-2026-09-25` and the exact card/Oracle revision. Record unknown
   upstream vintage as a review gap until semantics are independently reconciled.
3. Audit combat especially: the 2024 Foundations update removed combat damage
   assignment order. Do not preserve old “order blockers, then respond” choices.
   Check multiple blockers, trample/deathtouch and actual current numbered rules.
   Audit all other applicable rule/card changes; a modern commit alone proves none.
4. Translate hidden harness defaults (deck size, opening hands, mulligans, phases,
   priority, automatic passes and AI). Explicitly script every consequential
   choice and ordered libraries. Keep synthetic-state assumptions separate from
   evidence of reachable full games.
5. Independently justify intermediate checkpoints, invalid actions and expected
   observations. Record adapted-source dependence and genuine holdout status.
6. Review notices and output payloads; link review and evidence before admission.
   Missing terms, provenance, rules adjudication or required reference access
   blocks the affected fixture/capability, not an invitation to skip the check.
7. On rules/card/source upgrades, preserve old fixtures/receipts, produce an
   explicit migration diff, re-audit semantics and notices, and rerun required
   references. Missing/failed engines remain failures. GH-13/GH-14 own reference
   build/bridge proof; GH-15 owns initial corpus; GH-38 retains mandatory dual
   reference coverage. This document completes none of those execution obligations.

## Tracked release blockers

These open items are tracked here under GH-11 and the existing
[release gate GH-40](https://github.com/pabloxrl/mtg-lab/issues/40); they do not add
program tasks or authorize later milestones. GH-16 checks that this conservative
M0 decision exists, not that public release has been cleared.

| ID | Status / exact obstacle | Affected boundary and resolution evidence |
| --- | --- | --- |
| `REL-01` | OPEN: repository license unselected; no owner instruction selecting one. | Public release blocked. Owner must select it explicitly; record reviewed license/notice inventory for exact release contents. No inferred choice in package metadata or this document. |
| `REL-02` | OPEN: Wizards fan-policy/terms applicability to a standalone simulation/research engine and redistribution of Oracle/rules/ruling-derived data is unresolved. | Public release and inclusion of such external payloads blocked. Owner-led rights review must identify exact allowed uses/fields, terms or permission, required notices, and exclusions. Separate fetches are a conservative boundary, not legal clearance. |
| `REL-03` | OPEN: no release-specific Scryfall field/use review or upstream dependency/bridge redistribution inventory exists. | External data/reference payload distribution blocked. GH-10 reviews selected metadata fields; GH-13/GH-14 review actual bridge pins/patches; GH-40 verifies exact release artifacts and resolves remaining data/notice questions. GPL version/options must be determined per incorporated artifact if policy is revisited. |

For any newly unavailable source/engine or ambiguous artifact, add an exact blocker
record (ID, affected fixture/capability, source/version, attempted command and
error, evidence, responsible existing issue, next required evidence). Mark the
artifact blocked, and update its issue and the parent program workpad. If it
prevents the assigned task's acceptance, follow WORKFLOW's failed-task handoff and
blocking process. A tracked release question does not make unexecuted reference
checks pass. Close a blocker only with a linked decision/review and exact affected
artifact revision, preserving its history.
