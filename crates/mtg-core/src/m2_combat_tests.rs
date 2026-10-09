//! GH-210 composition only. Original synthetic setups; all actions use Game.
//! Independent CR 510/611/613/702/704/514 ledger is checked in, not generated.
use super::*;
use serde_json::{Value, json};
use turns::{Step, TurnAction, TurnSelection};

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
fn pair(g: &mut Game) {
    pass(g);
    pass(g);
}
fn add(g: &mut Game, key: &str, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, zone)
        .unwrap()
}
fn point(g: &Game, target: Handle, blockers: &[Handle]) -> Value {
    let stats = |h| {
        g.creature_state(h)
            .map(|s| [s.power, s.toughness, s.damage])
    };
    let mut result = json!({"target":stats(target),"blockers":blockers.iter().map(|h| stats(*h)).collect::<Vec<_>>(),
        "trample":g.has_trample(target),"life":g.life(),"mana":g.turns.mana[0].iter().sum::<u32>(),"stack":g.turns.stack.len()});
    if let Some(dragon) = g
        .objects
        .in_zone(Zone::Battlefield)
        .find(|h| g.objects.get(*h).unwrap().card.identity().key == "shivan-dragon")
    {
        result["dragon"] = json!(stats(dragon));
        result["flying"] =
            json!(card_definitions::definition(g.objects.get(dragon).unwrap().card).flying());
        result["reach"] =
            json!(card_definitions::definition(g.objects.get(target).unwrap().card).reach());
        result["deathtouch"] = json!(g.has_deathtouch(target));
    }
    result
}
fn growth(g: &mut Game, target: Handle) {
    let spell = add(g, "giant-growth", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    let t = g.begin_targeted_cast(Seat::P0, d.id, spell, 256).unwrap();
    let t = g.choose_target(Seat::P0, t.id, target).unwrap();
    let p = g.finish_targets(Seat::P0, t.id).unwrap();
    let p = g
        .choose_payment(Seat::P0, p.id, mana::Color::Green)
        .unwrap();
    g.finish_cast(Seat::P0, p.id).unwrap();
}
#[test]
fn m2_combat_reference_literal_checkpoints() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/m2-combat.json")).unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/m2-combat-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for case in fixture["cases"].as_array().unwrap() {
        let id = case["id"].as_str().unwrap();
        let mut g = Game::new().unwrap();
        g.reset(&Config::default(), 210, 0).unwrap();
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
        let target = add(
            &mut g,
            case["target"].as_str().unwrap(),
            Seat::P0,
            Zone::Battlefield,
        );
        let source = add(&mut g, "wildheart-invoker", Seat::P0, Zone::Battlefield);
        let dragon = (id == "cleanup-shivan-thorn")
            .then(|| add(&mut g, "shivan-dragon", Seat::P0, Zone::Battlefield));
        if dragon.is_some() {
            g.turns.mana[0][3] = 1;
        }
        let blockers: Vec<_> = (0..case["blockers"].as_u64().unwrap())
            .map(|_| {
                add(
                    &mut g,
                    case["blocker"].as_str().unwrap(),
                    Seat::P1,
                    Zone::Battlefield,
                )
            })
            .collect();
        let boosts = case["boosts"].as_u64().unwrap();
        let growths = case["growths"].as_u64().unwrap();
        g.turns.mana[0][4] = (boosts * 8 + growths) as u32;
        let mut points = serde_json::Map::new();
        points.insert("initial".into(), point(&g, target, &blockers));
        for n in 0..boosts {
            let d = g.turn_decision().unwrap();
            g.begin_activation(d.actor, d.id, source).unwrap();
            let a = g.turns.activation.as_ref().unwrap().id;
            g.choose_activation_target(Seat::P0, a, target).unwrap();
            for _ in 0..8 {
                let a = g.turns.activation.as_ref().unwrap().id;
                g.pay_activation(Seat::P0, a, mana::Color::Green).unwrap();
            }
            let a = g.turns.activation.as_ref().unwrap().id;
            g.finish_activation(Seat::P0, a).unwrap();
            points.insert(
                format!("activation-{}-stack", n + 1),
                point(&g, target, &blockers),
            );
            pair(&mut g);
            points.insert(
                format!("activation-{}-resolved", n + 1),
                point(&g, target, &blockers),
            );
        }
        if let Some(dragon) = dragon {
            let d = g.turn_decision().unwrap();
            g.begin_activation(d.actor, d.id, dragon).unwrap();
            let a = g.turns.activation.as_ref().unwrap().id;
            g.pay_activation(Seat::P0, a, mana::Color::Red).unwrap();
            let a = g.turns.activation.as_ref().unwrap().id;
            g.finish_activation(Seat::P0, a).unwrap();
            points.insert("dragon-stack".into(), point(&g, target, &blockers));
            pair(&mut g);
            points.insert("dragon-resolved".into(), point(&g, target, &blockers));
        }
        // Reserved Growth mana is supplied at its actual priority window: mana
        // empties when main ends, so this fixture models a fresh Forest then.
        if !case["departure"].as_bool().unwrap() {
            for n in 0..growths {
                growth(&mut g, target);
                pair(&mut g);
                points.insert(format!("growth-{}", n + 1), point(&g, target, &blockers));
            }
        }
        if !blockers.is_empty() {
            g.turns.position = Some((1, Seat::P0, Step::BeginningCombat));
            pair(&mut g);
            let d = g.turn_decision().unwrap();
            let d = g.select_attackers(d.actor, d.id, &[target]).unwrap();
            g.finish_combat(d.actor, d.id).unwrap();
            pair(&mut g);
            let d = g.turn_decision().unwrap();
            let d = g
                .select_blockers(
                    d.actor,
                    d.id,
                    &blockers.iter().map(|h| (*h, target)).collect::<Vec<_>>(),
                )
                .unwrap();
            g.finish_combat(d.actor, d.id).unwrap();
            if case["departure"].as_bool().unwrap() {
                g.objects
                    .move_to(blockers[0], Zone::Graveyard(Seat::P1))
                    .unwrap();
                g.turns.mana[0][4] = 1;
                growth(&mut g, target);
                pair(&mut g);
                points.insert("growth-1".into(), point(&g, target, &blockers));
            }
            points.insert("before-damage".into(), point(&g, target, &blockers));
            pair(&mut g);
            let d = g.turn_decision().unwrap();
            if id == "invoker-cubs-reject" {
                let before = g.snapshot();
                assert!(
                    g.assign_combat_damage(
                        d.actor,
                        d.id,
                        target,
                        &[(blockers[0], 0), (blockers[1], 1)]
                    )
                    .is_err()
                );
                assert_eq!(
                    before,
                    g.snapshot(),
                    "illegal trample allocation changes nothing"
                );
            }
            let d = if case["departure"].as_bool().unwrap() {
                d
            } else {
                g.assign_combat_damage(
                    d.actor,
                    d.id,
                    target,
                    &blockers.iter().map(|h| (*h, 1)).collect::<Vec<_>>(),
                )
                .unwrap()
            };
            g.finish_combat(d.actor, d.id).unwrap();
            points.insert("after-damage".into(), point(&g, target, &blockers));
        }
        g.turns.position = Some((1, Seat::P0, Step::End));
        pair(&mut g);
        points.insert("after-cleanup".into(), point(&g, target, &blockers));
        let actual = Value::Object(points);
        assert_eq!(actual, expected[id], "{id}");
        results.insert(id.into(), actual);
    }
    if let Ok(path) = std::env::var("MTG_M2_COMBAT_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}
