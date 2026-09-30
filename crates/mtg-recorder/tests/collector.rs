//! GH-163: actual normal resets, seed 163. CR 103.5 keeps and CR 104.3a
//! concession: two Keep decisions, then P1 concedes outside policy decisions.
//! Decision budget 2 instead truncates after both keeps, with zero reward.
use mtg_core::{
    episode::{Budget, Clock, Driver, EpisodeResult, Status},
    game::{
        Config,
        policy::{Choice, Submission},
    },
    objects::Seat,
    trajectory::{Limits, PolicyInfo},
};
use mtg_recorder::{
    Backpressure, Error,
    collector::{Run, Storage},
    manifest::{EpisodeStatus, LoadMode, RunEnd},
    schema::End,
};
use sha2::{Digest, Sha256};
use std::{
    io::{self, Write},
    num::NonZeroUsize,
};
const RUN: &str = "a34c952c-723c-44ef-95f9-dcdb066db576";
#[derive(Debug)]
struct Zero;
impl Clock for Zero {
    fn now_ms(&self) -> u64 {
        0
    }
}
fn run(n: u64, truncated: bool) -> Run {
    Run {
        id: RUN.into(),
        config: Config::default(),
        policies: ["script-v1".into(), "script-v2".into()],
        limits: Limits {
            decisions: Some(if truncated { 2 } else { 100 }),
            ..Limits::default()
        },
        first_ordinal: 7,
        started: n,
    }
}
fn storage() -> Storage {
    Storage {
        queue_bytes: 1_000_000,
        max_bytes: 4_000_000,
        backpressure: Backpressure::Block,
    }
}
fn result(run: &Run, ordinal: u64, kind: &str) -> EpisodeResult {
    let records = if kind == "failed" { 1 } else { 100 };
    let mut d = Driver::bounded(
        256,
        Budget {
            limits: run.limits.clone(),
            work_quantum: NonZeroUsize::MAX,
            records: NonZeroUsize::new(records).unwrap(),
        },
        Box::new(Zero),
    )
    .unwrap();
    let h = run.header(ordinal).unwrap();
    d.reset_captured(&run.config, 163, ordinal, NonZeroUsize::MAX, &h)
        .unwrap();
    if kind != "incomplete" {
        for seat in [Seat::P0, Seat::P1] {
            let domain = d.observe(seat).unwrap().decision.unwrap();
            let accepted = d.submit_with_policy(
                seat,
                &Submission {
                    schema_version: 1,
                    revision: domain.revision,
                    generation: domain.generation,
                    choices: vec![Choice::Keep],
                },
                &PolicyInfo {
                    checkpoint: Some("caller-checkpoint".into()),
                    ..PolicyInfo::default()
                },
            );
            if kind == "failed" && seat == Seat::P1 {
                assert!(accepted.is_err());
            } else {
                accepted.unwrap();
            }
        }
        if kind == "completed" {
            d.advance(NonZeroUsize::MAX).unwrap();
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
        }
    }
    d.finish().unwrap()
}
fn load(
    bundle: &mtg_recorder::collector::Bundle,
    mode: LoadMode,
) -> Result<mtg_recorder::manifest::LoadedRun<mtg_recorder::structured::Episode>, Error> {
    let m = bundle.manifest();
    m.load_v2(&m.file.name, bundle.bytes(), &m.versions, 4_000_000, mode)
}
#[test]
fn actual_completed_results_reload_with_literal_decisions_and_sealed_inventory() {
    let r = run(2, false);
    let results = vec![result(&r, 7, "completed"), result(&r, 8, "completed")];
    let before = results[0].privileged_snapshot().to_vec();
    let bundle = r
        .persist(&results, storage())
        .expect("completed owned games must persist");
    let loaded = load(&bundle, LoadMode::CompletedOnly).unwrap();
    assert_eq!(loaded.episodes().len(), 2);
    for (stored, original) in loaded.episodes().iter().zip(&results) {
        assert_eq!(
            stored,
            &mtg_recorder::from_core_v2(original.trajectory().unwrap()).unwrap()
        );
    }
    assert_eq!(bundle.manifest().file.episodes, 2);
    assert_eq!(bundle.manifest().file.decisions, 4);
    assert_eq!(bundle.manifest().file.bytes, bundle.bytes().len() as u64);
    assert_eq!(
        bundle.manifest().file.sha256,
        format!("{:x}", Sha256::digest(bundle.bytes()))
    );
    for (e, ordinal) in loaded.episodes().iter().zip([7, 8]) {
        assert_eq!(e.header.id.ordinal, ordinal);
        assert_eq!(e.header.restricted_replay, None);
        assert_eq!(
            e.decisions
                .iter()
                .map(|d| (d.index, d.actor))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 1)]
        );
        for d in &e.decisions {
            assert_eq!(
                d.choice.submission.choices,
                vec![mtg_recorder::structured::Command::Keep]
            );
            assert_eq!(
                d.choice.policy.checkpoint.as_deref(),
                Some("caller-checkpoint")
            );
            assert_eq!(d.choice.policy.log_probability, None);
        }
        let f = e.footer.as_ref().unwrap();
        assert_eq!(f.end, End::Completed);
        assert_eq!(f.returns, [1, -1]);
        assert_eq!(f.boundary_reward, [1, -1]);
    }
    assert_eq!(results[0].privileged_snapshot(), before);
}
#[test]
fn actual_decision_truncation_is_diagnostic_and_never_a_rules_completion() {
    let r = run(1, true);
    let results = vec![result(&r, 7, "truncated")];
    let b = r
        .persist(&results, storage())
        .expect("valid truncated rows must seal");
    assert!(b.manifest().recording_complete);
    assert!(matches!(b.manifest().end, RunEnd::Truncated(_)));
    assert!(matches!(
        load(&b, LoadMode::CompletedOnly),
        Err(Error::Incomplete)
    ));
    let loaded = load(&b, LoadMode::Diagnostic).unwrap();
    let e = &loaded.episodes()[0];
    assert_eq!(e.decisions.len(), 2);
    assert_eq!(e.footer.as_ref().unwrap().returns, [0, 0]);
    assert_eq!(
        e.footer.as_ref().unwrap().end,
        End::Truncated(mtg_recorder::schema::Limit::Decisions)
    );
}
#[test]
fn mixed_actual_failures_and_incomplete_ordinals_never_become_training_rows() {
    let r = run(3, false);
    let results = vec![
        result(&r, 7, "completed"),
        result(&r, 8, "failed"),
        result(&r, 9, "incomplete"),
    ];
    assert!(matches!(results[1].status(), Status::Failed(_)));
    let b = r
        .persist(&results, storage())
        .expect("diagnostic run retains valid rows");
    assert!(!b.manifest().recording_complete);
    assert!(matches!(b.manifest().end, RunEnd::Failed(_)));
    assert_eq!(
        b.manifest()
            .episodes
            .iter()
            .map(|e| e.ordinal)
            .collect::<Vec<_>>(),
        vec![7, 8, 9]
    );
    assert!(matches!(
        b.manifest().episodes[1].status,
        EpisodeStatus::Failed(_)
    ));
    assert_eq!(b.manifest().episodes[2].status, EpisodeStatus::Incomplete);
    assert!(matches!(
        load(&b, LoadMode::CompletedOnly),
        Err(Error::Incomplete)
    ));
    let loaded = load(&b, LoadMode::Diagnostic).unwrap();
    assert_eq!(loaded.episodes().len(), 1);
    assert_eq!(loaded.episodes()[0].header.id.ordinal, 7);
}
#[test]
fn missing_duplicate_reordered_and_mismatched_inputs_reject_without_consuming_results() {
    let r = run(2, false);
    let a = result(&r, 7, "completed");
    let b = result(&r, 8, "completed");
    for results in [
        vec![a.clone()],
        vec![a.clone(), a.clone()],
        vec![b.clone(), a.clone()],
    ] {
        assert!(r.persist(&results, storage()).is_err());
    }
    let results = vec![a, b];
    let mut wrong = r.clone();
    wrong.config.starting_seat = 1;
    assert!(wrong.persist(&results, storage()).is_err());
    wrong = r.clone();
    wrong.policies[0] = "unrelated".into();
    assert!(wrong.persist(&results, storage()).is_err());
    wrong = r.clone();
    wrong.limits.decisions = Some(99);
    assert!(wrong.persist(&results, storage()).is_err());
    assert_eq!(results[0].accepted_decisions(), 2);
    assert!(matches!(results[0].status(), Status::Completed(_)));
}
#[derive(Default)]
struct Fault {
    bytes: Vec<u8>,
    writes: usize,
    fail_write: usize,
    fail_flush: bool,
}
impl AsRef<[u8]> for Fault {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl Write for Fault {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        if self.writes == self.fail_write {
            return Err(io::Error::other("injected"));
        }
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::other("injected flush"))
        } else {
            Ok(())
        }
    }
}
#[test]
fn writer_drain_seal_flush_and_late_failure_propagate_and_originals_survive() {
    let r = run(1, false);
    let results = vec![result(&r, 7, "completed")];
    let original = results[0].privileged_snapshot().to_vec();
    for sink in [
        Fault {
            fail_write: 1,
            ..Fault::default()
        },
        Fault {
            fail_write: 2,
            ..Fault::default()
        },
        Fault {
            fail_flush: true,
            ..Fault::default()
        },
    ] {
        assert!(matches!(
            r.persist_with_sink(&results, storage(), sink),
            Err(Error::Io(_))
        ));
        assert_eq!(results[0].privileged_snapshot(), original);
    }
    let mut s = storage();
    s.queue_bytes = 1;
    assert!(matches!(r.persist(&results, s), Err(Error::Limit)));
    s = storage();
    s.max_bytes = 1;
    assert!(r.persist(&results, s).is_err());
}

#[test]
fn provenance_matches_independent_pins_and_actual_config_not_caller_labels() {
    let r = run(1, false);
    let h = r.header(7).unwrap();
    // Python hashlib over sorted compact frozen deck JSON, independently computed.
    assert_eq!(
        h.deck_hashes,
        [
            "9fa60b4d882f44291d75567a33303a7cba6052ae8e9bbaa93e2300a3007f9f2c",
            "23f9c575555d9d7e74f6b88e216b7646459484dcd1bd963b231923e889219f7e"
        ]
    );
    assert_eq!(
        h.config_hash,
        format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&r.config).unwrap())
        )
    );
    assert_eq!(
        h.versions.rules,
        format!(
            "{:x}",
            Sha256::digest(include_bytes!("../../../data/rules/cr-2026-09-25.json"))
        )
    );
    assert_eq!(
        h.versions.cards,
        format!(
            "{:x}",
            Sha256::digest(include_bytes!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
        )
    );
    let result = result(&r, 7, "completed");
    let snapshot: serde_json::Value = serde_json::from_slice(result.privileged_snapshot()).unwrap();
    assert_eq!(h.versions.engine, snapshot["engine"].as_str().unwrap());
    assert_eq!(h.versions.action, "policy-v1");
    assert_eq!(h.versions.observation, 1);
    let b = r.persist(&[result], storage()).unwrap();
    assert_eq!(b.manifest().policies, ["script-v1", "script-v2"]);
    let text = String::from_utf8(b.manifest().encode(100_000).unwrap()).unwrap();
    assert!(!text.contains("master"));
    assert!(!text.contains("snapshot"));
    let mut mirror = r.clone();
    mirror.config.seats[1].deck = "red".into();
    assert_eq!(
        mirror.header(7).unwrap().deck_hashes,
        [h.deck_hashes[0].clone(), h.deck_hashes[0].clone()]
    );
    assert_ne!(mirror.header(7).unwrap().config_hash, h.config_hash);
}
#[test]
fn unrelated_capture_header_versions_decks_and_config_are_rejected() {
    let r = run(1, false);
    for field in 0..4 {
        let mut h = r.header(7).unwrap();
        match field {
            0 => h.versions.engine = "unrelated-engine".into(),
            1 => h.deck_hashes[0] = "a".repeat(64),
            2 => h.config_hash = "c".repeat(64),
            _ => h.versions.action = "unknown-actions".into(),
        }
        let mut d = Driver::bounded(
            256,
            Budget {
                limits: r.limits.clone(),
                work_quantum: NonZeroUsize::MAX,
                records: NonZeroUsize::new(100).unwrap(),
            },
            Box::new(Zero),
        )
        .unwrap();
        d.reset_captured(&r.config, 163, 7, NonZeroUsize::MAX, &h)
            .unwrap();
        d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
        let result = d.finish().unwrap();
        let snapshot = result.privileged_snapshot().to_vec();
        assert!(matches!(
            r.persist(std::slice::from_ref(&result), storage()),
            Err(Error::Invalid)
        ));
        assert_eq!(result.privileged_snapshot(), snapshot);
    }
}
#[test]
fn append_backpressure_and_drain_failures_are_not_silent_drops() {
    let r = run(2, false);
    let results = vec![result(&r, 7, "completed"), result(&r, 8, "completed")];
    let baseline = r.persist(&results, storage()).unwrap();
    let row_capacity = baseline
        .bytes()
        .split_inclusive(|b| *b == b'\n')
        .take(2)
        .map(|b| b.len())
        .max()
        .unwrap();
    let mut s = storage();
    s.queue_bytes = row_capacity;
    s.backpressure = Backpressure::Fail;
    assert!(matches!(r.persist(&results, s), Err(Error::Overflow)));
    s.backpressure = Backpressure::Block;
    // First write occurs from append of the second episode; third writes seal.
    for fail_write in [1, 2, 3] {
        assert!(matches!(
            r.persist_with_sink(
                &results,
                s,
                Fault {
                    fail_write,
                    ..Fault::default()
                }
            ),
            Err(Error::Io(_))
        ));
        assert_eq!(results[1].accepted_decisions(), 2);
    }
    let b = r.persist(&results, s).unwrap();
    assert_eq!(b.bytes(), baseline.bytes());
    assert_eq!(b.metrics().batches, 2);
    assert!(b.metrics().buffer_high_water <= row_capacity);
}
struct Corrupt {
    bytes: Vec<u8>,
    mode: u8,
}
impl AsRef<[u8]> for Corrupt {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl Write for Corrupt {
    fn write(&mut self, b: &[u8]) -> io::Result<usize> {
        self.bytes.extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        // Declared synthetic broken sink, after actual played episodes were written.
        let mut rows: Vec<serde_json::Value> = self
            .bytes
            .split(|b| *b == b'\n')
            .filter(|b| !b.is_empty())
            .map(|b| serde_json::from_slice(b).unwrap())
            .collect();
        match self.mode {
            0 => {
                self.bytes[0] = b'!';
                return Ok(());
            }
            1 => {
                rows[0]["episode"]["header"]
                    .as_object_mut()
                    .unwrap()
                    .remove("config_hash");
            }
            2 => rows[0]["episode"]["header"]["versions"]["schema"] = 99.into(),
            3 => {
                rows.pop();
                self.bytes = rows
                    .iter()
                    .flat_map(|r| {
                        let mut b = serde_json::to_vec(r).unwrap();
                        b.push(b'\n');
                        b
                    })
                    .collect();
                return Ok(());
            }
            _ => rows[0]["episode"]["header"]["policies"][0] = "wrong-policy".into(),
        }
        let mut bytes = serde_json::to_vec(&rows[0]).unwrap();
        bytes.push(b'\n');
        rows[1]["sha256"] = format!("{:x}", Sha256::digest(&bytes)).into();
        bytes.extend(serde_json::to_vec(&rows[1]).unwrap());
        bytes.push(b'\n');
        self.bytes = bytes;
        Ok(())
    }
}
#[test]
fn corrupt_fields_versions_checksum_unsealed_and_late_changed_provenance_never_validate() {
    let r = run(1, false);
    let results = vec![result(&r, 7, "completed")];
    for mode in 0..5 {
        assert!(
            r.persist_with_sink(
                &results,
                storage(),
                Corrupt {
                    bytes: vec![],
                    mode
                }
            )
            .is_err()
        );
        assert_eq!(results[0].trajectory().unwrap().decisions().len(), 2);
    }
    let b = r.persist(&results, storage()).unwrap();
    let mut m = b.manifest().clone();
    m.file.sha256 = "0".repeat(64);
    assert!(matches!(
        m.load_v2(
            &m.file.name,
            b.bytes(),
            &m.versions,
            4_000_000,
            LoadMode::Diagnostic
        ),
        Err(Error::Integrity)
    ));
}
#[test]
fn zero_rows_and_all_unfinished_runs_still_require_valid_seals() {
    let r = run(0, false);
    let b = r.persist(&[], storage()).unwrap();
    assert_eq!(
        load(&b, LoadMode::CompletedOnly).unwrap().episodes().len(),
        0
    );
    let r = run(1, false);
    let b = r
        .persist(&[result(&r, 7, "incomplete")], storage())
        .unwrap();
    assert!(!b.manifest().recording_complete);
    assert_eq!(b.manifest().file.episodes, 0);
    assert!(load(&b, LoadMode::CompletedOnly).is_err());
    assert_eq!(load(&b, LoadMode::Diagnostic).unwrap().episodes().len(), 0);
    let mut invalid = r.clone();
    invalid.first_ordinal = u64::MAX;
    invalid.started = 2;
    assert!(
        invalid
            .persist(
                &[result(&r, 7, "completed"), result(&r, 8, "completed")],
                storage()
            )
            .is_err()
    );
}

#[test]
fn budget_stop_during_internal_reset_is_accounted_as_recording_incomplete() {
    use std::{cell::Cell, rc::Rc};
    #[derive(Debug, Clone)]
    struct Manual(Rc<Cell<u64>>);
    impl Clock for Manual {
        fn now_ms(&self) -> u64 {
            self.0.get()
        }
    }
    let mut r = run(1, false);
    r.limits.wall_time_ms = Some(5);
    let clock = Manual(Rc::new(Cell::new(0)));
    let mut d = Driver::bounded(
        256,
        Budget {
            limits: r.limits.clone(),
            work_quantum: NonZeroUsize::MIN,
            records: NonZeroUsize::new(100).unwrap(),
        },
        Box::new(clock.clone()),
    )
    .unwrap();
    d.reset_captured(&r.config, 163, 7, NonZeroUsize::MIN, &r.header(7).unwrap())
        .unwrap();
    clock.0.set(5);
    d.advance(NonZeroUsize::MIN).unwrap();
    let result = d.finish().unwrap();
    assert_eq!(
        result.status(),
        Status::Truncated(mtg_core::trajectory::Limit::WallTime)
    );
    assert!(result.final_observations().is_none());
    let b = r
        .persist(std::slice::from_ref(&result), storage())
        .expect("internal-work stop must retain explicit incomplete accounting");
    assert_eq!(b.manifest().episodes[0].status, EpisodeStatus::Incomplete);
    assert!(!b.manifest().recording_complete);
    assert!(load(&b, LoadMode::CompletedOnly).is_err());
    assert_eq!(load(&b, LoadMode::Diagnostic).unwrap().episodes().len(), 0);
    assert_eq!(
        result.status(),
        Status::Truncated(mtg_core::trajectory::Limit::WallTime)
    );
}
