use mtg_core::objects::{Seat, Zone};
use mtg_core::opening::{views::ViewError, *};
// Normal reset/mulligan path; independent expected cards from frozen manifest.
fn config(viewer: Seat, variant: bool) -> Config {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let seats = [Seat::P0, Seat::P1].map(|s| {
        let name = if s == viewer || variant {
            "red"
        } else {
            "green"
        };
        let deck = manifest["decks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == name)
            .unwrap();
        let mut order = vec![];
        for e in deck["cards"].as_array().unwrap() {
            for _ in 0..e["copies"].as_u64().unwrap() {
                order.push(e["card_id"].as_str().unwrap().to_owned());
            }
        }
        if s == viewer {
            order.swap(0, 16);
            if variant {
                order[7..].reverse();
            }
        }
        DeckConfig {
            deck: name.into(),
            order: Some(order),
        }
    });
    Config {
        seats: seats.to_vec(),
        starting_seat: if viewer == Seat::P0 { 0 } else { 1 },
        ..Config::default()
    }
}
fn game(c: &Config, seed: u64) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(c, seed, 3).unwrap();
    g
}
fn bytes(g: &Game, s: Seat) -> Vec<u8> {
    serde_json::to_vec(&g.observe(s).unwrap()).unwrap()
}
#[test]
fn views_opening_candidates_errors_and_bottoming_use_only_visible_rows() {
    for seat in [Seat::P0, Seat::P1] {
        let mut a = game(&config(seat, false), 42);
        let mut b = game(&config(seat, true), 99);
        assert_eq!(bytes(&a, seat), bytes(&b, seat));
        let view = a.observe(seat).unwrap();
        let d = view.opening.unwrap();
        assert_eq!(d.candidates, ["keep", "mulligan"]);
        let other = if seat == Seat::P0 { Seat::P1 } else { Seat::P0 };
        assert!(a.observe(other).unwrap().opening.is_none());
        for (actor, generation, rows, error) in [
            (other, d.generation, vec![0], ViewError::WrongActor),
            (seat, d.generation - 1, vec![0], ViewError::StaleDecision),
            (
                seat,
                d.generation,
                vec![usize::MAX],
                ViewError::InvalidSelection,
            ),
            (seat, d.generation, vec![0, 1], ViewError::InvalidSelection),
            (seat, d.generation, vec![], ViewError::InvalidSelection),
        ] {
            for g in [&mut a, &mut b] {
                let before = format!("{g:?}");
                assert_eq!(g.apply_opening_view(actor, generation, &rows), Err(error));
                assert_eq!(format!("{g:?}"), before);
            }
            assert_eq!(bytes(&a, seat), bytes(&b, seat));
        }
        // Inject the same own-hand replacement order through the existing
        // privileged chance hook. Opponent keeps; neither is a mock decision.
        for g in [&mut a, &mut b] {
            let d = g.decision().unwrap();
            let order = g
                .objects()
                .in_zone(Zone::Hand(seat))
                .chain(g.objects().in_zone(Zone::Library(seat)))
                .collect::<Vec<_>>();
            g.apply_with_order(
                seat,
                &OpeningAction {
                    decision: d.id,
                    selection: Selection::Choose(d.candidate(1)),
                },
                &order,
            )
            .unwrap();
            let d = g.observe(other).unwrap().opening.unwrap();
            g.apply_opening_view(other, d.generation, &[0]).unwrap();
        }
        assert_eq!(bytes(&a, seat), bytes(&b, seat));
        let d = a.observe(seat).unwrap().opening.unwrap();
        assert_eq!(d.kind, "bottom");
        assert_eq!(d.count, 1);
        assert_eq!(
            d.candidates,
            [
                "mountain",
                "mountain",
                "mountain",
                "mountain",
                "mountain",
                "mountain",
                "swab-goblin"
            ]
        );
        assert!(a.observe(other).unwrap().opening.is_none());
        for g in [&mut a, &mut b] {
            let before = format!("{g:?}");
            assert_eq!(
                g.apply_opening_view(seat, d.generation, &[7]),
                Err(ViewError::InvalidSelection)
            );
            assert_eq!(format!("{g:?}"), before);
            g.apply_opening_view(seat, d.generation, &[6]).unwrap();
            let h = g.objects().in_zone(Zone::Library(seat)).last().unwrap();
            assert_eq!(
                g.objects().get(h).unwrap().card.identity().key,
                "swab-goblin"
            );
            assert_eq!(g.observe(seat).unwrap().hand.len(), 6);
            assert_eq!(
                g.apply_opening_view(seat, d.generation, &[0]),
                Err(ViewError::StaleDecision)
            );
        }
        assert_eq!(bytes(&a, seat), bytes(&b, seat));
    }
}
#[test]
fn views_public_terminal_result_preserves_private_filter() {
    let mut a = game(&config(Seat::P0, false), 42);
    let mut b = game(&config(Seat::P0, true), 99);
    for g in [&mut a, &mut b] {
        g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
    }
    assert_eq!(bytes(&a, Seat::P0), bytes(&b, Seat::P0));
    let v = a.observe(Seat::P0).unwrap();
    assert_eq!(v.terminal.unwrap().winner, Some(0));
    assert_eq!(v.hand.len(), 7);
    assert!(v.opening.is_none());
}
#[test]
fn views_library_permutation_alone_preserves_bytes_candidates_and_errors() {
    for seat in [Seat::P0, Seat::P1] {
        for library_seat in 0..2 {
            let config_a = config(seat, false);
            let mut config_b = config_a.clone();
            config_b.seats[library_seat].order.as_mut().unwrap()[7..].reverse();
            let mut a = game(&config_a, 42);
            let mut b = game(&config_b, 42);
            assert_eq!(bytes(&a, seat), bytes(&b, seat));
            let d = a.observe(seat).unwrap().opening.unwrap();
            assert_eq!(d, b.observe(seat).unwrap().opening.unwrap());
            let ea = a.apply_opening_view(seat, d.generation, &[2]).unwrap_err();
            let eb = b.apply_opening_view(seat, d.generation, &[2]).unwrap_err();
            assert_eq!(serde_json::to_vec(&ea).unwrap(), b"\"invalid_selection\"");
            assert_eq!(
                serde_json::to_vec(&ea).unwrap(),
                serde_json::to_vec(&eb).unwrap()
            );
            assert_eq!(bytes(&a, seat), bytes(&b, seat));
        }
    }
}
