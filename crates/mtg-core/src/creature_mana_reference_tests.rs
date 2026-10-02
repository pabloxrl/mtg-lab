//! Original synthetic shared fixture. Literal oracle: CR 302.6, 601, 605,
//! 508–510 and pinned Elf 1/1 / Druid 1/3. No fixture rule implementation.
use super::mana::Color;
use super::turns::{Step, TurnAction, TurnSelection};
use super::*;
use serde_json::{Value, json};
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
fn finish(g: &mut Game) {
    let d = g.payment_decision(Seat::P0).unwrap();
    g.finish_cast(Seat::P0, d.id).unwrap();
}
fn pay(g: &mut Game) {
    let d = g.payment_decision(Seat::P0).unwrap();
    g.choose_payment(Seat::P0, d.id, Color::Green).unwrap();
}
fn add(g: &mut Game, key: &str, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, zone)
        .unwrap()
}
#[test]
fn creature_mana_reference_literal_checkpoints() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/creature-mana.json"
    ))
    .unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/creature-mana-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for case in fixture["cases"].as_array().unwrap() {
        let key = case["card"].as_str().unwrap();
        let mode = case["mode"].as_str().unwrap();
        let mut g = Game::new().unwrap();
        g.reset(&Config::default(), 195, 0).unwrap();
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
        let mut source = add(
            &mut g,
            key,
            if mode == "block" { Seat::P1 } else { Seat::P0 },
            Zone::Battlefield,
        );
        let mut cub = None;
        match mode {
            "priority" => {
                let d = g.turn_decision().unwrap();
                g.tap_mana(Seat::P0, d.id, source).unwrap();
                let before = g.snapshot();
                let d = g.turn_decision().unwrap();
                assert!(g.tap_mana(Seat::P0, d.id, source).is_err());
                assert_eq!(g.snapshot(), before);
            }
            "sick" => {
                source = g.objects.move_to(source, Zone::Hand(Seat::P0)).unwrap();
                g.turns.mana[0][4] = if key == "llanowar-elves" { 1 } else { 2 };
                let d = g.turn_decision().unwrap();
                g.begin_cast(Seat::P0, d.id, source).unwrap();
                pay(&mut g);
                if key == "druid-of-the-cowl" {
                    pay(&mut g);
                }
                finish(&mut g);
                pair(&mut g);
                source = g
                    .objects
                    .in_zone(Zone::Battlefield)
                    .find(|h| g.objects.get(*h).unwrap().card.identity().key == key)
                    .unwrap();
                assert!(g.summoning_sick(source));
                let before = g.snapshot();
                let d = g.turn_decision().unwrap();
                assert!(g.tap_mana(Seat::P0, d.id, source).is_err());
                assert_eq!(g.snapshot(), before);
            }
            "payment" | "floating" => {
                let forest = add(&mut g, "forest", Seat::P0, Zone::Battlefield);
                if mode == "floating" {
                    let d = g.turn_decision().unwrap();
                    g.tap_mana(Seat::P0, d.id, forest).unwrap();
                }
                let spell = add(&mut g, "bear-cub", Seat::P0, Zone::Hand(Seat::P0));
                let d = g.turn_decision().unwrap();
                g.begin_cast(Seat::P0, d.id, spell).unwrap();
                let d = g.payment_decision(Seat::P0).unwrap();
                g.cast_tap_mana(Seat::P0, d.id, source).unwrap();
                if mode == "payment" {
                    let d = g.payment_decision(Seat::P0).unwrap();
                    g.cast_tap_mana(Seat::P0, d.id, forest).unwrap();
                }
                assert!(g.turn_decision().is_none());
                pay(&mut g);
                pay(&mut g);
                finish(&mut g);
                assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
                assert_eq!(g.objects.in_zone(Zone::Stack).count(), 1);
                pair(&mut g);
                cub = g
                    .objects
                    .in_zone(Zone::Battlefield)
                    .find(|h| g.objects.get(*h).unwrap().card.identity().key == "bear-cub");
            }
            "opponent" => {
                pass(&mut g);
                let d = g.turn_decision().unwrap();
                let before = g.snapshot();
                assert!(g.tap_mana(Seat::P0, d.id, source).is_err());
                assert_eq!(g.snapshot(), before);
            }
            "attacked" | "block" => {
                g.turns.position = Some((1, Seat::P0, Step::BeginningCombat));
                let attacker = if mode == "block" {
                    let h = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
                    cub = Some(h);
                    h
                } else {
                    source
                };
                pair(&mut g);
                let d = g.turn_decision().unwrap();
                let d = g.select_attackers(Seat::P0, d.id, &[attacker]).unwrap();
                g.finish_combat(Seat::P0, d.id).unwrap();
                pair(&mut g);
                let d = g.turn_decision().unwrap();
                let blocks = if mode == "block" {
                    vec![(source, attacker)]
                } else {
                    vec![]
                };
                let d = g.select_blockers(Seat::P1, d.id, &blocks).unwrap();
                g.finish_combat(Seat::P1, d.id).unwrap();
                pair(&mut g);
                let d = g.turn_decision().unwrap();
                g.finish_combat(d.actor, d.id).unwrap();
                if mode == "attacked" {
                    for _ in 0..4 {
                        pass(&mut g);
                    }
                    assert_eq!(g.turn_position().unwrap().2, Step::PostcombatMain);
                    let d = g.turn_decision().unwrap();
                    let before = g.snapshot();
                    assert!(g.tap_mana(Seat::P0, d.id, source).is_err());
                    assert_eq!(g.snapshot(), before);
                }
            }
            _ => panic!("unsupported case"),
        }
        let o = g.objects.get(source).unwrap();
        let c = g.creature_state(source).unwrap();
        let result = json!({"power":c.power,"toughness":c.toughness,"damage":c.damage,"tapped":o.tapped,"green":g.mana()[0][4],"stack":g.objects.in_zone(Zone::Stack).count(),"priority":if g.turn_decision().unwrap().actor==Seat::P0{0}else{1},"cub":cub.is_some(),"cub_damage":if mode=="block"{cub.map(|h|g.creature_state(h).unwrap().damage)}else{None},"life":g.life()});
        assert_eq!(
            result,
            expected[case["id"].as_str().unwrap()],
            "{}",
            case["id"]
        );
        results.insert(case["id"].as_str().unwrap().into(), result);
    }
    if let Ok(path) = std::env::var("MTG_CREATURE_MANA_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}
