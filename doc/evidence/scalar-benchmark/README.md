# Scalar benchmark contract evidence

Issue #216 owns the versioned scalar measurement contract, not performance
qualification or a whole RFC requirement block. Original #25 and #26 retain
aggregate acceptance. Independent expectations come from RFC 0002 B008/B019/B020
and `SYS-PERF-001/002`: include reset and failures in elapsed time, count only
rules-terminal games, retain unfinished attempts, and publish repeated windows.

The controlled-clock regression uses three attempts with independently specified
reset/work durations of 2+3, 1+4 and 2+5 nanoseconds. For a requested 16ns window,
the denominator is 17ns, with three starts, one completion, one failure and one
unfinished episode. The completion rate is exactly 1e9/17 per second. These are
test-only accounting inputs, not a second game implementation or speed evidence.

Production validation must reject warmup below ten seconds, fewer than five
windows, windows below thirty seconds, incompatible versions and missing or
unsupported policy identities. Short injected-clock tests do not expose a CLI
override for those minima. Zero completions produce no successful completion
rate.

The historical issue description refers to the six-card M1 workload. Current
main now supports native full-pool policies (#208); its existing frozen 40-card
decks and normal reset remain authoritative. This delivery must not recreate an
old simplified rules path or call current full-pool games six-card evidence.
Small normal-run contract tests establish wiring; full-pool profiling and measured
qualification remain #217 and the aggregate gates.

## Executable evidence

[Compiled behavioral red](red.txt) preserves three intended assertion failures
before implementation. The tests are in ordinary `mtg-cli` test discovery;
`cargo test -p mtg-cli --bin mtg benchmark::tests` runs the focused contract.

| Tests in `benchmark::tests` | Independent expectation |
| --- | --- |
| `benchmark_clock_includes_reset_failure_and_boundary_overshoot` | Literal 17ns/three-start/one-completion arithmetic above. |
| `benchmark_distribution_retains_zero_windows_and_nearest_rank_percentiles` | Samples 9,1,0,3,2 retain order; mean 3, p50 2, p95 9. |
| `benchmark_rejects_invalid_contract_without_reducing_production_minima` | RFC 10s warmup and at least five 30s windows; explicit version/policy/strict JSON validation. |
| `benchmark_zero_completions_have_no_successful_rate` | No completed game means no successful completion rate. |
| `benchmark_natural_wins_and_draws_exclude_concessions` | Life/empty-draw and simultaneous natural losses qualify; concession does not. |
| `benchmark_real_native_attempt_and_clock_phases_preserve_semantics` | Two accepted decisions mean policy initialization plus two choose calls, two observations and two real JSON encodes. Each injected interval is 7ns. Native JSONL bytes remain identical. Rejected reset retains timing and fails. |
| `benchmark_real_window_boundary_retains_unfinished_and_terminal_game` | A boundary stop during a real reset/game is unfinished; an unbounded-clock normal game reaches one real terminal result. Existing counters agree with the owner's actual decisions/completion. |
| `benchmark_signal_before_warmup_retains_metadata_and_no_success` | A pending signal starts no game and retains the signal exit code, pins and an empty distribution. |
| `benchmark_repeated_windows_are_raw_and_zero_completion_run_fails` | Injected clock exercises 10s warmup and five 30s windows without a production override; unfinished games and zero samples remain explicit and the run fails. |
| `benchmark_command_routes_version_and_rejects_malformed_before_output` | Public command parser rejects malformed input before creating output; a valid interrupted command writes the versioned report. |

This change adds no mechanic or reference adapter. Existing rules-derived and
pinned-reference tests remain unchanged; no new XMage/Forge agreement is claimed
for timing arithmetic. The full torture suite retains their existing coverage.
Native normal-reset contract tests exercise the authoritative Driver, not a fake
game. Production benchmark timing and full-pool qualification are distinct from
these deterministic checks.

The issue's Agent workpad and delivery PR retain full torture, exact-candidate
independent review, protected merge and exact-main CI receipts. Completion is
conditional on those checks; this document alone is not a delivery verdict.
