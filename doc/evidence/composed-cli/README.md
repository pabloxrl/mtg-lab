# Composed played CLI acceptance — GH-120

This aggregate audit covers **every original #120 CLI clause**, after #179
(capture), #180 (replay verification), and #181 (dataset validation). Base:
`b661c838bbf77735b32f89387bf1130b9c415a35`. It does not accept M1: #21 still
owns its complete integration/catalog audit and initial active-policy measurement;
#22 owns the milestone verdict. M2–M5 requirements remain unchanged. Partial
R0002-B020/B039/B041 ownership does not satisfy those entire blocks.

Delivery is conditional on separate clean-candidate review, protected merge and
successful CI on that exact main commit. Final receipts and handoff are recorded
in the [single workpad](https://github.com/pabloxrl/mtg-lab/issues/120#issuecomment-5907720166)
and linked PR. Child completion is not this audit's executable verdict.

## Fresh composed tests and independent expectations

Run `cargo test --locked -p mtg-cli --test composed_cli`. Five normal-discovery
tests invoke real binaries with closed stdin, no DISPLAY/WAYLAND_DISPLAY, piped
JSON output, explicit deadlines and kill/reap on timeout. They exercise the actual
Driver → canonical recorder/publisher → public validation/replay commands. No
fake sibling, alternative engine, recorder or generated golden fixture is used.

- `simulate_to_validate_and_replay_preserves_literal_script_and_exact_capture`
  uses the unchanged seed-178/ordinal-0 ordered-green script. Its independently
  reviewed [CR-referenced ledger](../script-cli/README.md) supplies 101 choices
  plus concession, life `[20,15]`, hands `[5,7]`, libraries `[31,31]`, P0 winner,
  and once-only returns `[1,-1]`. The public manifest and explicit-v2 validators
  must report exactly one completed episode and 101 decisions; public replay
  verification must report that literal terminal outcome. Every replay action
  equals the original semantic input, with an explicit 102-record count.
  Re-executing the literal records through the owned API compares **every**
  canonical header/decision/footer field to CLI-produced bytes from another
  process. Reconstructed full state/RNG equals the retained owned snapshot,
  normalizing only process-local store/scope namespaces. This equality is
  metamorphic evidence, not an independent all-field rules oracle. Capture off
  preserves the entire public episode row. Removing private replay bytes leaves
  dataset validation unchanged and makes explicitly selected replay verification
  fail: an opaque link grants no authority and is never automatically followed.
- `native_captured_games_feed_both_verifiers_from_both_starting_seats` runs the
  delivered heuristic/random configuration from both starters. Real completed
  artifacts pass both public commands, counts/life agree across commands, and
  the complete episode row equals the same-seed capture-off run. Native outcomes
  are equality checks, not claimed predetermined winners or policy strength.
- `composed_diagnostics_reject_completed_claim_and_account_never_started_requests`
  requests three episodes and independently induces missing/illegal script,
  decision limit, work limit and record capacity cases. Exactly one starts and
  two never start. Four accepted opening/upkeep choices followed by missing or
  wrong-incarnation land preserve seven-card hands, `[20,20]` life, four consumed
  records and incomplete owner status. The public default reader rejects every
  diagnostic manifest; explicit diagnostics report one noncompleted declaration
  with the owner's exact status, and no private replay exists. Work-limit script
  summaries call the external stop truncated while retaining owner `incomplete`;
  the manifest truthfully records that owner status, not a fabricated game end.
- `composed_writer_failure_cannot_be_validated_as_completed_capture` requests
  three explicit concessions and a one-byte capture budget. All three genuine
  game outcomes remain completed; publication fails, no manifest exists, and
  even diagnostic public validation rejects the absent dataset. This storage
  limit case supplements the actual injected writer/flush/sync failures below.
- `captured_deadline_and_sigterm_leave_no_false_completed_artifacts` exercises
  a real one-millisecond native-run deadline and real SIGTERM after the run header. It
  checks exits 4/143, every one of 100 requested episodes, nonzero never-started
  accounting and failed interrupted publication. No completed manifest exists;
  diagnostic validation cannot turn absent artifacts into valid data. Timing
  does not fix the number started; deterministic injected-clock tests below
  cover exact owner boundaries.

These are acceptance additions with no production or rules change. Initial test
construction errors (wrong Rust return type, replay key `action` instead of
`choice`, and deadline code 124 instead of the documented 4) were corrected
against existing type/format/command contracts. The stop fixture uses native
policies so a delayed signal cannot instead exhaust a single-episode script;
completed-before-signal games are retained by the accounting equations rather
than requiring zero completions. They are **not** product
behavioral-red evidence. Existing component red/green evidence remains in the
linked reports; every existing test and assertion is retained unchanged.

## Complete original #120 crosswalk

All paths below are ordinary cargo test discovery and run in full torture.
`composed` means [composed_cli.rs](../../../crates/mtg-cli/tests/composed_cli.rs),
`capture` means [capture_tests.rs](../../../crates/mtg-cli/src/capture_tests.rs),
`script` means [script_tests.rs](../../../crates/mtg-cli/src/script_tests.rs).

| Original acceptance clause | Fresh executable evidence / retained regression |
| --- | --- |
| Native policies in existing simulate | Composed native both-starter producer/validator/replay; `native_simulate::native_real_games_repeat_and_match_direct_libraries_with_rules_checkpoints` |
| Explicit semantic scripts | Composed literal ledger; `script_simulate::script_played_combat_reaches_literal_rules_terminal_without_concession` and both-starter full-combat scripts |
| Existing verify commands | Composed public replay/manifest/v2; retained `headless_commands`, `played_replay_commands`, `trajectory_commands` |
| Canonical capture; no second recorder | Composed all-field owned-reference equality; capture `exact_owned_rows_replay_authority_and_capture_on_off_full_state_rng` |
| Opaque replay links | Composed removal/nonresolution; capture exact owner grant denial, missing/corrupt replay rejection |
| Explicit episode budget | Composed one/three/100 requested accounting; `native_bad_versions_policies_bounds_and_overflow_fail_without_output` |
| Explicit work budget | Composed work stop and owner/summary distinction; native/script limit tests and capture deterministic stops |
| Explicit capture budgets | Composed record/storage exhaustion; capture queue overflow; `invalid_capture_configuration_has_no_output_or_storage_mutation` |
| Headless structured errors | Every composed child closes stdin/removes displays; versioned stdout/stderr checks; retained malformed/config/dependency/redaction command tests |
| No second engine or performance qualification | Test-only addition calls production APIs/commands; no engine/recorder/runtime changes; no speed or strength inference |
| Played games produce run manifest | Composed actual simulate → manifest validator with literal counts |
| Played games produce JSONL | Same artifacts through explicit structured-v2 validator and strict library reload |
| Played games produce replay | Same publication through public replay verifier and privileged full reconstruction |
| Exact reload equality | Composed fresh-process CLI bytes equal every owned canonical field; full semantic snapshot/RNG equality; original semantic action equality |
| Missing choices preserve state | Composed four-choice checkpoint; script `routing_rejects_preserve_full_owner_and_capture_then_valid_script_recovers` checks complete debug state, RNG, history, cursor, capture at every position |
| Illegal choices preserve state | Same unchanged-state test plus wrong incarnation/second land; composed wrong-incarnation artifact diagnosis; no fallback |
| Writer failure boundaries | Composed storage failure → rejecting validator; capture `publication_fault_boundaries_keep_game_outcome_and_uncertainty_separate`, flush failure and collision/overflow regressions |
| SIGTERM boundaries | Composed real signal → absent-manifest rejection; `captured_os_signals_preserve_requested_accounting` retains SIGINT too; capture injected game/publication boundary tests |
| Deadline boundaries | Composed real deadline; capture `captured_limits_and_game_stops_preserve_diagnostics_and_not_started`, `publication_deadline_and_interrupt_before_during_and_after_commit`, configured-limit provenance regression |
| Truncation boundaries | Composed decision/work limits → completed-only rejection/diagnostic validation; native/script stop and trajectory mixed-status tests |
| ALL requested episodes | Composed equations: requested = started + not_started; started = completed + truncated + failed + incomplete; individual row count = started |
| Capture on/off same seed | Composed full public rows for literal script/native both starters; capture exact complete history/state/RNG equality for native/script |
| Existing commands supported | Retained passive simulate, output file/broken stdout, scalar-pass bench, opening verify/seat inspect, v1/v2/manifests, conformance mismatch/dependency/timeout tests; all run again in full torture |

The [complete prerequisite crosswalk](../../programs/cli-prerequisites.md)
continues to assign #21 its original catalog, SYS-CORE-009 configuration,
benchmark and all integration obligations. This audit neither transfers those
obligations nor changes the catalog/manifest/ledger.

## Boundaries and use

Use the documented [native/script/capture configurations](../../simulate.md)
and [validation/replay syntax](../../headless-commands.md). A dataset root and
private replay root must already exist and be disjoint. Supply explicit
`local-owner-v1` authorization and count/queue/per-artifact bounds. Capture writes
one run manifest and `episodes.jsonl`; replay file selection is separately
privileged. Validation defaults to completed-only. `--diagnostic` accepts valid
unfinished metadata, never corrupt/absent data or an apparently completed prefix.

The run summary includes never-started requests; manifests describe started owned
results. Gameplay completion and publication success are different fields. A
persistent signal/deadline prevents publication, so the absence of a manifest is
an explicit failed capture, not a promised durable unfinished episode. Publication
uncertainty remains distinct and is never silently retried. A failed stdout sink
may prevent the final summary; missing summary means incomplete run output.
Cooperative deadlines do not preempt filesystem stalls, SIGKILL or crashes.
Artifact/count bounds are not total RSS/disk quotas. Datasets with both seats and
resolved seeds/configuration remain privileged offline experiment artifacts.

No full-pool, training/batch/Parquet, persistent decision protocol, interactive
mode, performance or reference qualification is claimed. No rules change means
no new cached reference comparison is claimed or required by this audit; all
existing mandatory regressions and later reference gates remain intact. README
links this aggregate evidence while retaining #21/#22 and M1 as pending.
