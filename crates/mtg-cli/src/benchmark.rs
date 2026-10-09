//! Versioned scalar measurement contract (RFC B008/B019/B020).
use crate::{native, simulate};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

const WORKLOAD: &str = "scalar-windows-v1";
pub(crate) const FULL_POOL: &str = "scalar-full-pool-v1";
const ATTEMPTS: u64 = 100_000;
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    schema_version: u32,
    workload: String,
    warmup_seconds: u64,
    window_seconds: u64,
    windows: u32,
    policies: [String; 2],
    master_seed: u64,
    policy_seed: u64,
    instrumentation: mtg_core::metrics::Mode,
    #[serde(default)]
    encoding: bool,
}
impl Config {
    fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || ![WORKLOAD, FULL_POOL].contains(&self.workload.as_str()) {
            return Err("incompatible scalar workload/schema version".into());
        }
        if !matches!(
            self.instrumentation,
            mtg_core::metrics::Mode::Off | mtg_core::metrics::Mode::Counters
        ) {
            return Err("scalar-windows-v1 supports only off/counters instrumentation".into());
        }
        if !(10..=3600).contains(&self.warmup_seconds)
            || !(30..=3600).contains(&self.window_seconds)
            || !(5..=100).contains(&self.windows)
        {
            return Err("require warmup 10..3600s, 5..100 windows each 30..3600s".into());
        }
        self.run_config()?.validate()
    }
    fn run_config_at(&self, ordinal: u64) -> Result<simulate::Config, String> {
        let mut c = self.run_config()?;
        c.first_episode = ordinal;
        if self.workload == FULL_POOL {
            full_pool_row(&mut c, ordinal);
        }
        Ok(c)
    }
    fn run_config(&self) -> Result<simulate::Config, String> {
        // Pin the existing real normal-reset workload, not synthetic game states.
        let mut c: simulate::Config = serde_json::from_slice(include_bytes!(
            "../../../fixtures/bench/scalar-workload-v1.json"
        ))
        .map_err(|e| e.to_string())?;
        c.episodes = 1;
        c.policies = self.policies.clone();
        c.master_seed = self.master_seed;
        let n = c.native.as_mut().unwrap();
        n.policy_seed = self.policy_seed;
        n.instrumentation = self.instrumentation;
        Ok(c)
    }
}

/// RG, GR, RR and GG, each starting seat. Ordinal is also the stable seed input.
pub(crate) fn full_pool_row(c: &mut simulate::Config, ordinal: u64) {
    let row = ordinal as usize % 8;
    let decks = [
        ("red", "green"),
        ("green", "red"),
        ("red", "red"),
        ("green", "green"),
    ][row / 2];
    c.game.seats[0].deck = decks.0.into();
    c.game.seats[1].deck = decks.1.into();
    c.game.starting_seat = (row % 2) as u8;
}

#[derive(Default, Debug, Serialize)]
struct Window {
    elapsed_ns: u128,
    started: u64,
    completed: u64,
    failed: u64,
    unfinished: u64,
    truncated: u64,
    concessions: u64,
    wins: [u64; 2],
    draws: u64,
    decisions: u64,
    completed_decisions: u64,
    work_calls: u64,
    counters: Option<Value>,
    policy_ns: u128,
    phases: native::Phases,
    first_episode: u64,
    attempts: u64,
    row_attempts: [u64; 8],
    row_completed: [u64; 8],
    stop_code: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}
impl Window {
    fn rate(&self) -> Option<f64> {
        (self.completed > 0 && self.elapsed_ns > 0)
            .then(|| self.completed as f64 * 1e9 / self.elapsed_ns as f64)
    }
    fn merge(&mut self, other: Self) {
        for row in 0..8 {
            self.row_attempts[row] += other.row_attempts[row];
            self.row_completed[row] += other.row_completed[row];
        }
        self.started += other.started;
        self.completed += other.completed;
        self.failed += other.failed;
        self.unfinished += other.unfinished;
        self.truncated += other.truncated;
        self.concessions += other.concessions;
        self.wins[0] += other.wins[0];
        self.wins[1] += other.wins[1];
        self.draws += other.draws;
        self.decisions += other.decisions;
        self.completed_decisions += other.completed_decisions;
        self.work_calls += other.work_calls;
        if let Some(counters) = other.counters {
            match &mut self.counters {
                Some(total) => merge_counters(total, &counters),
                None => self.counters = Some(counters),
            }
        }
        self.policy_ns += other.policy_ns;
        self.phases.reset_ns += other.phases.reset_ns;
        self.phases.transition_ns += other.phases.transition_ns;
        self.phases.legality_and_view_ns += other.phases.legality_and_view_ns;
        self.phases.encoding_ns += other.phases.encoding_ns;
        self.phases.finalization_ns += other.phases.finalization_ns;
        self.phases.encoded_bytes += other.phases.encoded_bytes;
        self.stop_code = other.stop_code;
        self.error = other.error;
    }
}

// Only the existing fixed Counters schema enters this recursive sum. Its
// overflow flag is retained; an overflowed measurement cannot be successful.
fn merge_counters(total: &mut Value, other: &Value) {
    match (total, other) {
        (Value::Object(a), Value::Object(b)) => {
            for (key, value) in b {
                merge_counters(a.get_mut(key).expect("fixed counter schema"), value);
            }
        }
        (a @ Value::Number(_), Value::Number(b)) => {
            *a = json!(
                a.as_u64()
                    .unwrap()
                    .checked_add(b.as_u64().unwrap())
                    .expect("bounded measurement counters")
            );
        }
        (Value::Bool(a), Value::Bool(b)) => *a |= *b,
        _ => unreachable!("fixed counter schema"),
    }
}

// Test injection supplies elapsed time and attempt results, never alternate rules.
// Every attempt, including its reset, failure, finalization and accounting, stays
// inside the outer timer. Boundary overshoot is never subtracted.
fn window(
    duration_ns: u128,
    now: &dyn Fn() -> u128,
    attempt: &mut dyn FnMut() -> Window,
) -> Window {
    let start = now();
    let mut result = Window::default();
    while now() - start < duration_ns && result.attempts < ATTEMPTS {
        result.merge(attempt());
        result.attempts += 1;
        if result.stop_code != 0 {
            break;
        }
    }
    result.elapsed_ns = now() - start;
    if result.attempts == ATTEMPTS && result.elapsed_ns < duration_ns {
        result.stop_code = 3;
        result.error = Some("attempt safety bound reached before window minimum".into());
    }
    result
}
fn distribution(samples: &[f64]) -> Value {
    if samples.is_empty() {
        return json!({"samples":[],"mean":null,"p50":null,"p95":null,"min":null,"max":null});
    }
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    json!({"samples":samples,"mean":samples.iter().sum::<f64>() / samples.len() as f64,
        "p50":sorted[(sorted.len()*50).div_ceil(100)-1],"p95":sorted[(sorted.len()*95).div_ceil(100)-1],
        "min":sorted[0],"max":sorted[sorted.len()-1],"percentile_method":"nearest-rank"})
}
fn signal_stop(signal: &AtomicUsize) -> Option<simulate::Stop> {
    match signal.load(Ordering::Relaxed) {
        n if n == signal_hook::consts::SIGINT as usize => Some(simulate::Stop::Sigint),
        n if n == signal_hook::consts::SIGTERM as usize => Some(simulate::Stop::Sigterm),
        _ => None,
    }
}
fn natural_outcome(o: mtg_core::game::terminal::Outcome) -> bool {
    !o.losses
        .contains(&Some(mtg_core::game::terminal::LossReason::Concession))
}

fn attempt(
    c: &simulate::Config,
    now: Rc<dyn Fn() -> u128>,
    deadline: u128,
    signal: &AtomicUsize,
    encode: bool,
) -> Window {
    let mut timing = native::PolicyTiming {
        detailed: true,
        encode,
        clock: now.clone(),
        ..Default::default()
    };
    let mut bytes = Vec::new();
    let mut out = Window::default();
    let mut natural = false;
    let result = native::run_instrumented(
        c,
        &mut bytes,
        || signal_stop(signal).or_else(|| (now() >= deadline).then_some(simulate::Stop::Deadline)),
        &mut |_, _| Ok(()),
        &mut |r| {
            use mtg_core::episode::Status;
            if let Status::Completed(o) = r.status() {
                natural = natural_outcome(o);
            }
        },
        Some(&mut timing),
    );
    out.policy_ns = timing.elapsed_ns;
    out.phases = timing.phases;
    // Parsing and aggregation are intentionally within the measured window too.
    match result {
        Err(e) => {
            out.started = 1;
            out.failed = 1;
            out.stop_code = 3;
            out.error = Some(e.to_string());
        }
        Ok(code) => {
            let rows = bytes
                .split(|b| *b == b'\n')
                .filter(|b| !b.is_empty())
                .map(serde_json::from_slice::<Value>)
                .collect::<Result<Vec<_>, _>>();
            match rows {
                Err(e) => {
                    out.started = 1;
                    out.failed = 1;
                    out.stop_code = 3;
                    out.error = Some(e.to_string());
                }
                Ok(rows) => {
                    let episode = rows.iter().find(|r| r["type"] == "episode");
                    if let Some(e) = episode {
                        out.started = 1;
                        out.decisions = e["decisions"].as_u64().unwrap_or(0);
                        out.work_calls = e["work_calls"].as_u64().unwrap_or(0);
                        if !e["caller_error"].is_null() || e["status"] == "failed" {
                            out.failed = 1;
                        } else if e["status"] == "completed" && natural {
                            out.completed = 1;
                            out.completed_decisions = out.decisions;
                            if let Some(seat) = e["winner"].as_u64() {
                                out.wins[seat as usize] = 1;
                            } else {
                                out.draws = 1;
                            }
                        } else if e["status"] == "completed" {
                            out.concessions = 1;
                        } else if e["status"] == "truncated" {
                            out.truncated = 1;
                        } else {
                            out.unfinished = 1;
                        }
                    }
                    // Normal end-of-window is not an execution error. A signal
                    // retains its exit code; partial windows cannot qualify.
                    if code != 0 && code != 4 {
                        out.stop_code = code;
                    }
                    if out.failed > 0 {
                        out.stop_code = 3;
                    }
                    out.counters = rows
                        .last()
                        .and_then(|r| r.get("metrics"))
                        .and_then(|m| m.get("counters"))
                        .cloned();
                    if out
                        .counters
                        .as_ref()
                        .is_some_and(|c| c["overflowed"] == true)
                    {
                        out.stop_code = 3;
                        out.error = Some("instrumentation counters overflowed".into());
                    }
                }
            }
        }
    }
    out
}

fn measured_matrix_window(
    c: &mut simulate::Config,
    seconds: u64,
    now: Rc<dyn Fn() -> u128>,
    signal: &AtomicUsize,
    encode: bool,
    full_pool: bool,
) -> Window {
    let first = c.first_episode;
    let duration = seconds as u128 * 1_000_000_000;
    let deadline = now() + duration;
    let mut w = window(duration, now.as_ref(), &mut || {
        if full_pool {
            let ordinal = c.first_episode;
            full_pool_row(c, ordinal);
        }
        let mut r = attempt(c, now.clone(), deadline, signal, encode);
        if full_pool {
            let row = c.first_episode as usize % 8;
            r.row_attempts[row] = 1;
            r.row_completed[row] = r.completed;
        }
        c.first_episode += 1;
        r
    });
    w.first_episode = first;
    w
}

fn read_field(path: &str, prefix: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()?.lines().find_map(|l| {
        l.strip_prefix(prefix)
            .map(|s| s.trim().trim_start_matches(':').trim().to_owned())
    })
}
fn metadata() -> Value {
    json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,
        "cpu_model":read_field("/proc/cpuinfo","model name"),
        "ram_total":read_field("/proc/meminfo","MemTotal:"),
        "cpu_affinity":read_field("/proc/self/status","Cpus_allowed_list:"),
        "kernel":std::fs::read_to_string("/proc/sys/kernel/osrelease").ok().map(|s|s.trim().to_owned()),
        "logical_cpus_available":std::thread::available_parallelism().ok().map(|n|n.get()),
        "workers":1,"affinity_applied_by_benchmark":false,"physical_cores_allocated":null,
        "rustc":env!("MTG_BENCH_RUSTC"),"target":env!("MTG_BENCH_TARGET"),
        "opt_level":env!("MTG_BENCH_OPT_LEVEL"),"rustflags":env!("MTG_BENCH_RUSTFLAGS"),
        "debug_assertions":cfg!(debug_assertions),"build_commit":env!("MTG_BENCH_COMMIT"),
        "source_sha256":env!("MTG_BENCH_SOURCE"),
        "binary_sha256":std::env::current_exe().ok().and_then(|p| std::fs::read(p).ok()).map(|b| simulate::hash(&b)),
        "unavailable_metadata":"null means unavailable; no physical-core efficiency claim"})
}

pub(crate) fn run(bytes: &[u8], signal: &AtomicUsize) -> Result<(i32, Value), (i32, String)> {
    let start = Instant::now();
    run_with_clock(bytes, signal, Rc::new(move || start.elapsed().as_nanos()))
}
fn run_with_clock(
    bytes: &[u8],
    signal: &AtomicUsize,
    now: Rc<dyn Fn() -> u128>,
) -> Result<(i32, Value), (i32, String)> {
    let config: Config = serde_json::from_slice(bytes).map_err(|e| (2, e.to_string()))?;
    config.validate().map_err(|e| (2, e))?;
    let mut c = config.run_config_at(0).map_err(|e| (2, e))?;
    let pins = json!({"config":config,"resolved_episode":c,
        "deck_source_sha256":simulate::hash(include_bytes!("../../mtg-core/src/opening.rs")),
        "card_manifest_sha256":simulate::hash(include_bytes!("../../../data/cards/foundations_micro_v1.json")),
        "rules_manifest_sha256":simulate::hash(include_bytes!("../../../data/rules/cr-2026-09-25.json")),
        "action_schema_version":mtg_core::game::actions::ACTION_VERSION,
        "observation_schema_version":mtg_core::game::policy::SCHEMA_VERSION,
        "policy_source_sha256":simulate::hash(include_bytes!("../../mtg-policy/src/lib.rs")),
        "heuristic_source_sha256":simulate::hash(include_bytes!("../../mtg-policy/src/heuristic.rs")),
        "workload_source_sha256":simulate::hash(include_bytes!("benchmark.rs")),
        "fixture_sha256":simulate::hash(include_bytes!("../../../fixtures/bench/scalar-workload-v1.json")),
        "matchup_order":if config.workload == FULL_POOL {json!(["RG0","RG1","GR0","GR1","RR0","RR1","GG0","GG1"])}else{json!(["GG0"])},
        "instrumentation":config.instrumentation,"timing":"every-client-boundary-v1","capture":"none"});
    let hardware = metadata();
    let warmup = measured_matrix_window(
        &mut c,
        config.warmup_seconds,
        now.clone(),
        signal,
        config.encoding,
        config.workload == FULL_POOL,
    );
    let mut windows = Vec::new();
    if warmup.stop_code == 0 {
        for _ in 0..config.windows {
            let w = measured_matrix_window(
                &mut c,
                config.window_seconds,
                now.clone(),
                signal,
                config.encoding,
                config.workload == FULL_POOL,
            );
            let stopped = w.stop_code != 0;
            windows.push(w);
            if stopped {
                break;
            }
        }
    }
    let interrupted = windows.last().map_or(warmup.stop_code, |w| w.stop_code);
    let valid = interrupted == 0
        && windows.len() == config.windows as usize
        && windows.iter().all(|w| {
            w.completed > 0
                && w.failed == 0
                && w.elapsed_ns >= config.window_seconds as u128 * 1_000_000_000
        });
    let rates: Vec<_> = windows.iter().map(|w| w.rate().unwrap_or(0.0)).collect();
    let code = if interrupted != 0 {
        interrupted
    } else if valid {
        0
    } else {
        3
    };
    let raw: Vec<_> = windows
        .iter()
        .map(|w| {
            let mut v = serde_json::to_value(w).unwrap();
            if !config.encoding {
                v["phases"]["encoding_ns"] = Value::Null;
            }
            v["completed_games_per_second"] = json!(w.rate());
            v["has_rules_completions"] = json!(w.completed > 0);
            v["decisions_per_second"] =
                json!(w.decisions as f64 * 1e9 / w.elapsed_ns.max(1) as f64);
            v["mean_decisions_per_completed_game"] =
                json!((w.completed > 0).then(|| w.completed_decisions as f64 / w.completed as f64));
            v["logical_actions_per_second"] = json!(
                w.counters
                    .as_ref()
                    .and_then(|c| c["logical_actions"].as_u64())
                    .map(|n| n as f64 * 1e9 / w.elapsed_ns.max(1) as f64)
            );
            v
        })
        .collect();
    let mut warmup = serde_json::to_value(warmup).unwrap();
    if !config.encoding {
        warmup["phases"]["encoding_ns"] = Value::Null;
    }
    Ok((
        code,
        json!({"type":"benchmark","workload":config.workload,"qualification":"contract-only-not-speed-qualified",
        "status":if valid {"measured"}else{"incomplete-or-failed"},"pins":pins,"hardware":hardware,
            "warmup":warmup,"windows":raw,"completed_games_per_second":distribution(&rates),
            "distributions":{
                "elapsed_ns":distribution(&windows.iter().map(|w|w.elapsed_ns as f64).collect::<Vec<_>>()),
                "reset_ns":distribution(&windows.iter().map(|w|w.phases.reset_ns as f64).collect::<Vec<_>>()),
                "transition_ns":distribution(&windows.iter().map(|w|w.phases.transition_ns as f64).collect::<Vec<_>>()),
                "legality_and_view_ns":distribution(&windows.iter().map(|w|w.phases.legality_and_view_ns as f64).collect::<Vec<_>>()),
                "encoding_ns":if config.encoding {distribution(&windows.iter().map(|w|w.phases.encoding_ns as f64).collect::<Vec<_>>())}else{Value::Null},
                "policy_ns":distribution(&windows.iter().map(|w|w.policy_ns as f64).collect::<Vec<_>>())},
        "timing_contract":{
            "rollout":"elapsed_ns: reset, policy, legality/view, transitions, finalization, JSONL summary and accounting; includes failed/unfinished work and timer overhead",
            "reset":"Driver reset including initial bounded work",
            "transition":"Driver advance and submit; includes submission validation/semantic encoding, excludes reset and separately timed observe",
            "legality_and_view":"Driver observe seat probes including authorized view and legal candidate construction; not pure legality",
            "encoding":if config.encoding {"actual authorized Observation JSON serialization to temporary memory; not tensor encoding"}else{"not_measured: native track performs no extra observation serialization"},
            "policy":"policy initialization and every choose attempt",
            "finalization":"Driver finish including final observations, snapshot and history",
            "other":"elapsed minus disjoint measured phases; runner/report/clock/accounting work is retained",
            "startup_and_final_artifact":"excluded; config validation/hardware collection precede warmup, final report/output follow all windows",
            "python":"not_applicable","inference":"not_applicable","batch":"not_applicable",
            "cpu_seconds":"not_measured","memory":"not_measured"}}),
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use std::cell::Cell;
    fn input() -> Value {
        json!({"schema_version":1,"workload":"scalar-windows-v1","warmup_seconds":10,"window_seconds":30,"windows":5,"policies":[mtg_policy::HEURISTIC_VERSION,mtg_policy::VERSION],"master_seed":42,"policy_seed":42,"instrumentation":"counters"})
    }
    #[test]
    fn full_pool_contract_accepts_all_eight_rows_without_shorter_horizon() {
        let mut v = input();
        v["workload"] = json!("scalar-full-pool-v1");
        let c: Config = serde_json::from_value(v).unwrap();
        assert!(
            c.validate().is_ok(),
            "full-pool scalar workload must be supported"
        );
        // Frozen deck matrix: RG, GR, RR, GG, each with both starting seats.
        let expected = [
            ("red", "green"),
            ("green", "red"),
            ("red", "red"),
            ("green", "green"),
        ];
        for ordinal in 0..16 {
            let run = c.run_config_at(ordinal).unwrap();
            // Existing run configuration is the public behavior under test.
            let game = serde_json::to_value(&run.game).unwrap();
            let row = ordinal as usize % 8;
            assert_eq!(game["seats"][0]["deck"], expected[row / 2].0);
            assert_eq!(game["seats"][1]["deck"], expected[row / 2].1);
            assert_eq!(game["starting_seat"], row % 2);
            assert_eq!(run.max_decisions, 20_000);
            assert_eq!(run.native.unwrap().max_work_calls, 100_000);
        }
    }
    #[test]
    fn benchmark_rejects_invalid_contract_without_reducing_production_minima() {
        let c: Config = serde_json::from_value(input()).unwrap();
        c.validate().unwrap();
        for (field, value) in [
            ("schema_version", json!(2)),
            ("workload", json!("other-v1")),
            ("warmup_seconds", json!(0)),
            ("warmup_seconds", json!(9)),
            ("window_seconds", json!(0)),
            ("window_seconds", json!(29)),
            ("windows", json!(0)),
            ("windows", json!(4)),
            ("policies", json!(["", mtg_policy::VERSION])),
            ("policies", json!(["pass-v1", mtg_policy::VERSION])),
        ] {
            let mut v = input();
            v[field] = value;
            let c: Config = serde_json::from_value(v).unwrap();
            assert!(c.validate().is_err(), "accepted {field}");
        }
        for (field, value) in [
            ("windows", json!(-1)),
            ("window_seconds", json!(0.5)),
            ("windows", json!("5")),
            ("extra", json!(true)),
        ] {
            let mut v = input();
            v[field] = value;
            assert!(serde_json::from_value::<Config>(v).is_err());
        }
        let mut v = input();
        v.as_object_mut().unwrap().remove("policies");
        assert!(serde_json::from_value::<Config>(v).is_err());
        // Current main also supports diagnostics/replay modes in simulation;
        // they are not silently admitted into this frozen benchmark workload.
        for mode in ["sampled_trace", "full_replay"] {
            let mut v = input();
            v["instrumentation"] = json!(mode);
            assert!(
                serde_json::from_value::<Config>(v)
                    .unwrap()
                    .validate()
                    .is_err()
            );
        }
    }
    #[test]
    fn benchmark_clock_includes_reset_failure_and_boundary_overshoot() {
        // Independent arithmetic: three attempts of 2+3, 1+4 and 2+5 ns.
        // A 16ns window costs 17ns; only the first is rules-terminal.
        let clock = Cell::new(0);
        let mut count = 0;
        let w = window(16, &|| clock.get(), &mut || {
            count += 1;
            let (reset, work) = [(2, 3), (1, 4), (2, 5)][count - 1];
            clock.set(clock.get() + reset + work);
            Window {
                started: 1,
                completed: u64::from(count == 1),
                failed: u64::from(count == 2),
                unfinished: u64::from(count == 3),
                ..Window::default()
            }
        });
        assert_eq!(
            (w.elapsed_ns, w.started, w.completed, w.failed, w.unfinished),
            (17, 3, 1, 1, 1)
        );
        assert!((w.rate().unwrap() - 1e9 / 17.0).abs() < 1e-6);
    }
    #[test]
    fn benchmark_zero_completions_have_no_successful_rate() {
        assert_eq!(
            Window {
                elapsed_ns: 30_000_000_000,
                started: 2,
                failed: 1,
                unfinished: 1,
                ..Window::default()
            }
            .rate(),
            None
        );
    }
    #[test]
    fn benchmark_distribution_retains_zero_windows_and_nearest_rank_percentiles() {
        // Mean (0+1+2+3+9)/5 = 3; nearest-rank p50 is 2, p95 is 9.
        let d = distribution(&[9.0, 1.0, 0.0, 3.0, 2.0]);
        assert_eq!(d["samples"], json!([9.0, 1.0, 0.0, 3.0, 2.0]));
        assert_eq!(d["mean"], 3.0);
        assert_eq!(d["p50"], 2.0);
        assert_eq!(d["p95"], 9.0);
        assert_eq!(d["min"], 0.0);
        assert_eq!(d["max"], 9.0);
    }
    #[test]
    fn benchmark_natural_wins_and_draws_exclude_concessions() {
        use mtg_core::{
            game::terminal::{LossReason, Outcome},
            objects::Seat,
        };
        assert!(natural_outcome(Outcome {
            winner: Some(Seat::P0),
            losses: [None, Some(LossReason::Life)]
        }));
        assert!(natural_outcome(Outcome {
            winner: None,
            losses: [Some(LossReason::Life); 2]
        }));
        assert!(natural_outcome(Outcome {
            winner: Some(Seat::P1),
            losses: [Some(LossReason::EmptyDraw), None]
        }));
        assert!(!natural_outcome(Outcome {
            winner: Some(Seat::P0),
            losses: [None, Some(LossReason::Concession)]
        }));
    }
    #[test]
    fn benchmark_real_native_attempt_and_clock_phases_preserve_semantics() {
        let config: Config = serde_json::from_value(input()).unwrap();
        let mut c = config.run_config().unwrap();
        c.max_decisions = 2;
        let clock = Rc::new(Cell::new(0u128));
        let now: Rc<dyn Fn() -> u128> = {
            let clock = clock.clone();
            Rc::new(move || {
                let n = clock.get();
                clock.set(n + 7);
                n
            })
        };
        // Pure phase clock: every measured interval costs exactly 7ns.
        let mut timed = native::PolicyTiming {
            detailed: true,
            encode: true,
            clock: now,
            ..Default::default()
        };
        let mut plain = Vec::new();
        let mut encoded = Vec::new();
        assert_eq!(native::run(&c, &mut plain, || None).unwrap(), 0);
        assert_eq!(
            native::run_instrumented(
                &c,
                &mut encoded,
                || None,
                &mut |_, _| Ok(()),
                &mut |_| {},
                Some(&mut timed)
            )
            .unwrap(),
            0
        );
        assert_eq!(
            plain, encoded,
            "extra actual JSON encoding and timers must not alter native output"
        );
        assert_eq!(
            timed.elapsed_ns, 21,
            "policy initialization plus two choices"
        );
        assert_eq!(timed.phases.reset_ns, 7);
        assert_eq!(timed.phases.legality_and_view_ns, 14);
        assert_eq!(timed.phases.encoding_ns, 14);
        assert_eq!(timed.phases.finalization_ns, 7);
        assert!(timed.phases.transition_ns >= 14);
        assert!(timed.phases.encoded_bytes > 0);
        // Reset failure is timed too; its attempt cannot become a completion.
        c.game.format = "invalid".into();
        let mut timed = native::PolicyTiming {
            detailed: true,
            ..Default::default()
        };
        assert_eq!(
            native::run_instrumented(
                &c,
                &mut Vec::new(),
                || None,
                &mut |_, _| Ok(()),
                &mut |_| {},
                Some(&mut timed)
            )
            .unwrap(),
            3
        );
        assert!(timed.phases.reset_ns > 0);
        let failure = attempt(&c, Rc::new(|| 0), u128::MAX, &AtomicUsize::new(0), false);
        assert_eq!(
            (failure.started, failure.completed, failure.failed),
            (1, 0, 1)
        );
    }
    #[test]
    fn benchmark_real_window_boundary_retains_unfinished_and_terminal_game() {
        let config: Config = serde_json::from_value(input()).unwrap();
        let mut c = config.run_config().unwrap();
        c.policies = [
            mtg_policy::HEURISTIC_VERSION.into(),
            mtg_policy::HEURISTIC_VERSION.into(),
        ];
        let clock = Rc::new(Cell::new(0u128));
        let now: Rc<dyn Fn() -> u128> = {
            let clock = clock.clone();
            Rc::new(move || {
                let n = clock.get();
                clock.set(n + 1);
                n
            })
        };
        let w = attempt(&c, now, 10, &AtomicUsize::new(0), false);
        assert_eq!(
            (w.started, w.completed, w.unfinished, w.failed),
            (1, 0, 1, 0)
        );
        assert_eq!(w.rate(), None);
        let w = attempt(&c, Rc::new(|| 0), u128::MAX, &AtomicUsize::new(0), false);
        assert_eq!(
            (w.started, w.completed, w.unfinished, w.failed),
            (1, 1, 0, 0)
        );
        assert_eq!(w.wins.iter().sum::<u64>() + w.draws, 1);
        assert!(w.decisions > 2);
        assert_eq!(w.counters.as_ref().unwrap()["rules_completed"], 1);
        assert_eq!(w.counters.as_ref().unwrap()["decisions"], w.decisions);
        assert!(
            w.counters.as_ref().unwrap()["logical_actions"]
                .as_u64()
                .unwrap()
                > 0
        );
    }
    #[test]
    fn benchmark_signal_before_warmup_retains_metadata_and_no_success() {
        let (code, v) = run(
            &serde_json::to_vec(&input()).unwrap(),
            &AtomicUsize::new(signal_hook::consts::SIGTERM as usize),
        )
        .unwrap();
        assert_eq!(code, 143);
        assert_eq!(v["status"], "incomplete-or-failed");
        assert_eq!(v["warmup"]["started"], 0);
        assert_eq!(v["windows"], json!([]));
        assert_eq!(v["completed_games_per_second"]["mean"], Value::Null);
        for pin in [
            "deck_source_sha256",
            "card_manifest_sha256",
            "rules_manifest_sha256",
            "policy_source_sha256",
            "workload_source_sha256",
            "fixture_sha256",
        ] {
            assert_eq!(v["pins"][pin].as_str().unwrap().len(), 64, "{pin}");
        }
        assert!(!v["hardware"]["rustc"].as_str().unwrap().is_empty());
        assert_eq!(v["hardware"]["workers"], 1);
    }
    #[test]
    fn benchmark_repeated_windows_are_raw_and_zero_completion_run_fails() {
        let clock = Rc::new(Cell::new(0u128));
        let now: Rc<dyn Fn() -> u128> = {
            let clock = clock.clone();
            Rc::new(move || {
                let n = clock.get();
                clock.set(n + 1_000_000_000);
                n
            })
        };
        let (code, v) = run_with_clock(
            &serde_json::to_vec(&input()).unwrap(),
            &AtomicUsize::new(0),
            now,
        )
        .unwrap();
        assert_eq!(code, 3);
        assert_eq!(v["status"], "incomplete-or-failed");
        let windows = v["windows"].as_array().unwrap();
        assert_eq!(windows.len(), 5);
        assert!(v["warmup"]["elapsed_ns"].as_u64().unwrap() >= 10_000_000_000);
        let mut next = v["warmup"]["attempts"].as_u64().unwrap();
        for w in windows {
            assert!(w["elapsed_ns"].as_u64().unwrap() >= 30_000_000_000);
            assert_eq!(w["first_episode"], next);
            next += w["attempts"].as_u64().unwrap();
            assert_eq!(w["completed"], 0);
            assert_eq!(w["completed_games_per_second"], Value::Null);
            assert_eq!(w["phases"]["encoding_ns"], Value::Null);
            assert!(w["unfinished"].as_u64().unwrap() > 0);
            assert_eq!(w["started"], w["unfinished"]);
        }
        assert_eq!(
            v["completed_games_per_second"]["samples"],
            json!(vec![0.0; 5])
        );
        assert_eq!(v["distributions"]["encoding_ns"], Value::Null);
    }
    #[test]
    fn benchmark_command_routes_version_and_rejects_malformed_before_output() {
        use std::{ffi::OsString, fs};
        let root =
            std::env::temp_dir().join(format!("mtg-benchmark-contract-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let source = root.join("config.json");
        let output = root.join("report.json");
        let args = vec![
            OsString::from("bench"),
            "--workload".into(),
            WORKLOAD.into(),
            "--config".into(),
            source.clone().into_os_string(),
            "--output".into(),
            output.clone().into_os_string(),
        ];
        for bytes in [b"{".as_slice(), b"null", b"{}"] {
            fs::write(&source, bytes).unwrap();
            assert_eq!(
                crate::commands::execute(&args, &AtomicUsize::new(0))
                    .unwrap_err()
                    .0,
                2
            );
            assert!(!output.exists());
        }
        fs::write(&source, serde_json::to_vec(&input()).unwrap()).unwrap();
        assert_eq!(
            crate::commands::execute(
                &args,
                &AtomicUsize::new(signal_hook::consts::SIGINT as usize)
            )
            .unwrap(),
            130
        );
        let report: Value = serde_json::from_slice(&fs::read(&output).unwrap()).unwrap();
        assert_eq!(report["schema_version"], 1);
        assert_eq!(report["workload"], WORKLOAD);
        assert_eq!(report["status"], "incomplete-or-failed");
        fs::remove_dir_all(root).unwrap();
    }
}
