//! Synthetic positions, real cast/placement/resolution. Oracle Archer/Cyclops;
//! CR 601.2i, 603.2/603.3, 113.7a, 400.7, 611.2a.
use super::*;

fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 205, 0).unwrap();
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
fn add(g: &mut Game, key: &str, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, zone)
        .unwrap()
}
fn growth(g: &mut Game, target: Handle) {
    let h = add(g, "giant-growth", Seat::P0, Zone::Hand(Seat::P0));
    g.turns.mana[0][4] = 1;
    let d = g.turn_decision().unwrap();
    let t = g.begin_targeted_cast(Seat::P0, d.id, h, 256).unwrap();
    let t = g.choose_target(Seat::P0, t.id, target).unwrap();
    let p = g.finish_targets(Seat::P0, t.id).unwrap();
    let p = g
        .choose_payment(Seat::P0, p.id, mana::Color::Green)
        .unwrap();
    g.finish_cast(Seat::P0, p.id).unwrap();
}
fn order(g: &mut Game, rows: &[usize]) {
    let d = g.turn_decision().unwrap();
    assert_eq!(d.kind, turns::TurnKind::TriggerOrder);
    g.order_triggers_quantum(d.actor, d.id, rows, NonZeroUsize::MAX)
        .unwrap();
}
#[test]
fn archer_committed_cast_waits_for_resolution_and_survives_source_death() {
    let mut g = ready();
    let a = add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
    let cub = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    growth(&mut g, cub);
    assert_eq!(g.life(), [20, 20]);
    assert_eq!(g.creature_state(cub).unwrap().power, 2);
    assert_eq!(
        g.trigger_candidates(Seat::P0).len(),
        1,
        "CR 601.2i/603: committed noncreature cast triggers Archer"
    );
    order(&mut g, &[0]);
    assert_eq!(g.turns.stack.len(), 2);
    g.objects.move_to(a, Zone::Graveyard(Seat::P0)).unwrap();
    pair(&mut g);
    assert_eq!(g.life(), [20, 19], "CR 113.7a: dead source still deals one");
    assert_eq!(g.creature_state(cub).unwrap().power, 2);
    pair(&mut g);
    assert_eq!(g.creature_state(cub).unwrap().power, 5);
}
#[test]
fn cyclops_trigger_then_growth_literal_stat_ledger() {
    let mut g = ready();
    let c = add(&mut g, "crackling-cyclops", Seat::P0, Zone::Battlefield);
    assert_eq!(
        g.creature_state(c).map(|c| (c.power, c.toughness)),
        Some((0, 4)),
        "Pinned Cyclops is 0/4"
    );
    growth(&mut g, c);
    assert_eq!(g.creature_state(c).unwrap().power, 0);
    order(&mut g, &[0]);
    pair(&mut g);
    assert_eq!(
        g.creature_state(c).map(|c| (c.power, c.toughness)),
        Some((3, 4))
    );
    pair(&mut g);
    assert_eq!(
        g.creature_state(c).map(|c| (c.power, c.toughness)),
        Some((6, 7))
    );
    g.turns.position = Some((1, Seat::P0, turns::Step::End));
    pair(&mut g);
    assert_eq!(
        g.creature_state(c).map(|c| (c.power, c.toughness)),
        Some((0, 4))
    );
}

fn cast_plain(g: &mut Game, key: &str, n: usize) {
    let h = add(g, key, Seat::P0, Zone::Hand(Seat::P0));
    g.turns.mana[0][3] = n as u32;
    let d = g.turn_decision().unwrap();
    let mut p = g.begin_cast(Seat::P0, d.id, h).unwrap();
    for _ in 0..n {
        p = g.choose_payment(Seat::P0, p.id, mana::Color::Red).unwrap();
    }
    g.finish_cast(Seat::P0, p.id).unwrap();
}
#[test]
fn cast_trigger_all_six_orders_and_quantum_snapshots() {
    for rows in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let mut g = ready();
            add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
            add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
            add(&mut g, "crackling-cyclops", Seat::P0, Zone::Battlefield);
            cast_plain(&mut g, "dragon-fodder", 2);
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            for bad in [vec![0, 1], vec![0, 0, 2], vec![0, 1, 3]] {
                assert!(g.order_triggers_quantum(d.actor, d.id, &bad, q).is_err());
                assert_eq!(g.snapshot(), before);
            }
            let mut progress = g.order_triggers_quantum(d.actor, d.id, &rows, q).unwrap();
            while progress == Progress::InternalYield {
                let s = g.snapshot();
                g.restore(&s).unwrap();
                progress = g.resume(q);
            }
            assert_eq!(g.life(), [20, 20]);
            let kinds: Vec<_> = g
                .turns
                .triggered
                .iter()
                .map(|a| a.declaration.card.identity().key)
                .collect();
            let names = ["firebrand-archer", "firebrand-archer", "crackling-cyclops"];
            assert_eq!(kinds, rows.map(|r| names[r]));
            let mut life = 20;
            let mut power = 0;
            for row in rows.into_iter().rev() {
                pass(&mut g);
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
                if row == 2 {
                    power += 3;
                } else {
                    life -= 1;
                }
                assert_eq!(g.life(), [20, life]);
                let c = g
                    .objects
                    .in_zone(Zone::Battlefield)
                    .find(|h| g.objects.get(*h).unwrap().card.identity().key == "crackling-cyclops")
                    .unwrap();
                assert_eq!(g.creature_state(c).unwrap().power, power);
                assert!(
                    !g.objects.in_zone(Zone::Battlefield).any(|h| g
                        .objects
                        .get(h)
                        .unwrap()
                        .card
                        .identity()
                        .key
                        == "goblin-token")
                );
            }
            pair(&mut g);
            assert_eq!(
                g.objects
                    .in_zone(Zone::Battlefield)
                    .filter(|h| g.objects.get(*h).unwrap().card.identity().key == "goblin-token")
                    .count(),
                2
            );
        }
    }
}
#[test]
fn cast_trigger_negative_land_mana_creature_failed_and_cancelled() {
    let mut g = ready();
    add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
    let c = add(&mut g, "crackling-cyclops", Seat::P0, Zone::Battlefield);
    let land = add(&mut g, "mountain", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    g.play_land(Seat::P0, d.id, land).unwrap();
    let land = g
        .objects
        .in_zone(Zone::Battlefield)
        .find(|h| g.objects.get(*h).unwrap().card.identity().key == "mountain")
        .unwrap();
    let d = g.turn_decision().unwrap();
    g.tap_mana(Seat::P0, d.id, land).unwrap();
    cast_plain(&mut g, "swab-goblin", 2);
    assert!(g.turns.pending_triggers.is_empty());
    pair(&mut g);
    let h = add(&mut g, "giant-growth", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert!(g.begin_targeted_cast(Seat::P0, d.id, h, 256).is_err());
    assert_eq!(g.snapshot(), before);
    g.turns.mana[0][4] = 1;
    let t = g.begin_targeted_cast(Seat::P0, d.id, h, 256).unwrap();
    let t = g.choose_target(Seat::P0, t.id, c).unwrap();
    let p = g.finish_targets(Seat::P0, t.id).unwrap();
    let before = g.snapshot();
    assert!(g.finish_cast(Seat::P0, p.id).is_err());
    assert_eq!(g.snapshot(), before);
    let p = g
        .choose_payment(Seat::P0, p.id, mana::Color::Green)
        .unwrap();
    g.cancel_payment(Seat::P0, p.id).unwrap();
    assert!(g.turns.pending_triggers.is_empty() && g.turns.stack.is_empty());
    assert_eq!(g.life(), [20, 20]);
    assert_eq!(g.creature_state(c).unwrap().power, 0);
    assert_eq!(g.turns.mana[0][4], 1);
}
#[test]
fn cast_trigger_opponent_cast_and_departed_cyclops_incarnation() {
    let mut g = ready();
    add(&mut g, "firebrand-archer", Seat::P1, Zone::Battlefield);
    let c = add(&mut g, "crackling-cyclops", Seat::P0, Zone::Battlefield);
    growth(&mut g, c);
    assert_eq!(g.trigger_candidates(Seat::P0).len(), 1);
    assert!(g.trigger_candidates(Seat::P1).is_empty());
    order(&mut g, &[0]);
    let dead = g.objects.move_to(c, Zone::Graveyard(Seat::P0)).unwrap();
    let returned = g.objects.move_to(dead, Zone::Battlefield).unwrap();
    pair(&mut g);
    pair(&mut g);
    assert_eq!(g.creature_state(returned).unwrap().power, 0);
    assert_eq!(g.life(), [20, 20]);
}
#[test]
fn cast_trigger_two_archers_growth_and_invalidated_spell() {
    let mut g = ready();
    add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
    add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
    let c = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    growth(&mut g, c);
    order(&mut g, &[1, 0]);
    g.objects.move_to(c, Zone::Graveyard(Seat::P0)).unwrap();
    for life in [19, 18, 18] {
        pair(&mut g);
        assert_eq!(g.life(), [20, life]);
    }
    assert!(g.turns.stack.is_empty());
}

#[test]
fn cast_trigger_reference_literal_checkpoints() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/cast-triggers.json"
    ))
    .unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/cast-triggers-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for spec in fixture["cases"].as_array().unwrap() {
        let id = spec["id"].as_str().unwrap();
        let mut g = ready();
        let mut archers = vec![];
        for _ in 0..spec["archers"].as_u64().unwrap() {
            archers.push(add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield));
        }
        let cyclops = spec["cyclops"]
            .as_bool()
            .unwrap()
            .then(|| add(&mut g, "crackling-cyclops", Seat::P0, Zone::Battlefield));
        let target =
            cyclops.unwrap_or_else(|| add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield));
        let point = |g: &Game| {
            serde_json::json!([
                g.life(),
                cyclops.and_then(|c| g.creature_state(c).map(|c| [c.power, c.toughness])),
                g.turns.stack.len(),
                g.objects
                    .in_zone(Zone::Battlefield)
                    .filter(|h| g.objects.get(*h).unwrap().card.identity().key == "goblin-token")
                    .count()
            ])
        };
        let key = spec["spell"].as_str().unwrap();
        match id {
            "apnap" => {
                let other = add(&mut g, "firebrand-archer", Seat::P1, Zone::Battlefield);
                let declarations =
                    [(archers[0], Seat::P0), (other, Seat::P1)].map(|(source, controller)| {
                        triggers::PendingTrigger {
                            source,
                            controller,
                            card: CardId::from_key("firebrand-archer").unwrap(),
                            kind: triggers::TriggerKind::Archer,
                        }
                    });
                g.inject_triggers(declarations.to_vec());
                g.finish_work();
                order(&mut g, &[0]);
                order(&mut g, &[1]);
            }
            "no_mana" | "cancel" => {
                let h = add(&mut g, key, Seat::P0, Zone::Hand(Seat::P0));
                let d = g.turn_decision().unwrap();
                if id == "no_mana" {
                    let before = g.snapshot();
                    assert!(g.begin_targeted_cast(Seat::P0, d.id, h, 256).is_err());
                    assert_eq!(g.snapshot(), before);
                } else {
                    g.turns.mana[0][4] = 1;
                    let t = g.begin_targeted_cast(Seat::P0, d.id, h, 256).unwrap();
                    let t = g.choose_target(Seat::P0, t.id, target).unwrap();
                    let p = g.finish_targets(Seat::P0, t.id).unwrap();
                    let p = g
                        .choose_payment(Seat::P0, p.id, mana::Color::Green)
                        .unwrap();
                    g.cancel_payment(Seat::P0, p.id).unwrap();
                }
            }
            "thrill_fail" => {
                let hand: Vec<_> = g.objects.in_zone(Zone::Hand(Seat::P0)).collect();
                for h in hand {
                    g.objects.move_to(h, Zone::Graveyard(Seat::P0)).unwrap();
                }
                let h = add(&mut g, key, Seat::P0, Zone::Hand(Seat::P0));
                g.turns.mana[0][3] = 2;
                let d = g.turn_decision().unwrap();
                let before = g.snapshot();
                for _ in 0..3 {
                    assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
                    assert_eq!(g.snapshot(), before);
                }
            }
            "land" | "mana" => {
                let h = add(
                    &mut g,
                    "mountain",
                    Seat::P0,
                    if id == "land" {
                        Zone::Hand(Seat::P0)
                    } else {
                        Zone::Battlefield
                    },
                );
                let d = g.turn_decision().unwrap();
                if id == "land" {
                    g.play_land(Seat::P0, d.id, h).unwrap();
                } else {
                    g.tap_mana(Seat::P0, d.id, h).unwrap();
                }
            }
            "bite_reject" => {
                let enemy = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
                let h = add(&mut g, key, Seat::P0, Zone::Hand(Seat::P0));
                g.turns.mana[0][4] = 2;
                let d = g.turn_decision().unwrap();
                let t = g.begin_targeted_cast(Seat::P0, d.id, h, 256).unwrap();
                let t = g.choose_target(Seat::P0, t.id, archers[0]).unwrap();
                let before = g.snapshot();
                assert!(g.choose_target(Seat::P0, t.id, target).is_err());
                assert_eq!(g.snapshot(), before);
                assert!(g.turns.pending_triggers.is_empty());
                let t = g.choose_target(Seat::P0, t.id, enemy).unwrap();
                let mut p = g.finish_targets(Seat::P0, t.id).unwrap();
                for _ in 0..2 {
                    p = g
                        .choose_payment(Seat::P0, p.id, mana::Color::Green)
                        .unwrap();
                }
                g.finish_cast(Seat::P0, p.id).unwrap();
            }
            _ => {
                if key == "giant-growth" {
                    growth(&mut g, target);
                } else {
                    cast_plain(&mut g, key, 2);
                }
            }
        }
        let rows: Vec<_> = spec["order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as usize)
            .collect();
        if !rows.is_empty() {
            order(&mut g, &rows);
        }
        if let Some(death) = spec["death"].as_str() {
            g.objects
                .move_to(
                    if death == "archer" {
                        archers[0]
                    } else {
                        target
                    },
                    Zone::Graveyard(Seat::P0),
                )
                .unwrap();
        }
        let mut points = vec![point(&g)];
        while !g.turns.stack.is_empty() {
            pair(&mut g);
            points.push(point(&g));
        }
        if spec["cleanup"] == true {
            g.turns.position = Some((1, Seat::P0, turns::Step::End));
            pair(&mut g);
            points.push(point(&g));
        }
        let actual = serde_json::json!(points);
        assert_eq!(actual, expected[id], "{id}");
        results.insert(id.into(), actual);
    }
    if let Ok(path) = std::env::var("MTG_CAST_TRIGGERS_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}

#[test]
fn cast_trigger_archer_lethal_stops_before_growth_and_apnap_fixture() {
    let mut g = ready();
    let a = add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
    growth(&mut g, a);
    order(&mut g, &[0]);
    g.life[1] = 1;
    pair(&mut g);
    assert_eq!(g.life(), [20, 0]);
    assert!(g.turn_decision().is_none());
    assert_eq!(g.creature_state(a).unwrap().power, 2);
    let mut g = ready();
    // Catalog explicitly synthetic: pool casts cannot trigger both controllers.
    let mut pending = vec![];
    for seat in [Seat::P0, Seat::P1] {
        let source = add(&mut g, "firebrand-archer", seat, Zone::Battlefield);
        pending.push(triggers::PendingTrigger {
            source,
            card: CardId::from_key("firebrand-archer").unwrap(),
            controller: seat,
            kind: triggers::TriggerKind::Archer,
        });
    }
    g.inject_triggers(pending);
    g.finish_work();
    order(&mut g, &[0]);
    order(&mut g, &[1]);
    pair(&mut g);
    assert_eq!(g.life(), [19, 20]);
    pair(&mut g);
    assert_eq!(g.life(), [19, 19]);
}

#[test]
fn cast_trigger_completed_growth_on_cub_detects_each_source_once() {
    let mut g = ready();
    add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
    let c = add(&mut g, "crackling-cyclops", Seat::P0, Zone::Battlefield);
    let cub = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    growth(&mut g, cub);
    assert_eq!(g.trigger_candidates(Seat::P0).len(), 2);
    order(&mut g, &[0, 1]);
    pair(&mut g);
    assert_eq!(g.creature_state(c).unwrap().power, 3);
    assert_eq!(g.life(), [20, 20]);
    pair(&mut g);
    assert_eq!(g.life(), [20, 19]);
    assert_eq!(g.creature_state(cub).unwrap().power, 2);
    pair(&mut g);
    assert_eq!(g.creature_state(cub).unwrap().power, 5);
}
