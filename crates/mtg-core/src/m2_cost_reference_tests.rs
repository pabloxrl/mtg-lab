//! GH-209 compositions: synthetic positions, real casts/choices/stack/mana.
//! Independent CR 302.6, 702.10, 601.2h/i, 603, 605 and 121 ledgers.
use super::*;
use serde_json::{Value, json};

fn add(g: &mut Game, key: &str, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, zone)
        .unwrap()
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
fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 209, 0).unwrap();
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
    for zone in [Zone::Hand(Seat::P0), Zone::Library(Seat::P0)] {
        for h in g.objects.in_zone(zone).collect::<Vec<_>>() {
            g.objects.remove(h).unwrap();
        }
    }
    g
}
fn choice(g: &mut Game, value: Value) {
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    g.apply_policy(
        Seat::P0,
        &policy::Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![serde_json::from_value(value.clone()).unwrap()],
        },
        256,
    )
    .unwrap_or_else(|error| panic!("choice {value}: {error:?}; decision {d:?}"));
}
fn keys(g: &Game, zone: Zone) -> Vec<&'static str> {
    g.objects
        .in_zone(zone)
        .map(|h| g.objects.get(h).unwrap().card.identity().key)
        .collect()
}
fn thrill(spec: &Value) -> Value {
    let mut g = ready();
    let count = spec["archers"].as_u64().unwrap() as usize;
    for _ in 0..count {
        add(&mut g, "firebrand-archer", Seat::P0, Zone::Battlefield);
    }
    let spell = add(
        &mut g,
        "thrill-of-possibility",
        Seat::P0,
        Zone::Hand(Seat::P0),
    );
    let discard = add(
        &mut g,
        spec["discard"].as_str().unwrap(),
        Seat::P0,
        Zone::Hand(Seat::P0),
    );
    g.life[1] = spec["opponent_life"].as_i64().unwrap();
    for key in spec["library"].as_array().unwrap() {
        add(
            &mut g,
            key.as_str().unwrap(),
            Seat::P0,
            Zone::Library(Seat::P0),
        );
    }
    for _ in 0..2 {
        let land = add(&mut g, "mountain", Seat::P0, Zone::Battlefield);
        let d = g.turn_decision().unwrap();
        g.tap_mana(Seat::P0, d.id, land).unwrap();
    }
    let d = g.turn_decision().unwrap();
    g.begin_cast(Seat::P0, d.id, spell).unwrap();
    let discard_row = g
        .view_hand(Seat::P0)
        .iter()
        .position(|h| *h == discard)
        .unwrap();
    choice(
        &mut g,
        json!({"kind":"discard", "card":{"zone":"hand", "row":discard_row}}),
    );
    // The selected cost remains private and uncommitted until payment finishes.
    assert_eq!(keys(&g, Zone::Graveyard(Seat::P0)), Vec::<&str>::new());
    for _ in 0..2 {
        let p = g.payment_decision(Seat::P0).unwrap();
        g.choose_payment(Seat::P0, p.id, mana::Color::Red).unwrap();
    }
    let p = g.payment_decision(Seat::P0).unwrap();
    g.finish_cast(Seat::P0, p.id).unwrap();
    let d = g.turn_decision().unwrap();
    g.order_triggers_quantum(
        Seat::P0,
        d.id,
        &(0..count).collect::<Vec<_>>(),
        NonZeroUsize::MAX,
    )
    .unwrap();
    let point = |g: &Game, label: &str| {
        json!({"checkpoint":label, "life":g.life(), "stack":g.turns.stack.len(),
        "hand":keys(g, Zone::Hand(Seat::P0)), "graveyard":keys(g, Zone::Graveyard(Seat::P0)), "library":keys(g, Zone::Library(Seat::P0)), "mana":g.mana()[0].iter().sum::<u32>(),
        "tapped_mountains":g.objects.in_zone(Zone::Battlefield).filter(|h| {let o=g.objects.get(*h).unwrap(); o.card.identity().key=="mountain" && o.tapped}).count()})
    };
    let mut points = vec![point(&g, "committed")];
    for i in 0..count {
        pair(&mut g);
        points.push(point(&g, &format!("trigger-{}", i + 1)));
    }
    pair(&mut g);
    points.push(point(&g, "draws"));
    Value::from(points)
}
fn haste_mana(spec: &Value) -> Value {
    let mut g = ready();
    let cavalry = add(&mut g, "axgard-cavalry", Seat::P0, Zone::Battlefield);
    let key = spec["card"].as_str().unwrap();
    // Actually cast the new creature; its sickness is not merely injected.
    let spell = add(&mut g, key, Seat::P0, Zone::Hand(Seat::P0));
    let cost = if key == "llanowar-elves" { 1 } else { 2 };
    g.turns.mana[0][4] = cost;
    let d = g.turn_decision().unwrap();
    g.begin_cast(Seat::P0, d.id, spell).unwrap();
    for _ in 0..cost {
        let p = g.payment_decision(Seat::P0).unwrap();
        g.choose_payment(Seat::P0, p.id, mana::Color::Green)
            .unwrap();
    }
    let p = g.payment_decision(Seat::P0).unwrap();
    g.finish_cast(Seat::P0, p.id).unwrap();
    pair(&mut g);
    let creature = g
        .objects
        .in_zone(Zone::Battlefield)
        .find(|h| g.objects.get(*h).unwrap().card.identity().key == key)
        .unwrap();
    assert!(g.summoning_sick(creature));
    let payment = spec["payment"].as_bool().unwrap();
    let enemy = payment.then(|| add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield));
    let point = |g: &Game, label: &str| {
        json!({"checkpoint":label, "green":g.mana()[0][4], "stack":g.turns.stack.len(),
        "cavalry_tapped":g.objects.get(cavalry).unwrap().tapped, "creature_tapped":g.objects.get(creature).unwrap().tapped,
        "haste":g.has_haste(creature), "enemy_damage":enemy.map_or(0, |h| g.creature_state(h).unwrap().damage)})
    };
    let mut points = vec![point(&g, "fresh")];
    let before = g.snapshot();
    let d = g.turn_decision().unwrap();
    assert!(g.tap_mana(Seat::P0, d.id, creature).is_err());
    assert_eq!(g.snapshot(), before);
    let cavalry_row = g
        .objects
        .in_zone(Zone::Battlefield)
        .position(|h| h == cavalry)
        .unwrap();
    let creature_row = g
        .objects
        .in_zone(Zone::Battlefield)
        .position(|h| h == creature)
        .unwrap();
    choice(
        &mut g,
        json!({"kind":"activate","card":{"zone":"battlefield","row":cavalry_row}}),
    );
    choice(
        &mut g,
        json!({"kind":"target","card":{"zone":"battlefield","row":creature_row}}),
    );
    choice(&mut g, json!({"kind":"finish_activation"}));
    points.push(point(&g, "haste-pending"));
    pair(&mut g);
    points.push(point(&g, "haste-resolved"));
    if payment {
        // A floated G plus Druid's G during payment pays Bite's 1G. No
        // opponent priority/stack entry exists inside this staged payment.
        g.turns.mana[0][4] = 1;
        let bite = add(&mut g, "bite-down", Seat::P0, Zone::Hand(Seat::P0));
        let d = g.turn_decision().unwrap();
        let t = g.begin_targeted_cast(Seat::P0, d.id, bite, 256).unwrap();
        let t = g.choose_target(Seat::P0, t.id, creature).unwrap();
        let t = g.choose_target(Seat::P0, t.id, enemy.unwrap()).unwrap();
        let p = g.finish_targets(Seat::P0, t.id).unwrap();
        g.cast_tap_mana(Seat::P0, p.id, creature).unwrap();
        assert!(g.turn_decision().is_none());
        assert!(g.turns.stack.is_empty());
        for _ in 0..2 {
            let p = g.payment_decision(Seat::P0).unwrap();
            g.choose_payment(Seat::P0, p.id, mana::Color::Green)
                .unwrap();
        }
        let p = g.payment_decision(Seat::P0).unwrap();
        g.finish_cast(Seat::P0, p.id).unwrap();
        points.push(point(&g, "bite-paid"));
        pair(&mut g);
        points.push(point(&g, "bite-resolved"));
    } else {
        let d = g.turn_decision().unwrap();
        g.tap_mana(Seat::P0, d.id, creature).unwrap();
        points.push(point(&g, "mana-produced"));
    }
    Value::from(points)
}
fn land(spec: &Value) -> Value {
    let mut g = ready();
    let first = add(
        &mut g,
        spec["first"].as_str().unwrap(),
        Seat::P0,
        Zone::Hand(Seat::P0),
    );
    let second = add(&mut g, "mountain", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    g.play_land(Seat::P0, d.id, first).unwrap();
    let first = g.objects.in_zone(Zone::Battlefield).next().unwrap();
    let d = g.turn_decision().unwrap();
    g.tap_mana(Seat::P0, d.id, first).unwrap();
    let point = |g: &Game, label: &str| {
        json!({"checkpoint":label,
        "battlefield":keys(g, Zone::Battlefield), "hand":keys(g, Zone::Hand(Seat::P0)),
        "stack":g.turns.stack.len(), "red":g.mana()[0][3], "green":g.mana()[0][4],
        "tapped":g.objects.get(first).unwrap().tapped})
    };
    let before = g.snapshot();
    let a = point(&g, "first-land");
    let d = g.turn_decision().unwrap();
    assert!(g.play_land(Seat::P0, d.id, second).is_err());
    assert_eq!(g.snapshot(), before);
    json!([a, point(&g, "second-rejected")])
}

#[test]
fn cost_compositions_match_independent_literal_checkpoints() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/m2-cost-composition.json"
    ))
    .unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/m2-cost-composition-expectations.json"
    ))
    .unwrap();
    let mut result = serde_json::Map::new();
    for spec in fixture["cases"].as_array().unwrap() {
        let id = spec["id"].as_str().unwrap();
        let actual = match spec["kind"].as_str().unwrap() {
            "haste-mana" => haste_mana(spec),
            "land" => land(spec),
            "archer-thrill" => thrill(spec),
            other => panic!("unsupported composition {other}"),
        };
        assert_eq!(actual, expected[id], "{id}");
        result.insert(id.to_owned(), actual);
    }
    assert_eq!(
        Value::Object(result.clone()),
        expected,
        "exact composition inventory"
    );
    if let Ok(path) = std::env::var("MTG_M2_COST_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    }
}
