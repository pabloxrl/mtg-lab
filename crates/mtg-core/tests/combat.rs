//! Real reset, keep, lands, cast, combat and cleanup. CR 302.6/508–510/514.2.
use mtg_core::objects::{Seat, Zone};
use mtg_core::opening::mana::Color;
use mtg_core::opening::turns::{Step, TurnAction, TurnKind, TurnSelection};
use mtg_core::opening::{Config, DeckConfig, Game, OpeningAction, Selection};
fn advance(g: &mut Game) {
    let d = g.turn_decision().unwrap();
    let selection = match d.kind {
        TurnKind::TriggerOrder | TurnKind::TriggerTarget => {
            panic!("no trigger sources in this script")
        }
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
fn combat_normal_reset_cast_sickness_attack_cleanup_and_next_untap_both_seats() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    for (deck, key, color) in [
        ("green", "bear-cub", Color::Green),
        ("red", "swab-goblin", Color::Red),
    ] {
        for actor in [Seat::P0, Seat::P1] {
            let definition = manifest["decks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["id"] == deck)
                .unwrap();
            let mut order = vec![];
            for c in definition["cards"].as_array().unwrap() {
                for _ in 0..c["copies"].as_u64().unwrap() {
                    order.push(c["card_id"].as_str().unwrap().to_string());
                }
            }
            order.sort_by_key(|c| {
                if c == key {
                    0
                } else if c == "forest" || c == "mountain" {
                    1
                } else {
                    2
                }
            });
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
            let first = if actor == Seat::P0 { 1 } else { 2 };
            let mut lands = vec![];
            for turn in [first, first + 2] {
                for _ in 0..60 {
                    if g.turn_position() == Some((turn, actor, Step::PrecombatMain)) {
                        break;
                    }
                    advance(&mut g);
                }
                assert_eq!(g.turn_position(), Some((turn, actor, Step::PrecombatMain)));
                let d = g.turn_decision().unwrap();
                g.play_land(actor, d.id, g.land_candidates(actor)[0])
                    .unwrap();
                lands.push(g.objects().in_zone(Zone::Battlefield).last().unwrap());
            }
            // Cast after combat; no synthetic jump or unsupported combat required.
            for _ in 0..8 {
                advance(&mut g);
            }
            assert_eq!(
                g.turn_position(),
                Some((first + 2, actor, Step::PostcombatMain))
            );
            let d = g.turn_decision().unwrap();
            let card = g.cast_candidates(actor)[0];
            let mut p = g.begin_cast(actor, d.id, card).unwrap();
            for land in &lands {
                p = g.cast_tap_mana(actor, p.id, *land).unwrap();
            }
            p = g.choose_payment(actor, p.id, color).unwrap();
            p = g.choose_payment(actor, p.id, color).unwrap();
            g.finish_cast(actor, p.id).unwrap();
            assert_eq!(g.mana(), [[0; 6]; 2]);
            assert_eq!(g.turn_decision().unwrap().actor, actor);
            assert_eq!(g.objects().in_zone(Zone::Stack).count(), 1);
            advance(&mut g);
            advance(&mut g);
            let creature = g.objects().in_zone(Zone::Battlefield).last().unwrap();
            assert_eq!(g.objects().get(creature).unwrap().card.identity().key, key);
            assert!(g.summoning_sick(creature));
            for _ in 0..60 {
                if g.turn_position() == Some((first + 4, actor, Step::Upkeep)) {
                    break;
                }
                advance(&mut g);
            }
            assert_eq!(g.turn_position(), Some((first + 4, actor, Step::Upkeep)));
            assert!(!g.summoning_sick(creature));
            assert!(lands.iter().all(|h| !g.objects().get(*h).unwrap().tapped));
            for _ in 0..16 {
                if g.turn_position().unwrap().2 == Step::DeclareAttackers {
                    break;
                }
                advance(&mut g);
            }
            let d = g.turn_decision().unwrap();
            let choices = g.combat_decision(actor, 80).unwrap();
            assert_eq!(choices.attackers, vec![creature]);
            let d = g.select_attackers(actor, d.id, &[creature]).unwrap();
            g.finish_combat(actor, d.id).unwrap();
            assert!(g.objects().get(creature).unwrap().tapped);
            advance(&mut g);
            advance(&mut g);
            let d = g.turn_decision().unwrap();
            let defender = if actor == Seat::P0 {
                Seat::P1
            } else {
                Seat::P0
            };
            assert_eq!(d.actor, defender);
            let d = g.select_blockers(defender, d.id, &[]).unwrap();
            g.finish_combat(defender, d.id).unwrap();
            advance(&mut g);
            advance(&mut g);
            let d = g.turn_decision().unwrap();
            g.finish_combat(actor, d.id).unwrap();
            assert_eq!(
                g.life(),
                if actor == Seat::P0 {
                    [20, 18]
                } else {
                    [18, 20]
                }
            );
            for _ in 0..60 {
                if g.turn_position() == Some((first + 6, actor, Step::Upkeep)) {
                    break;
                }
                advance(&mut g);
            }
            assert_eq!(g.turn_position(), Some((first + 6, actor, Step::Upkeep)));
            assert!(!g.objects().get(creature).unwrap().tapped);
            assert_eq!(g.creature_state(creature).unwrap().damage, 0);
            assert!(g.combat().is_empty());
        }
    }
}
