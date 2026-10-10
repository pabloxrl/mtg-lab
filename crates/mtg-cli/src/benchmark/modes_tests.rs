use super::*;
use mtg_core::metrics::Mode;
use sha2::{Digest, Sha256};
use std::{
    cell::{Cell, RefCell},
    fs,
    path::PathBuf,
    sync::atomic::AtomicU64,
};

fn config(mode: Mode) -> Config {
    serde_json::from_value(json!({"schema_version":2,"workload":FOUR_MODES,
        "warmup_seconds":10,"window_seconds":30,"windows":5,
        "policies":[mtg_policy::HEURISTIC_VERSION,mtg_policy::HEURISTIC_VERSION],"master_seed":42,"policy_seed":42,
        "instrumentation":mode,"encoding":true})).unwrap()
}
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Roots(PathBuf);
impl Roots {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "mtg-four-modes-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("data")).unwrap();
        fs::create_dir_all(root.join("replay")).unwrap();
        Self(root)
    }
    fn capture(&self) -> crate::capture::Config {
        serde_json::from_value(
            json!({"dataset_root":self.0.join("data"),"replay_root":self.0.join("replay"),
            "authorization":"local-owner-v1","max_episodes":1,"backpressure":"fail",
            "queue_bytes":67108864,"max_bytes":67108864}),
        )
        .unwrap()
    }
    fn validate(&self) -> usize {
        use mtg_recorder::manifest::{LoadMode, Manifest};
        let mut count = 0;
        for path in fs::read_dir(self.0.join("data")).unwrap() {
            let path = path.unwrap().path();
            let m = Manifest::parse(&fs::read(path.join("manifest.json")).unwrap()[..], 67108864)
                .unwrap();
            let bytes = fs::read(path.join("episodes.jsonl")).unwrap();
            let loaded = m
                .load_v2(
                    "episodes.jsonl",
                    &bytes[..],
                    &m.versions,
                    67108864,
                    LoadMode::CompletedOnly,
                )
                .unwrap();
            assert_eq!(loaded.episodes().len(), 1);
            count += 1;
        }
        count
    }
}
impl Drop for Roots {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn clock(step: u128) -> Rc<dyn Fn() -> u128> {
    let n = Cell::new(0);
    Rc::new(move || {
        let old = n.get();
        n.set(old + step);
        old
    })
}

#[test]
fn four_modes_export_native_contract() {
    let mut rows = Vec::new();
    let mut baselines = vec![None; 8];
    for mode in [
        Mode::Off,
        Mode::Counters,
        Mode::SampledTrace,
        Mode::FullReplay,
    ] {
        for capture in [false, true] {
            for row in 0..8 {
                let roots = Roots::new();
                let mut cfg = config(mode);
                if mode == Mode::SampledTrace {
                    cfg.trace = Some(mtg_core::metrics::TraceConfig {
                        every: std::num::NonZeroU64::new(2).unwrap(),
                        capacity: 1,
                    });
                }
                if capture {
                    cfg.capture = Some(roots.capture());
                }
                cfg.validate().unwrap();
                let run = cfg.run_config_at(row).unwrap();
                let observations = Rc::new(RefCell::new((0u64, Sha256::new())));
                let probe = observations.clone();
                let now = clock(7);
                let timing = native::PolicyTiming {
                    replay: Some(native::RetainedReplay::new(67_108_864)),
                    detailed: true,
                    encode: true,
                    clock: now.clone(),
                    sample: Some(Box::new(move |_, o| {
                        let mut p = probe.borrow_mut();
                        p.0 += 1;
                        p.1.update(serde_json::to_vec(o).unwrap());
                    })),
                    ..Default::default()
                };
                let mut history = String::new();
                let mut final_hash = String::new();
                let start = now();
                let mut w = attempt_observed(
                    &run,
                    now.clone(),
                    u128::MAX,
                    &AtomicUsize::new(0),
                    timing,
                    &mut |r| {
                        history =
                            simulate::hash(&serde_json::to_vec(r.privileged_history()).unwrap());
                        final_hash =
                            simulate::hash(&serde_json::to_vec(&r.final_observations()).unwrap());
                    },
                    &mut |_, _| Ok(()),
                );
                w.elapsed_ns = now() - start;
                w.attempts = 1;
                w.first_episode = row;
                w.row_attempts[row as usize] = 1;
                w.row_completed[row as usize] = w.completed;
                assert_eq!(
                    (w.started, w.completed, w.failed, w.stop_code),
                    (1, 1, 0, 0)
                );
                let (count, hash) = &*observations.borrow();
                assert!(*count > 0);
                assert_eq!(*count, w.decisions);
                let observed_hash = format!("{:x}", hash.clone().finalize());
                let identity = (
                    history.clone(),
                    final_hash.clone(),
                    observed_hash.clone(),
                    w.decisions,
                );
                let baseline = &mut baselines[row as usize];
                if let Some(expected) = baseline {
                    assert_eq!(&identity, expected);
                } else {
                    *baseline = Some(identity);
                }
                let e = w.execution.as_ref().unwrap();
                if mode == Mode::SampledTrace {
                    assert_eq!(e.trace_selected, w.decisions / 2);
                    assert_eq!(e.trace_retained, 1);
                    assert_eq!(e.trace_dropped, w.decisions / 2 - 1);
                }
                assert_eq!(e.replay_verified, u64::from(mode == Mode::FullReplay));
                if mode == Mode::FullReplay {
                    assert!(e.replay_bytes > 0);
                    assert_eq!(e.replay_ns, 7);
                }
                assert_eq!(roots.validate(), usize::from(capture));
                rows.push(json!({"mode":mode,"capture":capture,"row":row,"observations":count,
                    "history_sha256":history,"observations_sha256":observed_hash,"final_sha256":final_hash,
                    "capture_loaded":capture,"window":w,"execution":{"replay":ReplayConfig::default(),
                    "trace":cfg.trace,"capture":if capture {"canonical-v2-durable"} else {"none"},"sampled_timing":"not_measured"}}));
            }
        }
    }
    if let Ok(path) = std::env::var("MTG_FOUR_MODE_TEST_OUTPUT") {
        fs::write(
            path,
            serde_json::to_vec(&json!({"schema_version":2,"workload":FOUR_MODES,"runs":rows}))
                .unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn four_modes_failures_keep_their_denominator_and_replay_is_never_incomplete_success() {
    let cfg = config(Mode::FullReplay);
    let mut c = cfg.run_config_at(0).unwrap();
    for failure in [
        "bytes",
        "records",
        "reset",
        "unfinished",
        "pre_reset",
        "write",
    ] {
        let roots = Roots::new();
        let mut max_bytes = 67_108_864;
        c.native.as_mut().unwrap().max_records = std::num::NonZeroUsize::new(20_000).unwrap();
        c.game.format = "foundations_micro_v1".into();
        c.max_decisions = 20_000;
        c.capture = None;
        let signal = AtomicUsize::new(0);
        match failure {
            "bytes" => max_bytes = 1,
            "records" => c.native.as_mut().unwrap().max_records = std::num::NonZeroUsize::MIN,
            "reset" => c.game.format = "invalid".into(),
            "unfinished" => c.max_decisions = 2,
            "pre_reset" => signal.store(signal_hook::consts::SIGINT as usize, Ordering::Relaxed),
            "write" => c.capture = Some(roots.capture()),
            _ => unreachable!(),
        }
        let now = clock(7);
        let start = now();
        let timing = native::PolicyTiming {
            replay: Some(native::RetainedReplay::new(max_bytes)),
            detailed: true,
            clock: now.clone(),
            ..Default::default()
        };
        let mut w = attempt_observed(
            &c,
            now.clone(),
            u128::MAX,
            &signal,
            timing,
            &mut |_| {},
            &mut |_, stage| {
                if failure == "write" && stage == mtg_recorder::FileStage::Write {
                    Err(std::io::Error::other("injected write failure"))
                } else {
                    Ok(())
                }
            },
        );
        w.elapsed_ns = now() - start;
        assert!(w.elapsed_ns >= 7);
        assert_eq!(w.completed, 0, "{failure}");
        if failure == "pre_reset" {
            assert_eq!(w.started, 0);
            assert_eq!(w.execution.as_ref().unwrap().not_started, 1);
            assert_eq!(w.stop_code, 130);
        } else if failure == "unfinished" {
            assert_eq!(w.truncated, 1);
            assert_eq!(w.execution.as_ref().unwrap().replay_incomplete, 1);
        } else {
            assert_eq!(w.failed, 1, "{failure}");
            assert_eq!(w.stop_code, 3);
        }
        if failure == "write" {
            assert_eq!(w.execution.as_ref().unwrap().publication_failed, 1);
            for entry in fs::read_dir(roots.0.join("data")).unwrap() {
                assert!(!entry.unwrap().path().join("manifest.json").exists());
            }
        }
    }
}

#[test]
fn four_modes_zero_trace_capacity_and_real_window_overshoot() {
    let mut cfg = config(Mode::SampledTrace);
    cfg.trace = Some(mtg_core::metrics::TraceConfig {
        every: std::num::NonZeroU64::new(2).unwrap(),
        capacity: 0,
    });
    cfg.validate().unwrap();
    let mut c = cfg.run_config_at(0).unwrap();
    let now = clock(1_000_000_000);
    let w = measured_window(
        &mut c,
        30,
        now,
        &AtomicUsize::new(0),
        false,
        true,
        Some(&ReplayConfig::default()),
    );
    // A real game takes many injected ticks. It still finishes once started,
    // and the full cost, including the overshoot, remains in the denominator.
    assert_eq!((w.attempts, w.started, w.completed), (1, 1, 1));
    assert!(w.elapsed_ns > 30_000_000_000);
    let e = w.execution.unwrap();
    assert_eq!(e.trace_retained, 0);
    assert_eq!(e.trace_selected, w.decisions / 2);
    assert_eq!(e.trace_dropped, e.trace_selected);
}

#[test]
fn four_modes_new_report_keeps_windows_and_signal_failure() {
    let cfg = config(Mode::FullReplay);
    let bytes = serde_json::to_vec(&cfg).unwrap();
    let (code, report) =
        run_with_clock(&bytes, &AtomicUsize::new(0), clock(1_000_000_000)).unwrap();
    assert_eq!(code, 0);
    assert_eq!(report["schema_version"], 2);
    assert_eq!(report["windows"].as_array().unwrap().len(), 5);
    assert_eq!(
        report["completed_games_per_second"]["samples"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    for w in report["windows"].as_array().unwrap() {
        assert_eq!(w["attempts"], 1);
        assert_eq!(w["completed"], 1);
        assert_eq!(w["execution"]["replay_verified"], 1);
        assert!(w["elapsed_ns"].as_u64().unwrap() > 30_000_000_000);
    }
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        let (code, report) =
            run_with_clock(&bytes, &AtomicUsize::new(signal as usize), clock(7)).unwrap();
        assert_eq!(code, 128 + signal);
        assert_eq!(report["status"], "incomplete-or-failed");
        assert_eq!(report["warmup"]["execution"]["not_started"], 1);
        assert_eq!(report["warmup"]["completed"], 0);
    }
}

#[test]
fn four_modes_fixed_semantic_script_keeps_literal_combat_and_concession_accounting() {
    // Existing normal-reset ordered-deck script: Growth makes Bear Cub 5/5;
    // its unblocked attack removes exactly five life, then P1 concedes.
    let mut baseline = None;
    for mode in [
        Mode::Off,
        Mode::Counters,
        Mode::SampledTrace,
        Mode::FullReplay,
    ] {
        for capture in [false, true] {
            let roots = Roots::new();
            let mut c: simulate::Config =
                serde_json::from_str(include_str!("../../../../fixtures/simulate/script-v3.json"))
                    .unwrap();
            c.native.as_mut().unwrap().instrumentation = mode;
            if capture {
                c.capture = Some(roots.capture());
            }
            let now = clock(7);
            let timing = native::PolicyTiming {
                replay: Some(native::RetainedReplay::new(67_108_864)),
                clock: now.clone(),
                detailed: true,
                ..Default::default()
            };
            let mut history = Vec::new();
            let w = attempt_observed(
                &c,
                now,
                u128::MAX,
                &AtomicUsize::new(0),
                timing,
                &mut |r| {
                    assert_eq!(r.final_observations().unwrap()[0].view.life, [20, 15]);
                    history = r.privileged_history().to_vec();
                },
                &mut |_, _| Ok(()),
            );
            assert_eq!((w.completed, w.concessions, w.failed), (0, 1, 0));
            assert_eq!(
                w.execution.unwrap().replay_verified,
                u64::from(mode == Mode::FullReplay)
            );
            if let Some(expected) = &baseline {
                assert_eq!(&history, expected);
            } else {
                baseline = Some(history);
            }
            assert_eq!(roots.validate(), usize::from(capture));
        }
    }
}
