# mtg-lab

A headless Magic: The Gathering research engine for reproducible games,
AI-player experiments and trustworthy game data. Built in Rust, with a
command-line interface for simulation, replay and validation.

The project is under active development. The M1 six-card scalar slice has passed
its acceptance audit, and additional card mechanics are being delivered toward
the frozen 20-card MVP pool. Full-pool native policies, Python bindings, batching
and RL training integrations remain planned. See [supported functionality and
limitations](doc/capabilities.md) for precise coverage and verification evidence.

## Features

- Deterministic game execution with explicit seeds, budgets and player decisions.
- Headless simulation with native random and heuristic policies for the supported
  subset, plus explicit scripted play.
- Seat-filtered observations and validated choices that preserve private information.
- Versioned snapshots, semantic replay, JSONL trajectories and dataset validation.
- Automated regression tests and scoped comparisons against pinned XMage and
  Forge reference engines.

## Getting started

Install Git, Python 3, Docker Engine and Docker Compose v2, and start Docker.
Rust, Java and Maven run inside the toolchain container. Allow roughly 8 GiB of
Docker VM memory; the first build requires network access.

```sh
git clone https://github.com/pabloxrl/mtg-lab.git
cd mtg-lab
./scripts/verify-docker.sh
```

This builds the toolchain and runs documentation checks, Python and Rust tests,
and an isolated controller smoke check. No GitHub token, Codex login or running
agent service is needed. Some Linux hosts require sudo to prepare the sandbox;
see the [setup guide](doc/getting-started.md).

After verification builds the toolchain image, run two passive games:

```sh
docker run --rm --cap-drop ALL --security-opt no-new-privileges \
  --mount "type=bind,source=$PWD,target=/workspace,readonly" \
  --workdir /workspace -e CARGO_TARGET_DIR=/tmp/target \
  mtg-lab-toolchain:local \
  cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/pass-v1.json
```

The command emits JSONL episode summaries. Both players keep, pass and discard;
they do not play cards. For native-policy games, scripted play, replay and data
validation, follow the [examples](doc/getting-started.md#try-a-small-working-example)
and [simulation guide](doc/simulate.md).

## Documentation

| Guide | Contents |
| --- | --- |
| [Getting started](doc/getting-started.md) | Setup, verification and runnable examples. |
| [Supported functionality](doc/capabilities.md) | Complete capability matrix, limitations and acceptance evidence. |
| [CLI reference](doc/headless-commands.md) | Replay, inspection, trajectory validation, conformance and benchmarks. |
| [Architecture](doc/architecture.md) | Core module responsibilities, compatibility boundaries and repository layout. |
| [Scope and roadmap](doc/roadmap.md) | Frozen MVP scope, milestones and verification strategy. |
| [Testing](doc/testing/README.md) | Test designs, independent reference checks and delivery gates. |
| [Documentation index](doc/README.md) | Technical contracts, RFCs and operating guides. |

## Contributing

Start with [AGENTS.md](AGENTS.md) and the [contributor guide](doc/contributing.md).
Changes require independently justified tests, the full regression suite and
separate review before protected integration. Keep capability claims and their
evidence up to date when behavior changes.

Work is tracked in [GitHub issues](https://github.com/pabloxrl/mtg-lab/issues).
The [implementation program](https://github.com/pabloxrl/mtg-lab/issues/7) records
current progress and blockers. For the automated development service, see the
[Symphony runbook](doc/symphony-runbook.md).

## License and provenance

Repository licensing and release/data redistribution decisions remain open
release prerequisites. See the [provenance and distribution policy](doc/provenance-policy.md)
for source attribution and release restrictions.
