//! RFC B008/B021, SYS-METRIC-001/002: literal normal-reset traces, not goldens.
use mtg_core::{
    episode::{Driver, Progress, RecordContext},
    game::{
        Config, DeckConfig,
        policy::{self, Choice, VisibleRef, VisibleZone},
    },
    metrics::Mode,
    objects::Seat,
    trajectory::{EpisodeKey, Header, Limits, Versions},
};
use serde_json::{Value, json};
use std::num::NonZeroUsize;

fn ready(d: &mut Driver) {
    while d.advance(NonZeroUsize::MAX).unwrap() == Progress::InternalYield {}
}
fn send(d: &mut Driver, seat: Seat, choice: Choice) {
    let decision = d.observe(seat).unwrap().decision.unwrap();
    d.submit(
        seat,
        &policy::Submission {
            schema_version: 1,
            revision: decision.revision,
            generation: decision.generation,
            choices: vec![choice.clone()],
        },
    )
    .unwrap_or_else(|error| {
        panic!(
            "{seat:?} {choice:?} at {:?}: {error:?}",
            d.observe(seat).unwrap().view.turn
        )
    });
}

#[test]
fn two_keeps_one_pass_and_concession_have_literal_counts() {
    let mut d = Driver::instrumented(256, Mode::Counters).unwrap();
    d.reset(&Config::default(), 214, 0, NonZeroUsize::MAX)
        .unwrap();
    send(&mut d, Seat::P0, Choice::Keep);
    send(&mut d, Seat::P1, Choice::Keep);
    ready(&mut d);
    send(&mut d, Seat::P0, Choice::Pass);
    d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
    d.finish().unwrap();
    let c = d.metrics().unwrap();
    assert_eq!((c.resets, c.started, c.completed), (1, 1, 1));
    assert_eq!((c.decisions, c.logical_actions, c.concessions), (3, 3, 1));
    assert_eq!(
        (c.decisions_by_kind.opening, c.decisions_by_kind.priority),
        (2, 1)
    );
    assert_eq!((c.rules_completed, c.committed_actions), (0, 3));
    assert_eq!(
        (c.failed, c.truncated, c.incomplete, c.cancelled_actions),
        (0, 0, 0, 0)
    );
    assert!(d.finish().is_err());
    assert_eq!(d.metrics().unwrap().completed, 1);
}

#[test]
fn off_is_absent_rather_than_measured_zero() {
    let mut d = Driver::instrumented(256, Mode::Off).unwrap();
    d.reset(&Config::default(), 214, 0, NonZeroUsize::MAX)
        .unwrap();
    send(&mut d, Seat::P0, Choice::Keep);
    assert!(d.metrics().is_none());
}

fn ordered() -> Config {
    // Literal frozen green deck. Ordering avoids RNG-dependent work counts.
    let inventory = [
        ("forest", 16),
        ("bear-cub", 4),
        ("giant-growth", 3),
        ("bite-down", 3),
        ("llanowar-elves", 3),
        ("druid-of-the-cowl", 2),
        ("magnigoth-sentry", 2),
        ("tajuru-pathwarden", 2),
        ("thornweald-archer", 3),
        ("wildheart-invoker", 2),
    ];
    let mut order = vec!["llanowar-elves".to_owned(), "forest".to_owned()];
    for (card, count) in inventory {
        for _ in order.iter().filter(|c| c.as_str() == card).count()..count {
            order.push(card.to_owned());
        }
    }
    Config {
        seats: vec![
            DeckConfig {
                deck: "green".into(),
                order: Some(order)
            };
            2
        ],
        ..Config::default()
    }
}
fn header() -> Header {
    Header {
        id: EpisodeKey {
            run: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
            ordinal: 0,
        },
        versions: Versions {
            schema: 2,
            engine: "test".into(),
            rules: "cr-20260925".into(),
            cards: "pool-v1".into(),
            action: "policy-v1".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["ledger".into(), "ledger".into()],
        starting_seat: 0,
        limits: Limits::default(),
        restricted_replay: None,
    }
}
fn hand(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Hand,
        row,
    }
}
fn normalized(bytes: &[u8]) -> Value {
    fn scrub(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "scope" || k == "store" {
                        *v = json!(0);
                    } else {
                        scrub(v);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    scrub(v);
                }
            }
            _ => (),
        }
    }
    let envelope: Value = serde_json::from_slice(bytes).unwrap();
    let mut v: Value = serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
    v["objects"]["id"] = json!(0);
    scrub(&mut v);
    v
}

#[test]
fn counts_capture_and_quantum_preserve_normal_play_and_rng() {
    let mut baseline = None;
    let mut baseline_capture = None;
    let mut baseline_counts = None;
    for mode in [Mode::Off, Mode::Counters] {
        for capture in [false, true] {
            for quantum in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
                let mut d = Driver::instrumented(256, mode).unwrap();
                if capture {
                    d.reset_captured(&ordered(), 214, 0, quantum, &header())
                        .unwrap();
                } else {
                    d.reset(&ordered(), 214, 0, quantum).unwrap();
                }
                while d.advance(quantum).unwrap() == Progress::InternalYield {}
                if mode == Mode::Counters {
                    // Two shuffle bypasses, 80 allocations, 14 opening draws.
                    assert_eq!(d.metrics().unwrap().rules_work_units, 96);
                }
                send(&mut d, Seat::P0, Choice::Keep);
                send(&mut d, Seat::P1, Choice::Keep);
                while d.advance(quantum).unwrap() == Progress::InternalYield {}
                send(&mut d, Seat::P0, Choice::Pass);
                send(&mut d, Seat::P1, Choice::Pass);
                // Authorized hands are sorted by card key, not library order:
                // six Forests then one Elf; after the land, Elf is row five.
                let h = d.observe(Seat::P0).unwrap().view.hand;
                assert_eq!(h[1].card, "forest");
                assert_eq!(h[6].card, "llanowar-elves");
                send(&mut d, Seat::P0, Choice::PlayLand { card: hand(1) });
                send(&mut d, Seat::P0, Choice::Cast { card: hand(5) });
                send(&mut d, Seat::P0, Choice::CancelPayment);
                send(&mut d, Seat::P0, Choice::Cast { card: hand(5) });
                send(
                    &mut d,
                    Seat::P0,
                    Choice::TapMana {
                        card: VisibleRef {
                            zone: VisibleZone::Battlefield,
                            row: 0,
                        },
                    },
                );
                send(&mut d, Seat::P0, Choice::Pay { color: 4 });
                send(&mut d, Seat::P0, Choice::FinishPayment);
                d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
                let r = d.finish().unwrap();
                let current = (
                    normalized(r.privileged_snapshot()),
                    r.privileged_history().to_vec(),
                    serde_json::to_value(r.final_observations()).unwrap(),
                );
                if let Some(b) = &baseline {
                    assert_eq!(&current, b);
                } else {
                    baseline = Some(current);
                }
                if let Some(e) = r.trajectory() {
                    let current = serde_json::to_value(e).unwrap();
                    assert_eq!(e.footer().unwrap().decisions, 11);
                    assert_eq!(e.footer().unwrap().logical_actions, 7);
                    assert_eq!(e.footer().unwrap().cancelled_actions, 1);
                    if let Some(b) = &baseline_capture {
                        assert_eq!(&current, b);
                    } else {
                        baseline_capture = Some(current);
                    }
                }
                if let Some(c) = d.metrics() {
                    assert_eq!(
                        (
                            c.decisions,
                            c.logical_actions,
                            c.cancelled_actions,
                            c.concessions
                        ),
                        (11, 7, 1, 1)
                    );
                    assert_eq!(c.committed_actions, 6);
                    assert_eq!(
                        (
                            c.decisions_by_kind.opening,
                            c.decisions_by_kind.priority,
                            c.decisions_by_kind.payment
                        ),
                        (2, 5, 4)
                    );
                    if let Some(b) = &baseline_counts {
                        assert_eq!(&c, b);
                    } else {
                        baseline_counts = Some(c);
                    }
                }
            }
        }
    }
}

#[test]
fn invalid_and_stale_inputs_count_once_without_mutating_state() {
    let mut d = Driver::instrumented(256, Mode::Counters).unwrap();
    d.reset(&ordered(), 214, 0, NonZeroUsize::MAX).unwrap();
    let v = d.observe(Seat::P0).unwrap().decision.unwrap();
    let mut input = policy::Submission {
        schema_version: 1,
        revision: v.revision,
        generation: v.generation,
        choices: vec![Choice::Pass],
    };
    let before = d.privileged_snapshot();
    assert!(d.submit(Seat::P0, &input).is_err());
    assert_eq!(d.privileged_snapshot(), before);
    input.revision += 1;
    assert!(d.submit(Seat::P0, &input).is_err());
    assert_eq!(d.privileged_snapshot(), before);
    let ctx = RecordContext::Decision {
        revision: v.revision,
        generation: v.generation,
    };
    assert!(
        d.submit_record(Seat::P0, ctx, b"hidden-error-sentinel")
            .is_err()
    );
    assert_eq!(d.privileged_snapshot(), before);
    send(&mut d, Seat::P0, Choice::Keep);
    let c = d.metrics().unwrap();
    assert_eq!(
        (
            c.invalid_actions,
            c.stale_actions,
            c.boundary_errors,
            c.decisions
        ),
        (2, 1, 3, 1)
    );
    let public = serde_json::to_string(&c).unwrap();
    for forbidden in [
        "hidden-error-sentinel",
        "llanowar",
        "forest",
        "seed",
        "episode",
        "a34c952c",
    ] {
        assert!(
            !public.contains(forbidden),
            "public metrics leaked {forbidden}"
        );
    }
}

#[derive(Debug)]
struct TestClock(std::rc::Rc<std::cell::Cell<u64>>);
impl mtg_core::episode::Clock for TestClock {
    fn now_ms(&self) -> u64 {
        self.0.get()
    }
}
fn bounded(limit: Option<u64>, records: usize) -> (Driver, std::rc::Rc<std::cell::Cell<u64>>) {
    let clock = std::rc::Rc::new(std::cell::Cell::new(10));
    let d = Driver::bounded_instrumented(
        256,
        mtg_core::episode::Budget {
            limits: Limits {
                decisions: limit,
                ..Limits::default()
            },
            work_quantum: NonZeroUsize::MAX,
            records: NonZeroUsize::new(records).unwrap(),
        },
        Box::new(TestClock(clock.clone())),
        Mode::Counters,
    )
    .unwrap();
    (d, clock)
}
#[test]
fn lifecycle_truncation_clock_failure_record_overflow_and_rejected_reset() {
    for case in 0..3 {
        let (mut d, clock) = bounded(if case == 0 { Some(1) } else { None }, 1);
        d.reset(&ordered(), 214, 0, NonZeroUsize::MAX).unwrap();
        let before = d.privileged_snapshot();
        assert!(d.reset(&ordered(), 214, 1, NonZeroUsize::MAX).is_err());
        assert_eq!(d.privileged_snapshot(), before);
        send(&mut d, Seat::P0, Choice::Keep);
        if case == 1 {
            clock.set(9);
            assert!(d.advance(NonZeroUsize::MAX).is_err());
        }
        if case == 2 {
            let v = d.observe(Seat::P1).unwrap().decision.unwrap();
            assert!(
                d.submit(
                    Seat::P1,
                    &policy::Submission {
                        schema_version: 1,
                        revision: v.revision,
                        generation: v.generation,
                        choices: vec![Choice::Keep]
                    }
                )
                .is_err()
            );
        }
        d.finish().unwrap();
        let c = d.metrics().unwrap();
        assert_eq!(
            (
                c.started,
                c.completed,
                c.incomplete,
                c.decisions,
                c.rejected_resets
            ),
            (1, 0, 0, 1, 1)
        );
        assert_eq!(
            (c.truncated, c.failed),
            (u64::from(case == 0), u64::from(case != 0))
        );
        assert_eq!(c.clock_failures, u64::from(case == 1));
        assert_eq!(c.capacity_overflows, u64::from(case == 2));
        d.reset(&ordered(), 214, 1, NonZeroUsize::MAX).unwrap();
        d.finish().unwrap();
        let c = d.metrics().unwrap();
        assert_eq!((c.resets, c.started, c.incomplete), (2, 2, 1));
    }
}

#[test]
fn hidden_identifiers_never_become_metric_keys_or_values() {
    let mut keys = None;
    for seed in [0, 1, 999999999, u64::MAX] {
        let mut d = Driver::instrumented(256, Mode::Counters).unwrap();
        d.reset(&Config::default(), seed, seed, NonZeroUsize::MAX)
            .unwrap();
        assert!(
            d.submit_record(
                Seat::P0,
                RecordContext::Concession {
                    episode: d.episode_id().unwrap()
                },
                b"PRIVATE_GAME_CARD_ERROR_SENTINEL"
            )
            .is_err()
        );
        d.finish().unwrap();
        let value = serde_json::to_value(d.metrics().unwrap()).unwrap();
        let current: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
        if let Some(ref before) = keys {
            assert_eq!(&current, before);
        } else {
            keys = Some(current);
        }
        let text = value.to_string();
        for private in [
            "PRIVATE_GAME_CARD_ERROR_SENTINEL",
            "999999999",
            "18446744073709551615",
            "forest",
            "seed",
            "run_id",
            "game_id",
        ] {
            assert!(!text.contains(private));
        }
    }
}

#[test]
fn counter_aggregation_saturates_visibly_instead_of_wrapping() {
    let mut c = mtg_core::metrics::Counters {
        started: u64::MAX,
        ..Default::default()
    };
    let delta = mtg_core::metrics::Counters {
        started: 1,
        decisions: 3,
        ..Default::default()
    };
    c.merge(&delta);
    assert_eq!(c.started, u64::MAX);
    assert_eq!(c.decisions, 3);
    assert!(c.overflowed);
}
#[test]
fn natural_empty_draw_completion_has_a_distinct_success_numerator() {
    let mut d = Driver::instrumented(256, Mode::Counters).unwrap();
    d.reset(&ordered(), 214, 0, NonZeroUsize::MAX).unwrap();
    send(&mut d, Seat::P0, Choice::Keep);
    send(&mut d, Seat::P1, Choice::Keep);
    for _ in 0..2000 {
        if d.status().is_some() {
            break;
        }
        if d.advance(NonZeroUsize::MAX).unwrap() == Progress::InternalYield {
            continue;
        }
        if d.status().is_some() {
            break;
        }
        let (seat, decision) = [Seat::P0, Seat::P1]
            .into_iter()
            .find_map(|s| d.observe(s).unwrap().decision.map(|v| (s, v)))
            .unwrap();
        let choices = if decision.kind == "cleanup_discard" {
            decision
                .candidates
                .into_iter()
                .take(decision.count)
                .collect()
        } else {
            vec![Choice::Pass]
        };
        d.submit(
            seat,
            &policy::Submission {
                schema_version: 1,
                revision: decision.revision,
                generation: decision.generation,
                choices,
            },
        )
        .unwrap();
    }
    let r = d.finish().unwrap();
    let mtg_core::episode::Status::Completed(outcome) = r.status() else {
        panic!("passive normal game must deck out");
    };
    assert_eq!(
        outcome.losses,
        [None, Some(mtg_core::game::terminal::LossReason::EmptyDraw)]
    );
    let c = d.metrics().unwrap();
    assert_eq!(
        (
            c.completed,
            c.rules_completed,
            c.concessions,
            c.failed,
            c.truncated
        ),
        (1, 1, 0, 0, 0)
    );
    assert_eq!(c.decisions, r.accepted_decisions());
}

#[derive(Debug)]
struct CountingClock(std::rc::Rc<std::cell::Cell<usize>>);
impl mtg_core::episode::Clock for CountingClock {
    fn now_ms(&self) -> u64 {
        self.0.set(self.0.get() + 1);
        0
    }
}
#[test]
fn instrumentation_adds_no_clock_reads_at_any_work_quantum() {
    for quantum in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
        let mut baseline = None;
        for mode in [Mode::Off, Mode::Counters] {
            let reads = std::rc::Rc::new(std::cell::Cell::new(0));
            let mut d = Driver::bounded_instrumented(
                256,
                mtg_core::episode::Budget {
                    limits: Limits::default(),
                    work_quantum: quantum,
                    records: NonZeroUsize::new(10).unwrap(),
                },
                Box::new(CountingClock(reads.clone())),
                mode,
            )
            .unwrap();
            d.reset(&ordered(), 214, 0, quantum).unwrap();
            while d.advance(quantum).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, Choice::Keep);
            d.finish().unwrap();
            assert!(
                reads.get() > 0,
                "positive control: owner uses injected boundary clock"
            );
            if let Some(n) = baseline {
                assert_eq!(reads.get(), n);
            } else {
                baseline = Some(reads.get());
            }
        }
    }
}

#[test]
fn semantic_submission_is_counted_once_and_stale_concession_preserves_reset() {
    let mut direct = Driver::instrumented(256, Mode::Counters).unwrap();
    let mut semantic = Driver::instrumented(256, Mode::Counters).unwrap();
    for d in [&mut direct, &mut semantic] {
        d.reset(&ordered(), 214, 0, NonZeroUsize::MAX).unwrap();
    }
    for seat in [Seat::P0, Seat::P1] {
        let v = semantic.observe(seat).unwrap().decision.unwrap();
        send(&mut direct, seat, Choice::Keep);
        semantic
            .submit_record(
                seat,
                RecordContext::Decision {
                    revision: v.revision,
                    generation: v.generation,
                },
                direct.privileged_history().last().unwrap(),
            )
            .unwrap();
    }
    assert_eq!(direct.metrics(), semantic.metrics());
    assert_eq!(direct.metrics().unwrap().decisions, 2);
    assert_eq!(
        normalized(&direct.privileged_snapshot()),
        normalized(&semantic.privileged_snapshot())
    );
    let old = semantic.episode_id().unwrap();
    semantic.finish().unwrap();
    semantic
        .reset(&ordered(), 214, 1, NonZeroUsize::MAX)
        .unwrap();
    let before = semantic.privileged_snapshot();
    assert!(semantic.concede(Seat::P0, old).is_err());
    assert_eq!(semantic.privileged_snapshot(), before);
    let c = semantic.metrics().unwrap();
    assert_eq!(
        (c.stale_actions, c.concessions, c.completed, c.incomplete),
        (1, 0, 0, 1)
    );
}
