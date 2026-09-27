//! Original expectations: GH-113 durable identity contract; CR 103, 117,
//! 305, 601, 608, 400.7, 508–510, 514; frozen Bear Cub and Giant Growth text.
use super::*;
use serde_json::{Value, json};
const CAP: usize = 256;
fn config() -> Config {
    // Literal top seven: Cub, Cub, Growth, Forest x4. Complete frozen green deck.
    let order = [
        ("bear-cub", 2),
        ("giant-growth", 1),
        ("forest", 16),
        ("bear-cub", 2),
        ("giant-growth", 2),
        ("llanowar-elves", 3),
        ("druid-of-the-cowl", 2),
        ("magnigoth-sentry", 2),
        ("tajuru-pathwarden", 2),
        ("thornweald-archer", 3),
        ("bite-down", 3),
        ("wildheart-invoker", 2),
    ]
    .into_iter()
    .flat_map(|(k, n)| vec![k.to_owned(); n])
    .collect::<Vec<_>>();
    assert_eq!(order.len(), 40);
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
fn game() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&config(), 113, 0).unwrap();
    g
}
fn object(s: Seat, birth: u64, key: &str, zone: Zone, incarnation: u64) -> Value {
    json!({"birth":birth,"card":key,"owner":s,"zone":zone,"incarnation":incarnation})
}
fn hand(s: Seat, n: u64, key: &str) -> Value {
    object(
        s,
        n + if s == Seat::P0 { 0 } else { 40 },
        key,
        Zone::Hand(s),
        1,
    )
}
fn permanent(s: Seat, n: u64, key: &str) -> Value {
    object(
        s,
        n + if s == Seat::P0 { 0 } else { 40 },
        key,
        Zone::Battlefield,
        if key == "forest" { 2 } else { 3 },
    )
}
fn record(s: Seat, kind: &str, choices: Value) -> Value {
    json!({"version":1,"actor":s,"decision":kind,"choices":choices})
}
fn send(g: &mut Game, s: Seat, kind: &str, choices: Value) {
    let bytes = serde_json::to_vec(&record(s, kind, choices)).unwrap();
    let before = format!("{g:?}");
    let Decoded::Decision { actor, submission } = decode(g, &bytes, CAP).unwrap_or_else(|e| {
        panic!(
            "{e:?}: {} at {:?}",
            String::from_utf8_lossy(&bytes),
            g.turn_position()
        )
    }) else {
        panic!("decision required")
    };
    assert_eq!(format!("{g:?}"), before, "decoding is read-only");
    let encoded = encode(g, actor, &submission, CAP).unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&encoded).unwrap(),
        serde_json::from_slice::<Value>(&bytes).unwrap()
    );
    apply(g, &bytes, CAP).unwrap();
}
fn one(g: &mut Game, s: Seat, kind: &str, choice: Value) {
    send(g, s, kind, json!([choice]));
}
fn keep(g: &mut Game) {
    for s in [Seat::P0, Seat::P1] {
        one(g, s, "keep_or_mulligan", json!({"kind":"keep"}));
    }
    g.start_turns().unwrap();
}
fn pass(g: &mut Game) {
    let s = g.turn_decision().unwrap().actor;
    one(g, s, "priority", json!({"kind":"pass"}));
}
fn pair(g: &mut Game) {
    pass(g);
    pass(g);
}
fn idle(g: &mut Game) {
    let s = g.turn_decision().unwrap().actor;
    let d = g.policy_observe(s, CAP).unwrap().decision.unwrap();
    match d.kind {
        "priority" => pass(g),
        "attackers" | "blockers" | "combat_damage" => {
            one(g, s, d.kind, json!({"kind":"finish_combat"}))
        }
        "cleanup_discard" => {
            // This helper is only used before a cleanup obligation in the main script.
            panic!("unexpected cleanup obligation");
        }
        _ => panic!("unexpected {}", d.kind),
    }
}
fn until(g: &mut Game, turn: u64, step: turns::Step) {
    for _ in 0..300 {
        if g.turn_position().unwrap().0 == turn && g.turn_position().unwrap().2 == step {
            return;
        }
        idle(g);
    }
    panic!("bounded script");
}
fn cast(g: &mut Game, s: Seat, n: u64) {
    one(
        g,
        s,
        "priority",
        json!({"kind":"cast","card":hand(s,n,"bear-cub")}),
    );
    for land in [3, 4] {
        one(
            g,
            s,
            "payment",
            json!({"kind":"tap_mana","card":permanent(s,land,"forest")}),
        );
    }
    for _ in 0..2 {
        one(g, s, "payment", json!({"kind":"pay","color":4}));
    }
    one(g, s, "payment", json!({"kind":"finish_payment"}));
    pair(g);
}
#[test]
fn actions_hand_authored_normal_reset_spell_combat() {
    let mut g = game();
    keep(&mut g);
    for turn in 1..=4 {
        let s = if turn % 2 == 1 { Seat::P0 } else { Seat::P1 };
        until(&mut g, turn, turns::Step::PrecombatMain);
        let n = if turn <= 2 { 3 } else { 4 };
        one(
            &mut g,
            s,
            "priority",
            json!({"kind":"play_land","card":hand(s,n,"forest")}),
        );
        if turn >= 3 {
            cast(&mut g, s, 0);
        }
    }
    until(&mut g, 6, turns::Step::PrecombatMain);
    cast(&mut g, Seat::P1, 1);
    until(&mut g, 7, turns::Step::PrecombatMain);
    // Growth gives the original P0 Cub 5/5. Same-name copy 1 remains in hand.
    one(
        &mut g,
        Seat::P0,
        "priority",
        json!({"kind":"cast","card":hand(Seat::P0,2,"giant-growth")}),
    );
    one(
        &mut g,
        Seat::P0,
        "growth_target",
        json!({"kind":"target","card":permanent(Seat::P0,0,"bear-cub")}),
    );
    one(
        &mut g,
        Seat::P0,
        "targets_complete",
        json!({"kind":"finish_targets"}),
    );
    one(
        &mut g,
        Seat::P0,
        "payment",
        json!({"kind":"tap_mana","card":permanent(Seat::P0,3,"forest")}),
    );
    one(&mut g, Seat::P0, "payment", json!({"kind":"pay","color":4}));
    one(
        &mut g,
        Seat::P0,
        "payment",
        json!({"kind":"finish_payment"}),
    );
    pair(&mut g);
    let cub = g
        .objects
        .in_zone(Zone::Battlefield)
        .find(|h| {
            g.objects.get(*h).unwrap().card.identity().key == "bear-cub"
                && g.objects.get(*h).unwrap().owner == Seat::P0
        })
        .unwrap();
    assert_eq!(g.creature_state(cub).unwrap().power, 5);
    until(&mut g, 7, turns::Step::DeclareAttackers);
    one(
        &mut g,
        Seat::P0,
        "attackers",
        json!({"kind":"select_attackers","cards":[permanent(Seat::P0,0,"bear-cub")]}),
    );
    one(
        &mut g,
        Seat::P0,
        "attackers",
        json!({"kind":"finish_combat"}),
    );
    pair(&mut g);
    one(
        &mut g,
        Seat::P1,
        "blockers",
        json!({"kind":"select_blockers","blocks":[[permanent(Seat::P1,0,"bear-cub"),permanent(Seat::P0,0,"bear-cub")],[permanent(Seat::P1,1,"bear-cub"),permanent(Seat::P0,0,"bear-cub")]]}),
    );
    one(
        &mut g,
        Seat::P1,
        "blockers",
        json!({"kind":"finish_combat"}),
    );
    pair(&mut g);
    one(
        &mut g,
        Seat::P0,
        "combat_damage",
        json!({"kind":"assign_damage","attacker":permanent(Seat::P0,0,"bear-cub"),"amounts":[[permanent(Seat::P1,0,"bear-cub"),3],[permanent(Seat::P1,1,"bear-cub"),2]]}),
    );
    one(
        &mut g,
        Seat::P0,
        "combat_damage",
        json!({"kind":"finish_combat"}),
    );
    assert_eq!(g.life(), [20, 20]);
    assert_eq!(g.objects.in_zone(Zone::Graveyard(Seat::P1)).count(), 2);
    assert_eq!(g.creature_state(cub).unwrap().damage, 4);
    until(&mut g, 8, turns::Step::Upkeep);
    assert_eq!(g.creature_state(cub).unwrap().power, 2);
    assert_eq!(g.creature_state(cub).unwrap().damage, 0);
}
#[test]
fn actions_fresh_processes() {
    for _ in 0..2 {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "opening::actions::tests::actions_hand_authored_normal_reset_spell_combat",
                "--nocapture",
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while child.try_wait().unwrap().is_none() {
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("fresh-process script exceeded 30 seconds");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success()
                && String::from_utf8_lossy(&out.stdout).contains("1 passed; 0 failed"),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
}
#[test]
fn actions_rejections_do_not_mutate() {
    let mut g = game();
    keep(&mut g);
    until(&mut g, 1, turns::Step::PrecombatMain);
    let valid = record(
        Seat::P0,
        "priority",
        json!([{"kind":"play_land","card":hand(Seat::P0,3,"forest")}]),
    );
    let mut invalid = vec![];
    for (path, v) in [
        ("/version", json!(2)),
        ("/actor", json!("P1")),
        ("/decision", json!("payment")),
        ("/choices/0/card/incarnation", json!(0)),
        ("/choices/0/card/card", json!("bear-cub")),
        ("/choices/0/card/owner", json!("P1")),
        ("/choices/0/card/zone", json!("Battlefield")),
        ("/choices/0/card/birth", json!(99)),
    ] {
        let mut r = valid.clone();
        *r.pointer_mut(path).unwrap() = v;
        invalid.push(r);
    }
    for field in ["birth", "card", "owner", "zone", "incarnation"] {
        let mut r = valid.clone();
        r["choices"][0]["card"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        invalid.push(r);
    }
    let mut r = valid.clone();
    r["choices"][0]["card"]["row"] = json!(0);
    invalid.push(r);
    for r in invalid {
        let before = format!("{g:?}");
        assert!(
            apply(&mut g, &serde_json::to_vec(&r).unwrap(), CAP).is_err(),
            "{r}"
        );
        assert_eq!(format!("{g:?}"), before);
    }
    apply(&mut g, &serde_json::to_vec(&valid).unwrap(), CAP).unwrap();
    let before = format!("{g:?}");
    assert!(apply(&mut g, &serde_json::to_vec(&valid).unwrap(), CAP).is_err());
    assert_eq!(format!("{g:?}"), before);
}

fn rejected(g: &mut Game, r: Value) {
    let before = format!("{g:?}");
    assert!(
        decode(g, &serde_json::to_vec(&r).unwrap(), CAP).is_err(),
        "{r}"
    );
    assert!(
        apply(g, &serde_json::to_vec(&r).unwrap(), CAP).is_err(),
        "{r}"
    );
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn actions_duplicate_copy_zone_reentry_and_storage_reuse() {
    // Explicit synthetic edge: two identical untapped lands, then a removed
    // object's slot reused. Birth identity must not be a slot or current row.
    let mut g = game();
    keep(&mut g);
    until(&mut g, 1, turns::Step::PrecombatMain);
    for _ in 0..2 {
        g.objects
            .allocate(
                CardId::from_key("forest").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
    }
    let first = object(Seat::P0, 80, "forest", Zone::Battlefield, 0);
    let second = object(Seat::P0, 81, "forest", Zone::Battlefield, 0);
    one(
        &mut g,
        Seat::P0,
        "priority",
        json!({"kind":"tap_mana","card":second}),
    );
    let hs = g.objects.in_zone(Zone::Battlefield).collect::<Vec<_>>();
    assert!(!g.objects.get(hs[0]).unwrap().tapped);
    assert!(g.objects.get(hs[1]).unwrap().tapped);
    let moved = g.objects.move_to(hs[0], Zone::Exile).unwrap();
    g.objects.move_to(moved, Zone::Battlefield).unwrap();
    rejected(
        &mut g,
        record(
            Seat::P0,
            "priority",
            json!([{"kind":"tap_mana","card":first}]),
        ),
    );
    let reentered = object(Seat::P0, 80, "forest", Zone::Battlefield, 2);
    one(
        &mut g,
        Seat::P0,
        "priority",
        json!({"kind":"tap_mana","card":reentered}),
    );
    g.objects.remove(hs[1]).unwrap();
    g.objects
        .allocate(
            CardId::from_key("forest").unwrap(),
            Seat::P0,
            Zone::Battlefield,
        )
        .unwrap();
    rejected(
        &mut g,
        record(
            Seat::P0,
            "priority",
            json!([{"kind":"tap_mana","card":second}]),
        ),
    );
    one(
        &mut g,
        Seat::P0,
        "priority",
        json!({"kind":"tap_mana","card":object(Seat::P0,82,"forest",Zone::Battlefield,0)}),
    );
}
#[test]
fn actions_cleanup_duplicate_selection_and_snapshot_scope() {
    let mut g = game();
    keep(&mut g);
    until(&mut g, 2, turns::Step::Cleanup);
    assert_eq!(g.discard_cards().unwrap().len(), 8);
    let r = record(
        Seat::P1,
        "cleanup_discard",
        json!([{"kind":"discard","card":hand(Seat::P1,0,"bear-cub")}]),
    );
    let mut restored = Game::new().unwrap();
    restored.restore(&g.snapshot()).unwrap();
    let bytes = serde_json::to_vec(&r).unwrap();
    apply(&mut restored, &bytes, CAP).unwrap();
    apply(&mut g, &bytes, CAP).unwrap();
    for game in [&g, &restored] {
        let dead = game
            .objects
            .in_zone(Zone::Graveyard(Seat::P1))
            .next()
            .unwrap();
        assert_eq!(game.objects.semantic_identity(dead), Ok((40, 2)));
        assert_eq!(game.objects.in_zone(Zone::Hand(Seat::P1)).count(), 7);
    }
    // Explicit nine-card synthetic cleanup exercises a two-card selection.
    let mut g = game();
    keep(&mut g);
    g.draw_top(Seat::P1).unwrap();
    until(&mut g, 2, turns::Step::Cleanup);
    let c = json!({"kind":"discard","card":hand(Seat::P1,0,"bear-cub")});
    rejected(&mut g, record(Seat::P1, "cleanup_discard", json!([c, c])));
    send(
        &mut g,
        Seat::P1,
        "cleanup_discard",
        json!([{"kind":"discard","card":hand(Seat::P1,1,"bear-cub")},{"kind":"discard","card":hand(Seat::P1,0,"bear-cub")}]),
    );
    assert_eq!(g.objects.in_zone(Zone::Graveyard(Seat::P1)).count(), 2);
}
#[test]
fn actions_opening_mulligan_bottom_and_strict_schema() {
    let mut g = game();
    one(
        &mut g,
        Seat::P0,
        "keep_or_mulligan",
        json!({"kind":"mulligan"}),
    );
    one(&mut g, Seat::P1, "keep_or_mulligan", json!({"kind":"keep"}));
    // Exercise general encoding after a real shuffled mulligan, independent
    // expectation is the selected actual card goes to the library bottom.
    let d = g.policy_observe(Seat::P0, CAP).unwrap().decision.unwrap();
    assert_eq!(d.kind, "bottom");
    let h = g.view_hand(Seat::P0)[0];
    let id = g.objects.semantic_identity(h).unwrap();
    let sub = Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation,
        choices: vec![policy::Choice::Bottom {
            card: policy::VisibleRef {
                zone: policy::VisibleZone::Hand,
                row: 0,
            },
        }],
    };
    let bytes = encode(&g, Seat::P0, &sub, CAP).unwrap();
    for field in ["version", "actor", "decision", "choices"] {
        let mut r: Value = serde_json::from_slice(&bytes).unwrap();
        r.as_object_mut().unwrap().remove(field);
        rejected(&mut g, r);
    }
    let mut r: Value = serde_json::from_slice(&bytes).unwrap();
    r["choices"][0]["index"] = json!(0);
    rejected(&mut g, r);
    apply(&mut g, &bytes, CAP).unwrap();
    let last = g.objects.in_zone(Zone::Library(Seat::P0)).last().unwrap();
    assert_eq!(g.objects.semantic_identity(last), Ok((id.0, id.1 + 1)));
}
#[test]
fn actions_targets_payment_cancellation_and_every_color() {
    // Synthetic setup is deliberately isolated from the reachable combat script.
    let mut g = game();
    keep(&mut g);
    until(&mut g, 1, turns::Step::PrecombatMain);
    for s in [Seat::P0, Seat::P1] {
        g.objects
            .allocate(CardId::from_key("bear-cub").unwrap(), s, Zone::Battlefield)
            .unwrap();
    }
    one(
        &mut g,
        Seat::P0,
        "priority",
        json!({"kind":"play_land","card":hand(Seat::P0,3,"forest")}),
    );
    for cancel in [true, false] {
        one(
            &mut g,
            Seat::P0,
            "priority",
            json!({"kind":"cast","card":hand(Seat::P0,2,"giant-growth")}),
        );
        if cancel {
            one(
                &mut g,
                Seat::P0,
                "growth_target",
                json!({"kind":"cancel_targets"}),
            );
            continue;
        }
        one(
            &mut g,
            Seat::P0,
            "growth_target",
            json!({"kind":"target","card":object(Seat::P0,80,"bear-cub",Zone::Battlefield,0)}),
        );
        one(
            &mut g,
            Seat::P0,
            "targets_complete",
            json!({"kind":"finish_targets"}),
        );
        one(
            &mut g,
            Seat::P0,
            "payment",
            json!({"kind":"cancel_payment"}),
        );
    }
    for color in 0..6 {
        g.turns.mana[0] = [1; 6];
        g.begin_payment(
            Seat::P0,
            g.turn_decision().unwrap().id,
            mana::ManaCost {
                generic: 1,
                colored: [0; 6],
            },
        )
        .unwrap();
        one(
            &mut g,
            Seat::P0,
            "payment",
            json!({"kind":"pay","color":color}),
        );
        one(
            &mut g,
            Seat::P0,
            "payment",
            json!({"kind":"finish_payment"}),
        );
        assert_eq!(g.turns.mana[0][color], 0);
    }
}
#[test]
fn actions_concession_is_not_a_priority_choice() {
    for s in [Seat::P0, Seat::P1] {
        let mut g = game();
        keep(&mut g);
        let bytes =
            serde_json::to_vec(&record(s, "concession", json!([{"kind":"concede"}]))).unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&encode_concession(&g, s).unwrap()).unwrap(),
            serde_json::from_slice::<Value>(&bytes).unwrap()
        );
        apply(&mut g, &bytes, CAP).unwrap();
        assert_eq!(
            g.outcome().unwrap().winner,
            Some(if s == Seat::P0 { Seat::P1 } else { Seat::P0 })
        );
        let before = format!("{g:?}");
        assert!(apply(&mut g, &bytes, CAP).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
}

#[test]
fn actions_bite_ordered_targets_and_encoder_validation() {
    let mut g = game();
    keep(&mut g);
    until(&mut g, 1, turns::Step::PrecombatMain);
    // Explicit synthetic position with sufficient mana, testing records independently
    // of the future envelope. Bite Down's first target is a creature you control.
    for s in [Seat::P0, Seat::P1] {
        g.objects
            .allocate(CardId::from_key("bear-cub").unwrap(), s, Zone::Battlefield)
            .unwrap();
    }
    g.objects
        .allocate(
            CardId::from_key("bite-down").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    g.turns.mana[0] = [0, 0, 0, 0, 2, 0];
    let d = g.policy_observe(Seat::P0, CAP).unwrap().decision.unwrap();
    let stale = Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation + 1,
        choices: vec![policy::Choice::Pass],
    };
    let before = format!("{g:?}");
    assert_eq!(
        encode(&g, Seat::P0, &stale, CAP),
        Err(ActionError::Policy(PolicyError::StaleDecision))
    );
    assert_eq!(format!("{g:?}"), before);
    one(
        &mut g,
        Seat::P0,
        "priority",
        json!({"kind":"cast","card":object(Seat::P0,82,"bite-down",Zone::Hand(Seat::P0),0)}),
    );
    rejected(
        &mut g,
        record(
            Seat::P0,
            "bite_source",
            json!([{"kind":"target","card":object(Seat::P1,81,"bear-cub",Zone::Battlefield,0)}]),
        ),
    );
    one(
        &mut g,
        Seat::P0,
        "bite_source",
        json!({"kind":"target","card":object(Seat::P0,80,"bear-cub",Zone::Battlefield,0)}),
    );
    one(
        &mut g,
        Seat::P0,
        "bite_destination",
        json!({"kind":"target","card":object(Seat::P1,81,"bear-cub",Zone::Battlefield,0)}),
    );
    one(
        &mut g,
        Seat::P0,
        "targets_complete",
        json!({"kind":"finish_targets"}),
    );
    for _ in 0..2 {
        one(&mut g, Seat::P0, "payment", json!({"kind":"pay","color":4}));
    }
    one(
        &mut g,
        Seat::P0,
        "payment",
        json!({"kind":"finish_payment"}),
    );
    pair(&mut g);
    assert_eq!(g.objects.in_zone(Zone::Graveyard(Seat::P1)).count(), 1);
    assert_eq!(g.objects.in_zone(Zone::Graveyard(Seat::P0)).count(), 1);
    assert_eq!(g.life(), [20, 20]);
}

#[test]
fn actions_fieldless_variants_reject_extra_indices() {
    let mut g = game();
    for k in ["keep", "mulligan"] {
        rejected(
            &mut g,
            record(Seat::P0, "keep_or_mulligan", json!([{"kind":k,"index":0}])),
        );
    }
    rejected(
        &mut g,
        record(
            Seat::P1,
            "concession",
            json!([{"kind":"concede","index":0}]),
        ),
    );
}
