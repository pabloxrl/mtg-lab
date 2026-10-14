//! Original CR 508/509/510/514 cases. Synthetic positions are explicit;
//! the normal-reset test below uses only policy submissions after start_turns.
use super::*;
fn bf(row: usize) -> VisibleRef {
    reference(VisibleZone::Battlefield, row)
}
fn wire(v: serde_json::Value) -> Choice {
    serde_json::from_value(v).unwrap()
}
fn send(g: &mut Game, s: Seat, v: serde_json::Value) {
    g.policy_observe(s, CAP).unwrap();
    submit(g, s, vec![wire(v)]).unwrap();
}
fn view(g: &Game, s: Seat) -> serde_json::Value {
    serde_json::to_value(g.policy_observe(s, CAP).unwrap()).unwrap()
}
fn finish(g: &mut Game, s: Seat) {
    send(g, s, serde_json::json!({"kind":"finish_combat"}));
}
fn attacks(g: &mut Game, s: Seat, rows: &[usize]) {
    send(
        g,
        s,
        serde_json::json!({"kind":"select_attackers","cards":rows.iter().map(|r|bf(*r)).collect::<Vec<_>>()}),
    );
}
fn blocks(g: &mut Game, s: Seat, rows: &[(usize, usize)]) {
    send(
        g,
        s,
        serde_json::json!({"kind":"select_blockers","blocks":rows.iter().map(|(b,a)|(bf(*b),bf(*a))).collect::<Vec<_>>()}),
    );
}
fn pair(g: &mut Game) {
    pass(g);
    pass(g);
}
fn position(s: Seat, hidden: bool) -> Game {
    let mut g = game(s, hidden);
    keep(&mut g, s);
    g.turns.position = Some((3, s, turns::Step::BeginningCombat));
    // Allocation order is public battlefield order, independent of hidden hands.
    for seat in [s, s, other(s), other(s)] {
        g.objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                seat,
                Zone::Battlefield,
            )
            .unwrap();
    }
    pair(&mut g);
    g
}
fn reject(g: &mut Game, s: Seat, v: serde_json::Value) {
    let d = g.policy_observe(s, CAP).unwrap().decision.unwrap();
    unchanged(
        g,
        s,
        Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![wire(v)],
        },
        CAP,
        PolicyError::InvalidSelection,
    );
}
#[test]
fn policy_combat_independent_subsets_maps_and_allocations() {
    // Two attackers give 2^2 subsets; each of two blockers has three options
    // (none/A/B), giving 3^2 maps. A 2-power attacker has 3 divisions over two blockers.
    for s in [Seat::P0, Seat::P1] {
        for bits in 0..4 {
            let mut g = position(s, false);
            assert_eq!(view(&g, s)["decision"]["kind"], "attackers");
            let rows = (0..2).filter(|r| bits & (1 << r) != 0).collect::<Vec<_>>();
            attacks(&mut g, s, &rows);
            finish(&mut g, s);
            assert_eq!(view(&g, s)["combat"].as_array().unwrap().len(), rows.len());
            for r in 0..2 {
                assert_eq!(
                    g.policy_observe(s, CAP).unwrap().view.public_zones[2].cards[r].tapped,
                    rows.contains(&r)
                );
            }
        }
        for x in 0..3 {
            for y in 0..3 {
                let mut g = position(s, false);
                attacks(&mut g, s, &[0, 1]);
                finish(&mut g, s);
                pair(&mut g);
                let map = [(2, x), (3, y)]
                    .into_iter()
                    .filter(|(_, a)| *a != 0)
                    .map(|(b, a)| (b, a - 1))
                    .collect::<Vec<_>>();
                blocks(&mut g, other(s), &map);
                finish(&mut g, other(s));
                let expected=(0..2).map(|a|serde_json::json!({"attacker":bf(a),"blocked":map.iter().any(|(_,v)|*v==a),"blockers":map.iter().filter(|(_,v)|*v==a).map(|(b,_)|bf(*b)).collect::<Vec<_>>()})).collect::<Vec<_>>();
                assert_eq!(view(&g, s)["combat"], serde_json::json!(expected));
            }
        }
        for n in 0..=2 {
            let mut g = position(s, false);
            attacks(&mut g, s, &[0]);
            finish(&mut g, s);
            pair(&mut g);
            blocks(&mut g, other(s), &[(2, 0), (3, 0)]);
            finish(&mut g, other(s));
            pair(&mut g);
            assert_eq!(view(&g, s)["decision"]["factored"]["damage"][0]["power"], 2);
            let opponent = view(&g, other(s));
            send(
                &mut g,
                s,
                serde_json::json!({"kind":"assign_damage","attacker":bf(0),"amounts":[[bf(2),n],[bf(3),2-n]]}),
            );
            assert_eq!(view(&g, other(s)), opponent);
            finish(&mut g, s);
            assert_eq!(g.life(), [20, 20]);
            assert_eq!(g.objects.in_zone(Zone::Graveyard(s)).count(), 1);
            assert_eq!(
                g.objects.in_zone(Zone::Graveyard(other(s))).count(),
                usize::from(n != 1)
            );
            // 1+1 leaves both blockers alive with one damage each (no lethal-first rule).
            if n == 1 {
                for h in g.objects.in_zone(Zone::Battlefield).skip(1) {
                    assert_eq!(g.creature_state(h).unwrap().damage, 1);
                }
            }
        }
    }
}
#[test]
fn policy_combat_twins_private_backtracking_errors_and_capacity() {
    for s in [Seat::P0, Seat::P1] {
        let mut a = position(s, false);
        let mut b = position(s, true);
        assert_eq!(view(&a, s), view(&b, s));
        let d = a.policy_observe(s, CAP).unwrap().decision.unwrap();
        let before_other = view(&a, other(s));
        for g in [&mut a, &mut b] {
            for cards in [
                vec![bf(0), bf(0)],
                vec![bf(2)],
                vec![bf(99)],
                vec![reference(VisibleZone::Hand, 0)],
            ] {
                reject(
                    g,
                    s,
                    serde_json::json!({"kind":"select_attackers","cards":cards}),
                );
            }
            assert_eq!(g.policy_observe(s, 1), Err(PolicyError::CapacityExceeded));
            attacks(g, s, &[0, 1]);
            attacks(g, s, &[]);
            attacks(g, s, &[0]);
            unchanged(
                g,
                s,
                Submission {
                    schema_version: 1,
                    revision: d.revision,
                    generation: d.generation,
                    choices: vec![wire(serde_json::json!({"kind":"finish_combat"}))],
                },
                CAP,
                PolicyError::StaleDecision,
            );
            let now = g.policy_observe(s, CAP).unwrap().decision.unwrap();
            unchanged(
                g,
                other(s),
                Submission {
                    schema_version: 1,
                    revision: now.revision,
                    generation: now.generation,
                    choices: vec![Choice::Pass],
                },
                CAP,
                PolicyError::WrongActor,
            );
        }
        assert_eq!(view(&a, other(s)), before_other);
        assert_eq!(view(&a, s), view(&b, s));
        for g in [&mut a, &mut b] {
            finish(g, s);
            pair(g);
        }
        assert_eq!(view(&a, s), view(&b, s));
    }
}
#[test]
fn policy_combat_invalid_block_damage_and_finish_preserve_state() {
    for s in [Seat::P0, Seat::P1] {
        let mut g = position(s, false);
        attacks(&mut g, s, &[0]);
        finish(&mut g, s);
        pair(&mut g);
        for map in [
            vec![(bf(2), bf(0)), (bf(2), bf(0))],
            vec![(bf(0), bf(0))],
            vec![(bf(2), bf(1))],
        ] {
            reject(
                &mut g,
                other(s),
                serde_json::json!({"kind":"select_blockers","blocks":map}),
            );
        }
        let opp = view(&g, s);
        blocks(&mut g, other(s), &[(2, 0)]);
        blocks(&mut g, other(s), &[]);
        assert_eq!(view(&g, s), opp);
        blocks(&mut g, other(s), &[(2, 0), (3, 0)]);
        finish(&mut g, other(s));
        pair(&mut g);
        reject(&mut g, s, serde_json::json!({"kind":"finish_combat"}));
        for amounts in [
            vec![(bf(2), 1)],
            vec![(bf(2), 1), (bf(2), 1)],
            vec![(bf(1), 2)],
            vec![(bf(2), u32::MAX), (bf(3), u32::MAX)],
        ] {
            reject(
                &mut g,
                s,
                serde_json::json!({"kind":"assign_damage","attacker":bf(0),"amounts":amounts}),
            );
        }
        send(
            &mut g,
            s,
            serde_json::json!({"kind":"assign_damage","attacker":bf(0),"amounts":[[bf(2),2]]}),
        );
        assert_eq!(
            view(&g, s)["decision"]["legal_mask"],
            serde_json::json!([true])
        );
        finish(&mut g, s);
    }
}
#[test]
fn policy_cleanup_normal_reset_both_seats_and_distinct_sorted_rows() {
    for s in [Seat::P0, Seat::P1] {
        let mut g = game(s, false);
        keep(&mut g, s);
        // No lands played: the nonstarting player draws an eighth card and must discard.
        for _ in 0..50 {
            if matches!(
                g.turn_decision().unwrap().kind,
                turns::TurnKind::Discard { .. }
            ) {
                break;
            }
            pass(&mut g);
        }
        let actor = other(s);
        let o = g.policy_observe(actor, CAP).unwrap();
        assert_eq!(o.decision.as_ref().unwrap().kind, "cleanup_discard");
        assert_eq!(o.decision.as_ref().unwrap().count, 1);
        assert_eq!(o.view.hand.len(), 8);
        assert!(g.policy_observe(s, 0).unwrap().decision.is_none());
        let c = wire(serde_json::json!({"kind":"discard","card":reference(VisibleZone::Hand,0)}));
        let d = o.decision.unwrap();
        unchanged(
            &mut g,
            actor,
            Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices: vec![c.clone(), c.clone()],
            },
            CAP,
            PolicyError::InvalidSelection,
        );
        submit(&mut g, actor, vec![c]).unwrap();
        assert_eq!(g.objects.in_zone(Zone::Graveyard(actor)).count(), 1);
        assert_eq!(g.objects.in_zone(Zone::Hand(actor)).count(), 7);
    }
}
#[test]
fn policy_combat_normal_reset_cast_attack_block_damage_cleanup_both_seats() {
    // Real green mirrors, four Cubs followed by lands in each valid 40-card deck.
    // Two land plays and two separately paid casts per player, then a 2/2 attacks
    // into two 2/2s. CR 510: 1+1 kills only the attacker; cleanup clears both marks.
    for start in [Seat::P0, Seat::P1] {
        let mut cfg = config(start, false);
        cfg.seats = vec![config(Seat::P1, false).seats[0].clone(); 2];
        for deck in &mut cfg.seats {
            deck.order.as_mut().unwrap().sort_by_key(|k| {
                if k == "bear-cub" {
                    0
                } else if k == "forest" {
                    1
                } else {
                    2
                }
            });
        }
        let mut g = Game::new().unwrap();
        g.reset(&cfg, 112, 0).unwrap();
        keep(&mut g, start);
        let mut lands = [0; 2];
        let mut casts = [0; 2];
        let mut completed = false;
        for _ in 0..400 {
            let (turn, active, step) = g.turn_position().unwrap();
            if turn == 8 && step == turns::Step::Upkeep {
                completed = true;
                break;
            }
            let actor = g.turn_decision().unwrap().actor;
            let o = g.policy_observe(actor, CAP).unwrap();
            let d = o.decision.as_ref().unwrap();
            if step == turns::Step::PrecombatMain
                && actor == active
                && lands[seat_index(actor)] < 2
                && let Some((c, _)) = d
                    .candidates
                    .iter()
                    .zip(&d.legal_mask)
                    .find(|(c, m)| **m && matches!(c, Choice::PlayLand { .. }))
            {
                submit(&mut g, actor, vec![c.clone()]).unwrap();
                lands[seat_index(actor)] += 1;
                continue;
            }
            if step == turns::Step::PrecombatMain
                && actor == active
                && lands[seat_index(actor)] == 2
                && casts[seat_index(actor)] < 2
            {
                let cast = d
                    .candidates
                    .iter()
                    .zip(&d.legal_mask)
                    .find(|(c, m)| **m && matches!(c, Choice::Cast { .. }))
                    .map(|(c, _)| c.clone());
                if let Some(c) = cast {
                    submit(&mut g, actor, vec![c]).unwrap();
                    let sources = g
                        .policy_observe(actor, CAP)
                        .unwrap()
                        .decision
                        .unwrap()
                        .candidates
                        .into_iter()
                        .filter(|c| matches!(c, Choice::TapMana { .. }))
                        .collect::<Vec<_>>();
                    for c in sources {
                        let p = g.policy_observe(actor, CAP).unwrap().decision.unwrap();
                        if p.candidates
                            .iter()
                            .zip(&p.legal_mask)
                            .any(|(x, m)| *m && *x == c)
                        {
                            submit(&mut g, actor, vec![c]).unwrap();
                        }
                    }
                    submit(&mut g, actor, vec![Choice::Pay { color: 4 }]).unwrap();
                    submit(&mut g, actor, vec![Choice::Pay { color: 4 }]).unwrap();
                    submit(&mut g, actor, vec![Choice::FinishPayment]).unwrap();
                    casts[seat_index(actor)] += 1;
                    continue;
                }
            }
            match d.kind {
                "attackers" => {
                    if turn == 7 {
                        let rows = d.factored.as_ref().unwrap().attackers.clone();
                        assert_eq!(rows.len(), 2);
                        attacks(&mut g, actor, &[rows[0].row]);
                    } else {
                        attacks(&mut g, actor, &[]);
                    }
                    finish(&mut g, actor);
                }
                "blockers" => {
                    let c = d.factored.as_ref().unwrap();
                    if turn == 7 {
                        assert_eq!(c.blockers.len(), 2);
                        let map = c
                            .blockers
                            .iter()
                            .map(|b| (b.row, c.attackers[0].row))
                            .collect::<Vec<_>>();
                        blocks(&mut g, actor, &map);
                    } else {
                        blocks(&mut g, actor, &[]);
                    }
                    finish(&mut g, actor);
                }
                "combat_damage" => {
                    if turn == 7 {
                        let allocation = &d.factored.as_ref().unwrap().damage[0];
                        assert_eq!(allocation.power, 2);
                        send(
                            &mut g,
                            actor,
                            serde_json::json!({"kind":"assign_damage","attacker":allocation.attacker,"amounts":allocation.blockers.iter().map(|r|(*r,1)).collect::<Vec<_>>()}),
                        );
                    }
                    finish(&mut g, actor);
                    if turn == 7 {
                        assert_eq!(g.objects.in_zone(Zone::Graveyard(start)).count(), 1);
                        assert_eq!(g.objects.in_zone(Zone::Graveyard(other(start))).count(), 0);
                    }
                }
                "cleanup_discard" => {
                    submit(&mut g, actor, d.candidates[..d.count].to_vec()).unwrap();
                }
                "priority" => pass(&mut g),
                k => panic!("unexpected decision {k}"),
            }
        }
        assert!(completed);
        assert_eq!(casts, [2, 2]);
        assert_eq!(g.life(), [20, 20]);
        assert!(g.combat().is_empty());
        for h in g.objects.in_zone(Zone::Battlefield) {
            if let Some(c) = g.creature_state(h) {
                assert_eq!(c.damage, 0);
            }
        }
    }
}
fn check_boundary(g: &mut Game, actor: Seat) {
    let observed = view(g, actor);
    let hidden = g
        .objects
        .in_zone(Zone::Hand(other(actor)))
        .chain(g.objects.in_zone(Zone::Library(Seat::P0)))
        .chain(g.objects.in_zone(Zone::Library(Seat::P1)))
        .map(|h| (h, g.objects.get(h).unwrap().card))
        .collect::<Vec<_>>();
    let d = g.policy_observe(actor, CAP).unwrap().decision.unwrap();
    let mut needed = d.candidates.len();
    if let Some(c) = &d.factored {
        needed += c.attackers.len() + c.blockers.len() + c.selected.len() + c.blocks.len();
        for a in &c.damage {
            needed += 1 + a.blockers.len() + a.amounts.as_ref().map_or(0, Vec::len);
        }
    }
    for variant in [false, true] {
        let game = &mut *g;
        if variant {
            for (h, _) in &hidden {
                game.objects.get_mut(*h).unwrap().card = CardId::from_key("giant-growth").unwrap();
            }
        }
        assert_eq!(view(game, actor), observed);
        assert!(game.policy_observe(actor, needed).is_ok());
        let before = format!("{game:?}");
        assert_eq!(
            game.policy_observe(actor, needed - 1),
            Err(PolicyError::CapacityExceeded)
        );
        assert_eq!(format!("{game:?}"), before);
        for (seat, generation, cap, error) in [
            (other(actor), d.generation, CAP, PolicyError::WrongActor),
            (actor, d.generation - 1, CAP, PolicyError::StaleDecision),
            (
                actor,
                d.generation,
                needed - 1,
                PolicyError::CapacityExceeded,
            ),
        ] {
            unchanged(
                game,
                seat,
                Submission {
                    schema_version: 1,
                    revision: d.revision,
                    generation,
                    choices: vec![Choice::Pass],
                },
                cap,
                error,
            );
        }
        for c in [
            Choice::Pass,
            wire(serde_json::json!({"kind":"discard","card":bf(0)})),
        ] {
            unchanged(
                game,
                actor,
                Submission {
                    schema_version: 1,
                    revision: d.revision,
                    generation: d.generation,
                    choices: vec![c],
                },
                CAP,
                PolicyError::InvalidSelection,
            );
        }
    }
    for (h, card) in hidden {
        g.objects.get_mut(h).unwrap().card = card;
    }
}
#[test]
fn policy_combat_every_boundary_hidden_twins_masks_and_atomic_errors() {
    for s in [Seat::P0, Seat::P1] {
        let mut g = position(s, false);
        check_boundary(&mut g, s);
        attacks(&mut g, s, &[0]);
        check_boundary(&mut g, s);
        finish(&mut g, s);
        pair(&mut g);
        check_boundary(&mut g, other(s));
        blocks(&mut g, other(s), &[(2, 0), (3, 0)]);
        check_boundary(&mut g, other(s));
        finish(&mut g, other(s));
        pair(&mut g);
        check_boundary(&mut g, s);
        send(
            &mut g,
            s,
            serde_json::json!({"kind":"assign_damage","attacker":bf(0),"amounts":[[bf(2),1],[bf(3),1]]}),
        );
        check_boundary(&mut g, s);
        // Replacing a complete allocation is explicit backtracking, still private.
        let public = view(&g, other(s));
        send(
            &mut g,
            s,
            serde_json::json!({"kind":"assign_damage","attacker":bf(0),"amounts":[[bf(3),2]]}),
        );
        assert_eq!(view(&g, other(s)), public);
        assert_eq!(
            view(&g, s)["decision"]["factored"]["damage"][0]["amounts"],
            serde_json::json!([[bf(3), 2]])
        );
        let d = g.policy_observe(s, CAP).unwrap().decision.unwrap();
        g.generation = u64::MAX;
        unchanged(
            &mut g,
            s,
            Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices: vec![wire(serde_json::json!({"kind":"finish_combat"}))],
            },
            CAP,
            PolicyError::CapacityExceeded,
        );
    }
}
#[test]
fn policy_cleanup_independent_pairs_sorted_identity_and_hidden_twins() {
    // Synthetic nine-card hand at End: 514.1 requires exactly two discards.
    // All C(9,2)=36 distinct pairs are expressible, regardless of sorted row order.
    for s in [Seat::P0, Seat::P1] {
        let build = || {
            let mut base = game(s, false);
            keep(&mut base, s);
            base.objects
                .allocate(CardId::from_key("swab-goblin").unwrap(), s, Zone::Hand(s))
                .unwrap();
            base.objects
                .allocate(CardId::from_key("bear-cub").unwrap(), s, Zone::Hand(s))
                .unwrap();
            base.turns.position = Some((3, s, turns::Step::End));
            pair(&mut base);
            base
        };
        let mut base = build();
        check_boundary(&mut base, s);
        let d = base.policy_observe(s, CAP).unwrap().decision.unwrap();
        assert_eq!(d.count, 2);
        assert_eq!(d.legal_mask, vec![true; 9]);
        for x in 0..9 {
            for y in x + 1..9 {
                let mut g = build();
                let hand = g.policy_observe(s, CAP).unwrap().view.hand;
                let expected = [hand[x].card, hand[y].card];
                let choices=[x,y].map(|r|wire(serde_json::json!({"kind":"discard","card":reference(VisibleZone::Hand,r)}))).to_vec();
                let request = Submission {
                    schema_version: 1,
                    revision: d.revision,
                    generation: d.generation,
                    choices,
                };
                g.apply_policy(s, &request, CAP).unwrap();
                let actual = g
                    .objects
                    .in_zone(Zone::Graveyard(s))
                    .map(|h| g.objects.get(h).unwrap().card.identity().key)
                    .collect::<Vec<_>>();
                assert_eq!(actual, expected);
                assert_eq!(g.objects.in_zone(Zone::Hand(s)).count(), 7);
                // Repeating accepted cleanup input cannot discard more cards.
                let before = format!("{g:?}");
                assert!(g.apply_policy(s, &request, CAP).is_err());
                assert_eq!(format!("{g:?}"), before);
            }
        }
        let c = wire(serde_json::json!({"kind":"discard","card":reference(VisibleZone::Hand,0)}));
        unchanged(
            &mut base,
            s,
            Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices: vec![c.clone(), c],
            },
            CAP,
            PolicyError::InvalidSelection,
        );
    }
}
#[test]
fn policy_combat_storage_twins_and_departed_blockers_keep_public_identity() {
    for s in [Seat::P0, Seat::P1] {
        let build = |variant| {
            let mut g = game(s, false);
            keep(&mut g, s);
            g.objects.reset().unwrap();
            if variant {
                g.objects
                    .allocate(
                        CardId::from_key("giant-growth").unwrap(),
                        other(s),
                        Zone::Hand(other(s)),
                    )
                    .unwrap();
            }
            for seat in [s, other(s), other(s)] {
                g.objects
                    .allocate(
                        CardId::from_key("bear-cub").unwrap(),
                        seat,
                        Zone::Battlefield,
                    )
                    .unwrap();
            }
            if !variant {
                g.objects
                    .allocate(
                        CardId::from_key("bite-down").unwrap(),
                        other(s),
                        Zone::Hand(other(s)),
                    )
                    .unwrap();
            }
            g.turns.position = Some((3, s, turns::Step::BeginningCombat));
            g.set_turn_decision(s, turns::TurnKind::Priority);
            pair(&mut g);
            g
        };
        let mut a = build(false);
        let mut b = build(true);
        assert_eq!(view(&a, s), view(&b, s));
        for g in [&mut a, &mut b] {
            attacks(g, s, &[0]);
            finish(g, s);
            pair(g);
            blocks(g, other(s), &[(1, 0)]);
            finish(g, other(s));
        }
        assert_eq!(view(&a, s), view(&b, s));
        for g in [&mut a, &mut b] {
            let h = g.objects.in_zone(Zone::Battlefield).nth(1).unwrap();
            // Explicit synthetic departure/reentry while priority is available.
            let h = g.objects.move_to(h, Zone::Hand(other(s))).unwrap();
            g.objects.move_to(h, Zone::Battlefield).unwrap();
            assert_eq!(
                view(g, s)["combat"],
                serde_json::json!([{"attacker":bf(0),"blocked":true,"blockers":[]}])
            );
            pair(g);
            finish(g, s);
            assert_eq!(g.life(), [20, 20]);
        }
    }
}

// GH-19: CR 510.1c permits 1+1 rather than lethal-first allocation. The
// synthetic four-Cub position isolates restore; reachable games are covered
// separately by tests/played_replay.rs.
#[test]
fn snapshot_nondefault_combat_fresh_process() {
    const INPUT: &str = "MTG_COMBAT_SNAPSHOT_TEST";
    if let Ok(path) = std::env::var(INPUT) {
        let mut g = Game::new().unwrap();
        g.restore(&std::fs::read(path).unwrap()).unwrap();
        let s = g.turn_decision().unwrap().actor;
        assert_eq!(view(&g, s)["decision"]["kind"], "combat_damage");
        assert_eq!(view(&g, s)["decision"]["factored"]["damage"][0]["power"], 2);
        let hidden = view(&g, other(s));
        send(
            &mut g,
            s,
            serde_json::json!({"kind":"assign_damage","attacker":bf(0),"amounts":[[bf(2),1],[bf(3),1]]}),
        );
        assert_eq!(view(&g, other(s)), hidden);
        // Saving an already selected nondefault division must preserve it too.
        let saved = g.snapshot();
        g.restore(&saved).unwrap();
        finish(&mut g, s);
        assert_eq!(g.life(), [20, 20]);
        assert_eq!(g.objects.in_zone(Zone::Graveyard(s)).count(), 1);
        assert_eq!(g.objects.in_zone(Zone::Graveyard(other(s))).count(), 0);
        let survivors = g
            .objects
            .in_zone(Zone::Battlefield)
            .filter(|h| g.objects.get(*h).unwrap().controller == other(s))
            .collect::<Vec<_>>();
        assert_eq!(survivors.len(), 2);
        for h in survivors {
            let c = g.creature_state(h).unwrap();
            assert_eq!((c.power, c.toughness, c.damage), (2, 2, 1));
        }
        return;
    }
    for s in [Seat::P0, Seat::P1] {
        let mut g = position(s, false);
        attacks(&mut g, s, &[0]);
        finish(&mut g, s);
        pair(&mut g);
        blocks(&mut g, other(s), &[(2, 0), (3, 0)]);
        finish(&mut g, other(s));
        pair(&mut g);
        let path =
            std::env::temp_dir().join(format!("combat-snapshot-{}.json", std::process::id()));
        std::fs::write(&path, g.snapshot()).unwrap();
        let output = std::process::Command::new("timeout")
            .arg("30")
            .arg(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "game::policy::tests::combat_policy::snapshot_nondefault_combat_fresh_process",
                "--nocapture",
            ])
            .env(INPUT, &path)
            .env_remove("DISPLAY")
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        std::fs::remove_file(path).unwrap();
        assert!(
            output.status.success(),
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
    }
}
