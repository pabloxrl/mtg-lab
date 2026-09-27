//! Normal reset through real lands, Cub, and a two-seat instant response chain.
//! CR 117.3/4, 601, 613.4c; pinned Growth is +3/+3, so 2+3+3 = 8.
use mtg_core::objects::{Seat, Zone};
use mtg_core::opening::mana::Color;
use mtg_core::opening::turns::{Step, TurnAction, TurnKind, TurnSelection};
use mtg_core::opening::{Config, DeckConfig, Game, OpeningAction, Selection};
fn advance(g: &mut Game) {
    let d = g.turn_decision().unwrap();
    let selection = match d.kind {
        TurnKind::Combat(_) => panic!("unexpected combat choice in this script"),
        TurnKind::Priority => TurnSelection::Pass(d.candidate(0)),
        TurnKind::Discard { count } => {
            TurnSelection::Discard((0..count).map(|i| d.candidate(i)).collect())
        }
    };
    g.apply_turn(
        d.actor,
        &TurnAction {
            decision: d.id,
            selection,
        },
    )
    .unwrap();
}
#[test]
fn targets_normal_reset_real_response_chain() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let deck = manifest["decks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == "green")
        .unwrap();
    let mut order = vec![
        "bear-cub".to_string(),
        "giant-growth".into(),
        "forest".into(),
        "forest".into(),
        "forest".into(),
        "forest".into(),
        "forest".into(),
    ];
    for c in deck["cards"].as_array().unwrap() {
        let key = c["card_id"].as_str().unwrap();
        let used = order.iter().filter(|s| s.as_str() == key).count();
        for _ in used..c["copies"].as_u64().unwrap() as usize {
            order.push(key.to_string());
        }
    }
    let mut g = Game::new().unwrap();
    g.reset(
        &Config {
            seats: vec![
                DeckConfig {
                    deck: "green".into(),
                    order: Some(order.clone()),
                },
                DeckConfig {
                    deck: "green".into(),
                    order: Some(order),
                },
            ],
            ..Config::default()
        },
        1,
        2,
    )
    .unwrap();
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
    for turn in 1..=3 {
        let seat = if turn == 2 { Seat::P1 } else { Seat::P0 };
        for _ in 0..40 {
            if g.turn_position() == Some((turn, seat, Step::PrecombatMain)) {
                break;
            }
            advance(&mut g);
        }
        assert_eq!(g.turn_position(), Some((turn, seat, Step::PrecombatMain)));
        let d = g.turn_decision().unwrap();
        g.play_land(seat, d.id, g.land_candidates(seat)[0]).unwrap();
    }
    while g.turn_position() != Some((3, Seat::P0, Step::PostcombatMain)) {
        advance(&mut g);
    }
    let cub = g
        .cast_candidates(Seat::P0)
        .into_iter()
        .find(|h| g.objects().get(*h).unwrap().card.identity().key == "bear-cub")
        .unwrap();
    let d = g.turn_decision().unwrap();
    g.begin_cast(Seat::P0, d.id, cub).unwrap();
    for _ in 0..2 {
        let p = g.payment_decision(Seat::P0).unwrap();
        g.cast_tap_mana(Seat::P0, p.id, g.cast_mana_sources(Seat::P0)[0])
            .unwrap();
        let p = g.payment_decision(Seat::P0).unwrap();
        g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
    }
    let p = g.payment_decision(Seat::P0).unwrap();
    g.finish_cast(Seat::P0, p.id).unwrap();
    advance(&mut g);
    advance(&mut g);
    for _ in 0..50 {
        if g.turn_position() == Some((5, Seat::P0, Step::Upkeep)) {
            break;
        }
        advance(&mut g);
    }
    assert_eq!(g.turn_position(), Some((5, Seat::P0, Step::Upkeep)));
    let cub = g
        .objects()
        .in_zone(Zone::Battlefield)
        .find(|h| g.objects().get(*h).unwrap().card.identity().key == "bear-cub")
        .unwrap();
    for seat in [Seat::P0, Seat::P1] {
        let spell = g
            .cast_candidates(seat)
            .into_iter()
            .find(|h| g.objects().get(*h).unwrap().card.identity().key == "giant-growth")
            .unwrap();
        let d = g.turn_decision().unwrap();
        let t = g.begin_targeted_cast(seat, d.id, spell, 80).unwrap();
        let t = g.choose_target(seat, t.id, cub).unwrap();
        let p = g.finish_targets(seat, t.id).unwrap();
        let land = g.cast_mana_sources(seat)[0];
        let p = g.cast_tap_mana(seat, p.id, land).unwrap();
        let p = g.choose_payment(seat, p.id, Color::Green).unwrap();
        g.finish_cast(seat, p.id).unwrap();
        assert_eq!(g.turn_decision().unwrap().actor, seat);
        advance(&mut g);
    }
    advance(&mut g);
    assert_eq!(g.creature_state(cub).unwrap().power, 5);
    assert_eq!(g.objects().in_zone(Zone::Stack).count(), 1);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
    advance(&mut g);
    advance(&mut g);
    assert_eq!(g.creature_state(cub).unwrap().power, 8);
    assert_eq!(g.objects().in_zone(Zone::Stack).count(), 0);
}
