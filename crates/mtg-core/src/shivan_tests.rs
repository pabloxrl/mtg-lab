//! Pinned Shivan Oracle: 4RR 5/5 flying, R: +1/+0 until end of turn.
//! CR 602, 302.6, 113.7a, 400.7, 611.2a, 613. Synthetic unit positions.
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
fn command(g: &mut Game, value: serde_json::Value) {
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    let c = serde_json::from_value(value).unwrap();
    g.apply_policy(
        Seat::P0,
        &policy::Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![c],
        },
        256,
    )
    .unwrap();
}
fn activate(g: &mut Game) {
    command(
        g,
        serde_json::json!({"kind":"activate","card":{"zone":"battlefield","row":0}}),
    );
    command(g, serde_json::json!({"kind":"pay","color":3}));
    command(g, serde_json::json!({"kind":"finish_activation"}));
}
#[test]
fn shivan_sick_repeated_paid_boost_cleanup() {
    let mut g = ready();
    let h = add(&mut g, "shivan-dragon");
    g.turns.sick.push(h);
    g.turns.mana[0][3] = 2;
    assert!(
        g.activation_candidates(Seat::P0).contains(&h),
        "CR 302.6: nontap activation ignores sickness"
    );
    for _ in 0..2 {
        activate(&mut g);
    }
    assert_eq!(g.turns.mana[0][3], 0);
    assert_eq!(g.turns.stack.len(), 2);
    assert_eq!(g.creature_state(h).unwrap().power, 5);
    for power in [6, 7] {
        pass(&mut g);
        pass(&mut g);
        let c = g.creature_state(h).unwrap();
        assert_eq!((c.power, c.toughness), (power, 5));
    }
    assert!(!g.objects.get(h).unwrap().tapped);
    g.turns.position = Some((1, Seat::P0, turns::Step::End));
    pass(&mut g);
    pass(&mut g);
    let c = g.creature_state(h).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5));
}
#[test]
fn shivan_invalid_sources_and_foreign_target_atomic() {
    let mut g = ready();
    let h = add(&mut g, "shivan-dragon");
    let target = add(&mut g, "bear-cub");
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert!(g.begin_activation(Seat::P0, d.id, h).is_err());
    assert_eq!(g.snapshot(), before);
    g.turns.mana[0][3] = 1;
    g.begin_activation(Seat::P0, d.id, h)
        .expect("paid Shivan activation available");
    let id = g.turns.activation.as_ref().unwrap().id;
    let before = g.snapshot();
    assert!(g.choose_activation_target(Seat::P0, id, target).is_err());
    assert_eq!(g.snapshot(), before);
    assert!(g.finish_activation(Seat::P0, id).is_err());
    assert_eq!(g.snapshot(), before);
}
#[test]
fn shivan_stacked_source_return_cannot_receive_boost() {
    let mut g = ready();
    let h = add(&mut g, "shivan-dragon");
    g.turns.mana[0][3] = 1;
    assert!(g.activation_candidates(Seat::P0).contains(&h));
    activate(&mut g);
    let dead = g.objects.move_to(h, Zone::Graveyard(Seat::P0)).unwrap();
    assert_eq!(g.turns.stack.len(), 1);
    let new = g.objects.move_to(dead, Zone::Battlefield).unwrap();
    pass(&mut g);
    pass(&mut g);
    assert!(g.turns.stack.is_empty());
    assert_eq!(g.creature_state(new).unwrap().power, 5);
}

fn cast_targeted(g: &mut Game, seat: Seat, key: &str, ts: &[Handle]) {
    g.turns.mana[seat_index(seat)][4] = 2;
    let h = g
        .objects
        .allocate(CardId::from_key(key).unwrap(), seat, Zone::Hand(seat))
        .unwrap();
    let d = g.turn_decision().unwrap();
    let mut t = g.begin_targeted_cast(seat, d.id, h, 256).unwrap();
    for h in ts {
        t = g.choose_target(seat, t.id, *h).unwrap();
    }
    let mut p = g.finish_targets(seat, t.id).unwrap();
    for _ in 0..if key == "bite-down" { 2 } else { 1 } {
        p = g.choose_payment(seat, p.id, mana::Color::Green).unwrap();
    }
    g.finish_cast(seat, p.id).unwrap();
}
fn pair(g: &mut Game) {
    pass(g);
    pass(g);
}
#[test]
fn shivan_reference_literal_checkpoints() {
    let fixture_text = std::env::var("MTG_SHIVAN_FIXTURE")
        .map(|p| std::fs::read_to_string(p).unwrap())
        .unwrap_or_else(|_| include_str!("../../../fixtures/reference/shivan.json").into());
    let fixture: serde_json::Value = serde_json::from_str(&fixture_text).unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/shivan-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for spec in fixture["cases"].as_array().unwrap() {
        if spec["id"].as_str().unwrap().starts_with("exact_") {
            let result = exact::execute(spec);
            let id = spec["id"].as_str().unwrap();
            exact::assert_observation(&expected[id], &result, id);
            results.insert(id.into(), result);
            continue;
        }
        let mode = spec["mode"].as_str().unwrap();
        let mut g = ready();
        let mut legal = true;
        let casting = mode == "cast" || mode == "short_cast";
        let mut dragon = g
            .objects
            .allocate(
                CardId::from_key("shivan-dragon").unwrap(),
                Seat::P0,
                if casting {
                    Zone::Hand(Seat::P0)
                } else {
                    Zone::Battlefield
                },
            )
            .unwrap();
        let combat = mode.starts_with("split_")
            || [
                "sentry",
                "cub_illegal",
                "thorn",
                "excess",
                "grown_split",
                "haste",
            ]
            .contains(&mode);
        let mut others = vec![];
        if mode == "bite" {
            let h = g
                .objects
                .allocate(
                    CardId::from_key("shivan-dragon").unwrap(),
                    Seat::P1,
                    Zone::Battlefield,
                )
                .unwrap();
            cast_targeted(&mut g, Seat::P0, "giant-growth", &[h]);
            pair(&mut g);
            others.push(h);
        }
        if mode == "thorn_bite" {
            others.push(
                g.objects
                    .allocate(
                        CardId::from_key("thornweald-archer").unwrap(),
                        Seat::P1,
                        Zone::Battlefield,
                    )
                    .unwrap(),
            );
        }
        if combat && mode != "haste" {
            let key = match mode {
                "cub_illegal" => "bear-cub",
                "thorn" => "thornweald-archer",
                _ => "magnigoth-sentry",
            };
            for _ in 0..if spec["amounts"].is_array() { 2 } else { 1 } {
                others.push(
                    g.objects
                        .allocate(CardId::from_key(key).unwrap(), Seat::P1, Zone::Battlefield)
                        .unwrap(),
                );
            }
        }
        if casting {
            g.turns.mana[0][3] = if mode == "cast" { 6 } else { 5 };
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            legal = g.begin_cast(Seat::P0, d.id, dragon).is_ok();
            if !legal {
                assert_eq!(g.snapshot(), before);
            } else {
                for _ in 0..6 {
                    let id = g.payment_decision(Seat::P0).unwrap().id;
                    g.choose_payment(Seat::P0, id, mana::Color::Red).unwrap();
                }
                let id = g.payment_decision(Seat::P0).unwrap().id;
                g.finish_cast(Seat::P0, id).unwrap();
                pair(&mut g);
                dragon = g.objects.in_zone(Zone::Battlefield).next().unwrap();
                assert!(g.summoning_sick(dragon));
            }
        } else if ["no_red", "foreign", "dead_before"].contains(&mode) {
            g.turns.mana[0][3] = u32::from(mode != "no_red");
            if mode == "dead_before" {
                g.objects
                    .move_to(dragon, Zone::Graveyard(Seat::P0))
                    .unwrap();
            }
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            if mode == "foreign" {
                g.begin_activation(Seat::P0, d.id, dragon).unwrap();
                let target = add(&mut g, "bear-cub");
                let id = g.turns.activation.as_ref().unwrap().id;
                let before = g.snapshot();
                legal = g.choose_activation_target(Seat::P0, id, target).is_ok();
                assert_eq!(before, g.snapshot());
            } else {
                legal = g.begin_activation(Seat::P0, d.id, dragon).is_ok();
                assert_eq!(before, g.snapshot());
            }
        } else if mode == "thorn_bite" {
            pass(&mut g);
            cast_targeted(&mut g, Seat::P1, "bite-down", &[others[0], dragon]);
            pair(&mut g);
        } else if !combat || mode == "haste" {
            if mode == "bite" {
                cast_targeted(&mut g, Seat::P0, "bite-down", &[dragon, others[0]]);
            }
            let n = if ["double", "bite"].contains(&mode) {
                2
            } else {
                1
            };
            g.turns.mana[0][3] = n;
            if ["sick", "cleanup", "haste"].contains(&mode) {
                g.turns.sick.push(dragon);
            }
            for _ in 0..n {
                activate(&mut g);
            }
            if mode == "dead_after" {
                g.objects
                    .move_to(dragon, Zone::Graveyard(Seat::P0))
                    .unwrap();
                assert_eq!(g.turns.stack.len(), 1);
            }
            for _ in 0..n {
                pair(&mut g);
            }
            if mode == "bite" {
                pair(&mut g);
            }
            if mode == "cleanup" {
                assert_eq!(g.creature_state(dragon).unwrap().power, 6);
                g.turns.position = Some((1, Seat::P0, turns::Step::End));
                pair(&mut g);
            }
            if mode == "haste" {
                let cav = add(&mut g, "axgard-cavalry");
                let d = g.turn_decision().unwrap();
                g.begin_activation(Seat::P0, d.id, cav).unwrap();
                let id = g.turns.activation.as_ref().unwrap().id;
                g.choose_activation_target(Seat::P0, id, dragon).unwrap();
                let id = g.turns.activation.as_ref().unwrap().id;
                g.finish_activation(Seat::P0, id).unwrap();
                pair(&mut g);
                assert!(!g.summoning_sick(dragon));
            }
        }
        if combat {
            g.turns.position = Some((1, Seat::P0, turns::Step::BeginningCombat));
            pair(&mut g);
            let d = g.turn_decision().unwrap();
            let d = g.select_attackers(d.actor, d.id, &[dragon]).unwrap();
            g.finish_combat(d.actor, d.id).unwrap();
            pair(&mut g);
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            let blocks: Vec<_> = others.iter().map(|h| (*h, dragon)).collect();
            let selected = g.select_blockers(d.actor, d.id, &blocks);
            if mode == "cub_illegal" {
                assert!(selected.is_err());
                legal = false;
                assert_eq!(before, g.snapshot());
                g.finish_combat(d.actor, d.id).unwrap();
            } else {
                let d = selected.unwrap();
                g.finish_combat(d.actor, d.id).unwrap();
            }
            if mode == "grown_split" {
                cast_targeted(&mut g, Seat::P0, "giant-growth", &[others[0]]);
                pair(&mut g);
            }
            pair(&mut g);
            if let Some(ns) = spec["amounts"].as_array() {
                let amounts: Vec<_> = others
                    .iter()
                    .zip(ns)
                    .map(|(h, n)| (*h, n.as_u64().unwrap() as u32))
                    .collect();
                let d = g.turn_decision().unwrap();
                let before = g.snapshot();
                let a = g.assign_combat_damage(d.actor, d.id, dragon, &amounts);
                if mode == "excess" {
                    assert!(a.is_err());
                    assert_eq!(before, g.snapshot());
                    legal = false;
                } else {
                    a.unwrap();
                }
            }
            if mode != "excess" {
                let d = g.turn_decision().unwrap();
                g.finish_combat(d.actor, d.id).unwrap();
            }
        }
        let stats = |h| {
            g.objects
                .get(h)
                .ok()
                .filter(|o| o.zone == Zone::Battlefield)
                .and_then(|_| g.creature_state(h))
                .map(|c| [c.power, c.toughness, c.damage])
        };
        let result = serde_json::json!({"dragon":stats(dragon),"others":others.iter().filter_map(|h|stats(*h)).collect::<Vec<_>>(),"life":g.life(),"mana":g.turns.mana[0][3],"stack":g.turns.stack.len(),"legal":legal});
        assert_eq!(result, expected[spec["id"].as_str().unwrap()], "{mode}");
        results.insert(spec["id"].as_str().unwrap().into(), result);
    }
    if let Ok(path) = std::env::var("MTG_SHIVAN_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}

#[test]
fn shivan_payment_cancel_stale_semantic_and_quantum_restore() {
    let mut g = ready();
    let h = add(&mut g, "shivan-dragon");
    g.turns.mana[0][3] = 2;
    // A tap is not part of the cost. CR 302.6/602 permits this tapped source.
    g.objects.get_mut(h).unwrap().tapped = true;
    let d = g.turn_decision().unwrap();
    g.begin_activation(Seat::P0, d.id, h).unwrap();
    let id = g.turns.activation.as_ref().unwrap().id;
    let before = g.snapshot();
    assert!(g.pay_activation(Seat::P1, id, mana::Color::Red).is_err());
    assert!(g.pay_activation(Seat::P0, id, mana::Color::Green).is_err());
    assert_eq!(g.snapshot(), before);
    g.pay_activation(Seat::P0, id, mana::Color::Red).unwrap();
    let before = g.snapshot();
    assert!(g.finish_activation(Seat::P0, id).is_err());
    assert_eq!(before, g.snapshot());
    let id = g.turns.activation.as_ref().unwrap().id;
    assert!(g.pay_activation(Seat::P0, id, mana::Color::Red).is_err());
    assert_eq!(before, g.snapshot());
    g.cancel_activation(Seat::P0, id).unwrap();
    assert_eq!(g.turns.mana[0][3], 2);
    assert!(g.turns.stack.is_empty());
    command(
        &mut g,
        serde_json::json!({"kind":"activate","card":{"zone":"battlefield","row":0}}),
    );
    command(&mut g, serde_json::json!({"kind":"pay","color":3}));
    let saved = g.snapshot();
    let mut r = Game::new().unwrap();
    r.restore(&saved).unwrap();
    assert!(r.policy_observe(Seat::P1, 256).unwrap().pending.is_none());
    command(&mut r, serde_json::json!({"kind":"finish_activation"}));
    assert_eq!(r.turns.mana[0][3], 1);
    assert_eq!(
        r.stack_targets(*r.turns.stack.last().unwrap()),
        Some(vec![])
    );
    pass(&mut r);
    let saved = r.snapshot();
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
        let h = r.objects.in_zone(Zone::Battlefield).next().unwrap();
        let c = r.creature_state(h).unwrap();
        assert_eq!((c.power, c.toughness), (6, 5));
        assert!(r.turns.stack.is_empty());
        assert_eq!(r.objects.in_zone(Zone::Graveyard(Seat::P0)).count(), 0);
    }
    // Semantic actions reject a removed incarnation, even if its card returns.
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    let request = policy::Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation,
        choices: vec![policy::Choice::FinishActivation],
    };
    let bytes = actions::encode(&g, Seat::P0, &request, 256).unwrap();
    let dead = g.objects.move_to(h, Zone::Graveyard(Seat::P0)).unwrap();
    g.objects.move_to(dead, Zone::Battlefield).unwrap();
    let before = g.snapshot();
    assert!(actions::apply(&mut g, &bytes, 256).is_err());
    assert_eq!(before, g.snapshot());
}

#[test]
fn shivan_growth_addition_overflow_and_concession_are_atomic() {
    let mut g = ready();
    let h = add(&mut g, "shivan-dragon");
    g.turns.mana[0][3] = 1;
    activate(&mut g);
    pair(&mut g);
    cast_targeted(&mut g, Seat::P0, "giant-growth", &[h]);
    pair(&mut g);
    let c = g.creature_state(h).unwrap();
    assert_eq!((c.power, c.toughness), (9, 8));
    g.turns.mana[0][3] = 1;
    activate(&mut g);
    pass(&mut g);
    g.turns.modifications[0].power_boost = u32::MAX - 8;
    let before = g.snapshot();
    let d = g.turn_decision().unwrap();
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
    for paid in [false, true] {
        let mut g = ready();
        let h = add(&mut g, "shivan-dragon");
        g.turns.mana[0][3] = 1;
        let d = g.turn_decision().unwrap();
        g.begin_activation(Seat::P0, d.id, h).unwrap();
        if paid {
            let id = g.turns.activation.as_ref().unwrap().id;
            g.pay_activation(Seat::P0, id, mana::Color::Red).unwrap();
        }
        let id = g.turns.activation.as_ref().unwrap().id;
        g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
        let before = g.snapshot();
        assert!(g.pay_activation(Seat::P0, id, mana::Color::Red).is_err());
        assert!(g.finish_activation(Seat::P0, id).is_err());
        assert_eq!(before, g.snapshot());
        assert_eq!(g.turns.mana[0][3], 1);
        for seat in [Seat::P0, Seat::P1] {
            let o = g.policy_observe(seat, 256).unwrap();
            assert!(o.pending.is_none() && o.decision.is_none());
        }
    }
}

#[path = "cub_sentry_reference.rs"]
mod exact;
