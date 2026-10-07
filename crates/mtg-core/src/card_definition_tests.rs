//! Refactor baseline: six-card M1 acceptance, pinned manifest characteristics,
//! CR 107.4/302/305/601, and explicit unsupported-content/nonmutation contract.
//! Positions below are synthetic; existing Driver/real reference tests cover play.
use super::casting::CastError;
use super::mana::{Color, ManaCost};
use super::targets::{CreatureState, TargetError};
use super::turns::Step;
use super::*;

const SUPPORTED: [&str; 18] = [
    "firebrand-archer",
    "crackling-cyclops",
    "forest",
    "mountain",
    "bear-cub",
    "swab-goblin",
    "giant-growth",
    "bite-down",
    "llanowar-elves",
    "druid-of-the-cowl",
    "magnigoth-sentry",
    "axgard-cavalry",
    "tajuru-pathwarden",
    "thornweald-archer",
    "shivan-dragon",
    "wildheart-invoker",
    "thrill-of-possibility",
    "goblin-surprise",
];

fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 132, 0).unwrap();
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
    g.turns.mana = [[10; 6]; 2];
    g
}

#[test]
fn card_definitions_six_card_manifest_and_public_candidates() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    // Literal support list and behavior expectations come from M1 acceptance,
    // not the implementation's support lookup or reserved manifest behavior flag.
    for (key, mana_text, color, stats, is_instant) in [
        ("firebrand-archer", "{1}{R}", None, Some((2, 1)), false),
        ("crackling-cyclops", "{2}{R}", None, Some((0, 4)), false),
        ("goblin-surprise", "{2}{R}", None, None, true),
        ("thrill-of-possibility", "{1}{R}", None, None, true),
        ("wildheart-invoker", "{2}{G}{G}", None, Some((4, 3)), false),
        ("shivan-dragon", "{4}{R}{R}", None, Some((5, 5)), false),
        ("forest", "", Some(Color::Green), None, false),
        ("mountain", "", Some(Color::Red), None, false),
        ("bear-cub", "{1}{G}", None, Some((2, 2)), false),
        ("swab-goblin", "{1}{R}", None, Some((2, 2)), false),
        ("giant-growth", "{G}", None, None, true),
        ("bite-down", "{1}{G}", None, None, true),
        ("llanowar-elves", "{G}", None, Some((1, 1)), false),
        ("druid-of-the-cowl", "{1}{G}", None, Some((1, 3)), false),
        ("thornweald-archer", "{1}{G}", None, Some((2, 1)), false),
        ("tajuru-pathwarden", "{4}{G}", None, Some((5, 4)), false),
        ("axgard-cavalry", "{1}{R}", None, Some((2, 2)), false),
        ("magnigoth-sentry", "{3}{G}", None, Some((4, 4)), false),
    ] {
        let entry = manifest["cards"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == key)
            .unwrap();
        let card = CardId::from_key(key).unwrap();
        assert_eq!(
            card.identity().content_sha256,
            entry["content_sha256"].as_str().unwrap()
        );
        let characteristics = &entry["characteristics"];
        assert_eq!(characteristics["mana_cost"], mana_text);
        let expected_cost = if mana_text.is_empty() {
            None
        } else {
            let mut cost = ManaCost::default();
            // Independent tiny test decoder of the pinned symbols, not runtime code.
            for symbol in mana_text.split(['{', '}']).filter(|s| !s.is_empty()) {
                match symbol {
                    "R" => cost.colored[3] += 1,
                    "G" => cost.colored[4] += 1,
                    n => cost.generic += n.parse::<u32>().unwrap(),
                }
            }
            Some(cost)
        };
        assert_eq!(casting::cost(card), expected_cost, "{key}");
        assert_eq!(mana::basic_color(card), color, "{key}");
        assert_eq!(targets::instant(card), is_instant, "{key}");
        assert_eq!(
            characteristics["type_line"]
                .as_str()
                .unwrap()
                .contains("Instant"),
            is_instant
        );
        let mut g = ready();
        for seat in [Seat::P0, Seat::P1] {
            g.objects
                .allocate(
                    CardId::from_key("bear-cub").unwrap(),
                    seat,
                    Zone::Battlefield,
                )
                .unwrap();
        }
        let h = g
            .objects
            .allocate(card, Seat::P0, Zone::Hand(Seat::P0))
            .unwrap();
        assert_eq!(
            g.land_candidates(Seat::P0).contains(&h),
            color.is_some(),
            "{key}"
        );
        assert_eq!(
            g.cast_candidates(Seat::P0).contains(&h),
            expected_cost.is_some(),
            "{key}"
        );
        let expected_stats = stats.map(|(power, toughness)| {
            assert_eq!(characteristics["power"], power.to_string());
            assert_eq!(characteristics["toughness"], toughness.to_string());
            CreatureState {
                power,
                toughness,
                damage: 0,
            }
        });
        assert_eq!(g.creature_state(h), expected_stats, "{key}");
    }
}

#[test]
fn card_definitions_every_other_frozen_identity_rejects_play_without_mutation() {
    for card in CardId::all().filter(|c| {
        !SUPPORTED.contains(&c.identity().key)
            && c.identity().key != "dragon-fodder"
            && c.identity().key != "goblin-token"
    }) {
        let mut g = ready();
        let h = g
            .objects
            .allocate(card, Seat::P0, Zone::Hand(Seat::P0))
            .unwrap();
        let d = g.turn_decision().unwrap();
        let before = g.snapshot();
        assert_eq!(casting::cost(card), None);
        assert_eq!(mana::basic_color(card), None);
        assert!(!targets::instant(card));
        assert!(!g.cast_candidates(Seat::P0).contains(&h));
        assert!(!g.land_candidates(Seat::P0).contains(&h));
        assert_eq!(
            g.begin_cast(Seat::P0, d.id, h),
            Err(CastError::IllegalSpell)
        );
        assert_eq!(
            g.begin_targeted_cast(Seat::P0, d.id, h, 80),
            Err(TargetError::Cast(CastError::IllegalSpell))
        );
        assert_eq!(g.snapshot(), before);
        g.objects.move_to(h, Zone::Battlefield).unwrap();
        assert!(!g.supported_combat(), "{}", card.identity().key);
    }
}

#[test]
fn card_definitions_sentry_characteristics_and_complete_support() {
    let mut g = ready();
    let card = CardId::from_key("magnigoth-sentry").unwrap();
    let h = g
        .objects
        .allocate(card, Seat::P0, Zone::Battlefield)
        .unwrap();
    assert_eq!(
        g.creature_state(h),
        Some(CreatureState {
            power: 4,
            toughness: 4,
            damage: 0
        })
    );
    assert!(g.supported_combat());
    assert_eq!(
        casting::cost(card),
        Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 3
        })
    );
}

#[test]
fn card_definitions_unknown_and_corrupt_identity_rejection() {
    assert_eq!(CardId::from_key("unknown-card"), None);
    for json in ["21", "255", "-1", "256", "null", "\"bear-cub\""] {
        assert!(serde_json::from_str::<CardId>(json).is_err(), "{json}");
    }
    // All frozen identities, including unsupported ones, retain their wire IDs.
    for (i, card) in CardId::all().enumerate() {
        assert_eq!(serde_json::to_string(&card).unwrap(), i.to_string());
        assert_eq!(
            serde_json::from_str::<CardId>(&i.to_string()).unwrap(),
            card
        );
    }
}

#[test]
fn flying_reach_sentry_pinned_cost_casts_for_three_generic_one_green() {
    let mut g = ready();
    let card = CardId::from_key("magnigoth-sentry").unwrap();
    let h = g
        .objects
        .allocate(card, Seat::P0, Zone::Hand(Seat::P0))
        .unwrap();
    assert_eq!(
        casting::cost(card),
        Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 3
        })
    );
    let d = g.turn_decision().unwrap();
    g.begin_cast(Seat::P0, d.id, h).unwrap();
    for _ in 0..4 {
        let d = g.payment_decision(Seat::P0).unwrap();
        g.choose_payment(Seat::P0, d.id, Color::Green).unwrap();
    }
    let d = g.payment_decision(Seat::P0).unwrap();
    g.finish_cast(Seat::P0, d.id).unwrap();
    assert_eq!(g.objects.in_zone(Zone::Stack).count(), 1);
}
