//! Synthetic positions exercise real casts. Pinned Pyromancer; CR 603.2,
//! 603.3d, 115, 113.7a and 704 independently require these literal outcomes.
use super::*;
fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 206, 0).unwrap();
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
    pair(&mut g);
    g
}
fn pass(g: &mut Game) {
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
fn pair(g: &mut Game) {
    pass(g);
    pass(g);
}
fn cast(g: &mut Game) {
    let h = g
        .objects
        .allocate(
            CardId::from_key("viashino-pyromancer").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    g.turns.mana[0][3] = 2;
    let d = g.turn_decision().unwrap();
    let result = g.begin_cast(Seat::P0, d.id, h);
    assert!(
        result.is_ok(),
        "Pinned 1R Pyromancer must be castable: {result:?}"
    );
    let mut p = result.unwrap();
    for _ in 0..2 {
        p = g.choose_payment(Seat::P0, p.id, mana::Color::Red).unwrap();
    }
    g.finish_cast(Seat::P0, p.id).unwrap();
}
fn target(g: &mut Game, seat: u8) {
    let o = g.policy_observe(Seat::P0, 256).unwrap();
    let d = o.decision.unwrap();
    assert_eq!(d.kind, "trigger_target");
    let c =
        serde_json::from_value(serde_json::json!({"kind":"target_player","seat":seat})).unwrap();
    g.apply_policy(
        Seat::P0,
        &policy::Submission {
            revision: d.revision,
            schema_version: 1,
            generation: d.generation,
            choices: vec![c],
        },
        256,
    )
    .unwrap();
}
#[test]
fn pyromancer_etb_requires_player_target_then_deals_two() {
    for seat in [0, 1] {
        let mut g = ready();
        cast(&mut g);
        assert_eq!(g.life(), [20, 20]);
        assert!(
            g.turns.pending_triggers.is_empty(),
            "Casting is not entering"
        );
        pair(&mut g);
        let h = g
            .objects
            .in_zone(Zone::Battlefield)
            .find(|h| g.objects.get(*h).unwrap().card.identity().key == "viashino-pyromancer")
            .unwrap();
        assert_eq!(
            g.creature_state(h).map(|c| (c.power, c.toughness)),
            Some((2, 1))
        );
        assert_eq!(g.trigger_candidates(Seat::P0), vec![0]);
        let d = g.turn_decision().unwrap();
        g.order_triggers_quantum(d.actor, d.id, &[0], NonZeroUsize::MAX)
            .unwrap();
        target(&mut g, seat);
        assert_eq!(g.life(), [20, 20]);
        assert_eq!(g.turns.stack.len(), 1);
        g.objects.move_to(h, Zone::Graveyard(Seat::P0)).unwrap();
        pair(&mut g);
        assert_eq!(
            g.life(),
            if seat == 0 { [18, 20] } else { [20, 18] },
            "CR 113.7a: independent of departed source"
        );
    }
}
#[test]
fn pyromancer_missing_creature_wrong_seat_stale_target_reject_without_mutation() {
    let mut g = ready();
    cast(&mut g);
    pair(&mut g);
    let d = g.turn_decision().unwrap();
    g.order_triggers_quantum(d.actor, d.id, &[0], NonZeroUsize::MAX)
        .unwrap();
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    assert_eq!(d.kind, "trigger_target");
    let before = g.snapshot();
    for (actor, generation, choices) in [
        (Seat::P0, d.generation, vec![]),
        (Seat::P0, d.generation, vec![policy::Choice::Pass]),
        (
            Seat::P0,
            d.generation,
            vec![policy::Choice::Target {
                card: policy::VisibleRef {
                    zone: policy::VisibleZone::Battlefield,
                    row: 0,
                },
            }],
        ),
        (Seat::P1, d.generation, d.candidates[..1].to_vec()),
        (Seat::P0, d.generation - 1, d.candidates[..1].to_vec()),
    ] {
        assert!(
            g.apply_policy(
                actor,
                &policy::Submission {
                    revision: d.revision,
                    schema_version: 1,
                    generation,
                    choices
                },
                256
            )
            .is_err()
        );
        assert_eq!(g.snapshot(), before);
    }
    g.life[0] = 2;
    target(&mut g, 0);
    pair(&mut g);
    assert_eq!(g.life(), [0, 20]);
    assert!(g.outcome.is_some());
}
#[test]
fn pyromancer_corrupt_target_continuation_rejects_restore() {
    use sha2::{Digest, Sha256};
    let mut g = ready();
    cast(&mut g);
    pair(&mut g);
    let d = g.turn_decision().unwrap();
    g.order_triggers_quantum(d.actor, d.id, &[0], NonZeroUsize::MAX)
        .unwrap();
    let before = g.snapshot();
    for mode in 0..4 {
        let mut e: serde_json::Value = serde_json::from_slice(&before).unwrap();
        let mut p: serde_json::Value =
            serde_json::from_str(e["payload"].as_str().unwrap()).unwrap();
        match mode {
            0 => p["turns"]["trigger_placement"] = serde_json::json!([]),
            1 => p["turns"]["trigger_placement"] = serde_json::json!([0, 0]),
            2 => p["turns"]["decision"]["actor"] = serde_json::json!("P1"),
            _ => p["turns"]["trigger_return"] = serde_json::Value::Null,
        }
        let payload = serde_json::to_string(&p).unwrap();
        e["sha256"] = format!("{:x}", Sha256::digest(payload.as_bytes())).into();
        e["payload"] = payload.into();
        assert_eq!(
            g.restore(&serde_json::to_vec(&e).unwrap()),
            Err(snapshot::RestoreError::Corrupt),
            "mode {mode}"
        );
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn pyromancer_quantum_snapshot_target_and_resolution() {
    for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
        let mut g = ready();
        cast(&mut g);
        for _ in 0..2 {
            let d = g.turn_decision().unwrap();
            let mut p = g
                .apply_turn_quantum(
                    d.actor,
                    &turns::TurnAction {
                        decision: d.id,
                        selection: turns::TurnSelection::Pass(d.candidate(0)),
                    },
                    q,
                )
                .unwrap();
            while p == Progress::InternalYield {
                let s = g.snapshot();
                g.restore(&s).unwrap();
                p = g.resume(q);
            }
        }
        let d = g.turn_decision().unwrap();
        let mut p = g.order_triggers_quantum(d.actor, d.id, &[0], q).unwrap();
        while p == Progress::InternalYield {
            let s = g.snapshot();
            g.restore(&s).unwrap();
            p = g.resume(q);
        }
        let s = g.snapshot();
        g.restore(&s).unwrap();
        let d = g.turn_decision().unwrap();
        let before = g.snapshot();
        assert!(
            g.target_trigger_quantum(Seat::P1, d.id, Seat::P0, q)
                .is_err()
        );
        assert_eq!(g.snapshot(), before);
        let mut p = g
            .target_trigger_quantum(d.actor, d.id, Seat::P1, q)
            .unwrap();
        while p == Progress::InternalYield {
            let s = g.snapshot();
            g.restore(&s).unwrap();
            p = g.resume(q);
        }
        let o = g.policy_observe(Seat::P0, 256).unwrap();
        assert_eq!(o.stack[0].trigger.as_ref().unwrap().target_player, Some(1));
        assert_eq!(g.life(), [20, 20]);
        for _ in 0..2 {
            let d = g.turn_decision().unwrap();
            let mut p = g
                .apply_turn_quantum(
                    d.actor,
                    &turns::TurnAction {
                        decision: d.id,
                        selection: turns::TurnSelection::Pass(d.candidate(0)),
                    },
                    q,
                )
                .unwrap();
            while p == Progress::InternalYield {
                let s = g.snapshot();
                g.restore(&s).unwrap();
                p = g.resume(q);
            }
        }
        assert_eq!(g.life(), [20, 18]);
    }
}
#[test]
fn pyromancer_synthetic_multi_target_apnap_placement_and_end_step() {
    for starting in [Seat::P0, Seat::P1] {
        let mut g = ready();
        g.turns.position = Some((1, starting, turns::Step::End));
        let mut batch = vec![];
        for controller in [starting, starting, turns::opponent(starting)] {
            let card = CardId::from_key("viashino-pyromancer").unwrap();
            let source = g
                .objects
                .allocate(card, controller, Zone::Battlefield)
                .unwrap();
            batch.push(triggers::PendingTrigger {
                source,
                card,
                controller,
                kind: triggers::TriggerKind::Pyromancer { target: None },
            });
        }
        g.inject_triggers(batch);
        g.finish_work();
        for rows in [vec![1, 0], vec![2]] {
            let d = g.turn_decision().unwrap();
            g.order_triggers_quantum(d.actor, d.id, &rows, NonZeroUsize::MAX)
                .unwrap();
            for row in rows {
                let s = g.snapshot();
                g.restore(&s).unwrap();
                let d = g.turn_decision().unwrap();
                let o = g.policy_observe(d.actor, 256).unwrap();
                assert_eq!(
                    o.pending_triggers
                        .iter()
                        .find(|p| p.selecting_target)
                        .unwrap()
                        .row,
                    row
                );
                g.target_trigger_quantum(
                    d.actor,
                    d.id,
                    turns::opponent(d.actor),
                    NonZeroUsize::MAX,
                )
                .unwrap();
                assert_eq!(g.life(), [20, 20]);
            }
        }
        assert_eq!(
            g.turns
                .triggered
                .iter()
                .map(|a| a.declaration.controller)
                .collect::<Vec<_>>(),
            [starting, starting, turns::opponent(starting)]
        );
        for _ in 0..3 {
            pair(&mut g);
            assert_eq!(g.turn_position().unwrap().2, turns::Step::End);
        }
        assert_eq!(
            g.life(),
            if starting == Seat::P0 {
                [18, 16]
            } else {
                [16, 18]
            }
        );
        pair(&mut g);
        // Empty-hand cleanup completes before the next upkeep.
        assert_eq!(g.turn_position().unwrap().2, turns::Step::Upkeep);
    }
}
#[test]
fn pyromancer_reference_literal_checkpoints() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/etb-triggers.json"
    ))
    .unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/etb-triggers-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for spec in fixture["cases"].as_array().unwrap() {
        let id = spec["id"].as_str().unwrap();
        let mut g = ready();
        g.life[0] = spec["life"].as_i64().unwrap();
        let card = CardId::from_key("viashino-pyromancer").unwrap();
        let cub = g
            .objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                Seat::P1,
                Zone::Battlefield,
            )
            .unwrap();
        let point = |g: &Game| {
            let stats = g
                .objects
                .in_zone(Zone::Battlefield)
                .find(|h| g.objects.get(*h).unwrap().card == card)
                .and_then(|h| g.creature_state(h).map(|c| [c.power, c.toughness]));
            let targets: Vec<_> = g
                .turns
                .triggered
                .iter()
                .filter_map(|a| {
                    if let triggers::TriggerKind::Pyromancer { target } = a.declaration.kind {
                        target.map(seat_index)
                    } else {
                        None
                    }
                })
                .collect();
            serde_json::json!([
                g.life(),
                stats,
                g.turns.stack.len(),
                targets,
                g.outcome
                    .map_or([false, false], |o| o.losses.map(|l| l.is_some()))
            ])
        };
        let mut points = vec![];
        if spec["end_step"] == true {
            let source = g
                .objects
                .allocate(card, Seat::P0, Zone::Battlefield)
                .unwrap();
            g.turns.position = Some((1, Seat::P0, turns::Step::End));
            points.push(point(&g));
            g.inject_triggers(vec![triggers::PendingTrigger {
                source,
                card,
                controller: Seat::P0,
                kind: triggers::TriggerKind::Pyromancer { target: None },
            }]);
            g.finish_work();
        } else {
            cast(&mut g);
            assert!(g.turns.pending_triggers.is_empty());
            points.push(point(&g));
            pair(&mut g);
        }
        let d = g.turn_decision().unwrap();
        g.order_triggers_quantum(d.actor, d.id, &[0], NonZeroUsize::MAX)
            .unwrap();
        let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
        let cub_row = g
            .objects
            .in_zone(Zone::Battlefield)
            .position(|h| h == cub)
            .unwrap();
        for choices in [
            vec![],
            vec![policy::Choice::Target {
                card: policy::VisibleRef {
                    zone: policy::VisibleZone::Battlefield,
                    row: cub_row,
                },
            }],
        ] {
            let before = g.snapshot();
            assert!(
                g.apply_policy(
                    Seat::P0,
                    &policy::Submission {
                        revision: d.revision,
                        schema_version: 1,
                        generation: d.generation,
                        choices
                    },
                    256
                )
                .is_err()
            );
            assert_eq!(g.snapshot(), before);
        }
        target(&mut g, spec["target"].as_u64().unwrap() as u8);
        points.push(point(&g));
        let source = g
            .objects
            .in_zone(Zone::Battlefield)
            .find(|h| g.objects.get(*h).unwrap().card == card)
            .unwrap();
        match spec["death"].as_str().unwrap() {
            "bite" => {
                pass(&mut g);
                let bite = g
                    .objects
                    .allocate(
                        CardId::from_key("bite-down").unwrap(),
                        Seat::P1,
                        Zone::Hand(Seat::P1),
                    )
                    .unwrap();
                g.turns.mana[1][4] = 2;
                let d = g.turn_decision().unwrap();
                let t = g.begin_targeted_cast(Seat::P1, d.id, bite, 256).unwrap();
                let t = g.choose_target(Seat::P1, t.id, cub).unwrap();
                let t = g.choose_target(Seat::P1, t.id, source).unwrap();
                let mut p = g.finish_targets(Seat::P1, t.id).unwrap();
                for _ in 0..2 {
                    p = g
                        .choose_payment(Seat::P1, p.id, mana::Color::Green)
                        .unwrap();
                }
                g.finish_cast(Seat::P1, p.id).unwrap();
                pair(&mut g);
                points.push(point(&g));
            }
            "hook" => {
                g.objects
                    .move_to(source, Zone::Graveyard(Seat::P0))
                    .unwrap();
                points.push(point(&g));
            }
            _ => {}
        }
        pair(&mut g);
        points.push(point(&g));
        if spec["end_step"] == true {
            assert_eq!(g.turn_position().unwrap().2, turns::Step::End);
            pair(&mut g);
            assert_eq!(g.turn_position().unwrap().2, turns::Step::Upkeep);
        }
        assert_eq!(serde_json::json!(points), expected[id], "{id}");
        results.insert(id.into(), serde_json::json!(points));
    }
    if let Ok(path) = std::env::var("MTG_ETB_TRIGGERS_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}
#[test]
fn pyromancer_concession_during_target_clears_choice_and_restores_terminal() {
    let mut g = ready();
    cast(&mut g);
    pair(&mut g);
    let d = g.turn_decision().unwrap();
    g.order_triggers_quantum(d.actor, d.id, &[0], NonZeroUsize::MAX)
        .unwrap();
    g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
    let s = g.snapshot();
    assert_eq!(
        g.restore(&s),
        Ok(()),
        "Conceding during a required target must leave a restorable terminal snapshot"
    );
    assert!(
        g.policy_observe(Seat::P0, 256)
            .unwrap()
            .pending_triggers
            .iter()
            .all(|p| !p.selecting_target)
    );
    assert_eq!(g.outcome().unwrap().winner, Some(Seat::P0));
}
