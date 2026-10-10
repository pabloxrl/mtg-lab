//! Native/script client of the authoritative episode owner.
use crate::simulate::{self, Stop, emit, hash};
use mtg_core::{
    episode::{Budget, Clock, Driver, Failure, Progress, Status},
    game::policy::{Observation, Submission},
    objects::Seat,
    trajectory::{Limit, Limits},
};
use mtg_policy::{Heuristic, LegalRandom};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    cell::Cell,
    io::{self, Write},
    num::NonZeroUsize,
    rc::Rc,
    time::Instant,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, skip_serializing_if = "mtg_core::metrics::Mode::is_off")]
    pub instrumentation: mtg_core::metrics::Mode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<mtg_core::metrics::TraceConfig>,
    pub policy_seed: u64,
    pub rng_version: String,
    pub work_quantum: NonZeroUsize,
    pub max_work_calls: u64,
    pub max_records: NonZeroUsize,
}
impl Config {
    pub fn validate_bounds(&self) -> Result<(), String> {
        if self.trace.is_some_and(|t| {
            t.capacity > 65_536 || self.instrumentation != mtg_core::metrics::Mode::SampledTrace
        }) {
            return Err("trace configuration requires sampled_trace and capacity <= 65536".into());
        }
        if self.max_work_calls == 0 || self.rng_version != mtg_policy::RNG_VERSION {
            return Err("unsupported native RNG version or zero work bound".into());
        }
        Ok(())
    }
    pub fn validate(&self, policies: &[String; 2]) -> Result<(), String> {
        self.validate_bounds()?;
        if policies
            .iter()
            .any(|p| ![mtg_policy::VERSION, mtg_policy::HEURISTIC_VERSION].contains(&p.as_str()))
        {
            return Err("unsupported native policy; no fallback".into());
        }
        Ok(())
    }
}
// The run's control function owns the monotonic deadline (and is injectable).
// Publish expiry to the owner's clock before finish; the owner decides terminal
// versus decision/time truncation precedence. Rules never read wall time.
#[derive(Debug)]
struct Deadline(Rc<Cell<u64>>);
impl Clock for Deadline {
    fn now_ms(&self) -> u64 {
        self.0.get()
    }
}
enum Policy {
    Random(LegalRandom),
    Heuristic(Heuristic),
}
impl Policy {
    fn new(id: &str, c: &Config, episode: u64, seat: u8) -> Result<Self, mtg_policy::Error> {
        match id {
            mtg_policy::VERSION => {
                LegalRandom::new(id, &c.rng_version, c.policy_seed, episode, seat).map(Self::Random)
            }
            mtg_policy::HEURISTIC_VERSION => Heuristic::new(id, seat).map(Self::Heuristic),
            _ => Err(mtg_policy::Error::UnsupportedVersion),
        }
    }
    fn choose(&mut self, o: &Observation) -> Result<Submission, mtg_policy::Error> {
        match self {
            Self::Random(p) => p.choose(o),
            Self::Heuristic(p) => p.choose(o),
        }
    }
}
#[derive(Default, Serialize)]
struct Counts {
    requested: u64,
    started: u64,
    not_started: u64,
    completed: u64,
    truncated: u64,
    failed: u64,
    incomplete: u64,
    wins: [u64; 2],
    draws: u64,
}

pub fn run(
    c: &simulate::Config,
    w: &mut impl Write,
    control: impl FnMut() -> Option<Stop>,
) -> io::Result<i32> {
    run_observed(c, w, control, &mut |_, _| Ok(()), &mut |_| {})
}

pub(crate) fn run_observed(
    c: &simulate::Config,
    w: &mut impl Write,
    control: impl FnMut() -> Option<Stop>,
    hook: &mut mtg_recorder::FileHook<'_>,
    observe: &mut impl FnMut(&mtg_core::episode::EpisodeResult),
) -> io::Result<i32> {
    run_instrumented(c, w, control, hook, observe, None)
}

/// Optional timing for the benchmark client only; ordinary simulation does not
/// read a performance clock for each choice. Includes policy initialization and
/// all choose attempts (including failures), excludes observation/submission.
type StateProbe = dyn FnMut(&Driver, &Observation);
/// Privileged benchmark-only byte retention, never serialized into public output.
pub(crate) struct RetainedReplay {
    pub max_bytes: usize,
    pub bytes: Option<Vec<u8>>,
    pub verified: u64,
    pub failed: u64,
    pub incomplete: u64,
    pub elapsed_ns: u128,
}
impl RetainedReplay {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            max_bytes,
            bytes: None,
            verified: 0,
            failed: 0,
            incomplete: 0,
            elapsed_ns: 0,
        }
    }
}
pub(crate) struct PolicyTiming {
    pub sample: Option<Box<StateProbe>>,
    pub replay: Option<RetainedReplay>,
    pub publication_ns: u128,
    pub elapsed_ns: u128,
    pub phases: Phases,
    pub detailed: bool,
    pub encode: bool,
    pub clock: Rc<dyn Fn() -> u128>,
}
#[derive(Default, Clone, Debug, Serialize)]
pub(crate) struct Phases {
    pub reset_ns: u128,
    pub transition_ns: u128,
    pub legality_and_view_ns: u128,
    pub encoding_ns: u128,
    pub finalization_ns: u128,
    pub encoded_bytes: u64,
}
impl Default for PolicyTiming {
    fn default() -> Self {
        let start = Instant::now();
        Self {
            sample: None,
            replay: None,
            publication_ns: 0,
            elapsed_ns: 0,
            phases: Phases::default(),
            detailed: false,
            encode: false,
            clock: Rc::new(move || start.elapsed().as_nanos()),
        }
    }
}
fn stamp(t: &Option<&mut PolicyTiming>, detailed: bool) -> Option<u128> {
    t.as_ref()
        .filter(|t| !detailed || t.detailed)
        .map(|t| (t.clock)())
}
fn elapsed(t: &Option<&mut PolicyTiming>, start: Option<u128>) -> u128 {
    start.map_or(0, |start| (t.as_ref().unwrap().clock)() - start)
}

pub(crate) fn run_instrumented(
    c: &simulate::Config,
    w: &mut impl Write,
    mut control: impl FnMut() -> Option<Stop>,
    hook: &mut mtg_recorder::FileHook<'_>,
    observe: &mut impl FnMut(&mtg_core::episode::EpisodeResult),
    mut timing: Option<&mut PolicyTiming>,
) -> io::Result<i32> {
    let mut capture = c
        .capture
        .as_ref()
        .map(|capture| crate::capture::Session::new(c, capture))
        .transpose()?;
    let n = c
        .native
        .as_ref()
        .ok_or_else(|| io::Error::other("native configuration missing"))?;
    let mut resolved = serde_json::to_value(c)?;
    if c.script.is_some() {
        resolved["script"]
            .as_object_mut()
            .unwrap()
            .remove("records");
    }
    // Root paths and the local declaration are never public replay grants.
    if let Some(config) = &c.capture {
        resolved["capture"] = json!({"mode":"canonical-v2","max_episodes":config.max_episodes,"backpressure":config.backpressure,"queue_bytes":config.queue_bytes,"max_bytes":config.max_bytes});
    }
    let mut cursor = 0;
    emit(
        w,
        json!({"type":"run","config":resolved,"config_sha256":hash(&serde_json::to_vec(c)?),"engine_sha256":env!("MTG_ENGINE_SHA256"),"cli_version":env!("CARGO_PKG_VERSION"),"policies":c.policies,"policy_rng_version":n.rng_version,"rules_sha256":hash(include_bytes!("../../../data/rules/cr-2026-09-25.json")),"cards_sha256":hash(include_bytes!("../../../data/cards/foundations_micro_v1.json")),"workers":1,"instrumentation":n.instrumentation,"capture":if capture.is_some() {"canonical-v2"} else {"none"},"execution":if c.script.is_some() {"owned-script-v1"} else {"owned-native-v1"},"script_privacy":c.script.as_ref().map(|_|"privileged")}),
    )?;
    let mut counts = Counts {
        requested: c.episodes,
        ..Counts::default()
    };
    let mut metrics = (n.instrumentation != mtg_core::metrics::Mode::Off)
        .then(mtg_core::metrics::Counters::default);
    let mut exit = 0;
    let mut run_reason = "budget_complete";
    for offset in 0..c.episodes {
        if let Some(stop) = control() {
            exit = stop.code();
            run_reason = stop.reason();
            break;
        }
        let episode = c
            .first_episode
            .checked_add(offset)
            .ok_or_else(|| io::Error::other("episode overflow"))?;
        let clock = Rc::new(Cell::new(0));
        let mut d = Driver::bounded_instrumented(
            256,
            Budget {
                limits: Limits {
                    decisions: Some(c.max_decisions),
                    wall_time_ms: c.deadline_ms,
                    turns: None,
                },
                work_quantum: n.work_quantum,
                records: n.max_records,
            },
            Box::new(Deadline(clock.clone())),
            n.instrumentation,
        )
        .map_err(|_| io::Error::other("owner initialization failed"))?;
        if let Some(trace) = n.trace {
            d.set_trace_config(trace)
                .map_err(|_| io::Error::other("invalid trace configuration"))?;
        }
        let episode_start = cursor;
        let policy_start = stamp(&timing, false);
        let mut policies = c.script.is_none().then(|| {
            [
                Policy::new(&c.policies[0], n, episode, 0),
                Policy::new(&c.policies[1], n, episode, 1),
            ]
        });
        let duration = elapsed(&timing, policy_start);
        if let Some(timing) = timing.as_mut() {
            timing.elapsed_ns += duration;
        }
        counts.started += 1;
        let mut work_calls = 1;
        let reset_start = stamp(&timing, true);
        let reset = if let Some(session) = &capture {
            let header = session
                .run
                .header(episode)
                .map_err(|_| io::Error::other("capture header failed"))?;
            d.reset_captured(&c.game, c.master_seed, episode, n.work_quantum, &header)
        } else {
            d.reset(&c.game, c.master_seed, episode, n.work_quantum)
        };
        let duration = elapsed(&timing, reset_start);
        if let Some(timing) = timing.as_mut() {
            timing.phases.reset_ns += duration;
        }
        let mut caller_error = None;
        let mut reason = "abandoned";
        let mut progress = match reset {
            Ok(p) => p,
            Err(_) => {
                caller_error = Some("reset_error");
                Progress::Ready
            }
        };
        loop {
            if caller_error.is_some() || d.status().is_some() {
                break;
            }
            if let Some(stop) = control() {
                exit = stop.code();
                run_reason = stop.reason();
                reason = stop.reason();
                if matches!(stop, Stop::Deadline) {
                    clock.set(c.deadline_ms.unwrap_or(0));
                }
                break;
            }
            if work_calls >= n.max_work_calls {
                reason = "work_limit";
                break;
            }
            work_calls += 1;
            let result = match progress {
                Progress::InternalYield => {
                    let start = stamp(&timing, true);
                    let result = d.advance(n.work_quantum);
                    let duration = elapsed(&timing, start);
                    if let Some(t) = timing.as_mut() {
                        t.phases.transition_ns += duration;
                    }
                    result
                }
                Progress::Ready if c.script.is_some() => {
                    match crate::script::submit(
                        c.script.as_ref().unwrap(),
                        &mut cursor,
                        episode,
                        &mut d,
                    ) {
                        Ok(()) => Ok(Progress::InternalYield),
                        Err(e) => {
                            caller_error = Some(e);
                            break;
                        }
                    }
                }
                Progress::Ready => {
                    // Only authorized policy observations cross into a policy.
                    let start = stamp(&timing, true);
                    let ready = [Seat::P0, Seat::P1].into_iter().find_map(|seat| {
                        d.observe(seat)
                            .ok()
                            .filter(|o| o.decision.is_some())
                            .map(|o| (seat, o))
                    });
                    let duration = elapsed(&timing, start);
                    if let Some(t) = timing.as_mut() {
                        t.phases.legality_and_view_ns += duration;
                    }
                    match ready {
                        Some((seat, o)) => {
                            if let Some(probe) = timing.as_mut().and_then(|t| t.sample.as_mut()) {
                                probe(&d, &o);
                            }
                            let index = usize::from(seat == Seat::P1);
                            if timing.as_ref().is_some_and(|t| t.encode) {
                                let start = stamp(&timing, true);
                                let bytes = serde_json::to_vec(&o);
                                let duration = elapsed(&timing, start);
                                let t = timing.as_mut().unwrap();
                                t.phases.encoding_ns += duration;
                                t.phases.encoded_bytes += std::hint::black_box(bytes?).len() as u64;
                            }
                            let policy_start = stamp(&timing, false);
                            let submission = policies.as_mut().unwrap()[index]
                                .as_mut()
                                .map_err(|_| ())
                                .and_then(|p| p.choose(&o).map_err(|_| ()));
                            let duration = elapsed(&timing, policy_start);
                            if let Some(timing) = timing.as_mut() {
                                timing.elapsed_ns += duration;
                            }
                            match submission {
                                Ok(s) => {
                                    let start = stamp(&timing, true);
                                    let result =
                                        d.submit(seat, &s).map(|()| Progress::InternalYield);
                                    let duration = elapsed(&timing, start);
                                    if let Some(t) = timing.as_mut() {
                                        t.phases.transition_ns += duration;
                                    }
                                    result
                                }
                                Err(()) => {
                                    caller_error = Some("policy_error");
                                    break;
                                }
                            }
                        }
                        None => {
                            caller_error = Some("observation_error");
                            break;
                        }
                    }
                }
                Progress::Terminal(_) | Progress::Stopped(_) => break,
            };
            match result {
                Ok(p) => progress = p,
                Err(_) => {
                    if d.status().is_none() {
                        caller_error = Some("driver_error");
                    }
                    break;
                }
            }
        }
        if matches!(d.status(), Some(Status::Completed(_)))
            && c.script.as_ref().is_some_and(|s| {
                s.records
                    .get(cursor)
                    .is_some_and(|e| e.episode == episode || offset + 1 == c.episodes)
            })
        {
            caller_error = Some("script_extra");
        }
        let start = stamp(&timing, true);
        let result = d.finish();
        let duration = elapsed(&timing, start);
        if let Some(t) = timing.as_mut() {
            t.phases.finalization_ns += duration;
        }
        // Replay bytes are privileged. Validate complete export here but never
        // send it to public JSONL; durable publication uses the existing capture path.
        let replay_start = if timing.as_ref().is_some_and(|t| t.replay.is_some())
            && n.instrumentation == mtg_core::metrics::Mode::FullReplay
        {
            stamp(&timing, true)
        } else {
            None
        };
        let replay_status = if n.instrumentation == mtg_core::metrics::Mode::FullReplay {
            Some(match &result {
                Ok(r) if matches!(r.status(), Status::Completed(_)) => {
                    let max = timing.as_ref().and_then(|t| t.replay.as_ref()).map_or_else(
                        || c.capture.as_ref().map_or(67_108_864, |c| c.max_bytes),
                        |r| r.max_bytes,
                    );
                    match r.privileged_replay(max) {
                        Ok(Some(bytes)) => {
                            if let Some(r) = timing.as_mut().and_then(|t| t.replay.as_mut()) {
                                r.verified += 1;
                                r.bytes = Some(bytes);
                            }
                            "available_in_memory"
                        }
                        _ => {
                            if let Some(r) = timing.as_mut().and_then(|t| t.replay.as_mut()) {
                                r.failed += 1;
                            }
                            caller_error = Some("replay_recording_error");
                            "failed"
                        }
                    }
                }
                _ => {
                    if let Some(r) = timing.as_mut().and_then(|t| t.replay.as_mut()) {
                        r.incomplete += 1;
                    }
                    "incomplete"
                }
            })
        } else {
            None
        };
        let replay_duration = elapsed(&timing, replay_start);
        if let Some(r) = timing.as_mut().and_then(|t| t.replay.as_mut()) {
            r.elapsed_ns += replay_duration;
        }
        let (status, winner) = match result.as_ref().map(|r| r.status()) {
            Ok(Status::Completed(o)) => {
                counts.completed += 1;
                let winner = o.winner.map(|s| usize::from(s == Seat::P1));
                if let Some(s) = winner {
                    counts.wins[s] += 1;
                } else {
                    counts.draws += 1;
                }
                reason = "rules_terminal";
                ("completed", winner)
            }
            Ok(Status::Truncated(limit)) => {
                counts.truncated += 1;
                reason = match limit {
                    Limit::Decisions => "decision_limit",
                    Limit::Turns => "turn_limit",
                    Limit::WallTime => "deadline",
                };
                ("truncated", None)
            }
            Ok(Status::Failed(f)) => {
                counts.failed += 1;
                reason = match f {
                    Failure::RecordCapacity => "record_capacity",
                    Failure::Capacity => "capacity",
                    Failure::Clock => "clock_error",
                    Failure::Recording => "recording_error",
                };
                exit = 3;
                run_reason = reason;
                ("failed", None)
            }
            Ok(Status::Incomplete) if c.script.is_some() && reason == "work_limit" => {
                // Caller work bounds do not alter the owner's game status. This
                // run reports its external truncation and retains owner_status.
                counts.truncated += 1;
                ("truncated", None)
            }
            Ok(Status::Incomplete) | Err(_) => {
                counts.incomplete += 1;
                if result.is_err() {
                    caller_error = Some("finalization_error");
                }
                ("incomplete", None)
            }
        };
        if let Some(error) = caller_error {
            exit = 3;
            run_reason = error;
            reason = error;
        }
        if let (Some(total), Some(mut local)) = (&mut metrics, d.metrics()) {
            // A caller/script failure invalidates this run attempt even when
            // the rules owner reached its own terminal boundary.
            if caller_error.is_some() && local.failed == 0 {
                local.completed = 0;
                local.rules_completed = 0;
                local.truncated = 0;
                local.incomplete = 0;
                local.failed = 1;
            } else if status == "truncated" && local.incomplete == 1 {
                local.incomplete = 0;
                local.truncated = 1;
            }
            total.merge(&local);
        }
        let owned = result.as_ref().ok();
        let view = owned
            .and_then(|r| r.final_observations())
            .map(|o| &o[0].view);
        let mut row = json!({"type":"episode","episode":episode,"status":status,"reason":reason,"winner":winner,"decisions":owned.map(|r|r.accepted_decisions()),"work_calls":work_calls,"life":view.map(|v|v.life),"turn":view.and_then(|v|v.turn.map(|t|t.0)),"turn_position":view.and_then(|v|v.turn),"hand_counts":view.map(|v|v.hand_counts),"library_counts":view.map(|v|v.library_counts),"public_zones":view.map(|v|&v.public_zones),"terminal":view.and_then(|v|v.terminal.as_ref()),"history_sha256":owned.map(|r|hash(&serde_json::to_vec(r.privileged_history()).unwrap())),"caller_error":caller_error,"owner_status":owned.map(|r| match r.status() {Status::Completed(_)=>"completed",Status::Truncated(_)=>"truncated",Status::Failed(_)=>"failed",Status::Incomplete=>"incomplete"}),"script_consumed":c.script.as_ref().map(|_|cursor-episode_start),"script_status":c.script.as_ref().map(|_|if caller_error.is_some() {"error"} else if status=="completed" {"complete"} else if status=="truncated" {"truncated"} else {"incomplete"})});
        if let Some(trace) = d.diagnostic_trace() {
            row["diagnostics"] = serde_json::to_value(trace)?;
        }
        if let Some(status) = replay_status {
            row["replay_status"] = json!(status);
        }
        emit(w, row)?;
        if let Ok(result) = result {
            observe(&result);
            if let Some(session) = &mut capture {
                session.results.push(result);
            }
        }
        if exit != 0 || (c.script.is_some() && status != "completed") {
            if exit == 0 && c.script.is_some() {
                run_reason = reason;
            }
            break;
        }
    }
    let mut publication = None;
    if let Some(session) = &mut capture {
        let publication_start = stamp(&timing, true);
        let (row, stop, failed) = session.publish(
            c.capture.as_ref().unwrap(),
            counts.started,
            &mut control,
            hook,
        );
        let duration = elapsed(&timing, publication_start);
        if let Some(t) = timing.as_mut() {
            t.publication_ns += duration;
        }
        if let Some(stop) = stop {
            exit = stop.code();
            run_reason = stop.reason();
        } else if failed && exit == 0 {
            exit = 3;
            run_reason = "publication_failed";
        }
        publication = Some(row["status"].clone());
        emit(w, row)?;
    }
    counts.not_started = counts.requested - counts.started;
    let mut summary = serde_json::to_value(counts)?;
    if let Some(script) = &c.script {
        summary["script_consumed"] = json!(cursor);
        summary["script_remaining"] = json!(script.records.len() - cursor);
    }
    if let Some(status) = publication {
        summary["publication"] = status;
    }
    if let Some(metrics) = &metrics {
        summary["metrics"] = serde_json::to_value(metrics.report())?;
    }
    summary["type"] = json!("summary");
    summary["reason"] = json!(run_reason);
    summary["exit_code"] = json!(exit);
    emit(w, summary)?;
    Ok(exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    fn config() -> simulate::Config {
        serde_json::from_value(json!({"schema_version":2,"game":mtg_core::game::Config::default(),"policies":[mtg_policy::HEURISTIC_VERSION,mtg_policy::HEURISTIC_VERSION],"master_seed":42,"first_episode":0,"episodes":3,"max_decisions":3000,"deadline_ms":null,"native":{"policy_seed":42,"rng_version":mtg_policy::RNG_VERSION,"work_quantum":1000,"max_work_calls":100000,"max_records":10000}})).unwrap()
    }
    fn rows(bytes: &[u8]) -> Vec<Value> {
        String::from_utf8_lossy(bytes)
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect()
    }
    #[test]
    fn native_deterministic_prestart_and_midgame_stop_deadline_accounting() {
        for (stop, code, status, field) in [
            (Stop::Sigterm, 143, "incomplete", "incomplete"),
            (Stop::Sigint, 130, "incomplete", "incomplete"),
            (Stop::Deadline, 4, "truncated", "truncated"),
        ] {
            for poll in [1, 3, 200] {
                let mut output = vec![];
                let mut polls = 0;
                let mut c = config();
                // Exercise stop accounting inside the versioned policy's supported
                // pool. Red now exposes Pyromancer targets, owned by scripted play
                // until full-pool native policy delivery (#208).
                c.game.seats = vec![mtg_core::game::DeckConfig::new("green"); 2];
                // Inject expiry of an explicitly configured deadline. Production
                // control cannot produce Deadline when the config disables it.
                if matches!(stop, Stop::Deadline) {
                    c.deadline_ms = Some(7);
                }
                assert_eq!(
                    run(&c, &mut output, || {
                        polls += 1;
                        (polls == poll).then_some(stop)
                    })
                    .unwrap(),
                    code
                );
                let r = rows(&output);
                let s = r.last().unwrap();
                assert_eq!(s["reason"], stop.reason());
                assert_eq!(s["completed"], 0);
                assert_eq!(s["failed"], 0);
                assert_eq!(s["draws"], 0);
                if poll == 1 {
                    assert_eq!(r.len(), 2);
                    assert_eq!(s["started"], 0);
                    assert_eq!(s["not_started"], 3);
                } else {
                    assert_eq!(r.len(), 3);
                    assert_eq!(s["started"], 1);
                    assert_eq!(s["not_started"], 2);
                    assert_eq!(s[field], 1);
                    assert_eq!(r[1]["status"], status);
                    assert_eq!(r[1]["reason"], stop.reason());
                    assert_eq!(r[1]["winner"], Value::Null);
                    assert_eq!(r[1]["work_calls"], poll - 1);
                    if poll == 3 {
                        assert_eq!(r[1]["decisions"], 1);
                    } else {
                        assert!(r[1]["decisions"].as_u64().unwrap() > 2);
                    }
                }
            }
        }
    }
    #[test]
    fn native_synthetic_policy_failure_is_incomplete_caller_error_not_rules_failure() {
        // Deliberately bypass public validation to fault-inject an unavailable
        // policy at runtime. Production rejects it before the run starts.
        let mut c = config();
        c.policies = ["unavailable".into(), "unavailable".into()];
        let mut output = vec![];
        assert_eq!(run(&c, &mut output, || None).unwrap(), 3);
        let r = rows(&output);
        assert_eq!(r[1]["status"], "incomplete");
        assert_eq!(r[1]["caller_error"], "policy_error");
        assert_eq!(r[1]["decisions"], 0);
        assert_eq!(r[2]["incomplete"], 1);
        assert_eq!(r[2]["not_started"], 2);
        assert_eq!(r[2]["failed"], 0);
        assert_eq!(r[2]["draws"], 0);
    }
    #[test]
    fn native_output_failure_at_header_episode_and_summary_never_returns_success() {
        struct Broken(usize);
        impl Write for Broken {
            fn write(&mut self, b: &[u8]) -> io::Result<usize> {
                if self.0 == 0 {
                    return Err(io::Error::other("synthetic write failure"));
                }
                let n = self.0.min(b.len());
                self.0 -= n;
                Ok(n)
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut c = config();
        c.max_decisions = 1;
        let mut output = vec![];
        run(&c, &mut output, || None).unwrap();
        for limit in [0, 1, output.len() / 2, output.len() - 1] {
            assert!(run(&c, &mut Broken(limit), || None).is_err());
        }
        struct Flush;
        impl Write for Flush {
            fn write(&mut self, b: &[u8]) -> io::Result<usize> {
                Ok(b.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("synthetic flush failure"))
            }
        }
        assert!(run(&c, &mut Flush, || None).is_err());
    }
    #[test]
    fn native_maximum_episode_count_prestart_stop_does_not_allocate_or_overflow() {
        let mut c = config();
        c.episodes = u64::MAX;
        c.validate().unwrap();
        let mut output = vec![];
        assert_eq!(run(&c, &mut output, || Some(Stop::Sigterm)).unwrap(), 143);
        let r = rows(&output);
        assert_eq!(r[1]["not_started"], u64::MAX);
        assert_eq!(r[1]["started"], 0);
    }
}
