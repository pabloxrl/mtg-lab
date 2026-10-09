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
// CR 602.2b applies 601.2g to activation: mana abilities may be activated
// during payment (also 605.3a), not only by floating mana before announcement.
#[test]
fn activation_mana_shivan_can_announce_with_untapped_mountain() {
    let mut g = ready();
    let dragon = add(&mut g, "shivan-dragon");
    add(&mut g, "mountain");
    assert_eq!(g.mana()[0][3], 0);
    assert!(g.activation_candidates(Seat::P0).contains(&dragon));
    let d = g.turn_decision().unwrap();
    g.begin_activation(Seat::P0, d.id, dragon).unwrap();
}

#[test]
fn activation_mana_payment_offers_untapped_sources_even_with_floating_mana() {
    let mut g = ready();
    let dragon = add(&mut g, "shivan-dragon");
    add(&mut g, "mountain");
    g.turns.mana[0][3] = 1;
    let d = g.turn_decision().unwrap();
    g.begin_activation(Seat::P0, d.id, dragon).unwrap();
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    assert_eq!(d.kind, "activation_payment");
    assert!(
        d.candidates
            .iter()
            .zip(&d.legal_mask)
            .any(|(c, legal)| { *legal && matches!(c, policy::Choice::TapMana { .. }) })
    );
}

#[test]
fn activation_mana_invoker_can_announce_with_eight_untapped_forests() {
    let mut g = ready();
    let invoker = add(&mut g, "wildheart-invoker");
    for _ in 0..8 {
        add(&mut g, "forest");
    }
    assert_eq!(g.mana()[0], [0; 6]);
    assert!(g.activation_candidates(Seat::P0).contains(&invoker));
    let d = g.turn_decision().unwrap();
    g.begin_activation(Seat::P0, d.id, invoker).unwrap();
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
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/shivan.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/shivan-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for spec in fixture["cases"].as_array().unwrap() {
        let mode = spec["mode"].as_str().unwrap();
        if mode == "payment_sources" {
            let result = super::shivan_tests::activation_mana_reference(false);
            assert_eq!(result, expected[mode], "{mode}");
            results.insert(mode.into(), result);
            continue;
        }
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
        assert_eq!(result, expected[mode], "{mode}");
        results.insert(mode.into(), result);
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

#[test]
fn activation_mana_private_cancel_commit_surplus_and_resolution() {
    // Synthetic position; R ability on printed 5/5 flying Shivan, CR 602.2b,
    // 601.2g/h, 605.3a, 611.2. Public pools/taps change only on atomic commit.
    let mut g = ready();
    let dragon = add(&mut g, "shivan-dragon");
    let mountain = add(&mut g, "mountain");
    g.turns.sick.push(dragon);
    g.turns.mana[0][3] = 1;
    for cancel in [true, false] {
        command(
            &mut g,
            serde_json::json!({"kind":"activate","card":{"zone":"battlefield","row":0}}),
        );
        let before = g.policy_observe(Seat::P1, 256).unwrap();
        command(
            &mut g,
            serde_json::json!({"kind":"tap_mana","card":{"zone":"battlefield","row":1}}),
        );
        assert_eq!(g.mana()[0][3], 1);
        assert!(!g.objects.get(mountain).unwrap().tapped);
        assert!(g.turns.stack.is_empty());
        assert!(g.turn_decision().is_none());
        assert_eq!(
            serde_json::to_value(before).unwrap(),
            serde_json::to_value(g.policy_observe(Seat::P1, 256).unwrap()).unwrap()
        );
        let pending = g.policy_observe(Seat::P0, 256).unwrap().pending.unwrap();
        assert_eq!(pending.pool.unwrap()[3], 2);
        command(&mut g, serde_json::json!({"kind":"pay","color":3}));
        command(
            &mut g,
            serde_json::json!({"kind":if cancel {"cancel_activation"} else {"finish_activation"}}),
        );
        assert_eq!(g.mana()[0][3], 1);
        assert_eq!(g.objects.get(mountain).unwrap().tapped, !cancel);
        assert_eq!(g.turns.stack.len(), usize::from(!cancel));
    }
    pass(&mut g);
    pass(&mut g);
    assert_eq!(g.creature_state(dragon).unwrap().power, 6);
    assert_eq!(g.creature_state(dragon).unwrap().toughness, 5);
}

fn attempt(g: &mut Game, value: serde_json::Value) -> Result<(), policy::PolicyError> {
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    g.apply_policy(
        Seat::P0,
        &policy::Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![serde_json::from_value(value).unwrap()],
        },
        256,
    )
}

#[test]
fn activation_mana_invoker_target_first_mixed_sources_and_sickness() {
    // Synthetic mana sources with explicit age/haste. CR 302.6 and 702.10:
    // sick Elf/Druid tap costs are restricted; Invoker's nontap cost is not.
    let mut g = ready();
    let invoker = add(&mut g, "wildheart-invoker");
    let cub = add(&mut g, "bear-cub");
    let elf = add(&mut g, "llanowar-elves");
    let druid = add(&mut g, "druid-of-the-cowl");
    let mountain = add(&mut g, "mountain");
    let forest = add(&mut g, "forest");
    g.turns.sick.extend([invoker, elf, druid]);
    g.turns.haste.push(druid);
    g.turns.mana[0][4] = 5;
    command(
        &mut g,
        serde_json::json!({"kind":"activate","card":{"zone":"battlefield","row":0}}),
    );
    for value in [
        serde_json::json!({"kind":"tap_mana","card":{"zone":"battlefield","row":3}}),
        serde_json::json!({"kind":"pay","color":4}),
        serde_json::json!({"kind":"finish_activation"}),
    ] {
        let before = format!("{g:?}");
        assert!(attempt(&mut g, value).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
    command(
        &mut g,
        serde_json::json!({"kind":"target","card":{"zone":"battlefield","row":1}}),
    );
    let before = format!("{g:?}");
    assert!(
        attempt(
            &mut g,
            serde_json::json!({"kind":"tap_mana","card":{"zone":"battlefield","row":2}})
        )
        .is_err()
    );
    assert_eq!(format!("{g:?}"), before);
    for row in [3, 4, 5] {
        command(
            &mut g,
            serde_json::json!({"kind":"tap_mana","card":{"zone":"battlefield","row":row}}),
        );
        let before = format!("{g:?}");
        assert!(
            attempt(
                &mut g,
                serde_json::json!({"kind":"tap_mana","card":{"zone":"battlefield","row":row}})
            )
            .is_err()
        );
        assert_eq!(format!("{g:?}"), before);
    }
    for color in [4, 4, 4, 4, 4, 4, 4, 3] {
        command(&mut g, serde_json::json!({"kind":"pay","color":color}));
    }
    command(&mut g, serde_json::json!({"kind":"finish_activation"}));
    assert_eq!(g.mana()[0], [0; 6]);
    assert!(!g.objects.get(elf).unwrap().tapped);
    for h in [druid, mountain, forest] {
        assert!(g.objects.get(h).unwrap().tapped);
    }
    assert_eq!(g.turns.stack.len(), 1);
    pass(&mut g);
    pass(&mut g);
    let c = g.creature_state(cub).unwrap();
    assert_eq!((c.power, c.toughness), (7, 7));
    assert!(g.has_trample(cub));
}

#[test]
fn activation_mana_source_revalidation_overflow_and_rejection_atomicity() {
    use mana::Color;
    use turns::TurnError;
    let mut g = ready();
    let dragon = add(&mut g, "shivan-dragon");
    let mountain = add(&mut g, "mountain");
    let enemy = g
        .objects
        .allocate(
            CardId::from_key("mountain").unwrap(),
            Seat::P1,
            Zone::Battlefield,
        )
        .unwrap();
    let hand = g
        .objects
        .allocate(
            CardId::from_key("mountain").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    let d = g.turn_decision().unwrap();
    g.begin_activation(Seat::P0, d.id, dragon).unwrap();
    let id = g.turns.activation.as_ref().unwrap().id;
    let before = g.snapshot();
    assert!(g.activation_tap_mana(Seat::P1, id, mountain).is_err());
    assert!(g.activation_tap_mana(Seat::P0, d.id, mountain).is_err());
    for h in [dragon, enemy, hand] {
        assert!(g.activation_tap_mana(Seat::P0, id, h).is_err());
    }
    assert!(g.pay_activation(Seat::P0, id, Color::Red).is_err());
    assert!(g.finish_activation(Seat::P0, id).is_err());
    assert_eq!(g.snapshot(), before);
    assert!(g.policy_observe(Seat::P0, 1).is_err());
    assert_eq!(g.snapshot(), before);
    g.turns.mana[0][3] = u32::MAX;
    let before = g.snapshot();
    assert_eq!(
        g.activation_tap_mana(Seat::P0, id, mountain),
        Err(TurnError::EffectOverflow)
    );
    assert_eq!(g.snapshot(), before);
    g.turns.mana[0][3] = 0;
    g.generation = u64::MAX;
    let before = g.snapshot();
    assert!(g.activation_tap_mana(Seat::P0, id, mountain).is_err());
    assert_eq!(g.snapshot(), before);
    g.generation = id.generation;
    g.activation_tap_mana(Seat::P0, id, mountain).unwrap();
    let id = g.turns.activation.as_ref().unwrap().id;
    let before = g.snapshot();
    assert!(g.activation_tap_mana(Seat::P0, id, mountain).is_err());
    assert!(g.pay_activation(Seat::P0, id, Color::Green).is_err());
    assert_eq!(g.snapshot(), before);
    g.pay_activation(Seat::P0, id, Color::Red).unwrap();
    let dead = g
        .objects
        .move_to(mountain, Zone::Graveyard(Seat::P0))
        .unwrap();
    let replacement = g.objects.move_to(dead, Zone::Battlefield).unwrap();
    let id = g.turns.activation.as_ref().unwrap().id;
    let before = g.snapshot();
    assert!(g.finish_activation(Seat::P0, id).is_err());
    assert_eq!(g.snapshot(), before);
    assert!(!g.objects.get(replacement).unwrap().tapped);
    g.cancel_activation(Seat::P0, id).unwrap();
    assert_eq!(g.mana()[0], [0; 6]);
}

#[test]
fn activation_mana_bounded_source_payment_oracle_and_semantic_snapshots() {
    // Independent enumeration: subsets of two Mountains and one Forest, plus
    // zero/one floated R. Exactly R is consumed; all other generated mana remains.
    // Traverse every order of taps/payment, restoring every private continuation.
    use std::collections::BTreeSet;
    fn walk(g: &Game, out: &mut BTreeSet<(u8, [u32; 6])>) {
        let o = g.policy_observe(Seat::P0, 256).unwrap();
        let d = o.decision.unwrap();
        assert!(g.policy_observe(Seat::P1, 256).unwrap().pending.is_none());
        for (c, legal) in d.candidates.iter().zip(&d.legal_mask) {
            if !legal {
                continue;
            }
            let s = policy::Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices: vec![c.clone()],
            };
            let bytes = actions::encode(g, Seat::P0, &s, 256).unwrap();
            let mut child = Game::new().unwrap();
            child.restore(&g.snapshot()).unwrap();
            actions::decode(&child, &bytes, 256).unwrap();
            actions::apply(&mut child, &bytes, 256).unwrap();
            if matches!(c, policy::Choice::CancelActivation) {
                assert_eq!(child.turns.stack.len(), 0);
                assert!(
                    child.objects.in_zone(Zone::Battlefield).all(|h| !child
                        .objects
                        .get(h)
                        .unwrap()
                        .tapped)
                );
                assert_eq!(child.mana(), g.mana());
            } else if matches!(c, policy::Choice::FinishActivation) {
                let taps = child
                    .objects
                    .in_zone(Zone::Battlefield)
                    .skip(1)
                    .enumerate()
                    .fold(0, |mask, (i, h)| {
                        mask | (u8::from(child.objects.get(h).unwrap().tapped) << i)
                    });
                assert_eq!(child.turns.stack.len(), 1);
                out.insert((taps, child.mana()[0]));
            } else {
                walk(&child, out);
            }
        }
    }
    for floating in [0, 1] {
        let mut expected = BTreeSet::new();
        for mask in 0u8..8 {
            let red = floating + u32::from(mask & 1 != 0) + u32::from(mask & 2 != 0);
            if red > 0 {
                expected.insert((mask, [0, 0, 0, red - 1, u32::from(mask & 4 != 0), 0]));
            }
        }
        let mut g = ready();
        let dragon = add(&mut g, "shivan-dragon");
        for key in ["mountain", "mountain", "forest"] {
            add(&mut g, key);
        }
        g.turns.mana[0][3] = floating;
        let d = g.turn_decision().unwrap();
        g.begin_activation(Seat::P0, d.id, dragon).unwrap();
        let mut actual = BTreeSet::new();
        walk(&g, &mut actual);
        assert_eq!(actual, expected);
    }
}

pub(super) fn activation_mana_reference(invoker: bool) -> serde_json::Value {
    // These are the same synthetic inputs as the real XMage payment scripts.
    // Native keeps announcement/taps private until commit; announced_stack
    // normalizes the pending nonmana object, never a mana stack object.
    let mut g = ready();
    let source = add(
        &mut g,
        if invoker {
            "wildheart-invoker"
        } else {
            "shivan-dragon"
        },
    );
    let target = invoker.then(|| add(&mut g, "bear-cub"));
    let sources: Vec<_> = (0..if invoker { 8 } else { 1 })
        .map(|_| add(&mut g, if invoker { "forest" } else { "mountain" }))
        .collect();
    let d = g.turn_decision().unwrap();
    g.begin_activation(Seat::P0, d.id, source).unwrap();
    if let Some(target) = target {
        let id = g.turns.activation.as_ref().unwrap().id;
        g.choose_activation_target(Seat::P0, id, target).unwrap();
    }
    let mut payment = vec![];
    for (i, h) in sources.iter().enumerate() {
        let id = g.turns.activation.as_ref().unwrap().id;
        g.activation_tap_mana(Seat::P0, id, *h).unwrap();
        let p = g.turns.activation.as_ref().unwrap();
        assert_eq!(p.target, target);
        assert!(g.turn_decision().is_none());
        assert!(g.turns.stack.is_empty());
        assert_eq!(g.mana()[0], [0; 6]);
        assert!(!g.objects.get(*h).unwrap().tapped);
        payment.push(serde_json::json!({"stage":"mana","source":format!("pay{}",i+1),"target":target.map(|_|"bear-cub"),"pool":g.activation_pool(p).unwrap().iter().sum::<u32>(),"tapped":p.sources.len(),"announced_stack":g.turns.stack.len()+usize::from(g.turns.activation.is_some()),"priority_calls":0}));
        let id = p.id;
        g.pay_activation(
            Seat::P0,
            id,
            if invoker {
                mana::Color::Green
            } else {
                mana::Color::Red
            },
        )
        .unwrap();
    }
    let id = g.turns.activation.as_ref().unwrap().id;
    g.finish_activation(Seat::P0, id).unwrap();
    payment.push(serde_json::json!({"stage":"committed","source":null,"target":target.map(|_|"bear-cub"),"pool":g.mana()[0].iter().sum::<u32>(),"tapped":sources.iter().filter(|h|g.objects.get(**h).unwrap().tapped).count(),"announced_stack":g.turns.stack.len(),"priority_calls":0}));
    pass(&mut g);
    pass(&mut g);
    let stats = |h| {
        let c = g.creature_state(h).unwrap();
        [c.power, c.toughness, c.damage]
    };
    if let Some(target) = target {
        serde_json::json!({"source":stats(source),"target":stats(target),"trample":g.has_trample(target),"life":g.life(),"mana":g.mana()[0].iter().sum::<u32>(),"stack":g.turns.stack.len(),"legal":true,"dragon":null,"payment":payment})
    } else {
        serde_json::json!({"dragon":stats(source),"others":[],"life":g.life(),"mana":g.mana()[0][3],"stack":g.turns.stack.len(),"legal":true,"payment":payment})
    }
}

#[test]
fn activation_mana_commit_revalidates_resources_and_exhaustion() {
    // Synthetic departures/ownership changes cannot occur through a response
    // window during payment, but every public commit boundary must revalidate.
    for fault in 0..8 {
        let mut g = ready();
        let invoker = add(&mut g, "wildheart-invoker");
        let cub = add(&mut g, "bear-cub");
        let forest = add(&mut g, "forest");
        g.turns.mana[0][4] = 7;
        let d = g.turn_decision().unwrap();
        g.begin_activation(Seat::P0, d.id, invoker).unwrap();
        let id = g.turns.activation.as_ref().unwrap().id;
        g.choose_activation_target(Seat::P0, id, cub).unwrap();
        let id = g.turns.activation.as_ref().unwrap().id;
        g.activation_tap_mana(Seat::P0, id, forest).unwrap();
        for _ in 0..8 {
            let id = g.turns.activation.as_ref().unwrap().id;
            g.pay_activation(Seat::P0, id, mana::Color::Green).unwrap();
        }
        match fault {
            0 => {
                g.objects.get_mut(forest).unwrap().tapped = true;
            }
            1 => {
                g.objects.get_mut(forest).unwrap().controller = Seat::P1;
            }
            2 => {
                g.objects.move_to(forest, Zone::Hand(Seat::P0)).unwrap();
            }
            3 => {
                let dead = g.objects.move_to(cub, Zone::Graveyard(Seat::P0)).unwrap();
                g.objects.move_to(dead, Zone::Battlefield).unwrap();
            }
            4 => {
                let dead = g
                    .objects
                    .move_to(invoker, Zone::Graveyard(Seat::P0))
                    .unwrap();
                g.objects.move_to(dead, Zone::Battlefield).unwrap();
            }
            5 => {
                g.turns.mana[0][4] = 6;
            }
            6 => {
                g.generation = u64::MAX;
            }
            7 => {
                g.objects.test_exhaust_births_after_one();
                add(&mut g, "forest");
            }
            _ => unreachable!(),
        }
        let id = g.turns.activation.as_ref().unwrap().id;
        let before = g.snapshot();
        assert!(g.finish_activation(Seat::P0, id).is_err(), "fault {fault}");
        assert_eq!(before, g.snapshot(), "fault {fault}");
        assert!(g.turns.stack.is_empty());
    }
}

#[test]
fn activation_mana_hidden_information_and_cancel_all_invoker_stages() {
    for stage in 0..5 {
        let mut g = ready();
        let invoker = add(&mut g, "wildheart-invoker");
        let cub = add(&mut g, "bear-cub");
        let forest = add(&mut g, "forest");
        g.turns.mana[0][4] = 7;
        let d = g.turn_decision().unwrap();
        g.begin_activation(Seat::P0, d.id, invoker).unwrap();
        if stage > 0 {
            let id = g.turns.activation.as_ref().unwrap().id;
            g.choose_activation_target(Seat::P0, id, cub).unwrap();
        }
        if stage > 1 {
            let id = g.turns.activation.as_ref().unwrap().id;
            g.activation_tap_mana(Seat::P0, id, forest).unwrap();
        }
        for _ in 0..if stage == 4 {
            8
        } else {
            usize::from(stage == 3)
        } {
            let id = g.turns.activation.as_ref().unwrap().id;
            g.pay_activation(Seat::P0, id, mana::Color::Green).unwrap();
        }
        let visible = serde_json::to_value(g.policy_observe(Seat::P0, 256).unwrap()).unwrap();
        let bad = serde_json::json!({"kind":"pay","color":2});
        let before = g.snapshot();
        let error = attempt(&mut g, bad.clone());
        assert!(error.is_err());
        assert_eq!(before, g.snapshot());
        for zone in [
            Zone::Hand(Seat::P1),
            Zone::Library(Seat::P1),
            Zone::Library(Seat::P0),
        ] {
            let h = g.objects.in_zone(zone).next().unwrap();
            g.objects.get_mut(h).unwrap().card = CardId::from_key("shivan-dragon").unwrap();
        }
        assert_eq!(
            visible,
            serde_json::to_value(g.policy_observe(Seat::P0, 256).unwrap()).unwrap()
        );
        let before = g.snapshot();
        assert_eq!(attempt(&mut g, bad), error);
        assert_eq!(before, g.snapshot());
        let mut restored = Game::new().unwrap();
        restored.restore(&g.snapshot()).unwrap();
        command(
            &mut restored,
            serde_json::json!({"kind":"cancel_activation"}),
        );
        assert_eq!(restored.mana()[0][4], 7);
        assert!(restored.turns.stack.is_empty());
        assert!(
            restored
                .objects
                .in_zone(Zone::Battlefield)
                .all(|h| !restored.objects.get(h).unwrap().tapped)
        );
    }
}

#[test]
fn activation_mana_snapshot_rejects_missing_reservations_and_old_engine() {
    use sha2::{Digest, Sha256};
    let mut g = ready();
    let source = add(&mut g, "shivan-dragon");
    add(&mut g, "mountain");
    let d = g.turn_decision().unwrap();
    g.begin_activation(Seat::P0, d.id, source).unwrap();
    let before = g.snapshot();
    let mut envelope: serde_json::Value = serde_json::from_slice(&before).unwrap();
    envelope["engine"] = serde_json::json!("previous-engine-fingerprint");
    assert_eq!(
        g.restore(&serde_json::to_vec(&envelope).unwrap()),
        Err(snapshot::RestoreError::IncompatibleEngine)
    );
    assert_eq!(g.snapshot(), before);
    let mut envelope: serde_json::Value = serde_json::from_slice(&before).unwrap();
    let mut payload: serde_json::Value =
        serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
    payload["turns"]["activation"]
        .as_object_mut()
        .unwrap()
        .remove("sources");
    let payload = serde_json::to_string(&payload).unwrap();
    envelope["sha256"] = serde_json::json!(format!("{:x}", Sha256::digest(payload.as_bytes())));
    envelope["payload"] = serde_json::json!(payload);
    assert_eq!(
        g.restore(&serde_json::to_vec(&envelope).unwrap()),
        Err(snapshot::RestoreError::Corrupt)
    );
    assert_eq!(g.snapshot(), before);
}
