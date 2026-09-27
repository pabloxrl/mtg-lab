//! Independent synthetic ledger: P0 passes, P1 concedes; P0 +1, P1 -1.
//! RFC 0002 B036/B037: exact records, intact identities, explicit loss/failure.
use super::*;
use serde_json::{Value, json};
fn fixture() -> Episode {
    serde_json::from_str(include_str!("../tests/episode.json")).unwrap()
}
fn bytes(e: &Episode) -> Vec<u8> {
    let mut w = Writer::new(Vec::new(), 65536, Backpressure::Block).unwrap();
    w.append(e).unwrap();
    w.finish().unwrap()
}
#[test]
fn jsonl_hand_authored_roundtrip_and_canonical_bytes() {
    let e = fixture();
    let b = bytes(&e);
    assert_eq!(read(b.as_slice(), 65536).unwrap(), vec![e]);
    assert_eq!(b, include_bytes!("../tests/episode.jsonl"));
}
#[test]
fn jsonl_rejects_corruption_truncation_and_whole_record_loss() {
    let good = include_bytes!("../tests/episode.jsonl");
    assert!(read(good.as_slice(), 65536).is_ok());
    for end in [0, 1, good.len() / 2, good.len() - 1] {
        assert!(read(&good[..end], 65536).is_err(), "prefix {end}");
    }
    let mut corrupt = good.to_vec();
    let p = corrupt.windows(8).position(|s| s == b"script-a").unwrap();
    corrupt[p] = b'X';
    assert!(read(corrupt.as_slice(), 65536).is_err());
    let end = good.iter().position(|&b| b == b'\n').unwrap() + 1;
    assert!(read(&good[end..], 65536).is_err(), "removed whole episode");
    let mut trailing = good.to_vec();
    trailing.extend_from_slice(b"{}\n");
    assert!(read(trailing.as_slice(), 65536).is_err());
    assert!(matches!(read(good.as_slice(), 10), Err(Error::Limit)));
}
#[test]
fn jsonl_semantic_validation_does_not_trust_checksum() {
    let base: Value = serde_json::to_value(fixture()).unwrap();
    for (pointer, value) in [
        ("/header/versions/schema", json!(999)),
        ("/header/id/run", json!("not-a-uuid")),
        ("/header/starting_seat", json!(2)),
        ("/decisions/0/index", json!(1)),
        ("/decisions/0/episode/ordinal", json!(8)),
        ("/decisions/0/seat_index", json!(1)),
        ("/decisions/0/actor", json!(2)),
        ("/decisions/0/observation/seat", json!(1)),
        ("/decisions/0/choice/legal_mask/0", json!(false)),
        ("/decisions/0/choice/micro_choice", json!(1)),
        ("/decisions/0/action", json!("different")),
        ("/decisions/0/reward", json!([1, -1])),
        ("/decisions/0/terminated", json!(false)),
        ("/footer/returns", json!([-1, 1])),
        ("/footer/decisions", json!(2)),
        ("/footer/logical_actions", json!(2)),
        ("/footer/complete", json!(false)),
        ("/footer/final_observations/1/seat", json!(0)),
        ("/footer/final_observations/1/terminal/winner", json!(1)),
    ] {
        let mut v = base.clone();
        *v.pointer_mut(pointer).unwrap() = value;
        let e: Episode = serde_json::from_value(v).unwrap();
        assert!(validate(&e).is_err(), "accepted {pointer}");
    }
    let mut e = fixture();
    e.footer = None;
    assert!(validate(&e).is_err());
    let mut e = fixture();
    e.footer.as_mut().unwrap().end = schema::End::Failed("disk full".into());
    assert!(validate(&e).is_err());
}
#[test]
fn jsonl_duplicate_episode_aborts_recording() {
    let mut w = Writer::new(Vec::new(), 65536, Backpressure::Block).unwrap();
    w.append(&fixture()).unwrap();
    assert!(w.append(&fixture()).is_err());
    assert!(w.finish().is_err());
}
#[test]
fn jsonl_bounded_overflow_and_write_failure_are_sticky() {
    let e = fixture();
    let mut tiny = Writer::new(Vec::new(), 10, Backpressure::Block).unwrap();
    assert!(matches!(tiny.append(&e), Err(Error::Limit)));
    assert!(tiny.finish().is_err());
    let mut fail = Writer::new(Vec::new(), 4096, Backpressure::Fail).unwrap();
    fail.append(&e).unwrap();
    let mut next = e.clone();
    next.header.id.ordinal += 1;
    next.decisions[0].episode = next.header.id.clone();
    assert!(matches!(fail.append(&next), Err(Error::Overflow)));
    assert!(fail.finish().is_err());
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("disk full"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut w = Writer::new(Broken, 65536, Backpressure::Block).unwrap();
    w.append(&e).unwrap();
    assert!(matches!(w.finish(), Err(Error::Io(_))));
}
#[test]
fn jsonl_real_episode_capture_off_matches_and_owned_records_survive_reset() {
    use mtg_core::{
        objects::Seat,
        opening::{
            Config, Game,
            turns::{TurnAction, TurnSelection},
        },
        trajectory as core,
    };
    let mut games = [Game::new().unwrap(), Game::new().unwrap()];
    for g in &mut games {
        g.reset(&Config::default(), 773, 21).unwrap();
    }
    let h = core::Header {
        id: core::EpisodeKey {
            run: fixture().header.id.run,
            ordinal: 21,
        },
        versions: core::Versions {
            schema: 1,
            engine: "test".into(),
            rules: "cr-20260925".into(),
            cards: "pool-v1".into(),
            action: "test".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["script".into(), "script".into()],
        starting_seat: 0,
        limits: core::Limits::default(),
        restricted_replay: None,
    };
    let mut previous = core::Frame::capture(&games[0]).unwrap();
    let mut recorder = core::Recorder::new(&h, &previous).unwrap();
    for i in 0..6 {
        let (kind, candidates) = if i < 2 {
            let d = games[0].decision().unwrap();
            let v = games[0].observe(d.actor).unwrap().opening.unwrap();
            (
                v.kind.to_string(),
                v.candidates
                    .iter()
                    .enumerate()
                    .map(|(i, s)| core::Candidate {
                        semantic: s.to_string(),
                        features: vec![i as i64],
                    })
                    .collect(),
            )
        } else {
            (
                "priority".into(),
                vec![core::Candidate {
                    semantic: "pass".into(),
                    features: vec![0],
                }],
            )
        };
        for g in &mut games {
            if i < 2 {
                let d = g.decision().unwrap();
                let o = g.observe(d.actor).unwrap().opening.unwrap();
                g.apply_opening_view(d.actor, o.generation, &[0]).unwrap();
                if i == 1 {
                    g.start_turns().unwrap();
                }
            } else {
                let d = g.turn_decision().unwrap();
                g.apply_turn(
                    d.actor,
                    &TurnAction {
                        decision: d.id,
                        selection: TurnSelection::Pass(d.candidate(0)),
                    },
                )
                .unwrap();
            }
        }
        let after = core::Frame::capture(&games[0]).unwrap();
        let c = core::Choice {
            kind,
            logical_action: i,
            micro_choice: 0,
            legal_mask: vec![true; candidates.len()],
            candidates,
            selected: 0,
            policy: core::PolicyInfo::default(),
        };
        recorder.append(&previous, &c, &after).unwrap();
        previous = after;
    }
    for g in &mut games {
        g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
    }
    recorder
        .finish(
            &core::Frame::capture(&games[0]).unwrap(),
            core::End::Completed,
        )
        .unwrap();
    fn semantic(g: &Game) -> Value {
        let envelope: Value = serde_json::from_slice(&g.snapshot()).unwrap();
        let mut v: Value = serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
        fn normalize(v: &mut Value) {
            match v {
                Value::Object(m) => {
                    for (k, v) in m {
                        if k == "scope" || k == "store" {
                            *v = json!(0)
                        } else {
                            normalize(v)
                        }
                    }
                }
                Value::Array(a) => {
                    for v in a {
                        normalize(v)
                    }
                }
                _ => (),
            }
        }
        normalize(&mut v);
        v["objects"]["id"] = json!(0);
        v
    }
    assert_eq!(semantic(&games[0]), semantic(&games[1]));
    let e = from_core(recorder.episode()).unwrap();
    assert_eq!(e.decisions.len(), 6);
    assert_eq!(e.footer.as_ref().unwrap().returns, [1, -1]);
    let b = bytes(&e);
    games[0].reset(&Config::default(), 123, 22).unwrap();
    let loaded = read(b.as_slice(), 65536).unwrap();
    assert_eq!(
        serde_json::to_value(&loaded[0]).unwrap(),
        serde_json::to_value(recorder.episode()).unwrap()
    );
    assert_eq!(loaded[0], e);
}
#[test]
fn jsonl_order_continuations_truncation_and_zero_decisions() {
    let mut e = fixture();
    e.decisions.clear();
    let f = e.footer.as_mut().unwrap();
    f.decisions = 0;
    f.logical_actions = 0;
    assert_eq!(read(bytes(&e).as_slice(), 65536).unwrap(), vec![e.clone()]);
    for limit in [Limit::Decisions, Limit::Turns, Limit::WallTime] {
        let f = e.footer.as_mut().unwrap();
        f.end = End::Truncated(limit);
        f.returns = [0, 0];
        f.boundary_reward = [0, 0];
        for v in &mut f.final_observations {
            v.terminal = None;
            v.acting_seat = Some(0);
        }
        assert_eq!(read(bytes(&e).as_slice(), 65536).unwrap(), vec![e.clone()]);
    }
    let mut e = fixture();
    let d = e.decisions[0].clone();
    for (i, actor) in [0, 0, 1, 1, 0, 1].into_iter().enumerate() {
        let mut row = d.clone();
        row.index = i;
        row.actor = actor;
        row.observation.seat = actor;
        row.observation.acting_seat = Some(actor);
        row.seat_index = [0, 1, 0, 1, 2, 2][i];
        row.choice.logical_action = if i < 3 { 0 } else { (i - 2) as u64 };
        row.choice.micro_choice = if i < 3 { i as u64 } else { 0 };
        row.terminated = i == 5;
        row.next_actor = if i == 5 {
            None
        } else {
            Some([0, 1, 1, 0, 1][i])
        };
        if i == 0 {
            e.decisions.clear();
        }
        e.decisions.push(row);
    }
    let f = e.footer.as_mut().unwrap();
    f.decisions = 6;
    f.logical_actions = 4;
    assert_eq!(read(bytes(&e).as_slice(), 65536).unwrap(), vec![e.clone()]);
    e.decisions.swap(1, 2);
    assert!(validate(&e).is_err());
}
#[test]
fn jsonl_blocking_buffer_drains_without_dropping_and_reports_wait() {
    let mut w = Writer::new(Vec::new(), 4096, Backpressure::Block).unwrap();
    let mut expected = vec![];
    for n in 7..12 {
        let mut e = fixture();
        e.header.id.ordinal = n;
        e.decisions[0].episode = e.header.id.clone();
        w.append(&e).unwrap();
        expected.push(e);
    }
    assert!(w.metrics().batches >= 1);
    assert!(w.metrics().buffer_high_water <= 4096);
    assert!(w.metrics().bytes > 0);
    let b = w.finish().unwrap();
    assert_eq!(read(b.as_slice(), 65536).unwrap(), expected);
}
#[test]
fn jsonl_flush_partial_write_and_read_errors_propagate() {
    struct Partial {
        bytes: Vec<u8>,
    }
    impl Write for Partial {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if self.bytes.len() >= 25 {
                return Err(std::io::Error::other("full"));
            }
            let n = b.len().min(25);
            self.bytes.extend_from_slice(&b[..n]);
            Ok(n)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut sink = Partial { bytes: vec![] };
    let mut w = Writer::new(&mut sink, 4096, Backpressure::Block).unwrap();
    w.append(&fixture()).unwrap();
    assert!(matches!(w.flush(), Err(Error::Io(_))));
    assert!(matches!(w.append(&fixture()), Err(Error::Poisoned)));
    assert!(w.finish().is_err());
    assert!(read(sink.bytes.as_slice(), 65536).is_err());
    struct FlushError;
    impl Write for FlushError {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("flush failed"))
        }
    }
    let w = Writer::new(FlushError, 4096, Backpressure::Block).unwrap();
    assert!(matches!(w.finish(), Err(Error::Io(_))));
    struct ReadError;
    impl Read for ReadError {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("read failed"))
        }
    }
    assert!(matches!(read(ReadError, 65536), Err(Error::Io(_))));
}
#[test]
fn jsonl_duplicate_unknown_missing_fields_and_mixed_run_are_rejected() {
    let good = include_str!("../tests/episode.jsonl");
    for bad in [
        good.replacen(
            "\"kind\":\"episode\"",
            "\"kind\":\"episode\",\"kind\":\"episode\"",
            1,
        ),
        good.replacen("\"seat\":0", "\"private_seed\":42,\"seat\":0", 1),
        good.replacen("\"restricted_replay\":null,", "", 1),
    ] {
        assert!(read(bad.as_bytes(), 65536).is_err());
    }
    let mut e = fixture();
    let mut w = Writer::new(Vec::new(), 65536, Backpressure::Block).unwrap();
    w.append(&e).unwrap();
    e.header.id.run = "b34c952c-723c-44ef-95f9-dcdb066db576".into();
    e.decisions[0].episode = e.header.id.clone();
    assert!(w.append(&e).is_err());
    let b = Writer::new(Vec::new(), 1, Backpressure::Block)
        .unwrap()
        .finish()
        .unwrap();
    assert!(read(b.as_slice(), 65536).unwrap().is_empty());
}
#[test]
fn jsonl_atomic_publication_preserves_failed_fragment_and_existing_file() {
    let directory = std::env::temp_dir().join(format!(
        "mtg-jsonl-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let target = directory.join("data.jsonl");
    write_file(&target, &[fixture()], 4096, Backpressure::Block).unwrap();
    assert!(
        target.exists(),
        "successful publication must create the final file"
    );
    let saved = std::fs::read(&target).unwrap();
    assert_eq!(read(saved.as_slice(), 65536).unwrap(), vec![fixture()]);
    assert!(write_file(&target, &[], 4096, Backpressure::Block).is_err());
    assert_eq!(std::fs::read(&target).unwrap(), saved);
    let failed = directory.join("failed.jsonl");
    let mut invalid = fixture();
    invalid.footer = None;
    assert!(write_file(&failed, &[invalid], 4096, Backpressure::Block).is_err());
    assert!(!failed.exists());
    let partial = directory.join("failed.jsonl.partial");
    assert!(partial.exists());
    assert!(read(std::fs::File::open(partial).unwrap(), 65536).is_err());
    std::fs::remove_dir_all(directory).unwrap();
}
#[test]
fn jsonl_interrupted_writer_leaves_unsealed_prefix() {
    let mut sink = Vec::new();
    {
        let mut w = Writer::new(&mut sink, 4096, Backpressure::Block).unwrap();
        w.append(&fixture()).unwrap();
        w.flush().unwrap();
        // Deliberately abandon without finish, just as a killed producer would.
    }
    assert!(!sink.is_empty());
    assert!(matches!(
        read(sink.as_slice(), 65536),
        Err(Error::Incomplete)
    ));
}
#[test]
fn jsonl_terminal_action_draw_and_nonempty_truncation_remain_distinct() {
    let mut e = fixture();
    e.decisions[0].reward = [1, -1];
    e.footer.as_mut().unwrap().boundary_reward = [0, 0];
    assert_eq!(read(bytes(&e).as_slice(), 65536).unwrap(), vec![e.clone()]);
    e.decisions[0].reward = [0, 0];
    let f = e.footer.as_mut().unwrap();
    f.returns = [0, 0];
    for v in &mut f.final_observations {
        let t = v.terminal.as_mut().unwrap();
        t.winner = None;
        t.losses = [Some("life".into()), Some("life".into())];
    }
    assert_eq!(read(bytes(&e).as_slice(), 65536).unwrap(), vec![e.clone()]);
    e.decisions[0].terminated = false;
    e.decisions[0].truncated = true;
    let f = e.footer.as_mut().unwrap();
    f.end = End::Truncated(Limit::Decisions);
    for v in &mut f.final_observations {
        v.terminal = None;
        v.acting_seat = Some(0);
    }
    assert_eq!(read(bytes(&e).as_slice(), 65536).unwrap(), vec![e.clone()]);
    e.decisions[0].choice.policy.log_probability = Some(f64::NAN);
    assert!(validate(&e).is_err());
}
#[test]
fn jsonl_backpressure_waits_for_sink_before_accepting_next_episode() {
    use std::sync::mpsc::{TryRecvError, channel};
    let (entered_tx, entered_rx) = channel();
    let (release_tx, release_rx) = channel();
    let (done_tx, done_rx) = channel();
    struct Gated {
        entered: std::sync::mpsc::Sender<()>,
        release: std::sync::mpsc::Receiver<()>,
        bytes: Vec<u8>,
    }
    impl Write for Gated {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            if self.bytes.is_empty() {
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
    let worker = std::thread::spawn(move || {
        let mut w = Writer::new(
            Gated {
                entered: entered_tx,
                release: release_rx,
                bytes: vec![],
            },
            4096,
            Backpressure::Block,
        )
        .unwrap();
        let mut e = fixture();
        w.append(&e).unwrap();
        e.header.id.ordinal += 1;
        e.decisions[0].episode = e.header.id.clone();
        w.append(&e).unwrap();
        assert!(w.metrics().write_wait > Duration::ZERO);
        done_tx.send(()).unwrap();
        w.finish().unwrap().bytes
    });
    entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(matches!(done_rx.try_recv(), Err(TryRecvError::Empty)));
    release_tx.send(()).unwrap();
    let bytes = worker.join().unwrap();
    assert_eq!(read(bytes.as_slice(), 65536).unwrap().len(), 2);
}
#[test]
fn jsonl_final_metrics_include_the_final_batch_and_seal() {
    let mut w = Writer::new(Vec::new(), 65536, Backpressure::Block).unwrap();
    w.append(&fixture()).unwrap();
    let (b, m) = w.finish_with_metrics().unwrap();
    assert_eq!(m.bytes, b.len() as u64);
    assert_eq!(m.batches, 1);
    assert!(m.write_wait > Duration::ZERO);
}
