# Scalar unattended simulation, schema 1

Run inside the project toolchain container:

```sh
cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/pass-v1.json
cargo test -p mtg-cli simulate
```

The binary is `mtg`. It accepts only `simulate --config FILE` and optional
`--output NEW_FILE`. Existing files are never overwritten. Without `--output`,
stdout contains JSONL only; diagnostics are versioned JSON on stderr. No stdin,
TTY, display, browser, controller prompt or process per game is used. The
[example](../fixtures/simulate/pass-v1.json) is executable and fully resolved.

Configuration schema 1 rejects unknown fields and unsupported policies, game
configurations, versions and overflowing episode ranges. `game` is the existing
[opening configuration](opening.md), including frozen deck identities, optional
explicit orders, starting seat and RNG/shuffle versions. `master_seed`,
`first_episode`, positive `episodes`, positive `max_decisions`, and both `policies`
are required. Episode IDs are the consecutive range starting at `first_episode`;
core streams derive from master seed and each stable episode ID. `deadline_ms`
is an optional positive run wall-time limit, or null. Config files must be regular
files at most 1 MiB. The run clock begins after validation and output opening;
it includes reset, policy, core transitions and writing. The decision budget
includes opening and cleanup choices. Reset and automatic rules work are not
policy decisions.

Only `pass-v1` is implemented here. Both seats explicitly choose it: keep the
opening seven, pass priority, and select the first required cleanup candidate
rows in the core's hand order. The core validates every action. This deterministic
policy consumes no policy RNG and reads no hidden cards. There is no random or
human fallback. It plays no lands or spells and never creates combat choices;
an unexpected choice is a failed episode, not an implicit action. Unplayed cards
retain their real identities. This does not claim complete behavior for either
frozen deck. Broader native random/heuristic policies remain M2; CLI recording,
replay and cross-feature acceptance remain #79/#21. No rules are implemented in
the CLI. No trajectory or performance benchmark is produced.

Every record carries `schema_version: 1`:

- `run`: resolved configuration, configuration SHA-256 (compact Serde struct JSON
  in declaration order), exact core Rust source fingerprint, CLI/policy version,
  rules/card manifest byte hashes, worker count 1, `summary-v1` instrumentation,
  and `capture: none`. The card manifest hash pins both deck definitions. The
  core fingerprint hashes sorted source basenames, NUL delimiters and file bytes;
  it is a conservative source identity, not a Git commit or binary checksum.
- `episode`: stable episode ID, `status`, reason, successful decision count,
  winner seat 0/1 or null, final life and turn when available, and error or null.
  Failed reset has no final game fields. Completed records come only from the
  core's rules outcome. These records are summaries, not replays or trajectories.
- `summary`: requested, started, completed, truncated, failed, unfinished,
  not_started, per-seat wins, draws, stop reason and exit code.

`started = completed + truncated + failed + unfinished` and
`requested = started + not_started`. Only completed games contribute wins/draws.
A per-episode decision limit truncates that episode and continues the run. A run
deadline truncates the active episode and leaves remaining episodes not started.
SIGTERM/SIGINT mark an active episode unfinished and stop before starting another.
A signal between episodes can leave zero unfinished episodes. Terminal outcomes
take precedence over an observed limit after the final successful action. Engine
errors quarantine one failed episode and stop; they do not become truncations.

Exit codes: 0 for fulfilled episode budget (including explicitly limited episodes),
2 for command/config errors, 3 for runtime/output errors, 4 for run deadline,
130 for SIGINT and 143 for SIGTERM. Check status counts even on exit 0.
Validation failures emit no run; output failures return nonzero and may leave a
partial JSONL file with no final summary. Treat any file without the final summary
as incomplete. Output flushes each record and keeps only one game resident;
it does not claim durable fsync or atomic publication. Seeds/configuration are
restricted experiment metadata, not player observation output.

Signal handlers only set atomics; work checks them between core decisions and
episodes. Reset and each passive transition are bounded by the frozen deck.
No process can guarantee a final summary after SIGKILL, a crash, or a failed or
indefinitely blocked output sink. The subprocess acceptance harness imposes an
external ten-second deadline and continuously drains output. Run under an
external supervisor when storage/pipe consumers can stall. `deadline_ms` is a
cooperative simulation deadline, not an I/O preemption mechanism.

[Behavioral red/green and regression evidence](evidence/simulate/README.md).
