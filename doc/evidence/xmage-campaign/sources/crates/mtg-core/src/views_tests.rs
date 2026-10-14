use super::*;
use crate::objects::CardId;
// Original expectations: CR 400.2, 401.2/3, 402.3 and RFC B016/B033.
// Deliberately synthetic pairs permit Forest versus Mountain at the same draw
// boundary without inventing a mixed deck supported by normal reset.
fn pair(viewer: Seat) -> (Game, Game) {
    let other = if viewer == Seat::P0 {
        Seat::P1
    } else {
        Seat::P0
    };
    let build = |hidden: [&str; 2], seed| {
        let mut g = Game::new().unwrap();
        g.reset(&Config::default(), seed, 11).unwrap();
        g.objects.reset().unwrap();
        g.decision = None;
        g.kept = [true; 2];
        for key in ["mountain", "swab-goblin"] {
            g.objects
                .allocate(CardId::from_key(key).unwrap(), viewer, Zone::Hand(viewer))
                .unwrap();
        }
        for key in hidden {
            g.objects
                .allocate(CardId::from_key(key).unwrap(), other, Zone::Library(other))
                .unwrap();
        }
        g.objects
            .allocate(
                CardId::from_key(if seed == 7 {
                    "giant-growth"
                } else {
                    "bite-down"
                })
                .unwrap(),
                other,
                Zone::Hand(other),
            )
            .unwrap();
        g.objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                other,
                Zone::Battlefield,
            )
            .unwrap();
        g
    };
    (
        build(["forest", "mountain"], 7),
        build(["mountain", "forest"], 93),
    )
}
fn bytes(g: &Game, seat: Seat) -> Vec<u8> {
    serde_json::to_vec(&g.observe(seat).unwrap()).unwrap()
}
#[test]
fn views_library_privacy_positive() {
    for seat in [Seat::P0, Seat::P1] {
        let (g, _) = pair(seat);
        let v = g.observe(seat).unwrap();
        assert_eq!(v.life, [20, 20]);
        assert_eq!(
            v.hand.iter().map(|c| c.card).collect::<Vec<_>>(),
            ["mountain", "swab-goblin"]
        );
        assert_eq!(v.library_counts[1 - seat_index(seat)], 2);
        assert_eq!(v.hand_counts[seat_index(seat)], 2);
        assert_eq!(v.hand_counts[1 - seat_index(seat)], 1);
        assert_eq!(
            v.public_zones.iter().map(|z| z.zone).collect::<Vec<_>>(),
            [
                "graveyard_0",
                "graveyard_1",
                "battlefield",
                "stack",
                "exile"
            ]
        );
        assert_eq!(v.public_zones[2].cards[0].card, "bear-cub");
        assert_eq!(v.public_zones[2].cards[0].creature, Some([2, 2, 0]));
        let s = String::from_utf8(bytes(&g, seat)).unwrap();
        for forbidden in ["forest", "rng", "seed", "scope", "epoch", "slot", "Handle"] {
            assert!(!s.contains(forbidden), "{forbidden}: {s}");
        }
    }
}
#[test]
fn views_library_privacy_negative_independent_states() {
    for seat in [Seat::P0, Seat::P1] {
        let (a, b) = pair(seat);
        assert_ne!(format!("{:?}", a.objects), format!("{:?}", b.objects));
        assert_eq!(bytes(&a, seat), bytes(&b, seat));
        assert_eq!(a.observe(seat).unwrap().hand.len(), 2);
    }
}
#[test]
fn views_library_privacy_interaction_forest_versus_mountain_draw() {
    for seat in [Seat::P0, Seat::P1] {
        let (mut a, mut b) = pair(seat);
        let other = if seat == Seat::P0 { Seat::P1 } else { Seat::P0 };
        let x = a.draw_top(other).unwrap();
        let y = b.draw_top(other).unwrap();
        assert_eq!(a.objects.get(x).unwrap().card.identity().key, "forest");
        assert_eq!(b.objects.get(y).unwrap().card.identity().key, "mountain");
        assert_eq!(bytes(&a, seat), bytes(&b, seat));
        let v = a.observe(seat).unwrap();
        assert_eq!(v.hand_counts[seat_index(other)], 2);
        assert_eq!(v.library_counts[seat_index(other)], 1);
        assert_ne!(bytes(&a, other), bytes(&b, other));
    }
}
#[test]
fn views_revelations_are_historical_seat_scoped_and_reset() {
    let (mut g, _) = pair(Seat::P0);
    let h = g.objects.in_zone(Zone::Library(Seat::P1)).next().unwrap();
    let before = bytes(&g, Seat::P1);
    g.privileged_reveal_to(Seat::P0, h).unwrap();
    assert_eq!(bytes(&g, Seat::P1), before);
    let memory = g.observe(Seat::P0).unwrap().remembered;
    assert_eq!(
        memory.last().unwrap(),
        &Revelation {
            card: "forest",
            owner: 1,
            zone_at_reveal: "library_1"
        }
    );
    let h = g.objects.move_to(h, Zone::Hand(Seat::P1)).unwrap();
    assert_eq!(g.observe(Seat::P0).unwrap().remembered, memory);
    g.objects.remove(h).unwrap();
    assert_eq!(g.observe(Seat::P0).unwrap().remembered, memory);
    let public = g.objects.in_zone(Zone::Battlefield).next().unwrap();
    g.objects.move_to(public, Zone::Library(Seat::P1)).unwrap();
    assert!(
        g.observe(Seat::P0)
            .unwrap()
            .remembered
            .iter()
            .any(|r| r.card == "bear-cub" && r.zone_at_reveal == "battlefield")
    );
    g.reset(&Config::default(), 1, 2).unwrap();
    assert!(g.observe(Seat::P0).unwrap().remembered.is_empty());
}
#[test]
fn views_unavailable_boundaries_and_uniform_errors_do_not_mutate() {
    assert_eq!(
        Game::new().unwrap().observe(Seat::P0),
        Err(ViewError::Unavailable)
    );
    let (mut a, mut b) = pair(Seat::P0);
    for rows in [vec![], vec![0], vec![usize::MAX]] {
        let before_a = format!("{a:?}");
        let before_b = format!("{b:?}");
        assert_eq!(
            a.apply_opening_view(Seat::P0, 0, &rows),
            Err(ViewError::Unavailable)
        );
        assert_eq!(
            b.apply_opening_view(Seat::P0, 0, &rows),
            Err(ViewError::Unavailable)
        );
        assert_eq!(format!("{a:?}"), before_a);
        assert_eq!(format!("{b:?}"), before_b);
    }
    a.reset_quantum(&Config::default(), 1, 2, NonZeroUsize::new(1).unwrap())
        .unwrap();
    assert_eq!(a.observe(Seat::P0), Err(ViewError::Unavailable));
}
#[test]
fn views_public_zones_history_and_turn_boundary() {
    let (mut a, mut b) = pair(Seat::P0);
    for g in [&mut a, &mut b] {
        for (zone, key) in [
            (Zone::Graveyard(Seat::P0), "mountain"),
            (Zone::Graveyard(Seat::P1), "forest"),
            (Zone::Stack, "giant-growth"),
            (Zone::Exile, "bite-down"),
        ] {
            g.objects
                .allocate(CardId::from_key(key).unwrap(), Seat::P1, zone)
                .unwrap();
        }
        // Synthetic public settled state, with no pending casting/payment.
        g.turns.position = Some((3, Seat::P1, turns::Step::PrecombatMain));
        g.turns.decision = Some(turns::TurnDecision {
            id: DecisionId {
                scope: g.objects.scope(),
                generation: 50,
            },
            actor: Seat::P0,
            kind: turns::TurnKind::Priority,
        });
        g.life = [13, 17];
        g.turns.mana = [[0, 0, 0, 2, 0, 0], [0, 0, 0, 0, 1, 0]];
        let h = g.objects.in_zone(Zone::Battlefield).next().unwrap();
        g.objects.get_mut(h).unwrap().tapped = true;
        g.turns.modifications.push(targets::Modification {
            handle: h,
            power_boost: 0,
            boost: 3,
            damage: 2,
        });
        g.turns.sick.push(h);
    }
    assert_eq!(bytes(&a, Seat::P0), bytes(&b, Seat::P0));
    let v = a.observe(Seat::P0).unwrap();
    assert_eq!(v.turn, Some((3, 1, "precombat_main")));
    assert_eq!(v.acting_seat, Some(0));
    assert_eq!(v.life, [13, 17]);
    assert_eq!(v.mana, [[0, 0, 0, 2, 0, 0], [0, 0, 0, 0, 1, 0]]);
    assert_eq!(
        v.public_zones
            .iter()
            .map(|z| z.cards[0].card)
            .collect::<Vec<_>>(),
        [
            "mountain",
            "forest",
            "bear-cub",
            "giant-growth",
            "bite-down"
        ]
    );
    assert_eq!(v.public_zones[2].cards[0].creature, Some([5, 5, 2]));
    assert!(v.public_zones[2].cards[0].tapped);
    assert!(v.public_zones[2].cards[0].summoning_sick);
    assert_eq!(v.remembered.len(), 5);
}
#[test]
fn views_private_spell_continuations_are_explicitly_unavailable_in_core_schema() {
    let (mut g, _) = pair(Seat::P0);
    let d = g.start_turns().unwrap();
    g.turns.mana[0] = [0, 0, 0, 1, 0, 0];
    g.begin_payment(
        Seat::P0,
        d.id,
        mana::ManaCost {
            colored: [0; 6],
            generic: 1,
        },
    )
    .unwrap();
    for seat in [Seat::P0, Seat::P1] {
        assert_eq!(g.observe(seat), Err(ViewError::Unavailable));
    }
}
#[test]
fn views_error_serialization_is_literal_and_context_free() {
    for (error, literal) in [
        (ViewError::Unavailable, "\"unavailable\""),
        (ViewError::WrongActor, "\"wrong_actor\""),
        (ViewError::StaleDecision, "\"stale_decision\""),
        (ViewError::InvalidSelection, "\"invalid_selection\""),
    ] {
        assert_eq!(serde_json::to_string(&error).unwrap(), literal);
    }
}
#[test]
fn views_only_unseen_library_permutation_and_hidden_allocation_order_are_invariant() {
    for seat in [Seat::P0, Seat::P1] {
        let other = if seat == Seat::P0 { Seat::P1 } else { Seat::P0 };
        let build = |reverse: bool| {
            let mut g = Game::new().unwrap();
            g.reset(&Config::default(), 42, 11).unwrap();
            g.objects.reset().unwrap();
            g.decision = None;
            g.kept = [true; 2];
            let hidden = if reverse {
                ["forest", "mountain"]
            } else {
                ["mountain", "forest"]
            };
            if reverse {
                for key in hidden {
                    g.objects
                        .allocate(CardId::from_key(key).unwrap(), other, Zone::Library(other))
                        .unwrap();
                }
            }
            for key in if reverse {
                ["swab-goblin", "mountain"]
            } else {
                ["mountain", "swab-goblin"]
            } {
                g.objects
                    .allocate(CardId::from_key(key).unwrap(), seat, Zone::Hand(seat))
                    .unwrap();
            }
            g.objects
                .allocate(
                    CardId::from_key("bear-cub").unwrap(),
                    other,
                    Zone::Battlefield,
                )
                .unwrap();
            if !reverse {
                for key in hidden {
                    g.objects
                        .allocate(CardId::from_key(key).unwrap(), other, Zone::Library(other))
                        .unwrap();
                }
            }
            g
        };
        let a = build(false);
        let b = build(true);
        assert_eq!(bytes(&a, seat), bytes(&b, seat));
        let own_a = a
            .objects
            .in_zone(Zone::Hand(seat))
            .map(|h| a.objects.get(h).unwrap().card)
            .collect::<Vec<_>>();
        let own_b = b
            .objects
            .in_zone(Zone::Hand(seat))
            .map(|h| b.objects.get(h).unwrap().card)
            .collect::<Vec<_>>();
        assert_ne!(own_a, own_b); // hand insertion order is not a policy feature
        assert_eq!(a.observe(seat).unwrap().hand.len(), 2);
    }
}
#[test]
fn views_remembered_facts_do_not_track_hidden_handles_or_order() {
    for seat in [Seat::P0, Seat::P1] {
        let (mut a, mut b) = pair(seat);
        let other = if seat == Seat::P0 { Seat::P1 } else { Seat::P0 };
        for g in [&mut a, &mut b] {
            let h = g
                .objects
                .in_zone(Zone::Library(other))
                .find(|h| g.objects.get(*h).unwrap().card.identity().key == "forest")
                .unwrap();
            g.privileged_reveal_to(seat, h).unwrap();
            let before = format!("{g:?}");
            // A handle from a separate store cannot mutate remembered facts.
            let foreign = Game::new()
                .unwrap()
                .objects
                .allocate(
                    CardId::from_key("forest").unwrap(),
                    other,
                    Zone::Hand(other),
                )
                .unwrap();
            assert_eq!(
                g.privileged_reveal_to(seat, foreign),
                Err(StorageError::InvalidHandle)
            );
            assert_eq!(format!("{g:?}"), before);
        }
        assert_eq!(bytes(&a, seat), bytes(&b, seat));
        let facts = &a.observe(seat).unwrap().remembered;
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[1].card, "forest");
        assert_eq!(
            facts[1].zone_at_reveal,
            if other == Seat::P0 {
                "library_0"
            } else {
                "library_1"
            }
        );
    }
}
#[test]
fn views_literal_empty_schema_and_owned_snapshot() {
    let (mut g, _) = pair(Seat::P0);
    g.objects.reset().unwrap();
    let v = g.observe(Seat::P0).unwrap();
    assert_eq!(
        serde_json::to_value(&v).unwrap(),
        serde_json::json!({
            "schema_version":1,"seat":0,"life":[20,20],"hand_counts":[0,0],"library_counts":[0,0],
            "hand":[],"public_zones":[{"zone":"graveyard_0","cards":[]},{"zone":"graveyard_1","cards":[]},{"zone":"battlefield","cards":[]},{"zone":"stack","cards":[]},{"zone":"exile","cards":[]}],
            "remembered":[],"starting_seat":0,"turn":null,"mana":[[0,0,0,0,0,0],[0,0,0,0,0,0]],"acting_seat":null,"opening":null,"terminal":null
        })
    );
    let saved = serde_json::to_vec(&v).unwrap();
    g.reset(&Config::default(), 9, 10).unwrap();
    assert_eq!(serde_json::to_vec(&v).unwrap(), saved);
    assert_ne!(bytes(&g, Seat::P0), saved);
}
