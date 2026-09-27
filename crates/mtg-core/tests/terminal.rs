//! Original normal-reset scripts, frozen deck multisets, CR 103/104/704.
use mtg_core::objects::{Handle, Seat, Zone};
use mtg_core::opening::combat::CombatKind;
use mtg_core::opening::mana::Color;
use mtg_core::opening::terminal::{LossReason, Outcome};
use mtg_core::opening::turns::{Step, TurnAction, TurnKind, TurnSelection};
use mtg_core::opening::{Config, DeckConfig, Game, OpeningAction, ResetError, Selection};
fn config(deck: &str, start: u8) -> Config {
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
    let mut remaining = vec![];
    for c in d["cards"].as_array().unwrap() {
        for _ in 0..c["copies"].as_u64().unwrap() {
            remaining.push(c["card_id"].as_str().unwrap().to_string());
        }
    }
    let prefix = if deck == "green" {
        vec![
            "forest",
            "forest",
            "forest",
            "forest",
            "bear-cub",
            "giant-growth",
            "bite-down",
        ]
    } else {
        vec![
            "mountain",
            "mountain",
            "mountain",
            "mountain",
            "swab-goblin",
            "swab-goblin",
            "mountain",
        ]
    };
    let mut order = vec![];
    for k in prefix {
        let i = remaining.iter().position(|c| c == k).unwrap();
        order.push(remaining.remove(i));
    }
    order.extend(remaining);
    Config {
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
        starting_seat: start,
        ..Config::default()
    }
}
fn keep(g: &mut Game) {
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
}
fn advance(g: &mut Game, attack: bool) {
    let d = g.turn_decision().unwrap();
    match d.kind {
        TurnKind::Combat(kind) => {
            let mut id = d.id;
            if kind == CombatKind::Attackers {
                let c = g.combat_decision(d.actor, 80).unwrap();
                id = g
                    .select_attackers(d.actor, id, if attack { &c.attackers } else { &[] })
                    .unwrap()
                    .id;
            }
            if kind == CombatKind::Blockers {
                id = g.select_blockers(d.actor, id, &[]).unwrap().id;
            }
            g.finish_combat(d.actor, id).unwrap();
        }
        kind => {
            let selection = match kind {
                TurnKind::Priority => TurnSelection::Pass(d.candidate(0)),
                TurnKind::Discard { count } => {
                    let n = g.discard_cards().unwrap().len();
                    TurnSelection::Discard((n - count..n).map(|i| d.candidate(i)).collect())
                }
                _ => unreachable!(),
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
    }
}
fn reach(g: &mut Game, turn: u64, seat: Seat) {
    for _ in 0..100 {
        if g.turn_position() == Some((turn, seat, Step::PrecombatMain)) {
            return;
        }
        advance(g, false);
    }
    panic!("script did not reach main");
}
fn land(g: &mut Game, seat: Seat) {
    let d = g.turn_decision().unwrap();
    g.play_land(seat, d.id, g.land_candidates(seat)[0]).unwrap();
}
fn cast(g: &mut Game, seat: Seat, key: &str, targets: &[Handle], color: Color) {
    let h = g
        .objects()
        .in_zone(Zone::Hand(seat))
        .find(|h| g.objects().get(*h).unwrap().card.identity().key == key)
        .unwrap();
    let d = g.turn_decision().unwrap();
    let mut p = if targets.is_empty() {
        g.begin_cast(seat, d.id, h).unwrap()
    } else {
        let mut t = g.begin_targeted_cast(seat, d.id, h, 80).unwrap();
        for &h in targets {
            t = g.choose_target(seat, t.id, h).unwrap();
        }
        g.finish_targets(seat, t.id).unwrap()
    };
    let cost = if key == "giant-growth" { 1 } else { 2 };
    let sources = g.cast_mana_sources(seat);
    assert!(sources.len() >= cost);
    for h in sources.into_iter().take(cost) {
        p = g.cast_tap_mana(seat, p.id, h).unwrap();
    }
    for _ in 0..cost {
        p = g.choose_payment(seat, p.id, color).unwrap();
    }
    g.finish_cast(seat, p.id).unwrap();
    advance(g, false);
    advance(g, false);
}
#[test]
fn terminal_full_opening_turns_combat_best_of_one_and_explicit_reset() {
    for deck in ["red", "green"] {
        for start in [0, 1] {
            let cfg = config(deck, start);
            let mut g = Game::new().unwrap();
            g.reset(&cfg, 42, 9).unwrap();
            keep(&mut g);
            let actor = if start == 0 { Seat::P0 } else { Seat::P1 };
            let other = if start == 0 { Seat::P1 } else { Seat::P0 };
            reach(&mut g, 1, actor);
            land(&mut g, actor);
            reach(&mut g, 3, actor);
            land(&mut g, actor);
            cast(
                &mut g,
                actor,
                if deck == "red" {
                    "swab-goblin"
                } else {
                    "bear-cub"
                },
                &[],
                if deck == "red" {
                    Color::Red
                } else {
                    Color::Green
                },
            );
            let mut damage_events = 0;
            let mut last = 20;
            for _ in 0..1500 {
                if g.outcome().is_some() {
                    break;
                }
                advance(&mut g, true);
                let life = g.life()[1 - start as usize];
                if life != last {
                    damage_events += 1;
                    assert_eq!(life, 20 - 2 * damage_events);
                    last = life;
                }
            }
            let mut losses = [None; 2];
            losses[1 - start as usize] = Some(LossReason::Life);
            assert_eq!(
                g.outcome(),
                Some(Outcome {
                    winner: Some(actor),
                    losses
                })
            );
            assert_eq!(damage_events, 10);
            assert_eq!(g.life()[start as usize], 20);
            assert!(g.turn_decision().is_none());
            let before = format!("{g:?}");
            for _ in 0..3 {
                assert_eq!(g.outcome().unwrap().winner, Some(actor));
            }
            assert_eq!(format!("{g:?}"), before);
            let old = g.episode_id().unwrap();
            g.reset(&cfg, 42, 9).unwrap();
            assert_eq!(g.life(), [20, 20]);
            assert_eq!(g.outcome(), None);
            assert_ne!(g.episode_id(), Some(old));
            assert!(g.concede(other, old).is_err());
        }
    }
}
#[test]
fn terminal_full_opening_empty_draw_and_external_stop_remain_distinct() {
    for start in [0, 1] {
        let cfg = config("green", start);
        let mut g = Game::new().unwrap();
        g.reset(&cfg, 1, 1).unwrap();
        keep(&mut g);
        for _ in 0..10 {
            advance(&mut g, false);
        }
        assert_eq!(g.outcome(), None); // collector stops; not a rules result
        for _ in 0..2000 {
            if g.outcome().is_some() {
                break;
            }
            advance(&mut g, false);
        }
        let winner = if start == 0 { Seat::P0 } else { Seat::P1 };
        let loser = if start == 0 { Seat::P1 } else { Seat::P0 };
        let mut losses = [None; 2];
        losses[1 - start as usize] = Some(LossReason::EmptyDraw);
        assert_eq!(
            g.outcome(),
            Some(Outcome {
                winner: Some(winner),
                losses
            })
        );
        assert_eq!(g.turn_position(), Some((68, loser, Step::Draw)));
        assert_eq!(g.life(), [20, 20]);
        assert_eq!(g.objects().in_zone(Zone::Library(winner)).count(), 0);
        assert_eq!(g.objects().in_zone(Zone::Library(loser)).count(), 0);
    }
}
#[test]
fn terminal_reset_after_real_mana_growth_and_bite_damage() {
    let cfg = config("green", 0);
    let mut g = Game::new().unwrap();
    g.reset(&cfg, 42, 9).unwrap();
    keep(&mut g);
    for turn in 1..=7 {
        let seat = if turn % 2 == 1 { Seat::P0 } else { Seat::P1 };
        reach(&mut g, turn, seat);
        land(&mut g, seat);
        if turn == 3 || turn == 4 {
            cast(&mut g, seat, "bear-cub", &[], Color::Green);
        }
    }
    let own = g
        .objects()
        .in_zone(Zone::Battlefield)
        .find(|h| {
            let o = g.objects().get(*h).unwrap();
            o.owner == Seat::P0 && o.card.identity().key == "bear-cub"
        })
        .unwrap();
    let enemy = g
        .objects()
        .in_zone(Zone::Battlefield)
        .find(|h| {
            let o = g.objects().get(*h).unwrap();
            o.owner == Seat::P1 && o.card.identity().key == "bear-cub"
        })
        .unwrap();
    cast(&mut g, Seat::P0, "giant-growth", &[enemy], Color::Green);
    cast(&mut g, Seat::P0, "bite-down", &[own, enemy], Color::Green);
    assert_eq!(g.creature_state(enemy).unwrap().damage, 2);
    assert_eq!(g.creature_state(enemy).unwrap().toughness, 5);
    let d = g.turn_decision().unwrap();
    g.tap_mana(Seat::P0, d.id, g.mana_sources(Seat::P0)[0])
        .unwrap();
    assert_eq!(g.mana()[0][4], 1);
    let mut bad = cfg.clone();
    bad.seats[0].deck = "malformed".into();
    let before = format!("{g:?}");
    assert_eq!(g.reset(&bad, 99, 99), Err(ResetError::UnsupportedDeck));
    assert_eq!(format!("{g:?}"), before);
    // Same random reset after played state and fresh allocation: compare RNG/state
    // semantic projection, excluding necessarily fresh handles/decision epochs.
    let random = Config::default();
    g.reset(&random, 77, 33).unwrap();
    let mut fresh = Game::new().unwrap();
    fresh.reset(&random, 77, 33).unwrap();
    assert_eq!(g.life(), [20, 20]);
    assert_eq!(g.mana(), [[0; 6]; 2]);
    assert!(g.turn_position().is_none());
    assert!(g.combat().is_empty());
    assert!(g.creature_state(enemy).is_none());
    assert!(g.objects().get(own).is_err());
    for seat in [Seat::P0, Seat::P1] {
        for zone in [Zone::Hand(seat), Zone::Library(seat)] {
            let cards = |game: &Game| {
                game.objects()
                    .in_zone(zone)
                    .map(|h| game.objects().get(h).unwrap().card)
                    .collect::<Vec<_>>()
            };
            assert_eq!(cards(&g), cards(&fresh));
        }
    }
    assert_eq!(g.outcome(), None);
    assert_eq!(g.decision().unwrap().actor, Seat::P0);
}
