//! RFC 0002 §8 and #152's documented v2 contract. Literal synthetic decision
//! boundaries, followed by an external P1 concession: +1/-1 exactly once.
//! These fixtures test storage, not played-game collection or engine legality.
use mtg_recorder::{Backpressure, Writer, manifest::Manifest, read_v2, structured};
use serde_json::{Value, json};
const DATA: &[u8] = include_bytes!("structured.jsonl");
fn fixtures() -> Vec<structured::Episode> {
    serde_json::from_str(include_str!("structured.json")).unwrap()
}
#[test]
fn structured_literal_records_validate_and_read_exactly() {
    let episodes = fixtures();
    for e in &episodes {
        structured::validate(e).unwrap();
    }
    assert_eq!(read_v2(DATA, 200_000).unwrap(), episodes);
}
#[test]
fn structured_existing_writer_matches_independent_python_bytes() {
    let mut w = Writer::new_v2(Vec::new(), 200_000, Backpressure::Block).unwrap();
    for e in fixtures() {
        w.append_v2(&e).unwrap();
    }
    assert_eq!(w.finish().unwrap(), DATA);
}
#[test]
fn structured_manifest_version_is_explicit() {
    let m = Manifest::parse(
        include_bytes!("structured-manifest.json").as_slice(),
        20_000,
    )
    .unwrap();
    assert_eq!(m.dataset_schema, 2);
    assert_eq!(m.versions.schema, 2);
    assert_eq!(m.file.format, 2);
}
// Re-seal intentional corruptions independently: checksum success must never
// turn a malformed or unauthorized policy input into usable training data.
fn sealed(v: &Value) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let mut row = serde_json::to_vec(&json!({"kind":"episode","episode":v})).unwrap();
    row.push(b'\n');
    let seal = json!({"kind":"seal","format":2,"episodes":1,"decisions":1,"sha256":format!("{:x}",Sha256::digest(&row))});
    row.extend(serde_json::to_vec(&seal).unwrap());
    row.push(b'\n');
    row
}
#[test]
fn structured_missing_extra_invalid_fields_and_versions_reject() {
    let base: Value =
        serde_json::from_str::<Value>(include_str!("structured.json")).unwrap()[0].clone();
    for (ptr, value) in [
        ("/header/versions/schema", json!(1)),
        ("/header/versions/schema", json!(99)),
        ("/decisions/0/observation/schema_version", json!(2)),
        ("/decisions/0/observation/decision/legal_mask", json!([])),
        ("/decisions/0/observation/decision/actor", json!(1)),
        ("/decisions/0/choice/submission/schema_version", json!(99)),
        ("/decisions/0/choice/submission/revision", json!(9)),
        ("/decisions/0/choice/submission/choices", json!([])),
        ("/decisions/0/choice/status", json!("Committed")),
        ("/decisions/0/observation/pending/card/row", json!(99)),
        ("/footer/cancelled_actions", json!(0)),
        ("/footer/complete", json!(false)),
        (
            "/footer/final_observations/1/pending",
            base["decisions"][0]["observation"]["pending"].clone(),
        ),
    ] {
        let mut v = base.clone();
        *v.pointer_mut(ptr).unwrap() = value;
        assert!(read_v2(sealed(&v).as_slice(), 200_000).is_err(), "{ptr}");
    }
    for ptr in [
        "/decisions/0/observation",
        "/decisions/0/observation/decision",
        "/decisions/0/observation/pending",
        "/decisions/0/choice/submission",
    ] {
        let mut v = base.clone();
        v.pointer_mut(ptr)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("private_seed".into(), json!(123));
        assert!(
            read_v2(sealed(&v).as_slice(), 200_000).is_err(),
            "extra {ptr}"
        );
    }
    for (ptr, key) in [
        ("/decisions/0/observation", "pending"),
        ("/decisions/0/observation/decision", "factored"),
        ("/decisions/0/observation/decision", "legal_mask"),
        ("/decisions/0/choice", "submission"),
        ("/decisions/0/choice/submission", "choices"),
    ] {
        let mut v = base.clone();
        v.pointer_mut(ptr)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(
            read_v2(sealed(&v).as_slice(), 200_000).is_err(),
            "missing {key}"
        );
    }
}
fn manifest() -> Manifest {
    Manifest::parse(
        include_bytes!("structured-manifest.json").as_slice(),
        20_000,
    )
    .unwrap()
}
#[test]
fn structured_manifest_roundtrip_provenance_integrity_and_seat_isolation() {
    use mtg_recorder::manifest::LoadMode;
    let m = manifest();
    assert_eq!(
        Manifest::parse(m.encode(20_000).unwrap().as_slice(), 20_000).unwrap(),
        m
    );
    let run = m
        .load_v2(
            "structured.jsonl",
            DATA,
            &m.versions,
            200_000,
            LoadMode::CompletedOnly,
        )
        .unwrap();
    assert_eq!(run.episodes(), fixtures());
    assert_eq!(run.policy_decisions(0).unwrap().count(), 7);
    assert_eq!(run.policy_decisions(1).unwrap().count(), 0);
    assert!(run.policy_decisions(2).is_err());
    let rows =
        serde_json::to_string(&run.policy_decisions(0).unwrap().collect::<Vec<_>>()).unwrap();
    assert!(!rows.contains("mountain"));
    assert!(!rows.contains("restricted_replay"));
    assert!(
        m.load(
            "structured.jsonl",
            DATA,
            &m.versions,
            200_000,
            LoadMode::CompletedOnly
        )
        .is_err()
    );
    for (ptr, value) in [
        ("/dataset_schema", json!(3)),
        ("/file/format", json!(1)),
        ("/versions/schema", json!(1)),
        ("/versions/observation", json!(2)),
        ("/versions/action", json!("wrong")),
        ("/policies/0", json!("wrong")),
        ("/config_hash", json!("d".repeat(64))),
        ("/deck_hashes/1", json!("d".repeat(64))),
        ("/starting_seat", json!(1)),
        ("/limits/turns", json!(100)),
        ("/file/decisions", json!(6)),
        ("/file/episodes", json!(8)),
        ("/file/bytes", json!(1)),
        ("/file/sha256", json!("0".repeat(64))),
        ("/recording_complete", json!(false)),
    ] {
        let mut v = serde_json::to_value(&m).unwrap();
        *v.pointer_mut(ptr).unwrap() = value;
        let bad: Manifest = serde_json::from_value(v).unwrap();
        assert!(
            bad.load_v2(
                "structured.jsonl",
                DATA,
                &bad.versions,
                200_000,
                LoadMode::CompletedOnly
            )
            .is_err(),
            "{ptr}"
        );
    }
    let mut incomplete = m.clone();
    incomplete
        .episodes
        .push(mtg_recorder::manifest::DeclaredEpisode {
            ordinal: 7,
            status: mtg_recorder::manifest::EpisodeStatus::Incomplete,
        });
    incomplete.capture = mtg_recorder::manifest::Capture::AllEpisodes {
        first_ordinal: 0,
        count: 8,
    };
    incomplete.end = mtg_recorder::manifest::RunEnd::Truncated("interrupted".into());
    incomplete.recording_complete = false;
    assert!(
        incomplete
            .load_v2(
                "structured.jsonl",
                DATA,
                &m.versions,
                200_000,
                LoadMode::CompletedOnly
            )
            .is_err()
    );
    assert!(
        incomplete
            .load_v2(
                "structured.jsonl",
                DATA,
                &m.versions,
                200_000,
                LoadMode::Diagnostic
            )
            .is_ok()
    );
    assert!(
        incomplete
            .load_v2(
                "structured.jsonl",
                &DATA[..DATA.len() - 1],
                &m.versions,
                200_000,
                LoadMode::Diagnostic
            )
            .is_err()
    );
}
#[test]
fn structured_overflow_errors_interruption_and_version_mixing_never_seal() {
    use mtg_recorder::Error;
    use std::io::Write;
    let es = fixtures();
    let row_size = es[..2]
        .iter()
        .map(|e| {
            serde_json::to_vec(&json!({"kind":"episode","episode":e}))
                .unwrap()
                .len()
                + 1
        })
        .max()
        .unwrap();
    let mut w = Writer::new_v2(Vec::new(), row_size, Backpressure::Fail).unwrap();
    w.append_v2(&es[0]).unwrap();
    assert!(matches!(w.append_v2(&es[1]), Err(Error::Overflow)));
    assert!(matches!(w.finish(), Err(Error::Poisoned)));
    let mut w = Writer::new_v2(Vec::new(), 100, Backpressure::Block).unwrap();
    assert!(matches!(w.append_v2(&es[0]), Err(Error::Limit)));
    assert!(w.finish().is_err());
    let mut prefix = Vec::new();
    {
        let mut w = Writer::new_v2(&mut prefix, 200_000, Backpressure::Block).unwrap();
        w.append_v2(&es[0]).unwrap();
        w.flush().unwrap();
    }
    assert!(matches!(
        read_v2(prefix.as_slice(), 200_000),
        Err(Error::Incomplete)
    ));
    for n in [0, 1, DATA.len() / 2, DATA.len() - 1] {
        assert!(read_v2(&DATA[..n], 200_000).is_err());
    }
    struct Broken {
        bytes: Vec<u8>,
        flush_failure: bool,
    }
    impl Write for Broken {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if self.flush_failure {
                self.bytes.extend_from_slice(b);
                Ok(b.len())
            } else if self.bytes.is_empty() {
                self.bytes.extend_from_slice(&b[..10.min(b.len())]);
                Ok(10.min(b.len()))
            } else {
                Err(std::io::Error::other("disk full"))
            }
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("flush failed"))
        }
    }
    for flush_failure in [false, true] {
        let mut w = Writer::new_v2(
            Broken {
                bytes: vec![],
                flush_failure,
            },
            200_000,
            Backpressure::Block,
        )
        .unwrap();
        w.append_v2(&es[0]).unwrap();
        assert!(matches!(w.finish(), Err(Error::Io(_))));
    }
    let mut w = Writer::new(Vec::new(), 200_000, Backpressure::Block).unwrap();
    assert!(w.append_v2(&es[0]).is_err());
    assert!(w.finish().is_err());
    let legacy = serde_json::from_str(include_str!("episode.json")).unwrap();
    let mut w = Writer::new_v2(Vec::new(), 200_000, Backpressure::Block).unwrap();
    assert!(w.append(&legacy).is_err());
    assert!(w.finish().is_err());
    assert!(read_v2(include_bytes!("episode.jsonl").as_slice(), 200_000).is_err());
    assert!(mtg_recorder::read(DATA, 200_000).is_err());
}
#[test]
fn structured_buffer_ownership_and_optional_statistics() {
    let mut es = fixtures();
    let expected = es.clone();
    let mut w = Writer::new_v2(Vec::new(), 200_000, Backpressure::Block).unwrap();
    for e in &es {
        w.append_v2(e).unwrap();
    }
    es[0].decisions[0]
        .observation
        .pending
        .as_mut()
        .unwrap()
        .targets
        .clear();
    es.clear();
    let bytes = w.finish().unwrap();
    assert_eq!(read_v2(bytes.as_slice(), 200_000).unwrap(), expected);
    assert!(
        !String::from_utf8(bytes)
            .unwrap()
            .contains("log_probability")
    );
    for value in [-0.0, 1.23456789012345, f64::MIN_POSITIVE, f64::MAX] {
        let mut e = expected[0].clone();
        e.decisions[0].choice.policy.value = Some(value);
        e.decisions[0].choice.policy.log_probability = Some(-0.000000000000000123456789);
        let mut w = Writer::new_v2(Vec::new(), 200_000, Backpressure::Block).unwrap();
        w.append_v2(&e).unwrap();
        let bytes = w.finish().unwrap();
        let got = read_v2(bytes.as_slice(), 200_000).unwrap();
        assert_eq!(
            got[0].decisions[0].choice.policy.value.unwrap().to_bits(),
            value.to_bits()
        );
        assert_eq!(got[0], e);
    }
}
fn interleaved() -> structured::Episode {
    // Literal ledger: P0 cancels payment; P1 passes; P0 passes; P1 concedes.
    // No reward before concession. P0 transition spans two decisions then one;
    // P1 transition spans the final two. Boundary credit reaches each once.
    let mut e = fixtures().remove(0);
    let base = e.decisions[0].clone();
    for (i, actor, seat_index) in [(1, 1, 0), (2, 0, 1)] {
        let mut d = base.clone();
        d.index = i;
        d.actor = actor;
        d.seat_index = seat_index;
        d.observation = e.footer.as_ref().unwrap().final_observations[actor as usize].clone();
        d.observation.view.terminal = None;
        d.observation.view.acting_seat = Some(actor);
        d.observation.decision = Some(structured::Domain {
            revision: 10 + i as u64,
            generation: 20 + i as u64,
            actor,
            kind: "priority".into(),
            count: 1,
            candidates: vec![structured::Command::Pass],
            legal_mask: vec![true],
            factored: None,
        });
        d.choice.submission = structured::Submission {
            schema_version: 1,
            revision: 10 + i as u64,
            generation: 20 + i as u64,
            choices: vec![structured::Command::Pass],
        };
        d.choice.logical_action = i as u64;
        d.choice.status = structured::ActionStatus::Committed;
        e.decisions.push(d);
    }
    for i in 0..2 {
        e.decisions[i].terminated = false;
        e.decisions[i].next_actor = Some(e.decisions[i + 1].actor);
    }
    let f = e.footer.as_mut().unwrap();
    f.decisions = 3;
    f.logical_actions = 3;
    e
}
#[test]
fn structured_same_seat_readers_preserve_rewards_time_and_privacy() {
    let e = interleaved();
    structured::validate(&e).unwrap();
    let p0 = e.seat(0).unwrap();
    let p1 = e.seat(1).unwrap();
    assert_eq!(
        p0.transitions.iter().map(|t| t.reward).collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(p1.transitions[0].reward, -1);
    assert_eq!(
        p0.transitions
            .iter()
            .map(|t| t.decisions_elapsed)
            .collect::<Vec<_>>(),
        [2, 1]
    );
    assert_eq!(p1.transitions[0].decisions_elapsed, 2);
    assert_eq!(
        p0.transitions[0].next_observation,
        e.decisions[2].observation
    );
    assert_eq!(p0.transitions[0].cancelled_actions_elapsed, 1);
    assert!(!serde_json::to_string(&p0).unwrap().contains("mountain"));
    assert!(!serde_json::to_string(&p1).unwrap().contains("giant-growth"));
    assert!(e.seat(2).is_err());
    let no_action = fixtures()[0].seat(1).unwrap();
    assert!(no_action.transitions.is_empty());
    assert_eq!(no_action.unassigned_reward, -1);
}
#[test]
fn structured_factored_multichoice_temporal_and_seal_negatives() {
    let all: Value = serde_json::from_str(include_str!("structured.json")).unwrap();
    for (i, ptr, value) in [
        (
            1,
            "/decisions/0/choice/submission/choices/1",
            json!({"kind":"bottom","card":{"zone":"hand","row":1}}),
        ),
        (1, "/decisions/0/observation/decision/count", json!(1)),
        (
            3,
            "/decisions/0/choice/submission/choices/0/cards/1/row",
            json!(3),
        ),
        (
            3,
            "/decisions/0/choice/submission/choices/0/cards/1/row",
            json!(0),
        ),
        (
            4,
            "/decisions/0/choice/submission/choices/0/blocks/1/0/row",
            json!(0),
        ),
        (
            4,
            "/decisions/0/choice/submission/choices/0/blocks/1/1/row",
            json!(1),
        ),
        (
            5,
            "/decisions/0/choice/submission/choices/0/amounts/0/1",
            json!(2),
        ),
        (
            5,
            "/decisions/0/choice/submission/choices/0/amounts/1/0/row",
            json!(2),
        ),
        (
            5,
            "/decisions/0/observation/decision/factored/damage/0/attacker/row",
            json!(99),
        ),
        (5, "/decisions/0/observation/decision/factored", Value::Null),
        (
            0,
            "/decisions/0/observation/decision/legal_mask/12",
            json!(false),
        ),
        (0, "/decisions/0/choice/policy/log_probability", json!(0.1)),
    ] {
        let mut v = all[i].clone();
        if ptr.ends_with("log_probability") {
            v["decisions"][0]["choice"]["policy"]["log_probability"] = value;
        } else {
            *v.pointer_mut(ptr).unwrap() = value;
        }
        assert!(read_v2(sealed(&v).as_slice(), 200_000).is_err(), "{ptr}");
    }
    let mut sparse = all[5].clone();
    sparse["decisions"][0]["choice"]["submission"]["choices"][0]["amounts"] =
        json!([[{"zone":"battlefield","row":2},2]]);
    assert!(
        read_v2(sealed(&sparse).as_slice(), 200_000).is_ok(),
        "zero recipient may be omitted per policy-v1"
    );
    let good = interleaved();
    for (ptr, value) in [
        ("/decisions/1/choice/logical_action", json!(0)),
        ("/decisions/1/choice/micro_choice", json!(1)),
        ("/decisions/1/choice/submission/generation", json!(20)),
        ("/decisions/1/next_actor", json!(1)),
        ("/footer/logical_actions", json!(2)),
    ] {
        let mut v = serde_json::to_value(&good).unwrap();
        *v.pointer_mut(ptr).unwrap() = value;
        let e: structured::Episode = serde_json::from_value(v).unwrap();
        assert!(structured::validate(&e).is_err(), "{ptr}");
    }
    let mut lines: Vec<Value> = String::from_utf8(DATA.to_vec())
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    for (key, value) in [
        ("format", json!(1)),
        ("format", json!(99)),
        ("episodes", json!(6)),
        ("decisions", json!(6)),
        ("sha256", json!("0".repeat(64))),
    ] {
        let old = lines.last_mut().unwrap()[key].clone();
        lines.last_mut().unwrap()[key] = value;
        let mut bytes = Vec::new();
        for line in &lines {
            bytes.extend(serde_json::to_vec(line).unwrap());
            bytes.push(b'\n');
        }
        assert!(read_v2(bytes.as_slice(), 200_000).is_err(), "seal {key}");
        lines.last_mut().unwrap()[key] = old;
    }
}
#[test]
fn structured_blocking_is_measured_and_does_not_drop_rows() {
    use std::{io::Write, sync::mpsc, time::Duration};
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    struct Gated {
        bytes: Vec<u8>,
        entered: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
        first: bool,
    }
    impl Write for Gated {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if self.first {
                self.first = false;
                self.entered.send(()).unwrap();
                self.release.recv_timeout(Duration::from_secs(5)).unwrap();
            }
            self.bytes.extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let handle = std::thread::spawn(move || {
        let es = fixtures();
        let cap = es[..2]
            .iter()
            .map(|e| {
                serde_json::to_vec(&json!({"kind":"episode","episode":e}))
                    .unwrap()
                    .len()
                    + 1
            })
            .max()
            .unwrap();
        let mut w = Writer::new_v2(
            Gated {
                bytes: vec![],
                entered: entered_tx,
                release: release_rx,
                first: true,
            },
            cap,
            Backpressure::Block,
        )
        .unwrap();
        w.append_v2(&es[0]).unwrap();
        w.append_v2(&es[1]).unwrap();
        let (sink, metrics) = w.finish_with_metrics().unwrap();
        done_tx.send(()).unwrap();
        (sink.bytes, metrics)
    });
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(
        done_rx.recv_timeout(Duration::from_millis(20)),
        Err(mpsc::RecvTimeoutError::Timeout)
    ));
    release_tx.send(()).unwrap();
    let (bytes, metrics) = handle.join().unwrap();
    assert_eq!(read_v2(bytes.as_slice(), 200_000).unwrap(), fixtures()[..2]);
    assert!(metrics.write_wait >= Duration::from_millis(20));
    assert_eq!(metrics.batches, 2);
}
#[test]
fn structured_file_publication_is_atomic_and_retains_failed_fragments() {
    use mtg_recorder::write_file_v2;
    let root = std::env::temp_dir().join(format!("mtg-structured-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("run.jsonl");
    write_file_v2(&path, &fixtures(), 200_000, Backpressure::Block).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), DATA);
    assert!(write_file_v2(&path, &[], 200_000, Backpressure::Block).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), DATA);
    let bad = root.join("broken.jsonl");
    let mut e = fixtures()[0].clone();
    e.footer = None;
    assert!(write_file_v2(&bad, &[e], 200_000, Backpressure::Block).is_err());
    assert!(!bad.exists());
    assert!(root.join("broken.jsonl.partial").exists());
    assert!(
        read_v2(
            std::fs::File::open(root.join("broken.jsonl.partial")).unwrap(),
            200_000
        )
        .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn structured_core_conversion_and_reset_preserve_owned_final_inputs() {
    use mtg_core::{
        objects::Seat,
        opening::{Config, Game},
        trajectory as t,
    };
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 0).unwrap();
    let h = t::Header {
        id: t::EpisodeKey {
            run: fixtures()[0].header.id.run.clone(),
            ordinal: 0,
        },
        versions: t::Versions {
            schema: 2,
            engine: "test".into(),
            rules: "cr-20260925".into(),
            cards: "pool-v1".into(),
            action: "policy-v1".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["script".into(), "script".into()],
        starting_seat: 0,
        limits: t::Limits::default(),
        restricted_replay: None,
    };
    let frame = t::v2::Frame::capture(&g, 256).unwrap();
    let mut r = t::v2::Recorder::new(&h, &frame).unwrap();
    r.finish(&frame, t::End::Truncated(t::Limit::Decisions))
        .unwrap();
    let e = mtg_recorder::from_core_v2(r.episode()).unwrap();
    assert_eq!(
        serde_json::to_value(&e).unwrap(),
        serde_json::to_value(r.episode()).unwrap()
    );
    let expected = serde_json::to_value(g.policy_observe(Seat::P0, 256).unwrap()).unwrap();
    g.reset(&Config::default(), 43, 1).unwrap();
    assert_eq!(
        serde_json::to_value(e.seat(0).unwrap().final_observation).unwrap(),
        expected
    );
    let mut w = Writer::new_v2(Vec::new(), 200_000, Backpressure::Block).unwrap();
    w.append_v2(&e).unwrap();
    let b = w.finish().unwrap();
    assert_eq!(read_v2(b.as_slice(), 200_000).unwrap(), [e]);
}
#[test]
fn structured_kind_cardinality_and_command_grammar_reject_corruption() {
    // policy-v1: Bottom is a bottom decision, never a priority action. A Pay
    // submission uses exactly one color, never a multi-choice priority payload.
    let all: Value = serde_json::from_str(include_str!("structured.json")).unwrap();
    let mut v = all[1].clone();
    v["decisions"][0]["observation"]["decision"]["kind"] = json!("priority");
    assert!(
        read_v2(sealed(&v).as_slice(), 200_000).is_err(),
        "bottom mislabeled priority"
    );
    let mut v = all[0].clone();
    v["decisions"][0]["observation"]["decision"]["count"] = json!(2);
    v["decisions"][0]["choice"]["submission"]["choices"] =
        json!([{"kind":"pay","color":4},{"kind":"cancel_payment"}]);
    v["decisions"][0]["choice"]["status"] = json!("Continuing");
    v["footer"]["cancelled_actions"] = json!(0);
    assert!(
        read_v2(sealed(&v).as_slice(), 200_000).is_err(),
        "payment cardinality"
    );
}
#[test]
fn structured_nonempty_stack_provisional_and_departed_references_are_lossless() {
    let mut all: Value = serde_json::from_str(include_str!("structured.json")).unwrap();
    // Opaque departed target/source remains null. It must not resolve a hidden row.
    all[0]["decisions"][0]["observation"]["pending"]["targets"] = json!([null]);
    all[0]["decisions"][0]["observation"]["pending"]["sources"] =
        json!([null,{"zone":"battlefield","row":4}]);
    all[3]["decisions"][0]["observation"]["decision"]["factored"]["selected"] =
        json!([{"zone":"battlefield","row":0}]);
    all[4]["decisions"][0]["observation"]["decision"]["factored"]["blocks"] =
        json!([[{"zone":"battlefield","row":0},{"zone":"battlefield","row":3}]]);
    all[5]["decisions"][0]["observation"]["decision"]["factored"]["damage"][0]["amounts"] =
        json!([[{"zone":"battlefield","row":2},2]]);
    let o = &mut all[6]["decisions"][0]["observation"];
    o["view"]["public_zones"][1]["cards"] = json!([{"card":"giant-growth","owner":1,"controller":1,"tapped":false,"creature":null,"summoning_sick":false}]);
    o["stack"] = json!([{"row":0,"targets":[null,{"zone":"battlefield","row":1}]}]);
    for v in all.as_array().unwrap() {
        let expected: structured::Episode = serde_json::from_value(v.clone()).unwrap();
        assert_eq!(read_v2(sealed(v).as_slice(), 200_000).unwrap(), [expected]);
    }
}
#[test]
fn structured_native_payment_without_a_spell_preserves_null_pending() {
    // The existing mana integration boundary can reserve a cost without casting.
    // policy-v1 exposes that payment domain with pending=null, and v2 can capture it.
    use mtg_core::{
        game::policy,
        objects::Seat,
        opening::{Config, Game, mana::ManaCost},
        trajectory as t,
    };
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 0).unwrap();
    for seat in [Seat::P0, Seat::P1] {
        let d = g.policy_observe(seat, 256).unwrap().decision.unwrap();
        g.apply_policy(
            seat,
            &policy::Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices: vec![policy::Choice::Keep],
            },
            256,
        )
        .unwrap();
    }
    g.start_turns().unwrap();
    let d = g.turn_decision().unwrap();
    g.begin_payment(d.actor, d.id, ManaCost::default()).unwrap();
    let h = t::Header {
        id: t::EpisodeKey {
            run: fixtures()[0].header.id.run.clone(),
            ordinal: 0,
        },
        versions: t::Versions {
            schema: 2,
            engine: "test".into(),
            rules: "cr-20260925".into(),
            cards: "pool-v1".into(),
            action: "policy-v1".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["script".into(), "script".into()],
        starting_seat: 0,
        limits: t::Limits::default(),
        restricted_replay: None,
    };
    let frame = t::v2::Frame::capture(&g, 256).unwrap();
    let mut r = t::v2::Recorder::new(&h, &frame).unwrap();
    r.finish(&frame, t::End::Truncated(t::Limit::Decisions))
        .unwrap();
    let e = mtg_recorder::from_core_v2(r.episode()).unwrap();
    let o = &e.footer.as_ref().unwrap().final_observations[0];
    assert!(o.pending.is_none());
    assert_eq!(o.decision.as_ref().unwrap().kind, "payment");
    assert_eq!(
        serde_json::to_value(e).unwrap(),
        serde_json::to_value(r.episode()).unwrap()
    );
}
