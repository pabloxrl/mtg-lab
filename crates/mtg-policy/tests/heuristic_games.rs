//! Normal reset games, no injected battlefield. CR704.5a/b independently justify
//! terminal loss predicates; policy outcomes are never used as a rules oracle.
use mtg_core::{
    game::{
        Config, DeckConfig, Game,
        policy::{Choice, Observation, Submission},
    },
    objects::Seat,
};
use mtg_policy::{HEURISTIC_VERSION, Heuristic};
const CAP: usize = 1024;
fn config(start: u8, twin: bool) -> Config {
    let mut order = [
        ("bear-cub", 2),
        ("giant-growth", 1),
        ("bite-down", 1),
        ("forest", 16),
        ("bear-cub", 2),
        ("giant-growth", 2),
        ("bite-down", 2),
        ("llanowar-elves", 3),
        ("druid-of-the-cowl", 2),
        ("magnigoth-sentry", 2),
        ("tajuru-pathwarden", 2),
        ("thornweald-archer", 3),
        ("wildheart-invoker", 2),
    ]
    .into_iter()
    .flat_map(|(s, n)| vec![s.to_owned(); n])
    .collect::<Vec<_>>();
    if twin {
        order[30..].reverse();
    }
    Config {
        starting_seat: start,
        seats: vec![
            DeckConfig {
                deck: "green".into(),
                order: Some(order)
            };
            2
        ],
        ..Config::default()
    }
}
fn active(g: &Game) -> (Seat, Observation) {
    [Seat::P0, Seat::P1]
        .into_iter()
        .find_map(|s| {
            let o = g.policy_observe(s, CAP).unwrap();
            o.decision.is_some().then_some((s, o))
        })
        .unwrap()
}
fn apply(g: &mut Game, s: Seat, a: &Submission) {
    g.apply_policy(s, a, CAP).unwrap();
    if g.decision().is_none() && g.turn_position().is_none() && g.outcome().is_none() {
        g.start_turns().unwrap();
    }
}
fn policies() -> [Heuristic; 2] {
    [0, 1].map(|s| Heuristic::new(HEURISTIC_VERSION, s).unwrap())
}
fn run(start: u8, shuffled: bool, limit: usize) -> (Vec<Submission>, Observation, [usize; 2]) {
    let mut g = Game::new().unwrap();
    let mut c = config(start, false);
    if shuffled {
        for d in &mut c.seats {
            d.order = None;
        }
    }
    g.reset(&c, 42, 7).unwrap();
    let ps = policies();
    let mut trace = vec![];
    let mut counts = [0; 2];
    for _ in 0..limit {
        if g.outcome().is_some() {
            break;
        }
        let (s, o) = active(&g);
        let i = usize::from(o.view.seat);
        let before = serde_json::to_value(&g).unwrap();
        let a = ps[i].choose(&o).unwrap();
        assert_eq!(ps[i].choose(&o).unwrap(), a);
        assert_eq!(serde_json::to_value(&g).unwrap(), before);
        apply(&mut g, s, &a);
        trace.push(a);
        counts[i] += 1;
    }
    (trace, g.policy_observe(Seat::P0, CAP).unwrap(), counts)
}
#[test]
fn heuristic_complete_games_both_seats_repeat_and_account_terminal() {
    for start in [0, 1] {
        for shuffled in [false, true] {
            let a = run(start, shuffled, 20000);
            let b = run(start, shuffled, 20000);
            assert_eq!(
                a, b,
                "full submissions and final authorized state reproduce"
            );
            assert!(a.2.iter().all(|n| *n > 0));
            let t =
                a.1.view
                    .terminal
                    .as_ref()
                    .expect("normal-reset game must really finish");
            let mut losers = vec![];
            for (i, reason) in t.losses.iter().enumerate() {
                match reason {
                    Some("life") => {
                        assert!(a.1.view.life[i] <= 0);
                        losers.push(i);
                    }
                    Some("empty_draw") => {
                        assert_eq!(a.1.view.library_counts[i], 0);
                        let (_, actor, step) = a.1.view.turn.unwrap();
                        assert_eq!(usize::from(actor), i);
                        assert_eq!(step, "draw");
                        losers.push(i);
                    }
                    None => (),
                    _ => panic!("unexpected loss {reason:?}"),
                }
            }
            assert!(!losers.is_empty());
            assert_eq!(
                t.winner,
                if losers.len() == 1 {
                    Some((1 - losers[0]) as u8)
                } else {
                    None
                }
            );
            for predicate in [
                |c: &Choice| matches!(c, Choice::PlayLand { .. }),
                |c: &Choice| matches!(c, Choice::Cast { .. }),
                |c: &Choice| matches!(c,Choice::SelectAttackers{cards} if !cards.is_empty()),
            ] {
                assert!(a.0.iter().flat_map(|s| &s.choices).any(predicate));
            }
            println!(
                "start={start} shuffled={shuffled} decisions={:?} terminal={t:?}",
                a.2
            );
        }
    }
    for limit in [0, 1, 20] {
        let r = run(0, false, limit);
        assert_eq!(r.0.len(), limit);
        assert!(
            r.1.view.terminal.is_none(),
            "external limits are truncations, not draws"
        );
    }
}
#[test]
fn heuristic_hidden_library_twins_through_real_payment_and_combat() {
    for start in [0, 1] {
        let mut games = [Game::new().unwrap(), Game::new().unwrap()];
        for (i, g) in games.iter_mut().enumerate() {
            g.reset(&config(start, i == 1), 42, 7).unwrap();
        }
        let ps = policies();
        let mut families = std::collections::BTreeSet::new();
        for _ in 0..2000 {
            if games[0].outcome().is_some() {
                break;
            }
            // Stop before any deliberately changed library suffix could be drawn.
            if games[0].turn_position().is_some_and(|p| p.0 > 12) {
                break;
            }
            let (s, a) = active(&games[0]);
            let (_, b) = active(&games[1]);
            assert_eq!(a, b);
            families.insert(a.decision.as_ref().unwrap().kind);
            let p = &ps[usize::from(a.view.seat)];
            let action = p.choose(&a).unwrap();
            assert_eq!(action, p.choose(&b).unwrap());
            for g in &mut games {
                apply(g, s, &action);
            }
        }
        for kind in [
            "priority",
            "payment",
            "growth_target",
            "bite_source",
            "bite_destination",
            "attackers",
            "blockers",
        ] {
            assert!(families.contains(kind), "missing {kind}: {families:?}");
        }
    }
}
#[test]
fn heuristic_opponent_hand_twins_at_normal_opening() {
    for start in [0, 1] {
        let a = config(start, false);
        let mut b = a.clone();
        b.seats[usize::from(1 - start)]
            .order
            .as_mut()
            .unwrap()
            .swap(2, 39);
        let mut ga = Game::new().unwrap();
        let mut gb = Game::new().unwrap();
        ga.reset(&a, 0, 0).unwrap();
        gb.reset(&b, 0, 0).unwrap();
        let (seat, oa) = active(&ga);
        let (_, ob) = active(&gb);
        assert_eq!(oa, ob);
        let other = if seat == Seat::P0 { Seat::P1 } else { Seat::P0 };
        assert_ne!(
            ga.policy_observe(other, CAP).unwrap().view.hand,
            gb.policy_observe(other, CAP).unwrap().view.hand
        );
        let p = Heuristic::new(HEURISTIC_VERSION, start).unwrap();
        assert_eq!(p.choose(&oa), p.choose(&ob));
    }
}
