//! GH-211: synthetic starting positions, real existing cast/trigger rules.
//! Expected ledgers are authored from pinned Oracle and CR 101.4, 603, 608,
//! 113.7a, 514 and 704, never emitted by either engine under comparison.
use super::*;
use serde_json::{Value, json};

fn ready(active: Seat) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 211, 0).unwrap();
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
    // APNAP injection is explicitly a synthetic scheduler fixture.
    g.turns.position.as_mut().unwrap().1 = active;
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

fn order(g: &mut Game, rows: &[usize]) {
    let d = g.turn_decision().unwrap();
    g.order_triggers_quantum(d.actor, d.id, rows, NonZeroUsize::MAX)
        .unwrap();
}

fn cast(g: &mut Game, key: &str, targets: &[Handle]) {
    let h = add(g, key, Seat::P0, Zone::Hand(Seat::P0));
    let color = if key == "dragon-fodder" {
        mana::Color::Red
    } else {
        mana::Color::Green
    };
    let amount = if key == "giant-growth" { 1 } else { 2 };
    g.turns.mana[0][if color == mana::Color::Red { 3 } else { 4 }] = amount;
    let d = g.turn_decision().unwrap();
    let mut payment = if targets.is_empty() {
        g.begin_cast(Seat::P0, d.id, h).unwrap()
    } else {
        let mut t = g.begin_targeted_cast(Seat::P0, d.id, h, 256).unwrap();
        for target in targets {
            t = g.choose_target(Seat::P0, t.id, *target).unwrap();
        }
        g.finish_targets(Seat::P0, t.id).unwrap()
    };
    for _ in 0..amount {
        payment = g.choose_payment(Seat::P0, payment.id, color).unwrap();
    }
    g.finish_cast(Seat::P0, payment.id).unwrap();
}

// Restore deliberately refreshes the capability scope. Fixture identity keeps
// epoch/slot/generation, including departed incarnations, but not that scope.
fn same_incarnation(a: Handle, b: Handle) -> bool {
    let mut a = serde_json::to_value(a).unwrap();
    let mut b = serde_json::to_value(b).unwrap();
    a.as_object_mut().unwrap().remove("store");
    b.as_object_mut().unwrap().remove("store");
    a == b
}

fn point(g: &Game, sources: &[(String, Handle)], creatures: &[(&str, Handle)]) -> Value {
    let mut stack = vec![];
    for h in &g.turns.stack {
        if let Some(trigger) = g.turns.triggered.iter().find(|t| t.object == *h) {
            let name = &sources
                .iter()
                .find(|(_, source)| same_incarnation(*source, trigger.declaration.source))
                .expect("every fixture source has a stable label")
                .0;
            stack.push(format!("trigger:{name}"));
        } else {
            stack.push(format!(
                "spell:{}",
                g.objects.get(*h).unwrap().card.identity().key
            ));
        }
    }
    let states: serde_json::Map<String, Value> = creatures
        .iter()
        .map(|(name, h)| {
            (
                (*name).into(),
                g.objects
                    .in_zone(Zone::Battlefield)
                    .find(|current| same_incarnation(*h, *current))
                    .and_then(|current| g.creature_state(current))
                    .map(|c| json!([c.power, c.toughness, c.damage]))
                    .unwrap_or(Value::Null),
            )
        })
        .collect();
    json!({
        "life": g.life(), "stack": stack, "creatures": states,
        "goblins": g.objects.in_zone(Zone::Battlefield)
            .filter(|h| g.objects.get(*h).unwrap().card.identity().key == "goblin-token").count(),
        "lost": g.outcome.map(|o| o.losses.map(|v| v.is_some())).unwrap_or([false, false])
    })
}

#[test]
fn m2_trigger_composition_literal_checkpoints() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/m2-trigger-composition.json"
    ))
    .unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/m2-trigger-composition-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for spec in fixture["cases"].as_array().unwrap() {
        assert_eq!(spec["setup"], "synthetic");
        let id = spec["id"].as_str().unwrap();
        let kind = spec["kind"].as_str().unwrap();
        let active = if spec["active"] == 1 {
            Seat::P1
        } else {
            Seat::P0
        };
        let mut g = ready(active);
        let mut sources = vec![];
        let mut creatures = vec![];
        if kind == "apnap" {
            let count = spec["per_seat"].as_u64().unwrap() as usize;
            let mut pending = vec![];
            for (seat_index, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
                for row in 0..count {
                    let h = add(&mut g, "firebrand-archer", seat, Zone::Battlefield);
                    sources.push((format!("p{seat_index}-archer-{row}"), h));
                    pending.push(triggers::PendingTrigger {
                        source: h,
                        controller: seat,
                        card: CardId::from_key("firebrand-archer").unwrap(),
                        kind: triggers::TriggerKind::Archer,
                    });
                }
            }
            g.inject_triggers(pending);
            if spec["dead_sources"] == true {
                for (_, h) in &sources {
                    let seat = g.objects.get(*h).unwrap().owner;
                    g.objects.move_to(*h, Zone::Graveyard(seat)).unwrap();
                }
            }
            g.finish_work();
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            // Combined APNAP order, foreign ownership and missing local rows
            // are invalid raw submissions, never translated into legal choices.
            let combined: Vec<_> = (0..2 * count).rev().collect();
            for rows in [&combined[..], &[][..]] {
                assert!(
                    g.order_triggers_quantum(d.actor, d.id, rows, NonZeroUsize::MAX)
                        .is_err()
                );
                assert_eq!(g.snapshot(), before);
            }
            for local in spec["orders"].as_array().unwrap() {
                let controller = g.turn_decision().unwrap().actor;
                let offset = if controller == Seat::P0 { 0 } else { count };
                let rows: Vec<_> = local
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| offset + v.as_u64().unwrap() as usize)
                    .collect();
                order(&mut g, &rows);
            }
        } else {
            let archer = add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
            sources.push(("p0-archer".into(), archer));
            if kind == "lethal_fodder" {
                g.life[1] = 1;
                cast(&mut g, "dragon-fodder", &[]);
                order(&mut g, &[0]);
            } else {
                let cyclops = add(&mut g, "crackling-cyclops", Seat::P0, Zone::Battlefield);
                sources.push(("p0-cyclops".into(), cyclops));
                creatures.push(("cyclops", cyclops));
                match kind {
                    "holdout" => {
                        let other = add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
                        sources.push(("p0-archer-b".into(), other));
                        creatures.push(("archer", archer));
                        creatures.push(("archer-b", other));
                        cast(&mut g, "giant-growth", &[cyclops]);
                    }
                    "growth_cub" => {
                        let cub = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
                        creatures.push(("cub", cub));
                        cast(&mut g, "giant-growth", &[cub]);
                    }
                    "cyclops_bite" | "cyclops_bite_survivor" => {
                        let enemy_card = if kind == "cyclops_bite" {
                            "bear-cub"
                        } else {
                            "magnigoth-sentry"
                        };
                        let enemy = add(&mut g, enemy_card, Seat::P1, Zone::Battlefield);
                        creatures.push(("enemy", enemy));
                        cast(&mut g, "bite-down", &[cyclops, enemy]);
                    }
                    _ => panic!("unsupported fixture kind: {kind}"),
                }
                if kind == "holdout" {
                    order(&mut g, &[0, 1, 2]);
                    // Declared synthetic departure; existing ability retains
                    // its original source incarnation (CR 113.7a).
                    g.objects
                        .move_to(cyclops, Zone::Graveyard(Seat::P0))
                        .unwrap();
                } else {
                    order(&mut g, &[0, 1]);
                }
            }
        }
        // Restore the actual chosen stack before every resolution; no expected
        // state is injected and no identity sorting may replace player order.
        let mut points = vec![point(&g, &sources, &creatures)];
        while !g.turns.stack.is_empty() && g.outcome.is_none() {
            let s = g.snapshot();
            g.restore(&s).unwrap();
            pair(&mut g);
            points.push(point(&g, &sources, &creatures));
        }
        let actual = json!(points);
        assert_eq!(actual, expected[id], "{id}");
        assert!(results.insert(id.into(), actual).is_none());
    }
    assert_eq!(results.len(), expected.as_object().unwrap().len());
    if let Ok(path) = std::env::var("MTG_M2_TRIGGER_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}
