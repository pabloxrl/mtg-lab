# Pinned headless Forge smoke (GH-14)

This is a test-only M0 reference bridge, using Forge's real
`BaseGameSimulationTest` and `GameWrapper`. It executes the **identical unchanged
neutral fixture** used by [XMage](../xmage/README.md):
[`xmage-priority-pass.json`](../../fixtures/scenarios/xmage-priority-pass.json).
The historical fixture name is not an engine dependency. CR 117.3d independently
requires one pass to transfer priority to the other player without changing life,
libraries, mana or step. No production mtg-lab rules implementation is involved.

## Reproduce

Use the managed Linux ARM64 image's Temurin 21.0.9+10 and Maven 3.9.11:

```sh
python3 scripts/forge.py prepare --cache /tmp/mtg-forge
python3 scripts/forge.py build --cache /tmp/mtg-forge
python3 scripts/forge.py acceptance --cache /tmp/mtg-forge
```

`prepare` fetches the checksum-pinned source into a separate cache; `build`
executes the smoke and checks the dependency inventory. `acceptance` executes two
offline positive runs, compares their checkpoints to each other, the independent
fixture and GH-13's recorded XMage checkpoints, then runs three negative controls.
`run` performs the two positive comparisons only. Do not call it full acceptance.
Every failure returns a nonzero status with the log path. A stale acceptance
receipt is removed before build/run/acceptance. An unavailable build is a failure.

Commands close stdin, unset DISPLAY/WAYLAND_DISPLAY, set a headless JVM and use an
isolated Java user directory below the cache. Maven stdin is `/dev/null`;
Surefire uses the fork's OS stdin for its control protocol, so the bridge installs
an EOF-only `System.in` before harness initialization. The Java test checks EOF,
absent display variables and headless mode. Source download is bounded to 180 seconds,
extraction to 300, build to 1,200, and each offline smoke to 180. Timeouts kill the
process group. The exact Maven command is in [evidence.json](evidence.json).
`--cache` must be outside the checkout and have no spaces. The pinned source files
are verified byte-for-byte against the archive before execution; custom user data
is rejected. Source/build/runtime artifacts remain in that external cache.

[pins.json](pins.json) separates Forge source, card-data revision, JDK/runtime,
Maven, base-image digest, bridge version and upstream patches (none).
[dependencies.json](dependencies.json) pins 787 Maven artifacts: 543 POMs and 244
JARs, including plugins and test dependencies. Repeated runs use Maven offline and
require an exact inventory match. This is a Linux ARM64 receipt, not cross-platform
qualification or a bit-for-bit reproducible binary claim (upstream build metadata
includes timestamps). The source-only original test is added to the external test
tree; no upstream file is patched. Upgrades require an explicit pin/inventory and
vintage review plus new actual execution evidence.

## Execution and comparison boundary

`GameWrapper` constructs a synthetic position with empty hands/battlefields and
the fixture's top-first libraries. It skips shuffling/opening hands/mulligans,
starts the first turn normally and reaches upkeep. No independent RNG matching
is assumed. Each seat has two distinct basic lands. The bridge maps actual card
IDs by owner and card identity, not library position; duplicate identities are
explicitly unsupported. Ordered zone lists therefore detect reversed libraries.
Card names actually read from Forge are exported separately and checked.

Forge's `PlayerControllerForTests` is replaced through construction mocking with
an exhaustive, fail-closed controller: metadata access and `setPlayerActions`
bookkeeping are allowed; the single `chooseSpellAbilityToPlay` call consumes the
fixture's actor/kind/source/choice/value tuple. Only then does it return Forge's
explicit pass result. Forge's `PhaseHandler` performs the priority transfer; the
bridge does not set the resulting priority. The next decision captures the second
checkpoint and throws a private stop signal, caught only around the harness run.
No second pass, concession, terminal result or winner is invented. All other
controller calls fail rather than invoking inherited AI or default choices.

The base harness provides its real card loader and test mocks; the bridge replaces
its GUI interface with a strict stub that allows only asset-path/desktop-mode
metadata. Any GUI operation fails. Upstream APIs, mocks and consulted files are
pinned separately; game rules and the priority loop are not mocked.

[checkpoints.json](checkpoints.json) contains actual exported initial and
`priority-p1` states. Both are compared with CR 117.3d expectations; their semantic
content matches [XMage's executed checkpoints](../xmage/checkpoints.json).
Object IDs and upkeep timing are normalized. Comparison covers turn, active
player, priority holder, phase/step, both life totals, six mana counts, top-first
libraries, hand/battlefield/graveyard/exile zones, and the empty stack. Every
unexported field is declared: permanent characteristics/status/marked damage,
land plays used, legal-choice enumeration, effects/private views and outcome.
These are privileged synthetic test snapshots, not player observations or
full-state/full-game verification.

## Evidence and negative controls

The [acceptance receipt](evidence.json) binds the fixture, source/bridge/runner
pins, shared comparator, dependency inventory, observed checkpoints and external
logs by SHA-256. No raw external logs/card text are redistributed. Surefire must
report exactly one executed, passing, unskipped `neutralSmoke` test; missing
checkpoints or configuration-only/zero-test output cannot pass.

Three real Forge negative controls must fail for the specific intended reason:

- Initial life 19 with unchanged independently specified checkpoint life 20:
  `/players/0/life` mismatch.
- Wrong actor for the scripted pass: `script actor mismatch` from the strict
  controller, not a build error or timeout.
- Wrong expected priority: `/priority` mismatch.

Mutant fixture and log hashes are in the receipt; minimized one-field changes,
actual outputs and logs remain in the external cache. These negative controls
are not rules disagreements. No unmodified-scenario disagreement was found;
no rule exception or changed expectation was needed. Planned/executed/agreed
smoke scenarios: **1/1/1**; disputed 0. Three detected mutants are not three more
agreed scenarios. An initial local configuration failure (Forge tried using the
agent user directory) was repaired with an isolated Java user directory and
explicit asset path; that failure was not counted as verification.

Before implementation, two assertion failures demonstrated that reusing the
existing comparator alone admitted missing/changed card-identity evidence. The
Forge wrapper now rejects both. Fast tests also cover independent pass
expectations, life/priority/library/mana/stack mutations at both checkpoints,
missing observability/checkpoints, unpinned tools, skipped/wrong test reports,
and CLI failure with closed stdin/no display. `./scripts/verify.sh` runs these
checks; heavyweight Forge acceptance is a separate mandatory issue check.
The [GH-14 workpad](https://github.com/pabloxrl/mtg-lab/issues/14#issuecomment-5850667068)
records independent review and delivery CI.

## Scoped provenance and distribution review

Reviewed 2026-09-26 for GH-14 under the
[M0 distribution policy](../../doc/provenance-policy.md). The fixture is original
GH-13 project material, not translated from a Forge scenario. This original
source-only bridge consults the pinned API/harness files listed in `pins.json`;
no GPL test implementation or card script is copied into the corpus. The main
corpus and registry are unchanged. Forge source, GPL tests, card scripts, compiled
bridge, runtime and dependencies stay separately installed; nothing from their
build trees is included in this repository or CI artifacts.

Forge's pinned `LICENSE` is GNU GPL version 3 (its checksum is recorded); the
consulted BaseGameSimulationTest/GameWrapper/controller files have no separate
license header. Do not infer an "or later" option from the license appendix.
The external source retains all original license/notices. New adapter source is
original project material with repository license still unselected. This scoped
M0 review admits the original source-only adapter, hashes and minimal semantic
receipts, **not redistribution of the linked Forge/bridge runtime**. Importing
GPL interfaces and running a separate process do not settle obligations for that
combined artifact; release-specific bridge licensing remains REL-03.

Dependency POM inspection includes Apache-2.0 (Maven and many dependencies), MIT,
BSD, EPL, LGPL, CDDL/GPL-with-classpath-exception and MPL families. In particular,
Checkstyle's POM declares LGPL-2.1-or-later and soundlibs' parent declares LGPL 2.1;
these are not silently treated as Apache/MIT. POMs, JAR META-INF notices and exact
binaries remain external, checksum-pinned; this is not a release-wide transitive
license clearance. Temurin is GPLv2 with Classpath Exception. No permission over
Wizards content is inferred from engine licensing. Raw rules/Oracle text, artwork,
upstream trees, dependency bundles and unchecked logs are not redistributed.
REL-01/02/03 remain open for the release gate.

The vintage review is limited to CR 117.3d. Basic-land identifiers and ordered
library objects suffice here; no card ability, combat ordering, full rules-version
alignment or broad capability is certified. Fixture admission and corpus review
remain GH-15; full card/critical dual-reference/full-game obligations remain with
the later assigned tasks. This smoke does not complete M0 or certify the planned
capability registry.
