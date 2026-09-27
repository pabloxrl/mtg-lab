//! Declared synthetic positions. Expectations: CR 107.4, 117.3c/b, 117.4,
//! 302.1/302.6, 601.2f-i, 605.3, plus RFC atomic invalid-input contract.
use super::*;
use crate::opening::mana::Color;
use crate::opening::turns::{Step, TurnAction, TurnSelection};
fn ready(key: &str, pool: [u32; 6]) -> (Game, Handle) {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 9).unwrap();
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
    let h = g
        .objects
        .allocate(
            CardId::from_key(key).unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    g.turns.mana[0] = pool;
    (g, h)
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
fn start(g: &mut Game, h: Handle) -> PaymentDecision {
    let d = g.turn_decision().unwrap();
    assert!(
        g.cast_candidates(d.actor).contains(&h),
        "CR 601: payable vanilla creature must be offered"
    );
    g.begin_cast(d.actor, d.id, h).unwrap()
}
fn pay(g: &mut Game, color: Color) {
    let d = g.payment_decision(Seat::P0).unwrap();
    g.choose_payment(Seat::P0, d.id, color).unwrap();
}
fn finish(g: &mut Game) {
    let d = g.payment_decision(Seat::P0).unwrap();
    g.finish_cast(Seat::P0, d.id).unwrap();
}
#[test]
fn casting_literal_payment_stack_resolution_and_priority() {
    for (key, color) in [("bear-cub", Color::Green), ("swab-goblin", Color::Red)] {
        let (mut g, h) = ready(key, [0, 0, 0, 1, 1, 0]);
        let rng = format!("{:?}", g.rng);
        start(&mut g, h);
        assert!(g.turn_decision().is_none());
        assert!(g.payment_decision(Seat::P1).is_none());
        assert!(g.objects.get(h).is_ok());
        pay(&mut g, color);
        pay(
            &mut g,
            if color == Color::Green {
                Color::Red
            } else {
                Color::Green
            },
        );
        assert_eq!(g.mana()[0], [0, 0, 0, 1, 1, 0]);
        finish(&mut g);
        assert_eq!(g.mana(), [[0; 6]; 2]);
        assert!(g.objects.get(h).is_err());
        let stack: Vec<_> = g.objects.in_zone(Zone::Stack).collect();
        assert_eq!(stack.len(), 1);
        assert_eq!(g.objects.get(stack[0]).unwrap().card.identity().key, key);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        pass(&mut g);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P1);
        assert_eq!(g.objects.in_zone(Zone::Stack).count(), 1);
        pass(&mut g);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        assert_eq!(g.turn_position(), Some((1, Seat::P0, Step::PrecombatMain)));
        assert_eq!(g.objects.in_zone(Zone::Stack).count(), 0);
        let creature = g.objects.in_zone(Zone::Battlefield).last().unwrap();
        assert!(g.objects.get(stack[0]).is_err());
        let o = g.objects.get(creature).unwrap();
        assert_eq!(
            (o.card.identity().key, o.owner, o.controller, o.tapped),
            (key, Seat::P0, Seat::P0, false)
        );
        assert!(g.summoning_sick(creature));
        assert_eq!(g.life(), [20; 2]);
        assert_eq!(format!("{:?}", g.rng), rng);
    }
}
#[test]
fn casting_generic_choices_and_atomic_rejected_final_command() {
    let (mut g, h) = ready("bear-cub", [0, 0, 0, 1, 2, 0]);
    let old = g.turn_decision().unwrap();
    let p = start(&mut g, h);
    pay(&mut g, Color::Green);
    let d = g.payment_decision(Seat::P0).unwrap();
    assert_eq!(d.choices, vec![Color::Red, Color::Green]);
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, d.id).is_err());
    assert!(g.choose_payment(Seat::P0, p.id, Color::Green).is_err());
    assert!(g.begin_cast(Seat::P0, old.id, h).is_err());
    assert!(g.finish_payment(Seat::P0, d.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    pay(&mut g, Color::Red);
    finish(&mut g);
    assert_eq!(g.mana()[0], [0, 0, 0, 0, 1, 0]);
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, d.id).is_err());
    assert!(g.begin_cast(Seat::P0, old.id, h).is_err());
    assert_eq!(format!("{g:?}"), before);
    let (mut alternate, card) = ready("bear-cub", [0, 0, 0, 1, 2, 0]);
    start(&mut alternate, card);
    pay(&mut alternate, Color::Green);
    pay(&mut alternate, Color::Green);
    finish(&mut alternate);
    assert_eq!(alternate.mana()[0], [0, 0, 0, 1, 0, 0]);
    let (mut g, h) = ready("bear-cub", [0, 0, 0, 1, 1, 0]);
    start(&mut g, h);
    pay(&mut g, Color::Green);
    let d = g.payment_decision(Seat::P0).unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.choose_payment(Seat::P0, d.id, Color::Green),
        Err(ManaError::IllegalPayment)
    );
    assert_eq!(format!("{g:?}"), before);
    pay(&mut g, Color::Red);
    finish(&mut g);
}
#[test]
fn casting_missing_colors_masked_timing_foreign_and_wrong_actor() {
    for pool in [[0, 0, 0, 2, 0, 0], [0, 0, 0, 0, 1, 0], [0; 6]] {
        let (mut g, h) = ready("bear-cub", pool);
        let d = g.turn_decision().unwrap();
        assert!(!g.cast_candidates(Seat::P0).contains(&h));
        let before = format!("{g:?}");
        assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
    let (mut g, h) = ready("bear-cub", [0, 0, 0, 1, 1, 0]);
    let (foreign, fh) = ready("bear-cub", [0, 0, 0, 1, 1, 0]);
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    for (actor, id, card) in [
        (Seat::P1, d.id, h),
        (Seat::P0, foreign.turn_decision().unwrap().id, h),
        (Seat::P0, d.id, fh),
    ] {
        assert!(g.begin_cast(actor, id, card).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
    for step in [Step::Upkeep, Step::Draw, Step::BeginningCombat, Step::End] {
        g.turns.position = Some((1, Seat::P0, step));
        let before = format!("{g:?}");
        assert!(g.cast_candidates(Seat::P0).is_empty());
        assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
}
#[test]
fn casting_land_activation_choices_cancel_and_retry() {
    for key in ["bear-cub", "swab-goblin"] {
        let (mut g, h) = ready(key, [0; 6]);
        let lands: Vec<_> = ["forest", "mountain", "forest"]
            .into_iter()
            .map(|key| {
                g.objects
                    .allocate(CardId::from_key(key).unwrap(), Seat::P0, Zone::Battlefield)
                    .unwrap()
            })
            .collect();
        let p = start(&mut g, h);
        assert_eq!(g.cast_mana_sources(Seat::P0), lands);
        assert!(g.cast_mana_sources(Seat::P1).is_empty());
        let p = g.cast_tap_mana(Seat::P0, p.id, lands[1]).unwrap();
        let before = format!("{g:?}");
        assert!(g.finish_cast(Seat::P0, p.id).is_err());
        assert_eq!(format!("{g:?}"), before);
        let p = g.cast_tap_mana(Seat::P0, p.id, lands[0]).unwrap();
        let before = format!("{g:?}");
        assert!(g.cast_tap_mana(Seat::P0, p.id, lands[0]).is_err());
        assert!(g.cast_tap_mana(Seat::P1, p.id, lands[2]).is_err());
        assert_eq!(format!("{g:?}"), before);
        assert!(lands.iter().all(|h| !g.objects.get(*h).unwrap().tapped));
        assert_eq!(g.mana(), [[0; 6]; 2]);
        g.cancel_payment(Seat::P0, p.id).unwrap();
        assert!(g.objects.get(h).is_ok());
        assert!(lands.iter().all(|h| !g.objects.get(*h).unwrap().tapped));
        let p = start(&mut g, h);
        let p = g.cast_tap_mana(Seat::P0, p.id, lands[1]).unwrap();
        g.cast_tap_mana(Seat::P0, p.id, lands[0]).unwrap();
        pay(
            &mut g,
            if key == "bear-cub" {
                Color::Green
            } else {
                Color::Red
            },
        );
        pay(
            &mut g,
            if key == "bear-cub" {
                Color::Red
            } else {
                Color::Green
            },
        );
        finish(&mut g);
        assert!(g.objects.get(lands[0]).unwrap().tapped);
        assert!(g.objects.get(lands[1]).unwrap().tapped);
        assert!(!g.objects.get(lands[2]).unwrap().tapped);
        assert_eq!(g.mana(), [[0; 6]; 2]);
    }
}
#[test]
fn casting_lifo_synthetic_stack_and_mana_response_resets_passes() {
    // Explicit synthetic stack: these two sorcery-speed creatures cannot be
    // legally nested by casting. Instant response effects belong to GH-70.
    let (mut g, hand) = ready("bear-cub", [0; 6]);
    g.objects.remove(hand).unwrap();
    let bottom = g
        .objects
        .allocate(CardId::from_key("bear-cub").unwrap(), Seat::P0, Zone::Stack)
        .unwrap();
    let top = g
        .objects
        .allocate(
            CardId::from_key("swab-goblin").unwrap(),
            Seat::P1,
            Zone::Stack,
        )
        .unwrap();
    g.turns.stack = vec![bottom, top];
    let land = g
        .objects
        .allocate(
            CardId::from_key("mountain").unwrap(),
            Seat::P1,
            Zone::Battlefield,
        )
        .unwrap();
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    g.tap_mana(Seat::P1, d.id, land).unwrap();
    assert_eq!(
        g.objects.in_zone(Zone::Stack).collect::<Vec<_>>(),
        vec![bottom, top]
    );
    pass(&mut g); // P1's action reset consecutive passes: no resolution yet.
    assert_eq!(g.objects.in_zone(Zone::Stack).count(), 2);
    pass(&mut g);
    assert_eq!(
        g.objects.in_zone(Zone::Stack).collect::<Vec<_>>(),
        vec![bottom]
    );
    assert!(g.objects.get(top).is_err());
    let goblin = g.objects.in_zone(Zone::Battlefield).last().unwrap();
    assert_eq!(
        g.objects.get(goblin).unwrap().card.identity().key,
        "swab-goblin"
    );
    assert_eq!(g.objects.get(goblin).unwrap().controller, Seat::P1);
    assert!(g.summoning_sick(goblin));
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0); // active, not controller
    assert_eq!(g.mana()[1], [0, 0, 0, 1, 0, 0]);
    pass(&mut g);
    pass(&mut g);
    assert_eq!(g.objects.in_zone(Zone::Stack).count(), 0);
    let cub = g.objects.in_zone(Zone::Battlefield).last().unwrap();
    assert_eq!(g.objects.get(cub).unwrap().card.identity().key, "bear-cub");
    assert!(g.summoning_sick(cub));
    // Declared end-step position bypasses unimplemented combat, not a full game.
    g.turns.position = Some((1, Seat::P0, Step::End));
    pass(&mut g);
    pass(&mut g);
    assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::Upkeep)));
    assert!(!g.summoning_sick(goblin));
    assert!(g.summoning_sick(cub));
    g.turns.position = Some((2, Seat::P1, Step::End));
    pass(&mut g);
    pass(&mut g);
    assert!(!g.summoning_sick(cub));
}
#[test]
fn casting_fresh_forest_mountain_negative_and_unsupported_cards() {
    let (mut g, h) = ready("bear-cub", [0; 6]);
    let mountain = g
        .objects
        .allocate(
            CardId::from_key("mountain").unwrap(),
            Seat::P0,
            Zone::Battlefield,
        )
        .unwrap();
    let forest = g
        .objects
        .allocate(
            CardId::from_key("forest").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    let d = g.turn_decision().unwrap();
    g.play_land(Seat::P0, d.id, forest).unwrap();
    let forest = g.objects.in_zone(Zone::Battlefield).last().unwrap();
    let d = start(&mut g, h);
    let d = g.cast_tap_mana(Seat::P0, d.id, mountain).unwrap();
    g.cast_tap_mana(Seat::P0, d.id, forest).unwrap();
    pay(&mut g, Color::Green);
    pay(&mut g, Color::Red);
    finish(&mut g);
    let (mut g, h) = ready("bear-cub", [0; 6]);
    for _ in 0..2 {
        g.objects
            .allocate(
                CardId::from_key("mountain").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
    }
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    assert!(!g.cast_candidates(Seat::P0).contains(&h));
    assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
    assert_eq!(format!("{g:?}"), before);
    for key in ["giant-growth", "forest", "llanowar-elves"] {
        let (mut g, h) = ready(key, [0, 0, 0, 2, 2, 0]);
        let d = g.turn_decision().unwrap();
        let before = format!("{g:?}");
        assert!(!g.cast_candidates(Seat::P0).contains(&h));
        assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
}
#[test]
fn casting_nested_stale_cancel_reset_and_exhaustion() {
    let (mut g, h) = ready("bear-cub", [0, 0, 0, 2, 2, 0]);
    let h2 = g
        .objects
        .allocate(
            CardId::from_key("bear-cub").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    start(&mut g, h);
    pay(&mut g, Color::Green);
    pay(&mut g, Color::Red);
    let d = g.payment_decision(Seat::P0).unwrap();
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P1, d.id).is_err());
    assert!(g.finish_payment(Seat::P0, d.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    g.generation = u64::MAX;
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, d.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    g.generation = d.id.generation;
    finish(&mut g);
    for _ in 0..2 {
        let d = g.turn_decision().unwrap();
        let before = format!("{g:?}");
        assert!(g.cast_candidates(d.actor).is_empty());
        assert!(g.begin_cast(d.actor, d.id, h2).is_err());
        assert_eq!(format!("{g:?}"), before);
        pass(&mut g);
    }
    let p = start(&mut g, h2);
    pay(&mut g, Color::Green);
    let before = format!("{g:?}");
    assert!(g.cancel_payment(Seat::P0, p.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    let p = g.payment_decision(Seat::P0).unwrap();
    g.cancel_payment(Seat::P0, p.id).unwrap();
    assert_eq!(g.mana()[0], [0, 0, 0, 1, 1, 0]);
    let p = start(&mut g, h2);
    g.reset(&Config::default(), 42, 9).unwrap();
    assert!(g.turns.stack.is_empty());
    assert!(g.turns.sick.is_empty());
    assert!(g.turns.casting.is_none());
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, p.id).is_err());
    assert_eq!(format!("{g:?}"), before);
}
