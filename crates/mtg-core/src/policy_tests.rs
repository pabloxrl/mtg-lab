use super::super::mana::ManaCost;
use super::*;
use crate::objects::CardId;
const CAP: usize = 256;
fn other(s: Seat) -> Seat {
    if s == Seat::P0 { Seat::P1 } else { Seat::P0 }
}
// Original normal-reset scripts: CR 103 (seven cards), 117 (passes),
// 305.1/2 (one land on own main), 605 (immediate mana). Frozen manifest order
// starts with 16 basic lands, so literal opening hands below are seven lands.
fn config(start: Seat, variant: bool) -> Config {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let seats = [Seat::P0, Seat::P1].map(|s| {
        let name = if s == start || variant {
            "red"
        } else {
            "green"
        };
        let deck = manifest["decks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == name)
            .unwrap();
        let mut order = vec![];
        for e in deck["cards"].as_array().unwrap() {
            for _ in 0..e["copies"].as_u64().unwrap() {
                order.push(e["card_id"].as_str().unwrap().to_owned());
            }
        }
        if variant {
            order[7..].reverse();
        }
        DeckConfig {
            deck: name.into(),
            order: Some(order),
        }
    });
    Config {
        seats: seats.to_vec(),
        starting_seat: seat_index(start) as u8,
        ..Config::default()
    }
}
fn game(start: Seat, variant: bool) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&config(start, variant), 42, 0).unwrap();
    g
}
fn submit(g: &mut Game, s: Seat, choices: Vec<Choice>) -> Result<(), PolicyError> {
    let d = g.policy_observe(s, CAP)?.decision.unwrap();
    g.apply_policy(
        s,
        &Submission {
            revision: d.revision,
            schema_version: 1,
            generation: d.generation,
            choices,
        },
        CAP,
    )
}
fn keep(g: &mut Game, start: Seat) {
    submit(g, start, vec![Choice::Keep]).unwrap();
    submit(g, other(start), vec![Choice::Keep]).unwrap();
    g.start_turns().unwrap();
}
fn pass(g: &mut Game) {
    let actor = g.turn_decision().unwrap().actor;
    submit(g, actor, vec![Choice::Pass]).unwrap();
}
fn reference(zone: VisibleZone, row: usize) -> VisibleRef {
    VisibleRef { zone, row }
}
fn unchanged(g: &mut Game, actor: Seat, action: Submission, cap: usize, error: PolicyError) {
    let before = format!("{g:?}");
    assert_eq!(g.apply_policy(actor, &action, cap), Err(error));
    assert_eq!(
        format!("{g:?}"),
        before,
        "full state/RNG/history must be unchanged"
    );
}
#[test]
fn policy_normal_reset_both_seat_opening_land_mana_pass_literal_states() {
    for start in [Seat::P0, Seat::P1] {
        let mut g = game(start, false);
        let o = g.policy_observe(start, CAP).unwrap();
        assert_eq!(o.schema_version, 1);
        assert_eq!(o.view.life, [20, 20]);
        assert_eq!(o.view.hand_counts, [7, 7]);
        assert_eq!(o.view.library_counts, [33, 33]);
        assert_eq!(
            o.view.hand.iter().map(|c| c.card).collect::<Vec<_>>(),
            vec!["mountain"; 7]
        );
        let d = o.decision.unwrap();
        assert_eq!(d.kind, "keep_or_mulligan");
        assert_eq!(d.candidates, vec![Choice::Keep, Choice::Mulligan]);
        assert_eq!(d.legal_mask, vec![true, true]);
        assert!(
            g.policy_observe(other(start), CAP)
                .unwrap()
                .decision
                .is_none()
        );
        keep(&mut g, start);
        assert_eq!(g.turn_position(), Some((1, start, turns::Step::Upkeep)));
        pass(&mut g);
        pass(&mut g);
        assert_eq!(
            g.turn_position(),
            Some((1, start, turns::Step::PrecombatMain))
        );
        let land = Choice::PlayLand {
            card: reference(VisibleZone::Hand, 0),
        };
        let d = g.policy_observe(start, CAP).unwrap().decision.unwrap();
        assert_eq!(
            d.candidates,
            std::iter::once(Choice::Pass)
                .chain((0..7).map(|row| Choice::PlayLand {
                    card: reference(VisibleZone::Hand, row)
                }))
                .collect::<Vec<_>>()
        );
        assert_eq!(d.legal_mask, vec![true; 8]);
        submit(&mut g, start, vec![land.clone()]).unwrap();
        let o = g.policy_observe(start, CAP).unwrap();
        assert_eq!(o.view.hand_counts[seat_index(start)], 6);
        assert_eq!(o.view.library_counts, [33, 33]);
        assert_eq!(o.view.acting_seat, Some(seat_index(start) as u8));
        assert_eq!(o.view.public_zones[2].cards.len(), 1);
        assert_eq!(o.view.public_zones[2].cards[0].card, "mountain");
        assert!(!o.view.public_zones[2].cards[0].tapped);
        assert_eq!(
            o.decision.unwrap().legal_mask,
            vec![true, false, false, false, false, false, false, true]
        );
        assert_eq!(
            submit(&mut g, start, vec![land]),
            Err(PolicyError::InvalidSelection)
        );
        submit(
            &mut g,
            start,
            vec![Choice::TapMana {
                card: reference(VisibleZone::Battlefield, 0),
            }],
        )
        .unwrap();
        let o = g.policy_observe(start, CAP).unwrap();
        assert!(o.view.public_zones[2].cards[0].tapped);
        assert_eq!(o.view.mana[seat_index(start)], [0, 0, 0, 1, 0, 0]);
        assert_eq!(
            o.decision.unwrap().legal_mask,
            vec![true, false, false, false, false, false, false, false]
        );
        pass(&mut g);
        assert_eq!(
            g.policy_observe(other(start), CAP).unwrap().view.mana[seat_index(start)],
            [0, 0, 0, 1, 0, 0]
        );
        pass(&mut g);
        assert_eq!(
            g.turn_position(),
            Some((1, start, turns::Step::BeginningCombat))
        );
        assert_eq!(g.mana(), [[0; 6]; 2]);
        // Continue the same normal game through empty combat and cleanup so
        // the other seat also draws, plays a Forest, floats G and passes.
        for _ in 0..5 {
            pass(&mut g);
            pass(&mut g);
        }
        let second = other(start);
        assert_eq!(g.turn_position(), Some((2, second, turns::Step::Upkeep)));
        pass(&mut g);
        pass(&mut g);
        assert_eq!(g.turn_position(), Some((2, second, turns::Step::Draw)));
        let o = g.policy_observe(second, CAP).unwrap();
        assert_eq!(o.view.hand_counts[seat_index(second)], 8);
        assert_eq!(o.view.library_counts[seat_index(second)], 32);
        assert_eq!(
            o.view.hand.iter().map(|c| c.card).collect::<Vec<_>>(),
            vec!["forest"; 8]
        );
        pass(&mut g);
        pass(&mut g);
        submit(
            &mut g,
            second,
            vec![Choice::PlayLand {
                card: reference(VisibleZone::Hand, 0),
            }],
        )
        .unwrap();
        submit(
            &mut g,
            second,
            vec![Choice::TapMana {
                card: reference(VisibleZone::Battlefield, 1),
            }],
        )
        .unwrap();
        let o = g.policy_observe(second, CAP).unwrap();
        assert_eq!(o.view.hand_counts[seat_index(second)], 7);
        assert_eq!(
            o.view.public_zones[2]
                .cards
                .iter()
                .map(|c| (c.card, c.tapped))
                .collect::<Vec<_>>(),
            vec![("mountain", true), ("forest", true)]
        );
        assert_eq!(o.view.mana[seat_index(second)], [0, 0, 0, 0, 1, 0]);
        assert_eq!(o.view.life, [20, 20]);
        pass(&mut g);
        pass(&mut g);
        assert_eq!(g.mana(), [[0; 6]; 2]);
    }
}
#[test]
fn policy_hidden_twins_and_invalid_inputs_preserve_full_state() {
    for start in [Seat::P0, Seat::P1] {
        let mut a = game(start, false);
        let mut b = game(start, true);
        for stage in 0..4 {
            let oa = a.policy_observe(start, CAP).unwrap();
            let ob = b.policy_observe(start, CAP).unwrap();
            assert_eq!(
                serde_json::to_vec(&oa).unwrap(),
                serde_json::to_vec(&ob).unwrap()
            );
            let generation = oa.decision.unwrap().generation;
            for (actor, version, submitted_generation, choice, cap, error) in [
                (
                    other(start),
                    1,
                    generation,
                    Choice::Pass,
                    CAP,
                    PolicyError::WrongActor,
                ),
                (
                    start,
                    2,
                    generation,
                    Choice::Pass,
                    CAP,
                    PolicyError::UnsupportedVersion,
                ),
                (
                    start,
                    1,
                    generation - 1,
                    Choice::Pass,
                    CAP,
                    PolicyError::StaleDecision,
                ),
                (
                    start,
                    1,
                    generation,
                    Choice::PlayLand {
                        card: reference(VisibleZone::Hand, usize::MAX),
                    },
                    CAP,
                    PolicyError::InvalidSelection,
                ),
                (
                    start,
                    1,
                    generation,
                    Choice::TapMana {
                        card: reference(VisibleZone::Battlefield, usize::MAX),
                    },
                    CAP,
                    PolicyError::InvalidSelection,
                ),
                (
                    start,
                    1,
                    generation,
                    Choice::Spell,
                    CAP,
                    PolicyError::UnsupportedSpell,
                ),
                (
                    start,
                    1,
                    generation,
                    Choice::Combat,
                    CAP,
                    PolicyError::UnsupportedCombat,
                ),
                (
                    start,
                    1,
                    generation,
                    Choice::Pass,
                    0,
                    PolicyError::CapacityExceeded,
                ),
            ] {
                for g in [&mut a, &mut b] {
                    unchanged(
                        g,
                        actor,
                        Submission {
                            revision: 0,
                            schema_version: version,
                            generation: submitted_generation,
                            choices: vec![choice.clone()],
                        },
                        cap,
                        error,
                    );
                }
            }
            for g in [&mut a, &mut b] {
                let before = format!("{g:?}");
                assert_eq!(
                    g.policy_observe(start, 0),
                    Err(PolicyError::CapacityExceeded)
                );
                assert_eq!(format!("{g:?}"), before);
                match stage {
                    0 => keep(g, start),
                    1 => {
                        pass(g);
                        pass(g);
                    }
                    2 => {
                        submit(
                            g,
                            start,
                            vec![Choice::PlayLand {
                                card: reference(VisibleZone::Hand, 0),
                            }],
                        )
                        .unwrap();
                    }
                    _ => (),
                }
            }
        }
    }
}
#[test]
fn policy_standalone_payment_is_private_and_explicit() {
    // Synthetic public pool RGG, trusted rules supply 1G. CR 107.4: G is
    // compulsory; either R or G pays generic. Policy cannot invent a cost.
    for start in [Seat::P0, Seat::P1] {
        let mut g = game(start, false);
        keep(&mut g, start);
        g.turns.mana[seat_index(start)] = [0, 0, 0, 1, 2, 0];
        let id = g.turn_decision().unwrap().id;
        g.begin_payment(
            start,
            id,
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
            },
        )
        .unwrap();
        let original = g.policy_observe(other(start), CAP).unwrap();
        assert!(original.decision.is_none());
        let d = g.policy_observe(start, CAP).unwrap().decision.unwrap();
        assert_eq!(d.kind, "payment");
        assert_eq!(
            d.candidates,
            (0..6)
                .map(|color| Choice::Pay { color })
                .chain([Choice::FinishPayment, Choice::CancelPayment])
                .collect::<Vec<_>>()
        );
        assert_eq!(
            d.legal_mask,
            vec![false, false, false, false, true, false, false, true]
        );
        submit(&mut g, start, vec![Choice::Pay { color: 4 }]).unwrap();
        assert_eq!(g.policy_observe(other(start), CAP).unwrap(), original);
        assert_eq!(
            g.policy_observe(start, CAP)
                .unwrap()
                .decision
                .unwrap()
                .legal_mask,
            vec![false, false, false, true, true, false, false, true]
        );
        unchanged(
            &mut g,
            start,
            Submission {
                revision: 0,
                schema_version: 1,
                generation: d.generation,
                choices: vec![Choice::Pay { color: 3 }],
            },
            CAP,
            PolicyError::StaleDecision,
        );
        assert_eq!(
            submit(&mut g, start, vec![Choice::FinishPayment]),
            Err(PolicyError::InvalidSelection)
        );
        submit(&mut g, start, vec![Choice::Pay { color: 3 }]).unwrap();
        assert_eq!(g.policy_observe(other(start), CAP).unwrap(), original);
        submit(&mut g, start, vec![Choice::FinishPayment]).unwrap();
        assert_eq!(g.mana()[seat_index(start)], [0, 0, 0, 0, 1, 0]);
        let id = g.turn_decision().unwrap().id;
        g.begin_payment(
            start,
            id,
            ManaCost {
                generic: 1,
                ..ManaCost::default()
            },
        )
        .unwrap();
        submit(&mut g, start, vec![Choice::Pay { color: 4 }]).unwrap();
        submit(&mut g, start, vec![Choice::CancelPayment]).unwrap();
        assert_eq!(g.mana()[seat_index(start)], [0, 0, 0, 0, 1, 0]);
    }
}
#[test]
fn policy_pending_spell_is_private_combat_and_internal_work_fail_explicitly() {
    let mut g = game(Seat::P0, false);
    keep(&mut g, Seat::P0);
    // Synthetic current combat decision, not a fake implementation of combat.
    g.turns.decision.as_mut().unwrap().kind =
        turns::TurnKind::Combat(combat::CombatKind::Attackers);
    assert_eq!(
        g.policy_observe(Seat::P0, CAP),
        Err(PolicyError::UnsupportedCombat)
    );
    g.turns.decision.as_mut().unwrap().kind = turns::TurnKind::Priority;
    g.turns.position = Some((1, Seat::P0, turns::Step::PrecombatMain));
    let h = g
        .objects
        .allocate(
            CardId::from_key("swab-goblin").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    g.turns.mana[0] = [0, 0, 0, 2, 0, 0];
    let id = g.turn_decision().unwrap().id;
    g.begin_cast(Seat::P0, id, h).unwrap();
    let acting = g.policy_observe(Seat::P0, CAP).unwrap();
    assert_eq!(acting.decision.unwrap().kind, "payment");
    assert!(acting.pending.is_some());
    let opponent = g.policy_observe(Seat::P1, CAP).unwrap();
    assert!(opponent.pending.is_none());
    assert!(opponent.decision.is_none());
    assert!(opponent.stack.is_empty());
    assert_eq!(opponent.view.mana[0], [0, 0, 0, 2, 0, 0]);
    g.reset_quantum(&Config::default(), 1, 0, NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert_eq!(
        g.policy_observe(Seat::P0, CAP),
        Err(PolicyError::Unavailable)
    );
}
#[test]
fn policy_mulligans_ordered_bottoming_and_old_opening_api_coexist() {
    for start in [Seat::P0, Seat::P1] {
        let mut g = game(start, false);
        for seat in [start, other(start)] {
            submit(&mut g, seat, vec![Choice::Mulligan]).unwrap();
        }
        // Each redraw bottoms the cumulative count before the next round's
        // keep/mulligan declarations (the existing CR 103 opening contract).
        for seat in [start, other(start)] {
            let generation = g
                .policy_observe(seat, CAP)
                .unwrap()
                .decision
                .unwrap()
                .generation;
            g.apply_opening_view(seat, generation, &[0]).unwrap();
        }
        for seat in [start, other(start)] {
            submit(&mut g, seat, vec![Choice::Mulligan]).unwrap();
        }
        for seat in [start, other(start)] {
            let o = g.policy_observe(seat, CAP).unwrap();
            let d = o.decision.unwrap();
            assert_eq!(d.kind, "bottom");
            assert_eq!(d.count, 2);
            assert_eq!(d.legal_mask, vec![true; 7]);
            assert_eq!(
                d.candidates,
                (0..7)
                    .map(|row| Choice::Bottom {
                        card: reference(VisibleZone::Hand, row)
                    })
                    .collect::<Vec<_>>()
            );
            let bottom = |row| Choice::Bottom {
                card: reference(VisibleZone::Hand, row),
            };
            unchanged(
                &mut g,
                seat,
                Submission {
                    revision: 0,
                    schema_version: 1,
                    generation: d.generation,
                    choices: vec![bottom(0), bottom(0)],
                },
                CAP,
                PolicyError::InvalidSelection,
            );
            // CR 103: ordered bottom sequence, independent expected last two
            // identities taken from the selected input cards, not generated output.
            let expected = [o.view.hand[1].card, o.view.hand[0].card];
            if seat == start {
                submit(&mut g, seat, vec![bottom(1), bottom(0)]).unwrap();
            } else {
                g.apply_opening_view(seat, d.generation, &[1, 0]).unwrap();
            }
            assert_eq!(g.observe(seat).unwrap().hand.len(), 5);
            let keys = g
                .objects
                .in_zone(Zone::Library(seat))
                .map(|h| g.objects.get(h).unwrap().card.identity().key)
                .collect::<Vec<_>>();
            assert_eq!(&keys[keys.len() - 2..], expected);
        }
        for seat in [start, other(start)] {
            submit(&mut g, seat, vec![Choice::Keep]).unwrap();
        }
        assert!(g.policy_observe(start, CAP).unwrap().decision.is_none());
    }
}
#[test]
fn policy_generation_and_resource_exhaustion_are_transactional() {
    let mut g = game(Seat::P0, false);
    let old = g
        .policy_observe(Seat::P0, CAP)
        .unwrap()
        .decision
        .unwrap()
        .generation;
    g.reset(&config(Seat::P0, false), 42, 0).unwrap();
    unchanged(
        &mut g,
        Seat::P0,
        Submission {
            revision: 0,
            schema_version: 1,
            generation: old,
            choices: vec![Choice::Keep],
        },
        CAP,
        PolicyError::StaleDecision,
    );
    keep(&mut g, Seat::P0);
    pass(&mut g);
    pass(&mut g);
    let id = g.turn_decision().unwrap().id;
    g.generation = u64::MAX; // declared synthetic resource boundary
    unchanged(
        &mut g,
        Seat::P0,
        Submission {
            revision: 0,
            schema_version: 1,
            generation: id.generation,
            choices: vec![Choice::PlayLand {
                card: reference(VisibleZone::Hand, 0),
            }],
        },
        CAP,
        PolicyError::CapacityExceeded,
    );
    g.generation = id.generation;
    submit(
        &mut g,
        Seat::P0,
        vec![Choice::PlayLand {
            card: reference(VisibleZone::Hand, 0),
        }],
    )
    .unwrap();
    g.turns.mana[0][3] = u32::MAX;
    let id = g.turn_decision().unwrap().id;
    unchanged(
        &mut g,
        Seat::P0,
        Submission {
            revision: 0,
            schema_version: 1,
            generation: id.generation,
            choices: vec![Choice::TapMana {
                card: reference(VisibleZone::Battlefield, 0),
            }],
        },
        CAP,
        PolicyError::CapacityExceeded,
    );
}
#[test]
fn policy_visible_references_ignore_hidden_storage_and_track_visible_rows() {
    // Synthetic allocation twins: the same visible cards and events, different
    // private identities and slots. RFC B016/B033 forbids slot-based policy IDs.
    let build = |variant| {
        let mut g = game(Seat::P0, false);
        keep(&mut g, Seat::P0);
        g.objects.reset().unwrap();
        if variant {
            g.objects
                .allocate(
                    CardId::from_key("giant-growth").unwrap(),
                    Seat::P1,
                    Zone::Hand(Seat::P1),
                )
                .unwrap();
        }
        for key in if variant {
            ["swab-goblin", "mountain"]
        } else {
            ["mountain", "swab-goblin"]
        } {
            g.objects
                .allocate(
                    CardId::from_key(key).unwrap(),
                    Seat::P0,
                    Zone::Hand(Seat::P0),
                )
                .unwrap();
        }
        if !variant {
            g.objects
                .allocate(
                    CardId::from_key("bite-down").unwrap(),
                    Seat::P1,
                    Zone::Hand(Seat::P1),
                )
                .unwrap();
        }
        g.objects
            .allocate(
                CardId::from_key("forest").unwrap(),
                Seat::P1,
                Zone::Battlefield,
            )
            .unwrap();
        g.objects
            .allocate(
                CardId::from_key("mountain").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        g.turns.position = Some((1, Seat::P0, turns::Step::PrecombatMain));
        // Match a valid decision to the deliberately replaced object store.
        g.set_turn_decision(Seat::P0, turns::TurnKind::Priority);
        g
    };
    let mut a = build(false);
    let mut b = build(true);
    assert_eq!(
        a.policy_observe(Seat::P0, CAP),
        b.policy_observe(Seat::P0, CAP)
    );
    let o = a.policy_observe(Seat::P0, CAP).unwrap();
    assert_eq!(
        o.decision.unwrap().legal_mask,
        vec![true, true, false, false, false, true]
    );
    for g in [&mut a, &mut b] {
        let before = format!("{g:?}");
        assert_eq!(
            submit(
                g,
                Seat::P0,
                vec![Choice::TapMana {
                    card: reference(VisibleZone::Battlefield, 0)
                }]
            ),
            Err(PolicyError::InvalidSelection)
        );
        assert_eq!(format!("{g:?}"), before);
        submit(
            g,
            Seat::P0,
            vec![Choice::TapMana {
                card: reference(VisibleZone::Battlefield, 1),
            }],
        )
        .unwrap();
        assert_eq!(g.mana()[0], [0, 0, 0, 1, 0, 0]);
    }
    assert_eq!(
        a.policy_observe(Seat::P0, CAP),
        b.policy_observe(Seat::P0, CAP)
    );
}
#[test]
fn policy_library_only_twins_payment_errors_and_masked_rows() {
    for start in [Seat::P0, Seat::P1] {
        for library in 0..2 {
            let mut config_b = config(start, false);
            config_b.seats[library].order.as_mut().unwrap()[7..].reverse();
            let mut a = game(start, false);
            let mut b = Game::new().unwrap();
            b.reset(&config_b, 99, 0).unwrap();
            for g in [&mut a, &mut b] {
                keep(g, start);
                pass(g);
                pass(g);
                submit(
                    g,
                    start,
                    vec![Choice::PlayLand {
                        card: reference(VisibleZone::Hand, 0),
                    }],
                )
                .unwrap();
                submit(
                    g,
                    start,
                    vec![Choice::TapMana {
                        card: reference(VisibleZone::Battlefield, 0),
                    }],
                )
                .unwrap();
            }
            assert_eq!(a.policy_observe(start, CAP), b.policy_observe(start, CAP));
            for g in [&mut a, &mut b] {
                let d = g.policy_observe(start, CAP).unwrap().decision.unwrap();
                assert!(g.policy_observe(start, d.candidates.len()).is_ok());
                assert_eq!(
                    g.policy_observe(start, d.candidates.len() - 1),
                    Err(PolicyError::CapacityExceeded)
                );
                for (choice, legal) in d.candidates.iter().zip(&d.legal_mask) {
                    if !legal {
                        unchanged(
                            g,
                            start,
                            Submission {
                                revision: 0,
                                schema_version: 1,
                                generation: d.generation,
                                choices: vec![choice.clone()],
                            },
                            CAP,
                            PolicyError::InvalidSelection,
                        );
                    }
                }
                let id = g.turn_decision().unwrap().id;
                g.begin_payment(
                    start,
                    id,
                    ManaCost {
                        generic: 1,
                        ..ManaCost::default()
                    },
                )
                .unwrap();
            }
            assert_eq!(a.policy_observe(start, CAP), b.policy_observe(start, CAP));
            let generation = a
                .policy_observe(start, CAP)
                .unwrap()
                .decision
                .unwrap()
                .generation;
            for g in [&mut a, &mut b] {
                for choice in [
                    Choice::Pass,
                    Choice::Pay { color: 4 },
                    Choice::Pay { color: 255 },
                    Choice::FinishPayment,
                ] {
                    unchanged(
                        g,
                        start,
                        Submission {
                            revision: 0,
                            schema_version: 1,
                            generation,
                            choices: vec![choice],
                        },
                        CAP,
                        PolicyError::InvalidSelection,
                    );
                }
                unchanged(
                    g,
                    other(start),
                    Submission {
                        revision: 0,
                        schema_version: 1,
                        generation,
                        choices: vec![Choice::CancelPayment],
                    },
                    CAP,
                    PolicyError::WrongActor,
                );
                unchanged(
                    g,
                    start,
                    Submission {
                        revision: 0,
                        schema_version: 1,
                        generation,
                        choices: vec![Choice::Pay { color: 3 }],
                    },
                    7,
                    PolicyError::CapacityExceeded,
                );
                submit(g, start, vec![Choice::Pay { color: 3 }]).unwrap();
            }
            assert_eq!(a.policy_observe(start, CAP), b.policy_observe(start, CAP));
            assert_eq!(
                a.policy_observe(other(start), CAP),
                b.policy_observe(other(start), CAP)
            );
        }
    }
}
#[test]
fn policy_wire_contract_owned_records_and_terminal_boundary() {
    let mut g = game(Seat::P0, false);
    let o = g.policy_observe(Seat::P0, CAP).unwrap();
    let saved = serde_json::to_vec(&o).unwrap();
    let text = String::from_utf8(saved.clone()).unwrap();
    for forbidden in [
        "scope",
        "slot",
        "epoch",
        "rng",
        "seed",
        "Handle",
        "library_order",
    ] {
        assert!(!text.contains(forbidden));
    }
    assert_eq!(o.unsupported_families, ["combat", "cleanup_discard"]);
    let generation = o.decision.as_ref().unwrap().generation;
    let request = Submission {
        revision: 0,
        schema_version: 1,
        generation,
        choices: vec![Choice::Keep],
    };
    assert_eq!(
        serde_json::to_value(&request).unwrap(),
        serde_json::json!({"schema_version":1,"revision":0,"generation":generation,"choices":[{"kind":"keep"}]})
    );
    let decoded: Submission =
        serde_json::from_value(serde_json::to_value(&request).unwrap()).unwrap();
    g.apply_policy(Seat::P0, &decoded, CAP).unwrap();
    assert_eq!(serde_json::to_vec(&o).unwrap(), saved);
    for error in [
        PolicyError::WrongActor,
        PolicyError::StaleDecision,
        PolicyError::InvalidSelection,
        PolicyError::CapacityExceeded,
        PolicyError::UnsupportedSpell,
        PolicyError::UnsupportedCombat,
    ] {
        let value = serde_json::to_string(&error).unwrap();
        assert!(!value.contains('{')); // context-free tag, no private diagnostic payload
    }
    // Libraries/opponent hands cannot be named in the reference type.
    assert!(serde_json::from_str::<VisibleRef>(r#"{"zone":"library","row":0}"#).is_err());
    g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
    let terminal = g.policy_observe(Seat::P0, 0).unwrap();
    assert!(terminal.decision.is_none());
    assert_eq!(terminal.view.terminal.unwrap().winner, Some(0));
    unchanged(&mut g, Seat::P0, request, CAP, PolicyError::Unavailable);
}
#[test]
fn policy_restore_invalidates_pre_restore_submissions_without_state_change() {
    let mut g = game(Seat::P0, false);
    keep(&mut g, Seat::P0);
    let d = g.policy_observe(Seat::P0, CAP).unwrap().decision.unwrap();
    let stale = Submission {
        revision: d.revision,
        schema_version: 1,
        generation: d.generation,
        choices: vec![Choice::Pass],
    };
    let saved = g.snapshot();
    g.apply_policy(Seat::P0, &stale, CAP).unwrap();
    g.restore(&saved).unwrap();
    unchanged(
        &mut g,
        Seat::P0,
        stale.clone(),
        CAP,
        PolicyError::StaleDecision,
    );
    let fresh = g.policy_observe(Seat::P0, CAP).unwrap().decision.unwrap();
    assert_eq!(fresh.generation, d.generation);
    assert_eq!(fresh.revision, d.revision + 1);
    submit(&mut g, Seat::P0, vec![Choice::Pass]).unwrap();
    g.restore(&saved).unwrap();
    assert_eq!(
        g.policy_observe(Seat::P0, CAP)
            .unwrap()
            .decision
            .unwrap()
            .revision,
        d.revision + 2
    );
    unchanged(
        &mut g,
        Seat::P0,
        Submission {
            revision: fresh.revision,
            ..stale
        },
        CAP,
        PolicyError::StaleDecision,
    );
    g.policy_revision = u64::MAX; // synthetic finite route-revision boundary
    let before = format!("{g:?}");
    assert_eq!(
        g.restore(&saved),
        Err(snapshot::RestoreError::IdentityExhausted)
    );
    assert_eq!(format!("{g:?}"), before);
}

// GH-111: CR 601.2 and RFC 0002 seat privacy require available acting-seat
// casting decisions, while the opponent continues to see committed state only.
#[test]
fn policy_spell_cast_is_offered_and_pending_cast_is_seat_safe() {
    let mut g = game(Seat::P0, false);
    keep(&mut g, Seat::P0);
    g.turns.position = Some((1, Seat::P0, turns::Step::PrecombatMain));
    let h = g
        .objects
        .allocate(
            CardId::from_key("swab-goblin").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    g.turns.mana[0] = [0, 0, 0, 2, 0, 0];
    let o = g.policy_observe(Seat::P0, CAP).unwrap();
    let choices = serde_json::to_value(o.decision.unwrap().candidates).unwrap();
    assert!(
        choices
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["kind"] == "cast"),
        "CR 601: a payable creature must be offered through the safe interface"
    );
    let public = g.policy_observe(Seat::P1, CAP).unwrap();
    g.begin_cast(Seat::P0, g.turn_decision().unwrap().id, h)
        .unwrap();
    assert!(g.policy_observe(Seat::P0, CAP).unwrap().decision.is_some());
    assert_eq!(g.policy_observe(Seat::P1, CAP).unwrap(), public);
}
#[test]
fn policy_spell_target_continuation_keeps_opponent_view_available() {
    let mut g = game(Seat::P0, false);
    keep(&mut g, Seat::P0);
    g.objects
        .allocate(
            CardId::from_key("bear-cub").unwrap(),
            Seat::P0,
            Zone::Battlefield,
        )
        .unwrap();
    let h = g
        .objects
        .allocate(
            CardId::from_key("giant-growth").unwrap(),
            Seat::P0,
            Zone::Hand(Seat::P0),
        )
        .unwrap();
    g.turns.mana[0][4] = 1;
    let public = g.policy_observe(Seat::P1, CAP).unwrap();
    g.begin_targeted_cast(Seat::P0, g.turn_decision().unwrap().id, h, CAP)
        .unwrap();
    assert_eq!(
        g.policy_observe(Seat::P1, CAP),
        Ok(public),
        "RFC privacy: pending target selection does not hide or alter committed public state"
    );
}

#[path = "policy_spell_tests.rs"]
mod spells;
