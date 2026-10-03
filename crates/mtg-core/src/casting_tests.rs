//! Declared synthetic positions. Expectations: CR 107.4, 117.3c/b, 117.4,
//! 302.1/302.6, 601.2f-i, 605.3, plus RFC atomic invalid-input contract.
use super::*;
use crate::opening::mana::Color;
use crate::opening::turns::{Step, TurnAction, TurnSelection};
fn ready(key: &str, pool: [u32; 6]) -> (Game, Handle) {
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
    pass(&mut g);
    pass(&mut g);
    let h = g
        .objects
        .allocate(
            CardId::from_key(key).unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    g.turns.mana[0] = pool;
    (g, h)
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
fn start(g: &mut Game, h: Handle) -> PaymentDecision {
    let d = g.turn_decision().unwrap();
    assert!(
        g.cast_candidates(d.actor).contains(&h),
        "CR 601: payable vanilla creature must be offered"
    );
    g.begin_cast(d.actor, d.id, h).unwrap()
}
fn pay(g: &mut Game, color: Color) {
    let d = g.payment_decision(Seat::P0).unwrap();
    g.choose_payment(Seat::P0, d.id, color).unwrap();
}
fn finish(g: &mut Game) {
    let d = g.payment_decision(Seat::P0).unwrap();
    g.finish_cast(Seat::P0, d.id).unwrap();
}
#[test]
fn casting_literal_payment_stack_resolution_and_priority() {
    for (key, color) in [("bear-cub", Color::Green), ("swab-goblin", Color::Red)] {
        let (mut g, h) = ready(key, [0, 0, 0, 1, 1, 0]);
        let rng = format!("{:?}", g.rng);
        start(&mut g, h);
        assert!(g.turn_decision().is_none());
        assert!(g.payment_decision(Seat::P1).is_none());
        assert!(g.objects.get(h).is_ok());
        pay(&mut g, color);
        pay(
            &mut g,
            if color == Color::Green {
                Color::Red
            } else {
                Color::Green
            },
        );
        assert_eq!(g.mana()[0], [0, 0, 0, 1, 1, 0]);
        finish(&mut g);
        assert_eq!(g.mana(), [[0; 6]; 2]);
        assert!(g.objects.get(h).is_err());
        let stack: Vec<_> = g.objects.in_zone(Zone::Stack).collect();
        assert_eq!(stack.len(), 1);
        assert_eq!(g.objects.get(stack[0]).unwrap().card.identity().key, key);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        pass(&mut g);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P1);
        assert_eq!(g.objects.in_zone(Zone::Stack).count(), 1);
        pass(&mut g);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        assert_eq!(g.turn_position(), Some((1, Seat::P0, Step::PrecombatMain)));
        assert_eq!(g.objects.in_zone(Zone::Stack).count(), 0);
        let creature = g.objects.in_zone(Zone::Battlefield).last().unwrap();
        assert!(g.objects.get(stack[0]).is_err());
        let o = g.objects.get(creature).unwrap();
        assert_eq!(
            (o.card.identity().key, o.owner, o.controller, o.tapped),
            (key, Seat::P0, Seat::P0, false)
        );
        assert!(g.summoning_sick(creature));
        assert_eq!(g.life(), [20; 2]);
        assert_eq!(format!("{:?}", g.rng), rng);
    }
}
#[test]
fn casting_generic_choices_and_atomic_rejected_final_command() {
    let (mut g, h) = ready("bear-cub", [0, 0, 0, 1, 2, 0]);
    let old = g.turn_decision().unwrap();
    let p = start(&mut g, h);
    pay(&mut g, Color::Green);
    let d = g.payment_decision(Seat::P0).unwrap();
    assert_eq!(d.choices, vec![Color::Red, Color::Green]);
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, d.id).is_err());
    assert!(g.choose_payment(Seat::P0, p.id, Color::Green).is_err());
    assert!(g.begin_cast(Seat::P0, old.id, h).is_err());
    assert!(g.finish_payment(Seat::P0, d.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    pay(&mut g, Color::Red);
    finish(&mut g);
    assert_eq!(g.mana()[0], [0, 0, 0, 0, 1, 0]);
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, d.id).is_err());
    assert!(g.begin_cast(Seat::P0, old.id, h).is_err());
    assert_eq!(format!("{g:?}"), before);
    let (mut alternate, card) = ready("bear-cub", [0, 0, 0, 1, 2, 0]);
    start(&mut alternate, card);
    pay(&mut alternate, Color::Green);
    pay(&mut alternate, Color::Green);
    finish(&mut alternate);
    assert_eq!(alternate.mana()[0], [0, 0, 0, 1, 0, 0]);
    let (mut g, h) = ready("bear-cub", [0, 0, 0, 1, 1, 0]);
    start(&mut g, h);
    pay(&mut g, Color::Green);
    let d = g.payment_decision(Seat::P0).unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.choose_payment(Seat::P0, d.id, Color::Green),
        Err(ManaError::IllegalPayment)
    );
    assert_eq!(format!("{g:?}"), before);
    pay(&mut g, Color::Red);
    finish(&mut g);
}
#[test]
fn casting_missing_colors_masked_timing_foreign_and_wrong_actor() {
    for pool in [[0, 0, 0, 2, 0, 0], [0, 0, 0, 0, 1, 0], [0; 6]] {
        let (mut g, h) = ready("bear-cub", pool);
        let d = g.turn_decision().unwrap();
        assert!(!g.cast_candidates(Seat::P0).contains(&h));
        let before = format!("{g:?}");
        assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
    let (mut g, h) = ready("bear-cub", [0, 0, 0, 1, 1, 0]);
    let (foreign, fh) = ready("bear-cub", [0, 0, 0, 1, 1, 0]);
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    for (actor, id, card) in [
        (Seat::P1, d.id, h),
        (Seat::P0, foreign.turn_decision().unwrap().id, h),
        (Seat::P0, d.id, fh),
    ] {
        assert!(g.begin_cast(actor, id, card).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
    for step in [Step::Upkeep, Step::Draw, Step::BeginningCombat, Step::End] {
        g.turns.position = Some((1, Seat::P0, step));
        let before = format!("{g:?}");
        assert!(g.cast_candidates(Seat::P0).is_empty());
        assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
}
#[test]
fn casting_land_activation_choices_cancel_and_retry() {
    for key in ["bear-cub", "swab-goblin"] {
        let (mut g, h) = ready(key, [0; 6]);
        let lands: Vec<_> = ["forest", "mountain", "forest"]
            .into_iter()
            .map(|key| {
                g.objects
                    .allocate(CardId::from_key(key).unwrap(), Seat::P0, Zone::Battlefield)
                    .unwrap()
            })
            .collect();
        let p = start(&mut g, h);
        assert_eq!(g.cast_mana_sources(Seat::P0), lands);
        assert!(g.cast_mana_sources(Seat::P1).is_empty());
        let p = g.cast_tap_mana(Seat::P0, p.id, lands[1]).unwrap();
        let before = format!("{g:?}");
        assert!(g.finish_cast(Seat::P0, p.id).is_err());
        assert_eq!(format!("{g:?}"), before);
        let p = g.cast_tap_mana(Seat::P0, p.id, lands[0]).unwrap();
        let before = format!("{g:?}");
        assert!(g.cast_tap_mana(Seat::P0, p.id, lands[0]).is_err());
        assert!(g.cast_tap_mana(Seat::P1, p.id, lands[2]).is_err());
        assert_eq!(format!("{g:?}"), before);
        assert!(lands.iter().all(|h| !g.objects.get(*h).unwrap().tapped));
        assert_eq!(g.mana(), [[0; 6]; 2]);
        g.cancel_payment(Seat::P0, p.id).unwrap();
        assert!(g.objects.get(h).is_ok());
        assert!(lands.iter().all(|h| !g.objects.get(*h).unwrap().tapped));
        let p = start(&mut g, h);
        let p = g.cast_tap_mana(Seat::P0, p.id, lands[1]).unwrap();
        g.cast_tap_mana(Seat::P0, p.id, lands[0]).unwrap();
        pay(
            &mut g,
            if key == "bear-cub" {
                Color::Green
            } else {
                Color::Red
            },
        );
        pay(
            &mut g,
            if key == "bear-cub" {
                Color::Red
            } else {
                Color::Green
            },
        );
        finish(&mut g);
        assert!(g.objects.get(lands[0]).unwrap().tapped);
        assert!(g.objects.get(lands[1]).unwrap().tapped);
        assert!(!g.objects.get(lands[2]).unwrap().tapped);
        assert_eq!(g.mana(), [[0; 6]; 2]);
    }
}
#[test]
fn casting_lifo_synthetic_stack_and_mana_response_resets_passes() {
    // Explicit synthetic stack: these two sorcery-speed creatures cannot be
    // legally nested by casting. Instant response effects belong to GH-70.
    let (mut g, hand) = ready("bear-cub", [0; 6]);
    g.objects.remove(hand).unwrap();
    let bottom = g
        .objects
        .allocate(CardId::from_key("bear-cub").unwrap(), Seat::P0, Zone::Stack)
        .unwrap();
    let top = g
        .objects
        .allocate(
            CardId::from_key("swab-goblin").unwrap(),
            Seat::P1,
            Zone::Stack,
        )
        .unwrap();
    g.turns.stack = vec![bottom, top];
    let land = g
        .objects
        .allocate(
            CardId::from_key("mountain").unwrap(),
            Seat::P1,
            Zone::Battlefield,
        )
        .unwrap();
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    g.tap_mana(Seat::P1, d.id, land).unwrap();
    assert_eq!(
        g.objects.in_zone(Zone::Stack).collect::<Vec<_>>(),
        vec![bottom, top]
    );
    pass(&mut g); // P1's action reset consecutive passes: no resolution yet.
    assert_eq!(g.objects.in_zone(Zone::Stack).count(), 2);
    pass(&mut g);
    assert_eq!(
        g.objects.in_zone(Zone::Stack).collect::<Vec<_>>(),
        vec![bottom]
    );
    assert!(g.objects.get(top).is_err());
    let goblin = g.objects.in_zone(Zone::Battlefield).last().unwrap();
    assert_eq!(
        g.objects.get(goblin).unwrap().card.identity().key,
        "swab-goblin"
    );
    assert_eq!(g.objects.get(goblin).unwrap().controller, Seat::P1);
    assert!(g.summoning_sick(goblin));
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0); // active, not controller
    assert_eq!(g.mana()[1], [0, 0, 0, 1, 0, 0]);
    pass(&mut g);
    pass(&mut g);
    assert_eq!(g.objects.in_zone(Zone::Stack).count(), 0);
    let cub = g.objects.in_zone(Zone::Battlefield).last().unwrap();
    assert_eq!(g.objects.get(cub).unwrap().card.identity().key, "bear-cub");
    assert!(g.summoning_sick(cub));
    // Declared end-step position bypasses unimplemented combat, not a full game.
    g.turns.position = Some((1, Seat::P0, Step::End));
    pass(&mut g);
    pass(&mut g);
    assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::Upkeep)));
    assert!(!g.summoning_sick(goblin));
    assert!(g.summoning_sick(cub));
    g.turns.position = Some((2, Seat::P1, Step::End));
    pass(&mut g);
    pass(&mut g);
    assert!(!g.summoning_sick(cub));
}
#[test]
fn casting_fresh_forest_mountain_negative_and_unsupported_cards() {
    let (mut g, h) = ready("bear-cub", [0; 6]);
    let mountain = g
        .objects
        .allocate(
            CardId::from_key("mountain").unwrap(),
            Seat::P0,
            Zone::Battlefield,
        )
        .unwrap();
    let forest = g
        .objects
        .allocate(
            CardId::from_key("forest").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    let d = g.turn_decision().unwrap();
    g.play_land(Seat::P0, d.id, forest).unwrap();
    let forest = g.objects.in_zone(Zone::Battlefield).last().unwrap();
    let d = start(&mut g, h);
    let d = g.cast_tap_mana(Seat::P0, d.id, mountain).unwrap();
    g.cast_tap_mana(Seat::P0, d.id, forest).unwrap();
    pay(&mut g, Color::Green);
    pay(&mut g, Color::Red);
    finish(&mut g);
    let (mut g, h) = ready("bear-cub", [0; 6]);
    for _ in 0..2 {
        g.objects
            .allocate(
                CardId::from_key("mountain").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
    }
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    assert!(!g.cast_candidates(Seat::P0).contains(&h));
    assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
    assert_eq!(format!("{g:?}"), before);
    for key in ["giant-growth", "forest", "viashino-pyromancer"] {
        let (mut g, h) = ready(key, [0, 0, 0, 2, 2, 0]);
        let d = g.turn_decision().unwrap();
        let before = format!("{g:?}");
        assert!(!g.cast_candidates(Seat::P0).contains(&h));
        assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
}
#[test]
fn casting_nested_stale_cancel_reset_and_exhaustion() {
    let (mut g, h) = ready("bear-cub", [0, 0, 0, 2, 2, 0]);
    let h2 = g
        .objects
        .allocate(
            CardId::from_key("bear-cub").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    start(&mut g, h);
    pay(&mut g, Color::Green);
    pay(&mut g, Color::Red);
    let d = g.payment_decision(Seat::P0).unwrap();
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P1, d.id).is_err());
    assert!(g.finish_payment(Seat::P0, d.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    g.generation = u64::MAX;
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, d.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    g.generation = d.id.generation;
    finish(&mut g);
    for _ in 0..2 {
        let d = g.turn_decision().unwrap();
        let before = format!("{g:?}");
        assert!(g.cast_candidates(d.actor).is_empty());
        assert!(g.begin_cast(d.actor, d.id, h2).is_err());
        assert_eq!(format!("{g:?}"), before);
        pass(&mut g);
    }
    let p = start(&mut g, h2);
    pay(&mut g, Color::Green);
    let before = format!("{g:?}");
    assert!(g.cancel_payment(Seat::P0, p.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    let p = g.payment_decision(Seat::P0).unwrap();
    g.cancel_payment(Seat::P0, p.id).unwrap();
    assert_eq!(g.mana()[0], [0, 0, 0, 1, 1, 0]);
    let p = start(&mut g, h2);
    g.reset(&Config::default(), 42, 9).unwrap();
    assert!(g.turns.stack.is_empty());
    assert!(g.turns.sick.is_empty());
    assert!(g.turns.casting.is_none());
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, p.id).is_err());
    assert_eq!(format!("{g:?}"), before);
}

// GH-195: synthetic old creatures; CR 302.6, 605.1a/605.3b and 601.2.
// Printed definitions: Llanowar Elves G 1/1; Druid of the Cowl 1G 1/3.
#[test]
fn creature_mana_priority_is_immediate_and_rejects_illegal_sources() {
    for key in ["llanowar-elves", "druid-of-the-cowl"] {
        let (mut g, _) = ready("bear-cub", [0; 6]);
        let card = CardId::from_key(key).unwrap();
        let own = g
            .objects
            .allocate(card, Seat::P0, Zone::Battlefield)
            .unwrap();
        let enemy = g
            .objects
            .allocate(card, Seat::P1, Zone::Battlefield)
            .unwrap();
        let sick = g
            .objects
            .allocate(card, Seat::P0, Zone::Battlefield)
            .unwrap();
        g.turns.sick.push(sick);
        let stale = g
            .objects
            .allocate(card, Seat::P0, Zone::Battlefield)
            .unwrap();
        g.objects.move_to(stale, Zone::Graveyard(Seat::P0)).unwrap();
        let d = g.turn_decision().unwrap();
        for h in [enemy, sick, stale] {
            let before = format!("{g:?}");
            assert!(g.tap_mana(Seat::P0, d.id, h).is_err());
            assert_eq!(format!("{g:?}"), before);
        }
        assert!(
            g.mana_sources(Seat::P0).contains(&own),
            "CR 605: old {key} offers G"
        );
        let next = g.tap_mana(Seat::P0, d.id, own).unwrap();
        assert_eq!(g.mana(), [[0, 0, 0, 0, 1, 0], [0; 6]]);
        assert!(g.objects.get(own).unwrap().tapped);
        assert_eq!(next.actor, Seat::P0);
        assert_eq!(g.objects.in_zone(Zone::Stack).count(), 0);
        let before = format!("{g:?}");
        assert!(g.tap_mana(Seat::P0, next.id, own).is_err());
        assert!(g.tap_mana(Seat::P0, d.id, own).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
}

#[test]
fn creature_mana_staged_cancel_and_commit_are_atomic() {
    for key in ["llanowar-elves", "druid-of-the-cowl"] {
        let (mut g, cub) = ready("bear-cub", [0; 6]);
        let creature = g
            .objects
            .allocate(CardId::from_key(key).unwrap(), Seat::P0, Zone::Battlefield)
            .unwrap();
        let forest = g
            .objects
            .allocate(
                CardId::from_key("forest").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        for cancel in [true, false] {
            start(&mut g, cub);
            for source in [creature, forest] {
                let d = g.payment_decision(Seat::P0).unwrap();
                g.cast_tap_mana(Seat::P0, d.id, source).unwrap();
                let before = format!("{g:?}");
                let d = g.payment_decision(Seat::P0).unwrap();
                assert!(g.cast_tap_mana(Seat::P0, d.id, source).is_err());
                assert_eq!(format!("{g:?}"), before);
            }
            pay(&mut g, Color::Green);
            pay(&mut g, Color::Green);
            assert_eq!(g.mana(), [[0; 6]; 2]);
            assert!(!g.objects.get(creature).unwrap().tapped);
            assert!(!g.objects.get(forest).unwrap().tapped);
            assert!(g.turn_decision().is_none());
            if cancel {
                let d = g.payment_decision(Seat::P0).unwrap();
                g.cancel_payment(Seat::P0, d.id).unwrap();
                assert!(g.objects.get(cub).is_ok());
                assert!(!g.objects.get(creature).unwrap().tapped);
                assert!(!g.objects.get(forest).unwrap().tapped);
                assert_eq!(g.mana(), [[0; 6]; 2]);
            } else {
                finish(&mut g);
                assert!(g.objects.get(creature).unwrap().tapped);
                assert!(g.objects.get(forest).unwrap().tapped);
                assert_eq!(g.objects.in_zone(Zone::Stack).count(), 1);
                assert_eq!(g.mana(), [[0; 6]; 2]);
                assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
            }
        }
    }
}

#[test]
fn creature_mana_pending_snapshot_and_illegal_payment_sources() {
    use super::super::policy::{Choice, Submission, VisibleRef, VisibleZone};
    fn command(g: &mut Game, c: Choice) {
        let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
        g.apply_policy(
            Seat::P0,
            &Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices: vec![c],
            },
            256,
        )
        .unwrap();
    }
    for key in ["llanowar-elves", "druid-of-the-cowl"] {
        let (mut g, cub) = ready("bear-cub", [0; 6]);
        let source = g
            .objects
            .allocate(CardId::from_key(key).unwrap(), Seat::P0, Zone::Battlefield)
            .unwrap();
        let forest = g
            .objects
            .allocate(
                CardId::from_key("forest").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        let enemy = g
            .objects
            .allocate(CardId::from_key(key).unwrap(), Seat::P1, Zone::Battlefield)
            .unwrap();
        let sick = g
            .objects
            .allocate(CardId::from_key(key).unwrap(), Seat::P0, Zone::Battlefield)
            .unwrap();
        g.turns.sick.push(sick);
        let stale = g
            .objects
            .allocate(CardId::from_key(key).unwrap(), Seat::P0, Zone::Battlefield)
            .unwrap();
        g.objects.move_to(stale, Zone::Graveyard(Seat::P0)).unwrap();
        start(&mut g, cub);
        for h in [enemy, sick, stale] {
            let d = g.payment_decision(Seat::P0).unwrap();
            let before = g.snapshot();
            assert!(g.cast_tap_mana(Seat::P0, d.id, h).is_err());
            assert_eq!(g.snapshot(), before);
        }
        let choices = [
            Choice::TapMana {
                card: VisibleRef {
                    zone: VisibleZone::Battlefield,
                    row: 0,
                },
            },
            Choice::Pay { color: 4 },
            Choice::TapMana {
                card: VisibleRef {
                    zone: VisibleZone::Battlefield,
                    row: 1,
                },
            },
            Choice::Pay { color: 4 },
        ];
        for stage in 0..=choices.len() {
            let bytes = g.snapshot();
            for cancel in [true, false] {
                let mut restored = Game::new().unwrap();
                restored.restore(&bytes).unwrap();
                assert_eq!(restored.mana(), [[0; 6]; 2]);
                assert!(restored.turn_decision().is_none());
                if cancel {
                    command(&mut restored, Choice::CancelPayment);
                } else {
                    for c in &choices[stage..] {
                        command(&mut restored, c.clone());
                    }
                    command(&mut restored, Choice::FinishPayment);
                }
                let board: Vec<_> = restored.objects.in_zone(Zone::Battlefield).collect();
                assert_eq!(restored.objects.get(board[0]).unwrap().tapped, !cancel);
                assert_eq!(restored.objects.get(board[1]).unwrap().tapped, !cancel);
                assert_eq!(
                    restored.objects.in_zone(Zone::Stack).count(),
                    usize::from(!cancel)
                );
                assert_eq!(restored.mana(), [[0; 6]; 2]);
            }
            if stage < choices.len() {
                command(&mut g, choices[stage].clone());
            }
        }
        // Commit revalidates sources even if a test-only mutation invalidates a
        // reservation. Neither payments nor any other reserved source may leak.
        g.turns.sick.push(source);
        let d = g.payment_decision(Seat::P0).unwrap();
        let before = g.snapshot();
        assert!(g.finish_cast(Seat::P0, d.id).is_err());
        assert_eq!(g.snapshot(), before);
        assert!(!g.objects.get(forest).unwrap().tapped);
    }
}

/// Pinned Thrill of Possibility, CR 601.2h / 117: another hand card
/// and 1R make this instant available at upkeep without any creature targets.
#[test]
fn thrill_payable_instant_is_available_without_targets() {
    let (mut g, h) = ready("thrill-of-possibility", [0, 0, 0, 2, 0, 0]);
    g.turns.position = Some((1, Seat::P0, Step::Upkeep));
    assert!(g.objects.in_zone(Zone::Hand(Seat::P0)).count() > 1);
    assert!(
        g.cast_candidates(Seat::P0).contains(&h),
        "Pinned 1R Thrill with another card must be castable at upkeep"
    );
}

fn thrill_position(library: &[&str]) -> (Game, Handle, Handle) {
    let (mut g, _) = ready("thrill-of-possibility", [0, 0, 0, 2, 0, 0]);
    for zone in [Zone::Hand(Seat::P0), Zone::Library(Seat::P0)] {
        let old: Vec<_> = g.objects.in_zone(zone).collect();
        for h in old {
            g.objects.remove(h).unwrap();
        }
    }
    let spell = g
        .objects
        .allocate(
            CardId::from_key("thrill-of-possibility").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    let discard = g
        .objects
        .allocate(
            CardId::from_key("mountain").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    for key in library {
        g.objects
            .allocate(
                CardId::from_key(key).unwrap(),
                Seat::P0,
                Zone::Library(Seat::P0),
            )
            .unwrap();
    }
    (g, spell, discard)
}
fn thrill_select(
    g: &mut Game,
    choices: Vec<super::super::policy::Choice>,
) -> Result<(), super::super::policy::PolicyError> {
    let o = g.policy_observe(Seat::P0, 256).unwrap();
    let d = o.decision.unwrap();
    g.apply_policy(
        Seat::P0,
        &super::super::policy::Submission {
            revision: d.revision,
            schema_version: 1,
            generation: d.generation,
            choices,
        },
        256,
    )
}
fn discard_choice(row: usize) -> super::super::policy::Choice {
    super::super::policy::Choice::Discard {
        card: super::super::policy::VisibleRef {
            zone: super::super::policy::VisibleZone::Hand,
            row,
        },
    }
}
fn keys(g: &Game, zone: Zone) -> Vec<&'static str> {
    g.objects
        .in_zone(zone)
        .map(|h| g.objects.get(h).unwrap().card.identity().key)
        .collect()
}
#[test]
fn thrill_discard_atomic_ordered_draw_and_failed_second_draw() {
    // CR 601.2h, 121.2, 608, 704.5b. Literal decks, never derived expectations.
    for library in [
        vec!["forest", "bear-cub", "giant-growth"],
        vec!["forest"],
        vec![],
    ] {
        let (mut g, spell, discarded) = thrill_position(&library);
        start(&mut g, spell);
        let spell_row = g
            .view_hand(Seat::P0)
            .iter()
            .position(|h| *h == spell)
            .unwrap();
        let discard_row = g
            .view_hand(Seat::P0)
            .iter()
            .position(|h| *h == discarded)
            .unwrap();
        let before = format!("{g:?}");
        assert!(thrill_select(&mut g, vec![]).is_err());
        assert!(thrill_select(&mut g, vec![discard_choice(spell_row)]).is_err());
        assert!(
            thrill_select(
                &mut g,
                vec![discard_choice(discard_row), discard_choice(discard_row)]
            )
            .is_err()
        );
        assert_eq!(format!("{g:?}"), before);
        thrill_select(&mut g, vec![discard_choice(discard_row)])
            .expect("one other hand card is a legal additional cost");
        assert_eq!(g.objects.get(discarded).unwrap().zone, Zone::Hand(Seat::P0));
        let before = format!("{g:?}");
        let d = g.payment_decision(Seat::P0).unwrap();
        assert!(g.finish_cast(Seat::P0, d.id).is_err());
        assert!(g.finish_cast(Seat::P1, d.id).is_err());
        assert_eq!(format!("{g:?}"), before);
        pay(&mut g, Color::Red);
        pay(&mut g, Color::Red);
        finish(&mut g);
        assert_eq!(keys(&g, Zone::Graveyard(Seat::P0)), ["mountain"]);
        assert_eq!(keys(&g, Zone::Stack), ["thrill-of-possibility"]);
        assert_eq!(keys(&g, Zone::Hand(Seat::P0)), Vec::<&str>::new());
        assert_eq!(g.mana()[0], [0; 6]);
        pass(&mut g);
        assert_eq!(keys(&g, Zone::Hand(Seat::P0)), Vec::<&str>::new());
        pass(&mut g);
        assert_eq!(
            keys(&g, Zone::Graveyard(Seat::P0)),
            ["mountain", "thrill-of-possibility"]
        );
        if library.len() == 3 {
            assert_eq!(keys(&g, Zone::Hand(Seat::P0)), ["forest", "bear-cub"]);
            assert_eq!(keys(&g, Zone::Library(Seat::P0)), ["giant-growth"]);
            assert!(g.outcome().is_none());
        } else {
            assert_eq!(keys(&g, Zone::Hand(Seat::P0)), library);
            assert_eq!(
                g.outcome().unwrap().losses,
                [Some(super::super::terminal::LossReason::EmptyDraw), None]
            );
        }
    }
}
#[test]
fn thrill_alone_or_wrong_color_rejects_without_mutation() {
    for alone in [true, false] {
        let (mut g, spell, discard) = thrill_position(&["forest"]);
        if alone {
            g.objects.remove(discard).unwrap();
        } else {
            g.turns.mana[0] = [0, 0, 0, 0, 2, 0];
        }
        let before = format!("{g:?}");
        let d = g.turn_decision().unwrap();
        assert!(!g.cast_candidates(Seat::P0).contains(&spell));
        assert!(g.begin_cast(Seat::P0, d.id, spell).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
}

#[test]
fn thrill_private_pending_snapshots_cancel_stale_and_quantum() {
    use super::super::policy::Choice;
    let (mut g, spell, discard) = thrill_position(&["forest", "bear-cub", "giant-growth"]);
    let baseline = g.policy_observe(Seat::P1, 256).unwrap();
    let start_id = start(&mut g, spell).id;
    for stage in 0_usize..4 {
        let bytes = g.snapshot();
        let p = g.payment_decision(Seat::P0).unwrap();
        assert!(g.choose_cast_discard(Seat::P1, p.id, &[discard]).is_err());
        assert!(g.choose_cast_discard(Seat::P0, p.id, &[]).is_err());
        assert!(
            g.choose_cast_discard(Seat::P0, p.id, &[discard, discard])
                .is_err()
        );
        if stage > 0 {
            assert!(
                g.choose_cast_discard(Seat::P0, start_id, &[discard])
                    .is_err()
            );
        }
        assert_eq!(g.snapshot(), bytes);
        let opponent = g.policy_observe(Seat::P1, 256).unwrap();
        assert!(opponent.pending.is_none() && opponent.decision.is_none());
        assert_eq!(opponent.view.hand, baseline.view.hand);
        assert_eq!(opponent.view.public_zones, baseline.view.public_zones);
        for cancel in [false, true] {
            let mut restored = Game::new().unwrap();
            restored.restore(&bytes).unwrap();
            if cancel {
                thrill_select(&mut restored, vec![Choice::CancelPayment]).unwrap();
                assert_eq!(
                    keys(&restored, Zone::Hand(Seat::P0)),
                    ["thrill-of-possibility", "mountain"]
                );
                assert!(keys(&restored, Zone::Graveyard(Seat::P0)).is_empty());
                assert_eq!(restored.mana()[0], [0, 0, 0, 2, 0, 0]);
            } else {
                if stage == 0 {
                    let row = restored
                        .view_hand(Seat::P0)
                        .iter()
                        .position(|h| {
                            restored.objects.get(*h).unwrap().card.identity().key == "mountain"
                        })
                        .unwrap();
                    thrill_select(&mut restored, vec![discard_choice(row)]).unwrap();
                }
                for _ in stage.saturating_sub(1)..2 {
                    pay(&mut restored, Color::Red);
                }
                finish(&mut restored);
                pass(&mut restored);
                let d = restored.turn_decision().unwrap();
                let mut progress = restored
                    .apply_turn_quantum(
                        d.actor,
                        &TurnAction {
                            decision: d.id,
                            selection: TurnSelection::Pass(d.candidate(0)),
                        },
                        NonZeroUsize::MIN,
                    )
                    .unwrap();
                assert_eq!(keys(&restored, Zone::Hand(Seat::P0)), ["forest"]);
                while progress == Progress::InternalYield {
                    assert!(restored.turn_decision().is_none());
                    let b = restored.snapshot();
                    let mut copy = Game::new().unwrap();
                    copy.restore(&b).unwrap();
                    assert_eq!(copy.rng, restored.rng);
                    assert_eq!(
                        keys(&copy, Zone::Hand(Seat::P0)),
                        keys(&restored, Zone::Hand(Seat::P0))
                    );
                    restored = copy;
                    progress = restored.resume(NonZeroUsize::MIN);
                }
                assert_eq!(
                    keys(&restored, Zone::Hand(Seat::P0)),
                    ["forest", "bear-cub"]
                );
            }
        }
        if stage == 0 {
            g.choose_cast_discard(Seat::P0, p.id, &[discard]).unwrap();
        } else if stage < 3 {
            pay(&mut g, Color::Red);
        }
    }
}

#[test]
fn thrill_reference_literal_checkpoints() {
    use serde_json::{Value, json};
    fn point(g: &Game) -> Value {
        json!({"hand":keys(g,Zone::Hand(Seat::P0)),"graveyard":keys(g,Zone::Graveyard(Seat::P0)),"library":keys(g,Zone::Library(Seat::P0)),"stack":keys(g,Zone::Stack),"mana":g.mana()[0].iter().sum::<u32>(),"lost":g.outcome().is_some_and(|o|o.losses[0].is_some())})
    }
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/thrill.json")).unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/thrill-expectations.json"
    ))
    .unwrap();
    let mut actual = serde_json::Map::new();
    for c in fixture["cases"].as_array().unwrap() {
        let library: Vec<_> = c["library"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        let (mut g, spell, discard) = thrill_position(&library);
        g.turns.position = Some((1, Seat::P0, Step::Upkeep));
        if !c["discard"].as_bool().unwrap() {
            g.objects.remove(discard).unwrap();
        }
        g.turns.mana[0] = [0; 6];
        let color = if c["mana"] == "red" {
            Color::Red
        } else {
            Color::Green
        };
        g.turns.mana[0][color.index()] = c["amount"].as_u64().unwrap() as u32;
        let legal = g.cast_candidates(Seat::P0).contains(&spell);
        if legal {
            start(&mut g, spell);
            let p = g.payment_decision(Seat::P0).unwrap();
            g.choose_cast_discard(Seat::P0, p.id, &[discard]).unwrap();
            pay(&mut g, Color::Red);
            pay(&mut g, color);
            finish(&mut g);
        } else {
            let old = g.snapshot();
            let d = g.turn_decision().unwrap();
            assert!(g.begin_cast(Seat::P0, d.id, spell).is_err());
            assert_eq!(g.snapshot(), old);
        }
        let before = point(&g);
        if legal {
            pass(&mut g);
            pass(&mut g);
        }
        let result = json!({"legal":legal,"before":before,"after":point(&g)});
        assert_eq!(result, expected[c["id"].as_str().unwrap()]);
        actual.insert(c["id"].as_str().unwrap().into(), result);
    }
    if let Ok(path) = std::env::var("MTG_THRILL_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&actual).unwrap()).unwrap();
    }
}

#[test]
fn thrill_response_window_missing_discard_and_departed_cost_reject_atomically() {
    let (mut g, spell, discard) = thrill_position(&["forest", "bear-cub"]);
    let growth = g
        .objects
        .allocate(
            CardId::from_key("giant-growth").unwrap(),
            Seat::P1,
            Zone::Hand(Seat::P1),
        )
        .unwrap();
    let original = g.turn_decision().unwrap();
    start(&mut g, spell);
    let before = g.snapshot();
    assert!(
        g.begin_targeted_cast(Seat::P1, original.id, growth, 256)
            .is_err()
    );
    assert_eq!(before, g.snapshot());
    pay(&mut g, Color::Red);
    pay(&mut g, Color::Red);
    let p = g.payment_decision(Seat::P0).unwrap();
    let before = g.snapshot();
    assert!(g.finish_cast(Seat::P0, p.id).is_err());
    assert_eq!(before, g.snapshot());
    g.choose_cast_discard(Seat::P0, p.id, &[discard]).unwrap();
    // Explicit storage mutation probes the commit revalidation boundary.
    g.objects
        .move_to(discard, Zone::Graveyard(Seat::P0))
        .unwrap();
    let p = g.payment_decision(Seat::P0).unwrap();
    let before = g.snapshot();
    assert!(g.finish_cast(Seat::P0, p.id).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(g.mana()[0], [0, 0, 0, 2, 0, 0]);
}

/// Pinned Goblin Surprise {2}{R}, CR 700.2/601.2b: either mode is
/// available at instant speed without targets, even on an empty battlefield.
#[test]
fn surprise_payable_instant_requires_explicit_mode() {
    let (mut g, h) = ready("goblin-surprise", [0, 0, 0, 3, 0, 0]);
    g.turns.position = Some((1, Seat::P1, Step::Upkeep));
    assert!(
        g.cast_candidates(Seat::P0).contains(&h),
        "Pinned Surprise must be available at instant speed"
    );
    start(&mut g, h);
    let o = g.policy_observe(Seat::P0, 256).unwrap();
    assert_eq!(o.decision.unwrap().kind, "cast_mode");
    assert!(thrill_select(&mut g, vec![]).is_err());
    assert!(
        g.finish_cast(Seat::P0, g.payment_decision(Seat::P0).unwrap().id)
            .is_err()
    );
}

fn mode_choice(mode: u8) -> super::super::policy::Choice {
    serde_json::from_value(serde_json::json!({"kind":"mode","mode":mode})).unwrap()
}
#[test]
fn surprise_modes_literal_affected_set_and_rejection() {
    // Pinned Oracle / CR 611.2c: fix the affected set at resolution, not cast.
    for mode in [0, 1] {
        let (mut g, spell) = ready("goblin-surprise", [0, 0, 0, 4, 2, 0]);
        let cub = g
            .objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        let token = g
            .objects
            .allocate(
                CardId::from_key("goblin-token").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        let enemy = g
            .objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                Seat::P1,
                Zone::Battlefield,
            )
            .unwrap();
        start(&mut g, spell);
        let before = g.snapshot();
        for cs in [
            vec![],
            vec![mode_choice(0), mode_choice(1)],
            vec![mode_choice(2)],
        ] {
            assert!(thrill_select(&mut g, cs).is_err());
            assert_eq!(g.snapshot(), before);
        }
        thrill_select(&mut g, vec![mode_choice(mode)]).unwrap();
        assert!(g.policy_observe(Seat::P1, 256).unwrap().pending.is_none());
        pay(&mut g, Color::Red);
        pay(&mut g, Color::Green);
        pay(&mut g, Color::Red);
        finish(&mut g);
        let during = g
            .objects
            .allocate(
                CardId::from_key("goblin-token").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        pass(&mut g);
        pass(&mut g);
        let boost = if mode == 0 { 2 } else { 0 };
        assert_eq!(g.creature_state(cub).unwrap().power, 2 + boost);
        assert_eq!(g.creature_state(cub).unwrap().toughness, 2);
        for h in [token, during] {
            assert_eq!(g.creature_state(h).unwrap().power, 1 + boost);
            assert_eq!(g.creature_state(h).unwrap().toughness, 1);
        }
        assert_eq!(g.creature_state(enemy).unwrap().power, 2);
        let after = g
            .objects
            .allocate(
                CardId::from_key("goblin-token").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        assert_eq!(g.creature_state(after).unwrap().power, 1);
        assert_eq!(
            g.objects.in_zone(Zone::Battlefield).count(),
            if mode == 0 { 5 } else { 7 }
        );
        g.turns.position = Some((1, Seat::P0, Step::End));
        // Keep the declared fixture's hand within the cleanup limit.
        let excess: Vec<_> = g.objects.in_zone(Zone::Hand(Seat::P0)).skip(7).collect();
        for h in excess {
            g.objects.remove(h).unwrap();
        }
        pass(&mut g);
        pass(&mut g);
        assert_eq!(g.creature_state(cub).unwrap().power, 2);
        assert_eq!(g.creature_state(token).unwrap().power, 1);
    }
}

#[test]
fn surprise_modes_payments_cancellation_snapshots_and_quantum() {
    use super::super::policy::Choice;
    for mode in [0, 1] {
        for colors in [
            [Color::Red, Color::Red],
            [Color::Red, Color::Green],
            [Color::Green, Color::Red],
            [Color::Green, Color::Green],
        ] {
            let (mut g, spell) = ready("goblin-surprise", [0, 0, 0, 3, 2, 0]);
            g.objects
                .allocate(
                    CardId::from_key("bear-cub").unwrap(),
                    Seat::P0,
                    Zone::Battlefield,
                )
                .unwrap();
            let old = start(&mut g, spell).id;
            for stage in 0..5 {
                let bytes = g.snapshot();
                let id = g.payment_decision(Seat::P0).unwrap().id;
                assert!(g.choose_cast_mode(Seat::P1, id, &[mode]).is_err());
                assert!(g.choose_cast_mode(Seat::P0, id, &[]).is_err());
                assert!(g.choose_cast_mode(Seat::P0, id, &[0, 1]).is_err());
                assert!(g.choose_cast_mode(Seat::P0, id, &[9]).is_err());
                if stage > 0 {
                    assert!(g.choose_cast_mode(Seat::P0, old, &[mode]).is_err());
                } else {
                    assert!(g.choose_payment(Seat::P0, id, Color::Red).is_err());
                }
                assert_eq!(bytes, g.snapshot());
                let mut restored = Game::new().unwrap();
                restored.restore(&bytes).unwrap();
                thrill_select(&mut restored, vec![Choice::CancelPayment]).unwrap();
                assert_eq!(restored.mana()[0], [0, 0, 0, 3, 2, 0]);
                assert!(restored.objects.in_zone(Zone::Stack).next().is_none());
                assert!(
                    restored
                        .policy_observe(Seat::P0, 256)
                        .unwrap()
                        .pending
                        .is_none()
                );
                let mut restored = Game::new().unwrap();
                restored.restore(&bytes).unwrap();
                assert_eq!(
                    restored
                        .policy_observe(Seat::P0, 256)
                        .unwrap()
                        .pending
                        .unwrap()
                        .mode,
                    if stage == 0 { None } else { Some(mode) }
                );
                if stage == 0 {
                    thrill_select(&mut g, vec![Choice::Mode { mode }]).unwrap();
                } else if stage == 1 {
                    pay(&mut g, Color::Red);
                } else if stage < 4 {
                    pay(&mut g, colors[stage - 2]);
                }
            }
            finish(&mut g);
            pass(&mut g);
            let before = g.snapshot();
            let d = g.turn_decision().unwrap();
            g.apply_turn_quantum(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Pass(d.candidate(0)),
                },
                std::num::NonZeroUsize::MIN,
            )
            .unwrap();
            // Include the immediate first yield; each engine-produced work snapshot must restore.
            loop {
                let bytes = g.snapshot();
                let mut copy = Game::new().unwrap();
                copy.restore(&bytes).unwrap();
                if g.resume(std::num::NonZeroUsize::MIN) != Progress::InternalYield {
                    break;
                }
            }
            let mut full = Game::new().unwrap();
            full.restore(&before).unwrap();
            pass(&mut full);
            assert_eq!(keys(&g, Zone::Battlefield), keys(&full, Zone::Battlefield));
            let states = |x: &Game| {
                x.objects
                    .in_zone(Zone::Battlefield)
                    .filter_map(|h| x.creature_state(h))
                    .collect::<Vec<_>>()
            };
            assert_eq!(states(&g), states(&full));
        }
    }
}

fn surprise_point(g: &Game) -> serde_json::Value {
    let mut rows: Vec<_> = g
        .objects
        .in_zone(Zone::Battlefield)
        .filter_map(|h| {
            let c = g.creature_state(h)?;
            let o = g.objects.get(h).unwrap();
            Some(format!(
                "{}:{}:{}/{}:{}",
                seat_index(o.controller),
                o.card.identity().key,
                c.power,
                c.toughness,
                c.damage
            ))
        })
        .collect();
    rows.sort();
    serde_json::json!({"creatures":rows,"life":g.life()})
}
fn surprise_add(g: &mut Game, key: &str, seat: Seat) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, Zone::Battlefield)
        .unwrap()
}
fn surprise_pair(g: &mut Game) {
    pass(g);
    pass(g);
}
fn surprise_cleanup(g: &mut Game) {
    g.turns.position = Some((3, Seat::P0, Step::End));
    g.set_turn_decision(Seat::P0, super::super::turns::TurnKind::Priority);
    surprise_pair(g);
}
#[test]
fn surprise_reference_literal_checkpoints() {
    use super::super::turns::TurnKind;
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/surprise-expectations.json"
    ))
    .unwrap();
    let mut result = serde_json::Map::new();
    for scenario in [
        "boost", "tokens", "later", "stacked", "block", "end", "swab",
    ] {
        let (mut g, spell) = ready("goblin-surprise", [0, 0, 0, 10, 2, 0]);
        // Synthetic fixtures retain only the spell in hand to avoid unrelated cleanup discard.
        let extra: Vec<_> = g
            .objects
            .in_zone(Zone::Hand(Seat::P0))
            .filter(|h| *h != spell)
            .collect();
        for h in extra {
            g.objects.remove(h).unwrap();
        }
        if ["boost", "tokens"].contains(&scenario) {
            surprise_add(&mut g, "bear-cub", Seat::P0);
            surprise_add(&mut g, "bear-cub", Seat::P1);
        }
        if ["boost", "tokens", "later"].contains(&scenario) {
            for _ in 0..if scenario == "boost" { 2 } else { 1 } {
                surprise_add(&mut g, "goblin-token", Seat::P0);
            }
        }
        if scenario == "stacked" {
            let dragon = surprise_add(&mut g, "shivan-dragon", Seat::P0);
            let growth = g
                .objects
                .allocate(
                    CardId::from_key("giant-growth").unwrap(),
                    Seat::P0,
                    Zone::Hand(Seat::P0),
                )
                .unwrap();
            let d = g.turn_decision().unwrap();
            let t = g.begin_targeted_cast(Seat::P0, d.id, growth, 256).unwrap();
            let t = g.choose_target(Seat::P0, t.id, dragon).unwrap();
            g.finish_targets(Seat::P0, t.id).unwrap();
            pay(&mut g, Color::Green);
            finish(&mut g);
            surprise_pair(&mut g);
            let d = g.turn_decision().unwrap();
            g.begin_activation(Seat::P0, d.id, dragon).unwrap();
            let id = g.turns.activation.as_ref().unwrap().id;
            g.pay_activation(Seat::P0, id, Color::Red).unwrap();
            let id = g.turns.activation.as_ref().unwrap().id;
            g.finish_activation(Seat::P0, id).unwrap();
            surprise_pair(&mut g);
        }
        let mut attacker = None;
        if ["block", "swab"].contains(&scenario) {
            if scenario == "swab" {
                surprise_add(&mut g, "swab-goblin", Seat::P0);
            }
            let h = surprise_add(
                &mut g,
                if scenario == "block" {
                    "bear-cub"
                } else {
                    "magnigoth-sentry"
                },
                Seat::P1,
            );
            attacker = Some(h);
            g.turns.position = Some((2, Seat::P1, Step::BeginningCombat));
            g.set_turn_decision(Seat::P1, TurnKind::Priority);
            surprise_pair(&mut g);
            let d = g.turn_decision().unwrap();
            let d = g.select_attackers(Seat::P1, d.id, &[h]).unwrap();
            g.finish_combat(Seat::P1, d.id).unwrap();
            pass(&mut g);
        }
        if scenario == "end" {
            g.turns.position = Some((2, Seat::P1, Step::End));
        }
        let mode = u8::from(["tokens", "block", "end"].contains(&scenario));
        g.turns.mana[0] = [0, 0, 0, 10, 2, 0];
        start(&mut g, spell);
        let id = g.payment_decision(Seat::P0).unwrap().id;
        g.choose_cast_mode(Seat::P0, id, &[mode]).unwrap();
        for _ in 0..3 {
            pay(&mut g, Color::Red);
        }
        finish(&mut g);
        assert_eq!(
            g.policy_observe(Seat::P1, 256).unwrap().stack[0].mode,
            Some(mode)
        );
        surprise_pair(&mut g);
        let mut out = serde_json::json!({"mode":mode,"resolved":surprise_point(&g)});
        if scenario == "later" {
            let fodder = g
                .objects
                .allocate(
                    CardId::from_key("dragon-fodder").unwrap(),
                    Seat::P0,
                    Zone::Hand(Seat::P0),
                )
                .unwrap();
            start(&mut g, fodder);
            pay(&mut g, Color::Red);
            pay(&mut g, Color::Red);
            finish(&mut g);
            surprise_pair(&mut g);
            out["later"] = surprise_point(&g);
        }
        if let Some(attacker) = attacker {
            surprise_pair(&mut g); // blockers declaration
            let blocker = g
                .objects
                .in_zone(Zone::Battlefield)
                .find(|h| g.objects.get(*h).unwrap().controller == Seat::P0)
                .unwrap();
            let d = g.turn_decision().unwrap();
            let d = g
                .select_blockers(Seat::P0, d.id, &[(blocker, attacker)])
                .unwrap();
            g.finish_combat(Seat::P0, d.id).unwrap();
            surprise_pair(&mut g);
            let d = g.turn_decision().unwrap();
            g.finish_combat(d.actor, d.id).unwrap();
            out["combat"] = surprise_point(&g);
        }
        if scenario == "end" {
            surprise_pair(&mut g); // finish opponent end and own untap/upkeep
            assert!(
                g.objects
                    .in_zone(Zone::Battlefield)
                    .all(|h| !g.summoning_sick(h))
            );
            g.turns.position = Some((3, Seat::P0, Step::BeginningCombat));
            g.set_turn_decision(Seat::P0, TurnKind::Priority);
            surprise_pair(&mut g);
            let tokens: Vec<_> = g.objects.in_zone(Zone::Battlefield).collect();
            let d = g.turn_decision().unwrap();
            let d = g.select_attackers(Seat::P0, d.id, &tokens).unwrap();
            g.finish_combat(Seat::P0, d.id).unwrap();
            surprise_pair(&mut g);
            let d = g.turn_decision().unwrap();
            g.finish_combat(Seat::P1, d.id).unwrap();
            surprise_pair(&mut g);
            let d = g.turn_decision().unwrap();
            g.finish_combat(d.actor, d.id).unwrap();
            out["combat"] = surprise_point(&g);
        }
        surprise_cleanup(&mut g);
        out["cleanup"] = surprise_point(&g);
        assert_eq!(out, expected[scenario], "{scenario}");
        result.insert(scenario.into(), out);
    }
    if let Ok(path) = std::env::var("MTG_SURPRISE_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    }
}

#[test]
fn surprise_explicit_source_combinations_and_cancel_keep_taps_private() {
    // Four distinct sources, choose any three including red: every physical
    // choice is retained for both modes. CR 601.2b/g/h and cancellation contract.
    for mode in [0, 1] {
        for omitted in 0..4 {
            for cancel in [false, true] {
                let (mut g, spell) = ready("goblin-surprise", [0; 6]);
                let sources: Vec<_> = ["mountain", "mountain", "forest", "forest"]
                    .into_iter()
                    .map(|k| surprise_add(&mut g, k, Seat::P0))
                    .collect();
                start(&mut g, spell);
                let p = g.payment_decision(Seat::P0).unwrap();
                g.choose_cast_mode(Seat::P0, p.id, &[mode]).unwrap();
                for (i, h) in sources.iter().enumerate() {
                    if i != omitted {
                        let p = g.payment_decision(Seat::P0).unwrap();
                        g.cast_tap_mana(Seat::P0, p.id, *h).unwrap();
                    }
                }
                assert!(sources.iter().all(|h| !g.objects.get(*h).unwrap().tapped));
                let bytes = g.snapshot();
                let p = g.payment_decision(Seat::P0).unwrap();
                assert!(g.finish_cast(Seat::P0, p.id).is_err());
                assert_eq!(bytes, g.snapshot());
                pay(&mut g, Color::Red);
                for _ in 0..2 {
                    let p = g.payment_decision(Seat::P0).unwrap();
                    let c = p.choices[0];
                    g.choose_payment(Seat::P0, p.id, c).unwrap();
                }
                if cancel {
                    let p = g.payment_decision(Seat::P0).unwrap();
                    g.cancel_payment(Seat::P0, p.id).unwrap();
                    assert!(g.objects.get(spell).is_ok());
                } else {
                    finish(&mut g);
                    surprise_pair(&mut g);
                }
                for (i, h) in sources.iter().enumerate() {
                    assert_eq!(g.objects.get(*h).unwrap().tapped, !cancel && i != omitted);
                }
            }
        }
    }
}
