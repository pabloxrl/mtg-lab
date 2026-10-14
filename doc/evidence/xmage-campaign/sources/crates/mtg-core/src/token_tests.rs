//! Synthetic rule probes; normal-reset Driver coverage is separate.
//! Pinned Dragon Fodder (1R, two red 1/1 Goblins); CR 111, 302.6, 307,
//! 400.7, 601, 608, 704.5d/g. Literal outcomes are independent of engine output.
use super::*;
use mana::Color;
use turns::{Step, TurnAction, TurnSelection};
fn add(g: &mut Game, key: &str, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, zone)
        .unwrap()
}
fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 194, 0).unwrap();
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
    g.turns.position = Some((1, Seat::P0, Step::PrecombatMain));
    g.turns.mana = [[0, 0, 0, 8, 8, 0]; 2];
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
fn pair(g: &mut Game) {
    pass(g);
    pass(g);
}
fn fodder(g: &mut Game) {
    let h = add(g, "dragon-fodder", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    assert!(
        g.cast_candidates(Seat::P0).contains(&h),
        "CR 307/601: payable Fodder offered in own main"
    );
    let p = g.begin_cast(Seat::P0, d.id, h).unwrap();
    let p = g.choose_payment(Seat::P0, p.id, Color::Red).unwrap();
    let p = g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
    g.finish_cast(Seat::P0, p.id).unwrap();
}
fn tokens(g: &Game) -> Vec<Handle> {
    g.objects
        .in_zone(Zone::Battlefield)
        .filter(|h| g.objects.get(*h).unwrap().card.identity().key == "goblin-token")
        .collect()
}
fn instant(g: &mut Game, key: &str, hs: &[Handle]) {
    let d = g.turn_decision().unwrap();
    let h = add(g, key, d.actor, Zone::Hand(d.actor));
    let mut t = g.begin_targeted_cast(d.actor, d.id, h, 128).unwrap();
    for &h in hs {
        t = g.choose_target(d.actor, t.id, h).unwrap();
    }
    let p = g.finish_targets(d.actor, t.id).unwrap();
    let mut p = g.choose_payment(d.actor, p.id, Color::Green).unwrap();
    if key == "bite-down" {
        p = g.choose_payment(d.actor, p.id, Color::Green).unwrap();
    }
    g.finish_cast(d.actor, p.id).unwrap();
}
#[test]
fn tokens_fodder_waits_for_resolution_then_creates_four_distinct_goblins() {
    let mut g = ready();
    for count in [0, 2] {
        fodder(&mut g);
        assert_eq!(tokens(&g).len(), count);
        pass(&mut g);
        assert_eq!(tokens(&g).len(), count);
        pass(&mut g);
        assert_eq!(tokens(&g).len(), count + 2);
    }
    let hs = tokens(&g);
    for (i, &h) in hs.iter().enumerate() {
        assert!(!hs[..i].contains(&h));
        let o = g.objects.get(h).unwrap();
        assert_eq!(
            (o.owner, o.controller, o.tapped),
            (Seat::P0, Seat::P0, false)
        );
        assert_eq!(
            g.creature_state(h),
            Some(targets::CreatureState {
                power: 1,
                toughness: 1,
                damage: 0
            })
        );
        assert!(g.summoning_sick(h));
    }
    assert_eq!(g.objects.in_zone(Zone::Graveyard(Seat::P0)).count(), 2);
    assert_eq!(g.life(), [20, 20]);
    g.turns.position = Some((1, Seat::P0, Step::BeginningCombat));
    pair(&mut g);
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert!(g.select_attackers(d.actor, d.id, &[hs[0]]).is_err());
    assert_eq!(g.snapshot(), before);
    assert!(
        g.combat_decision(Seat::P0, 128)
            .unwrap()
            .attackers
            .is_empty()
    );
}
#[test]
fn tokens_sorcery_timing_and_unpaid_rejections_preserve_state() {
    let mut g = ready();
    let h = add(&mut g, "dragon-fodder", Seat::P0, Zone::Hand(Seat::P0));
    for pos in [
        (1, Seat::P1, Step::PrecombatMain),
        (1, Seat::P0, Step::BeginningCombat),
    ] {
        g.turns.position = Some(pos);
        let d = g.turn_decision().unwrap();
        let before = g.snapshot();
        assert!(g.begin_cast(d.actor, d.id, h).is_err());
        assert_eq!(g.snapshot(), before);
    }
    g.turns.position = Some((1, Seat::P0, Step::PrecombatMain));
    let d = g.turn_decision().unwrap();
    let p = g
        .begin_cast(d.actor, d.id, h)
        .expect("payable Fodder must begin");
    let before = g.snapshot();
    assert!(g.finish_cast(d.actor, p.id).is_err());
    assert_eq!(g.snapshot(), before);
    g.cancel_payment(d.actor, p.id).unwrap();
    fodder(&mut g);
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert!(g.begin_cast(d.actor, d.id, h).is_err());
    assert_eq!(g.snapshot(), before);
}
#[test]
fn tokens_lethal_bite_ceases_and_pending_growth_cannot_recreate_dead_target() {
    let mut g = ready();
    fodder(&mut g);
    pair(&mut g);
    let hs = tokens(&g);
    let cub = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    instant(&mut g, "giant-growth", &[hs[0]]);
    pass(&mut g);
    instant(&mut g, "bite-down", &[cub, hs[0]]);
    pair(&mut g);
    assert!(g.objects.get(hs[0]).is_err());
    assert_eq!(tokens(&g), vec![hs[1]]);
    assert!(
        g.objects.in_zone(Zone::Graveyard(Seat::P0)).all(|h| g
            .objects
            .get(h)
            .unwrap()
            .card
            .identity()
            .key
            != "goblin-token")
    );
    // Test-only storage reuse while the old-target Growth remains on stack.
    let replacement = add(&mut g, "goblin-token", Seat::P0, Zone::Battlefield);
    assert_ne!(replacement, hs[0]);
    pair(&mut g);
    assert_eq!(g.creature_state(replacement).unwrap().power, 1);
    assert!(!g.last_resolution().unwrap().resolved);
    assert_eq!(g.creature_state(hs[1]).unwrap().power, 1);
    fodder(&mut g);
    pair(&mut g);
    assert_eq!(tokens(&g).len(), 4);
    assert!(!tokens(&g).contains(&hs[0]));
    assert_eq!(g.life(), [20, 20]);
}

fn declare_attack(g: &mut Game, hs: &[Handle]) {
    let d = g.turn_decision().unwrap();
    let d = g.select_attackers(d.actor, d.id, hs).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
}
fn declare_block(g: &mut Game, hs: &[(Handle, Handle)]) {
    let d = g.turn_decision().unwrap();
    let d = g.select_blockers(d.actor, d.id, hs).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
}
fn damage(g: &mut Game) {
    let d = g.turn_decision().unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
}
#[test]
fn tokens_two_attack_one_dies_other_deals_one() {
    let mut g = ready();
    fodder(&mut g);
    pair(&mut g);
    let hs = tokens(&g);
    let cub = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    // Synthetic next controlled turn; normal-reset played proof is separate.
    g.turns.sick.clear();
    g.turns.position = Some((3, Seat::P0, Step::BeginningCombat));
    pair(&mut g);
    declare_attack(&mut g, &hs);
    pair(&mut g);
    declare_block(&mut g, &[(cub, hs[0])]);
    pair(&mut g);
    damage(&mut g);
    assert_eq!(g.life(), [20, 19]);
    assert_eq!(tokens(&g), vec![hs[1]]);
    assert_eq!(g.creature_state(cub).unwrap().damage, 1);
    assert!(g.objects.get(hs[0]).is_err());
    assert_eq!(g.objects.in_zone(Zone::Graveyard(Seat::P0)).count(), 1); // Fodder only
}
#[test]
fn tokens_sick_blocker_growth_survives_and_cleanup_restores_one_one() {
    let mut g = ready();
    fodder(&mut g);
    pair(&mut g);
    let hs = tokens(&g);
    let cub = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    instant(&mut g, "giant-growth", &[hs[0]]);
    pair(&mut g);
    assert!(g.summoning_sick(hs[0]));
    // Synthetic opponent combat on the same turn number retains the sickness flag.
    g.turns.position = Some((1, Seat::P1, Step::BeginningCombat));
    g.set_turn_decision(Seat::P1, turns::TurnKind::Priority);
    pair(&mut g);
    declare_attack(&mut g, &[cub]);
    pair(&mut g);
    declare_block(&mut g, &[(hs[0], cub)]);
    pair(&mut g);
    damage(&mut g);
    assert_eq!(g.life(), [20, 20]);
    assert_eq!(tokens(&g), hs);
    assert_eq!(
        g.creature_state(hs[0]),
        Some(targets::CreatureState {
            power: 4,
            toughness: 4,
            damage: 2
        })
    );
    assert!(g.objects.get(cub).is_err());
    g.turns.position = Some((1, Seat::P1, Step::End));
    // Synthetic hand clear avoids an unrelated cleanup-discard decision.
    let hand: Vec<_> = g.objects.in_zone(Zone::Hand(Seat::P1)).collect();
    for h in hand {
        g.objects.remove(h).unwrap();
    }
    pair(&mut g);
    assert_eq!(
        g.creature_state(hs[0]),
        Some(targets::CreatureState {
            power: 1,
            toughness: 1,
            damage: 0
        })
    );
}
#[test]
fn tokens_resolution_snapshot_at_every_quantum_and_no_partial_exhausted_batch() {
    let mut g = ready();
    fodder(&mut g);
    pass(&mut g);
    let initial = g.snapshot();
    let d = g.turn_decision().unwrap();
    let action = TurnAction {
        decision: d.id,
        selection: TurnSelection::Pass(d.candidate(0)),
    };
    let mut result = g
        .apply_turn_quantum(d.actor, &action, NonZeroUsize::MIN)
        .unwrap();
    let mut steps = 0;
    while result == Progress::InternalYield {
        let snap = g.snapshot();
        g.restore(&snap).unwrap();
        assert!(g.turn_decision().is_none());
        result = g.resume(NonZeroUsize::MIN);
        steps += 1;
        assert!(steps < 10);
    }
    assert_eq!(tokens(&g).len(), 2);
    let final_state = g.snapshot();
    g.restore(&initial).unwrap();
    pass(&mut g);
    assert_eq!(normalized(&g.snapshot()), normalized(&final_state));
    g.restore(&initial).unwrap();
    g.objects.test_exhaust_births_after_one();
    let before = g.snapshot();
    let d = g.turn_decision().unwrap();
    assert_eq!(
        g.apply_turn(
            d.actor,
            &TurnAction {
                decision: d.id,
                selection: TurnSelection::Pass(d.candidate(0))
            }
        ),
        Err(turns::TurnError::Storage(StorageError::IdentityExhausted))
    );
    assert_eq!(g.snapshot(), before);
    assert!(tokens(&g).is_empty());
}

fn normalized(bytes: &[u8]) -> serde_json::Value {
    fn scrub(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, v) in m {
                    if k == "scope" || k == "store" {
                        *v = 0.into();
                    } else {
                        scrub(v);
                    }
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    scrub(v);
                }
            }
            _ => {}
        }
    }
    let e: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(e["payload"].as_str().unwrap()).unwrap();
    v["objects"]["id"] = 0.into();
    scrub(&mut v);
    v
}

#[test]
fn tokens_pinned_characteristics_and_no_token_card_cast() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let token = manifest["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "goblin-token");
    let token = token.unwrap();
    assert_eq!(token["characteristics"]["colors"], serde_json::json!(["R"]));
    assert_eq!(
        token["characteristics"]["type_line"],
        "Token Creature — Goblin"
    );
    assert_eq!(token["characteristics"]["power"], "1");
    assert_eq!(token["characteristics"]["toughness"], "1");
    let mut g = ready();
    let h = add(&mut g, "goblin-token", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert!(g.begin_cast(d.actor, d.id, h).is_err());
    assert_eq!(g.snapshot(), before);
}

#[test]
fn tokens_reference_observations_match_literal_oracle() {
    let mut results = serde_json::Map::new();
    for mode in ["repeat", "lethal", "stale-growth", "combat", "growth-block"] {
        let mut g = ready();
        fodder(&mut g);
        let pending_zero = tokens(&g).is_empty();
        pair(&mut g);
        let hs = tokens(&g);
        let mut births = hs.clone();
        if mode == "repeat" {
            fodder(&mut g);
            pair(&mut g);
            births = tokens(&g);
        } else {
            let cub = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
            match mode {
                "lethal" | "stale-growth" => {
                    if mode == "stale-growth" {
                        instant(&mut g, "giant-growth", &[hs[0]]);
                    }
                    pass(&mut g);
                    instant(&mut g, "bite-down", &[cub, hs[0]]);
                    pair(&mut g);
                    if mode == "stale-growth" {
                        pair(&mut g);
                        assert!(!g.last_resolution().unwrap().resolved);
                    }
                }
                "combat" => {
                    g.turns.sick.clear();
                    g.turns.position = Some((3, Seat::P0, Step::BeginningCombat));
                    pair(&mut g);
                    declare_attack(&mut g, &hs);
                    pair(&mut g);
                    declare_block(&mut g, &[(cub, hs[0])]);
                    pair(&mut g);
                    damage(&mut g);
                }
                "growth-block" => {
                    instant(&mut g, "giant-growth", &[hs[0]]);
                    pair(&mut g);
                    g.turns.position = Some((2, Seat::P1, Step::BeginningCombat));
                    g.set_turn_decision(Seat::P1, turns::TurnKind::Priority);
                    pair(&mut g);
                    declare_attack(&mut g, &[cub]);
                    pair(&mut g);
                    declare_block(&mut g, &[(hs[0], cub)]);
                    pair(&mut g);
                    damage(&mut g);
                    assert_eq!(g.creature_state(hs[0]).unwrap().damage, 2);
                    g.turns.position = Some((2, Seat::P1, Step::End));
                    let hand: Vec<_> = g.objects.in_zone(Zone::Hand(Seat::P1)).collect();
                    for h in hand {
                        g.objects.remove(h).unwrap();
                    }
                    pair(&mut g);
                }
                _ => unreachable!(),
            }
        }
        assert!(
            births
                .iter()
                .enumerate()
                .all(|(i, h)| !births[..i].contains(h))
        );
        let grave_tokens = [Seat::P0, Seat::P1]
            .into_iter()
            .flat_map(|seat| g.objects.in_zone(Zone::Graveyard(seat)))
            .filter(|h| g.objects.get(*h).unwrap().card.identity().key == "goblin-token")
            .count();
        let one_one = tokens(&g).iter().all(|h| {
            g.creature_state(*h)
                == Some(targets::CreatureState {
                    power: 1,
                    toughness: 1,
                    damage: 0,
                })
        });
        results.insert(mode.into(),serde_json::json!({"tokens":tokens(&g).len(),"unique_created":births.len(),"life":g.life(),"pending_zero":pending_zero,"one_one":one_one,"grave_tokens":grave_tokens}));
    }
    let actual = serde_json::Value::Object(results);
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/token-expectations.json"
    ))
    .unwrap();
    assert_eq!(actual, expected);
    if let Ok(path) = std::env::var("MTG_TOKEN_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&actual).unwrap()).unwrap();
    }
}
