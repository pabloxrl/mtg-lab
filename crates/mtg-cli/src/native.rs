//! Capture-disabled native client of the authoritative episode owner.
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
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub policy_seed: u64,
    pub rng_version: String,
    pub work_quantum: NonZeroUsize,
    pub max_work_calls: u64,
    pub max_records: NonZeroUsize,
}
impl Config {
    pub fn validate(&self, policies: &[String; 2]) -> Result<(), String> {
        if self.max_work_calls == 0 || self.rng_version != mtg_policy::RNG_VERSION {
            return Err("unsupported native RNG version or zero work bound".into());
        }
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
    mut control: impl FnMut() -> Option<Stop>,
) -> io::Result<i32> {
    let n = c
        .native
        .as_ref()
        .ok_or_else(|| io::Error::other("native configuration missing"))?;
    emit(
        w,
        json!({"type":"run","config":c,"config_sha256":hash(&serde_json::to_vec(c)?),"engine_sha256":env!("MTG_ENGINE_SHA256"),"cli_version":env!("CARGO_PKG_VERSION"),"policies":c.policies,"policy_rng_version":n.rng_version,"rules_sha256":hash(include_bytes!("../../../data/rules/cr-2026-09-25.json")),"cards_sha256":hash(include_bytes!("../../../data/cards/foundations_micro_v1.json")),"workers":1,"instrumentation":"summary-v1","capture":"none","execution":"owned-native-v1"}),
    )?;
    let mut counts = Counts {
        requested: c.episodes,
        ..Counts::default()
    };
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
        let mut d = Driver::bounded(
            256,
            Budget {
                limits: Limits {
                    decisions: Some(c.max_decisions),
                    wall_time_ms: Some(1),
                    turns: None,
                },
                work_quantum: n.work_quantum,
                records: n.max_records,
            },
            Box::new(Deadline(clock.clone())),
        )
        .map_err(|_| io::Error::other("owner initialization failed"))?;
        let mut policies = [
            Policy::new(&c.policies[0], n, episode, 0),
            Policy::new(&c.policies[1], n, episode, 1),
        ];
        counts.started += 1;
        let mut work_calls = 1;
        let reset = d.reset(&c.game, c.master_seed, episode, n.work_quantum);
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
                    clock.set(1);
                }
                break;
            }
            if work_calls >= n.max_work_calls {
                reason = "work_limit";
                break;
            }
            work_calls += 1;
            let result = match progress {
                Progress::InternalYield => d.advance(n.work_quantum),
                Progress::Ready => {
                    // Only authorized policy observations cross into a policy.
                    let ready = [Seat::P0, Seat::P1].into_iter().find_map(|seat| {
                        d.observe(seat)
                            .ok()
                            .filter(|o| o.decision.is_some())
                            .map(|o| (seat, o))
                    });
                    match ready {
                        Some((seat, o)) => {
                            let index = usize::from(seat == Seat::P1);
                            let submission = policies[index]
                                .as_mut()
                                .map_err(|_| ())
                                .and_then(|p| p.choose(&o).map_err(|_| ()));
                            match submission {
                                Ok(s) => d.submit(seat, &s).map(|()| Progress::InternalYield),
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
        let result = d.finish();
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
        let owned = result.as_ref().ok();
        let view = owned
            .and_then(|r| r.final_observations())
            .map(|o| &o[0].view);
        emit(
            w,
            json!({"type":"episode","episode":episode,"status":status,"reason":reason,"winner":winner,"decisions":owned.map(|r|r.accepted_decisions()),"work_calls":work_calls,"life":view.map(|v|v.life),"turn":view.and_then(|v|v.turn.map(|t|t.0)),"turn_position":view.and_then(|v|v.turn),"hand_counts":view.map(|v|v.hand_counts),"library_counts":view.map(|v|v.library_counts),"public_zones":view.map(|v|&v.public_zones),"terminal":view.and_then(|v|v.terminal.as_ref()),"history_sha256":owned.map(|r|hash(&serde_json::to_vec(r.privileged_history()).unwrap())),"caller_error":caller_error}),
        )?;
        if exit != 0 {
            break;
        }
    }
    counts.not_started = counts.requested - counts.started;
    let mut summary = serde_json::to_value(counts)?;
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
                assert_eq!(
                    run(&config(), &mut output, || {
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
