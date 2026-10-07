//! Declared synthetic trigger placement, not real card detection/cast evidence.
//! Independent expectations: CR 603.3b, 101.4 and 117.5.
use super::*;

fn pending() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 204, 0).unwrap();
    while let Some(d) = g.decision() {
        g.apply(
            d.actor,
            &OpeningAction {
                decision: d.id,
                selection: Selection::Choose(d.candidate(0)),
            },
        )
        .unwrap();
    }
    g.start_turns().unwrap();
    let card = CardId::from_key("bear-cub").unwrap();
    let mut triggers = vec![];
    for (tag, controller) in [
        (10, Seat::P0),
        (11, Seat::P0),
        (20, Seat::P1),
        (21, Seat::P1),
    ] {
        let source = g
            .objects
            .allocate(card, controller, Zone::Battlefield)
            .unwrap();
        triggers.push(triggers::PendingTrigger {
            source,
            card,
            controller,
            kind: triggers::TriggerKind::Synthetic { tag },
        });
    }
    g.inject_triggers(triggers);
    g
}

#[test]
fn mandatory_trigger_order_precedes_priority() {
    let mut g = pending();
    g.finish_work();
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    assert_eq!(
        (d.kind, d.count),
        ("trigger_order", 2),
        "CR 117.5: mandatory placement precedes priority"
    );
}

#[test]
fn incomplete_trigger_permutation_rejects_without_mutation() {
    // Unchanged catalog rules-triggers-ordering-negative: two pending P0 triggers.
    let mut g = pending();
    g.finish_work();
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    let before = g.snapshot();
    let result = g.apply_policy(
        Seat::P0,
        &policy::Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![d.candidates[0].clone()],
        },
        256,
    );
    assert!(
        result.is_err(),
        "CR 603.3b: omission must reject, never pass priority"
    );
    assert_eq!(g.snapshot(), before);
}

fn tags(g: &Game) -> Vec<u32> {
    g.turns
        .stack
        .iter()
        .map(|h| {
            let a = g.turns.triggered.iter().find(|a| a.object == *h).unwrap();
            let triggers::TriggerKind::Synthetic { tag } = a.declaration.kind;
            tag
        })
        .collect()
}
fn order(g: &mut Game, rows: &[usize], q: NonZeroUsize) {
    let d = g.turn_decision().unwrap();
    let mut p = g.order_triggers_quantum(d.actor, d.id, rows, q).unwrap();
    while p == Progress::InternalYield {
        assert!(g.turn_decision().is_none());
        let s = g.snapshot();
        g.restore(&s).unwrap();
        p = g.resume(q);
    }
}
#[test]
fn trigger_apnap_dead_source_and_every_placement_snapshot() {
    for active in [Seat::P0, Seat::P1] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let mut g = pending();
            g.turns.position.as_mut().unwrap().1 = active;
            let source = g.turns.pending_triggers[1].unwrap().source;
            g.objects
                .move_to(source, Zone::Graveyard(Seat::P0))
                .unwrap();
            g.finish_work();
            assert_eq!(g.turn_decision().unwrap().actor, active);
            let s = g.snapshot();
            g.restore(&s).unwrap();
            let (first, second, expected): (&[usize], &[usize], &[u32]) = if active == Seat::P0 {
                (&[1, 0], &[2, 3], &[11, 10, 20, 21])
            } else {
                (&[3, 2], &[0, 1], &[21, 20, 10, 11])
            };
            order(&mut g, first, q);
            assert_eq!(
                g.turn_decision().unwrap().kind,
                turns::TurnKind::TriggerOrder
            );
            assert_eq!(g.turn_decision().unwrap().actor, turns::opponent(active));
            order(&mut g, second, q);
            assert_eq!(
                tags(&g),
                expected,
                "CR 603.3b: literal bottom-to-top APNAP order"
            );
            assert_eq!(g.turn_decision().unwrap().kind, turns::TurnKind::Priority);
            assert!(
                g.policy_observe(active, 256)
                    .unwrap()
                    .stack
                    .iter()
                    .all(|a| a.ability)
            );
        }
    }
}
#[test]
fn trigger_invalid_orders_and_capacity_preserve_snapshot() {
    let mut g = pending();
    g.finish_work();
    let d = g.turn_decision().unwrap();
    for (actor, id, rows) in [
        (Seat::P0, d.id, vec![0]),
        (Seat::P0, d.id, vec![0, 0]),
        (Seat::P0, d.id, vec![0, 2]),
        (Seat::P1, d.id, vec![2, 3]),
        (
            Seat::P0,
            DecisionId {
                generation: d.id.generation - 1,
                ..d.id
            },
            vec![0, 1],
        ),
    ] {
        let before = g.snapshot();
        assert!(
            g.order_triggers_quantum(actor, id, &rows, NonZeroUsize::MIN)
                .is_err()
        );
        assert_eq!(g.snapshot(), before);
    }
    let before = g.snapshot();
    assert!(
        g.policy_observe(Seat::P0, 1).is_err(),
        "never truncate the controller's two choices"
    );
    assert_eq!(g.snapshot(), before);
}
#[test]
fn synthetic_trigger_resolution_removes_ability_not_creature() {
    let mut g = pending();
    g.finish_work();
    order(&mut g, &[1, 0], NonZeroUsize::MAX);
    order(&mut g, &[2, 3], NonZeroUsize::MAX);
    let battlefield = g.objects.in_zone(Zone::Battlefield).count();
    let object = *g.turns.stack.last().unwrap();
    for _ in 0..2 {
        let d = g.turn_decision().unwrap();
        g.apply_turn(
            d.actor,
            &turns::TurnAction {
                decision: d.id,
                selection: turns::TurnSelection::Pass(d.candidate(0)),
            },
        )
        .unwrap();
    }
    assert!(
        g.objects.get(object).is_err(),
        "CR 608: an ability ceases to exist after resolution"
    );
    assert_eq!(g.objects.in_zone(Zone::Battlefield).count(), battlefield);
    assert_eq!(tags(&g), [11, 10, 20]);
    assert_eq!(g.turns.triggered.len(), 3);
}

#[test]
fn malformed_duplicate_placement_work_is_rejected_atomically() {
    use sha2::{Digest, Sha256};
    let mut g = pending();
    g.finish_work();
    let d = g.turn_decision().unwrap();
    assert_eq!(
        g.order_triggers_quantum(d.actor, d.id, &[0, 1], NonZeroUsize::MIN)
            .unwrap(),
        Progress::InternalYield
    );
    let mut envelope: serde_json::Value = serde_json::from_slice(&g.snapshot()).unwrap();
    let mut payload: serde_json::Value =
        serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
    let work = payload["work"].as_array_mut().unwrap();
    work.insert(0, work[0].clone());
    let payload = serde_json::to_string(&payload).unwrap();
    envelope["sha256"] = format!("{:x}", Sha256::digest(payload.as_bytes())).into();
    envelope["payload"] = payload.into();
    let before = g.snapshot();
    assert_eq!(
        g.restore(&serde_json::to_vec(&envelope).unwrap()),
        Err(snapshot::RestoreError::Corrupt)
    );
    assert_eq!(g.snapshot(), before);
}

#[test]
fn trigger_driver_capture_and_semantic_replay_from_declared_snapshot() {
    use crate::{
        episode::Driver,
        trajectory::{EpisodeKey, Header, Limits, Versions},
    };
    let mut g = pending();
    g.finish_work();
    let initial = g.snapshot();
    let h = Header {
        id: EpisodeKey {
            run: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
            ordinal: 204,
        },
        versions: Versions {
            schema: 2,
            engine: "synthetic-component".into(),
            rules: "cr-20260925".into(),
            cards: "pool-v1".into(),
            action: "policy-v1".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["test".into(), "test".into()],
        starting_seat: 0,
        limits: Limits::default(),
        restricted_replay: None,
    };
    let mut driver = Driver::synthetic_capture_test(g, h);
    for (seat, order) in [(Seat::P0, [1, 0]), (Seat::P1, [2, 3])] {
        let o = driver.observe(seat).unwrap();
        let d = o.decision.as_ref().unwrap();
        let s = policy::Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: order
                .map(|trigger| policy::Choice::OrderTrigger { trigger })
                .to_vec(),
        };
        let before = driver.privileged_snapshot();
        let history = driver.privileged_history().to_vec();
        assert!(driver.submit(turns::opponent(seat), &s).is_err());
        assert_eq!(driver.privileged_snapshot(), before);
        assert_eq!(driver.privileged_history(), history);
        driver.submit(seat, &s).unwrap();
        let row = driver.trajectory().unwrap().decisions().last().unwrap();
        assert_eq!(row.observation, o);
        assert_eq!(row.choice.submission, s);
        assert!(
            driver.submit(seat, &s).is_err(),
            "stale command cannot place twice"
        );
    }
    let mut replay = Game::new().unwrap();
    replay.restore(&initial).unwrap();
    for bytes in driver.privileged_history() {
        actions::apply(&mut replay, bytes, 256).unwrap();
    }
    assert_eq!(tags(&replay), [11, 10, 20, 21]);
    driver
        .concede(Seat::P0, driver.episode_id().unwrap())
        .unwrap();
    let result = driver.finish().unwrap();
    let e = result.trajectory().unwrap();
    assert_eq!(e.decisions().len(), 2);
    assert_eq!(e.seat(Seat::P0).unwrap().transitions[0].reward, -1);
    assert_eq!(e.seat(Seat::P1).unwrap().transitions[0].reward, 1);
}

#[test]
fn trigger_batch_storage_exhaustion_rejects_before_first_placement() {
    let mut g = pending();
    g.finish_work();
    g.objects.test_exhaust_births_after_one();
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.order_triggers_quantum(d.actor, d.id, &[0, 1], NonZeroUsize::MIN),
        Err(turns::TurnError::Storage(StorageError::IdentityExhausted))
    );
    assert_eq!(g.snapshot(), before);
    assert!(g.turns.stack.is_empty());
}

#[test]
fn mandatory_source_death_and_terminal_settlement_precede_placement() {
    let mut g = pending();
    let source = g.turns.pending_triggers[0].unwrap().source;
    g.objects
        .prepare_moves(&[source], Zone::Graveyard(Seat::P0))
        .unwrap();
    // Explicit synthetic SBA work boundary, using the production owned executor.
    g.work.push_front(Work::SpellMove {
        handle: source,
        zone: Zone::Graveyard(Seat::P0),
        controller: None,
    });
    assert_eq!(g.resume(NonZeroUsize::MIN), Progress::InternalYield);
    assert!(g.turn_decision().is_none());
    assert!(g.objects.get(source).is_err());
    g.finish_work();
    assert_eq!(
        g.turn_decision().unwrap().kind,
        turns::TurnKind::TriggerOrder
    );
    order(&mut g, &[0, 1], NonZeroUsize::MIN);
    order(&mut g, &[3, 2], NonZeroUsize::MIN);
    assert_eq!(tags(&g), [10, 11, 21, 20]);
    let mut lost = pending();
    lost.life[0] = 0;
    assert!(matches!(
        lost.resume(NonZeroUsize::MAX),
        Progress::Terminal(_)
    ));
    assert!(lost.turn_decision().is_none());
    assert!(lost.turns.stack.is_empty());
}
