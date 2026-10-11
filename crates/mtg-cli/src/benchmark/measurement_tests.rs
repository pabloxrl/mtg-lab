//! Independent B020 ledger plus real policy/encoding/mode episodes for GH-281.
use super::*;
use sha2::{Digest, Sha256};
use std::cell::{Cell, RefCell};

#[test]
fn measurement_failure_time_is_not_removed_from_completed_game_rate() {
    let ticks = Cell::new(0u128);
    let n = Cell::new(0);
    let w = window(100, &|| ticks.get(), &mut || {
        n.set(n.get() + 1);
        let good = n.get() == 1;
        ticks.set(ticks.get() + if good { 17 } else { 29 });
        Window {
            started: 1,
            completed: u64::from(good),
            failed: u64::from(!good),
            stop_code: if good { 0 } else { 3 },
            ..Default::default()
        }
    });
    assert_eq!(
        (w.attempts, w.completed, w.failed, w.elapsed_ns),
        (2, 1, 1, 46)
    );
    assert_eq!(w.rate(), Some(1e9 / 46.0));
    assert_ne!(w.rate(), Some(1e9 / 17.0));
}

#[test]
fn measurement_export_native_latency_and_equivalence() {
    let mut reports = Vec::new();
    for policy in [mtg_policy::HEURISTIC_VERSION, mtg_policy::VERSION] {
        for row in 0..8 {
            let mut baseline = None;
            for encoded in [false, true] {
                for mode in [
                    mtg_core::metrics::Mode::Off,
                    mtg_core::metrics::Mode::Counters,
                    mtg_core::metrics::Mode::SampledTrace,
                    mtg_core::metrics::Mode::FullReplay,
                ] {
                    let cfg:Config=serde_json::from_value(json!({"schema_version":2,"workload":FOUR_MODES,
                        "warmup_seconds":10,"window_seconds":30,"windows":5,"policies":[policy,policy],
                        "master_seed":42,"policy_seed":42,"instrumentation":mode,"encoding":encoded})).unwrap();
                    cfg.validate().unwrap();
                    let c = cfg.run_config_at(row).unwrap();
                    let observed = Rc::new(RefCell::new((0u64, Sha256::new())));
                    let probe = observed.clone();
                    let mut timing = native::PolicyTiming {
                        encode: encoded,
                        detailed: true,
                        replay: Some(native::RetainedReplay::new(67_108_864)),
                        sample: Some(Box::new(move |_, o| {
                            let mut x = probe.borrow_mut();
                            x.0 += 1;
                            x.1.update(serde_json::to_vec(o).unwrap());
                        })),
                        ..Default::default()
                    };
                    let mut output = Vec::new();
                    let mut history = String::new();
                    let mut final_state = String::new();
                    assert_eq!(
                        native::run_instrumented(
                            &c,
                            &mut output,
                            || None,
                            &mut |_, _| Ok(()),
                            &mut |r| {
                                assert!(matches!(r.status(), mtg_core::episode::Status::Completed(o) if natural_outcome(o)));
                                history = simulate::hash(
                                    &serde_json::to_vec(r.privileged_history()).unwrap(),
                                );
                                final_state = simulate::hash(
                                    &serde_json::to_vec(&r.final_observations()).unwrap(),
                                );
                            },
                            Some(&mut timing)
                        )
                        .unwrap(),
                        0
                    );
                    let lines: Vec<Value> = output
                        .split(|b| *b == b'\n')
                        .filter(|s| !s.is_empty())
                        .map(|s| serde_json::from_slice(s).unwrap())
                        .collect();
                    let episode = lines.iter().find(|v| v["type"] == "episode").unwrap();
                    assert_eq!(episode["status"], "completed");
                    let (count, digest) = &*observed.borrow();
                    assert!(*count > 0);
                    assert_eq!(episode["decisions"], *count);
                    let observation_hash = format!("{:x}", digest.clone().finalize());
                    let identity = (
                        history.clone(),
                        final_state.clone(),
                        observation_hash.clone(),
                        *count,
                    );
                    if let Some(b) = &baseline {
                        assert_eq!(b, &identity);
                    } else {
                        baseline = Some(identity);
                    }
                    let summary = lines.last().unwrap();
                    assert_eq!(
                        summary.get("latency").is_some(),
                        mode != mtg_core::metrics::Mode::Off
                    );
                    if mode == mtg_core::metrics::Mode::FullReplay {
                        assert_eq!(timing.replay.as_ref().unwrap().verified, 1);
                    }
                    reports.push(
                        json!({"policy":policy,"row":row,"encoded":encoded,"mode":mode,
                        "config":c,"replay_contract":modes::ReplayConfig::default(),"capture":"none",
                        "replay_verified":timing.replay.as_ref().unwrap().verified,
                        "observations":count,"history_sha256":history,"final_sha256":final_state,
                        "observations_sha256":observation_hash,"episode":episode,
                        "latency":summary.get("latency"),"phases":timing.phases}),
                    );
                }
            }
        }
    }
    if let Ok(path) = std::env::var("MTG_MEASUREMENT_NATIVE_OUTPUT") {
        std::fs::write(
            path,
            serde_json::to_vec(&json!({"schema_version":1,
            "scope":"fixed natural episodes; separate from timed-window benchmark",
            "hardware":metadata(),"runs":reports}))
            .unwrap(),
        )
        .unwrap();
    }
}
