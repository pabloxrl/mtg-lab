//! Original CR 305.1–3, 305.6, 106.4, 107.4, 117.3c, 605.3 and 601.2h expectations.
use mtg_core::objects::{Handle, Seat, Zone};
use mtg_core::opening::mana::*;
use mtg_core::opening::turns::*;
use mtg_core::opening::{Config, DeckConfig, Game, OpeningAction, Selection};
fn setup(deck: &str) -> Game {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let d = manifest["decks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == deck)
        .unwrap();
    let mut order = Vec::new();
    for c in d["cards"].as_array().unwrap() {
        for _ in 0..c["copies"].as_u64().unwrap() {
            order.push(c["card_id"].as_str().unwrap().to_string());
        }
    }
    order.sort_by_key(|s| !matches!(s.as_str(), "forest" | "mountain"));
    let mut g = Game::new().unwrap();
    g.reset(
        &Config {
            seats: vec![
                DeckConfig {
                    deck: deck.into(),
                    order: Some(order.clone()),
                },
                DeckConfig {
                    deck: deck.into(),
                    order: Some(order),
                },
            ],
            ..Config::default()
        },
        42,
        9,
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
fn main(g: &mut Game) {
    pass(g);
    pass(g);
}
fn land(g: &mut Game) -> Handle {
    let d = g.turn_decision().unwrap();
    let candidates = g.land_candidates(d.actor);
    assert!(
        !candidates.is_empty(),
        "CR 305: legal land play must be offered"
    );
    let h = candidates[0];
    g.play_land(d.actor, d.id, h).unwrap();
    g.objects().in_zone(Zone::Battlefield).last().unwrap()
}
fn tap(g: &mut Game, h: Handle) {
    let d = g.turn_decision().unwrap();
    g.tap_mana(d.actor, d.id, h).unwrap();
}
#[test]
fn mana_forest_mountain_positive_negative_regression() {
    for (deck, color) in [("green", Color::Green), ("red", Color::Red)] {
        let mut g = setup(deck);
        main(&mut g);
        assert_eq!(
            g.land_candidates(Seat::P0).len(),
            7,
            "CR 305: main-phase land choices"
        );
        let hand = g.objects().in_zone(Zone::Hand(Seat::P0)).next().unwrap();
        let h = land(&mut g);
        assert!(g.objects().get(hand).is_err());
        assert_eq!(g.objects().in_zone(Zone::Hand(Seat::P0)).count(), 6);
        tap(&mut g, h);
        assert!(g.objects().get(h).unwrap().tapped);
        let mut expected = [[0; 6]; 2];
        expected[0][color.index()] = 1;
        assert_eq!(g.mana(), expected);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        assert_eq!(g.objects().in_zone(Zone::Stack).count(), 0);
        let before = format!("{g:?}");
        let d = g.turn_decision().unwrap();
        assert_eq!(g.tap_mana(Seat::P0, d.id, h), Err(ManaError::IllegalSource));
        assert_eq!(format!("{g:?}"), before);
        pass(&mut g);
        assert_eq!(g.mana(), expected);
        pass(&mut g);
        assert_eq!(g.mana(), [[0; 6]; 2]);
        assert_eq!(g.life(), [20; 2]);
    }
}
#[test]
fn mana_masked_second_land_and_timing_rejected_transactionally() {
    let mut g = setup("green");
    let h = g.objects().in_zone(Zone::Hand(Seat::P0)).next().unwrap();
    let before = format!("{g:?}");
    let d = g.turn_decision().unwrap();
    assert_eq!(g.play_land(Seat::P0, d.id, h), Err(ManaError::IllegalLand));
    assert_eq!(format!("{g:?}"), before);
    main(&mut g);
    assert!(!g.land_candidates(Seat::P0).is_empty());
    land(&mut g);
    assert!(g.land_candidates(Seat::P0).is_empty());
    let h = g.objects().in_zone(Zone::Hand(Seat::P0)).next().unwrap();
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    assert_eq!(g.play_land(Seat::P0, d.id, h), Err(ManaError::IllegalLand));
    assert_eq!(format!("{g:?}"), before);
    // Land allowance remains spent through postcombat main and resets on next turn.
    for _ in 0..8 {
        pass(&mut g);
    }
    assert_eq!(g.turn_position().unwrap().2, Step::PostcombatMain);
    assert!(g.land_candidates(Seat::P0).is_empty());
    for _ in 0..30 {
        let d = g.turn_decision().unwrap();
        if g.turn_position() == Some((3, Seat::P0, Step::PrecombatMain)) {
            break;
        }
        match d.kind {
            TurnKind::TriggerOrder => panic!("no trigger sources in this script"),
            TurnKind::Combat(_) => panic!("unexpected combat choice in this script"),
            TurnKind::Priority => pass(&mut g),
            TurnKind::Discard { count } => {
                g.apply_turn(
                    d.actor,
                    &TurnAction {
                        decision: d.id,
                        selection: TurnSelection::Discard(
                            (0..count).map(|i| d.candidate(i)).collect(),
                        ),
                    },
                )
                .unwrap();
            }
        }
    }
    assert_eq!(g.turn_position(), Some((3, Seat::P0, Step::PrecombatMain)));
    assert!(!g.land_candidates(Seat::P0).is_empty());
}
#[test]
fn mana_payment_private_atomic_rejection_and_duplicate_commit() {
    let mut g = setup("green");
    main(&mut g);
    let h = land(&mut g);
    tap(&mut g, h);
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.begin_payment(
            Seat::P0,
            d.id,
            ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 0
            }
        ),
        Err(ManaError::InsufficientMana)
    );
    assert_eq!(format!("{g:?}"), before);
    let p = g
        .begin_payment(
            Seat::P0,
            d.id,
            ManaCost {
                generic: 1,
                ..ManaCost::default()
            },
        )
        .unwrap();
    assert_eq!(p.choices, vec![Color::Green]);
    assert!(g.turn_decision().is_none());
    assert!(g.payment_decision(Seat::P1).is_none());
    let before = format!("{g:?}");
    assert!(
        g.apply_turn(
            Seat::P0,
            &TurnAction {
                decision: d.id,
                selection: TurnSelection::Pass(d.candidate(0))
            }
        )
        .is_err()
    );
    assert_eq!(format!("{g:?}"), before);
    assert!(g.finish_payment(Seat::P0, p.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    assert!(g.choose_payment(Seat::P1, p.id, Color::Green).is_err());
    assert_eq!(format!("{g:?}"), before);
    assert_eq!(
        g.choose_payment(Seat::P0, p.id, Color::Red),
        Err(ManaError::IllegalPayment)
    );
    assert_eq!(format!("{g:?}"), before);
    g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
    assert_eq!(
        g.mana()[0][4],
        1,
        "provisional payment must remain invisible"
    );
    assert!(g.payment_decision(Seat::P1).is_none());
    assert!(g.turn_decision().is_none());
    let before = format!("{g:?}");
    assert!(g.choose_payment(Seat::P0, p.id, Color::Green).is_err());
    assert_eq!(format!("{g:?}"), before);
    let done = g.payment_decision(Seat::P0).unwrap();
    assert!(done.choices.is_empty());
    g.finish_payment(Seat::P0, done.id).unwrap();
    assert_eq!(g.mana(), [[0; 6]; 2]);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
    let before = format!("{g:?}");
    assert!(g.finish_payment(Seat::P0, done.id).is_err());
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn mana_cancel_payment_restores_pool_and_invalidates_decisions() {
    let mut g = setup("red");
    main(&mut g);
    let h = land(&mut g);
    tap(&mut g, h);
    let old = g.turn_decision().unwrap();
    let p = g
        .begin_payment(
            Seat::P0,
            old.id,
            ManaCost {
                generic: 1,
                ..ManaCost::default()
            },
        )
        .unwrap();
    g.choose_payment(Seat::P0, p.id, Color::Red).unwrap();
    let p = g.payment_decision(Seat::P0).unwrap();
    g.cancel_payment(Seat::P0, p.id).unwrap();
    assert_eq!(g.mana()[0][3], 1);
    assert_ne!(g.turn_decision().unwrap().id, old.id);
}

#[test]
fn mana_foreign_stale_wrong_actor_and_pending_commands_preserve_state() {
    let mut g = setup("green");
    main(&mut g);
    let initial = g.turn_decision().unwrap();
    let hand = g.land_candidates(Seat::P0)[0];
    let mut foreign = setup("green");
    main(&mut foreign);
    let foreign_id = foreign.turn_decision().unwrap().id;
    let foreign_land = foreign.land_candidates(Seat::P0)[0];
    for (actor, id, h) in [
        (Seat::P1, initial.id, hand),
        (Seat::P0, foreign_id, hand),
        (Seat::P0, initial.id, foreign_land),
    ] {
        let before = format!("{g:?}");
        assert!(g.play_land(actor, id, h).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
    let h = land(&mut g);
    let before = format!("{g:?}");
    assert!(g.tap_mana(Seat::P0, initial.id, h).is_err());
    assert_eq!(format!("{g:?}"), before);
    tap(&mut g, h);
    let d = g.turn_decision().unwrap();
    let p = g
        .begin_payment(
            Seat::P0,
            d.id,
            ManaCost {
                generic: 1,
                ..ManaCost::default()
            },
        )
        .unwrap();
    for actor in [Seat::P0, Seat::P1] {
        assert!(g.land_candidates(actor).is_empty());
        assert!(g.mana_sources(actor).is_empty());
        let before = format!("{g:?}");
        assert!(g.play_land(actor, d.id, hand).is_err());
        assert!(g.tap_mana(actor, d.id, h).is_err());
        assert!(g.begin_payment(actor, d.id, ManaCost::default()).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
    let before = format!("{g:?}");
    assert!(g.cancel_payment(Seat::P1, p.id).is_err());
    assert!(
        g.choose_payment(Seat::P0, foreign_id, Color::Green)
            .is_err()
    );
    assert_eq!(format!("{g:?}"), before);
    g.reset(&Config::default(), 42, 9).unwrap();
    assert!(g.payment_decision(Seat::P0).is_none());
    let before = format!("{g:?}");
    assert!(g.finish_payment(Seat::P0, p.id).is_err());
    assert_eq!(format!("{g:?}"), before);
}

#[test]
fn mana_nonactive_priority_can_tap_but_cannot_play_land_and_action_resets_passes() {
    let mut g = setup("green");
    main(&mut g);
    let h = land(&mut g);
    // Pass through to P1 main: P0's existing Forest can activate on P1's turn.
    for _ in 0..16 {
        pass(&mut g);
    }
    assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::PrecombatMain)));
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    assert_eq!(d.actor, Seat::P0);
    assert!(g.land_candidates(Seat::P0).is_empty());
    tap(&mut g, h);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
    pass(&mut g);
    assert_eq!(g.turn_position().unwrap().2, Step::PrecombatMain);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P1);
    pass(&mut g);
    assert_eq!(g.turn_position().unwrap().2, Step::BeginningCombat);
}
