//! Original synthetic edge positions; CR 104.3a, 104.4a, 704.5a/b/g.
//! Normal-reset complete games live in tests/terminal.rs.
use super::*;
use crate::opening::turns::{Step, TurnAction, TurnSelection};
fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 9).unwrap();
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
    g
}
fn pass(g: &mut Game) {
    let d = g.turn_decision().unwrap();
    g.apply_turn(
        d.actor,
        &TurnAction {
            decision: d.id,
            selection: TurnSelection::Pass(d.candidate(0)),
        },
    )
    .unwrap();
}
fn cub(g: &mut Game, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key("bear-cub").unwrap(), seat, zone)
        .unwrap()
}
fn damage_position(life: i64, block: bool) -> (Game, Handle) {
    let mut g = ready();
    g.turns.position = Some((3, Seat::P0, Step::BeginningCombat));
    g.life = [20, life];
    let a = cub(&mut g, Seat::P0, Zone::Battlefield);
    let b = block.then(|| cub(&mut g, Seat::P1, Zone::Battlefield));
    pass(&mut g);
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    let d = g.select_attackers(d.actor, d.id, &[a]).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
    pass(&mut g);
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    let blocks = b.map(|h| vec![(h, a)]).unwrap_or_default();
    let d = g.select_blockers(d.actor, d.id, &blocks).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
    pass(&mut g);
    pass(&mut g);
    (g, a)
}
#[test]
fn terminal_lethal_zero_negative_and_creature_death_is_not_player_loss() {
    for (life, block, expected_life, ended) in [
        (2, false, 0, true),
        (1, false, -1, true),
        (3, false, 1, false),
        (20, true, 20, false),
    ] {
        let (mut g, a) = damage_position(life, block);
        let d = g.turn_decision().unwrap();
        g.finish_combat(d.actor, d.id).unwrap();
        assert_eq!(g.life(), [20, expected_life]);
        assert_eq!(
            g.turn_decision().is_none(),
            ended,
            "CR 704.5a: lethal ends before priority"
        );
        if ended {
            assert_eq!(
                g.outcome(),
                Some(Outcome {
                    winner: Some(Seat::P0),
                    losses: [None, Some(LossReason::Life)]
                })
            );
        } else {
            assert_eq!(g.outcome(), None);
        }
        if block {
            assert!(g.objects.get(a).is_err());
            assert_eq!(g.objects.in_zone(Zone::Battlefield).count(), 0);
        }
    }
}
#[test]
fn terminal_empty_draw_versus_empty_library_and_first_draw_skip() {
    for turn in [1, 3] {
        let mut g = ready();
        g.turns.position = Some((turn, Seat::P0, Step::Upkeep));
        let cards: Vec<_> = g.objects.in_zone(Zone::Library(Seat::P0)).collect();
        for h in cards {
            g.objects.remove(h).unwrap();
        }
        pass(&mut g);
        pass(&mut g);
        if turn == 1 {
            assert_eq!(g.outcome(), None);
            assert_eq!(g.turn_position().unwrap().2, Step::PrecombatMain);
            pass(&mut g);
            assert_eq!(g.outcome(), None);
        } else {
            assert_eq!(
                g.outcome(),
                Some(Outcome {
                    winner: Some(Seat::P1),
                    losses: [Some(LossReason::EmptyDraw), None]
                })
            );
            assert!(g.turn_decision().is_none());
        }
    }
}
#[test]
fn terminal_concede_nonacting_seat_and_finality() {
    let mut g = ready();
    let id = g.episode_id().unwrap();
    let d = g.turn_decision().unwrap();
    let result = Outcome {
        winner: Some(Seat::P0),
        losses: [None, Some(LossReason::Concession)],
    };
    assert_eq!(g.concede(Seat::P1, id), Ok(result));
    let before = format!("{g:?}");
    assert_eq!(g.concede(Seat::P0, id), Err(ConcedeError::AlreadyEnded));
    assert!(
        g.apply_turn(
            d.actor,
            &TurnAction {
                decision: d.id,
                selection: TurnSelection::Pass(d.candidate(0))
            }
        )
        .is_err()
    );
    assert!(g.draw_top(Seat::P0).is_err());
    assert!(g.start_turns().is_err());
    assert_eq!(g.outcome(), Some(result));
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn terminal_concession_cancels_payment_without_commit() {
    use crate::opening::mana::{Color, ManaCost};
    let mut g = ready();
    g.turns.mana[0][4] = 2;
    let d = g.turn_decision().unwrap();
    let p = g
        .begin_payment(
            Seat::P0,
            d.id,
            ManaCost {
                colored: [0; 6],
                generic: 1,
            },
        )
        .unwrap();
    g.concede(Seat::P0, g.episode_id().unwrap()).unwrap();
    let before = format!("{g:?}");
    assert!(g.choose_payment(Seat::P0, p.id, Color::Green).is_err());
    assert!(g.finish_payment(Seat::P0, p.id).is_err());
    assert!(g.cancel_payment(Seat::P0, p.id).is_err());
    assert_eq!(g.mana()[0][4], 2);
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn terminal_hand_cub_cannot_block() {
    let mut g = ready();
    g.turns.position = Some((3, Seat::P0, Step::BeginningCombat));
    let a = cub(&mut g, Seat::P0, Zone::Battlefield);
    let b = cub(&mut g, Seat::P1, Zone::Hand(Seat::P1));
    pass(&mut g);
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    let d = g.select_attackers(d.actor, d.id, &[a]).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
    pass(&mut g);
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.select_blockers(d.actor, d.id, &[(b, a)]),
        Err(crate::opening::combat::CombatError::IllegalBlocker)
    );
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn terminal_simultaneous_losses_and_external_work_limits() {
    // Synthetic simultaneous SBA position; current cards cannot damage both seats.
    let mut g = ready();
    g.life = [0, -2];
    assert_eq!(
        g.settle_terminal(None),
        Some(Outcome {
            winner: None,
            losses: [Some(LossReason::Life); 2]
        })
    );
    let before = format!("{g:?}");
    assert_eq!(g.settle_terminal(None), g.outcome());
    assert_eq!(format!("{g:?}"), before);
    let mut g = Game::new().unwrap();
    assert_eq!(
        g.reset_quantum(&Config::default(), 42, 9, NonZeroUsize::new(1).unwrap())
            .unwrap(),
        Progress::InternalYield
    );
    assert_eq!(g.outcome(), None); // work budget is never a rules draw
    let id = g.episode_id().unwrap();
    let result = g.concede(Seat::P0, id).unwrap();
    let before = format!("{g:?}");
    for q in [1, 100, usize::MAX] {
        assert_eq!(
            g.resume(NonZeroUsize::new(q).unwrap()),
            Progress::Terminal(result)
        );
    }
    assert_eq!(format!("{g:?}"), before);
    g.reset(&Config::default(), 42, 9).unwrap();
    assert_eq!(g.outcome(), None);
    let before = format!("{g:?}");
    assert_eq!(g.concede(Seat::P0, id), Err(ConcedeError::StaleEpisode));
    assert_eq!(format!("{g:?}"), before);
    let foreign = ready().episode_id().unwrap();
    assert_eq!(
        g.concede(Seat::P0, foreign),
        Err(ConcedeError::StaleEpisode)
    );
}
fn add(g: &mut Game, key: &str, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, zone)
        .unwrap()
}
fn spell(g: &mut Game, seat: Seat, key: &str, targets: &[Handle]) {
    use crate::opening::mana::Color;
    let h = add(g, key, seat, Zone::Hand(seat));
    let d = g.turn_decision().unwrap();
    assert_eq!(d.actor, seat);
    let mut t = g.begin_targeted_cast(seat, d.id, h, 80).unwrap();
    for &h in targets {
        t = g.choose_target(seat, t.id, h).unwrap();
    }
    let mut p = g.finish_targets(seat, t.id).unwrap();
    p = g.choose_payment(seat, p.id, Color::Green).unwrap();
    if key == "bite-down" {
        p = g.choose_payment(seat, p.id, Color::Green).unwrap();
    }
    g.finish_cast(seat, p.id).unwrap();
}
#[test]
fn terminal_grown_dead_cub_then_real_cast_has_fresh_bookkeeping() {
    use crate::opening::mana::Color;
    use crate::opening::targets::CreatureState;
    let mut g = ready();
    g.turns.position = Some((3, Seat::P0, Step::PrecombatMain));
    g.turns.mana = [[0, 0, 0, 0, 20, 0]; 2];
    let old = cub(&mut g, Seat::P0, Zone::Battlefield);
    let enemy = cub(&mut g, Seat::P1, Zone::Battlefield);
    spell(&mut g, Seat::P0, "giant-growth", &[old]);
    pass(&mut g);
    pass(&mut g);
    assert_eq!(
        g.creature_state(old),
        Some(CreatureState {
            power: 5,
            toughness: 5,
            damage: 0
        })
    );
    for _ in 0..3 {
        pass(&mut g);
        spell(&mut g, Seat::P1, "bite-down", &[enemy, old]);
        pass(&mut g);
        pass(&mut g);
    }
    assert!(g.objects.get(old).is_err());
    assert_eq!(g.outcome(), None);
    let h = cub(&mut g, Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    let mut p = g.begin_cast(Seat::P0, d.id, h).unwrap();
    for _ in 0..2 {
        p = g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
    }
    g.finish_cast(Seat::P0, p.id).unwrap();
    pass(&mut g);
    pass(&mut g);
    let new = g
        .objects
        .in_zone(Zone::Battlefield)
        .find(|h| g.objects.get(*h).unwrap().owner == Seat::P0)
        .unwrap();
    assert_ne!(new, old);
    assert_eq!(
        g.creature_state(new),
        Some(CreatureState {
            power: 2,
            toughness: 2,
            damage: 0
        })
    );
}
#[test]
fn terminal_concession_preserves_stack_and_invalidates_target_and_cast_choices() {
    use crate::opening::mana::Color;
    for stage in [0, 1, 2] {
        let mut g = ready();
        g.turns.position = Some((3, Seat::P0, Step::PrecombatMain));
        g.turns.mana[0][4] = 2;
        let a = cub(&mut g, Seat::P0, Zone::Battlefield);
        let h = add(&mut g, "giant-growth", Seat::P0, Zone::Hand(Seat::P0));
        let d = g.turn_decision().unwrap();
        let t = g.begin_targeted_cast(Seat::P0, d.id, h, 80).unwrap();
        let mut payment = None;
        if stage > 0 {
            let t = g.choose_target(Seat::P0, t.id, a).unwrap();
            let p = g.finish_targets(Seat::P0, t.id).unwrap();
            let p = g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
            payment = Some(p.clone());
            if stage == 2 {
                g.finish_cast(Seat::P0, p.id).unwrap();
            }
        }
        let objects = format!("{:?}", g.objects);
        let life = g.life();
        let mana = g.mana();
        g.concede(Seat::P1, g.episode_id().unwrap()).unwrap(); // nonacting opponent, even inside choices
        let before = format!("{g:?}");
        assert!(g.choose_target(Seat::P0, t.id, a).is_err());
        assert!(g.finish_targets(Seat::P0, t.id).is_err());
        assert!(g.cancel_targets(Seat::P0, t.id).is_err());
        if let Some(p) = payment {
            assert!(g.finish_cast(Seat::P0, p.id).is_err());
            assert!(g.cast_tap_mana(Seat::P0, p.id, a).is_err());
        }
        assert_eq!(
            g.objects.in_zone(Zone::Stack).count(),
            usize::from(stage == 2)
        );
        assert_eq!(g.life(), life);
        assert_eq!(g.mana(), mana);
        assert_eq!(format!("{:?}", g.objects), objects);
        assert_eq!(format!("{g:?}"), before);
    }
}
#[test]
fn terminal_same_neutral_boundaries_as_xmage() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/terminal.json")).unwrap();
    for c in fixture["cases"].as_array().unwrap() {
        let mut g = ready();
        g.objects.reset().unwrap();
        g.life = [
            c["life"][0].as_i64().unwrap(),
            c["life"][1].as_i64().unwrap(),
        ];
        if c.get("library_card").is_some() {
            add(&mut g, "forest", Seat::P0, Zone::Library(Seat::P0));
        }
        match c["action"].as_str().unwrap() {
            "settle" => {
                g.settle_terminal(None);
            }
            "draw" => {
                let r = g.draw_top(Seat::P0);
                if c.get("library_card").is_some() {
                    r.unwrap();
                } else {
                    assert_eq!(r, Err(DrawError::EmptyLibrary));
                }
            }
            "concede" => {
                g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
            }
            _ => panic!("unknown neutral action"),
        }
        let lost = g
            .outcome()
            .map(|o| o.losses.map(|l| l.is_some()))
            .unwrap_or([false; 2]);
        let library = [Seat::P0, Seat::P1].map(|s| g.objects.in_zone(Zone::Library(s)).count());
        let hand = [Seat::P0, Seat::P1].map(|s| g.objects.in_zone(Zone::Hand(s)).count());
        assert_eq!(
            serde_json::json!({"life":g.life(),"lost":lost,"library":library,"hand":hand}),
            c["expected"],
            "{}",
            c["id"]
        );
    }
}
#[test]
fn terminal_main_priority_concession_each_seat_and_stale_actions_after_lethal() {
    for seat in [Seat::P0, Seat::P1] {
        let mut g = ready();
        g.turns.position = Some((3, seat, Step::PrecombatMain));
        g.set_turn_decision(seat, crate::opening::turns::TurnKind::Priority);
        let r = g.concede(seat, g.episode_id().unwrap()).unwrap();
        assert_eq!(r.winner, Some(crate::opening::turns::opponent(seat)));
        assert_eq!(r.losses[seat_index(seat)], Some(LossReason::Concession));
        assert!(g.turn_decision().is_none());
    }
    let (mut g, a) = damage_position(2, false);
    let d = g.turn_decision().unwrap();
    let progress = g.finish_combat(d.actor, d.id).unwrap();
    assert_eq!(
        progress,
        crate::opening::turns::TurnProgress::Terminal(g.outcome().unwrap())
    );
    let before = format!("{g:?}");
    assert!(g.finish_combat(d.actor, d.id).is_err());
    assert!(g.select_attackers(d.actor, d.id, &[a]).is_err());
    assert!(g.select_blockers(Seat::P1, d.id, &[]).is_err());
    assert!(g.begin_cast(d.actor, d.id, a).is_err());
    assert!(g.play_land(d.actor, d.id, a).is_err());
    assert!(g.tap_mana(d.actor, d.id, a).is_err());
    assert!(g.cast_candidates(d.actor).is_empty());
    assert!(g.land_candidates(d.actor).is_empty());
    assert!(g.mana_sources(d.actor).is_empty());
    assert_eq!(format!("{g:?}"), before);
}
