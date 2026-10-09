//! Synthetic Invoker positions. Pinned 2GG 4/3; {8}: target creature
//! gets +5/+5 and trample until end of turn. CR 602, 608.2b, 611.2, 702.19.
use super::turns::{TurnAction, TurnSelection};
use super::*;
fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 200, 0).unwrap();
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
fn add(g: &mut Game, key: &str) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), Seat::P0, Zone::Battlefield)
        .unwrap()
}

fn activate(g: &mut Game, source: Handle, target: Handle) {
    let d = g.turn_decision().unwrap();
    g.begin_activation(d.actor, d.id, source).unwrap();
    let id = g.turns.activation.as_ref().unwrap().id;
    g.choose_activation_target(d.actor, id, target).unwrap();
    for _ in 0..8 {
        let id = g.turns.activation.as_ref().unwrap().id;
        g.pay_activation(d.actor, id, mana::Color::Green).unwrap();
    }
    let id = g.turns.activation.as_ref().unwrap().id;
    g.finish_activation(d.actor, id).unwrap();
}
#[test]
fn invoker_sick_tapped_source_own_opposing_boost_and_cleanup() {
    for owner in [Seat::P0, Seat::P1] {
        let mut g = ready();
        let source = add(&mut g, "wildheart-invoker");
        let target = g
            .objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                owner,
                Zone::Battlefield,
            )
            .unwrap();
        g.turns.sick.push(source);
        g.objects.get_mut(source).unwrap().tapped = true;
        g.turns.mana[0][4] = 8;
        assert!(
            g.activation_candidates(Seat::P0).contains(&source),
            "eight generic, no tap or sickness restriction"
        );
        activate(&mut g, source, target);
        assert_eq!(g.turns.mana[0][4], 0);
        assert_eq!(g.creature_state(target).unwrap().power, 2);
        g.objects
            .move_to(source, Zone::Graveyard(Seat::P0))
            .unwrap();
        pass(&mut g);
        pass(&mut g);
        let c = g.creature_state(target).unwrap();
        assert_eq!((c.power, c.toughness), (7, 7));
        assert!(g.has_trample(target));
        g.turns.position = Some((1, Seat::P0, turns::Step::End));
        pass(&mut g);
        pass(&mut g);
        let c = g.creature_state(target).unwrap();
        assert_eq!((c.power, c.toughness), (2, 2));
        assert!(!g.has_trample(target));
    }
}
#[test]
fn invoker_payment_target_stale_cancel_reject_without_spending() {
    let mut g = ready();
    let source = add(&mut g, "wildheart-invoker");
    let target = add(&mut g, "bear-cub");
    let land = add(&mut g, "forest");
    g.turns.mana[0][4] = 7;
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert!(g.begin_activation(Seat::P0, d.id, source).is_err());
    assert_eq!(before, g.snapshot());
    g.turns.mana[0][4] = 8;
    g.begin_activation(Seat::P0, d.id, source)
        .expect("Invoker has enough generic mana");
    let id = g.turns.activation.as_ref().unwrap().id;
    let before = g.snapshot();
    assert!(g.finish_activation(Seat::P0, id).is_err());
    assert!(g.choose_activation_target(Seat::P0, id, land).is_err());
    assert_eq!(before, g.snapshot());
    g.choose_activation_target(Seat::P0, id, target).unwrap();
    let id = g.turns.activation.as_ref().unwrap().id;
    g.pay_activation(Seat::P0, id, mana::Color::Green).unwrap();
    let before = g.snapshot();
    assert!(g.pay_activation(Seat::P0, id, mana::Color::Green).is_err());
    assert_eq!(before, g.snapshot());
    let id = g.turns.activation.as_ref().unwrap().id;
    g.cancel_activation(Seat::P0, id).unwrap();
    assert_eq!(g.turns.mana[0][4], 8);
    assert!(g.turns.stack.is_empty());
}
#[test]
fn invoker_departed_target_does_not_boost_returned_incarnation() {
    let mut g = ready();
    let source = add(&mut g, "wildheart-invoker");
    let target = add(&mut g, "bear-cub");
    g.turns.mana[0][4] = 8;
    activate(&mut g, source, target);
    let dead = g
        .objects
        .move_to(target, Zone::Graveyard(Seat::P0))
        .unwrap();
    let new = g.objects.move_to(dead, Zone::Battlefield).unwrap();
    pass(&mut g);
    pass(&mut g);
    let c = g.creature_state(new).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2));
    assert!(g.turns.stack.is_empty());
}

#[test]
fn invoker_pending_payment_semantic_and_every_work_quantum_restore() {
    let mut g = ready();
    let source = add(&mut g, "wildheart-invoker");
    let target = add(&mut g, "bear-cub");
    g.turns.mana[0] = [1, 1, 1, 1, 3, 1];
    let d = g.turn_decision().unwrap();
    g.begin_activation(Seat::P0, d.id, source).unwrap();
    let id = g.turns.activation.as_ref().unwrap().id;
    g.choose_activation_target(Seat::P0, id, target).unwrap();
    for color in [
        mana::Color::White,
        mana::Color::Blue,
        mana::Color::Black,
        mana::Color::Red,
        mana::Color::Green,
        mana::Color::Green,
        mana::Color::Green,
        mana::Color::Colorless,
    ] {
        let id = g.turns.activation.as_ref().unwrap().id;
        g.pay_activation(Seat::P0, id, color).unwrap();
        let saved = g.snapshot();
        g.restore(&saved).unwrap();
        assert!(g.policy_observe(Seat::P1, 256).unwrap().pending.is_none());
    }
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    let request = policy::Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation,
        choices: vec![policy::Choice::FinishActivation],
    };
    let bytes = actions::encode(&g, Seat::P0, &request, 256).unwrap();
    actions::apply(&mut g, &bytes, 256).unwrap();
    assert_eq!(g.turns.mana[0], [0; 6]);
    pass(&mut g);
    let saved = g.snapshot();
    for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
        let mut r = Game::new().unwrap();
        r.restore(&saved).unwrap();
        let d = r.turn_decision().unwrap();
        let mut p = r
            .apply_turn_quantum(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Pass(d.candidate(0)),
                },
                q,
            )
            .unwrap();
        while p == Progress::InternalYield {
            let s = r.snapshot();
            r.restore(&s).unwrap();
            p = r.resume(q);
        }
        let target = r
            .objects
            .in_zone(Zone::Battlefield)
            .find(|h| r.objects.get(*h).unwrap().card.identity().key == "bear-cub")
            .unwrap();
        assert_eq!(r.creature_state(target).unwrap().power, 7);
        assert!(r.has_trample(target));
    }
}

#[test]
fn invoker_reference_literal_checkpoints() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/invoker.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/invoker-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for spec in fixture["cases"].as_array().unwrap() {
        let mode = spec["id"].as_str().unwrap();
        if mode == "payment_sources" {
            let result = super::shivan_tests::activation_mana_reference(true);
            assert_eq!(result, expected[mode], "{mode}");
            results.insert(mode.into(), result);
            continue;
        }
        let mut g = ready();
        let mut legal = true;
        let casting = ["cast", "short_cast"].contains(&mode);
        let source = g
            .objects
            .allocate(
                CardId::from_key("wildheart-invoker").unwrap(),
                Seat::P0,
                if casting {
                    Zone::Hand(Seat::P0)
                } else {
                    Zone::Battlefield
                },
            )
            .unwrap();
        let thorn = mode.starts_with("thorn");
        let owner = if mode == "opposing" {
            Seat::P1
        } else {
            Seat::P0
        };
        let target = g
            .objects
            .allocate(
                CardId::from_key(if thorn {
                    "thornweald-archer"
                } else {
                    "bear-cub"
                })
                .unwrap(),
                owner,
                Zone::Battlefield,
            )
            .unwrap();
        let dragon = if mode == "thorn_cleanup" {
            let h = add(&mut g, "shivan-dragon");
            g.turns.mana[0][3] = 1;
            let d = g.turn_decision().unwrap();
            g.begin_activation(Seat::P0, d.id, h).unwrap();
            let id = g.turns.activation.as_ref().unwrap().id;
            g.pay_activation(Seat::P0, id, mana::Color::Red).unwrap();
            let id = g.turns.activation.as_ref().unwrap().id;
            g.finish_activation(Seat::P0, id).unwrap();
            pass(&mut g);
            pass(&mut g);
            assert_eq!(g.creature_state(h).unwrap().power, 6);
            Some(h)
        } else {
            None
        };
        let mut target_now = target;
        let n = if mode == "thorn_double" { 2 } else { 1 };
        let mut source_now = Some(source);
        g.turns.mana[0][4] = if mode == "seven" { 7 } else { 8 * n };
        if mode == "mixed" {
            g.turns.mana[0] = [0, 0, 0, 4, 4, 0];
        }
        if casting {
            g.turns.mana[0][4] = if mode == "cast" { 4 } else { 3 };
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            legal = g.begin_cast(Seat::P0, d.id, source).is_ok();
            if legal {
                for _ in 0..4 {
                    let id = g.payment_decision(Seat::P0).unwrap().id;
                    g.choose_payment(Seat::P0, id, mana::Color::Green).unwrap();
                }
                let id = g.payment_decision(Seat::P0).unwrap().id;
                g.finish_cast(Seat::P0, id).unwrap();
                pass(&mut g);
                pass(&mut g);
                source_now = g.objects.in_zone(Zone::Battlefield).find(|h| {
                    g.objects.get(*h).unwrap().card.identity().key == "wildheart-invoker"
                });
            } else {
                assert_eq!(before, g.snapshot());
                source_now = None;
            }
        } else if mode == "seven" {
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            legal = g.begin_activation(Seat::P0, d.id, source).is_ok();
            assert!(!legal);
            assert_eq!(before, g.snapshot());
        } else {
            if mode == "opposing" {
                g.turns.sick.push(source);
            }
            if mode == "tapped" {
                g.objects.get_mut(source).unwrap().tapped = true;
            }
            for _ in 0..n {
                let d = g.turn_decision().unwrap();
                g.begin_activation(Seat::P0, d.id, source).unwrap();
                let id = g.turns.activation.as_ref().unwrap().id;
                if mode == "missing" || mode == "illegal" {
                    let before = g.snapshot();
                    legal = if mode == "missing" {
                        g.finish_activation(Seat::P0, id).is_ok()
                    } else {
                        let land = add(&mut g, "forest");
                        let before = g.snapshot();
                        let ok = g.choose_activation_target(Seat::P0, id, land).is_ok();
                        assert_eq!(before, g.snapshot());
                        ok
                    };
                    if mode == "missing" {
                        assert_eq!(before, g.snapshot());
                    }
                    assert!(!legal);
                    break;
                }
                g.choose_activation_target(Seat::P0, id, target).unwrap();
                for i in 0..8 {
                    let id = g.turns.activation.as_ref().unwrap().id;
                    g.pay_activation(
                        Seat::P0,
                        id,
                        if mode == "mixed" && i < 4 {
                            mana::Color::Red
                        } else {
                            mana::Color::Green
                        },
                    )
                    .unwrap();
                }
                let id = g.turns.activation.as_ref().unwrap().id;
                g.finish_activation(Seat::P0, id).unwrap();
            }
            if ["dead_source", "thorn_dead", "thorn_cleanup"].contains(&mode) {
                g.objects
                    .move_to(source, Zone::Graveyard(Seat::P0))
                    .unwrap();
                source_now = None;
            }
            if mode == "departed" {
                let dead = g.objects.move_to(target, Zone::Graveyard(owner)).unwrap();
                target_now = g.objects.move_to(dead, Zone::Battlefield).unwrap();
            }
            if legal {
                for _ in 0..n {
                    pass(&mut g);
                    pass(&mut g);
                }
            }
            if mode == "cleanup" || mode == "thorn_cleanup" {
                assert!(g.has_trample(target));
                g.turns.position = Some((1, Seat::P0, turns::Step::End));
                pass(&mut g);
                pass(&mut g);
            }
            if mode == "thorn_combat" {
                let mut blockers = vec![];
                for _ in 0..2 {
                    blockers.push(
                        g.objects
                            .allocate(
                                CardId::from_key("magnigoth-sentry").unwrap(),
                                Seat::P1,
                                Zone::Battlefield,
                            )
                            .unwrap(),
                    );
                }
                g.turns.position = Some((1, Seat::P0, turns::Step::BeginningCombat));
                pass(&mut g);
                pass(&mut g);
                let d = g.turn_decision().unwrap();
                let d = g.select_attackers(d.actor, d.id, &[target]).unwrap();
                g.finish_combat(d.actor, d.id).unwrap();
                pass(&mut g);
                pass(&mut g);
                let d = g.turn_decision().unwrap();
                let d = g
                    .select_blockers(
                        d.actor,
                        d.id,
                        &blockers.iter().map(|h| (*h, target)).collect::<Vec<_>>(),
                    )
                    .unwrap();
                g.finish_combat(d.actor, d.id).unwrap();
                pass(&mut g);
                pass(&mut g);
                let d = g.turn_decision().unwrap();
                let d = g
                    .assign_combat_damage(
                        d.actor,
                        d.id,
                        target,
                        &blockers.iter().map(|h| (*h, 1)).collect::<Vec<_>>(),
                    )
                    .unwrap();
                g.finish_combat(d.actor, d.id).unwrap();
                assert!(blockers.iter().all(|h| g.objects.get(*h).is_err()));
                assert_eq!(g.life(), [20, 15]);
            }
        }
        let stats = |h: Handle| {
            g.creature_state(h)
                .map(|c| vec![c.power, c.toughness, c.damage])
        };
        let actual = serde_json::json!({"dragon":dragon.and_then(stats),"source":source_now.and_then(stats),"target":stats(target_now),"trample":g.has_trample(target_now),"life":g.life(),"mana":g.turns.mana[0].iter().sum::<u32>(),"stack":g.turns.stack.len(),"legal":legal});
        assert_eq!(&actual, &expected[mode], "{mode}");
        results.insert(mode.into(), actual);
    }
    if let Ok(path) = std::env::var("MTG_INVOKER_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}

#[test]
fn invoker_atomic_unavailable_reserved_mana_missing_target_overflow_and_concession() {
    for seat in [Seat::P0, Seat::P1] {
        let mut g = ready();
        if seat == Seat::P1 {
            pass(&mut g);
        }
        let source = g
            .objects
            .allocate(
                CardId::from_key("wildheart-invoker").unwrap(),
                seat,
                Zone::Battlefield,
            )
            .unwrap();
        let target = add(&mut g, "bear-cub");
        let index = seat_index(seat);
        g.turns.mana[index][4] = 8;
        let d = g.turn_decision().unwrap();
        g.begin_activation(seat, d.id, source).unwrap();
        let id = g.turns.activation.as_ref().unwrap().id;
        let before = g.snapshot();
        assert!(g.pay_activation(seat, id, mana::Color::Green).is_err());
        assert_eq!(before, g.snapshot());
        g.choose_activation_target(seat, id, target).unwrap();
        for _ in 0..8 {
            let id = g.turns.activation.as_ref().unwrap().id;
            g.pay_activation(seat, id, mana::Color::Green).unwrap();
        }
        let id = g.turns.activation.as_ref().unwrap().id;
        let before = g.snapshot();
        assert!(g.pay_activation(seat, id, mana::Color::Green).is_err());
        assert_eq!(before, g.snapshot());
        g.turns.mana[index][4] = 7;
        let before = g.snapshot();
        assert!(g.finish_activation(seat, id).is_err());
        assert_eq!(before, g.snapshot());
        g.turns.mana[index][4] = 8;
        g.objects
            .move_to(target, Zone::Graveyard(Seat::P0))
            .unwrap();
        let before = g.snapshot();
        assert!(g.finish_activation(seat, id).is_err());
        assert_eq!(before, g.snapshot());
        g.concede(seat, g.episode_id().unwrap()).unwrap();
        let before = g.snapshot();
        assert!(g.finish_activation(seat, id).is_err());
        assert_eq!(before, g.snapshot());
        assert_eq!(g.turns.mana[index][4], 8);
        for viewer in [Seat::P0, Seat::P1] {
            let o = g.policy_observe(viewer, 256).unwrap();
            assert!(o.pending.is_none() && o.decision.is_none());
        }
    }
    let mut g = ready();
    let source = add(&mut g, "wildheart-invoker");
    let target = add(&mut g, "bear-cub");
    g.turns.mana[0][4] = 8;
    activate(&mut g, source, target);
    pass(&mut g);
    g.turns.modifications.push(targets::Modification {
        handle: target,
        boost: u32::MAX - 3,
        power_boost: 0,
        damage: 0,
    });
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert!(
        g.apply_turn(
            d.actor,
            &TurnAction {
                decision: d.id,
                selection: TurnSelection::Pass(d.candidate(0))
            }
        )
        .is_err()
    );
    assert_eq!(before, g.snapshot());
}
