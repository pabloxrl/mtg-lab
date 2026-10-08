//! Original expectations: pinned CR 103.8a, 117.3/4, 500-514, 106.4.
use mtg_core::objects::{Seat, Zone};
use mtg_core::opening::turns::*;
use mtg_core::opening::{ApplyError, Config, DeckConfig, Game, OpeningAction, Selection};
fn other(s: Seat) -> Seat {
    if s == Seat::P0 { Seat::P1 } else { Seat::P0 }
}
fn setup(starter: u8) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(
        &Config {
            starting_seat: starter,
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
    g
}
fn pass(g: &mut Game) {
    let d = g.turn_decision().unwrap();
    assert_eq!(d.kind, TurnKind::Priority);
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
fn seek(g: &mut Game, turn: u64, step: Step) {
    for _ in 0..100 {
        if g.turn_position().map(|p| (p.0, p.2)) == Some((turn, step)) {
            return;
        }
        let d = g.turn_decision().unwrap();
        match d.kind {
            TurnKind::TriggerOrder | TurnKind::TriggerTarget => {
                panic!("no trigger sources in this script")
            }
            TurnKind::Combat(_) => panic!("unexpected combat choice in this script"),
            TurnKind::Priority => pass(g),
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
    panic!("target turn/step not reached");
}
fn count(g: &Game, s: Seat) -> usize {
    g.objects().in_zone(Zone::Hand(s)).count()
}
#[test]
fn turns_first_draw_positive_negative_interaction_regression() {
    // Four assigned first-draw cases, both starting seats. Exact next card is
    // independently the eighth card of the supplied deck (frozen manifest order).
    for starter in 0..2 {
        let mut config = Config {
            starting_seat: starter,
            ..Config::default()
        };
        let manifest: serde_json::Value = serde_json::from_str(include_str!(
            "../../../data/cards/foundations_micro_v1.json"
        ))
        .unwrap();
        for input in &mut config.seats {
            let deck = manifest["decks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["id"] == input.deck)
                .unwrap();
            let mut order = Vec::new();
            for e in deck["cards"].as_array().unwrap() {
                for _ in 0..e["copies"].as_u64().unwrap() {
                    order.push(e["card_id"].as_str().unwrap().to_string());
                }
            }
            order.swap(7, 16); // independently chosen eighth card differs from basics
            input.order = Some(order);
        }
        let expected: Vec<_> = config
            .seats
            .iter()
            .map(|d| d.order.as_ref().unwrap()[7].clone())
            .collect();
        let mut g = Game::new().unwrap();
        g.reset(&config, 42, 9).unwrap();
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
        let active = if starter == 0 { Seat::P0 } else { Seat::P1 };
        let nonactive = other(active);
        let d = g
            .start_turns()
            .expect("completed opening must enter upkeep");
        assert_eq!(d.actor, active);
        assert_eq!(g.turn_position(), Some((1, active, Step::Upkeep)));
        pair(&mut g); // CR 103.8a skips the entire first draw STEP, including priority.
        assert_eq!(g.turn_position(), Some((1, active, Step::PrecombatMain)));
        assert_eq!(count(&g, active), 7);
        seek(&mut g, 2, Step::Upkeep);
        assert_eq!(count(&g, nonactive), 7);
        pair(&mut g);
        assert_eq!(g.turn_position(), Some((2, nonactive, Step::Draw)));
        assert_eq!(count(&g, nonactive), 8);
        let h = g.objects().in_zone(Zone::Hand(nonactive)).last().unwrap();
        assert_eq!(
            g.objects().get(h).unwrap().card.identity().key,
            expected[1 - starter as usize]
        );
        let before = count(&g, nonactive);
        pass(&mut g);
        assert_eq!(count(&g, nonactive), before);
        pass(&mut g);
        assert_eq!(count(&g, nonactive), before);
        seek(&mut g, 3, Step::Draw);
        assert_eq!(count(&g, active), 8);
        assert_eq!(count(&g, nonactive), 7);
        let h = g.objects().in_zone(Zone::Hand(active)).last().unwrap();
        assert_eq!(
            g.objects().get(h).unwrap().card.identity().key,
            expected[starter as usize]
        );
    }
}
#[test]
fn turns_priority_passing_positive_and_stale_candidates_negative() {
    let mut g = setup(0);
    g.start_turns().unwrap();
    pair(&mut g);
    let old = g.turn_decision().unwrap();
    pass(&mut g);
    assert_eq!(g.turn_position(), Some((1, Seat::P0, Step::PrecombatMain)));
    let current = g.turn_decision().unwrap();
    assert_eq!(current.actor, Seat::P1);
    let cases = [
        (
            current.actor,
            TurnAction {
                decision: old.id,
                selection: TurnSelection::Pass(old.candidate(0)),
            },
            ApplyError::StaleDecision,
        ),
        (
            current.actor,
            TurnAction {
                decision: current.id,
                selection: TurnSelection::Pass(old.candidate(0)),
            },
            ApplyError::StaleCandidate,
        ),
        (
            Seat::P0,
            TurnAction {
                decision: current.id,
                selection: TurnSelection::Pass(current.candidate(0)),
            },
            ApplyError::WrongActor,
        ),
        (
            current.actor,
            TurnAction {
                decision: current.id,
                selection: TurnSelection::Pass(current.candidate(1)),
            },
            ApplyError::IllegalCandidate,
        ),
        (
            current.actor,
            TurnAction {
                decision: current.id,
                selection: TurnSelection::Discard(vec![]),
            },
            ApplyError::WrongKind,
        ),
    ];
    for (actor, a, e) in cases {
        let before = format!("{g:?}");
        assert_eq!(g.apply_turn(actor, &a), Err(TurnError::Invalid(e)));
        assert_eq!(format!("{g:?}"), before);
    }
    pass(&mut g);
    assert_eq!(
        g.turn_position(),
        Some((1, Seat::P0, Step::BeginningCombat))
    );
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
}
#[test]
fn turns_red_mirror_mulligan_opening_interaction() {
    let mut g = Game::new().unwrap();
    let c = Config {
        starting_seat: 1,
        seats: vec![DeckConfig::new("red"), DeckConfig::new("red")],
        ..Config::default()
    };
    g.reset(&c, 42, 9).unwrap();
    for index in [0, 1] {
        let d = g.decision().unwrap();
        g.apply(
            d.actor,
            &OpeningAction {
                decision: d.id,
                selection: Selection::Choose(d.candidate(index)),
            },
        )
        .unwrap();
    }
    let d = g.decision().unwrap();
    assert_eq!(d.actor, Seat::P0);
    g.apply(
        d.actor,
        &OpeningAction {
            decision: d.id,
            selection: Selection::Bottom(vec![d.candidate(6)]),
        },
    )
    .unwrap();
    let d = g.decision().unwrap();
    g.apply(
        d.actor,
        &OpeningAction {
            decision: d.id,
            selection: Selection::Choose(d.candidate(0)),
        },
    )
    .unwrap();
    g.start_turns().unwrap();
    pair(&mut g);
    assert_eq!(g.turn_position(), Some((1, Seat::P1, Step::PrecombatMain)));
    assert_eq!([count(&g, Seat::P0), count(&g, Seat::P1)], [6, 7]);
    assert_eq!(g.life(), [20, 20]);
}
#[test]
fn turns_empty_combat_sequence_cleanup_and_reset_scope() {
    let mut g = setup(1);
    g.start_turns().unwrap();
    for step in [
        Step::Upkeep,
        Step::PrecombatMain,
        Step::BeginningCombat,
        Step::DeclareAttackers,
        Step::EndCombat,
        Step::PostcombatMain,
        Step::End,
    ] {
        assert_eq!(g.turn_position(), Some((1, Seat::P1, step)));
        let a = g.turn_decision().unwrap();
        assert_eq!(a.actor, Seat::P1);
        pass(&mut g);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        pass(&mut g);
    }
    assert_eq!(g.turn_position(), Some((2, Seat::P0, Step::Upkeep)));
    seek(&mut g, 2, Step::Cleanup);
    let d = g.turn_decision().unwrap();
    assert_eq!(d.kind, TurnKind::Discard { count: 1 });
    assert_eq!(d.actor, Seat::P0);
    let cards = g.discard_cards().unwrap();
    let card = g.objects().get(cards[7]).unwrap().card;
    for selection in [
        TurnSelection::Discard(vec![]),
        TurnSelection::Pass(d.candidate(0)),
        TurnSelection::Discard(vec![d.candidate(8)]),
    ] {
        let before = format!("{g:?}");
        assert!(
            g.apply_turn(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection
                }
            )
            .is_err()
        );
        assert_eq!(format!("{g:?}"), before);
    }
    g.apply_turn(
        d.actor,
        &TurnAction {
            decision: d.id,
            selection: TurnSelection::Discard(vec![d.candidate(7)]),
        },
    )
    .unwrap();
    assert_eq!(g.turn_position(), Some((3, Seat::P1, Step::Upkeep)));
    assert_eq!(count(&g, Seat::P0), 7);
    let h = g
        .objects()
        .in_zone(Zone::Graveyard(Seat::P0))
        .next()
        .unwrap();
    assert_eq!(g.objects().get(h).unwrap().card, card);
    let old = g.turn_decision().unwrap();
    g.reset(&Config::default(), 42, 9).unwrap();
    assert_eq!(g.turn_position(), None);
    assert_eq!(g.mana(), [[0; 6]; 2]);
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
    let new = g.start_turns().unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.apply_turn(
            new.actor,
            &TurnAction {
                decision: new.id,
                selection: TurnSelection::Pass(old.candidate(0))
            }
        ),
        Err(TurnError::Invalid(ApplyError::StaleCandidate))
    );
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn turns_cannot_start_before_opening_or_twice() {
    let mut g = Game::new().unwrap();
    assert_eq!(g.start_turns(), Err(TurnError::NotReady));
    g.reset(&Config::default(), 42, 9).unwrap();
    let before = format!("{g:?}");
    assert_eq!(g.start_turns(), Err(TurnError::NotReady));
    assert_eq!(format!("{g:?}"), before);
    let mut g = setup(0);
    g.start_turns().unwrap();
    let before = format!("{g:?}");
    assert_eq!(g.start_turns(), Err(TurnError::AlreadyStarted));
    assert_eq!(format!("{g:?}"), before);
}
