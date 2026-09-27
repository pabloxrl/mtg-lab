# Pinned XMage headless smoke bridge

This test-only bridge runs the real XMage `CardTestPlayerBase` harness. The
original neutral fixture describes a synthetic upkeep position: empty hands,
battlefields and stack, with two distinct basic lands in each top-first ordered
library. P0 explicitly passes; execution stops at P1's priority decision.
CR 117.3d independently determines the expected handoff and unchanged life,
zones and mana. No production engine computes expectations or participates.
This is one intermediate smoke case, not normal-reset reachability or full-game
verification. Later every-card and dual-reference obligations remain with
GH-24/GH-38; Forge smoke remains GH-14.

## Reproduction and pins

Use the managed **Linux ARM64** image with Temurin **21.0.9+10** and Maven
**3.9.11**. `pins.json` records the toolchain base-image digest, upstream commit,
archive checksum, card revision, bridge version and empty upstream patch list.
The runner checks installed Java/Maven versions, archive and original source
bytes, and the complete Maven JAR/POM/native compiler checksum inventory in
`dependencies.json`. The lock includes an ARM64 protoc binary; AMD64 is not
certified by this smoke. Missing tools require a coordinator image update.

```sh
python3 scripts/xmage.py prepare --cache /tmp/mtg-xmage
python3 scripts/xmage.py build --cache /tmp/mtg-xmage
python3 scripts/xmage.py acceptance --cache /tmp/mtg-xmage
# For subsequent repeated comparisons without mutation runs:
python3 scripts/xmage.py run --cache /tmp/mtg-xmage
```

The cache must be outside this repository. The delivery run reused
`/home/agent/.cache/xmage`. `prepare` downloads the checksum-pinned upstream
archive and extracts it externally; it restores original source on repetition.
No host packages or secondary toolchain are installed. Build/cache, Maven
settings, dependencies, generated databases and logs remain external to the
Rust workspace and production simulation. No upstream source patches are
applied: the one original Java bridge is added to the external test tree.
The committed bridge source is version 1; receipts also hash its exact bytes.

`build` resolves the upstream pinned POM dependencies and executes only
`NeutralSmokeTest`, then requires the resolved artifact inventory to match the
committed lock. A mismatched build is failure, not a new lock to accept.
`run` checks the lock before execution, invokes offline Maven twice, compares
against independent fixture assertions and requires identical checkpoints.
`acceptance` additionally executes three changed fixtures through real XMage:
starting life 19 with expected life 20, a P1 pass while P0 has priority, and an
expected priority of P0 after P0 passes. Each must fail for its specific intended
reason; unavailable builds, timeouts and unrelated failures cannot kill a mutant.

All Maven executions close stdin, remove `DISPLAY` and `WAYLAND_DISPLAY`, and
set Java headless mode. Each smoke/mutant has a 180-second timeout; build has
1,200 seconds, downloads 180 seconds and extraction 300 seconds. Timeout kills
the process group. Success exits 0; execution/validation failures emit JSON and
exit 2. `acceptance.json`, `smoke-{1,2}.json`, mutation fixtures, checkpoints and
logs remain in the cache. The receipt identifies fixture, bridge, runner, pins,
dependency inventory and logs by SHA-256. A failed run removes its old receipt.
`evidence.json` and `checkpoints.json` retain the reviewed delivery observation;
these observations are never used as expected results by the runner.

## Scope and observability

The controller bypasses `TestPlayer.priority`'s automatic-pass fallback. It
requires the explicit action and pass choice, checks actor/kind/values, then
calls `pass`. No AI priority method is called. Upstream strict choice mode is
enabled; unexpected mulligans fail. Empty synthetic hands skip mulligan choice.
The starting seat is injected, shuffling disabled, and ordered library cards
inserted in reverse top-first order. Actual XMage UUIDs map to fixture IDs.
The bridge selects FDN Forest/Mountain definitions from the pinned `Mage.Sets`
source. This tests order and identity preservation, not card text semantics.

| Canonical field | Smoke observation |
| --- | --- |
| Turn, active player, priority, phase/step | Read from XMage at initial and next decision; lowercase phase/step |
| Player life, six-color mana | Read from XMage at both checkpoints |
| Library, hand, graveyard | Actual ordered object IDs mapped to fixture IDs |
| Battlefield, exile, stack | Assert empty in XMage before exporting empty arrays |
| Object characteristics/status/damage, land plays | Explicitly unsupported in output |
| Legal-choice enumeration, effects, private views, outcome | Explicitly unobservable/unsupported in output |

The comparator requires all unsupported declarations. It compares all exported
initial fields to the fixture, then checks that passing changes only priority,
and checks every named fixture assertion. Library order, life, mana, stack and
priority mutations are also covered by offline tests. Broader initial states,
actions and checkpoints are explicitly rejected; no winner-only or full-state
verification claim is made. These are privileged test checkpoints, not policy
observations.

## Independent expectations and validation

CR 117.3d supplies the expectation; no output is generated by mtg-core. Two
behavioral regressions were demonstrated before fixes: an unpinned JDK and
missing observability declarations were accepted (two assertion failures).
After fixes all nine focused tests pass, including life/priority/library,
missing checkpoint, mana/stack mutation, closed stdin/unset display, nonzero
subprocess exit and timeout detection. `./scripts/verify.sh` also runs these
fast checks. It does not claim to run the heavyweight reference build; the
mandatory issue acceptance command above does.

The delivery report and independent review are linked from the
[GH-13 workpad](https://github.com/pabloxrl/mtg-lab/issues/13#issuecomment-5850213027).
Planned/executed/agreed smoke scenarios: 1/1/1; disputed 0; the three seeded
mutations are detected negative controls, not three additional agreed cases.
The production capability registry remains planned; this does not certify its
320 evidence slots or broad conformance corpus admission.

## Provenance and notices

Fixture and bridge are original project material; repository license remains
unselected. No upstream scenario was copied or translated. `consulted_sources`
in `pins.json` records exact API files and hashes. Preserve XMage's full MIT
notice in `UPSTREAM-LICENSE.txt` and the external source installation.
Consulted file author notices: CardTestPlayerBase (`ayratn`), MageTestPlayerBase
(`ayratn, JayDi85`), TestPlayer (`BetaSteward_at_googlemail.com, Simown, JayDi85`),
TestComputerPlayer (`JayDi85`), Library (`BetaSteward_at_googlemail.com`). These
credits identify consulted APIs, not authorship of the original fixture.

The scoped vintage review concerns CR 117.3d only: no combat assignment ordering
or card ability resolution is imported. Permanent characteristics and complete
legal choices remain unsupported, so this is not evidence of their alignment.
The fixture retains independent rules/card source pins and a pending corpus
admission review; bridge delivery does not bypass GH-15 admission obligations.

JDK (GPLv2 with Classpath Exception), Maven (Apache-2.0), XMage (MIT) and Maven
transitive dependencies remain separate external installations, with their
original notices retained. `dependencies.json` identifies the exact external
artifact inventory, including build/test plugins. Dependency POMs and JAR
META-INF notices remain in that cache; they are not redistributed here. This
M0 distribution review admits metadata, original bridge/fixture code, semantic
checkpoints and the required XMage notice only. It does not grant permission
to redistribute the external dependency bundle or card database. MIT permission
for XMage code does not license Wizards content. No raw Oracle text, artwork,
upstream tree or runtime binary is committed. Release-wide REL-01/02/03 remain
open, including release-specific transitive notice review.

## Vanilla combat extension (GH-71)

The smoke scope above remains unchanged. A separate original
[VanillaCombatTest](VanillaCombatTest.java) now executes five matched synthetic
combat cases using the same pinned external engine and dependency inventory.
[Combat acceptance](../../doc/evidence/combat/README.md) documents shared inputs,
all observed fields, strict scripted declarations/allocations/passes, actual
checkpoints, limits and the bounded reproduction command. No upstream scenario
is copied, no source patch is applied, and no older blocker-order expectation
is imported: the 1+1 allocation explicitly exercises the current rule.
[API consultation hashes](combat-provenance.json) supplement the smoke provenance;
the same retained upstream MIT notice and distribution limitations apply.

## Terminal boundary extension (GH-72)

[TerminalTest](TerminalTest.java) executes seven original synthetic settled
terminal boundaries with shared native/reference inputs. [Acceptance and scope](../../doc/evidence/terminal/README.md)
records observed life/loss flags/hand/library counts and limitations, including
no winner-flag or full-game reference claim. [API provenance](terminal-provenance.json)
supplements the existing pin and MIT notice. No upstream test is copied.

## Opening count extension (GH-17)

[OpeningCountsTest](OpeningCountsTest.java) scripts eight original London
mulligan count cases, starting from explicitly synthetic seven-card hands.
[Core audit and reproduction](../../doc/evidence/core-integration/README.md)
records the shared native/reference ledger, exact scope and execution receipt.
Both starting seats and 0/1/2/7 mulligans are covered, including per-round
bottoming and forced keep. Identical basic-card decks make shuffle identity and
order unobservable; neither is claimed. [Consulted API hashes](opening-provenance.json)
are verified by the runner. The existing MIT notice and distribution limits apply.
