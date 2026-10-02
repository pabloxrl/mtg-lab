//! Expectations specified before implementation: legal rows are uniformly sampled
//! in table order using unbiased modulo rejection. Existing independently derived
//! RNG vectors in mtg-core/tests/rng.rs fix the policy-seat streams.
use mtg_core::{
    game::{
        Config, Game,
        policy::{Choice, CombatChoices, DamageAllocation, Observation, VisibleRef, VisibleZone},
    },
    objects::Seat,
};
use mtg_policy::{Error, LegalRandom, RNG_VERSION, VERSION};
fn policy(seat: u8) -> LegalRandom {
    LegalRandom::new(VERSION, RNG_VERSION, 0, 0, seat).unwrap()
}
fn observation(seat: u8) -> Observation {
    let mut g = Game::new().unwrap();
    g.reset(
        &Config {
            starting_seat: seat,
            ..Config::default()
        },
        0,
        0,
    )
    .unwrap();
    g.policy_observe(if seat == 0 { Seat::P0 } else { Seat::P1 }, 256)
        .unwrap()
}
fn bf(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Battlefield,
        row,
    }
}
#[test]
fn literal_masked_selection_vectors_both_seats() {
    // Raw words modulo 3: seat0 [2,1,1,2], seat1 [0,0,0,1].
    // Threshold is 2^64 mod 3 = 1; none of these published words is rejected.
    for (seat, expected) in [(0, [2, 1, 1, 2]), (1, [0, 0, 0, 1])] {
        let mut o = observation(seat);
        o.view.hand[0].card = "forest";
        o.view.hand[1].card = "bear-cub";
        let d = o.decision.as_mut().unwrap();
        d.kind = "priority";
        d.candidates = vec![
            Choice::Pass,
            Choice::Spell,
            Choice::PlayLand {
                card: VisibleRef {
                    zone: VisibleZone::Hand,
                    row: 0,
                },
            },
            Choice::Cast {
                card: VisibleRef {
                    zone: VisibleZone::Hand,
                    row: 1,
                },
            },
        ];
        d.legal_mask = vec![true, false, true, true];
        let legal = [
            Choice::Pass,
            Choice::PlayLand {
                card: VisibleRef {
                    zone: VisibleZone::Hand,
                    row: 0,
                },
            },
            Choice::Cast {
                card: VisibleRef {
                    zone: VisibleZone::Hand,
                    row: 1,
                },
            },
        ];
        let mut p = policy(seat);
        for index in expected {
            let a = p.choose(&o).unwrap();
            assert_eq!(a.choices, vec![legal[index].clone()]);
            assert_eq!(a.generation, o.decision.as_ref().unwrap().generation);
            assert_eq!(a.revision, o.decision.as_ref().unwrap().revision);
        }
    }
}
#[test]
fn ordered_without_replacement_vector() {
    let mut o = observation(0);
    let d = o.decision.as_mut().unwrap();
    d.kind = "bottom";
    d.count = 2;
    d.candidates = (0..3)
        .map(|row| Choice::Bottom {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row,
            },
        })
        .collect();
    d.legal_mask = vec![true; 3];
    // First word %3=2, second word %2=0: remove row2, then row0.
    let expected = vec![
        Choice::Bottom {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: 2,
            },
        },
        Choice::Bottom {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: 0,
            },
        },
    ];
    assert_eq!(policy(0).choose(&o).unwrap().choices, expected);
    // Cleanup uses the same sampled rows, but different semantic commands.
    let d = o.decision.as_mut().unwrap();
    d.kind = "cleanup_discard";
    d.candidates = (0..3)
        .map(|row| Choice::Discard {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row,
            },
        })
        .collect();
    assert_eq!(
        policy(0).choose(&o).unwrap().choices,
        vec![
            Choice::Discard {
                card: VisibleRef {
                    zone: VisibleZone::Hand,
                    row: 2
                }
            },
            Choice::Discard {
                card: VisibleRef {
                    zone: VisibleZone::Hand,
                    row: 0
                }
            },
        ]
    );
}
#[test]
fn explicit_errors_no_unsupported_fallback() {
    assert!(matches!(
        LegalRandom::new("future", RNG_VERSION, 0, 0, 0),
        Err(Error::UnsupportedVersion)
    ));
    assert!(matches!(
        LegalRandom::new(VERSION, "future", 0, 0, 0),
        Err(Error::UnsupportedVersion)
    ));
    assert!(matches!(
        LegalRandom::new(VERSION, RNG_VERSION, 0, 0, 2),
        Err(Error::WrongSeat)
    ));
    let mut o = observation(0);
    assert_eq!(policy(1).choose(&o), Err(Error::WrongSeat));
    o.schema_version = 99;
    assert_eq!(policy(0).choose(&o), Err(Error::UnsupportedVersion));
    o.schema_version = 1;
    o.decision.as_mut().unwrap().kind = "future";
    assert_eq!(policy(0).choose(&o), Err(Error::UnsupportedDecision));
    o.decision.as_mut().unwrap().kind = "priority";
    o.decision.as_mut().unwrap().candidates = vec![Choice::Spell];
    o.decision.as_mut().unwrap().legal_mask = vec![true];
    assert_eq!(policy(0).choose(&o), Err(Error::UnsupportedDecision));
    o.decision = None;
    assert_eq!(policy(0).choose(&o), Err(Error::Unavailable));
}
#[test]
fn factored_vectors_have_declared_distribution() {
    // V1: fair coin: 0 replaces declarations, 1 finishes. A replacement
    // independently includes each attacker with probability 1/2. Seat0 first
    // three words are even, so replacement is the empty subset.
    let mut o = observation(0);
    let d = o.decision.as_mut().unwrap();
    d.kind = "attackers";
    d.candidates = vec![Choice::FinishCombat];
    d.legal_mask = vec![true];
    d.factored = Some(CombatChoices {
        attackers: vec![bf(0), bf(1)],
        blockers: vec![],
        selected: vec![],
        blocks: vec![],
        damage: vec![],
    });
    assert_eq!(
        policy(0).choose(&o).unwrap().choices,
        vec![Choice::SelectAttackers { cards: vec![] }]
    );
    // Seat1's first word is odd: finish the current (empty) selection.
    let mut o1 = o.clone();
    o1.view.seat = 1;
    o1.decision.as_mut().unwrap().actor = 1;
    assert_eq!(
        policy(1).choose(&o1).unwrap().choices,
        vec![Choice::FinishCombat]
    );
    let d = o.decision.as_mut().unwrap();
    d.kind = "blockers";
    d.factored.as_mut().unwrap().blockers = vec![bf(2), bf(3)];
    // After replacement coin, words modulo 3 are 1,1: both blockers -> attacker0.
    // Each blocker also has the unassigned option. Each map has probability 1/9 conditional on replacement.
    assert_eq!(
        policy(0).choose(&o).unwrap().choices,
        vec![Choice::SelectBlockers {
            blocks: vec![(bf(2), bf(0)), (bf(3), bf(0))]
        }]
    );
    let d = o.decision.as_mut().unwrap();
    d.kind = "combat_damage";
    d.legal_mask = vec![false];
    d.factored.as_mut().unwrap().damage = vec![DamageAllocation {
        attacker: bf(0),
        power: 2,
        blockers: vec![bf(2), bf(3)],
        amounts: None,
    }];
    // Sequential uniform split: first word %3=2, last recipient gets remainder.
    assert_eq!(
        policy(0).choose(&o).unwrap().choices,
        vec![Choice::AssignDamage {
            attacker: bf(0),
            amounts: vec![(bf(2), 2), (bf(3), 0)]
        }]
    );
}

#[test]
fn episode_vectors_interleaving_and_invalid_inputs_do_not_advance_stream() {
    // Published episode1 words in mtg-core/tests/rng.rs have these literal low bits.
    for (seat, bits) in [(0, [1, 1, 0, 1]), (1, [1, 1, 0, 0])] {
        let o = observation(seat);
        let mut p = LegalRandom::new(VERSION, RNG_VERSION, 0, 1, seat).unwrap();
        let mut peer = LegalRandom::new(VERSION, RNG_VERSION, 0, 1, seat).unwrap();
        let mut unrelated = policy(1 - seat);
        for bit in bits {
            // Malformed masks and an empty legal set are errors, never pass.
            let mut bad = o.clone();
            bad.decision.as_mut().unwrap().legal_mask.clear();
            assert_eq!(p.choose(&bad), Err(Error::InvalidObservation));
            bad.decision.as_mut().unwrap().legal_mask = vec![false; 2];
            assert_eq!(p.choose(&bad), Err(Error::InvalidObservation));
            unrelated.choose(&observation(1 - seat)).unwrap();
            let a = p.choose(&o).unwrap();
            assert_eq!(a, peer.choose(&o).unwrap());
            assert_eq!(
                a.choices,
                vec![if bit == 0 {
                    Choice::Keep
                } else {
                    Choice::Mulligan
                }]
            );
        }
    }
}

#[test]
fn three_recipient_distribution_is_sequential_not_uniform_compositions() {
    let mut o = observation(0);
    let d = o.decision.as_mut().unwrap();
    d.kind = "combat_damage";
    d.candidates = vec![Choice::FinishCombat];
    d.legal_mask = vec![false];
    d.factored = Some(CombatChoices {
        attackers: vec![],
        blockers: vec![],
        selected: vec![],
        blocks: vec![],
        damage: vec![DamageAllocation {
            attacker: bf(0),
            power: 2,
            blockers: vec![bf(1), bf(2), bf(3)],
            amounts: None,
        }],
    });
    // First draw 2 of 0..=2 leaves zero; the second still consumes a bound1 draw.
    // P(2,0,0)=1/3, P(0,0,2)=1/9 by the declared conditional split distribution.
    assert_eq!(
        policy(0).choose(&o).unwrap().choices,
        vec![Choice::AssignDamage {
            attacker: bf(0),
            amounts: vec![(bf(1), 2), (bf(2), 0), (bf(3), 0)]
        }]
    );
    let d = o.decision.as_mut().unwrap();
    d.factored.as_mut().unwrap().damage[0].amounts = Some(vec![(bf(1), 2)]);
    d.legal_mask = vec![true];
    assert_eq!(
        policy(0).choose(&o).unwrap().choices,
        vec![Choice::FinishCombat]
    );
}

#[test]
fn newly_enabled_unsupported_card_is_an_error_not_a_fallback() {
    let mut o = observation(0);
    o.view.hand[0].card = "shivan-dragon";
    let d = o.decision.as_mut().unwrap();
    d.kind = "priority";
    d.candidates = vec![
        Choice::Pass,
        Choice::Cast {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: 0,
            },
        },
    ];
    d.legal_mask = vec![true, true];
    // Error variant checked by debug text so the preimplementation test compiles.
    assert_eq!(
        format!("{:?}", policy(0).choose(&o)),
        "Err(UnsupportedContent)"
    );
    // Masked unsupported content is present in the frozen decks and remains uncastable.
    o.decision.as_mut().unwrap().legal_mask[1] = false;
    assert_eq!(policy(0).choose(&o).unwrap().choices, vec![Choice::Pass]);
}

#[test]
fn creature_mana_random_accepts_enabled_casts() {
    for key in ["llanowar-elves", "druid-of-the-cowl"] {
        let mut o = observation(0);
        o.view.hand[0].card = key;
        let d = o.decision.as_mut().unwrap();
        d.kind = "priority";
        d.candidates = vec![Choice::Cast {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: 0,
            },
        }];
        d.legal_mask = vec![true];
        let expected = d.candidates.clone();
        assert_eq!(policy(0).choose(&o).unwrap().choices, expected);
    }
}
