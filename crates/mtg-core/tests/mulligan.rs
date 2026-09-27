use mtg_core::objects::{Handle, Seat, Zone};
use mtg_core::opening::*;

// Independent expectations: pinned CR 103.5 (bottom after EACH mulligan,
// declarations in starting-player order, seven maximum), CR 121.1 (top draw),
// RFC 0002 B015/B016 (determinism, invalid input is transactional).
fn config() -> Config {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let seats = ["red", "green"].map(|name| {
        let deck = manifest["decks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == name)
            .unwrap();
        let mut order = Vec::new();
        for entry in deck["cards"].as_array().unwrap() {
            for _ in 0..entry["copies"].as_u64().unwrap() {
                order.push(entry["card_id"].as_str().unwrap().to_string());
            }
        }
        // Distinct spell plus basic land in each replacement opening seven.
        order.swap(1, 16);
        DeckConfig {
            deck: name.into(),
            order: Some(order),
        }
    });
    Config {
        seats: seats.to_vec(),
        ..Config::default()
    }
}
fn game(c: &Config) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(c, 42, 9).unwrap();
    g
}
fn handles(g: &Game, z: Zone) -> Vec<Handle> {
    g.objects().in_zone(z).collect()
}
fn cards(g: &Game, z: Zone) -> Vec<&'static str> {
    handles(g, z)
        .iter()
        .map(|h| g.objects().get(*h).unwrap().card.identity().key)
        .collect()
}
fn action(g: &Game, index: usize) -> OpeningAction {
    let d = g.decision().unwrap();
    OpeningAction {
        decision: d.id,
        selection: Selection::Choose(d.candidate(index)),
    }
}
fn choose(g: &mut Game, index: usize) {
    let a = action(g, index);
    g.apply(g.decision().unwrap().actor, &a).unwrap();
}
fn bottom_action(g: &Game, indices: &[usize]) -> OpeningAction {
    let d = g.decision().unwrap();
    OpeningAction {
        decision: d.id,
        selection: Selection::Bottom(indices.iter().map(|&i| d.candidate(i)).collect()),
    }
}
fn bottom(g: &mut Game, indices: &[usize]) {
    let a = bottom_action(g, indices);
    g.apply(g.decision().unwrap().actor, &a).unwrap();
}
fn replacement(g: &Game, seat: Seat) -> Vec<Handle> {
    // Privileged explicit chance result: current hand followed by library.
    let mut v = handles(g, Zone::Hand(seat));
    v.extend(handles(g, Zone::Library(seat)));
    v
}
fn mulligan_ordered(g: &mut Game) {
    let d = g.decision().unwrap();
    let a = action(g, 1);
    g.apply_with_order(d.actor, &a, &replacement(g, d.actor))
        .unwrap();
}
fn bottom_count(g: &Game) -> usize {
    match g.decision().unwrap().kind {
        OpeningKind::Bottom { count, .. } => count,
        _ => panic!("expected bottom continuation"),
    }
}
fn reject(g: &mut Game, actor: Seat, a: &OpeningAction, e: ApplyError) {
    let before = format!("{g:?}");
    let capacities = g.objects().capacities();
    assert_eq!(g.apply(actor, a), Err(e));
    assert_eq!(format!("{g:?}"), before); // all private semantic fields AND RNG
    assert_eq!(g.objects().capacities(), capacities);
}
fn finish(g: &mut Game) {
    for _ in 0..2 {
        if g.decision().is_some() {
            choose(g, 0);
        }
    }
    assert!(g.decision().is_none());
}

#[test]
fn mulligan_rules_setup_two_player_opening_positive() {
    for starter in 0..2 {
        let mut c = config();
        c.starting_seat = starter;
        let mut g = game(&c);
        let expected = Zone::ALL.map(|z| cards(&g, z));
        assert_eq!(
            g.decision().unwrap().actor,
            [Seat::P0, Seat::P1][starter as usize]
        );
        choose(&mut g, 0);
        assert_eq!(
            g.decision().unwrap().actor,
            [Seat::P1, Seat::P0][starter as usize]
        );
        choose(&mut g, 0);
        assert!(g.decision().is_none());
        assert_eq!(Zone::ALL.map(|z| cards(&g, z)), expected);
        assert_eq!(g.life(), [20, 20]);
        for s in [Seat::P0, Seat::P1] {
            assert_eq!(cards(&g, Zone::Hand(s)).len(), 7);
            assert_eq!(cards(&g, Zone::Library(s)).len(), 33);
        }
    }
}
#[test]
fn mulligan_rules_setup_mulligan_bottom_positive() {
    let mut g = game(&config());
    let initial = cards(&g, Zone::Hand(Seat::P0));
    mulligan_ordered(&mut g);
    // Both players declare before any redraw/bottom continuation.
    assert_eq!(g.decision().unwrap().actor, Seat::P1);
    assert_eq!(g.decision().unwrap().kind, OpeningKind::KeepOrMulligan);
    choose(&mut g, 0);
    assert_eq!(bottom_count(&g), 1);
    bottom(&mut g, &[0]);
    assert_eq!(cards(&g, Zone::Hand(Seat::P0)), initial[1..]);
    assert_eq!(cards(&g, Zone::Library(Seat::P0)).last(), Some(&"mountain"));
    assert_eq!(g.decision().unwrap().actor, Seat::P0);
    choose(&mut g, 0);
    assert!(g.decision().is_none());
}
fn twice() -> Game {
    let mut g = game(&config());
    mulligan_ordered(&mut g);
    choose(&mut g, 0);
    bottom(&mut g, &[6]);
    mulligan_ordered(&mut g);
    assert_eq!(bottom_count(&g), 2);
    g
}
#[test]
fn mulligan_rules_setup_mulligan_bottom_negative() {
    let mut g = twice();
    let a = bottom_action(&g, &[0]);
    reject(&mut g, Seat::P0, &a, ApplyError::WrongCardinality);
    bottom(&mut g, &[0, 1]);
    assert_eq!(cards(&g, Zone::Hand(Seat::P0)).len(), 5);
}
#[test]
fn mulligan_rules_setup_mulligan_bottom_interaction() {
    let mut g = twice();
    let other = cards(&g, Zone::Hand(Seat::P1));
    bottom(&mut g, &[1, 0]);
    let library = cards(&g, Zone::Library(Seat::P0));
    assert_eq!(&library[33..], &["swab-goblin", "mountain"]);
    assert_eq!(cards(&g, Zone::Hand(Seat::P0)).len(), 5);
    assert_eq!(cards(&g, Zone::Hand(Seat::P1)), other);
    finish(&mut g);
}
#[test]
fn mulligan_rules_setup_mulligan_bottom_regression() {
    let mut g = game(&config());
    mulligan_ordered(&mut g);
    choose(&mut g, 0);
    assert_eq!(cards(&g, Zone::Hand(Seat::P0)).len(), 7);
    assert_eq!(bottom_count(&g), 1);
    bottom(&mut g, &[0]);
    assert_eq!(cards(&g, Zone::Hand(Seat::P0)).len(), 6);
    mulligan_ordered(&mut g);
    assert_eq!(cards(&g, Zone::Hand(Seat::P0)).len(), 7);
    assert_eq!(bottom_count(&g), 2);
    bottom(&mut g, &[0, 1]);
    assert_eq!(cards(&g, Zone::Hand(Seat::P0)).len(), 5);
    finish(&mut g);
}
#[test]
fn mulligan_rules_setup_deterministic_reset_positive() {
    let mut a = game(&Config::default());
    let mut b = game(&Config::default());
    for _ in 0..2 {
        assert_eq!(
            Zone::ALL.map(|z| cards(&a, z)),
            Zone::ALL.map(|z| cards(&b, z))
        );
        assert_eq!(a.decision().unwrap().actor, b.decision().unwrap().actor);
        assert_eq!(
            a.decision().unwrap().candidates,
            b.decision().unwrap().candidates
        );
        choose(&mut a, 0);
        choose(&mut b, 0);
    }
    assert_eq!(a.decision(), None);
    assert_eq!(b.decision(), None);
}
#[test]
fn mulligan_rules_setup_deterministic_reset_interaction() {
    let mut a = game(&Config::default());
    let mut b = game(&Config::default());
    for g in [&mut a, &mut b] {
        choose(g, 1);
        choose(g, 0);
        bottom(g, &[2]);
        choose(g, 0);
    }
    assert_eq!(
        Zone::ALL.map(|z| cards(&a, z)),
        Zone::ALL.map(|z| cards(&b, z))
    );
}
#[test]
fn mulligan_rules_setup_deterministic_reset_regression() {
    let mut a = game(&Config::default());
    choose(&mut a, 1);
    choose(&mut a, 1);
    bottom(&mut a, &[0]);
    bottom(&mut a, &[0]);
    choose(&mut a, 1);
    choose(&mut a, 1);
    bottom(&mut a, &[0, 1]);
    bottom(&mut a, &[0, 1]);
    a.reset(&Config::default(), 42, 9).unwrap();
    let mut b = game(&Config::default());
    for g in [&mut a, &mut b] {
        choose(g, 1);
        choose(g, 0);
        bottom(g, &[2]);
        finish(g);
    }
    assert_eq!(
        Zone::ALL.map(|z| cards(&a, z)),
        Zone::ALL.map(|z| cards(&b, z))
    );
}
fn green_draw() -> Game {
    let mut c = config();
    c.seats.swap(0, 1);
    let o = c.seats[0].order.as_mut().unwrap();
    for (dest, name) in [(7, "forest"), (8, "bear-cub"), (9, "giant-growth")] {
        let src = o
            .iter()
            .enumerate()
            .find(|(i, n)| *i >= dest && n.as_str() == name)
            .unwrap()
            .0;
        o.swap(dest, src);
    }
    game(&c)
}
#[test]
fn mulligan_rules_setup_ordered_draw_positive() {
    let mut g = green_draw();
    finish(&mut g);
    let old = handles(&g, Zone::Library(Seat::P0))[0];
    let new = g.draw_top(Seat::P0).unwrap();
    assert_ne!(old, new);
    assert_eq!(g.objects().get(new).unwrap().card.identity().key, "forest");
    assert_eq!(cards(&g, Zone::Library(Seat::P0))[0], "bear-cub");
    assert_eq!(cards(&g, Zone::Hand(Seat::P0)).last(), Some(&"forest"));
}
#[test]
fn mulligan_rules_setup_ordered_draw_negative() {
    let mut g = green_draw();
    // There is no choose-a-draw action. Submitting a card selection at the
    // opening declaration boundary is a malformed non-choice (rules.md).
    let a = bottom_action(&g, &[1]);
    reject(&mut g, Seat::P0, &a, ApplyError::WrongKind);
    let before = format!("{g:?}");
    assert_eq!(g.draw_top(Seat::P0), Err(DrawError::OpeningPending));
    assert_eq!(format!("{g:?}"), before);
    assert_eq!(
        &cards(&g, Zone::Library(Seat::P0))[..3],
        &["forest", "bear-cub", "giant-growth"]
    );
}
#[test]
fn mulligan_rules_setup_ordered_draw_regression() {
    let mut g = twice();
    bottom(&mut g, &[0, 1]);
    finish(&mut g);
    let mut drawn = Vec::new();
    for _ in 0..35 {
        let h = g.draw_top(Seat::P0).unwrap();
        drawn.push(g.objects().get(h).unwrap().card.identity().key);
    }
    assert_eq!(&drawn[33..], &["mountain", "swab-goblin"]);
    let before = format!("{g:?}");
    assert_eq!(g.draw_top(Seat::P0), Err(DrawError::EmptyLibrary));
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn mulligan_rules_decisions_stale_candidates_regression() {
    let mut g = game(&config());
    let old = action(&g, 0);
    g.reset(&config(), 42, 9).unwrap();
    reject(&mut g, Seat::P0, &old, ApplyError::StaleDecision);
    let mut mixed = old;
    mixed.decision = g.decision().unwrap().id;
    reject(&mut g, Seat::P0, &mixed, ApplyError::StaleCandidate);
    finish(&mut g);
}
#[test]
fn mulligan_sys_core_003_rejections_between_every_real_stage() {
    let mut g = game(&config());
    let mut control = game(&config());
    let mut old = None;
    for stage in 0..5 {
        let d = g.decision().unwrap();
        let wrong = if d.actor == Seat::P0 {
            Seat::P1
        } else {
            Seat::P0
        };
        let valid = if matches!(d.kind, OpeningKind::Bottom { .. }) {
            bottom_action(&g, &[0])
        } else {
            action(&g, if stage == 0 { 1 } else { 0 })
        };
        reject(&mut g, wrong, &valid, ApplyError::WrongActor);
        if let Some(ref stale) = old {
            reject(&mut g, d.actor, stale, ApplyError::StaleDecision);
        }
        let bad = OpeningAction {
            decision: d.id,
            selection: Selection::Choose(d.candidate(usize::MAX)),
        };
        if matches!(d.kind, OpeningKind::Bottom { .. }) {
            reject(&mut g, d.actor, &bad, ApplyError::WrongKind);
            let empty = bottom_action(&g, &[]);
            reject(&mut g, d.actor, &empty, ApplyError::WrongCardinality);
            let bad = bottom_action(&g, &[7]);
            reject(&mut g, d.actor, &bad, ApplyError::IllegalCandidate);
        } else {
            reject(&mut g, d.actor, &bad, ApplyError::IllegalCandidate);
            let bad = bottom_action(&g, &[]);
            reject(&mut g, d.actor, &bad, ApplyError::WrongKind);
        }
        let foreign = action(&game(&config()), 0);
        reject(&mut g, d.actor, &foreign, ApplyError::StaleDecision);
        g.apply(d.actor, &valid).unwrap();
        old = Some(valid);
        if stage == 0 {
            choose(&mut control, 1);
        } else if stage == 2 {
            bottom(&mut control, &[0]);
        } else {
            choose(&mut control, 0);
        }
        assert_eq!(
            Zone::ALL.map(|z| cards(&g, z)),
            Zone::ALL.map(|z| cards(&control, z))
        );
        if g.decision().is_none() {
            break;
        }
    }
    assert!(g.decision().is_none());
    reject(&mut g, Seat::P0, &old.unwrap(), ApplyError::NoDecision);
    let mut g = twice();
    let a = bottom_action(&g, &[0, 0]);
    reject(&mut g, Seat::P0, &a, ApplyError::DuplicateCandidate);
    bottom(&mut g, &[0, 1]);
    finish(&mut g);
}
#[test]
fn mulligan_explicit_order_validation_and_seven_limit() {
    let mut g = game(&config());
    for count in 1..=7 {
        let d = g.decision().unwrap();
        let a = action(&g, 1);
        let full = replacement(&g, d.actor);
        let mut duplicate = full.clone();
        duplicate[1] = duplicate[0];
        for order in [&full[..39], &duplicate[..]] {
            let before = format!("{g:?}");
            assert_eq!(
                g.apply_with_order(d.actor, &a, order),
                Err(ApplyError::InvalidOrder)
            );
            assert_eq!(format!("{g:?}"), before);
        }
        mulligan_ordered(&mut g);
        if count == 1 {
            choose(&mut g, 0);
        }
        assert_eq!(bottom_count(&g), count);
        bottom(&mut g, &(0..count).collect::<Vec<_>>());
        assert_eq!(cards(&g, Zone::Hand(Seat::P0)).len(), 7 - count);
    }
    assert!(g.decision().is_none()); // forced keep at zero, no eighth mulligan
    assert_eq!(cards(&g, Zone::Library(Seat::P0)).len(), 40);
}

#[test]
fn mulligan_random_replacements_match_independent_full_order_vectors() {
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("../../../doc/evidence/mulligan/vectors.json")).unwrap();
    for vector in vectors.as_array().unwrap() {
        let c = Config {
            starting_seat: vector["starter"].as_u64().unwrap() as u8,
            ..Config::default()
        };
        let mut g = game(&c);
        choose(&mut g, 1);
        choose(&mut g, 1);
        bottom(&mut g, &[6]);
        bottom(&mut g, &[6]);
        for checkpoint in ["first", "second"] {
            if checkpoint == "second" {
                choose(&mut g, 1);
                choose(&mut g, 0);
                bottom(&mut g, &[5, 1]);
            }
            for (i, s) in [Seat::P0, Seat::P1].into_iter().enumerate() {
                for (field, z) in [("hands", Zone::Hand(s)), ("libraries", Zone::Library(s))] {
                    let expected: Vec<_> = vector[checkpoint][field][i]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|c| c.as_str().unwrap())
                        .collect();
                    assert_eq!(cards(&g, z), expected, "{checkpoint}/{field}/seat{i}");
                }
            }
        }
        finish(&mut g);
    }
}

#[test]
fn mulligan_both_seats_mirrors_and_distinct_bottom_generations() {
    for deck in ["red", "green"] {
        for starter in 0..2 {
            let mut c = config();
            let d = c.seats.iter().find(|d| d.deck == deck).unwrap().clone();
            c.seats = vec![d.clone(), d];
            c.starting_seat = starter;
            let mut g = game(&c);
            let seats = if starter == 0 {
                [Seat::P0, Seat::P1]
            } else {
                [Seat::P1, Seat::P0]
            };
            let initial = Zone::ALL.map(|z| handles(&g, z));
            mulligan_ordered(&mut g);
            // First declaration does not replace either hand before the second.
            assert_eq!(Zone::ALL.map(|z| handles(&g, z)), initial);
            mulligan_ordered(&mut g);
            assert_eq!(g.decision().unwrap().actor, seats[0]);
            let old = bottom_action(&g, &[0]);
            bottom(&mut g, &[0]);
            assert_eq!(g.decision().unwrap().actor, seats[1]);
            let mut mixed = old;
            mixed.decision = g.decision().unwrap().id;
            reject(&mut g, seats[1], &mixed, ApplyError::StaleCandidate);
            bottom(&mut g, &[1]);
            for s in seats {
                assert_eq!(cards(&g, Zone::Hand(s)).len(), 6);
            }
            finish(&mut g);
        }
    }
}

#[test]
fn mulligan_rejects_foreign_stale_and_unexpected_replacement_orders() {
    let mut g = game(&config());
    let old = replacement(&g, Seat::P0);
    let foreign = replacement(&game(&config()), Seat::P0);
    g.reset(&config(), 42, 9).unwrap();
    let mut wrong_seat = replacement(&g, Seat::P0);
    wrong_seat[0] = handles(&g, Zone::Hand(Seat::P1))[0];
    let mut extra = replacement(&g, Seat::P0);
    extra.push(extra[0]);
    for order in [old, foreign, wrong_seat, extra] {
        let before = format!("{g:?}");
        let a = action(&g, 1);
        assert_eq!(
            g.apply_with_order(Seat::P0, &a, &order),
            Err(ApplyError::InvalidOrder)
        );
        assert_eq!(format!("{g:?}"), before);
    }
    let order = replacement(&g, Seat::P0);
    let a = action(&g, 0);
    let before = format!("{g:?}");
    assert_eq!(
        g.apply_with_order(Seat::P0, &a, &order),
        Err(ApplyError::UnexpectedOrder)
    );
    assert_eq!(format!("{g:?}"), before);
    mulligan_ordered(&mut g);
    choose(&mut g, 0);
    let a = bottom_action(&g, &[0]);
    let before = format!("{g:?}");
    assert_eq!(
        g.apply_with_order(Seat::P0, &a, &[]),
        Err(ApplyError::UnexpectedOrder)
    );
    assert_eq!(format!("{g:?}"), before);
    let too_many = bottom_action(&g, &[0, 1]);
    reject(&mut g, Seat::P0, &too_many, ApplyError::WrongCardinality);
    bottom(&mut g, &[0]);
    finish(&mut g);
    let mut fresh = Game::new().unwrap();
    reject(&mut fresh, Seat::P0, &a, ApplyError::NoDecision);
    let before = format!("{fresh:?}");
    assert_eq!(fresh.draw_top(Seat::P0), Err(DrawError::NotStarted));
    assert_eq!(format!("{fresh:?}"), before);
}
