//! Synthetic boundary positions; expectations from CR 103.8a, 502.3, 504.1,
//! 514.1–3 and 704.5b, plus frozen vanilla 2/2 characteristics.
use super::super::targets::Modification;
use super::*;
fn q(n: usize) -> NonZeroUsize {
    NonZeroUsize::new(n).unwrap()
}
fn ready(step: Step, active: Seat, turn: u64) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(
        &Config {
            starting_seat: seat_index(if turn % 2 == 1 {
                active
            } else {
                opponent(active)
            }) as u8,
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
    g.turns.position = Some((turn, active, step));
    g.set_turn_decision(opponent(active), TurnKind::Priority);
    g.turns.passed = true;
    g.turns.mana = [[1; 6]; 2];
    g
}
fn action(g: &Game) -> TurnAction {
    let d = g.turn_decision().unwrap();
    TurnAction {
        decision: d.id,
        selection: match d.kind {
            TurnKind::Discard { count } => {
                TurnSelection::Discard((0..count).rev().map(|i| d.candidate(i)).collect())
            }
            _ => TurnSelection::Pass(d.candidate(0)),
        },
    }
}
fn state(g: &Game) -> serde_json::Value {
    fn normalize(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, v) in m {
                    if k == "scope" || k == "store" {
                        *v = 0.into();
                    } else {
                        normalize(v);
                    }
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    normalize(v);
                }
            }
            _ => (),
        }
    }
    let mut v = serde_json::to_value(g).unwrap();
    normalize(&mut v);
    v["objects"]["id"] = 0.into();
    v
}
fn drain(g: &mut Game, mut p: Progress, budget: usize) -> usize {
    let mut yields = 0;
    while p == Progress::InternalYield {
        yields += 1;
        assert!(yields < 200);
        assert!(g.turn_decision().is_none());
        assert!(g.decision().is_none());
        assert!(g.outcome().is_none());
        for seat in [Seat::P0, Seat::P1] {
            assert!(g.observe(seat).is_err());
        }
        let before = g.snapshot();
        assert_eq!(g.draw_top(Seat::P0), Err(DrawError::OpeningPending));
        assert_eq!(
            g.concede(Seat::P0, g.episode_id().unwrap()),
            Err(super::super::terminal::ConcedeError::SettlementPending)
        );
        assert!(g.reset(&Config::default(), 999, 1).is_err());
        assert!(g.start_turns_quantum(q(budget)).is_err());
        assert_eq!(g.snapshot(), before);
        let mut restored = Game::new().unwrap();
        restored.restore(&before).unwrap();
        p = g.resume(q(budget));
        restored.resume(q(budget));
        assert_eq!(state(g), state(&restored));
    }
    let before = g.snapshot();
    assert_eq!(g.resume(q(1)), p);
    assert_eq!(g.snapshot(), before);
    yields
}
fn run(g: &mut Game, budget: usize) -> usize {
    let d = g.turn_decision().unwrap();
    let a = action(g);
    let generation = g.generation;
    let rng = serde_json::to_value(&g.rng).unwrap();
    let p = g.apply_turn_quantum(d.actor, &a, q(budget)).unwrap();
    if budget == 1 {
        assert_eq!(p, Progress::InternalYield);
    }
    if p == Progress::InternalYield {
        let before = g.snapshot();
        assert_eq!(g.apply_turn(d.actor, &a), Err(TurnError::NotReady));
        assert_eq!(g.snapshot(), before);
    }
    let yields = drain(g, p, budget);
    assert_eq!(g.generation, generation + 1);
    assert_eq!(serde_json::to_value(&g.rng).unwrap(), rng);
    yields
}
#[test]
fn turn_quantum_draw_skip_order_and_empty_terminal() {
    for active in [Seat::P0, Seat::P1] {
        for (turn, empty) in [(1, false), (2, false), (2, true)] {
            let mut g = ready(Step::Upkeep, active, turn);
            if empty {
                let hs: Vec<_> = g.objects.in_zone(Zone::Library(active)).collect();
                for h in hs {
                    g.objects.remove(h).unwrap();
                }
            }
            let top = g
                .objects
                .in_zone(Zone::Library(active))
                .next()
                .map(|h| g.objects.get(h).unwrap().card);
            let start = g.snapshot();
            let mut scalar = Game::new().unwrap();
            scalar.restore(&start).unwrap();
            scalar
                .apply_turn(scalar.turn_decision().unwrap().actor, &action(&scalar))
                .unwrap();
            for budget in [1, 2, 3, 4, 64] {
                let mut actual = Game::new().unwrap();
                actual.restore(&start).unwrap();
                run(&mut actual, budget);
                assert_eq!(
                    actual.turn_position(),
                    Some((
                        turn,
                        active,
                        if turn == 1 {
                            Step::PrecombatMain
                        } else {
                            Step::Draw
                        }
                    ))
                );
                assert_eq!(actual.mana(), [[0; 6]; 2]);
                assert_eq!(
                    actual.objects.in_zone(Zone::Hand(active)).count(),
                    if turn == 2 && !empty { 8 } else { 7 }
                );
                if empty {
                    assert_eq!(actual.outcome().unwrap().winner, Some(opponent(active)));
                    assert_eq!(
                        actual.outcome().unwrap().losses[seat_index(active)],
                        Some(super::super::terminal::LossReason::EmptyDraw)
                    );
                } else {
                    assert_eq!(actual.turn_decision().unwrap().actor, active);
                    if turn == 2 {
                        let h = actual.objects.in_zone(Zone::Hand(active)).last().unwrap();
                        assert_eq!(Some(actual.objects.get(h).unwrap().card), top);
                    }
                }
                assert_eq!(state(&actual), state(&scalar));
            }
        }
    }
}
#[test]
fn turn_quantum_cleanup_discard_and_simultaneous_expiration() {
    for active in [Seat::P0, Seat::P1] {
        for extra in [0, 2] {
            let mut g = ready(Step::End, active, 3);
            for _ in 0..extra {
                g.draw_top(active).unwrap();
            }
            for seat in [active, opponent(active)] {
                let h = g
                    .objects
                    .allocate(
                        CardId::from_key("bear-cub").unwrap(),
                        seat,
                        Zone::Battlefield,
                    )
                    .unwrap();
                g.objects.get_mut(h).unwrap().tapped = true;
                g.turns.sick.push(h);
                g.turns.modifications.push(Modification {
                    handle: h,
                    boost: 3,
                    damage: 4,
                });
            }
            let start = g.snapshot();
            for budget in [1, 2, 3, 4, 5, 8, 64] {
                let mut actual = Game::new().unwrap();
                actual.restore(&start).unwrap();
                run(&mut actual, budget);
                if extra > 0 {
                    assert_eq!(actual.turn_position(), Some((3, active, Step::Cleanup)));
                    assert_eq!(
                        actual.turn_decision().unwrap().kind,
                        TurnKind::Discard { count: 2 }
                    );
                    for h in actual.objects.in_zone(Zone::Battlefield) {
                        let c = actual.creature_state(h).unwrap();
                        assert_eq!((c.power, c.toughness, c.damage), (5, 5, 4));
                    }
                    let chosen: Vec<_> = actual.discard_cards().unwrap()[..2]
                        .iter()
                        .map(|h| actual.objects.get(*h).unwrap().card)
                        .rev()
                        .collect();
                    run(&mut actual, budget);
                    let discarded: Vec<_> = actual
                        .objects
                        .in_zone(Zone::Graveyard(active))
                        .map(|h| actual.objects.get(h).unwrap().card)
                        .collect();
                    assert_eq!(discarded, chosen);
                }
                assert_eq!(actual.objects.in_zone(Zone::Hand(active)).count(), 7);
                assert_eq!(
                    actual.turn_position(),
                    Some((4, opponent(active), Step::Upkeep))
                );
                assert_eq!(actual.objects.in_zone(Zone::Battlefield).count(), 2);
                for h in actual.objects.in_zone(Zone::Battlefield) {
                    let o = actual.objects.get(h).unwrap();
                    let c = actual.creature_state(h).unwrap();
                    assert_eq!((c.power, c.toughness, c.damage), (2, 2, 0));
                    assert_eq!(o.tapped, o.controller == active);
                    assert_eq!(actual.turns.sick.contains(&h), o.controller == active);
                }
                let mut scalar = Game::new().unwrap();
                scalar.restore(&start).unwrap();
                scalar
                    .apply_turn(scalar.turn_decision().unwrap().actor, &action(&scalar))
                    .unwrap();
                if extra > 0 {
                    scalar.apply_turn(active, &action(&scalar)).unwrap();
                }
                assert_eq!(state(&actual), state(&scalar));
            }
        }
    }
}
#[test]
fn turn_quantum_priority_in_cleanup_repeats_discard_before_next_turn() {
    // Synthetic CR 514.3a cleanup-priority boundary. If an effect increased the
    // hand during that window, two empty-stack passes require another cleanup.
    for budget in [1, 2, 3, 64] {
        for extra in [0, 1] {
            let mut g = ready(Step::Cleanup, Seat::P0, 3);
            for _ in 0..extra {
                g.draw_top(Seat::P0).unwrap();
            }
            let h = g
                .objects
                .allocate(
                    CardId::from_key("bear-cub").unwrap(),
                    Seat::P0,
                    Zone::Battlefield,
                )
                .unwrap();
            g.turns.modifications.push(Modification {
                handle: h,
                boost: 3,
                damage: 4,
            });
            run(&mut g, budget);
            if extra > 0 {
                assert_eq!(g.turn_position(), Some((3, Seat::P0, Step::Cleanup)));
                assert_eq!(
                    g.turn_decision().unwrap().kind,
                    TurnKind::Discard { count: 1 }
                );
                run(&mut g, budget);
            }
            assert_eq!(g.turn_position(), Some((4, Seat::P1, Step::Upkeep)));
            assert_eq!(g.objects.in_zone(Zone::Hand(Seat::P0)).count(), 7);
            let c = g.creature_state(h).unwrap();
            assert_eq!((c.toughness, c.damage), (2, 0));
        }
    }
}

#[test]
fn turn_quantum_initial_untap_unit_bound_and_controller() {
    for active in [Seat::P0, Seat::P1] {
        let mut g = ready(Step::End, active, 1);
        g.turns = TurnState::default();
        let a = g
            .objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                opponent(active),
                Zone::Battlefield,
            )
            .unwrap();
        let b = g
            .objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                active,
                Zone::Battlefield,
            )
            .unwrap();
        g.objects.get_mut(a).unwrap().controller = active;
        g.objects.get_mut(b).unwrap().controller = opponent(active);
        for h in [a, b] {
            g.objects.get_mut(h).unwrap().tapped = true;
            g.turns.sick.push(h);
        }
        let start = g.snapshot();
        for budget in [1, 2, 3, 4, 64] {
            let mut actual = Game::new().unwrap();
            actual.restore(&start).unwrap();
            let p = actual.start_turns_quantum(q(budget)).unwrap();
            // One boundary + one sickness removal + one permanent untap + publication.
            let yields = drain(&mut actual, p, budget);
            assert_eq!(yields, 3 / budget);
            assert_eq!(actual.turn_position(), Some((1, active, Step::Upkeep)));
            assert_eq!(actual.turn_decision().unwrap().actor, active);
            for h in actual.objects.in_zone(Zone::Battlefield) {
                let o = actual.objects.get(h).unwrap();
                assert_eq!(o.tapped, o.controller == opponent(active));
                assert_eq!(
                    actual.turns.sick.contains(&h),
                    o.controller == opponent(active)
                );
            }
            let mut scalar = Game::new().unwrap();
            scalar.restore(&start).unwrap();
            scalar.start_turns().unwrap();
            assert_eq!(state(&actual), state(&scalar));
        }
    }
}
#[test]
fn turn_quantum_literal_intermediate_cleanup_and_draw_units() {
    let mut g = ready(Step::End, Seat::P0, 3);
    for _ in 0..2 {
        let h = g
            .objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        g.turns.modifications.push(Modification {
            handle: h,
            boost: 3,
            damage: 4,
        });
    }
    let a = action(&g);
    let p = g.apply_turn_quantum(Seat::P1, &a, q(1)).unwrap();
    assert_eq!(p, Progress::InternalYield);
    assert_eq!(g.turn_position(), Some((3, Seat::P0, Step::Cleanup)));
    assert_eq!(g.turns.modifications.len(), 2);
    // CR 514.2: a processed creature loses damage AND boost. The other is
    // still 5/5 with four marks; no priority/SBA boundary between the two.
    assert_eq!(g.resume(q(1)), Progress::InternalYield);
    assert_eq!(g.turns.modifications.len(), 1);
    let states: Vec<_> = g
        .objects
        .in_zone(Zone::Battlefield)
        .map(|h| g.creature_state(h).unwrap())
        .collect();
    assert_eq!((states[0].toughness, states[0].damage), (5, 4));
    assert_eq!((states[1].toughness, states[1].damage), (2, 0));
    assert!(g.turn_decision().is_none());
    assert_eq!(g.resume(q(1)), Progress::InternalYield);
    assert!(g.turns.modifications.is_empty());
    assert_eq!(g.turn_position(), Some((3, Seat::P0, Step::Cleanup)));
    assert_eq!(g.resume(q(1)), Progress::InternalYield);
    assert_eq!(g.turn_position(), Some((4, Seat::P1, Step::Upkeep)));
    assert!(matches!(g.resume(q(1)), Progress::TurnDecision(_)));
    assert_eq!(g.objects.in_zone(Zone::Battlefield).count(), 2);
    let mut g = ready(Step::Upkeep, Seat::P1, 2);
    let a = action(&g);
    assert_eq!(
        g.apply_turn_quantum(Seat::P0, &a, q(1)).unwrap(),
        Progress::InternalYield
    );
    assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::Draw)));
    assert_eq!(g.objects.in_zone(Zone::Hand(Seat::P1)).count(), 7);
    assert_eq!(g.resume(q(1)), Progress::InternalYield);
    assert_eq!(g.objects.in_zone(Zone::Hand(Seat::P1)).count(), 8);
    assert!(g.turn_decision().is_none());
    assert!(matches!(g.resume(q(1)), Progress::TurnDecision(_)));
}
#[test]
fn turn_quantum_each_empty_combat_step_and_rejections() {
    let g = ready(Step::Upkeep, Seat::P0, 1);
    let start = g.snapshot();
    for budget in [1, 2, 3, 64] {
        let mut g = Game::new().unwrap();
        g.restore(&start).unwrap();
        for step in [
            Step::PrecombatMain,
            Step::BeginningCombat,
            Step::DeclareAttackers,
            Step::EndCombat,
            Step::PostcombatMain,
            Step::End,
            Step::Upkeep,
        ] {
            let d = g.turn_decision().unwrap();
            let a = action(&g);
            let before = g.snapshot();
            assert_eq!(
                g.apply_turn_quantum(opponent(d.actor), &a, q(budget)),
                Err(TurnError::Invalid(ApplyError::WrongActor))
            );
            assert_eq!(g.snapshot(), before);
            run(&mut g, budget);
            assert_eq!(g.turn_position().unwrap().2, step);
            let d = g.turn_decision().unwrap();
            let a = action(&g);
            let p = g.apply_turn_quantum(d.actor, &a, q(budget)).unwrap();
            drain(&mut g, p, budget);
        }
    }
}

#[test]
fn turn_quantum_shared_xmage_draw_checkpoints() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/terminal.json")).unwrap();
    for c in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["action"] == "draw")
    {
        for budget in [1, 2, 3, 64] {
            let mut g = ready(Step::Upkeep, Seat::P0, 2);
            g.objects.reset().unwrap();
            if c.get("library_card").is_some() {
                g.objects
                    .allocate(
                        CardId::from_key("forest").unwrap(),
                        Seat::P0,
                        Zone::Library(Seat::P0),
                    )
                    .unwrap();
            }
            run(&mut g, budget);
            let lost = g
                .outcome()
                .map(|o| o.losses.map(|l| l.is_some()))
                .unwrap_or([false; 2]);
            let library = [Seat::P0, Seat::P1].map(|s| g.objects.in_zone(Zone::Library(s)).count());
            let hand = [Seat::P0, Seat::P1].map(|s| g.objects.in_zone(Zone::Hand(s)).count());
            assert_eq!(
                serde_json::json!({"life":g.life(),"lost":lost,"library":library,"hand":hand}),
                c["expected"]
            );
        }
    }
}
