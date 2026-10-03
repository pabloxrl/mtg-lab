//! Synthetic positions; independent Oracle 1R 2/2 Cavalry, CR 602/302.6/702.10.
use super::mana::Color;
use super::turns::{TurnAction, TurnSelection};
use super::*;

fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 197, 0).unwrap();
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
    pass(&mut g);
    pass(&mut g);
    g
}
fn pass(g: &mut Game) {
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
fn add(g: &mut Game, key: &str, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, zone)
        .unwrap()
}
#[test]
fn cavalry_pinned_cast_and_fresh_activation_rejection() {
    let mut g = ready();
    let h = add(&mut g, "axgard-cavalry", Seat::P0, Zone::Hand(Seat::P0));
    g.turns.mana[0][3] = 2;
    assert!(
        g.cast_candidates(Seat::P0).contains(&h),
        "Pinned Cavalry costs 1R and must be castable"
    );
    let d = g.turn_decision().unwrap();
    g.begin_cast(Seat::P0, d.id, h).unwrap();
    for _ in 0..2 {
        let d = g.payment_decision(Seat::P0).unwrap();
        g.choose_payment(Seat::P0, d.id, Color::Red).unwrap();
    }
    let d = g.payment_decision(Seat::P0).unwrap();
    g.finish_cast(Seat::P0, d.id).unwrap();
    pass(&mut g);
    pass(&mut g);
    let h = g.objects.in_zone(Zone::Battlefield).next().unwrap();
    let c = g.creature_state(h).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2));
    assert!(g.summoning_sick(h));
    let o = g.policy_observe(Seat::P0, 256).unwrap();
    let d = o.decision.unwrap();
    assert!(
        !d.candidates
            .iter()
            .zip(d.legal_mask)
            .any(|(c, legal)| legal && serde_json::to_value(c).unwrap()["kind"] == "activate")
    );
}

#[test]
fn old_cavalry_offers_targeted_activation() {
    let mut g = ready();
    add(&mut g, "axgard-cavalry", Seat::P0, Zone::Battlefield);
    add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    assert!(
        d.candidates
            .iter()
            .zip(d.legal_mask)
            .any(|(c, legal)| legal && serde_json::to_value(c).unwrap()["kind"] == "activate"),
        "CR 602: old untapped Cavalry must offer its activation"
    );
}

fn command(g: &mut Game, value: serde_json::Value) {
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    let c = serde_json::from_value(value).expect("supported activation command");
    g.apply_policy(
        Seat::P0,
        &policy::Submission {
            schema_version: policy::SCHEMA_VERSION,
            revision: d.revision,
            generation: d.generation,
            choices: vec![c],
        },
        256,
    )
    .unwrap();
}
fn activation(g: &mut Game, row: usize) {
    command(
        g,
        serde_json::json!({"kind":"activate","card":{"zone":"battlefield","row":0}}),
    );
    command(
        g,
        serde_json::json!({"kind":"target","card":{"zone":"battlefield","row":row}}),
    );
    command(g, serde_json::json!({"kind":"finish_activation"}));
}
#[test]
fn haste_source_independent_target_identity_and_cleanup() {
    // CR 113.7a: source loss does not remove ability. CR 400.7/608.2b:
    // departed target is illegal even when the same card returns. CR 611.2a/514.2.
    for mode in ["live", "source_dies", "target_returns", "opposing"] {
        let mut g = ready();
        let source = add(&mut g, "axgard-cavalry", Seat::P0, Zone::Battlefield);
        let target_seat = if mode == "opposing" {
            Seat::P1
        } else {
            Seat::P0
        };
        let mut target = add(&mut g, "bear-cub", target_seat, Zone::Battlefield);
        g.turns.sick.push(target);
        activation(&mut g, 1);
        assert!(g.objects.get(source).unwrap().tapped);
        assert!(
            g.summoning_sick(target),
            "haste is not granted before resolution"
        );
        assert_eq!(g.turns.stack.len(), 1);
        if mode == "source_dies" {
            g.objects
                .move_to(source, Zone::Graveyard(Seat::P0))
                .unwrap();
        }
        if mode == "target_returns" {
            target = g
                .objects
                .move_to(target, Zone::Graveyard(target_seat))
                .unwrap();
            target = g.objects.move_to(target, Zone::Battlefield).unwrap();
            g.turns.sick.push(target);
        }
        pass(&mut g);
        pass(&mut g);
        assert!(g.turns.stack.is_empty());
        assert_eq!(g.summoning_sick(target), mode == "target_returns");
        if mode == "live" {
            g.turns.position = Some((1, Seat::P0, turns::Step::BeginningCombat));
            pass(&mut g);
            pass(&mut g);
            let d = g.turn_decision().unwrap();
            let d = g.select_attackers(Seat::P0, d.id, &[target]).unwrap();
            g.finish_combat(Seat::P0, d.id).unwrap();
            assert!(g.objects.get(target).unwrap().tapped);
        }
        g.turns.position = Some((1, Seat::P0, turns::Step::End));
        g.turns.passed = false;
        g.set_turn_decision(Seat::P0, turns::TurnKind::Priority);
        pass(&mut g);
        pass(&mut g);
        assert_eq!(g.turn_position().unwrap().0, 2);
        if target_seat == Seat::P0 {
            assert!(
                g.summoning_sick(target),
                "cleanup expires haste before owner's next turn"
            );
        }
    }
}
#[test]
fn haste_cancel_and_illegal_policy_do_not_pay_cost() {
    let mut g = ready();
    let source = add(&mut g, "axgard-cavalry", Seat::P0, Zone::Battlefield);
    add(&mut g, "forest", Seat::P0, Zone::Battlefield);
    command(
        &mut g,
        serde_json::json!({"kind":"activate","card":{"zone":"battlefield","row":0}}),
    );
    assert!(!g.objects.get(source).unwrap().tapped);
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    let before = g.snapshot();
    let c = serde_json::from_value(
        serde_json::json!({"kind":"target","card":{"zone":"battlefield","row":1}}),
    )
    .unwrap();
    assert!(
        g.apply_policy(
            Seat::P0,
            &policy::Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices: vec![c]
            },
            256
        )
        .is_err()
    );
    assert_eq!(before, g.snapshot());
    command(&mut g, serde_json::json!({"kind":"cancel_activation"}));
    assert!(!g.objects.get(source).unwrap().tapped);
    assert!(g.turns.stack.is_empty());
}

#[test]
fn haste_reference_literal_checkpoints() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/haste.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/haste-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for c in fixture["cases"].as_array().unwrap() {
        let mode = c["mode"].as_str().unwrap();
        let mut g = ready();
        let mut source = add(
            &mut g,
            "axgard-cavalry",
            Seat::P0,
            if mode == "cast" {
                Zone::Hand(Seat::P0)
            } else {
                Zone::Battlefield
            },
        );
        let target = add(
            &mut g,
            c["target"].as_str().unwrap(),
            if mode == "opposing" {
                Seat::P1
            } else {
                Seat::P0
            },
            Zone::Battlefield,
        );
        g.turns.sick.push(target);
        if mode == "sick" {
            g.turns.sick.push(source);
        }
        if mode == "tapped" {
            g.objects.get_mut(source).unwrap().tapped = true;
        }
        if mode == "dead_before" {
            g.objects
                .move_to(source, Zone::Graveyard(Seat::P0))
                .unwrap();
        }
        if mode == "cast" {
            g.turns.mana[0][3] = 2;
            let d = g.turn_decision().unwrap();
            g.begin_cast(Seat::P0, d.id, source).unwrap();
            for _ in 0..2 {
                let d = g.payment_decision(Seat::P0).unwrap();
                g.choose_payment(Seat::P0, d.id, Color::Red).unwrap();
            }
            let d = g.payment_decision(Seat::P0).unwrap();
            g.finish_cast(Seat::P0, d.id).unwrap();
            pass(&mut g);
            pass(&mut g);
            source = g
                .objects
                .in_zone(Zone::Battlefield)
                .find(|h| g.objects.get(*h).unwrap().card.identity().key == "axgard-cavalry")
                .unwrap();
            assert!(g.summoning_sick(source));
        } else if ["sick", "tapped", "dead_before"].contains(&mode) {
            let before = g.snapshot();
            let d = g.turn_decision().unwrap();
            assert!(g.begin_activation(Seat::P0, d.id, source).is_err());
            assert_eq!(g.snapshot(), before);
        } else {
            activation(&mut g, 1);
            assert!(!g.has_haste(target));
            assert_eq!(g.turns.stack.len(), 1);
            if mode == "source_dies" {
                g.objects
                    .move_to(source, Zone::Graveyard(Seat::P0))
                    .unwrap();
            }
            if mode == "target_dies" {
                g.objects
                    .move_to(target, Zone::Graveyard(Seat::P0))
                    .unwrap();
            }
            pass(&mut g);
            pass(&mut g);
            if mode == "cleanup" || mode == "opposing" {
                assert!(g.has_haste(target));
                g.turns.position = Some((1, Seat::P0, turns::Step::End));
                pass(&mut g);
                pass(&mut g);
            }
        }
        if ["cub", "swab", "source_dies"].contains(&mode) {
            g.turns.position = Some((1, Seat::P0, turns::Step::BeginningCombat));
            pass(&mut g);
            pass(&mut g);
            let d = g.turn_decision().unwrap();
            let d = g.select_attackers(Seat::P0, d.id, &[target]).unwrap();
            g.finish_combat(Seat::P0, d.id).unwrap();
            pass(&mut g);
            pass(&mut g);
            let d = g.turn_decision().unwrap();
            g.finish_combat(d.actor, d.id).unwrap();
            pass(&mut g);
            pass(&mut g);
            let d = g.turn_decision().unwrap();
            g.finish_combat(d.actor, d.id).unwrap();
            assert!(g.objects.get(target).unwrap().tapped);
        }
        let source_present = g
            .objects
            .get(source)
            .is_ok_and(|o| o.zone == Zone::Battlefield);
        let target_present = g
            .objects
            .get(target)
            .is_ok_and(|o| o.zone == Zone::Battlefield);
        let stats = if source_present {
            let s = g.creature_state(source).unwrap();
            Some([s.power, s.toughness])
        } else {
            None
        };
        let result = serde_json::json!({"source":source_present,"source_tapped":source_present&&g.objects.get(source).unwrap().tapped,"target":target_present,"haste":g.has_haste(target),"stack":g.turns.stack.len(),"life":g.life(),"source_stats":stats});
        assert_eq!(result, expected[mode], "{mode}");
        results.insert(mode.into(), result);
    }
    if let Ok(path) = std::env::var("MTG_HASTE_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}

#[test]
fn activation_raw_rejections_pending_restore_and_capacity_are_atomic() {
    let mut g = ready();
    let source = add(&mut g, "axgard-cavalry", Seat::P0, Zone::Battlefield);
    let target = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert!(g.begin_activation(Seat::P1, d.id, source).is_err());
    assert_eq!(g.snapshot(), before);
    assert!(g.policy_observe(Seat::P0, 1).is_err());
    assert_eq!(g.snapshot(), before);
    g.begin_activation(Seat::P0, d.id, source).unwrap();
    let id = g.turns.activation.as_ref().unwrap().id;
    let before = g.snapshot();
    assert!(g.finish_activation(Seat::P0, id).is_err());
    assert_eq!(g.snapshot(), before);
    assert!(g.choose_activation_target(Seat::P1, id, target).is_err());
    assert_eq!(g.snapshot(), before);
    g.choose_activation_target(Seat::P0, id, target).unwrap();
    let before = g.snapshot();
    assert!(g.finish_activation(Seat::P0, id).is_err());
    assert_eq!(g.snapshot(), before);
    // Opponent sees committed state only; no provisional target/cost payload.
    assert!(g.policy_observe(Seat::P1, 256).unwrap().pending.is_none());
    let mut restored = Game::new().unwrap();
    restored.restore(&before).unwrap();
    let id = restored.turns.activation.as_ref().unwrap().id;
    let h = restored.turns.activation.as_ref().unwrap().source;
    assert!(!restored.objects.get(h).unwrap().tapped);
    restored.finish_activation(Seat::P0, id).unwrap();
    let before = restored.snapshot();
    let d = restored.turn_decision().unwrap();
    assert!(restored.begin_activation(Seat::P0, d.id, h).is_err());
    assert_eq!(restored.snapshot(), before);
    let id = g.turns.activation.as_ref().unwrap().id;
    // Numeric identity exhaustion must not pay the tap cost or publish a stack object.
    g.objects.test_exhaust_births_after_one();
    add(&mut g, "forest", Seat::P0, Zone::Battlefield);
    let before = g.snapshot();
    assert!(g.finish_activation(Seat::P0, id).is_err());
    assert_eq!(g.snapshot(), before);
}

#[test]
fn haste_every_resolution_quantum_restores_and_malformed_cost_rejects() {
    let mut g = ready();
    let source = add(&mut g, "axgard-cavalry", Seat::P0, Zone::Battlefield);
    let target = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    g.turns.sick.push(target);
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    let submission = policy::Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation,
        choices: vec![policy::Choice::Activate {
            card: policy::VisibleRef {
                zone: policy::VisibleZone::Battlefield,
                row: 0,
            },
        }],
    };
    let bytes = actions::encode(&g, Seat::P0, &submission, 256).unwrap();
    let mut forged: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    forged["choices"][0]["cost"] = serde_json::json!({"tap":false});
    let before = g.snapshot();
    assert!(actions::apply(&mut g, &serde_json::to_vec(&forged).unwrap(), 256).is_err());
    assert_eq!(g.snapshot(), before);
    activation(&mut g, 1);
    assert!(g.creature_state(*g.turns.stack.last().unwrap()).is_none());
    pass(&mut g);
    let before = g.snapshot();
    for quantum in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
        let mut r = Game::new().unwrap();
        r.restore(&before).unwrap();
        let d = r.turn_decision().unwrap();
        let mut p = r
            .apply_turn_quantum(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Pass(d.candidate(0)),
                },
                quantum,
            )
            .unwrap();
        let mut yields = 0;
        while p == Progress::InternalYield {
            yields += 1;
            let snapshot = r.snapshot();
            r.restore(&snapshot).unwrap();
            p = r.resume(quantum);
        }
        if quantum == NonZeroUsize::MIN {
            assert!(yields >= 3);
        }
        let target = r
            .objects
            .in_zone(Zone::Battlefield)
            .find(|h| r.objects.get(*h).unwrap().card.identity().key == "bear-cub")
            .unwrap();
        assert!(r.has_haste(target));
        assert!(!r.summoning_sick(target));
        assert!(r.turns.stack.is_empty());
        assert_eq!(
            r.objects.in_zone(Zone::Graveyard(Seat::P0)).count(),
            0,
            "an ability is removed, not put into graveyard"
        );
    }
    assert!(g.objects.get(source).unwrap().tapped);
}

#[test]
fn haste_bite_response_kills_source_not_ability() {
    let mut g = ready();
    let source = add(&mut g, "axgard-cavalry", Seat::P0, Zone::Battlefield);
    let target = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    g.turns.sick.push(target);
    let enemy = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    let bite = add(&mut g, "bite-down", Seat::P1, Zone::Hand(Seat::P1));
    activation(&mut g, 1);
    assert!(g.creature_state(*g.turns.stack.last().unwrap()).is_none());
    pass(&mut g);
    g.turns.mana[1][4] = 2;
    let d = g.turn_decision().unwrap();
    let d = g.begin_targeted_cast(Seat::P1, d.id, bite, 256).unwrap();
    let d = g.choose_target(Seat::P1, d.id, enemy).unwrap();
    let d = g.choose_target(Seat::P1, d.id, source).unwrap();
    let d = g.finish_targets(Seat::P1, d.id).unwrap();
    let d = g.choose_payment(Seat::P1, d.id, Color::Green).unwrap();
    let d = g.choose_payment(Seat::P1, d.id, Color::Green).unwrap();
    g.finish_cast(Seat::P1, d.id).unwrap();
    assert_eq!(g.turns.stack.len(), 2);
    pass(&mut g);
    pass(&mut g);
    assert!(g.objects.get(source).is_err());
    assert_eq!(g.turns.stack.len(), 1);
    assert!(!g.has_haste(target));
    pass(&mut g);
    pass(&mut g);
    assert!(g.has_haste(target));
    assert!(!g.summoning_sick(target));
}

#[test]
fn concession_before_and_after_activation_target_is_final() {
    // CR 104.3a and the terminal contract: concession is immediate, including
    // inside a private continuation, and subsequent commands cannot change play.
    for selected in [false, true] {
        for loser in [Seat::P0, Seat::P1] {
            let mut g = ready();
            let source = add(&mut g, "axgard-cavalry", Seat::P0, Zone::Battlefield);
            let target = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
            let d = g.turn_decision().unwrap();
            g.begin_activation(Seat::P0, d.id, source).unwrap();
            if selected {
                let id = g.turns.activation.as_ref().unwrap().id;
                g.choose_activation_target(Seat::P0, id, target).unwrap();
            }
            let id = g.turns.activation.as_ref().unwrap().id;
            g.concede(loser, g.episode_id().unwrap()).unwrap();
            for seat in [Seat::P0, Seat::P1] {
                let o = g.policy_observe(seat, 256).unwrap();
                assert!(
                    o.decision.is_none(),
                    "terminal games cannot offer activation decisions"
                );
                assert!(o.pending.is_none());
                assert_eq!(o.view.acting_seat, None);
                assert!(o.view.terminal.is_some());
                assert!(g.observe(seat).unwrap().terminal.is_some());
            }
            assert!(!g.objects.get(source).unwrap().tapped);
            assert!(!g.has_haste(target));
            assert!(g.turns.stack.is_empty());
            let before = g.snapshot();
            assert!(g.finish_activation(Seat::P0, id).is_err());
            assert!(g.choose_activation_target(Seat::P0, id, target).is_err());
            assert!(g.cancel_activation(Seat::P0, id).is_err());
            assert!(g.begin_activation(Seat::P0, id, source).is_err());
            assert_eq!(g.snapshot(), before);
        }
    }
}
