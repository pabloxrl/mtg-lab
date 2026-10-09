# Getting started

## Quickstart: run the checks

You can verify the current repository today. You do not need a GitHub token,
Codex login or a running Symphony worker for these checks.

Prerequisites: Git, Python 3, Docker Engine and Docker Compose v2, with Docker
running. Java, Rust and Maven run inside Docker. On macOS, use your existing
Docker provider; the project's current development setup uses Colima. Allow
roughly 8 GiB of VM memory for the managed setup and enough disk for build caches.
The first build needs network access and takes longer than later runs.

```sh
git clone https://github.com/pabloxrl/mtg-lab.git
cd mtg-lab
./scripts/verify-docker.sh
```

This builds the toolchain, checks documentation and the implementation/test plans,
runs every discovered Python test and Rust tests in debug and release, and exercises an isolated credential-free Symphony
runtime/sandbox smoke. Source is mounted read-only; build output stays in the
container. On AppArmor-enabled hosts, the runtime smoke's host-preparation helper
may need sudo to install the project's named profile; see the
[Docker runbook](symphony-runbook.md#docker-runtime).
It does not dispatch work or use the live worker's credentials/workspace volume.
The isolated smoke container and its temporary volume are removed afterward.

A successful run reports passing documentation/program/design checks, tests,
`nested-sandbox-ok` and `credential-free-controller-ok`. Test counts will grow
with the implementation; a green check proves only the capabilities actually
covered by those tests.

### Try a small working example

After the quickstart has built `mtg-lab-toolchain:local`, run the synthetic
checkpoint comparator:

```sh
docker run --rm --cap-drop ALL --security-opt no-new-privileges \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" \
  --workdir /workspace -e CARGO_TARGET_DIR=/tmp/target \
  mtg-lab-toolchain:local \
  cargo run --quiet --locked -p mtg-fixture -- fixtures/comparator/equal.json
```

It returns JSON with `"status":"pass"`. Replace `equal.json` with `life.json` to
see an intentional mismatch at `life.p1` and exit code 1. Both checkpoints are
supplied by the fixture: this demonstrates comparison, not a simulated game.
See the [comparator contract](fixture-comparator.md) for its schema and errors.

To run two passive games unattended, use the same toolchain image:

```sh
docker run --rm --cap-drop ALL --security-opt no-new-privileges \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" \
  --workdir /workspace -e CARGO_TARGET_DIR=/tmp/target \
  mtg-lab-toolchain:local \
  cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/pass-v1.json
```

This emits a run header, two completed episode summaries and aggregate counts as
JSONL. Both seats explicitly use `pass-v1`: keep seven, pass priority, discard at
cleanup. It plays no cards; the nonstarting seat loses to an empty draw on turn 68.
This is an automation/accounting baseline, not a measure of policy strength or
complete card support. [Configuration, exit codes and limits](simulate.md).

Run the explicit land/Cub/Growth/combat script (101 decisions, then P1 concedes):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/script-v3.json </dev/null
```

The script is privileged input, includes every pass/payment/target/combat choice,
and has no policy fallback. It ends with life `[20,15]`, consumes exactly 102
records, and reports script consumption separately from game outcome. Payloads
are redacted from output; resolved game configuration and seeds remain privileged
run metadata. Limits stop the script explicitly; unused records never imply success.

Run two bounded native games (green mirror, heuristic versus legal-random):

```sh
env -u DISPLAY -u WAYLAND_DISPLAY cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/native-v2.json </dev/null
```

This uses `heuristic-surprise-v1` and `legal-random-surprise-v1`; prior M1 policy IDs
reject explicitly. It uses the owned Driver and the delivered policy subset (M1 plus Dragon Fodder/Goblin tokens, Elf/Druid mana creatures, Magnigoth Sentry, Axgard Cavalry, Tajuru Pathwarden, Thornweald Archer, Shivan Dragon, Wildheart Invoker, Thrill of Possibility and Goblin Surprise). Archer/Cyclops/Pyromancer casts, trigger ordering and player targets require explicit scripts until the full-pool policy delivery; these policies reject those choices. Inspect
completed/truncated/failed/incomplete counts; explicit work exhaustion leaves an
incomplete episode. Capture defaults to disabled; schemas 2/3 can opt into
[bounded canonical publication](simulate.md#canonical-capture-schemas-2-and-3)
with existing disjoint roots and explicit local-owner authorization. The
[composed CLI audit](evidence/composed-cli/README.md) checks simulation through
dataset validation and replay verification, exact reload equality and failure
accounting. The [final CLI integration audit and initial baseline](evidence/unattended-integration/README.md) cover the command matrix; the [M1 gate audit](evidence/m1-gate/README.md) records scoped acceptance.

Inside the same toolchain container, try a bounded scalar command smoke:

```sh
cargo run --quiet --locked -p mtg-cli -- bench --workload scalar-pass-v1 --config fixtures/simulate/pass-v1.json
cargo run --quiet --locked --release -p mtg-cli -- bench --workload native-rollout-v1 --config fixtures/simulate/native-v2.json
cargo run --quiet --locked -p mtg-cli -- trajectories validate crates/mtg-recorder/tests/episode.jsonl
cargo run --quiet --locked -p mtg-cli -- trajectories validate crates/mtg-recorder/tests/episode.jsonl --manifest crates/mtg-recorder/tests/manifest.json
```

The benchmark emits raw elapsed time and separate completed/truncated/failed counts;
native rollout also reports policy time, accepted decisions and work calls. Neither
is performance qualification. The trajectory commands validate the handwritten
synthetic recorder fixture (one episode, one decision), not a captured full game.
See the [headless command contract](headless-commands.md) for opening/played replay verification, opening seat
inspection, strict checkpoint comparison, dependency/input limits and exit codes.

To inspect the completed test-design mapping without executing games:

```sh
docker run --rm --cap-drop ALL --security-opt no-new-privileges \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" \
  --workdir /workspace -e PYTHONDONTWRITEBYTECODE=1 \
  mtg-lab-toolchain:local python3 scripts/test_plan.py
```

At M0 this reports 80 capabilities, 320 designed slots, six related authored
fixtures and zero execution evidence added. Real reference acceptance requires
separately prepared pinned source/dependency caches; follow the
[XMage](../references/xmage/README.md) and [Forge](../references/forge/README.md)
instructions inside Docker. The quickstart does not run those heavyweight bridges.
