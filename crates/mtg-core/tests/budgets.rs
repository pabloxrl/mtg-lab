//! GH-161. Normal resets; RFC 0002 episode/time semantics, CR 103.5/104.3a.
//! Literal keep/mulligan scripts independently count accepted policy decisions.
use mtg_core::{
    episode::{Budget, Clock, Driver, Progress, Status},
    game::{
        Config, DeckConfig,
        policy::{self, Choice as C},
    },
    objects::Seat,
    trajectory::{EpisodeKey, Header, Limits, Versions},
};
use serde_json::{Value, json};
use std::{cell::Cell, num::NonZeroUsize, rc::Rc};
#[derive(Debug, Clone)]
struct Manual(Rc<Cell<u64>>);
impl Clock for Manual {
    fn now_ms(&self) -> u64 {
        self.0.get()
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
fn config() -> Config {
    let prefix = [
        "bear-cub",
        "bear-cub",
        "giant-growth",
        "forest",
        "forest",
        "forest",
        "forest",
    ];
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
    let mut order: Vec<String> = prefix.into_iter().map(String::from).collect();
    for (key, count) in inventory {
        for _ in order.iter().filter(|x| x.as_str() == key).count()..count {
            order.push(key.into());
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

fn budget(n: u64) -> Budget {
    Budget {
        limits: Limits {
            decisions: Some(n),
            ..Limits::default()
        },
        work_quantum: NonZeroUsize::new(4).unwrap(),
        records: NonZeroUsize::new(100).unwrap(),
    }
}
fn ready(d: &mut Driver) {
    for _ in 0..500 {
        if d.advance(NonZeroUsize::MAX).unwrap() == Progress::Ready {
            return;
        }
    }
    panic!("bounded reset did not finish")
}
fn send(d: &mut Driver, seat: Seat, choice: C) -> Result<(), mtg_core::episode::Error> {
    let o = d.observe(seat).unwrap();
    let v = o.decision.unwrap();
    d.submit(
        seat,
        &policy::Submission {
            schema_version: 1,
            revision: v.revision,
            generation: v.generation,
            choices: vec![choice],
        },
    )
}
#[test]
fn literal_one_decision_budget_stops_before_second_keep() {
    let mut d = Driver::bounded(256, budget(1), Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
    let mut h = header();
    h.limits.decisions = Some(1);
    d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
        .unwrap();
    ready(&mut d);
    send(&mut d, Seat::P0, C::Keep).unwrap();
    let before = d.privileged_snapshot();
    assert!(
        send(&mut d, Seat::P1, C::Keep).is_err(),
        "a second decision must not be accepted after budget one"
    );
    assert_eq!(d.privileged_snapshot(), before);
    assert_eq!(
        d.finish().unwrap().status(),
        Status::Truncated(mtg_core::trajectory::Limit::Decisions)
    );
    assert_eq!(d.accounting().started, 1);
    assert_eq!(d.accounting().truncated, 1);
    assert!(d.finish().is_err());
    assert_eq!(d.accounting().truncated, 1);
}
#[test]
fn invalid_zero_limit_is_rejected_before_start() {
    assert!(Driver::bounded(256, budget(0), Box::new(Manual(Rc::new(Cell::new(0))))).is_err());
}
#[test]
fn controlled_clock_expires_before_input_and_during_reset_without_work() {
    for partial in [false, true] {
        let clock = Manual(Rc::new(Cell::new(10)));
        let mut b = budget(100);
        b.limits.wall_time_ms = Some(5);
        let mut d = Driver::bounded(256, b, Box::new(clock.clone())).unwrap();
        let mut h = header();
        h.limits = Limits {
            decisions: Some(100),
            wall_time_ms: Some(5),
            turns: None,
        };
        d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
            .unwrap();
        if !partial {
            ready(&mut d)
        }
        let before = d.privileged_snapshot();
        clock.0.set(15);
        assert!(d.advance(NonZeroUsize::MAX).is_ok());
        assert_eq!(d.privileged_snapshot(), before);
        let r = d.finish().unwrap();
        assert_eq!(
            r.status(),
            Status::Truncated(mtg_core::trajectory::Limit::WallTime)
        );
        assert!(d.advance(NonZeroUsize::MIN).is_err());
        assert!(d.concede(Seat::P0, d.episode_id().unwrap()).is_err());
        if !partial {
            let e = r.trajectory().unwrap();
            assert_eq!(e.footer().unwrap().returns, [0, 0]);
            for seat in [Seat::P0, Seat::P1] {
                assert_eq!(e.seat(seat).unwrap().unassigned_reward, 0);
            }
        }
    }
}
#[test]
fn record_capacity_failure_retains_first_decision_and_blocks_all_input() {
    let mut b = budget(100);
    b.records = NonZeroUsize::MIN;
    let mut d = Driver::bounded(256, b, Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
    let mut h = header();
    h.limits.decisions = Some(100);
    d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
        .unwrap();
    ready(&mut d);
    send(&mut d, Seat::P0, C::Keep).unwrap();
    let before = d.privileged_snapshot();
    assert!(send(&mut d, Seat::P1, C::Keep).is_err());
    assert_eq!(before, d.privileged_snapshot());
    assert!(d.concede(Seat::P1, d.episode_id().unwrap()).is_err());
    assert!(d.advance(NonZeroUsize::MAX).is_err());
    let r = d.finish().unwrap();
    assert_eq!(
        r.status(),
        Status::Failed(mtg_core::episode::Failure::RecordCapacity)
    );
    assert_eq!(r.privileged_history().len(), 1);
    assert_eq!(r.trajectory().unwrap().decisions().len(), 1);
    assert!(r.trajectory().unwrap().seat(Seat::P0).is_err());
    assert_eq!(d.accounting().failed, 1);
    assert!(d.finish().is_err());
}
#[test]
fn every_mulligan_microchoice_counts_and_pending_bottom_survives() {
    let mut d = Driver::bounded(256, budget(4), Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
    let mut h = header();
    h.limits.decisions = Some(4);
    d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
        .unwrap();
    ready(&mut d);
    send(&mut d, Seat::P0, C::Mulligan).unwrap();
    send(&mut d, Seat::P1, C::Keep).unwrap();
    send(
        &mut d,
        Seat::P0,
        C::Bottom {
            card: policy::VisibleRef {
                zone: policy::VisibleZone::Hand,
                row: 0,
            },
        },
    )
    .unwrap();
    send(&mut d, Seat::P0, C::Mulligan).unwrap();
    assert_eq!(d.privileged_history().len(), 4);
    let r = d.finish().unwrap();
    let e = r.trajectory().unwrap();
    assert_eq!(
        r.status(),
        Status::Truncated(mtg_core::trajectory::Limit::Decisions)
    );
    assert_eq!(e.decisions().len(), 4);
    assert_eq!(e.footer().unwrap().returns, [0, 0]);
    assert!(e.footer().unwrap().final_observations[0].decision.is_some());
}
#[test]
fn completed_turn_budget_stops_at_next_upkeep() {
    let mut b = budget(100);
    b.limits.turns = Some(1);
    let mut h = header();
    h.limits = b.limits.clone();
    let mut d = Driver::bounded(256, b, Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
    d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
        .unwrap();
    ready(&mut d);
    send(&mut d, Seat::P0, C::Keep).unwrap();
    send(&mut d, Seat::P1, C::Keep).unwrap();
    ready(&mut d);
    // CR 103.8a skips the first draw step; no eligible creatures need a choice.
    // Empty board: upkeep, main, beginning combat, end combat,
    // postcombat main, end. Each priority window requires two explicit passes.
    for step in [
        "upkeep",
        "precombat_main",
        "beginning_combat",
        "declare_attackers",
        "end_combat",
        "postcombat_main",
        "end",
    ] {
        assert_eq!(d.observe(Seat::P0).unwrap().view.turn, Some((1, 0, step)));
        send(&mut d, Seat::P0, C::Pass).unwrap();
        send(&mut d, Seat::P1, C::Pass).unwrap();
    }
    let r = d.finish().unwrap();
    assert_eq!(
        r.status(),
        Status::Truncated(mtg_core::trajectory::Limit::Turns)
    );
    assert_eq!(r.privileged_history().len(), 16);
    assert_eq!(
        r.trajectory().unwrap().footer().unwrap().final_observations[0]
            .view
            .turn,
        Some((2, 1, "upkeep"))
    );
}
#[test]
fn invalid_limits_and_header_mismatch_do_not_start_or_replace_episodes() {
    for limits in [
        Limits {
            turns: Some(0),
            ..Limits::default()
        },
        Limits {
            wall_time_ms: Some(0),
            ..Limits::default()
        },
    ] {
        let mut b = budget(10);
        b.limits = limits;
        assert!(Driver::bounded(256, b, Box::new(Manual(Rc::new(Cell::new(0))))).is_err());
    }
    let mut d = Driver::bounded(256, budget(10), Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
    assert!(
        d.reset_captured(&config(), 161, 0, NonZeroUsize::MAX, &header())
            .is_err()
    );
    assert_eq!(d.accounting().started, 0);
    d.reset(&config(), 161, 0, NonZeroUsize::MAX).unwrap();
    let before = d.privileged_snapshot();
    assert!(d.reset(&config(), 161, 1, NonZeroUsize::MAX).is_err());
    assert_eq!(d.privileged_snapshot(), before);
    assert_eq!(d.accounting().started, 1);
    let a = d.finish().unwrap();
    assert_eq!(a.status(), Status::Incomplete);
    assert!(d.finish().is_err());
    d.reset(&config(), 161, 1, NonZeroUsize::MAX).unwrap();
    d.finish().unwrap();
    assert_eq!(d.accounting().started, 2);
    assert_eq!(d.accounting().incomplete, 2);
    assert_eq!(a.privileged_snapshot(), before);
}
#[test]
fn clock_regression_fails_and_can_be_finalized_once() {
    let c = Manual(Rc::new(Cell::new(10)));
    let mut d = Driver::bounded(256, budget(100), Box::new(c.clone())).unwrap();
    d.reset(&config(), 161, 0, NonZeroUsize::MIN).unwrap();
    let before = d.privileged_snapshot();
    c.0.set(9);
    assert!(d.advance(NonZeroUsize::MAX).is_err());
    assert_eq!(before, d.privileged_snapshot());
    assert_eq!(
        d.finish().unwrap().status(),
        Status::Failed(mtg_core::episode::Failure::Clock)
    );
    assert_eq!(d.accounting().failed, 1);
    assert!(d.finish().is_err());
}
#[test]
fn work_quantum_clamps_requested_reset_and_advance_work() {
    let mut b = budget(100);
    b.work_quantum = NonZeroUsize::MIN;
    let mut d = Driver::bounded(256, b, Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
    let mut core = mtg_core::game::Game::new().unwrap();
    d.reset(&config(), 161, 0, NonZeroUsize::MAX).unwrap();
    core.reset_quantum(&config(), 161, 0, NonZeroUsize::MIN)
        .unwrap();
    for _ in 0..30 {
        assert_eq!(
            normalized(&d.privileged_snapshot()),
            normalized(&core.snapshot())
        );
        assert_eq!(
            d.advance(NonZeroUsize::MAX).unwrap(),
            Progress::InternalYield
        );
        core.resume(NonZeroUsize::MIN);
    }
    assert_eq!(d.accounting().started, 1);
    assert_eq!(d.accounting().incomplete, 0);
}

fn normalized(bytes: &[u8]) -> Value {
    fn scrub(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "scope" || k == "store" {
                        *v = json!(0)
                    } else {
                        scrub(v)
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    scrub(v)
                }
            }
            _ => (),
        }
    }
    let e: Value = serde_json::from_slice(bytes).unwrap();
    let mut v: Value = serde_json::from_str(e["payload"].as_str().unwrap()).unwrap();
    v["objects"]["id"] = json!(0);
    scrub(&mut v);
    v
}
// 40 cards - seven opening = 33 draws. P1 draws on turns 2..66,
// then loses on the attempted draw at turn 68 (CR 104.3c/704.5b).
// No spells/lands played: turn 1 has 14 passes, turns 2..67 have
// 16 passes + one cleanup discard. Two keeps + 14 + 66*17 + 2 = 1140.
#[test]
fn actual_empty_library_terminal_at_decision_limit_matches_unbounded_core() {
    for decision_limit in [1140, 1200] {
        let mut b = budget(decision_limit);
        b.records = NonZeroUsize::new(1200).unwrap();
        let mut h = header();
        h.limits = b.limits.clone();
        let mut d = Driver::bounded(256, b, Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
        let mut core = mtg_core::game::Game::new().unwrap();
        core.reset(&config(), 161, 0).unwrap();
        d.reset_captured(&config(), 161, 0, NonZeroUsize::MAX, &h)
            .unwrap();
        ready(&mut d);
        fn both(d: &mut Driver, g: &mut mtg_core::game::Game, s: Seat, c: C) {
            let v = g.policy_observe(s, 256).unwrap().decision.unwrap();
            let sub = policy::Submission {
                schema_version: 1,
                revision: v.revision,
                generation: v.generation,
                choices: vec![c.clone()],
            };
            g.apply_policy(s, &sub, 256).unwrap();
            send(d, s, c).unwrap();
            assert_eq!(
                normalized(&d.privileged_snapshot()),
                normalized(&g.snapshot())
            );
        }
        both(&mut d, &mut core, Seat::P0, C::Keep);
        both(&mut d, &mut core, Seat::P1, C::Keep);
        core.start_turns().unwrap();
        ready(&mut d);
        for turn in 1..=68 {
            let active = if turn % 2 == 1 { Seat::P0 } else { Seat::P1 };
            let other = if active == Seat::P0 {
                Seat::P1
            } else {
                Seat::P0
            };
            let steps = if turn == 1 {
                vec![
                    "upkeep",
                    "precombat_main",
                    "beginning_combat",
                    "declare_attackers",
                    "end_combat",
                    "postcombat_main",
                    "end",
                ]
            } else {
                vec![
                    "upkeep",
                    "draw",
                    "precombat_main",
                    "beginning_combat",
                    "declare_attackers",
                    "end_combat",
                    "postcombat_main",
                    "end",
                ]
            };
            for step in steps {
                assert_eq!(
                    d.observe(active).unwrap().view.turn,
                    Some((turn, if active == Seat::P0 { 0 } else { 1 }, step))
                );
                both(&mut d, &mut core, active, C::Pass);
                both(&mut d, &mut core, other, C::Pass);
                if turn == 68 {
                    break;
                }
            }
            if turn == 68 {
                break;
            }
            if turn > 1 {
                both(
                    &mut d,
                    &mut core,
                    active,
                    C::Discard {
                        card: policy::VisibleRef {
                            zone: policy::VisibleZone::Hand,
                            row: 0,
                        },
                    },
                );
            }
        }
        let r = d.finish().unwrap();
        assert_eq!(r.privileged_history().len(), 1140);
        assert_eq!(
            r.status(),
            Status::Completed(mtg_core::game::terminal::Outcome {
                winner: Some(Seat::P0),
                losses: [None, Some(mtg_core::game::terminal::LossReason::EmptyDraw)]
            })
        );
        let e = r.trajectory().unwrap();
        assert_eq!(e.decisions().len(), 1140);
        assert_eq!(e.footer().unwrap().returns, [1, -1]);
        assert_eq!(
            e.footer().unwrap().end,
            mtg_core::trajectory::End::Completed
        );
        for (seat, reward) in [(Seat::P0, 1), (Seat::P1, -1)] {
            let seq = e.seat(seat).unwrap();
            assert_eq!(
                seq.transitions
                    .iter()
                    .map(|r| i64::from(r.reward))
                    .sum::<i64>()
                    + i64::from(seq.unassigned_reward),
                reward
            );
        }
        assert_eq!(d.accounting().completed, 1);
        assert_eq!(d.accounting().truncated, 0);
        assert!(d.finish().is_err());
        let saved = r.final_observations().cloned();
        d.reset(&config(), 161, 1, NonZeroUsize::MAX).unwrap();
        d.finish().unwrap();
        assert_eq!(r.final_observations().cloned(), saved);
        assert_eq!(d.accounting().started, 2);
        assert_eq!(d.accounting().completed, 1);
        assert_eq!(d.accounting().incomplete, 1);
    }
}
#[test]
fn pending_payment_is_not_committed_or_cancelled_by_a_budget() {
    let mut d = Driver::bounded(256, budget(40), Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
    let mut h = header();
    h.limits.decisions = Some(40);
    d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
        .unwrap();
    ready(&mut d);
    send(&mut d, Seat::P0, C::Keep).unwrap();
    send(&mut d, Seat::P1, C::Keep).unwrap();
    ready(&mut d);
    send(&mut d, Seat::P0, C::Pass).unwrap();
    send(&mut d, Seat::P1, C::Pass).unwrap();
    send(
        &mut d,
        Seat::P0,
        C::PlayLand {
            card: policy::VisibleRef {
                zone: policy::VisibleZone::Hand,
                row: 3,
            },
        },
    )
    .unwrap();
    // Finish turn 1 after the main-phase land. Turn 2 draws/discards once;
    // turn 3's upkeep/draw lead to the second legal land play (CR 305.2).
    for _ in 0..6 {
        send(&mut d, Seat::P0, C::Pass).unwrap();
        send(&mut d, Seat::P1, C::Pass).unwrap();
    }
    for _ in 0..8 {
        send(&mut d, Seat::P1, C::Pass).unwrap();
        send(&mut d, Seat::P0, C::Pass).unwrap();
    }
    send(
        &mut d,
        Seat::P1,
        C::Discard {
            card: policy::VisibleRef {
                zone: policy::VisibleZone::Hand,
                row: 0,
            },
        },
    )
    .unwrap();
    for _ in 0..2 {
        send(&mut d, Seat::P0, C::Pass).unwrap();
        send(&mut d, Seat::P1, C::Pass).unwrap();
    }
    send(
        &mut d,
        Seat::P0,
        C::PlayLand {
            card: policy::VisibleRef {
                zone: policy::VisibleZone::Hand,
                row: 3,
            },
        },
    )
    .unwrap();
    send(
        &mut d,
        Seat::P0,
        C::Cast {
            card: policy::VisibleRef {
                zone: policy::VisibleZone::Hand,
                row: 0,
            },
        },
    )
    .unwrap();
    let before = d.observe(Seat::P0).unwrap();
    assert!(before.pending.is_some());
    assert!(send(&mut d, Seat::P0, C::CancelPayment).is_err());
    let r = d.finish().unwrap();
    assert_eq!(
        r.status(),
        Status::Truncated(mtg_core::trajectory::Limit::Decisions)
    );
    assert_eq!(&r.final_observations().unwrap()[0], &before);
    let e = r.trajectory().unwrap();
    assert_eq!(
        e.decisions().last().unwrap().choice.status,
        mtg_core::trajectory::v2::ActionStatus::Continuing
    );
    assert_eq!(e.footer().unwrap().returns, [0, 0]);
    assert_eq!(e.footer().unwrap().decisions, 40);
}
#[test]
fn bounded_encoding_capacity_fails_without_losing_accepted_rows() {
    let mut d = Driver::bounded(2, budget(100), Box::new(Manual(Rc::new(Cell::new(0))))).unwrap();
    let mut h = header();
    h.limits.decisions = Some(100);
    d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
        .unwrap();
    ready(&mut d);
    send(&mut d, Seat::P0, C::Keep).unwrap();
    send(&mut d, Seat::P1, C::Keep).unwrap();
    let mut failed = false;
    for _ in 0..50 {
        if d.advance(NonZeroUsize::MAX).is_err() {
            failed = true;
            break;
        }
    }
    assert!(failed);
    let r = d.finish().unwrap();
    assert_eq!(
        r.status(),
        Status::Failed(mtg_core::episode::Failure::Capacity)
    );
    assert_eq!(r.privileged_history().len(), 2);
    assert_eq!(r.trajectory().unwrap().decisions().len(), 2);
    assert!(r.trajectory().unwrap().seat(Seat::P0).is_err());
}
#[derive(Clone, Debug)]
struct Ticking {
    now: Rc<Cell<u64>>,
    enabled: Rc<Cell<bool>>,
}
impl Clock for Ticking {
    fn now_ms(&self) -> u64 {
        let n = self.now.get();
        if self.enabled.get() {
            self.now.set(n + 1);
        }
        n
    }
}
#[test]
fn precedence_and_zero_nonacting_seat_rewards_with_controlled_boundary_clock() {
    for terminal in [false, true] {
        let clock = Ticking {
            now: Rc::new(Cell::new(0)),
            enabled: Rc::new(Cell::new(false)),
        };
        let mut b = budget(1);
        b.limits.wall_time_ms = Some(5);
        let mut h = header();
        h.limits = b.limits.clone();
        let mut d = Driver::bounded(256, b, Box::new(clock.clone())).unwrap();
        d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
            .unwrap();
        ready(&mut d);
        clock.now.set(4);
        clock.enabled.set(true);
        if terminal {
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
        } else {
            send(&mut d, Seat::P0, C::Keep).unwrap();
        }
        assert!(clock.now.get() >= 5);
        let r = d.finish().unwrap();
        let e = r.trajectory().unwrap();
        if terminal {
            assert!(matches!(r.status(), Status::Completed(_)));
            assert_eq!(e.footer().unwrap().returns, [1, -1]);
            assert_eq!(e.seat(Seat::P0).unwrap().unassigned_reward, 1);
            assert_eq!(e.seat(Seat::P1).unwrap().unassigned_reward, -1);
        } else {
            assert_eq!(
                r.status(),
                Status::Truncated(mtg_core::trajectory::Limit::Decisions)
            );
            assert_eq!(e.footer().unwrap().returns, [0, 0]);
            assert_eq!(e.seat(Seat::P1).unwrap().unassigned_reward, 0);
        }
        assert!(d.finish().is_err());
        assert_eq!(d.accounting().completed + d.accounting().truncated, 1);
    }
}
#[test]
fn clock_expiry_in_first_reset_quantum_never_fabricates_a_final_view() {
    let c = Ticking {
        now: Rc::new(Cell::new(0)),
        enabled: Rc::new(Cell::new(true)),
    };
    let mut b = budget(100);
    b.limits.wall_time_ms = Some(1);
    b.work_quantum = NonZeroUsize::MIN;
    let mut h = header();
    h.limits = b.limits.clone();
    let mut d = Driver::bounded(256, b, Box::new(c)).unwrap();
    assert_eq!(
        d.reset_captured(&config(), 161, 0, NonZeroUsize::MAX, &h)
            .unwrap(),
        Progress::Stopped(Status::Truncated(mtg_core::trajectory::Limit::WallTime))
    );
    let r = d.finish().unwrap();
    assert!(r.final_observations().is_none());
    assert!(r.trajectory().is_none());
    assert_eq!(r.privileged_history().len(), 0);
    assert_eq!(d.accounting().truncated, 1);
}
#[test]
fn expiry_during_opening_to_turn_work_keeps_an_unsealed_diagnostic_prefix() {
    let clock = Manual(Rc::new(Cell::new(0)));
    let mut b = budget(100);
    b.work_quantum = NonZeroUsize::MIN;
    b.limits.wall_time_ms = Some(5);
    let mut h = header();
    h.limits = b.limits.clone();
    let mut d = Driver::bounded(256, b, Box::new(clock.clone())).unwrap();
    d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
        .unwrap();
    ready(&mut d);
    send(&mut d, Seat::P0, C::Keep).unwrap();
    send(&mut d, Seat::P1, C::Keep).unwrap();
    assert_eq!(
        d.advance(NonZeroUsize::MIN).unwrap(),
        Progress::InternalYield
    );
    let before = d.privileged_snapshot();
    clock.0.set(5);
    assert_eq!(
        d.advance(NonZeroUsize::MAX).unwrap(),
        Progress::Stopped(Status::Truncated(mtg_core::trajectory::Limit::WallTime))
    );
    assert_eq!(d.privileged_snapshot(), before);
    let r = d.finish().unwrap();
    assert_eq!(
        r.status(),
        Status::Truncated(mtg_core::trajectory::Limit::WallTime)
    );
    assert!(r.final_observations().is_none());
    assert_eq!(r.trajectory().unwrap().decisions().len(), 2);
    assert!(r.trajectory().unwrap().seat(Seat::P0).is_err());
}
#[test]
fn reset_expiry_at_a_ready_boundary_seals_the_actual_zero_decision_frame() {
    let c = Ticking {
        now: Rc::new(Cell::new(0)),
        enabled: Rc::new(Cell::new(true)),
    };
    let mut b = budget(100);
    b.limits.wall_time_ms = Some(1);
    b.work_quantum = NonZeroUsize::MAX;
    let mut h = header();
    h.limits = b.limits.clone();
    let mut d = Driver::bounded(256, b, Box::new(c)).unwrap();
    d.reset_captured(&config(), 161, 0, NonZeroUsize::MAX, &h)
        .unwrap();
    let r = d.finish().unwrap();
    assert_eq!(
        r.status(),
        Status::Truncated(mtg_core::trajectory::Limit::WallTime)
    );
    assert!(r.final_observations().is_some());
    assert_eq!(r.trajectory().unwrap().footer().unwrap().decisions, 0);
    assert_eq!(d.accounting().truncated, 1);
}
#[test]
fn simultaneous_decision_turn_and_time_limits_have_fixed_precedence() {
    for decisions in [16, 100] {
        let clock = Ticking {
            now: Rc::new(Cell::new(0)),
            enabled: Rc::new(Cell::new(false)),
        };
        let mut b = budget(decisions);
        b.limits.turns = Some(1);
        b.limits.wall_time_ms = Some(5);
        let mut h = header();
        h.limits = b.limits.clone();
        let mut d = Driver::bounded(256, b, Box::new(clock.clone())).unwrap();
        d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
            .unwrap();
        ready(&mut d);
        send(&mut d, Seat::P0, C::Keep).unwrap();
        send(&mut d, Seat::P1, C::Keep).unwrap();
        ready(&mut d);
        for window in 0..7 {
            send(&mut d, Seat::P0, C::Pass).unwrap();
            if window == 6 {
                clock.now.set(4);
                clock.enabled.set(true);
            }
            send(&mut d, Seat::P1, C::Pass).unwrap();
        }
        let r = d.finish().unwrap();
        assert_eq!(
            r.status(),
            Status::Truncated(if decisions == 16 {
                mtg_core::trajectory::Limit::Decisions
            } else {
                mtg_core::trajectory::Limit::Turns
            })
        );
        assert_eq!(r.accepted_decisions(), 16);
        assert_eq!(r.budget().unwrap().limits, h.limits);
        assert_eq!(d.accounting().truncated, 1);
    }
}
#[test]
fn submission_during_internal_work_is_rejected_without_poisoning_either_capture_mode() {
    for (bounded, captured) in [(false, false), (false, true), (true, false), (true, true)] {
        let mut d = if bounded {
            Driver::bounded(256, budget(100), Box::new(Manual(Rc::new(Cell::new(0))))).unwrap()
        } else {
            Driver::new(256).unwrap()
        };
        let mut h = header();
        h.limits.decisions = Some(100);
        if captured {
            d.reset_captured(&config(), 161, 0, NonZeroUsize::MIN, &h)
                .unwrap();
        } else {
            d.reset(&config(), 161, 0, NonZeroUsize::MIN).unwrap();
        }
        let before = d.privileged_snapshot();
        for schema_version in [1, 99] {
            assert!(
                d.submit(
                    Seat::P0,
                    &policy::Submission {
                        schema_version,
                        revision: 0,
                        generation: 0,
                        choices: vec![C::Keep]
                    }
                )
                .is_err()
            );
            assert_eq!(d.privileged_snapshot(), before);
            assert_eq!(
                d.status(),
                None,
                "unavailable input must not poison an episode"
            );
            assert_eq!(d.accounting().failed, 0);
        }
        ready(&mut d);
        send(&mut d, Seat::P0, C::Keep).unwrap();
        send(&mut d, Seat::P1, C::Keep).unwrap();
        assert_eq!(
            d.advance(NonZeroUsize::MIN).unwrap(),
            Progress::InternalYield
        );
        let before = d.privileged_snapshot();
        assert!(
            d.submit(
                Seat::P0,
                &policy::Submission {
                    schema_version: 1,
                    revision: 1,
                    generation: 0,
                    choices: vec![C::Pass]
                }
            )
            .is_err()
        );
        assert_eq!(d.privileged_snapshot(), before);
        assert_eq!(d.status(), None);
        assert_eq!(d.accounting().failed, 0);
        ready(&mut d);
        send(&mut d, Seat::P0, C::Pass).unwrap();
        d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
        let r = d.finish().unwrap();
        assert!(matches!(r.status(), Status::Completed(_)));
        assert_eq!(r.accepted_decisions(), 3);
        assert_eq!(d.accounting().completed, 1);
        if captured {
            assert_eq!(r.trajectory().unwrap().decisions().len(), 3);
            assert_eq!(r.trajectory().unwrap().footer().unwrap().returns, [1, -1]);
        }
    }
}
