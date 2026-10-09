# Scalar unattended simulation

Run inside the project toolchain container:

```sh
cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/pass-v1.json
cargo test -p mtg-cli simulate
```

The binary is `mtg`. Its simulation command accepts `simulate --config FILE` and optional
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

Schema 1 supports `pass-v1`. Both seats explicitly choose it: keep the
opening seven, pass priority, and select the first required cleanup candidate
rows in the core's hand order. The core validates every action. This deterministic
policy consumes no policy RNG and reads no hidden cards. There is no random or
human fallback. It plays no lands or spells and never creates combat choices;
an unexpected choice is a failed episode, not an implicit action. Unplayed cards
retain their real identities. This does not claim complete behavior for either
frozen deck. Schema 2 native policies and schema 3 scripts support optional canonical capture below.
Played replay routing is documented in [commands](headless-commands.md); composed acceptance
remains with #120/#21. No rules are implemented in the CLI. Passive schema 1
produces no trajectory or performance benchmark.

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


## Native simulation, schema 2

```sh
env -u DISPLAY -u WAYLAND_DISPLAY cargo run --quiet --locked -p mtg-cli -- simulate --config fixtures/simulate/native-v2.json </dev/null
cargo test -p mtg-cli --test native_simulate
```

The [resolved example](../fixtures/simulate/native-v2.json) keeps the same required
seed, episode, decision and game fields, sets `schema_version: 2`, and adds a
required `native` object. Each seat explicitly selects `legal-random-activation-mana-v1` or
`heuristic-activation-mana-v1`; mixing these two is supported. `pass-v1` remains schema 1 only.
`bench --workload scalar-pass-v1` accepts schema 1 only, never silently substituting
a passive policy for native configuration. Unknown versions/policies fail before
output; no human, random, or pass fallback exists.

All native object fields are required:

| Field | Meaning |
| --- | --- |
| `policy_seed` | Unsigned 64-bit master for independent episode/seat policy streams; heuristic is stateless and does not consume it |
| `rng_version` | Exactly `legal-random-rng-v1`, independent from the environment RNG version |
| `work_quantum` | Positive platform-sized integer clamping each owner reset/advance call's internal work |
| `max_work_calls` | Positive u64 per-episode ceiling counting reset, each advance, and each attempted policy submission once |
| `max_records` | Positive platform-sized ceiling on accepted semantic history records; exhaustion fails the episode |

`max_decisions` remains a positive per-episode accepted-choice limit. These are
count/work limits, not byte/RSS or performance promises. Submit is synchronous;
one call can execute its existing atomic core operation. Limits include opening
choices. One work call with quantum one leaves a partial reset, zero decisions,
and unavailable final views. Work exhaustion abandons the owner and is reported
as **incomplete**, not a rules result or a bootstrappable truncation. Decision and
deadline limits use owner **truncated** outcomes. Record exhaustion is **failed**.
Work/decision limits continue to the next requested episode; a failure stops the
run. With capture disabled the run retains only one game and its explicitly bounded semantic history.
Policy observation capacity is 256 rows/domain entries; an owner capacity error
fails explicitly rather than clipping legal choices.

The caller checks run deadline/signals between owner operations and before every
episode. Deadline expiry is passed to the owner's injected monotonic clock before
finalization; owner precedence (failure, rules terminal, decision, time) remains
authoritative. SIGTERM/SIGINT finalize an active owner as incomplete. Neither
signals nor deadline fabricate a rules loss. Already-terminal episodes retain
completion. Output errors are caller errors and may prevent the final summary;
there is no claim of recovery from a stalled sink, SIGKILL or a crash.

Output records retain schema 1 envelopes, with `execution: owned-native-v1` in
the run header and the complete schema 2 config/policy identities. Native episode
rows expose final public life/turn/active seat/step, hand/library **counts**, public zones, terminal
predicates and a semantic-history digest, never private hand contents. The digest
is SHA-256 of compact JSON encoding of the owner's ordered semantic byte records;
it is an equivalence receipt, not a replay. The optional `caller_error` is a fixed
redacted category. Policy/routing errors leave an incomplete owner and nonzero
runtime exit, separately from owner failures. An unavailable finalization has
null state/decision fields rather than invented results.

Native summaries use `incomplete` instead of legacy `unfinished`:
`started = completed + truncated + failed + incomplete`,
`requested = started + not_started`. Only completed outcomes affect wins/draws.
Exit codes remain 0/2/3/4/130/143 as above; exit 0 can include explicit limited
outcomes. A maximum-u64 episode request is processed incrementally with checked
ID ranges and no episode-count allocation. Config 1 JSON and existing commands
remain compatible.

The sole rules owner is `episode::Driver`; policies consume seat-authorized
observations and submit through its validated boundary. Capture defaults to disabled.
Supported policy actions include the delivered thirteen-card subset (Forest,
Mountain, Bear Cub, Swab Goblin, Giant Growth, Bite Down, Dragon Fodder,
Llanowar Elves, Druid of the Cowl, Magnigoth Sentry, Axgard Cavalry, Tajuru Pathwarden
and Thornweald Archer), token combat, reach, haste, vigilance, trample and deathtouch;
other fixed-deck cards are uncastable. Scripts and optional capture use that same
loop as described below. No new algorithm, second game loop, batching/training,
performance or strength qualification is delivered. #120 retains all composed CLI clauses; #21/#22 and
later requirement owners remain unchanged.

[Native acceptance and independent expectations](evidence/native-cli/README.md).

## Semantic scripts (schema 3)

Schema 3 uses the same owned run loop and `Driver::submit_record`, with capture
disabled unless explicitly configured below. Both `policies` must be `semantic-script-v1`; mixing scripts and
native policies is rejected. Schema 1/pass-v1 and schema 2/native behavior remain.
The `native` bounds object is still required: work quantum/calls and history
capacity apply identically. Its policy seed/RNG version retain their validated
schema-2 representation but no native policy is instantiated in script mode.

Add `script` with `version: 1`, `privacy: "privileged"`, positive `max_bytes`
(up to 1 MiB), positive `max_records` (up to 100,000), and an ordered `records`
array. Each entry has:

- `episode`: exact requested episode ordinal, starting at `first_episode`.
- `decision`: zero-based accepted record position within that episode. Concession
  occupies the next position but does not increment the owner's decision count.
- `seat`: `P0` or `P1`, explicitly authorized for this record.
- `record`: a JSON string containing one existing [semantic v1 action](actions.md).

The byte budget measures UTF-8 compact JSON serialization of the complete entry
array, including envelopes and escaped record strings. The entire configuration
also has a 1 MiB read cap. Bounds/version/privacy checks occur before run startup;
individual records are decoded at their exact action boundary. Missing, stale
position, wrong episode/seat, malformed, illegal and wrong-incarnation inputs
stop with a redacted `script_*` caller error and exit 3. No skipping, looping,
implicit choices, record reuse across episodes or native fallback occurs.
Live process-local owner tokens are obtained only after the explicit static
position check; they are not persisted as portable decision identities.
Concession by either seat uses the owner's out-of-band concession path.

Scripts are privileged replay/debug input containing identities that must not be
provided to a player policy. Ordinary output omits record payloads (including
malformed ones), retaining the configuration hash, bounds and privacy declaration.
The resolved game configuration, exact deck order and seed retain their existing
privileged run-metadata meaning. This is not a player observation endpoint.

Episode output adds `script_consumed`, `script_status` and `owner_status`; summary
output adds total `script_consumed` and `script_remaining`. A terminal game with
extra records retains its genuine completed game/winner accounting but has
`script_status: "error"`, `caller_error: "script_extra"` and exit 3. Valid full
consumption requires every requested episode to complete, exit 0 and zero
remaining records. A decision/deadline stop is truncation. A script work stop is
also reported as external run truncation, while `owner_status: "incomplete"`
retains the owner's unchanged status (native work stops remain incomplete).
Signals remain incomplete, owner capacity failures remain failed. Any nonterminal
script stop ends the run and accounts for later requested episodes as not started;
remaining input is never implicitly moved to the next episode.

`fixtures/simulate/script-v3.json` is an original normal-reset ordered-green
example: land, Cub, Growth target/payment, unblocked combat, then concession.
[Executable acceptance and independently justified checkpoints](evidence/script-cli/README.md)
cover both starting seats, multiple episodes, exact records and nonmutation.
Scripts alone do not imply persistence or M1 completion; capture is separately opt-in.

## Canonical capture (schemas 2 and 3)

Add an optional `capture` object to a native or script configuration. Every field
is required; unknown fields/versions and schema-1 capture fail explicitly:

```json
{
  "dataset_root": "/tmp/mtg-example/datasets",
  "replay_root": "/tmp/mtg-example/replays",
  "authorization": "local-owner-v1",
  "max_episodes": 2,
  "queue_bytes": 67108864,
  "max_bytes": 67108864,
  "backpressure": "fail"
}
```

Both roots must already be directories, and their canonical paths must be
disjoint (neither may contain the other, including aliases). Validation precedes
output creation and game execution. The CLI neither creates roots nor changes
their permissions. The trusted local experiment owner must control them; this
is not protection against a hostile host or concurrent directory substitution.
Root paths and the authorization declaration are omitted from run output.
`local-owner-v1` deliberately authorizes this process to publish its own completed
replays. It creates exact ID/episode grants in memory only. No grant is persisted;
knowing a run/replay UUID grants no library read access. Privileged local file
selection remains a separate deliberate access path.

`max_episodes` is a positive ceiling on the requested count and retained results.
`native.max_records` bounds each episode's semantic history and captured choices.
`queue_bytes` bounds the existing writer queue; `backpressure` explicitly chooses
`block` (drain synchronously) or `fail` (reject overflow), never dropping records.
`max_bytes` bounds each sealed JSONL, manifest, and individual replay artifact.
These are explicit count/artifact bounds, not a total RSS or total disk quota.
The current publisher buffers the bounded run and replays in memory; peak use
includes serialization and verification copies. Final replay encoding is checked
after creation. Use modest episode/record limits; this is not a batch exporter.

Run and replay IDs come from OS randomness, independent of game/policy RNG. The
CLI uses `Driver::reset_captured`, `Run::header`, the canonical v2 recorder,
`Registry` binding, and the existing manifest-last publisher. Completed captures
produce `dataset_root/RUN/manifest.json`, `episodes.jsonl`, and opaque replay files
under `replay_root/RUN/REPLAY`. Private run directories are mode 0700 and replay
files 0600. Dataset readers must still be authorized: both seats' separate
observations reveal both hands to an offline reader. Policy inputs never gain
opponent observations or replay access.

After game rows and before the summary, a separate `publication` row reports
`run_id`, `status` (`published`, `failed`, `uncertain`), a redacted reason, `started`
and `retained_results`. Summary adds `publication`; game counts and winners keep
their genuine owner meaning even if storage fails. Publication errors use exit 3
unless a prior gameplay error/signal/deadline already determines the exit. An
observed publication stop uses 4/130/143. All requested-but-never-started episodes
remain in summary `not_started`; the canonical manifest describes started results
only and is not a replacement for that run summary.

Publication occurs once after the bounded game loop. Valid unfinished results
can publish diagnostic manifests, rejected by completed-only loaders. Signals or
deadlines still active at publication abort it explicitly. Control is checked
before replay registration and at filesystem stages including flush/sync/link;
withdrawal cleanup remains permitted after a stop. Cancellation is cooperative,
not preemption of serialization, replay verification or blocking I/O. A committed
manifest remains successful if a signal arrives after the last observed boundary.
A crash/SIGKILL/output failure can prevent acknowledgment or the final summary.
Inspect artifacts through existing validators; never infer failure from a missing
acknowledgment or silently retry reserved IDs. `uncertain` means a publication
link could not be durably withdrawn; it is never treated as successful publication.
See [publisher recovery limitations](collector-publication.md).

Bounded, closed-stdin script quickstart (choose a new local directory):

```sh
mkdir -p /tmp/mtg-example/datasets /tmp/mtg-example/replays
python3 - <<'PY'
import json
from pathlib import Path
c = json.loads(Path('fixtures/simulate/script-v3.json').read_text())
c['capture'] = dict(dataset_root='/tmp/mtg-example/datasets',
                    replay_root='/tmp/mtg-example/replays',
                    authorization='local-owner-v1', max_episodes=2,
                    queue_bytes=67108864, max_bytes=67108864, backpressure='fail')
Path('/tmp/mtg-example/config.json').write_text(json.dumps(c))
PY
env -u DISPLAY -u WAYLAND_DISPLAY cargo run --quiet --locked -p mtg-cli -- simulate --config /tmp/mtg-example/config.json </dev/null
```

Swap the source fixture for `native-v2.json` to capture native games. No second
recorder, replay format, persisted access grant, sampling/sharding, new verification
command, performance claim, or aggregate M1 completion is introduced.
[Executable component evidence](evidence/captured-cli/README.md).


## Optional scalar counters

For native (schema 2) or semantic-script (schema 3) runs, set
`native.instrumentation` to `"counters"`. Omit it or use `"off"` for the default.
The final summary then contains a versioned `metrics` object with bounded fields;
existing episode output and independently requested capture retain their contracts.
Unknown instrumentation modes fail configuration validation. See
[scalar metric definitions and limitations](scalar-metrics.md).

Create a short runnable example in the managed toolchain environment:

```sh
python3 - <<'PYTHON'
import json
from pathlib import Path
c = json.loads(Path("fixtures/simulate/native-v2.json").read_text())
c["native"]["instrumentation"] = "counters"
c["max_decisions"] = 4
Path("/tmp/mtg-counters.json").write_text(json.dumps(c))
PYTHON
cargo run --quiet --locked -p mtg-cli -- simulate --config /tmp/mtg-counters.json
```

This deliberately short run demonstrates decision-limit truncation, not completed-game throughput. The summary separates truncations from natural rules endings;
it makes no measured performance claim.


### Sampled diagnostics and full replay

The same native/script configurations accept `"sampled_trace"` and `"full_replay"`.
For sampled mode, optionally set `native.trace` to `{"every": 2, "capacity": 1}`.
Every second accepted decision is selected; the first fits and subsequent selected
records increment the episode's explicit `diagnostics.dropped` count. Defaults are
64 and 256; zero capacity counts all selected records as dropped, and capacities
above 65,536 or trace settings in another mode fail validation. Diagnostic rows
contain only numeric decision/work checkpoints, never replay payloads.

To exercise the example above, change its mode to `sampled_trace` and add that
`trace` object. Each four-decision episode retains decision 2 and drops decision 4.
The diagnostic report is separate from the aggregate `metrics` summary.

Full replay generates and verifies the existing complete played-replay artifact
for completed episodes. Public episode output contains only `replay_status`:
`available_in_memory`, `incomplete`, or `failed`. The short truncated example
correctly reports `incomplete`; it cannot masquerade as a complete replay.
In-memory export is limited to 64 MiB by the CLI, or `capture.max_bytes` when
capture is requested. This is an artifact-size limit, not a peak-memory guarantee.

Use the existing canonical capture example above for durable replay publication
in the restricted replay root. Without capture, CLI replay generation validates
in memory and does not persist an artifact. Capture remains independently usable
in every mode, including off. Overflow or write failure returns a failure instead
of dropping replay records; publication status is separate from the game outcome.
No new recorder format, storage backend or performance qualification is implied.
