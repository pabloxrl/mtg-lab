use mtg_core::objects::{Seat, StorageError, Zone};
use mtg_core::opening::{Config, DeckConfig, Game, OpeningChoice, ResetError};

// Independent setup source: RFC 0002 §3 frozen decks, CR 103.3/103.5.
fn ordered(deck: &str) -> DeckConfig {
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
    for entry in d["cards"].as_array().unwrap() {
        for _ in 0..entry["copies"].as_u64().unwrap() {
            order.push(entry["card_id"].as_str().unwrap().to_string());
        }
    }
    order.reverse(); // opening has spells, not only identical basic lands
    DeckConfig {
        deck: deck.into(),
        order: Some(order),
    }
}
fn cards(game: &Game, zone: Zone) -> Vec<String> {
    game.objects()
        .in_zone(zone)
        .map(|h| {
            game.objects()
                .get(h)
                .unwrap()
                .card
                .identity()
                .key
                .to_string()
        })
        .collect()
}
#[test]
fn opening_ordered_both_starters_and_mirrors() {
    for decks in [
        ["red", "green"],
        ["green", "red"],
        ["red", "red"],
        ["green", "green"],
    ] {
        for starter in 0..2 {
            let config = Config {
                seats: decks.map(ordered).to_vec(),
                starting_seat: starter,
                ..Config::default()
            };
            let mut game = Game::new().unwrap();
            let decision = game.reset(&config, 0, 0).unwrap();
            assert_eq!(game.life(), [20, 20]);
            assert_eq!(decision.actor, [Seat::P0, Seat::P1][starter as usize]);
            assert_eq!(
                decision.candidates,
                [OpeningChoice::Keep, OpeningChoice::Mulligan]
            );
            assert_eq!(game.decision(), Some(decision));
            for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
                let order = config.seats[i].order.as_ref().unwrap();
                assert_eq!(cards(&game, Zone::Hand(seat)), order[..7]);
                assert_eq!(cards(&game, Zone::Library(seat)), order[7..]);
                for zone in [Zone::Hand(seat), Zone::Library(seat)] {
                    for h in game.objects().in_zone(zone) {
                        assert_eq!(game.objects().get(h).unwrap().owner, seat);
                    }
                }
            }
            assert_eq!(game.objects().slot_count(), 80);
            for zone in [
                Zone::Battlefield,
                Zone::Stack,
                Zone::Exile,
                Zone::Graveyard(Seat::P0),
                Zone::Graveyard(Seat::P1),
            ] {
                assert_eq!(game.objects().in_zone(zone).count(), 0);
            }
        }
    }
}
#[test]
fn opening_invalid_config_preserves_complete_state_and_rng() {
    let mut cases = Vec::new();
    let c = Config::default();
    let mut bad = c.clone();
    bad.seats.push(DeckConfig::new("red"));
    cases.push((bad, ResetError::UnsupportedSeats));
    let mut bad = c.clone();
    bad.game_number = 2;
    bad.sideboards = true;
    cases.push((bad, ResetError::UnsupportedMatch));
    let mut bad = c.clone();
    bad.seats.clear();
    cases.push((bad, ResetError::UnsupportedSeats));
    let mut bad = c.clone();
    bad.seats.pop();
    cases.push((bad, ResetError::UnsupportedSeats));
    let mut bad = c.clone();
    bad.starting_seat = 2;
    cases.push((bad, ResetError::InvalidStartingSeat));
    let mut bad = c.clone();
    bad.game_number = 0;
    cases.push((bad, ResetError::UnsupportedMatch));
    let mut bad = c.clone();
    bad.sideboards = true;
    cases.push((bad, ResetError::UnsupportedMatch));
    let mut bad = c.clone();
    bad.format = "commander".into();
    cases.push((bad, ResetError::UnsupportedFormat));
    let mut bad = c.clone();
    bad.rng_version = "future".into();
    cases.push((bad, ResetError::UnsupportedRng));
    let mut bad = c.clone();
    bad.shuffle_version = "future".into();
    cases.push((bad, ResetError::UnsupportedShuffle));
    let mut bad = c.clone();
    bad.seats[1].deck = "blue".into();
    cases.push((bad, ResetError::UnsupportedDeck));
    for replacement in ["unknown", "goblin-token", "forest"] {
        let mut bad = c.clone();
        bad.seats[0] = ordered("red");
        bad.seats[0].order.as_mut().unwrap()[0] = replacement.into();
        cases.push((bad, ResetError::InvalidDeckOrder));
    }
    let mut bad = c.clone();
    bad.seats[0] = ordered("red");
    bad.seats[0].order.as_mut().unwrap().pop();
    cases.push((bad, ResetError::InvalidDeckOrder));
    let mut bad = c.clone();
    bad.seats[0] = ordered("red");
    bad.seats[0].order.as_mut().unwrap().push("mountain".into());
    cases.push((bad, ResetError::InvalidDeckOrder));
    for initialized in [false, true] {
        let mut game = Game::new().unwrap();
        if initialized {
            game.reset(&c, 42, 9).unwrap();
        }
        for (bad, error) in &cases {
            let before = format!("{game:?}"); // every field, including private RNG and identity counters
            let capacities = game.objects().capacities();
            assert_eq!(game.reset(bad, 123, 456), Err(*error));
            assert_eq!(format!("{game:?}"), before);
            assert_eq!(game.objects().capacities(), capacities);
        }
    }
}
#[test]
fn opening_reset_restarts_rng_reuses_allocations_and_isolates_games() {
    let mut a = Game::new().unwrap();
    let mut b = Game::new().unwrap();
    let c = Config::default();
    let first = a.reset(&c, 42, 9).unwrap();
    b.reset(&c, 8, 3).unwrap();
    let untouched = format!("{b:?}");
    let expected = Zone::ALL.map(|z| cards(&a, z));
    assert_eq!(a.objects().in_zone(Zone::Hand(Seat::P0)).count(), 7);
    let old = a.objects().in_zone(Zone::Hand(Seat::P0)).next().unwrap();
    assert_eq!(b.objects().get(old), Err(StorageError::InvalidHandle));
    let capacity = a.objects().capacities();
    // Each reset consumes two full shuffles; different intervening seed/episode too.
    for episode in 0..32 {
        a.reset(&c, 99, episode).unwrap();
        let next = a.reset(&c, 42, 9).unwrap();
        assert!(next.generation > first.generation);
        assert_eq!(Zone::ALL.map(|z| cards(&a, z)), expected);
        assert_eq!(a.life(), [20, 20]);
        assert_eq!(a.objects().capacities(), capacity);
        assert_eq!(a.objects().get(old), Err(StorageError::InvalidHandle));
        assert_eq!(format!("{b:?}"), untouched);
    }
}

#[test]
fn opening_shuffle_matches_independent_vectors() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("../../../doc/evidence/opening/vectors.json")).unwrap();
    for vector in vectors.as_array().unwrap() {
        let mut game = Game::new().unwrap();
        game.reset(
            &Config::default(),
            vector["master"].as_u64().unwrap(),
            vector["episode"].as_u64().unwrap(),
        )
        .unwrap();
        for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            let mut actual = cards(&game, Zone::Hand(seat));
            actual.extend(cards(&game, Zone::Library(seat)));
            let expected: Vec<_> = vector["decks"][i]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap())
                .collect();
            assert_eq!(actual, expected);
        }
    }
}
