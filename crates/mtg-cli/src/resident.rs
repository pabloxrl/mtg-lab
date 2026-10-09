//! Diagnostic normal-play sampling and resident core-state probes.
//! Privileged snapshots/history never enter native policy inputs.
use crate::{benchmark, native, simulate};
use mtg_core::{
    episode::Driver,
    game::{
        Game,
        policy::{Choice, Observation},
    },
    objects::Seat,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

const KINDS: [&str; 4] = ["typical", "token-heavy", "target-rich", "stack-heavy"];
const MINIMA: [usize; 4] = [1, 4, 6, 3];
const COUNTS: [usize; 8] = [0, 1, 32, 128, 512, 1000, 5000, 10000];
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Capture {},
    Sweep { specimen_path: PathBuf },
}

fn scores(o: &Observation) -> [usize; 4] {
    let tokens = o
        .view
        .public_zones
        .iter()
        .filter(|z| z.zone == "battlefield")
        .flat_map(|z| &z.cards)
        .filter(|c| c.card == "goblin-token")
        .count();
    let targets = o.decision.as_ref().map_or(0, |d| {
        d.candidates
            .iter()
            .zip(&d.legal_mask)
            .filter(|(c, legal)| **legal && matches!(c, Choice::Target { .. }))
            .count()
    });
    [
        usize::from(o.view.turn.is_some_and(|(n, _, _)| n >= 5)),
        tokens,
        targets,
        o.stack.len() + o.pending_triggers.len(),
    ]
}
fn proc_kib(text: &str, key: &str) -> Result<u64, String> {
    let line = text
        .lines()
        .find_map(|l| l.strip_prefix(key))
        .ok_or_else(|| format!("missing OS field {key}"))?;
    let fields: Vec<_> = line.split_whitespace().collect();
    if fields.len() != 2 || fields[1] != "kB" {
        return Err("unexpected proc memory units".into());
    }
    fields[0]
        .parse::<u64>()
        .ok()
        .and_then(|n| n.checked_mul(1024))
        .ok_or_else(|| "invalid proc memory value".into())
}
fn memory() -> Result<Value, String> {
    let status = std::fs::read_to_string("/proc/self/status").map_err(|e| e.to_string())?;
    let rollup = std::fs::read_to_string("/proc/self/smaps_rollup").map_err(|e| e.to_string())?;
    Ok(
        json!({"rss_bytes":proc_kib(&status,"VmRSS:")?,"high_water_bytes":proc_kib(&status,"VmHWM:")?,
        "smaps_rss_bytes":proc_kib(&rollup,"Rss:")?,"smaps_pss_bytes":proc_kib(&rollup,"Pss:")?,
        "private_dirty_bytes":proc_kib(&rollup,"Private_Dirty:")?,"shared_clean_bytes":proc_kib(&rollup,"Shared_Clean:")?}),
    )
}
fn restore(snapshot: &[u8]) -> Result<Game, String> {
    let mut game = Game::new().map_err(|e| format!("state allocation: {e:?}"))?;
    game.restore(snapshot)
        .map_err(|e| format!("snapshot rejected: {e:?}"))?;
    Ok(game)
}
#[cfg(test)]
fn sweep(snapshot: &[u8], counts: &[usize], cycles: usize) -> Result<Value, String> {
    sweep_controlled(snapshot, counts, cycles, &AtomicUsize::new(0))
}
fn sweep_controlled(
    snapshot: &[u8],
    counts: &[usize],
    cycles: usize,
    signal: &AtomicUsize,
) -> Result<Value, String> {
    // Warm the parser/shared compatibility tables before the zero-state point.
    drop(restore(snapshot)?);
    let mut samples = Vec::new();
    let mut released = Vec::new();
    let mut reset_churn = Vec::new();
    for cycle in 0..cycles {
        let mut states = Vec::new();
        for &count in counts {
            let start = Instant::now();
            while states.len() < count {
                if signal.load(Ordering::Relaxed) != 0 {
                    return Err("resident probe interrupted".into());
                }
                states.push(restore(snapshot)?);
            }
            std::hint::black_box(&states);
            let mut point = memory()?;
            point["cycle"] = json!(cycle);
            point["resident"] = json!(states.len());
            point["vector_capacity"] = json!(states.capacity());
            point["restore_elapsed_ns"] = json!(start.elapsed().as_nanos());
            samples.push(point);
        }
        let mut settle_calls = 0u64;
        for (ordinal, game) in states.iter_mut().enumerate() {
            if signal.load(Ordering::Relaxed) != 0 {
                return Err("reset probe interrupted".into());
            }
            let mut settled = false;
            for _ in 0..100_000 {
                settle_calls += 1;
                if game.resume(std::num::NonZeroUsize::new(64).unwrap())
                    != mtg_core::game::Progress::InternalYield
                {
                    settled = true;
                    break;
                }
            }
            if !settled {
                return Err("reset churn settling exceeded work bound".into());
            }
            game.reset(&mtg_core::game::Config::default(), 42, ordinal as u64)
                .map_err(|e| format!("reset churn failed: {e:?}"))?;
        }
        let mut reset = memory()?;
        reset["cycle"] = json!(cycle);
        reset["resident"] = json!(states.len());
        reset["settle_work_calls"] = json!(settle_calls);
        reset_churn.push(reset);
        drop(states);
        let mut point = memory()?;
        point["cycle"] = json!(cycle);
        released.push(point);
    }
    Ok(
        json!({"schema_version":1,"workload":"scalar-resident-v1","status":"measured","samples":samples,"released":released,"reset_churn":reset_churn,
        "core_inline_bytes":std::mem::size_of::<Game>(),"snapshot_input_bytes":snapshot.len(),
        "oracle":"Linux /proc/self/status and smaps_rollup, KiB converted to bytes",
        "scope":"restored core Game states only; shared tables/process/one snapshot buffer and retained prefix-replay/parser allocations in zero-state base; no resident policy, ready, inference, history or trajectory buffers",
        "cycles":"normal RG reset (seed 42, ordinal=slot) at maximum residency, then drop and reconstruct every state; allocator retention remains visible; no malloc trim or leak-freedom claim"}),
    )
}
fn capture(signal: &AtomicUsize) -> Result<(i32, Value), String> {
    capture_episodes(signal, 64)
}
fn capture_episodes(signal: &AtomicUsize, episodes: u64) -> Result<(i32, Value), String> {
    let specimens = Rc::new(RefCell::new([None::<Value>, None, None, None]));
    let mut attempts = Vec::new();
    let mut failed = false;
    for ordinal in 0..episodes {
        if signal.load(Ordering::Relaxed) != 0 {
            return Err("state sampling interrupted".into());
        }
        let mut c: simulate::Config = serde_json::from_slice(include_bytes!(
            "../../../fixtures/bench/scalar-workload-v1.json"
        ))
        .map_err(|e| e.to_string())?;
        c.first_episode = ordinal;
        let policy = if (ordinal / 8).is_multiple_of(2) {
            mtg_policy::HEURISTIC_VERSION
        } else {
            mtg_policy::VERSION
        };
        c.policies = [policy.into(), policy.into()];
        benchmark::full_pool_row(&mut c, ordinal);
        let config = serde_json::to_value(&c).map_err(|e| e.to_string())?;
        let saved = specimens.clone();
        let mut timing = native::PolicyTiming {
            sample: Some(Box::new(move |driver, o| {
                let shape = scores(o);
                let mut slots = saved.borrow_mut();
                for (i, &score) in shape.iter().enumerate() {
                    if score > 0
                        && slots[i]
                            .as_ref()
                            .is_none_or(|v| score as u64 > v["score"].as_u64().unwrap())
                    {
                        let snapshot = driver.privileged_snapshot();
                        let history: Vec<Value> = driver
                            .privileged_history()
                            .iter()
                            .map(|b| serde_json::from_slice(b).expect("owned semantic record"))
                            .collect();
                        slots[i] = Some(
                            json!({"kind":KINDS[i],"score":score,"shape":shape,"actor":o.view.seat,
                            "config":config,"ordinal":ordinal,"decision_index":history.len(),"history":history,
                            "snapshot_sha256":simulate::hash(&snapshot),"snapshot":String::from_utf8(snapshot).expect("snapshot JSON"),
                            "provenance":"existing native client/Driver, normal reset; privileged diagnostic sample before a policy choice"}),
                        );
                    }
                }
            })),
            ..Default::default()
        };
        let mut bytes = Vec::new();
        let code = native::run_instrumented(
            &c,
            &mut bytes,
            || (signal.load(Ordering::Relaxed) != 0).then_some(simulate::Stop::Sigint),
            &mut |_, _| Ok(()),
            &mut |_| {},
            Some(&mut timing),
        )
        .map_err(|e| e.to_string())?;
        failed |= code != 0;
        let rows: Vec<Value> = bytes
            .split(|b| *b == b'\n')
            .filter(|b| !b.is_empty())
            .map(serde_json::from_slice)
            .collect::<Result<_, _>>()
            .map_err(|e| e.to_string())?;
        attempts.push(json!({"ordinal":ordinal,"exit_code":code,"rows":rows}));
    }
    let mut selected = serde_json::Map::new();
    let mut missing = Vec::new();
    for (i, specimen) in specimens.borrow().iter().enumerate() {
        if specimen
            .as_ref()
            .is_none_or(|v| v["score"].as_u64().unwrap() < MINIMA[i] as u64)
        {
            missing.push(KINDS[i]);
        }
        if let Some(v) = specimen {
            selected.insert(KINDS[i].into(), v.clone());
        }
    }
    let code = if failed || !missing.is_empty() { 3 } else { 0 };
    Ok((
        code,
        json!({"schema_version":1,"workload":"scalar-resident-v1","status":if code==0{"sampled"}else{"incomplete-or-failed"},"specimens":selected,"missing":missing,
        "attempts":attempts,"source_sha256":env!("MTG_BENCH_SOURCE"),"engine":mtg_core::game::snapshot::engine(),
        "privacy":"privileged snapshots/actions; never a player export", "minima":MINIMA}),
    ))
}
fn replay_prefix(specimen: &Value) -> Result<Vec<u8>, String> {
    use mtg_core::episode::{Progress, RecordContext};
    let c: simulate::Config =
        serde_json::from_value(specimen["config"].clone()).map_err(|e| e.to_string())?;
    c.validate()?;
    let n = c.native.as_ref().ok_or("native configuration required")?;
    if c.max_decisions != 20_000 || n.max_work_calls != 100_000 || n.max_records.get() != 20_000 {
        return Err("changed state-sampling horizon".into());
    }
    let history = specimen["history"]
        .as_array()
        .ok_or("missing semantic prefix")?;
    if history.len() > 20_000 || specimen["decision_index"].as_u64() != Some(history.len() as u64) {
        return Err("prefix count mismatch".into());
    }
    let mut driver = Driver::new(256).map_err(|e| format!("{e:?}"))?;
    let mut progress = driver
        .reset(&c.game, c.master_seed, c.first_episode, n.work_quantum)
        .map_err(|e| format!("{e:?}"))?;
    let mut calls = 1;
    for record in history {
        while progress == Progress::InternalYield {
            calls += 1;
            if calls > 100_000 {
                return Err("prefix work bound exceeded".into());
            }
            progress = driver
                .advance(n.work_quantum)
                .map_err(|e| format!("{e:?}"))?;
        }
        let record: mtg_core::game::actions::Record =
            serde_json::from_value(record.clone()).map_err(|e| e.to_string())?;
        let decision = driver
            .observe(record.actor)
            .map_err(|e| format!("{e:?}"))?
            .decision
            .ok_or("prefix has no actor choice")?;
        driver
            .submit_record(
                record.actor,
                RecordContext::Decision {
                    revision: decision.revision,
                    generation: decision.generation,
                },
                &serde_json::to_vec(&record).map_err(|e| e.to_string())?,
            )
            .map_err(|e| format!("{e:?}"))?;
        progress = Progress::InternalYield;
    }
    while progress == Progress::InternalYield {
        calls += 1;
        if calls > 100_000 {
            return Err("prefix work bound exceeded".into());
        }
        progress = driver
            .advance(n.work_quantum)
            .map_err(|e| format!("{e:?}"))?;
    }
    let actor = match specimen["actor"].as_u64() {
        Some(0) => Seat::P0,
        Some(1) => Seat::P1,
        _ => return Err("invalid sample actor".into()),
    };
    let kind = KINDS
        .iter()
        .position(|k| specimen["kind"] == *k)
        .ok_or("unknown state class")?;
    let o = driver.observe(actor).map_err(|e| format!("{e:?}"))?;
    if scores(&o)[kind] as u64 != specimen["score"].as_u64().ok_or("missing score")?
        || scores(&o)[kind] < MINIMA[kind]
    {
        return Err("replayed stress shape mismatch".into());
    }
    let snapshot = specimen["snapshot"].as_str().ok_or("missing snapshot")?;
    if specimen["snapshot_sha256"] != simulate::hash(snapshot.as_bytes()) {
        return Err("snapshot integrity failure".into());
    }
    let original = restore(snapshot.as_bytes())?;
    let original = original
        .policy_observe(actor, 256)
        .map_err(|e| format!("{e:?}"))?;
    if original.view != o.view
        || original.stack != o.stack
        || original.pending != o.pending
        || original.combat != o.combat
    {
        return Err("snapshot/prefix public state mismatch".into());
    }
    // Use the independently re-executed normal-reset prefix state for the sweep.
    Ok(driver.privileged_snapshot())
}
pub(crate) fn run(bytes: &[u8], signal: &AtomicUsize) -> Result<(i32, Value), (i32, String)> {
    let request: Request = serde_json::from_slice(bytes).map_err(|e| (2, e.to_string()))?;
    let result = match request {
        Request::Capture {} => capture(signal),
        Request::Sweep { specimen_path } => (|| {
            let meta = std::fs::metadata(&specimen_path).map_err(|e| e.to_string())?;
            if !meta.is_file() || meta.len() > 16_777_216 {
                return Err("specimen must be a regular file <=16 MiB".into());
            }
            let specimen: Value =
                serde_json::from_slice(&std::fs::read(specimen_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let snapshot = replay_prefix(&specimen)?;
            // Free action history/config/snapshot text before the measured baseline.
            let summary = json!({"kind":specimen["kind"],"score":specimen["score"],"ordinal":specimen["ordinal"],"decision_index":specimen["decision_index"],"snapshot_sha256":specimen["snapshot_sha256"],"prefix_verified":true});
            drop(specimen);
            let mut report = sweep_controlled(&snapshot, &COUNTS, 3, signal)?;
            report["specimen"] = summary;
            report["source_sha256"] = json!(env!("MTG_BENCH_SOURCE"));
            Ok((0, report))
        })(),
    };
    result.map_err(|e| {
        let sig = signal.load(Ordering::Relaxed);
        (if sig == 0 { 3 } else { 128 + sig as i32 }, e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_pool_contract_resident_os_units_are_bytes_not_serialized_size() {
        assert_eq!(
            proc_kib("VmRSS:\t123 kB\nVmHWM:\t456 kB\n", "VmRSS:").unwrap(),
            123 * 1024
        );
        assert_eq!(
            proc_kib("VmRSS:\t123 kB\nVmHWM:\t456 kB\n", "VmHWM:").unwrap(),
            456 * 1024
        );
        assert!(proc_kib("VmRSS: 1 MB", "VmRSS:").is_err());
        assert!(proc_kib("VmHWM: 1 kB", "VmRSS:").is_err());
    }
    #[test]
    fn full_pool_contract_resident_stress_scores_use_actual_public_position() {
        use mtg_core::{episode::Driver, game::Config, objects::Seat};
        let mut d = Driver::new(256).unwrap();
        let q = std::num::NonZeroUsize::new(64).unwrap();
        let mut progress = d.reset(&Config::default(), 42, 0, q).unwrap();
        while progress == mtg_core::episode::Progress::InternalYield {
            progress = d.advance(q).unwrap();
        }
        let mut o = d.observe(Seat::P0).unwrap();
        // Synthetic observation only tests classification, never memory/game evidence.
        let card = o.view.hand[0].clone();
        o.view.public_zones.push(mtg_core::game::views::PublicZone {
            zone: "battlefield",
            cards: vec![
                mtg_core::game::views::VisibleCard {
                    card: "goblin-token",
                    ..card
                };
                4
            ],
        });
        assert_eq!(scores(&o)[1], 4);
        o.view.public_zones.clear();
        assert_eq!(scores(&o)[1], 0);
    }
    #[test]
    fn full_pool_contract_resident_stress_probe_preserves_native_output() {
        use std::{cell::Cell, rc::Rc};
        let mut c: crate::simulate::Config = serde_json::from_slice(include_bytes!(
            "../../../fixtures/bench/scalar-workload-v1.json"
        ))
        .unwrap();
        c.max_decisions = 2; // tiny transparency test; production horizon is unchanged.
        let calls = Rc::new(Cell::new(0));
        let seen = calls.clone();
        let mut timing = crate::native::PolicyTiming {
            sample: Some(Box::new(move |d, _| {
                assert!(!d.privileged_snapshot().is_empty());
                seen.set(seen.get() + 1);
            })),
            ..Default::default()
        };
        let mut plain = Vec::new();
        let mut probed = Vec::new();
        let a = crate::native::run(&c, &mut plain, || None).unwrap();
        let b = crate::native::run_instrumented(
            &c,
            &mut probed,
            || None,
            &mut |_, _| Ok(()),
            &mut |_| {},
            Some(&mut timing),
        )
        .unwrap();
        assert_eq!(a, b);
        assert_eq!(plain, probed);
        assert!(calls.get() > 0, "diagnostic state probe must execute");
    }

    #[test]
    fn full_pool_contract_resident_stress_sweep_retains_every_point_and_rejects_corruption() {
        let mut d = mtg_core::episode::Driver::new(256).unwrap();
        d.reset(
            &mtg_core::game::Config::default(),
            42,
            0,
            std::num::NonZeroUsize::new(64).unwrap(),
        )
        .unwrap();
        let report = sweep(&d.privileged_snapshot(), &[0, 1, 2], 2).unwrap();
        assert_eq!(report["samples"].as_array().unwrap().len(), 6);
        assert_eq!(report["samples"][5]["resident"], 2);
        assert_eq!(report["released"].as_array().unwrap().len(), 2);
        assert_eq!(report["reset_churn"].as_array().map(Vec::len), Some(2));
        assert_eq!(report["reset_churn"][1]["resident"], 2);
        assert!(sweep(b"corrupt", &[0, 1], 1).is_err());
    }
    #[test]
    fn full_pool_contract_resident_prefix_reexecutes_normal_play_and_rejects_corruption() {
        let (_, report) = capture_episodes(&AtomicUsize::new(0), 1).unwrap();
        let specimen = &report["specimens"]["typical"];
        assert!(
            !specimen.is_null(),
            "seed 42 RG must reach a normal turn-five sample"
        );
        assert!(!replay_prefix(specimen).unwrap().is_empty());
        let mut bad = specimen.clone();
        bad["snapshot_sha256"] = json!("corrupt");
        assert!(replay_prefix(&bad).is_err());
        let mut bad = specimen.clone();
        bad["history"].as_array_mut().unwrap().pop();
        assert!(replay_prefix(&bad).is_err());
        for input in [
            b"{\"operation\":\"unknown\"}".as_slice(),
            b"{\"operation\":\"capture\",\"extra\":1}",
        ] {
            assert!(matches!(run(input, &AtomicUsize::new(0)), Err((2, _))));
        }
    }
}
