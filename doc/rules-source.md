# Pinned Comprehensive Rules source

RFC 0002 §3 and requirement `R0002-B011` select **2026-09-25**. The
[versioned manifest](../data/rules/cr-2026-09-25.json) stores the exact official
URL, effective date, UTC retrieval date and SHA-256 of the original response
bytes. It pins the rules source only; it does not implement any game behavior.
Card definitions and Oracle text will have separate versions.

From the repository root, using Python 3:

```sh
python3 scripts/rules_source.py validate
python3 scripts/rules_source.py fetch-verify
```

`validate` is offline and checks the strict manifest schema and selected RFC
revision. `fetch-verify` explicitly accesses the official source using `curl`,
checks the raw-byte SHA-256 and publication title/effective-date header, then
prints a JSON receipt. Exit 0 means that command succeeded; exit 1 reports a
validation, source, or integrity failure on stderr; invalid CLI arguments exit 2.
Neither command modifies the pin. `--manifest PATH` selects a metadata file,
but cannot select an alternative revision or source under the current policy.

Fetching disables curl configuration, accepts only HTTPS and HTTP 200 without
redirects, limits connection time to 10 seconds and the entire transfer to 30
seconds, with an outer 35-second process deadline and a 4 MiB size limit. It
requires no stdin/display. Temporary downloaded bytes are removed on success
or failure. Install curl if absent; retry connectivity failures against the same
URL. Investigate date or checksum mismatches before changing anything. A
redirect, unavailable URL or different revision is a failure, never a fallback.

## Evidence and offline checks

The [live verification receipt](../data/rules/cr-2026-09-25.verification.json)
records a successful actual fetch, independently matching the initial curl
retrieval and `shasum -a 256` observation (977752 bytes). The first title and
effective-date lines were inspected directly. This is historical retrieval
evidence, not a promise of future availability or an independent publisher
signature. HTTPS establishes the source; the committed digest detects changes.
No full copyrighted rules document is committed or redistributed by this tool.

`./scripts/verify.sh` discovers `tests/test_rules_source.py` automatically.
Normal validation uses only committed metadata and synthetic payloads with
mocked transport: malformed/missing/duplicate fields, wrong digest, wrong date
even with a matching digest, wrong header, HTTP errors/redirects, timeout,
missing curl, size limits, temporary-file cleanup and unattended CLI behavior.
Synthetic successes are not evidence of official-source availability.

Behavioral baseline: the initial permissive API ran five test methods and
failed 40 assertions/subcases (missing rejection, not import/build errors).
The implementation then passed those checks; transport and CLI checks extend
the same requirements. No rules-engine/reference execution is claimed.

## Deliberate upgrades and corpus migration

Never edit a saved experiment's rules revision or resolve a pin to “latest.”
A future revision requires a reviewed change to the selected RFC baseline,
a new versioned manifest, a fresh successful source receipt, and an explicit
update to this tool's accepted revision/URL/header. Retain old manifests and
receipts; do not overwrite their digests to accommodate changed source bytes.
Unexpected changes at an existing URL require investigation and recorded
adjudication, even when its header date remains unchanged.

Before claiming the new revision supported, inventory impacted numbered rules,
audit original/imported scenarios (including the Foundations 2024 removal of
combat damage assignment order), re-justify changed expectations independently,
and execute affected conformance and reference checks. Version the migrated
corpus, record old/new rule mappings and incompatibilities, and explicitly
migrate or reject older experiments/replays. Do not regenerate expected outcomes
from engine output. These corpus and rules-behavior obligations remain with the
later assigned program tasks; this metadata delivery does not satisfy them.
