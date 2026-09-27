//! Original synthetic positions. CR 302.6, 508/509/510, 514.2, 704.5g;
//! literal expectations from pinned vanilla 2/2 card definitions, not engine output.
use super::*;
use crate::opening::turns::{Step, TurnAction, TurnKind, TurnSelection};
fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 71, 0).unwrap();
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
    g.turns.position = Some((3, Seat::P0, Step::BeginningCombat));
    g
}
fn add(g: &mut Game, seat: Seat, key: &str) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), seat, Zone::Battlefield)
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
fn pair(g: &mut Game) {
    pass(g);
    pass(g);
}
fn select_attack(g: &mut Game, hs: &[Handle]) {
    let d = g.turn_decision().unwrap();
    let d = g.select_attackers(d.actor, d.id, hs).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
}
fn select_block(g: &mut Game, bs: &[(Handle, Handle)]) {
    let d = g.turn_decision().unwrap();
    let d = g.select_blockers(d.actor, d.id, bs).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
}
fn damage(g: &mut Game) {
    let d = g.turn_decision().unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
}
fn unchanged<T: std::fmt::Debug + PartialEq>(
    g: &mut Game,
    expected: T,
    f: impl FnOnce(&mut Game) -> T,
) {
    let before = format!("{g:?}");
    assert_eq!(f(g), expected);
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn combat_declarations_and_simultaneous_trade_cub_and_swab() {
    for key in ["bear-cub", "swab-goblin"] {
        let mut g = ready();
        let a = add(&mut g, Seat::P0, "bear-cub");
        let b = add(&mut g, Seat::P1, key);
        pair(&mut g);
        assert_eq!(g.combat_decision(Seat::P0, 80).unwrap().attackers, vec![a]);
        let d = g.turn_decision().unwrap();
        let d = g.select_attackers(d.actor, d.id, &[a]).unwrap();
        assert!(!g.objects.get(a).unwrap().tapped); // provisional choices do not commit
        assert!(g.combat().is_empty());
        assert!(g.combat_decision(Seat::P1, 80).is_err());
        g.finish_combat(d.actor, d.id).unwrap();
        assert!(g.objects.get(a).unwrap().tapped);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        pair(&mut g);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P1);
        select_block(&mut g, &[(b, a)]);
        assert_eq!(
            g.combat(),
            vec![Attack {
                creature: a,
                blocked: true,
                blockers: vec![b]
            }]
        );
        assert!(!g.objects.get(b).unwrap().tapped);
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        pair(&mut g);
        damage(&mut g);
        assert!(g.objects.get(a).is_err());
        assert!(g.objects.get(b).is_err());
        assert_eq!(g.objects.in_zone(Zone::Graveyard(Seat::P0)).count(), 1);
        assert_eq!(g.objects.in_zone(Zone::Graveyard(Seat::P1)).count(), 1);
        assert_eq!(g.life(), [20, 20]);
        assert_eq!(g.turn_decision().unwrap().kind, TurnKind::Priority);
    }
}
#[test]
fn combat_sickness_taps_and_rejections_are_atomic() {
    for sick_key in ["bear-cub", "swab-goblin"] {
        let mut g = ready();
        let a = add(&mut g, Seat::P0, "bear-cub");
        let sick = add(&mut g, Seat::P0, sick_key);
        let b = add(&mut g, Seat::P1, "bear-cub");
        let land = add(&mut g, Seat::P0, "forest");
        g.turns.sick.push(sick);
        pair(&mut g);
        let d = g.turn_decision().unwrap();
        assert_eq!(g.combat_decision(Seat::P0, 80).unwrap().attackers, vec![a]);
        for hs in [vec![a, sick], vec![land], vec![b]] {
            unchanged(&mut g, Err(CombatError::IllegalAttacker), |g| {
                g.select_attackers(d.actor, d.id, &hs)
            });
        }
        unchanged(
            &mut g,
            Err(CombatError::Invalid(ApplyError::DuplicateCandidate)),
            |g| g.select_attackers(d.actor, d.id, &[a, a]),
        );
        unchanged(
            &mut g,
            Err(CombatError::Invalid(ApplyError::WrongActor)),
            |g| g.select_attackers(Seat::P1, d.id, &[a]),
        );
        unchanged(
            &mut g,
            Err(CombatError::CapacityExceeded {
                needed: 1,
                capacity: 0,
            }),
            |g| g.combat_decision(d.actor, 0),
        );
        let next = g.select_attackers(d.actor, d.id, &[a]).unwrap();
        unchanged(
            &mut g,
            Err(CombatError::Invalid(ApplyError::StaleDecision)),
            |g| g.finish_combat(d.actor, d.id),
        );
        unchanged(
            &mut g,
            Err(TurnError::Invalid(ApplyError::WrongKind)),
            |g| {
                g.apply_turn(
                    next.actor,
                    &TurnAction {
                        decision: next.id,
                        selection: TurnSelection::Pass(next.candidate(0)),
                    },
                )
            },
        );
        assert!(g.cast_candidates(d.actor).is_empty());
        assert!(g.mana_sources(d.actor).is_empty());
        g.finish_combat(next.actor, next.id).unwrap();
        pair(&mut g);
        g.objects.get_mut(b).unwrap().tapped = true;
        let d = g.turn_decision().unwrap();
        unchanged(&mut g, Err(CombatError::IllegalBlocker), |g| {
            g.select_blockers(d.actor, d.id, &[(b, a)])
        });
        g.objects.get_mut(b).unwrap().tapped = false;
        g.turns.sick.push(b); // sickness never prevents blocking (302.6).
        unchanged(
            &mut g,
            Err(CombatError::Invalid(ApplyError::DuplicateCandidate)),
            |g| g.select_blockers(d.actor, d.id, &[(b, a), (b, a)]),
        );
        unchanged(&mut g, Err(CombatError::IllegalBlocker), |g| {
            g.select_blockers(d.actor, d.id, &[(b, sick)])
        });
        select_block(&mut g, &[(b, a)]);
        pair(&mut g);
        damage(&mut g);
        assert!(g.objects.get(b).is_err());
        assert!(!g.objects.get(sick).unwrap().tapped);
    }
}

#[test]
fn combat_each_subset_and_blocker_mapping_is_expressible() {
    // Independent enumeration: three attackers => 8 subsets; each of two
    // blockers chooses none or one chosen attacker. No engine-derived oracle.
    for mask in 0..8 {
        let selected: Vec<_> = (0..3).filter(|i| mask & (1 << i) != 0).collect();
        for x in 0..=selected.len() {
            for y in 0..=selected.len() {
                let mut g = ready();
                let a: Vec<_> = (0..3).map(|_| add(&mut g, Seat::P0, "bear-cub")).collect();
                let b: Vec<_> = (0..2)
                    .map(|_| add(&mut g, Seat::P1, "swab-goblin"))
                    .collect();
                pair(&mut g);
                select_attack(&mut g, &selected.iter().map(|i| a[*i]).collect::<Vec<_>>());
                pair(&mut g);
                if selected.is_empty() {
                    assert_eq!(g.turn_position().unwrap().2, Step::EndCombat);
                    continue;
                }
                let mut bs = vec![];
                for (i, j) in [x, y].into_iter().enumerate() {
                    if j > 0 {
                        bs.push((b[i], a[selected[j - 1]]));
                    }
                }
                select_block(&mut g, &bs);
                for attack in g.combat() {
                    let blockers: Vec<_> = bs
                        .iter()
                        .filter(|(_, a)| *a == attack.creature)
                        .map(|(b, _)| *b)
                        .collect();
                    assert_eq!(attack.blocked, !blockers.is_empty());
                    assert_eq!(attack.blockers, blockers);
                }
            }
        }
    }
}
#[test]
fn combat_either_blocked_attacker_and_unblocked_swab_damage() {
    for second in ["bear-cub", "swab-goblin"] {
        for blocked in 0..2 {
            let mut g = ready();
            let a = [
                add(&mut g, Seat::P0, "bear-cub"),
                add(&mut g, Seat::P0, second),
            ];
            let b = add(&mut g, Seat::P1, "bear-cub");
            pair(&mut g);
            select_attack(&mut g, &a);
            pair(&mut g);
            select_block(&mut g, &[(b, a[blocked])]);
            pair(&mut g);
            damage(&mut g);
            assert_eq!(g.life(), [20, 18]);
            assert!(g.objects.get(a[blocked]).is_err());
            assert!(g.objects.get(a[1 - blocked]).unwrap().tapped);
        }
    }
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "swab-goblin");
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    select_block(&mut g, &[]);
    pair(&mut g);
    damage(&mut g);
    assert_eq!(g.life(), [20, 18]);
}
#[test]
fn combat_blocked_status_survives_departed_and_reentered_blocker() {
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "bear-cub");
    let b = add(&mut g, Seat::P1, "bear-cub");
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    select_block(&mut g, &[(b, a)]);
    let grave = g.objects.move_to(b, Zone::Graveyard(Seat::P1)).unwrap();
    let new = g.objects.move_to(grave, Zone::Battlefield).unwrap();
    assert_ne!(b, new);
    pair(&mut g);
    damage(&mut g);
    assert_eq!(g.life(), [20, 20]);
    assert_eq!(g.creature_state(new).unwrap().damage, 0);
    assert_eq!(g.creature_state(a).unwrap().damage, 0);
}
#[test]
fn combat_modern_split_does_not_require_lethal_or_blocker_order() {
    // CR 510.1c: two power may be divided 1+1 across two 2/2s. Both survive.
    for split in [[1, 1], [2, 0], [0, 2]] {
        let mut g = ready();
        let a = add(&mut g, Seat::P0, "bear-cub");
        let b = [
            add(&mut g, Seat::P1, "bear-cub"),
            add(&mut g, Seat::P1, "swab-goblin"),
        ];
        pair(&mut g);
        select_attack(&mut g, &[a]);
        pair(&mut g);
        select_block(&mut g, &[(b[0], a), (b[1], a)]);
        pair(&mut g);
        let d = g.turn_decision().unwrap();
        unchanged(&mut g, Err(CombatError::MissingDamage), |g| {
            g.finish_combat(d.actor, d.id)
        });
        for amounts in [
            vec![(b[0], 1)],
            vec![(b[0], 2), (b[1], 1)],
            vec![(b[0], 1), (b[0], 1)],
            vec![(a, 2)],
        ] {
            unchanged(&mut g, Err(CombatError::IllegalDamage), |g| {
                g.assign_combat_damage(d.actor, d.id, a, &amounts)
            });
        }
        let d = g
            .assign_combat_damage(d.actor, d.id, a, &[(b[1], split[1]), (b[0], split[0])])
            .unwrap();
        assert_eq!(g.creature_state(b[0]).unwrap().damage, 0);
        g.finish_combat(d.actor, d.id).unwrap();
        assert!(g.objects.get(a).is_err());
        for i in 0..2 {
            if split[i] == 2 {
                assert!(g.objects.get(b[i]).is_err());
            } else {
                assert_eq!(g.creature_state(b[i]).unwrap().damage, split[i]);
            }
        }
        pair(&mut g);
        pair(&mut g);
        assert!(g.combat().is_empty());
        pair(&mut g);
        pair(&mut g); // end/cleanup -> next upkeep; damage removed
        for h in b {
            if g.objects.get(h).is_ok() {
                assert_eq!(g.creature_state(h).unwrap().damage, 0);
            }
        }
    }
}
#[test]
fn combat_simultaneous_player_damage_crosses_zero_without_early_stop() {
    let mut g = ready();
    g.life[1] = 3;
    let a = [
        add(&mut g, Seat::P0, "bear-cub"),
        add(&mut g, Seat::P0, "bear-cub"),
    ];
    pair(&mut g);
    select_attack(&mut g, &a);
    pair(&mut g);
    select_block(&mut g, &[]);
    pair(&mut g);
    damage(&mut g);
    assert_eq!(g.life()[1], -1); // CR 119.6; negative life is legal, terminal adjudication is #72.
}
#[test]
fn combat_backtracking_stale_foreign_reset_and_generation_exhaustion() {
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "bear-cub");
    pair(&mut g);
    let mut other = ready();
    let foreign = add(&mut other, Seat::P0, "bear-cub");
    pair(&mut other);
    let d = g.turn_decision().unwrap();
    unchanged(&mut g, Err(CombatError::IllegalAttacker), |g| {
        g.select_attackers(d.actor, d.id, &[foreign])
    });
    unchanged(
        &mut g,
        Err(CombatError::Invalid(ApplyError::StaleDecision)),
        |g| g.select_attackers(d.actor, other.turn_decision().unwrap().id, &[a]),
    );
    let d = g.select_attackers(d.actor, d.id, &[a]).unwrap();
    let d = g.select_attackers(d.actor, d.id, &[]).unwrap();
    assert!(!g.objects.get(a).unwrap().tapped);
    g.generation = u64::MAX;
    unchanged(
        &mut g,
        Err(CombatError::Invalid(ApplyError::DecisionExhausted)),
        |g| g.finish_combat(d.actor, d.id),
    );
    g.generation = d.id.generation;
    let d = g.select_attackers(d.actor, d.id, &[a]).unwrap();
    // Final validation is transactional even if a synthetic caller changed state.
    g.objects.get_mut(a).unwrap().tapped = true;
    unchanged(&mut g, Err(CombatError::IllegalAttacker), |g| {
        g.finish_combat(d.actor, d.id)
    });
    g.reset(&Config::default(), 71, 1).unwrap();
    assert!(g.combat().is_empty());
    assert!(g.turn_decision().is_none());
    unchanged(&mut g, Err(CombatError::NotReady), |g| {
        g.finish_combat(d.actor, d.id)
    });
}
#[test]
fn combat_departed_attacker_tapped_blocker_and_overflow() {
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "bear-cub");
    let b = add(&mut g, Seat::P1, "bear-cub");
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    select_block(&mut g, &[(b, a)]);
    g.objects.get_mut(b).unwrap().tapped = true; // tapping after declaration does not remove combat
    pair(&mut g);
    damage(&mut g);
    assert!(g.objects.get(a).is_err());
    assert!(g.objects.get(b).is_err());
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "bear-cub");
    pair(&mut g);
    select_attack(&mut g, &[a]);
    g.objects.move_to(a, Zone::Graveyard(Seat::P0)).unwrap();
    pair(&mut g);
    assert_eq!(g.turn_position().unwrap().2, Step::DeclareBlockers);
    select_block(&mut g, &[]);
    pair(&mut g);
    damage(&mut g);
    assert_eq!(g.life(), [20, 20]);
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "bear-cub");
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    select_block(&mut g, &[]);
    pair(&mut g);
    g.life[1] = i64::MIN;
    let d = g.turn_decision().unwrap();
    unchanged(
        &mut g,
        Err(CombatError::Turn(TurnError::EffectOverflow)),
        |g| g.finish_combat(d.actor, d.id),
    );
}
#[test]
fn combat_linear_capacity_and_all_remaining_blockers() {
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "bear-cub");
    let bs: Vec<_> = (0..80)
        .map(|_| add(&mut g, Seat::P1, "swab-goblin"))
        .collect();
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    let d = g.turn_decision().unwrap();
    unchanged(
        &mut g,
        Err(CombatError::CapacityExceeded {
            needed: 80,
            capacity: 79,
        }),
        |g| g.combat_decision(d.actor, 79),
    );
    assert_eq!(g.combat_decision(d.actor, 80).unwrap().blockers, bs);
    select_block(&mut g, &bs.iter().map(|b| (*b, a)).collect::<Vec<_>>());
    pair(&mut g);
    let d = g.turn_decision().unwrap();
    assert_eq!(
        g.combat_decision(d.actor, 80).unwrap().damage[0].blockers,
        bs
    );
    unchanged(
        &mut g,
        Err(CombatError::CapacityExceeded {
            needed: 80,
            capacity: 79,
        }),
        |g| g.combat_decision(d.actor, 79),
    );
    // One choice per attacker, not enumeration of 81^80 mappings or allocations.
    let d = g
        .assign_combat_damage(d.actor, d.id, a, &[(bs[79], 2)])
        .unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
    assert!(g.objects.get(a).is_err());
    assert!(g.objects.get(bs[79]).is_err());
    assert_eq!(g.objects.in_zone(Zone::Battlefield).count(), 79);
}
