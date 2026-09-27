//! Independent expectations: pinned Growth/Bite Oracle; CR 117.3/117.4,
//! 400.7, 601.2c, 608.2b/h, 613.4c, 514.2, 704.5g. Synthetic positions.
use super::*;
use crate::opening::mana::Color;
use crate::opening::turns::{Step, TurnAction, TurnKind, TurnSelection};
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
    // Synthetic postcombat main avoids pretending that #71 combat is implemented.
    g.turns.position = Some((1, Seat::P0, Step::PostcombatMain));
    g.turns.mana = [[0, 0, 0, 4, 30, 0]; 2];
    g
}
fn add(g: &mut Game, key: &str, seat: Seat, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, zone)
        .unwrap()
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
#[test]
fn targets_payable_instants_offered_with_legal_targets() {
    for key in ["giant-growth", "bite-down"] {
        let mut g = ready();
        add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
        add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
        let h = add(&mut g, key, Seat::P0, Zone::Hand(Seat::P0));
        assert!(
            g.cast_candidates(Seat::P0).contains(&h),
            "CR 117/601: payable {key} with required legal targets is available"
        );
    }
}

use crate::opening::targets::{CreatureState, TargetError, TargetKind};
fn cast(g: &mut Game, seat: Seat, key: &str, targets: &[Handle]) -> Handle {
    let h = add(g, key, seat, Zone::Hand(seat));
    let d = g.turn_decision().unwrap();
    assert_eq!(d.actor, seat);
    let mut t = g.begin_targeted_cast(seat, d.id, h, 80).unwrap();
    for &target in targets {
        t = g.choose_target(seat, t.id, target).unwrap();
    }
    assert_eq!(t.kind, TargetKind::Complete);
    let p = g.finish_targets(seat, t.id).unwrap();
    let mut p = g.choose_payment(seat, p.id, Color::Green).unwrap();
    if key == "bite-down" {
        p = g.choose_payment(seat, p.id, Color::Green).unwrap();
    }
    g.finish_cast(seat, p.id).unwrap();
    g.objects.in_zone(Zone::Stack).last().unwrap()
}
fn stats(g: &Game, h: Handle, power: u32, damage: u32) {
    assert_eq!(
        g.creature_state(h),
        Some(CreatureState {
            power,
            toughness: power,
            damage
        })
    );
}
#[test]
fn targets_independent_all_pairs_and_capacity_boundaries() {
    for n in [1, 2, 4, 16, 40, 80] {
        let mut g = ready();
        let own: Vec<_> = (0..n)
            .map(|_| add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield))
            .collect();
        let opp: Vec<_> = (0..n)
            .map(|_| add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield))
            .collect();
        let spell = add(&mut g, "bite-down", Seat::P0, Zone::Hand(Seat::P0));
        let d = g.turn_decision().unwrap();
        let before = format!("{g:?}");
        assert_eq!(
            g.begin_targeted_cast(Seat::P0, d.id, spell, n - 1),
            Err(TargetError::CapacityExceeded {
                needed: n,
                capacity: n - 1
            })
        );
        assert_eq!(format!("{g:?}"), before);
        let t = g.begin_targeted_cast(Seat::P0, d.id, spell, n).unwrap();
        assert_eq!(t.kind, TargetKind::BiteSource);
        assert_eq!(t.choices, own);
        let saved = g.turns.clone();
        let generation = g.generation;
        let mut actual = vec![];
        for &a in &t.choices {
            let u = g.choose_target(Seat::P0, t.id, a).unwrap();
            assert_eq!(u.kind, TargetKind::BiteDestination);
            assert_eq!(u.choices, opp);
            let saved2 = g.turns.clone();
            let gen2 = g.generation;
            for &b in &u.choices {
                let v = g.choose_target(Seat::P0, u.id, b).unwrap();
                assert_eq!(v.kind, TargetKind::Complete);
                actual.push((a, b));
                g.turns = saved2.clone();
                g.generation = gen2;
            }
            g.turns = saved.clone();
            g.generation = generation;
        }
        // Independent Cartesian requirements oracle, not engine candidate output.
        let expected: Vec<_> = own
            .iter()
            .flat_map(|a| opp.iter().map(move |b| (*a, *b)))
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), n * n);
    }
    // Asymmetric roles: the later destination cannot silently exceed capacity.
    let mut g = ready();
    add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    for _ in 0..5 {
        add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    }
    let h = add(&mut g, "bite-down", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    assert_eq!(
        g.begin_targeted_cast(Seat::P0, d.id, h, 4),
        Err(TargetError::CapacityExceeded {
            needed: 5,
            capacity: 4
        })
    );
}
#[test]
fn targets_required_restricted_stale_and_atomic() {
    let mut g = ready();
    let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let b = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    let land = add(&mut g, "forest", Seat::P0, Zone::Battlefield);
    let h = add(&mut g, "bite-down", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    let t = g.begin_targeted_cast(Seat::P0, d.id, h, 8).unwrap();
    let before = format!("{g:?}");
    assert!(g.turn_decision().is_none());
    assert!(g.target_decision(Seat::P1).is_none());
    assert!(g.payment_decision(Seat::P0).is_none());
    assert_eq!(
        g.finish_targets(Seat::P0, t.id),
        Err(TargetError::MissingTargets)
    );
    for bad in [b, land, h] {
        assert_eq!(
            g.choose_target(Seat::P0, t.id, bad),
            Err(TargetError::IllegalTarget)
        );
    }
    assert!(g.choose_target(Seat::P1, t.id, a).is_err());
    assert!(g.begin_payment(Seat::P0, d.id, Default::default()).is_err());
    assert!(g.begin_cast(Seat::P0, d.id, h).is_err());
    assert_eq!(format!("{g:?}"), before);
    let u = g.choose_target(Seat::P0, t.id, a).unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.finish_targets(Seat::P0, u.id),
        Err(TargetError::MissingTargets)
    );
    assert_eq!(
        g.choose_target(Seat::P0, u.id, a),
        Err(TargetError::IllegalTarget)
    );
    assert!(g.choose_target(Seat::P0, t.id, b).is_err());
    assert_eq!(format!("{g:?}"), before);
    // Synthetic zone transition while a continuation exists: stale offered handle
    // must fail rather than silently retargeting the replacement generation.
    let dead = g.objects.move_to(b, Zone::Graveyard(Seat::P1)).unwrap();
    let replacement = g.objects.move_to(dead, Zone::Battlefield).unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.choose_target(Seat::P0, u.id, b),
        Err(TargetError::IllegalTarget)
    );
    assert_eq!(
        g.choose_target(Seat::P0, u.id, replacement),
        Err(TargetError::IllegalTarget)
    );
    assert_eq!(format!("{g:?}"), before);
    g.cancel_targets(Seat::P0, u.id).unwrap();
    assert_eq!(g.mana()[0], [0, 0, 0, 4, 30, 0]);
    assert!(g.objects.get(h).is_ok());
    let growth = add(&mut g, "giant-growth", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    let t = g.begin_targeted_cast(Seat::P0, d.id, growth, 8).unwrap();
    assert_eq!(t.choices, vec![a, replacement]);
    let before = format!("{g:?}");
    assert_eq!(
        g.choose_target(Seat::P0, t.id, land),
        Err(TargetError::IllegalTarget)
    );
    assert_eq!(
        g.finish_targets(Seat::P0, t.id),
        Err(TargetError::MissingTargets)
    );
    assert_eq!(format!("{g:?}"), before);
    let t = g.choose_target(Seat::P0, t.id, replacement).unwrap();
    let p = g.finish_targets(Seat::P0, t.id).unwrap();
    let p = g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
    g.finish_cast(Seat::P0, p.id).unwrap();
    assert_eq!(
        g.stack_targets(*g.turns.stack.last().unwrap()),
        Some(vec![replacement])
    );
    pass(&mut g);
    pass(&mut g);
    stats(&g, replacement, 5, 0);
}
#[test]
fn targets_response_chain_priority_current_power_and_lethal() {
    let mut g = ready();
    let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let b = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    let bite = cast(&mut g, Seat::P0, "bite-down", &[a, b]);
    assert_eq!(g.stack_targets(bite), Some(vec![a, b]));
    stats(&g, a, 2, 0);
    stats(&g, b, 2, 0);
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    assert!(
        g.apply_turn(
            Seat::P1,
            &TurnAction {
                decision: d.id,
                selection: TurnSelection::Pass(d.candidate(0))
            }
        )
        .is_err()
    );
    assert!(g.begin_targeted_cast(Seat::P1, d.id, bite, 80).is_err());
    assert_eq!(format!("{g:?}"), before);
    pass(&mut g);
    let growth = cast(&mut g, Seat::P1, "giant-growth", &[b]);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P1);
    pass(&mut g);
    let top = cast(&mut g, Seat::P0, "giant-growth", &[a]);
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    // Only pass candidate 0 exists; selecting a lower stack row is unavailable.
    assert!(
        g.apply_turn(
            d.actor,
            &TurnAction {
                decision: d.id,
                selection: TurnSelection::Pass(d.candidate(1))
            }
        )
        .is_err()
    );
    assert_eq!(format!("{g:?}"), before);
    pass(&mut g);
    assert_eq!(g.turns.stack, vec![bite, growth, top]);
    pass(&mut g);
    stats(&g, a, 5, 0);
    assert_eq!(g.turns.stack, vec![bite, growth]);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
    pass(&mut g);
    assert_eq!(g.turns.stack, vec![bite, growth]);
    pass(&mut g);
    stats(&g, b, 5, 0);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
    pass(&mut g);
    pass(&mut g);
    assert!(g.objects.get(b).is_err());
    stats(&g, a, 5, 0);
    assert!(g.turns.stack.is_empty());
    let dead = g
        .objects
        .in_zone(Zone::Graveyard(Seat::P1))
        .find(|h| g.objects.get(*h).unwrap().card.identity().key == "bear-cub")
        .unwrap();
    assert_ne!(dead, b);
    stats(&g, dead, 2, 0);
    assert_eq!(g.objects.get(dead).unwrap().owner, Seat::P1);
}
#[test]
fn targets_growth_below_bite_zone_identity_no_retarget() {
    let mut g = ready();
    let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let other = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let b = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    let growth = cast(&mut g, Seat::P0, "giant-growth", &[a]);
    pass(&mut g);
    cast(&mut g, Seat::P1, "bite-down", &[b, a]);
    pass(&mut g);
    pass(&mut g);
    assert!(g.objects.get(a).is_err());
    assert_eq!(g.turns.stack, vec![growth]);
    assert!(g.target_decision(Seat::P0).is_none());
    let dead = g
        .objects
        .in_zone(Zone::Graveyard(Seat::P0))
        .find(|h| g.objects.get(*h).unwrap().card.identity().key == "bear-cub")
        .unwrap();
    let reborn = g.objects.move_to(dead, Zone::Battlefield).unwrap();
    pass(&mut g);
    pass(&mut g);
    assert_eq!(g.last_resolution().unwrap().legal_targets, 0);
    assert!(!g.last_resolution().unwrap().resolved);
    stats(&g, reborn, 2, 0);
    stats(&g, other, 2, 0);
    let h = add(&mut g, "giant-growth", Seat::P0, Zone::Hand(Seat::P0));
    let d = g.turn_decision().unwrap();
    let t = g.begin_targeted_cast(Seat::P0, d.id, h, 8).unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.choose_target(Seat::P0, t.id, a),
        Err(TargetError::IllegalTarget)
    );
    assert_eq!(format!("{g:?}"), before);
    g.cancel_targets(Seat::P0, t.id).unwrap();
    cast(&mut g, Seat::P0, "giant-growth", &[other]);
    pass(&mut g);
    pass(&mut g);
    stats(&g, other, 5, 0);
    stats(&g, reborn, 2, 0);
}
#[test]
fn targets_partial_invalidation_no_lki_or_redirection() {
    for lost in [1, 2, 3] {
        let mut g = ready();
        let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
        let b = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
        let other = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
        cast(&mut g, Seat::P0, "bite-down", &[a, b]);
        // Real response kills source/destination; two additional live creatures
        // are declared to let the both-invalid case kill the remaining target.
        if lost & 1 != 0 {
            pass(&mut g);
            cast(&mut g, Seat::P1, "bite-down", &[b, a]);
            pass(&mut g);
            pass(&mut g);
        }
        if lost & 2 != 0 {
            let c = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
            cast(&mut g, Seat::P0, "bite-down", &[c, b]);
            pass(&mut g);
            pass(&mut g);
        }
        pass(&mut g);
        pass(&mut g);
        let r = g.last_resolution().unwrap();
        assert_eq!(r.legal_targets, if lost == 3 { 0 } else { 1 });
        assert_eq!(r.resolved, lost != 3);
        stats(&g, other, 2, 0);
        assert_eq!(g.life(), [20, 20]);
        if lost == 1 {
            stats(&g, b, 2, 0);
        }
        if lost == 2 {
            stats(&g, a, 2, 0);
        }
    }
    // Catalog's 4/4 Sentry is a declared characteristic-only synthetic object;
    // this test does not implement its casting or Reach.
    let mut g = ready();
    let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let b = add(&mut g, "magnigoth-sentry", Seat::P1, Zone::Battlefield);
    cast(&mut g, Seat::P0, "bite-down", &[a, b]);
    pass(&mut g);
    cast(&mut g, Seat::P1, "bite-down", &[b, a]);
    pass(&mut g);
    pass(&mut g);
    pass(&mut g);
    pass(&mut g);
    stats(&g, b, 4, 0);
}
#[test]
fn targets_cleanup_damage_and_expiration_simultaneous() {
    for (damage, discard) in [(2, false), (4, false), (2, true), (4, true)] {
        let mut g = ready();
        if discard {
            g.draw_top(Seat::P0).unwrap();
        }
        let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
        let b = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
        cast(&mut g, Seat::P0, "bite-down", &[a, b]);
        pass(&mut g);
        cast(&mut g, Seat::P1, "giant-growth", &[b]);
        pass(&mut g);
        pass(&mut g);
        stats(&g, b, 5, 0);
        pass(&mut g);
        pass(&mut g);
        stats(&g, b, 5, 2);
        if damage == 4 {
            cast(&mut g, Seat::P0, "bite-down", &[a, b]);
            pass(&mut g);
            pass(&mut g);
            stats(&g, b, 5, 4);
        }
        // Keep literal damage/boost through End and discard; cleanup 514.2 follows discard.
        pass(&mut g);
        pass(&mut g);
        stats(&g, b, 5, damage);
        pass(&mut g);
        pass(&mut g);
        if discard {
            let d = g.turn_decision().unwrap();
            assert_eq!(d.kind, TurnKind::Discard { count: 1 });
            assert_eq!(g.turn_position(), Some((1, Seat::P0, Step::Cleanup)));
            stats(&g, b, 5, damage);
            g.apply_turn(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Discard(vec![d.candidate(0)]),
                },
            )
            .unwrap();
        }
        stats(&g, b, 2, 0);
        stats(&g, a, 2, 0);
        assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::Upkeep)));
    }
}
#[test]
fn targets_generic_payment_preserves_future_growth_choices() {
    for generic in [Color::Green, Color::Red] {
        let mut g = ready();
        g.turns.mana[0] = [0, 0, 0, 1, 2, 0];
        let cub = add(&mut g, "bear-cub", Seat::P0, Zone::Hand(Seat::P0));
        let growth = add(&mut g, "giant-growth", Seat::P0, Zone::Hand(Seat::P0));
        let d = g.turn_decision().unwrap();
        let p = g.begin_cast(Seat::P0, d.id, cub).unwrap();
        let p = g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
        let p = g.choose_payment(Seat::P0, p.id, generic).unwrap();
        g.finish_cast(Seat::P0, p.id).unwrap();
        let spell = *g.turns.stack.last().unwrap();
        pass(&mut g);
        pass(&mut g);
        let permanent = g.objects.in_zone(Zone::Battlefield).last().unwrap();
        assert_ne!(spell, permanent);
        assert!(g.objects.get(spell).is_err());
        assert_eq!(
            g.cast_candidates(Seat::P0).contains(&growth),
            generic == Color::Red
        );
    }
}

#[test]
fn targets_stacked_growth_and_grown_source_invalid_destination() {
    let mut g = ready();
    let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let b = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    let other = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    cast(&mut g, Seat::P0, "giant-growth", &[a]);
    cast(&mut g, Seat::P0, "giant-growth", &[a]);
    pass(&mut g);
    pass(&mut g);
    stats(&g, a, 5, 0);
    pass(&mut g);
    pass(&mut g);
    stats(&g, a, 8, 0);
    cast(&mut g, Seat::P0, "bite-down", &[a, b]);
    cast(&mut g, Seat::P0, "bite-down", &[a, b]);
    pass(&mut g);
    pass(&mut g);
    assert!(g.objects.get(b).is_err());
    pass(&mut g);
    pass(&mut g);
    assert_eq!(g.last_resolution().unwrap().legal_targets, 1);
    stats(&g, other, 2, 0);
    assert_eq!(g.life(), [20, 20]);
}
#[test]
fn targets_empty_roles_payment_cancel_revalidation_and_exhaustion() {
    let mut g = ready();
    let growth = add(&mut g, "giant-growth", Seat::P0, Zone::Hand(Seat::P0));
    let bite = add(&mut g, "bite-down", Seat::P0, Zone::Hand(Seat::P0));
    assert!(!g.cast_candidates(Seat::P0).contains(&growth));
    assert!(!g.cast_candidates(Seat::P0).contains(&bite));
    let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    assert!(g.cast_candidates(Seat::P0).contains(&growth));
    assert!(!g.cast_candidates(Seat::P0).contains(&bite));
    let land = add(&mut g, "forest", Seat::P0, Zone::Battlefield);
    g.turns.mana[0] = [0; 6];
    let d = g.turn_decision().unwrap();
    let t = g.begin_targeted_cast(Seat::P0, d.id, growth, 1).unwrap();
    let foreign = ready();
    let before = format!("{g:?}");
    assert!(
        g.choose_target(Seat::P0, foreign.turn_decision().unwrap().id, a)
            .is_err()
    );
    assert_eq!(format!("{g:?}"), before);
    let t = g.choose_target(Seat::P0, t.id, a).unwrap();
    let p = g.finish_targets(Seat::P0, t.id).unwrap();
    let p = g.cast_tap_mana(Seat::P0, p.id, land).unwrap();
    let p = g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
    assert!(!g.objects.get(land).unwrap().tapped);
    let before = format!("{g:?}");
    assert!(g.finish_payment(Seat::P0, p.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    // Final cast checks still reject a now-illegal target without consuming staged mana.
    let dead = g.objects.move_to(a, Zone::Graveyard(Seat::P0)).unwrap();
    let before = format!("{g:?}");
    assert!(g.finish_cast(Seat::P0, p.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    g.cancel_payment(Seat::P0, p.id).unwrap();
    assert!(!g.objects.get(land).unwrap().tapped);
    assert_eq!(g.mana()[0], [0; 6]);
    let a = g.objects.move_to(dead, Zone::Battlefield).unwrap();
    let d = g.turn_decision().unwrap();
    let t = g.begin_targeted_cast(Seat::P0, d.id, growth, 1).unwrap();
    g.generation = u64::MAX;
    let before = format!("{g:?}");
    assert!(g.choose_target(Seat::P0, t.id, a).is_err());
    assert!(g.cancel_targets(Seat::P0, t.id).is_err());
    assert_eq!(format!("{g:?}"), before);
    g.generation = t.id.generation;
    g.reset(&Config::default(), 43, 10).unwrap();
    assert!(g.target_decision(Seat::P0).is_none());
    assert!(g.turns.modifications.is_empty());
    assert!(g.turns.effects.is_empty());
}
#[test]
fn targets_resolution_controller_recheck_and_numeric_overflow() {
    let mut g = ready();
    let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let b = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    cast(&mut g, Seat::P0, "bite-down", &[a, b]);
    g.objects.get_mut(b).unwrap().controller = Seat::P0;
    pass(&mut g);
    pass(&mut g);
    stats(&g, b, 2, 0);
    assert_eq!(g.last_resolution().unwrap().legal_targets, 1);
    cast(&mut g, Seat::P0, "giant-growth", &[a]);
    g.turns
        .modifications
        .push(crate::opening::targets::Modification {
            handle: a,
            boost: u32::MAX - 4,
            damage: 0,
        });
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    let before = format!("{g:?}");
    assert_eq!(
        g.apply_turn(
            d.actor,
            &TurnAction {
                decision: d.id,
                selection: TurnSelection::Pass(d.candidate(0))
            }
        ),
        Err(crate::opening::turns::TurnError::EffectOverflow)
    );
    assert_eq!(format!("{g:?}"), before);
}

#[test]
fn targets_hold_priority_bite_above_growth_and_five_no_redirection() {
    let mut g = ready();
    let a = add(&mut g, "bear-cub", Seat::P0, Zone::Battlefield);
    let b = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    let replacement = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    let growth = cast(&mut g, Seat::P0, "giant-growth", &[a]);
    cast(&mut g, Seat::P0, "bite-down", &[a, b]);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
    pass(&mut g);
    pass(&mut g);
    assert!(g.objects.get(b).is_err());
    assert_eq!(g.turns.stack, vec![growth]);
    stats(&g, a, 2, 0);
    pass(&mut g);
    pass(&mut g);
    stats(&g, a, 5, 0);
    let target = add(&mut g, "bear-cub", Seat::P1, Zone::Battlefield);
    cast(&mut g, Seat::P0, "bite-down", &[a, target]);
    cast(&mut g, Seat::P0, "bite-down", &[a, target]);
    pass(&mut g);
    pass(&mut g);
    assert!(g.objects.get(target).is_err());
    pass(&mut g);
    pass(&mut g);
    stats(&g, a, 5, 0);
    stats(&g, replacement, 2, 0);
    assert_eq!(g.life(), [20, 20]);
    assert_eq!(g.last_resolution().unwrap().legal_targets, 1);
}
