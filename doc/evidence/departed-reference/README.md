# Departed targets in shared instant scripts (GH-137)

Four new original scripts extend the [shared executor](../instant-reference/README.md).
Each was executed twice in native Rust and pinned XMage, from the same synthetic
turn-one upkeep boundary with the exact same consumed choices. No production
rules code changes. This is partial R0002-B017/B026/B029 evidence; #115/#18,
M1 qualification, combat/cleanup, Forge, full-pool and dual-reference/full-game
release gates remain outstanding.

## Independently authored acceptance

The [input corpus](../../../fixtures/reference/instant-responses.json) and
[literal expected checkpoints](../../../fixtures/reference/instant-expectations.json)
were extended before the adapters. Expected states were authored from the pinned
[CR manifest](../../../data/rules/cr-2026-09-25.json) and
[Oracle manifest](../../../data/cards/foundations_micro_v1.json), without engine
output or upstream test expectations. Bear Cub is 2/2; Bite Down makes its legal
source deal its power in damage to the legal opposing target; Growth gives +3/+3.

CR 117.3c/117.4 and 601 require the explicit actor, targets, mana/payment and
passes. CR 704.5g puts a Cub dealt two damage into its owner's graveyard.
CR 400.7 makes that graveyard card a new object. CR 608.2b rechecks targets:
all illegal means the spell does not resolve; one legal Bite target means it
resolves, but an illegal target cannot perform actions or have actions performed
on it. In particular an illegal Bite source deals **no damage**, even if its old
power is available as last-known information.

| Original case | Responding spell sequence | Independently required lower-spell outcome |
| --- | --- | --- |
| `growth-gone` | P0 Growth targets its Cub; P1 Bite kills that Cub | Zero legal targets, does not resolve; no replacement creature gets +3/+3 |
| `bite-source-gone` | P0 Bite; P1 Bite kills P0's selected source | One legal target, resolves without damage; no source-LKI damage |
| `bite-destination-gone` | P0 Bite; P0 retains priority and casts another Bite killing the destination | One legal target, resolves without damage; no substitute destination |
| `bite-both-gone` | P0 Bite; P1 Bite kills its source; P0's second Bite from another Cub kills its destination | Zero legal targets, does not resolve |

Both seats have a distinct same-name Cub in these setups. Their unchanged stats
and distinct identities are compared, as are the old targets still named by the
lower spell and the new same-name graveyard objects. The two pre-existing
Growth-response cases retain their original outcomes and now also assert
incarnations and resolution outcomes.

Setup explicitly declares every object, owner and zone, empty libraries, 20 life,
zero mana, untapped/undamaged permanents, active player and priority. Hands are
installed at XMage's first upkeep callback, as in #136. No shuffle, draw or other
chance occurs; native RNG version/seed is the existing zero-seed synthetic setup.
Scripts include every mana activation, target, payment, cast completion and pass.
No AI/default-choice fallback or caller-supplied expected state is accepted.

## Identity and observation contract

Object `id` identifies card lineage; `incarnation` counts zone changes since the
shared initial boundary. A stack target is `{id, incarnation}` and retains the
incarnation chosen at casting, even after that object no longer exists. The
native observer binds names to actual creation identities, retaining historical
handle mappings; it never looks up a stale target by card name/owner. XMage binds
names to UUIDs and normalizes its zone-change counters at the initial boundary;
target incarnations are captured from actual selected targets at cast completion.
Unknown/missing mappings fail explicitly.

Settled checkpoints compare complete named inventory/zones, empty libraries and
exile, active player/priority, life/mana, taps, creature power/toughness/damage,
bottom-to-top stack and historical targets, damage events and last resolution.
Native resolution status comes from `last_resolution`; XMage status comes from
the actual resolving Spell object (`isResolving`, `isCountered`, and retained
legal targets after its own revalidation). XMage internally calls an all-invalid
spell "countered"; the shared output correctly calls it `resolved: false`,
without claiming a modern rules counter event. Outcome is not inferred just from
both spells being in the graveyard. Damage is observed from actual events/changes,
not recomputed from source power.

The normal native identity regression separately moves a card out and back to
the battlefield and checks old, graveyard, returned and same-name-decoy identities.
That is an **observer-only synthetic test**, not a claimed matched reanimation
spell. The matched scripts exercise new graveyard objects and duplicate-name
permanents; no real battlefield reentry spell is in this bounded card slice.
Legal-action enumeration, hidden views, full continuous effects, zone ordering,
combat, cleanup and game completion are not compared.

## Regression and negative evidence

- [departure-red.log](departure-red.log) records the compiling baseline rejecting
  the duplicate-name setup. [stale-target-red.log](stale-target-red.log) separately
  reproduces `InvalidHandle` with the [minimized real-spell input](stale-target-minimized.json)
  against main `004eaeaf8c4d77aa5985b2475637e11b08b875c4`'s adapter. The fixed observer
  consumes it without substituting another object.
- Normal Rust discovery checks all literal cases, exact consumption, every new
  choice omission/duplication, inventory completeness, historical mapping loss
  and synthetic reentry identity. Python discovery mutates old-target IDs,
  incarnations, missing identity, illegal-source damage, resolution outcomes and
  unconsumed choices, requiring a first-divergence error at the appropriate case.
- Real-engine controls retain #136's omitted/extra target, wrong actor, payment
  order and legal changed target checks. A new missing post-departure pass must
  fail with `unfinished stack` in both engines.
- A legal removal targeting the other same-name Cub really executes in both
  engines. Both must first diverge at `growth-gone`'s `response-cast` target ID.
  The full mutated input, minimized one-case script (unused setup objects removed),
  independently derived reduced expectations, and first-divergence artifacts are
  retained. The reduced script is re-executed in both engines and must preserve
  the same first differing target. A build/unavailable-reference/unrelated failure never passes.

The old five checkpoint-name assertion was specific to #136's two scripts.
The general executor now consumes script-declared checkpoints; normal tests and
the corpus-bound comparator require every exact literal checkpoint, order and
consumed choice, including the seven-checkpoint both-departed case. Independent
candidate review must check that this is equivalent or stronger coverage, along
with README accuracy. No regression test was removed or skipped.

## Reproduction and provenance

Inside the managed Linux ARM64 Docker worker, use the image's pinned Rust/Java/
Maven/Python tools and writable external cache (no Docker socket or host install):

```sh
cp -a /home/agent/.cache/xmage /tmp/mtg-xmage
python3 scripts/instant_reference.py --cache /tmp/mtg-xmage --output /tmp/departed-acceptance
cargo test -p mtg-core --lib instant_
python3 -m unittest discover -s tests -p test_instant_reference.py
./scripts/torture.sh
```

To execute the minimized baseline reproduction with the fixed native observer:

```sh
MTG_INSTANT_INPUT="$PWD/doc/evidence/departed-reference/stale-target-minimized.json" \
MTG_INSTANT_OUTPUT=/tmp/departed-minimized.json \
cargo test -p mtg-core --locked --lib game::instant_reference_tests::instant_export_observations -- --exact
```

The full runner verifies pinned XMage source/archive, consulted source files,
Java/Maven versions and dependency hashes, with offline execution, closed stdin,
unset displays and bounded subprocess timeouts. The bridge, scenarios and
expectations are original project-authored material. XMage APIs were consulted;
its [MIT notice](../../../references/xmage/UPSTREAM-LICENSE.txt) remains retained.
No upstream scenario expectations, Forge code or new rules/card text were copied.

[acceptance.json](acceptance.json) hashes input/expectations, bridges, all native
sources, Rust lock, rules/card/source/toolchain/dependency pins and execution
artifacts. Repeated observations and negative controls are retained here; raw
build logs remain in the external receipt directory with recorded hashes.
The additional [metadata-red.log](metadata-red.log) records the test-first check
that identity binding retains actual card/owner validation; its regression also
rejects an unsupported controller change.
Full torture, candidate review, protected merge and successful exact-main CI are
recorded in the [issue workpad](https://github.com/pabloxrl/mtg-lab/issues/137).
The root README's reference coverage link is updated; the milestone table remains
unchanged. Completion is conditional on those delivery gates.
