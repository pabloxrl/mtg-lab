//! GH-212 synthetic decision probes. Independent oracles: CR 508.1,
//! current CR 510.1c (no lethal-first restriction), RFC 0002 section 4.
use super::*;
use crate::game::policy::{Choice, PolicyError, Submission, VisibleRef, VisibleZone};

fn bf(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Battlefield,
        row,
    }
}
fn submit(g: &mut Game, actor: Seat, choice: Choice) {
    let d = g.policy_observe(actor, 256).unwrap().decision.unwrap();
    g.apply_policy(
        actor,
        &Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![choice],
        },
        256,
    )
    .unwrap();
}

#[test]
fn m2_six_five_power_splits_have_literal_damage_and_no_lethal_order() {
    // Stars and bars: x+y=5 gives exactly (0,5)..(5,0), independent
    // of the engine's candidates. Two 4/4 blockers deal eight to a 5/5.
    for active in [Seat::P0, Seat::P1] {
        let opponent = crate::game::turns::opponent(active);
        for x in 0..=5 {
            let mut g = combat_quantum_setup(active);
            let dragon = add(&mut g, active, "shivan-dragon");
            let blockers = [
                add(&mut g, opponent, "magnigoth-sentry"),
                add(&mut g, opponent, "magnigoth-sentry"),
            ];
            pair(&mut g);
            select_attack(&mut g, &[dragon]);
            pair(&mut g);
            select_block(&mut g, &[(blockers[0], dragon), (blockers[1], dragon)]);
            pair(&mut g);
            let hidden = g.policy_observe(opponent, 256).unwrap();
            submit(
                &mut g,
                active,
                Choice::AssignDamage {
                    attacker: bf(0),
                    amounts: vec![(bf(1), x), (bf(2), 5 - x)],
                },
            );
            assert_eq!(g.policy_observe(opponent, 256).unwrap(), hidden);
            let saved = g.snapshot();
            let mut restored = Game::new().unwrap();
            restored.restore(&saved).unwrap();
            submit(&mut g, active, Choice::FinishCombat);
            submit(&mut restored, active, Choice::FinishCombat);
            assert_eq!(
                g.policy_observe(active, 256).unwrap().view,
                restored.policy_observe(active, 256).unwrap().view
            );
            assert!(g.objects.get(dragon).is_err());
            assert_eq!(g.life(), [20, 20]);
            for (blocker, amount) in blockers.into_iter().zip([x, 5 - x]) {
                if amount >= 4 {
                    assert!(g.objects.get(blocker).is_err());
                } else {
                    let c = g.creature_state(blocker).unwrap();
                    assert_eq!((c.power, c.toughness, c.damage), (4, 4, amount));
                }
            }
        }
    }
}

#[test]
fn m2_all_64_distinct_token_subsets_and_explicit_capacity_boundary() {
    for active in [Seat::P0, Seat::P1] {
        let mut base = combat_quantum_setup(active);
        let tokens: Vec<_> = (0..6)
            .map(|_| add(&mut base, active, "goblin-token"))
            .collect();
        pair(&mut base);
        assert_eq!(base.combat_decision(active, 6).unwrap().attackers, tokens);
        let before = base.snapshot();
        assert_eq!(
            base.combat_decision(active, 5),
            Err(CombatError::CapacityExceeded {
                needed: 6,
                capacity: 5
            })
        );
        assert_eq!(base.snapshot(), before);
        let mut seen = std::collections::BTreeSet::new();
        for bits in 0u8..64 {
            let mut g = Game::new().unwrap();
            g.restore(&before).unwrap();
            let tokens: Vec<_> = g.objects.in_zone(Zone::Battlefield).collect();
            let rows: Vec<_> = (0..6).filter(|i| bits & (1 << i) != 0).collect();
            submit(
                &mut g,
                active,
                Choice::SelectAttackers {
                    cards: rows.iter().copied().map(bf).collect(),
                },
            );
            submit(&mut g, active, Choice::FinishCombat);
            let actual: Vec<_> = g.combat().iter().map(|a| a.creature).collect();
            assert_eq!(actual, rows.iter().map(|i| tokens[*i]).collect::<Vec<_>>());
            for (i, h) in tokens.iter().enumerate() {
                assert_eq!(g.objects.get(*h).unwrap().tapped, rows.contains(&i));
            }
            seen.insert(rows);
        }
        assert_eq!(seen.len(), 64);
        assert!(seen.contains(&vec![]));
        assert!(seen.contains(&vec![0, 1, 2, 3, 4, 5]));
        // The caller-declared scalar boundary is linear, not 2^n. The
        // policy boundary must also fail as a whole, never return a prefix.
        let policy = base.policy_observe(active, 7).unwrap().decision.unwrap();
        assert_eq!(policy.candidates, vec![Choice::FinishCombat]);
        assert_eq!(
            policy.factored.unwrap().attackers,
            (0..6).map(bf).collect::<Vec<_>>()
        );
        assert_eq!(
            base.policy_observe(active, 6),
            Err(PolicyError::CapacityExceeded)
        );
        assert_eq!(base.snapshot(), before);
    }
}

#[test]
fn m2_last_token_and_finish_survive_high_board_capacity_boundary() {
    // Declared scalar domain capacity 80; 79/80 fit and 81 must report 81,
    // not return the first 80. The policy adds one explicit finish row.
    for count in [79, 80, 81] {
        let mut g = combat_quantum_setup(Seat::P0);
        let tokens: Vec<_> = (0..count)
            .map(|_| add(&mut g, Seat::P0, "goblin-token"))
            .collect();
        pair(&mut g);
        let before = g.snapshot();
        if count > 80 {
            assert_eq!(
                g.combat_decision(Seat::P0, 80),
                Err(CombatError::CapacityExceeded {
                    needed: 81,
                    capacity: 80
                })
            );
            assert_eq!(
                g.policy_observe(Seat::P0, 81),
                Err(PolicyError::CapacityExceeded)
            );
        } else {
            assert_eq!(g.combat_decision(Seat::P0, 80).unwrap().attackers, tokens);
            let d = g.policy_observe(Seat::P0, 81).unwrap().decision.unwrap();
            assert_eq!(d.candidates, vec![Choice::FinishCombat]);
            assert_eq!(d.legal_mask, vec![true]);
            assert_eq!(
                d.factored.unwrap().attackers,
                (0..count).map(bf).collect::<Vec<_>>()
            );
        }
        assert_eq!(g.snapshot(), before);
        // Raising the explicit caller capacity recovers the complete domain.
        assert_eq!(
            g.combat_decision(Seat::P0, count).unwrap().attackers,
            tokens
        );
        submit(
            &mut g,
            Seat::P0,
            Choice::SelectAttackers {
                cards: vec![bf(count - 1)],
            },
        );
        submit(&mut g, Seat::P0, Choice::FinishCombat);
        assert_eq!(g.combat()[0].creature, tokens[count - 1]);
    }
}

#[test]
fn m2_fresh_vanillas_and_masked_dragon_block_reject_raw_without_mutation() {
    for active in [Seat::P0, Seat::P1] {
        let opponent = crate::game::turns::opponent(active);
        for key in ["bear-cub", "swab-goblin"] {
            let mut g = combat_quantum_setup(active);
            g.turns.position = Some((3, active, Step::PrecombatMain));
            let spell = g
                .objects
                .allocate(CardId::from_key(key).unwrap(), active, Zone::Hand(active))
                .unwrap();
            g.turns.mana[seat_index(active)][3] = 1;
            g.turns.mana[seat_index(active)][4] = 1;
            let d = g.turn_decision().unwrap();
            let p = g.begin_cast(active, d.id, spell).unwrap();
            let first = if key == "bear-cub" {
                mana::Color::Green
            } else {
                mana::Color::Red
            };
            let second = if key == "bear-cub" {
                mana::Color::Red
            } else {
                mana::Color::Green
            };
            let p = g.choose_payment(active, p.id, first).unwrap();
            let p = g.choose_payment(active, p.id, second).unwrap();
            g.finish_cast(active, p.id).unwrap();
            pair(&mut g);
            let h = g
                .objects
                .in_zone(Zone::Battlefield)
                .find(|h| g.objects.get(*h).unwrap().card.identity().key == key)
                .unwrap();
            assert!(g.summoning_sick(h));
            g.turns.position = Some((3, active, Step::BeginningCombat));
            pair(&mut g);
            let before = g.snapshot();
            let d = g.turn_decision().unwrap();
            assert!(g.select_attackers(active, d.id, &[h]).is_err());
            assert_eq!(g.snapshot(), before);
        }
        let mut g = combat_quantum_setup(active);
        let dragon = add(&mut g, active, "shivan-dragon");
        let cub = add(&mut g, opponent, "bear-cub");
        pair(&mut g);
        select_attack(&mut g, &[dragon]);
        pair(&mut g);
        let d = g.policy_observe(opponent, 256).unwrap().decision.unwrap();
        assert_eq!(d.factored.unwrap().forbidden_blocks, vec![(bf(1), bf(0))]);
        let before = g.snapshot();
        let raw = g.turn_decision().unwrap();
        assert_eq!(
            g.select_blockers(opponent, raw.id, &[(cub, dragon)]),
            Err(CombatError::IllegalBlocker)
        );
        assert_eq!(g.snapshot(), before);
        assert!(
            g.apply_policy(
                opponent,
                &Submission {
                    schema_version: 1,
                    revision: d.revision,
                    generation: d.generation,
                    choices: vec![Choice::SelectBlockers {
                        blocks: vec![(bf(1), bf(0))]
                    }]
                },
                256
            )
            .is_err()
        );
        assert_eq!(g.snapshot(), before);
    }
}
