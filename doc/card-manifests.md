# Frozen Foundations manifests

GH-10 delivers metadata and validation for RFC 0002 `R0002-B007`, `B010` and
`B012`. It does not implement card effects, legal actions, games, reference
bridges or a sanctioned format. Shared later obligations remain with their
program owners. The custom pool is `foundations_micro_v1`, revision
`2026-09-26.1`.

## Use

From the repository root, with Python 3:

```sh
python3 scripts/card_manifest.py
python3 scripts/acquire_cards.py --cache /tmp/mtg-foundations-new-cache
python3 scripts/card_manifest.py --cache /tmp/mtg-foundations-new-cache
```

The first command is entirely offline and checks the committed
[manifest](../data/cards/foundations_micro_v1.json) against its adjacent
[raw-file SHA-256 pin](../data/cards/foundations_micro_v1.sha256), schema, record
hashes and exact RFC deck constraints. The second explicitly requires network
access and curl, acquires each selected Scryfall printing by UUID, verifies its
stable source projection and exact Oracle text, and writes external JSON files.
The third verifies that cache without HTTP. Missing files, mismatched content,
unsupported input, HTTP errors and timeouts fail with nonzero status; stdin is
never consulted. Offline metadata validation alone does not prove the text is
available or implement its meaning.

Acquisition requires a new directory outside the repository. A failed acquisition
can leave partial external files, but writes `verified.json` only after all 21
sources pass. Retry into a new directory; there is no silent fallback, retry,
substitution, update of pins or overwrite of an existing experiment cache.
Requests use accurate headers, at least 100 ms spacing, no redirects, a 30-second
curl deadline (35-second process deadline), and a 256 KiB response limit. There
is no network acquisition in the rules core or offline validator.

## Selected records and revisions

All 20 names are exactly those in the RFC table. FDN here includes supplemental
Beginner Box and Starter Collection entries. The token is TFDN 18, separate from
the 20 deck cards. Basic-land art variants do not change the selected printing:
Mountain 278 and Forest 280 are pinned. Shivan Dragon is deliberately FDN 763,
the RFC's example. Llanowar Elves is FDN 227. No fuzzy/name fallback is used.

Each card/token record contains:

- Project ID, exact name, card/token kind, Oracle UUID and selected Scryfall
  printing UUID, set, collector number, language, layout and release date.
- Mana cost/value, type line, colors, color identity, keywords and printed base
  power/toughness (null for noncreatures). These are frozen source characteristics,
  not calculated game state. Current Oracle type terminology is preserved:
  Viashino Pyromancer is a Lizard Wizard despite its name.
- A reserved per-card `behavior_id` and `reserved-not-implemented` status. These
  identify future typed implementations, not arbitrary scripts or parsed text.
- Retrieval date, exact source URL, historical raw response SHA-256, exact
  Oracle-text SHA-256, stable source-projection SHA-256 and metadata-content hash.

Scryfall does not expose an immutable Oracle revision number. This manifest
therefore pins the SHA-256 of the exact UTF-8 `oracle_text` string, including
newlines and reminder text, without normalization. Empty text is the SHA-256 of
zero bytes. This content revision is independent of `cr-2026-09-25` and the
printing's release date; none is represented as the date Oracle last changed.

Scryfall is the card-data acquisition source linked by the RFC; Wizards' Oracle
is the rules authority. These receipts demonstrate acquisition and consistency,
not a separate Wizards signature or an independent official Oracle audit.
The source projection comprises `id`, `oracle_id`, `name`, `set`,
`collector_number`, `lang`, `layout`, `released_at`, `oracle_text` and the
`characteristics` object with exactly the fields listed above. Absent source
power/toughness become null. Source projection and metadata-record hashes use
UTF-8 JSON with sorted keys, compact separators, unescaped Unicode and no
nonfinite numbers. Record hashes exclude only their own `content_sha256` field.

Raw Scryfall JSON also contains volatile prices/links. Its historical hash is
retained for provenance, but later acquisition compares the stable projection
and text hashes, not byte identity of irrelevant volatile fields. All admitted
identity/characteristic/text changes fail verification. The adjacent manifest
pin covers the complete file, including decks and matchups; it is an integrity
pin, not an authenticity signature. Keep its value in experiment metadata.

The [acquisition receipt](../data/cards/foundations_micro_v1.verification.json)
records a successful live fetch of all 21 immutable printing endpoints on
2026-09-26. It contains only reviewed metadata, not raw card text. Synthetic
transport tests are separate from this historical availability evidence.

## Decks and matchups

Red and green each contain exactly 40 cards, 16 basic lands and 24 spells, with
all multiplicities fixed to RFC 0002 §3. Deck-entry IDs must resolve to non-token
cards; duplicates and total-preserving multiplicity changes fail. Each deck has
its own content hash. There is no deck-construction API or balance claim.

Eight configurations enumerate red/green, green/red, red/red and green/green,
each with starting seat 0 and 1. `seats[0]` and `seats[1]` assign deck IDs to
stable player seats; `starting_seat` is separate. This is validated experiment
metadata, not proof that any matchup can yet be simulated.

## Field admission and distribution review

This artifact-specific M0 review applies the
[GH-11 provenance policy](provenance-policy.md), using its pinned primary
[Scryfall API/terms and Wizards policy receipts](provenance-sources.json).
Review date: 2026-09-26; author: GH-10 implementation session; independent review
and resolution are preserved in the GH-10 delivery PR. Admission is limited to
this small frozen research-fixture metadata set and original validation code.
It does not establish a general permission to redistribute Scryfall databases.

| Fields/material | M0 decision and purpose |
| --- | --- |
| Names, Oracle/printing identifiers, set/collector/language/layout/release dates | Admit this selected inventory to identify exact RFC fixtures; no artwork, branding assets or full database. |
| Mana/type/color/keyword/base-power/toughness characteristics | Admit these selected structured facts for offline consistency and future typed definitions; exclude free-form Oracle/ruling/flavor text. No game-semantics claim. |
| URLs, retrieval dates, hashes and byte counts | Admit as minimal provenance/integrity evidence; a hash does not grant permission or guarantee future availability. |
| Deck quantities, seat configurations, project/behavior IDs, schema/validation code and explanations | Original project material; repository license still unselected. |
| Raw API responses, Oracle text, ruling/release-note documents, artwork/images, symbols and reference-engine material | Exclude from tracked files, packages and CI artifacts. Fetch card responses separately into external caches. Image URLs in those external API responses are never followed. |

This tool adds specific validation and experiment reproducibility; it is not a
bulk proxy or republication of Scryfall. Scryfall's free API access is not an
unrestricted license, and their terms attribute Magic content to Wizards. This
project is unofficial, not endorsed by Wizards of the Coast or Scryfall. Magic:
The Gathering and associated card content belong to Wizards of the Coast.
No rights over that content or license over this repository are granted here.

REL-01 (repository license), REL-02 (Wizards applicability) and the release-wide
portion of REL-03 (actual payload/dependency review) remain OPEN under GH-40.
This completes GH-10's selected-field M0 review, not public release clearance.
Do not package the external cache or unchecked source/log payloads. Later
publication must review the exact artifact and required notices under the policy.

## Updates and tests

An upstream change requires a deliberate new pool revision, reviewed field/text
and deck diffs, new dated receipts/hashes and explicit experiment/corpus migration.
Preserve old pins and caches for saved experiments. Do not rewrite an old revision
or broaden accepted content just because the API changed. Future implemented
behavior status needs its own rules tests, capability mapping and mandatory
reference evidence; this task provides none of that game execution evidence.

`./scripts/verify.sh` automatically discovers the manifest and acquisition unit
tests. The exact deck expectation test reads the independent pinned RFC table.
Other tests reject unsupported/malformed metadata, duplicate fields and IDs,
wrong totals/multiplicities, missing tokens/decks/matchups, digest mismatches,
changed source identities/text/characteristics, unavailable/redirected sources,
partial acquisition and missing caches. Live retrieval remains an explicit
acceptance check, never a runtime or offline-CI dependency.
