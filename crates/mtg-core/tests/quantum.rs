//! RFC R0002-B015/B016: a yield is internal, resumable, and changes no choices.
use mtg_core::objects::{Seat, Zone};
use mtg_core::opening::*;
use std::num::NonZeroUsize;
fn q(n: usize) -> NonZeroUsize {
    NonZeroUsize::new(n).unwrap()
}
fn action(d: OpeningDecision, i: usize) -> OpeningAction {
    OpeningAction {
        decision: d.id,
        selection: Selection::Choose(d.candidate(i)),
    }
}
#[test]
fn quantum_reset_really_yields_before_dealing() {
    let mut g = Game::new().unwrap();
    assert_eq!(
        g.reset_quantum(&Config::default(), 42, 9, q(1)).unwrap(),
        Progress::InternalYield
    );
    assert_eq!(g.decision(), None);
    assert_eq!(g.objects().in_zone(Zone::Hand(Seat::P0)).count(), 0);
    assert_eq!(g.draw_top(Seat::P0), Err(DrawError::OpeningPending));
}
#[test]
fn quantum_mulligan_really_yields_and_resume_is_not_an_action() {
    let mut g = Game::new().unwrap();
    let d = g.reset(&Config::default(), 42, 9).unwrap();
    let d = g.apply(d.actor, &action(d, 1)).unwrap().unwrap();
    assert_eq!(
        g.apply_quantum(d.actor, &action(d, 0), None, q(1)).unwrap(),
        Progress::InternalYield
    );
    assert_eq!(g.decision(), None);
    let before = format!("{g:?}");
    assert!(g.apply(d.actor, &action(d, 0)).is_err());
    assert_eq!(format!("{g:?}"), before);
    let mut n = 0;
    loop {
        n += 1;
        assert!(n < 500);
        match g.resume(q(1)) {
            Progress::InternalYield => assert_eq!(g.decision(), None),
            Progress::Decision(next) => {
                assert_eq!(next.generation, d.generation + 1);
                assert_eq!(next.kind, OpeningKind::Bottom { count: 1 });
                break;
            }
            other => panic!("unexpected {other:?}"),
        }
    }
    let before = format!("{g:?}");
    let d = g.decision().unwrap();
    for _ in 0..3 {
        assert_eq!(g.resume(q(1)), Progress::Decision(d));
    }
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn quantum_resume_before_reset_is_not_opening_complete() {
    assert_eq!(Game::new().unwrap().resume(q(1)), Progress::NotStarted);
}

#[test]
fn quantum_explicit_ledger_and_changing_budgets() {
    // Independent CR 103.5 ledger: keep P1's seven, return P0's hand,
    // reverse the full deck as explicit chance, draw seven, bottom indices
    // 5 then (next round) 6,0. No game-generated expected cards.
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let mut config = Config::default();
    for input in &mut config.seats {
        let deck = manifest["decks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == input.deck)
            .unwrap();
        let mut order = Vec::new();
        for entry in deck["cards"].as_array().unwrap() {
            for _ in 0..entry["copies"].as_u64().unwrap() {
                order.push(entry["card_id"].as_str().unwrap().to_owned());
            }
        }
        input.order = Some(order);
    }
    let mut expected = config.seats[0].order.clone().unwrap();
    let other = config.seats[1].order.clone().unwrap();
    let mut g = Game::new().unwrap();
    let mut budget = 0;
    let drain = |g: &mut Game, mut p, budget: &mut usize| {
        while p == Progress::InternalYield {
            *budget += 1;
            p = g.resume(q([1, 7, 40, 2][*budget % 4]));
        }
        p
    };
    let p = g.reset_quantum(&config, 42, 9, q(1)).unwrap();
    drain(&mut g, p, &mut budget);
    let keys = |g: &Game, z| {
        g.objects()
            .in_zone(z)
            .map(|h| g.objects().get(h).unwrap().card.identity().key.to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&g, Zone::Hand(Seat::P0)), expected[..7]);
    for (round, indices) in [vec![5], vec![6, 0]].into_iter().enumerate() {
        let d = g.decision().unwrap();
        let mut order = g
            .objects()
            .in_zone(Zone::Hand(Seat::P0))
            .chain(g.objects().in_zone(Zone::Library(Seat::P0)))
            .collect::<Vec<_>>();
        order.reverse();
        expected.reverse();
        let p = g
            .apply_quantum(d.actor, &action(d, 1), Some(&order), q(1))
            .unwrap();
        drain(&mut g, p, &mut budget);
        if round == 0 {
            let d = g.decision().unwrap();
            let p = g.apply_quantum(d.actor, &action(d, 0), None, q(1)).unwrap();
            drain(&mut g, p, &mut budget);
        }
        assert_eq!(keys(&g, Zone::Hand(Seat::P0)), expected[..7]);
        assert_eq!(keys(&g, Zone::Library(Seat::P0)), expected[7..]);
        let d = g.decision().unwrap();
        assert_eq!(d.kind, OpeningKind::Bottom { count: round + 1 });
        let a = OpeningAction {
            decision: d.id,
            selection: Selection::Bottom(indices.iter().map(|&i| d.candidate(i)).collect()),
        };
        let p = g.apply_quantum(d.actor, &a, None, q(1)).unwrap();
        assert_eq!(p, Progress::InternalYield);
        let before = format!("{g:?}");
        assert_eq!(g.apply(d.actor, &a), Err(ApplyError::WorkPending));
        assert_eq!(format!("{g:?}"), before);
        drain(&mut g, p, &mut budget);
        let hand = expected[..7]
            .iter()
            .enumerate()
            .filter(|(i, _)| !indices.contains(i))
            .map(|(_, c)| c.clone())
            .collect::<Vec<_>>();
        let mut library = expected[7..].to_vec();
        for i in indices {
            library.push(expected[i].clone());
        }
        assert_eq!(keys(&g, Zone::Hand(Seat::P0)), hand);
        assert_eq!(keys(&g, Zone::Library(Seat::P0)), library);
        expected = hand.into_iter().chain(library).collect();
        assert_eq!(keys(&g, Zone::Hand(Seat::P1)), other[..7]);
        assert_eq!(keys(&g, Zone::Library(Seat::P1)), other[7..]);
    }
    let d = g.decision().unwrap();
    let p = g.apply_quantum(d.actor, &action(d, 0), None, q(1)).unwrap();
    assert_eq!(drain(&mut g, p, &mut budget), Progress::OpeningComplete);
    let before = format!("{g:?}");
    for _ in 0..4 {
        assert_eq!(g.resume(q(1)), Progress::OpeningComplete);
    }
    assert_eq!(format!("{g:?}"), before);
}
