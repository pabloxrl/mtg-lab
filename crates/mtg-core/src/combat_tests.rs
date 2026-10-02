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
    assert_eq!(g.life()[1], -1); // CR 119.6: final life remains negative; GH-72 adjudicates the loss.
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

#[test]
fn combat_shared_xmage_reference_checkpoints() {
    use serde_json::{Value, json};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/vanilla-combat.json"
    ))
    .unwrap();
    fn key(name: &str) -> &'static str {
        match name {
            "Bear Cub" => "bear-cub",
            "Swab Goblin" => "swab-goblin",
            _ => panic!("unsupported reference card"),
        }
    }
    fn name(key: &str) -> &'static str {
        match key {
            "bear-cub" => "Bear Cub",
            "swab-goblin" => "Swab Goblin",
            _ => panic!("unsupported reference card"),
        }
    }
    fn checkpoint(g: &Game, label: &str) -> Value {
        let mut battlefield = vec![];
        let mut graveyard = vec![];
        for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            let mut permanents: Vec<_> = g
                .objects
                .in_zone(Zone::Battlefield)
                .filter(|h| g.objects.get(*h).unwrap().controller == seat)
                .collect();
            permanents.sort_by_key(|h| name(g.objects.get(*h).unwrap().card.identity().key));
            for h in permanents {
                let o = g.objects.get(h).unwrap();
                let c = g.creature_state(h).unwrap();
                battlefield.push(json!({"seat":i,"card":name(o.card.identity().key),
                    "tapped":o.tapped,"damage":c.damage,"power":c.power,"toughness":c.toughness}));
            }
            let mut grave: Vec<_> = g
                .objects
                .in_zone(Zone::Graveyard(seat))
                .map(|h| name(g.objects.get(h).unwrap().card.identity().key))
                .collect();
            grave.sort();
            graveyard.push(grave);
        }
        let mut c =
            json!({"name":label,"life":g.life(),"battlefield":battlefield,"graveyard":graveyard});
        if label == "attackers" || label == "blockers" {
            let attacks = g.combat();
            assert_eq!(attacks.len(), 1);
            c["blocked"] = json!(attacks[0].blocked);
            let mut blockers: Vec<_> = attacks[0]
                .blockers
                .iter()
                .map(|h| name(g.objects.get(*h).unwrap().card.identity().key))
                .collect();
            blockers.sort();
            c["blockers"] = json!(blockers);
        }
        assert_eq!(g.turn_decision().unwrap().actor, Seat::P0);
        assert_eq!(g.objects.in_zone(Zone::Stack).count(), 0);
        c
    }
    for case in fixture["cases"].as_array().unwrap() {
        let mut g = ready();
        g.turns.position = Some((1, Seat::P0, Step::BeginningCombat));
        let a = add(&mut g, Seat::P0, key(case["attacker"].as_str().unwrap()));
        let blockers: Vec<_> = case["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| add(&mut g, Seat::P1, key(b.as_str().unwrap())))
            .collect();
        let mut points = vec![checkpoint(&g, "initial")];
        pair(&mut g);
        select_attack(&mut g, &[a]);
        points.push(checkpoint(&g, "attackers"));
        pair(&mut g);
        select_block(
            &mut g,
            &blockers.iter().map(|b| (*b, a)).collect::<Vec<_>>(),
        );
        points.push(checkpoint(&g, "blockers"));
        pair(&mut g);
        if blockers.len() > 1 {
            let d = g.turn_decision().unwrap();
            let amounts: Vec<_> = blockers
                .iter()
                .zip(case["amounts"].as_array().unwrap())
                .map(|(b, n)| (*b, n.as_u64().unwrap() as u32))
                .collect();
            g.assign_combat_damage(d.actor, d.id, a, &amounts).unwrap();
        }
        let before_damage = g.snapshot();
        for budget in [1, 2, 3, 5, 64] {
            let mut bounded = Game::new().unwrap();
            bounded.restore(&before_damage).unwrap();
            combat_quantum_finish(&mut bounded, budget);
            assert_eq!(
                checkpoint(&bounded, "damage"),
                case["expected"][3],
                "{} at quantum {budget}",
                case["id"]
            );
        }
        damage(&mut g);
        points.push(checkpoint(&g, "damage"));
        assert_eq!(json!(points), case["expected"], "{}", case["id"]);
    }
}

// GH-108: synthetic old vanilla creatures, followed by actual declaration,
// allocation and priority APIs. CR 510.1c/510.2: all assigned damage happens
// simultaneously; a blocked attacker with no blockers deals no player damage.
// CR 704.5a/g: life loss and lethal creatures settle before priority (117.5).
fn combat_quantum_state(g: &Game) -> serde_json::Value {
    fn normalize(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, v) in m {
                    if k == "store" || k == "scope" {
                        *v = 0.into();
                    } else {
                        normalize(v);
                    }
                }
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(normalize),
            _ => (),
        }
    }
    let mut value = serde_json::to_value(g).unwrap();
    normalize(&mut value);
    value["objects"]["id"] = 0.into();
    value
}
fn combat_quantum_finish(g: &mut Game, budget: usize) -> usize {
    let d = g.turn_decision().unwrap();
    let q = NonZeroUsize::new(budget).unwrap();
    let generation = g.generation;
    let rng = serde_json::to_value(&g.rng).unwrap();
    let mut progress = g.finish_combat_quantum(d.actor, d.id, q).unwrap();
    if budget == 1 {
        assert_eq!(
            progress,
            Progress::InternalYield,
            "damage must leave owned work at quantum 1"
        );
    }
    assert_eq!(g.generation, generation + 1);
    let mut yields = 0;
    while progress == Progress::InternalYield {
        yields += 1;
        assert!(yields < 100);
        assert_eq!(g.turn_decision(), None);
        assert_eq!(g.decision(), None);
        assert_eq!(g.outcome(), None);
        let before = g.snapshot();
        for seat in [Seat::P0, Seat::P1] {
            assert_eq!(
                g.observe(seat),
                Err(crate::opening::views::ViewError::Unavailable)
            );
            assert!(g.combat_decision(seat, 80).is_err());
            assert!(g.finish_combat_quantum(seat, d.id, q).is_err());
            assert!(g.finish_combat(seat, d.id).is_err());
            assert!(g.select_attackers(seat, d.id, &[]).is_err());
            assert!(g.select_blockers(seat, d.id, &[]).is_err());
            assert!(
                g.apply_turn(
                    seat,
                    &TurnAction {
                        decision: d.id,
                        selection: TurnSelection::Pass(d.candidate(0)),
                    }
                )
                .is_err()
            );
            assert!(g.concede(seat, g.episode_id().unwrap()).is_err());
            assert!(g.draw_top(seat).is_err());
        }
        assert!(g.reset(&Config::default(), 999, 1).is_err());
        assert_eq!(g.snapshot(), before);
        let mut restored = Game::new().unwrap();
        restored.restore(&before).unwrap();
        let restored_progress = restored.resume(q);
        progress = g.resume(q);
        assert_eq!(combat_quantum_state(&restored), combat_quantum_state(g));
        assert_eq!(
            matches!(restored_progress, Progress::InternalYield),
            matches!(progress, Progress::InternalYield)
        );
    }
    assert_eq!(serde_json::to_value(&g.rng).unwrap(), rng);
    assert_eq!(g.generation, generation + 1);
    assert!(matches!(
        progress,
        Progress::TurnDecision(_) | Progress::Terminal(_)
    ));
    let before = g.snapshot();
    assert!(g.finish_combat_quantum(d.actor, d.id, q).is_err());
    assert!(g.finish_combat(d.actor, d.id).is_err());
    assert_eq!(g.resume(q), progress);
    assert_eq!(g.resume(NonZeroUsize::MAX), progress);
    assert_eq!(g.snapshot(), before);
    yields
}
fn combat_quantum_setup(active: Seat) -> Game {
    let mut g = ready();
    g.turns.position = Some((3, active, Step::BeginningCombat));
    g.set_turn_decision(active, TurnKind::Priority);
    g
}
#[test]
fn combat_quantum_all_modern_allocations_simultaneous_lethal_both_seats() {
    for active in [Seat::P0, Seat::P1] {
        for first in 0..=2 {
            let defender = super::super::turns::opponent(active);
            let mut g = combat_quantum_setup(active);
            let a = add(&mut g, active, "bear-cub");
            let b = add(&mut g, defender, "swab-goblin");
            let c = add(&mut g, defender, "bear-cub");
            pair(&mut g);
            select_attack(&mut g, &[a]);
            pair(&mut g);
            select_block(&mut g, &[(b, a), (c, a)]);
            pair(&mut g);
            let d = g.turn_decision().unwrap();
            // All integer allocations are legal: no ordering or lethal-first rule.
            g.assign_combat_damage(active, d.id, a, &[(b, first), (c, 2 - first)])
                .unwrap();
            let start = g.snapshot();
            let mut scalar = Game::new().unwrap();
            scalar.restore(&start).unwrap();
            damage(&mut scalar);
            let expected = combat_quantum_state(&scalar);
            for budget in [1, 2, 3, 4, 5, 6, 8, 64] {
                let mut actual = Game::new().unwrap();
                actual.restore(&start).unwrap();
                combat_quantum_finish(&mut actual, budget);
                // Independently derived: attacker takes 2+2 and dies. A blocker
                // dies iff assigned 2, otherwise retains exactly the assigned mark.
                assert_eq!(actual.life(), [20, 20]);
                assert_eq!(actual.objects.in_zone(Zone::Graveyard(active)).count(), 1);
                assert_eq!(
                    actual.objects.in_zone(Zone::Graveyard(defender)).count(),
                    usize::from(first != 1)
                );
                let survivors: Vec<_> = actual.objects.in_zone(Zone::Battlefield).collect();
                assert_eq!(survivors.len(), if first == 1 { 2 } else { 1 });
                for h in survivors {
                    let o = actual.objects.get(h).unwrap();
                    assert_eq!(o.controller, defender);
                    let state = actual.creature_state(h).unwrap();
                    assert_eq!(
                        (state.power, state.toughness, state.damage),
                        (2, 2, u32::from(first == 1))
                    );
                    assert!(!o.tapped);
                }
                assert_eq!(
                    actual.turn_position(),
                    Some((3, active, Step::CombatDamage))
                );
                let d = actual.turn_decision().unwrap();
                assert_eq!((d.actor, d.kind), (active, TurnKind::Priority));
                assert!(actual.turns.stack.is_empty());
                assert!(actual.combat().is_empty());
                assert_eq!(actual.outcome(), None);
                assert_eq!(combat_quantum_state(&actual), expected);
            }
        }
    }
}
#[test]
fn combat_quantum_removed_blockers_remember_blocked_status() {
    for active in [Seat::P0, Seat::P1] {
        for remove_both in [false, true] {
            let defender = super::super::turns::opponent(active);
            let mut g = combat_quantum_setup(active);
            let a = add(&mut g, active, "bear-cub");
            let b = add(&mut g, defender, "swab-goblin");
            let c = add(&mut g, defender, "bear-cub");
            pair(&mut g);
            select_attack(&mut g, &[a]);
            pair(&mut g);
            select_block(&mut g, &[(b, a), (c, a)]);
            // Explicit synthetic zone departure at the legal response boundary.
            let departed = g.objects.move_to(b, Zone::Hand(defender)).unwrap();
            let reentered = g.objects.move_to(departed, Zone::Battlefield).unwrap();
            assert_ne!(reentered, b);
            if remove_both {
                g.objects.move_to(c, Zone::Hand(defender)).unwrap();
            }
            pair(&mut g);
            let start = g.snapshot();
            let mut scalar = Game::new().unwrap();
            scalar.restore(&start).unwrap();
            damage(&mut scalar);
            for budget in [1, 2, 3, 4, 5, 64] {
                let mut actual = Game::new().unwrap();
                actual.restore(&start).unwrap();
                combat_quantum_finish(&mut actual, budget);
                assert_eq!(actual.life(), [20, 20]);
                for seat in [active, defender] {
                    assert_eq!(
                        actual.objects.in_zone(Zone::Graveyard(seat)).count(),
                        usize::from(!remove_both)
                    );
                }
                if remove_both {
                    let attacks = actual.combat();
                    assert_eq!(attacks.len(), 1);
                    assert!(attacks[0].blocked);
                    assert!(attacks[0].blockers.is_empty());
                    assert_eq!(
                        actual.creature_state(attacks[0].creature).unwrap().damage,
                        0
                    );
                }
                assert_eq!(combat_quantum_state(&actual), combat_quantum_state(&scalar));
            }
        }
    }
}
#[test]
fn combat_quantum_terminal_after_all_damage_and_deaths() {
    for active in [Seat::P0, Seat::P1] {
        let defender = super::super::turns::opponent(active);
        let mut g = combat_quantum_setup(active);
        let a = add(&mut g, active, "bear-cub");
        let b = add(&mut g, defender, "swab-goblin");
        let c = add(&mut g, active, "swab-goblin");
        let e = add(&mut g, active, "bear-cub");
        g.life[seat_index(defender)] = 1;
        pair(&mut g);
        select_attack(&mut g, &[a, c, e]);
        pair(&mut g);
        select_block(&mut g, &[(b, a)]);
        pair(&mut g);
        let start = g.snapshot();
        let mut scalar = Game::new().unwrap();
        scalar.restore(&start).unwrap();
        damage(&mut scalar);
        for budget in [1, 2, 3, 4, 5, 6, 64] {
            let mut actual = Game::new().unwrap();
            actual.restore(&start).unwrap();
            combat_quantum_finish(&mut actual, budget);
            assert_eq!(actual.life()[seat_index(defender)], -3); // both unblocked 2/2s
            assert_eq!(actual.life()[seat_index(active)], 20);
            assert_eq!(actual.objects.in_zone(Zone::Graveyard(active)).count(), 1);
            assert_eq!(actual.objects.in_zone(Zone::Graveyard(defender)).count(), 1);
            let mut losses = [None; 2];
            losses[seat_index(defender)] = Some(super::super::terminal::LossReason::Life);
            assert_eq!(
                actual.outcome(),
                Some(super::super::terminal::Outcome {
                    winner: Some(active),
                    losses
                })
            );
            assert_eq!(actual.turn_decision(), None);
            assert_eq!(combat_quantum_state(&actual), combat_quantum_state(&scalar));
        }
    }
}

#[test]
fn combat_quantum_preflight_rejections_preserve_exact_snapshot() {
    let mut g = combat_quantum_setup(Seat::P0);
    let a = add(&mut g, Seat::P0, "bear-cub");
    let b = add(&mut g, Seat::P1, "swab-goblin");
    let c = add(&mut g, Seat::P1, "bear-cub");
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    select_block(&mut g, &[(b, a), (c, a)]);
    pair(&mut g);
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.finish_combat_quantum(Seat::P1, d.id, NonZeroUsize::MIN),
        Err(CombatError::Invalid(ApplyError::WrongActor))
    );
    assert_eq!(
        g.finish_combat_quantum(d.actor, d.id, NonZeroUsize::MIN),
        Err(CombatError::MissingDamage)
    );
    assert_eq!(
        g.combat_decision(d.actor, 1),
        Err(CombatError::CapacityExceeded {
            needed: 2,
            capacity: 1
        })
    );
    assert_eq!(
        g.assign_combat_damage(d.actor, d.id, a, &[(b, 2), (b, 0)]),
        Err(CombatError::IllegalDamage)
    );
    assert_eq!(g.snapshot(), before);
    let next = g
        .assign_combat_damage(d.actor, d.id, a, &[(b, 2), (c, 0)])
        .unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.finish_combat_quantum(d.actor, d.id, NonZeroUsize::MIN),
        Err(CombatError::Invalid(ApplyError::StaleDecision))
    );
    assert_eq!(g.snapshot(), before);
    // Explicit arithmetic/generation failure setups must not partially assign damage.
    g.turns
        .modifications
        .push(super::super::targets::Modification {
            handle: b,
            boost: 0,
            damage: u32::MAX,
        });
    let before = g.snapshot();
    assert_eq!(
        g.finish_combat_quantum(next.actor, next.id, NonZeroUsize::MIN),
        Err(CombatError::Turn(TurnError::EffectOverflow))
    );
    assert_eq!(g.snapshot(), before);
    g.turns.modifications.clear();
    g.generation = u64::MAX;
    let before = g.snapshot();
    assert_eq!(
        g.finish_combat_quantum(next.actor, next.id, NonZeroUsize::MIN),
        Err(CombatError::Invalid(ApplyError::DecisionExhausted))
    );
    assert_eq!(g.snapshot(), before);
}

/// GH-18 catalog response-window/simultaneous-damage negatives. CR 117,
/// 508.2, 509.2 and 510.2: Growth is possible after declarations, never inside
/// simultaneous damage/SBA work. Synthetic position; literal 2/2 trade oracle.
#[test]
fn integration_growth_windows_end_before_atomic_combat_damage() {
    for active in [Seat::P0, Seat::P1] {
        let defender = super::super::turns::opponent(active);
        let mut g = combat_quantum_setup(active);
        let a = add(&mut g, active, "bear-cub");
        let b = add(&mut g, defender, "swab-goblin");
        let growth = g
            .objects
            .allocate(
                CardId::from_key("giant-growth").unwrap(),
                active,
                Zone::Hand(active),
            )
            .unwrap();
        g.turns.mana[crate::game::seat_index(active)][4] = 1;
        pair(&mut g);
        select_attack(&mut g, &[a]);
        // Mana empties at step boundaries, so provide the declared synthetic
        // payment pool at each legal response window.
        g.turns.mana[crate::game::seat_index(active)][4] = 1;
        assert_eq!(g.turn_decision().unwrap().actor, active);
        assert!(g.cast_candidates(active).contains(&growth));
        pair(&mut g);
        select_block(&mut g, &[(b, a)]);
        g.turns.mana[crate::game::seat_index(active)][4] = 1;
        assert_eq!(g.turn_decision().unwrap().actor, active);
        assert!(g.cast_candidates(active).contains(&growth));
        pair(&mut g);
        let d = g.turn_decision().unwrap();
        let q = NonZeroUsize::new(1).unwrap();
        let mut p = g.finish_combat_quantum(active, d.id, q).unwrap();
        let mut yields = 0;
        while p == Progress::InternalYield {
            yields += 1;
            assert!(yields < 100);
            let before = g.snapshot();
            for seat in [active, defender] {
                assert!(g.begin_targeted_cast(seat, d.id, growth, 80).is_err());
                assert!(g.cast_candidates(seat).is_empty());
                assert!(g.observe(seat).is_err());
                assert!(g.turn_decision().is_none());
            }
            assert_eq!(g.snapshot(), before);
            p = g.resume(q);
        }
        assert!(yields > 1);
        assert!(g.objects.get(a).is_err());
        assert!(g.objects.get(b).is_err());
        assert_eq!(g.objects.in_zone(Zone::Graveyard(active)).count(), 1);
        assert_eq!(g.objects.in_zone(Zone::Graveyard(defender)).count(), 1);
        assert_eq!(g.objects.get(growth).unwrap().zone, Zone::Hand(active));
        assert_eq!(g.life(), [20, 20]);
        assert_eq!(g.turn_decision().unwrap().actor, active);
    }
}

#[test]
fn creature_mana_druid_blocks_cub_and_survives_two_damage() {
    // CR 302.6 permits even a sick creature to block; CR 510.1/704.5g.
    // Pinned Druid is 1/3 and Cub is 2/2, so both survive this exchange.
    let mut g = ready();
    let cub = add(&mut g, Seat::P0, "bear-cub");
    let druid = add(&mut g, Seat::P1, "druid-of-the-cowl");
    g.turns.sick.push(druid);
    pair(&mut g);
    select_attack(&mut g, &[cub]);
    pair(&mut g);
    select_block(&mut g, &[(druid, cub)]);
    pair(&mut g);
    damage(&mut g);
    let d = g.creature_state(druid).unwrap();
    assert_eq!((d.power, d.toughness, d.damage), (1, 3, 2));
    let c = g.creature_state(cub).unwrap();
    assert_eq!((c.power, c.toughness, c.damage), (2, 2, 1));
    assert_eq!(g.life(), [20, 20]);
}

// GH-196: synthetic flying Cub + Growth-sized boost, not Shivan support.
// CR 702.9b/702.17: flying restricts blockers, reach does not restrict attackers.
#[test]
fn flying_reach_illegal_cub_block_is_atomic() {
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "bear-cub");
    let b = add(&mut g, Seat::P1, "bear-cub");
    g.turns.combat.synthetic_flying.push(a);
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.select_blockers(d.actor, d.id, &[(b, a)]),
        Err(CombatError::IllegalBlocker)
    );
    assert_eq!(g.snapshot(), before);
}

#[test]
fn flying_reach_sentry_blocks_five_five_and_dies() {
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "bear-cub");
    let b = add(&mut g, Seat::P1, "magnigoth-sentry");
    g.turns.combat.synthetic_flying.push(a);
    g.turns
        .modifications
        .push(super::super::targets::Modification {
            handle: a,
            boost: 3,
            damage: 0,
        });
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    select_block(&mut g, &[(b, a)]);
    pair(&mut g);
    damage(&mut g);
    assert!(g.objects.get(b).is_err());
    let c = g.creature_state(a).unwrap();
    assert_eq!((c.power, c.toughness, c.damage), (5, 5, 4));
    assert_eq!(g.life(), [20, 20]);
}

#[test]
fn flying_reach_sentry_attacking_is_blockable_by_cub() {
    let mut g = ready();
    let a = add(&mut g, Seat::P0, "magnigoth-sentry");
    let b = add(&mut g, Seat::P1, "bear-cub");
    pair(&mut g);
    select_attack(&mut g, &[a]);
    pair(&mut g);
    select_block(&mut g, &[(b, a)]);
    pair(&mut g);
    damage(&mut g);
    assert!(g.objects.get(b).is_err());
    assert_eq!(g.creature_state(a).unwrap().damage, 2);
    assert_eq!(g.life(), [20, 20]);
}

#[test]
fn flying_reach_pair_matrix_policy_actions_snapshot_and_quantum() {
    use crate::game::actions;
    use crate::game::policy::{Choice, Submission, VisibleRef, VisibleZone};
    let bf = |row| VisibleRef {
        zone: VisibleZone::Battlefield,
        row,
    };
    for active in [Seat::P0, Seat::P1] {
        let defender = super::super::turns::opponent(active);
        for (key, flying, tapped, allowed) in [
            ("bear-cub", false, false, false),
            ("bear-cub", true, false, true),
            ("magnigoth-sentry", false, false, true),
            ("magnigoth-sentry", false, true, false),
        ] {
            let mut g = combat_quantum_setup(active);
            let a = add(&mut g, active, "bear-cub");
            let ground = add(&mut g, active, "bear-cub");
            let b = add(&mut g, defender, key);
            g.turns.combat.synthetic_flying.push(a);
            if flying {
                g.turns.combat.synthetic_flying.push(b);
            }
            g.objects.get_mut(b).unwrap().tapped = tapped;
            g.turns.sick.push(b); // Sickness is irrelevant to blocking, CR 302.6.
            pair(&mut g);
            select_attack(&mut g, &[a, ground]);
            pair(&mut g);
            let d = g.policy_observe(defender, 256).unwrap().decision.unwrap();
            let f = d.factored.unwrap();
            assert_eq!(
                f.forbidden_blocks,
                if !allowed && !tapped {
                    vec![(bf(2), bf(0))]
                } else {
                    vec![]
                }
            );
            assert_eq!(f.blockers.contains(&bf(2)), !tapped);
            let sub = Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices: vec![Choice::SelectBlockers {
                    blocks: vec![(bf(2), bf(0))],
                }],
            };
            let before = g.snapshot();
            if !allowed {
                assert!(g.apply_policy(defender, &sub, 256).is_err());
                assert_eq!(g.snapshot(), before);
                let d = g.turn_decision().unwrap();
                assert_eq!(
                    g.select_blockers(defender, d.id, &[(b, a)]),
                    Err(CombatError::IllegalBlocker)
                );
                assert_eq!(g.snapshot(), before);
                // The same ground blocker remains legal against the ground attacker.
                if !tapped {
                    select_block(&mut g, &[(b, ground)]);
                }
            } else {
                let encoded = actions::encode(&g, defender, &sub, 256).unwrap();
                let mut restored = Game::new().unwrap();
                restored.restore(&before).unwrap();
                actions::apply(&mut restored, &encoded, 256).unwrap();
                actions::apply(&mut g, &encoded, 256).unwrap();
                assert_eq!(
                    g.policy_observe(defender, 256)
                        .unwrap()
                        .decision
                        .unwrap()
                        .factored,
                    restored
                        .policy_observe(defender, 256)
                        .unwrap()
                        .decision
                        .unwrap()
                        .factored
                );
                let d = g.turn_decision().unwrap();
                g.finish_combat(defender, d.id).unwrap();
                pair(&mut g);
                let saved = g.snapshot();
                let mut expected = None;
                for budget in [1, 2, 64] {
                    let mut actual = Game::new().unwrap();
                    actual.restore(&saved).unwrap();
                    combat_quantum_finish(&mut actual, budget);
                    assert_eq!(actual.life()[seat_index(defender)], 18); // Unblocked ground Cub.
                    assert_eq!(actual.objects.in_zone(Zone::Graveyard(active)).count(), 1);
                    assert_eq!(
                        actual.objects.in_zone(Zone::Graveyard(defender)).count(),
                        usize::from(flying)
                    );
                    let state = combat_quantum_state(&actual);
                    if let Some(ref e) = expected {
                        assert_eq!(&state, e);
                    } else {
                        expected = Some(state);
                    }
                }
            }
        }
    }
}

#[test]
fn flying_reach_reference_literal_checkpoints() {
    use serde_json::{Value, json};
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/flying-reach.json"
    ))
    .unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/flying-reach-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for case in fixture["cases"].as_array().unwrap() {
        let mode = case["mode"].as_str().unwrap();
        let mut g = ready();
        let mut a = add(
            &mut g,
            Seat::P0,
            if mode == "ground" || mode == "cast" {
                "magnigoth-sentry"
            } else {
                "bear-cub"
            },
        );
        let b = if mode == "cast" {
            None
        } else {
            Some(add(
                &mut g,
                Seat::P1,
                if mode == "ground" || mode == "cub-illegal" {
                    "bear-cub"
                } else {
                    "magnigoth-sentry"
                },
            ))
        };
        if mode == "cast" {
            g.turns.position = Some((1, Seat::P0, Step::PrecombatMain));
            a = g.objects.move_to(a, Zone::Hand(Seat::P0)).unwrap();
            // 3 generic + G: one green and three red proves it is not 2GG.
            g.turns.mana[0] = [0, 0, 0, 3, 1, 0];
            let d = g.turn_decision().unwrap();
            g.begin_cast(Seat::P0, d.id, a).unwrap();
            for c in [
                super::super::mana::Color::Green,
                super::super::mana::Color::Red,
                super::super::mana::Color::Red,
                super::super::mana::Color::Red,
            ] {
                let d = g.payment_decision(Seat::P0).unwrap();
                g.choose_payment(Seat::P0, d.id, c).unwrap();
            }
            let d = g.payment_decision(Seat::P0).unwrap();
            g.finish_cast(Seat::P0, d.id).unwrap();
            pair(&mut g);
            a = g.objects.in_zone(Zone::Battlefield).next().unwrap();
            assert!(g.summoning_sick(a));
            assert_eq!(g.mana()[0], [0; 6]);
        } else {
            if mode != "ground" {
                g.turns.combat.synthetic_flying.push(a);
                g.turns
                    .modifications
                    .push(super::super::targets::Modification {
                        handle: a,
                        boost: 3,
                        damage: 0,
                    });
            }
            if mode == "tapped" {
                g.objects.get_mut(b.unwrap()).unwrap().tapped = true;
            }
            pair(&mut g);
            select_attack(&mut g, &[a]);
            pair(&mut g);
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            let legal = g
                .select_blockers(Seat::P1, d.id, &[(b.unwrap(), a)])
                .is_ok();
            assert_eq!(legal, mode != "tapped" && mode != "cub-illegal");
            if !legal {
                assert_eq!(g.snapshot(), before);
            }
            let d = g.turn_decision().unwrap();
            g.finish_combat(Seat::P1, d.id).unwrap();
            if mode == "growth" {
                // Real Growth response after committed block, CR 509.2/608.
                pass(&mut g);
                let spell = g
                    .objects
                    .allocate(
                        CardId::from_key("giant-growth").unwrap(),
                        Seat::P1,
                        Zone::Hand(Seat::P1),
                    )
                    .unwrap();
                g.turns.mana[1][4] = 1;
                let d = g.turn_decision().unwrap();
                g.begin_targeted_cast(Seat::P1, d.id, spell, 80).unwrap();
                let d = g.target_decision(Seat::P1).unwrap();
                g.choose_target(Seat::P1, d.id, b.unwrap()).unwrap();
                let d = g.target_decision(Seat::P1).unwrap();
                g.finish_targets(Seat::P1, d.id).unwrap();
                let d = g.payment_decision(Seat::P1).unwrap();
                g.choose_payment(Seat::P1, d.id, super::super::mana::Color::Green)
                    .unwrap();
                let d = g.payment_decision(Seat::P1).unwrap();
                g.finish_cast(Seat::P1, d.id).unwrap();
                pair(&mut g);
            }
            pair(&mut g);
            damage(&mut g);
        }
        let state = |h| {
            g.objects
                .get(h)
                .ok()
                .and_then(|_| g.creature_state(h))
                .map(|c| [c.power, c.toughness, c.damage])
        };
        let result = json!({"attacker":state(a),"blocker":b.and_then(state),"life":g.life(),"legal":mode!="tapped"&&mode!="cub-illegal"});
        assert_eq!(result, expected[case["id"].as_str().unwrap()]);
        results.insert(case["id"].as_str().unwrap().into(), result);
    }
    if let Ok(path) = std::env::var("MTG_FLYING_REACH_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}
