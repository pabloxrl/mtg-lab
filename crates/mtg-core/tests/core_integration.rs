//! GH-17: independently specified count ledger across real core components.
use mtg_core::objects::{Seat, Zone};
use mtg_core::opening::*;
use std::num::NonZeroUsize;

fn q(n: usize) -> NonZeroUsize {
    NonZeroUsize::new(n).unwrap()
}
fn settle(g: &mut Game, mut p: Progress, budget: usize) {
    for _ in 0..1000 {
        if p != Progress::InternalYield {
            return;
        }
        assert!(g.decision().is_none());
        p = g.resume(q(budget));
    }
    panic!("opening work exceeded bounded test budget");
}

#[test]
fn core_integration_reference_count_ledger_and_stale_reset() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/opening-counts.json"
    ))
    .unwrap();
    let mut g = Game::new().unwrap();
    let mut old = None;
    for case in fixture["cases"].as_array().unwrap() {
        for budget in [1, 7, 10000] {
            let starter = case["starter"].as_u64().unwrap() as u8;
            let rounds = case["rounds"].as_u64().unwrap();
            let config = Config {
                starting_seat: starter,
                ..Config::default()
            };
            let p = g.reset_quantum(&config, 42, 9, q(budget)).unwrap();
            settle(&mut g, p, budget);
            if let Some((actor, ref action, handle)) = old {
                let before = format!("{g:?}");
                assert!(g.apply(actor, action).is_err());
                assert!(g.objects().get(handle).is_err());
                assert_eq!(format!("{g:?}"), before);
            }
            let first = g.decision().unwrap();
            old = Some((
                first.actor,
                OpeningAction {
                    decision: first.id,
                    selection: Selection::Choose(first.candidate(0)),
                },
                g.objects().in_zone(Zone::Hand(first.actor)).next().unwrap(),
            ));
            let mut declarations = Vec::new();
            let mut taken = 0;
            let mut bottoms = 0;
            for _ in 0..30 {
                let Some(d) = g.decision() else { break };
                let seat = if d.actor == Seat::P0 { 0 } else { 1 };
                let selection = match d.kind {
                    OpeningKind::KeepOrMulligan => {
                        declarations.push(vec![
                            seat,
                            g.objects().in_zone(Zone::Hand(d.actor)).count() as u64,
                            g.objects().in_zone(Zone::Library(d.actor)).count() as u64,
                        ]);
                        let mulligan = seat == u64::from(starter) && taken < rounds;
                        taken += u64::from(mulligan);
                        Selection::Choose(d.candidate(usize::from(mulligan)))
                    }
                    OpeningKind::Bottom { count } => {
                        bottoms += count;
                        Selection::Bottom((0..count).map(|i| d.candidate(i)).collect())
                    }
                };
                let a = OpeningAction {
                    decision: d.id,
                    selection,
                };
                // Every continuation rejects the opposite actor before mutation.
                let wrong = if d.actor == Seat::P0 {
                    Seat::P1
                } else {
                    Seat::P0
                };
                let before = format!("{g:?}");
                assert_eq!(g.apply(wrong, &a), Err(ApplyError::WrongActor));
                assert_eq!(format!("{g:?}"), before);
                let p = g.apply_quantum(d.actor, &a, None, q(budget)).unwrap();
                settle(&mut g, p, budget);
            }
            assert!(g.decision().is_none());
            assert_eq!(g.resume(q(budget)), Progress::OpeningComplete);
            let counts = |zone: fn(Seat) -> Zone| {
                [Seat::P0, Seat::P1].map(|seat| g.objects().in_zone(zone(seat)).count())
            };
            assert_eq!(
                serde_json::json!({
                    "declarations": declarations, "hand": counts(Zone::Hand),
                    "library": counts(Zone::Library), "bottom_choices": bottoms
                }),
                case["expected"],
                "{} quantum {budget}",
                case["id"]
            );
            assert_eq!(g.life(), [20, 20]);
        }
    }
}
