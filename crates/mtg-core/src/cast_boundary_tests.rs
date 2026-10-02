//! Synthetic checkpoints, independently specified by CR 601.2c/f–i and the
//! established cancellation/privacy contract. Normal-reset evidence lives in
//! tests/capture.rs; these positions do not claim played reachability.
use super::*;
use policy::{Choice as C, Submission, VisibleRef, VisibleZone};
const CAP: usize = 256;
fn bf(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Battlefield,
        row,
    }
}
fn send(g: &mut Game, c: C) {
    let d = g.policy_observe(Seat::P0, CAP).unwrap().decision.unwrap();
    g.apply_policy(
        Seat::P0,
        &Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![c],
        },
        CAP,
    )
    .unwrap();
}
fn position() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 131, 0).unwrap();
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
    let hand: Vec<_> = g.objects.in_zone(Zone::Hand(Seat::P0)).collect();
    for h in hand {
        g.objects.remove(h).unwrap();
    }
    for (key, seat, zone) in [
        ("bite-down", Seat::P0, Zone::Hand(Seat::P0)),
        ("bear-cub", Seat::P0, Zone::Battlefield),
        ("bear-cub", Seat::P1, Zone::Battlefield),
        ("forest", Seat::P0, Zone::Battlefield),
        ("mountain", Seat::P0, Zone::Battlefield),
    ] {
        g.objects
            .allocate(CardId::from_key(key).unwrap(), seat, zone)
            .unwrap();
    }
    // Cancellation must preserve the pre-existing pass state, not reset it.
    g.turns.passed = true;
    g
}
fn sequence() -> Vec<C> {
    vec![
        C::Cast {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: 0,
            },
        },
        C::Target { card: bf(0) },
        C::Target { card: bf(1) },
        C::FinishTargets,
        C::TapMana { card: bf(2) },
        C::Pay { color: 4 },
        C::TapMana { card: bf(3) },
        C::Pay { color: 3 },
        C::FinishPayment,
    ]
}
fn assert_uncommitted(g: &Game) {
    assert_eq!(g.objects.in_zone(Zone::Hand(Seat::P0)).count(), 1);
    assert_eq!(g.objects.in_zone(Zone::Stack).count(), 0);
    assert_eq!(g.mana(), [[0; 6]; 2]);
    assert!(
        g.objects
            .in_zone(Zone::Battlefield)
            .all(|h| !g.objects.get(h).unwrap().tapped)
    );
}
#[test]
fn cast_boundary_restore_finish_or_cancel_at_every_target_and_payment_stage() {
    let steps = sequence();
    for stage in 1..steps.len() {
        let mut g = position();
        let opponent = g.policy_observe(Seat::P1, CAP).unwrap();
        for choice in &steps[..stage] {
            send(&mut g, choice.clone());
        }
        let bytes = g.snapshot();
        for cancel in [true, false] {
            let mut restored = Game::new().unwrap();
            restored.restore(&bytes).unwrap();
            assert_uncommitted(&restored);
            assert!(restored.turn_decision().is_none());
            assert_eq!(restored.policy_observe(Seat::P1, CAP).unwrap(), opponent);
            // Restore must retain the whole pending choice (only the destination
            // policy revision changes); stale source capabilities cannot apply.
            let mut actual = restored.policy_observe(Seat::P0, CAP).unwrap();
            let expected = g.policy_observe(Seat::P0, CAP).unwrap();
            actual.decision.as_mut().unwrap().revision =
                expected.decision.as_ref().unwrap().revision;
            assert_eq!(actual, expected);
            let before = restored.snapshot();
            if let Some(p) = g.payment_decision(Seat::P0) {
                assert!(
                    restored
                        .choose_payment(Seat::P0, p.id, mana::Color::Green)
                        .is_err()
                );
            } else {
                let t = g.target_decision(Seat::P0).unwrap();
                assert!(restored.cancel_targets(Seat::P0, t.id).is_err());
            }
            assert_eq!(restored.snapshot(), before);
            if cancel {
                send(
                    &mut restored,
                    if stage < 4 {
                        C::CancelTargets
                    } else {
                        C::CancelPayment
                    },
                );
                assert_uncommitted(&restored);
                assert!(restored.turns.passed);
                assert_eq!(restored.turn_decision().unwrap().actor, Seat::P0);
                assert_eq!(restored.policy_observe(Seat::P1, CAP).unwrap(), opponent);
                assert!(
                    restored
                        .policy_observe(Seat::P0, CAP)
                        .unwrap()
                        .pending
                        .is_none()
                );
            } else {
                for choice in &steps[stage..] {
                    send(&mut restored, choice.clone());
                }
                assert_eq!(restored.objects.in_zone(Zone::Hand(Seat::P0)).count(), 0);
                assert_eq!(restored.mana(), [[0; 6]; 2]);
                let board: Vec<_> = restored.objects.in_zone(Zone::Battlefield).collect();
                assert_eq!(
                    board
                        .iter()
                        .map(|h| restored.objects.get(*h).unwrap().tapped)
                        .collect::<Vec<_>>(),
                    vec![false, false, true, true]
                );
                let stack: Vec<_> = restored.objects.in_zone(Zone::Stack).collect();
                assert_eq!(stack.len(), 1);
                assert_eq!(
                    restored.objects.get(stack[0]).unwrap().card.identity().key,
                    "bite-down"
                );
                assert_eq!(
                    restored.stack_targets(stack[0]),
                    Some(vec![board[0], board[1]])
                );
                assert_eq!(restored.turn_decision().unwrap().actor, Seat::P0);
                assert!(!restored.turns.passed);
            }
        }
    }
}
#[test]
fn cast_boundary_rejections_preserve_snapshot_rng_and_private_choices() {
    let mut g = position();
    let steps = sequence();
    for (stage, choice) in steps.iter().enumerate() {
        let d = g.policy_observe(Seat::P0, CAP).unwrap().decision.unwrap();
        let request = Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![choice.clone()],
        };
        let before = g.snapshot();
        assert_eq!(
            g.apply_policy(Seat::P1, &request, CAP),
            Err(policy::PolicyError::WrongActor)
        );
        let mut stale = request.clone();
        stale.generation += 1;
        assert_eq!(
            g.apply_policy(Seat::P0, &stale, CAP),
            Err(policy::PolicyError::StaleDecision)
        );
        let bad = if stage < 4 {
            C::Target { card: bf(2) }
        } else if stage >= 5 {
            C::TapMana { card: bf(2) }
        } else {
            C::Pay { color: 3 }
        };
        assert_eq!(
            g.apply_policy(
                Seat::P0,
                &Submission {
                    choices: vec![bad],
                    ..request.clone()
                },
                CAP
            ),
            Err(policy::PolicyError::InvalidSelection)
        );
        assert_eq!(
            g.snapshot(),
            before,
            "includes RNG, generations and provisional choices"
        );
        send(&mut g, choice.clone());
    }
}
