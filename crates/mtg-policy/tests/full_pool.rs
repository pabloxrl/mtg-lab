//! Policy-contract fixtures, not synthetic game/rules evidence.
//! RFC0002 §§3/9 requires the frozen pool and every mandatory continuation.
use mtg_core::{
    game::{
        Config, Game,
        policy::{Choice, Observation, VisibleRef, VisibleZone},
    },
    objects::Seat,
};
use mtg_policy::{Error, HEURISTIC_VERSION, Heuristic, LegalRandom, RNG_VERSION, VERSION};

fn observation() -> Observation {
    let mut game = Game::new().unwrap();
    game.reset(&Config::default(), 0, 0).unwrap();
    game.policy_observe(Seat::P0, 1024).unwrap()
}
fn choices(o: &Observation) -> [Result<Vec<Choice>, Error>; 2] {
    [
        LegalRandom::new(VERSION, RNG_VERSION, 0, 0, 0)
            .unwrap()
            .choose(o)
            .map(|s| s.choices),
        Heuristic::new(HEURISTIC_VERSION, 0)
            .unwrap()
            .choose(o)
            .map(|s| s.choices),
    ]
}
#[test]
fn full_pool_trigger_creatures_are_supported_without_pass_fallback() {
    for name in [
        "firebrand-archer",
        "crackling-cyclops",
        "viashino-pyromancer",
    ] {
        let mut o = observation();
        o.view.hand[0].card = name;
        let d = o.decision.as_mut().unwrap();
        d.kind = "priority";
        let cast = Choice::Cast {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: 0,
            },
        };
        d.candidates = vec![cast.clone()];
        d.legal_mask = vec![true];
        for selected in choices(&o) {
            assert_eq!(selected, Ok(vec![cast.clone()]), "{name}");
        }
        o.view.hand[0].card = "unknown-content";
        o.decision.as_mut().unwrap().candidates.push(Choice::Pass);
        o.decision.as_mut().unwrap().legal_mask.push(true);
        for selected in choices(&o) {
            assert_eq!(selected, Err(Error::UnsupportedContent));
        }
    }
}
#[test]
fn full_pool_trigger_permutation_uses_existing_rng_and_stable_heuristic_order() {
    let mut o = observation();
    let d = o.decision.as_mut().unwrap();
    d.kind = "trigger_order";
    d.count = 3;
    d.candidates = (0..3)
        .map(|trigger| Choice::OrderTrigger { trigger })
        .collect();
    d.legal_mask = vec![true; 3];
    // Published policy-seat0 words modulo remaining lengths 3,2,1 give 2,0,0.
    let [random, heuristic] = choices(&o);
    assert_eq!(
        random,
        Ok([2, 0, 1]
            .map(|trigger| Choice::OrderTrigger { trigger })
            .to_vec())
    );
    assert_eq!(
        heuristic,
        Ok([0, 1, 2]
            .map(|trigger| Choice::OrderTrigger { trigger })
            .to_vec())
    );
}
#[test]
fn full_pool_player_targets_and_missing_choices_are_explicit() {
    let mut o = observation();
    let d = o.decision.as_mut().unwrap();
    d.kind = "trigger_target";
    d.candidates = vec![
        Choice::TargetPlayer { seat: 0 },
        Choice::TargetPlayer { seat: 1 },
    ];
    d.legal_mask = vec![true; 2];
    let [random, heuristic] = choices(&o);
    assert!(random.is_ok());
    assert_eq!(heuristic, Ok(vec![Choice::TargetPlayer { seat: 1 }]));
    o.decision.as_mut().unwrap().legal_mask.fill(false);
    for selected in choices(&o) {
        assert_eq!(selected, Err(Error::InvalidObservation));
    }
    o.decision = None;
    for selected in choices(&o) {
        assert_eq!(selected, Err(Error::Unavailable));
    }
}

// Ordered frozen decks still use normal reset. Both twins have the same own
// hand/public game but different opponent hand identities and library order.
fn ordered(prefix: &[&str]) -> Vec<String> {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let mut result = prefix.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let deck = manifest["decks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == "red")
        .unwrap();
    for c in deck["cards"].as_array().unwrap() {
        let name = c["card_id"].as_str().unwrap();
        for _ in result.iter().filter(|s| s.as_str() == name).count()
            ..c["copies"].as_u64().unwrap() as usize
        {
            result.push(name.into());
        }
    }
    result
}
fn actor(g: &Game) -> (Seat, Observation) {
    for seat in [Seat::P0, Seat::P1] {
        let o = g.policy_observe(seat, 1024).unwrap();
        if o.decision.is_some() {
            return (seat, o);
        }
    }
    panic!("no decision")
}
fn submit(g: &mut Game, a: Seat, o: &Observation, selected: Vec<Choice>) {
    let d = o.decision.as_ref().unwrap();
    g.apply_policy(
        a,
        &mtg_core::game::policy::Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: selected,
        },
        1024,
    )
    .unwrap();
    if g.decision().is_none() && g.turn_position().is_none() {
        g.start_turns().unwrap();
    }
}
#[test]
fn full_pool_scripted_trigger_checkpoints_and_hidden_twins() {
    use mtg_core::game::DeckConfig;
    for random in [false, true] {
        let own = ordered(&[
            "firebrand-archer",
            "crackling-cyclops",
            "viashino-pyromancer",
            "dragon-fodder",
            "mountain",
            "mountain",
            "mountain",
            "mountain",
            "mountain",
        ]);
        let other = ordered(&[
            "swab-goblin",
            "swab-goblin",
            "mountain",
            "mountain",
            "mountain",
            "mountain",
            "mountain",
            "mountain",
            "mountain",
            "mountain",
            "mountain",
        ]);
        let mut twin = other.clone();
        let last = twin.iter().rposition(|c| c == "shivan-dragon").unwrap();
        twin.swap(0, last);
        twin[15..].reverse();
        let mut games = [other, twin].map(|order| {
            let mut g = Game::new().unwrap();
            g.reset(
                &Config {
                    seats: vec![
                        DeckConfig {
                            deck: "red".into(),
                            order: Some(own.clone()),
                        },
                        DeckConfig {
                            deck: "red".into(),
                            order: Some(order),
                        },
                    ],
                    ..Config::default()
                },
                208,
                0,
            )
            .unwrap();
            g
        });
        let mut casts = std::collections::BTreeSet::new();
        let mut verified = false;
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..500 {
            let (a, o) = actor(&games[0]);
            let (b, p) = actor(&games[1]);
            assert_eq!(a, b);
            let d = o.decision.as_ref().unwrap();
            if a == Seat::P0 {
                assert_eq!(o, p, "hidden swap at {}", d.kind);
                for seed in 0..8 {
                    let mut l = LegalRandom::new(VERSION, RNG_VERSION, seed, 0, 0).unwrap();
                    let mut r = LegalRandom::new(VERSION, RNG_VERSION, seed, 0, 0).unwrap();
                    assert_eq!(l.choose(&o).unwrap(), r.choose(&p).unwrap());
                }
                assert_eq!(
                    Heuristic::new(HEURISTIC_VERSION, 0)
                        .unwrap()
                        .choose(&o)
                        .unwrap(),
                    Heuristic::new(HEURISTIC_VERSION, 0)
                        .unwrap()
                        .choose(&p)
                        .unwrap()
                );
                seen.insert(d.kind);
            }
            if casts.contains(&9)
                && d.kind == "priority"
                && o.stack.is_empty()
                && o.pending_triggers.is_empty()
            {
                // CR603 and pinned Oracle: Pyromancer two, Archer one;
                // Cyclops gets +3/+0, Fodder creates exactly two 1/1 Goblins.
                assert_eq!(o.view.life, [20, 17]);
                let bf = &o
                    .view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .unwrap()
                    .cards;
                assert!(
                    bf.iter()
                        .any(|c| c.card == "crackling-cyclops" && c.creature == Some([3, 4, 0]))
                );
                assert_eq!(
                    bf.iter()
                        .filter(|c| c.card == "goblin-token" && c.creature == Some([1, 1, 0]))
                        .count(),
                    2
                );
                verified = true;
                break;
            }
            let legal = d
                .candidates
                .iter()
                .zip(&d.legal_mask)
                .filter_map(|(c, m)| m.then_some(c))
                .collect::<Vec<_>>();
            let chosen = match d.kind {
                "keep_or_mulligan" => vec![Choice::Keep],
                "trigger_order" => {
                    assert_eq!(
                        o.view.life,
                        if casts.contains(&9) {
                            [20, 18]
                        } else {
                            [20, 20]
                        }
                    );
                    if random {
                        LegalRandom::new(VERSION, RNG_VERSION, 0, 0, 0)
                            .unwrap()
                            .choose(&o)
                            .unwrap()
                            .choices
                    } else {
                        Heuristic::new(HEURISTIC_VERSION, 0)
                            .unwrap()
                            .choose(&o)
                            .unwrap()
                            .choices
                    }
                }
                "trigger_target" => {
                    assert_eq!(o.view.life, [20, 20]);
                    // Exercise the random player's legal opponent target using
                    // a bounded seed search; the full games above are unforced.
                    if random {
                        (0..32)
                            .find_map(|seed| {
                                let s = LegalRandom::new(VERSION, RNG_VERSION, seed, 0, 0)
                                    .unwrap()
                                    .choose(&o)
                                    .unwrap();
                                (s.choices == [Choice::TargetPlayer { seat: 1 }])
                                    .then_some(s.choices)
                            })
                            .unwrap()
                    } else {
                        Heuristic::new(HEURISTIC_VERSION, 0)
                            .unwrap()
                            .choose(&o)
                            .unwrap()
                            .choices
                    }
                }
                "priority" => {
                    let turn = o.view.turn.unwrap();
                    let land = legal.iter().find(|c| matches!(c, Choice::PlayLand { .. }));
                    if let Some(c) = land {
                        vec![(*c).clone()]
                    } else if a == Seat::P0
                        && turn.2 == "precombat_main"
                        && !casts.contains(&turn.0)
                    {
                        let name = match turn.0 {
                            3 => "firebrand-archer",
                            5 => "crackling-cyclops",
                            7 => "viashino-pyromancer",
                            9 => "dragon-fodder",
                            _ => "none",
                        };
                        if let Some(c) = legal.iter().find(
                            |c| matches!(c,Choice::Cast {card} if o.view.hand[card.row].card==name),
                        ) {
                            casts.insert(turn.0);
                            vec![(*c).clone()]
                        } else {
                            vec![Choice::Pass]
                        }
                    } else {
                        vec![Choice::Pass]
                    }
                }
                "payment" => vec![
                    legal
                        .iter()
                        .find(|c| matches!(c, Choice::FinishPayment))
                        .or_else(|| legal.iter().find(|c| matches!(c, Choice::Pay { .. })))
                        .or_else(|| legal.iter().find(|c| matches!(c, Choice::TapMana { .. })))
                        .unwrap()
                        .to_owned()
                        .clone(),
                ],
                "attackers" | "blockers" | "combat_damage" => vec![Choice::FinishCombat],
                "cleanup_discard" => legal.iter().take(d.count).map(|c| (*c).clone()).collect(),
                _ => panic!("unexpected {}", d.kind),
            };
            submit(&mut games[0], a, &o, chosen.clone());
            submit(&mut games[1], b, &p, chosen);
        }
        assert!(verified);
        assert!(seen.contains("trigger_order") && seen.contains("trigger_target"));
        assert_ne!(
            games[0].policy_observe(Seat::P1, 1024).unwrap().view.hand,
            games[1].policy_observe(Seat::P1, 1024).unwrap().view.hand
        );
    }
}

#[test]
fn full_pool_trigger_order_rejects_incomplete_or_duplicate_permutations() {
    let mut o = observation();
    let d = o.decision.as_mut().unwrap();
    d.kind = "trigger_order";
    d.count = 1;
    d.candidates = vec![
        Choice::OrderTrigger { trigger: 0 },
        Choice::OrderTrigger { trigger: 1 },
    ];
    d.legal_mask = vec![true; 2];
    for selected in choices(&o) {
        assert_eq!(selected, Err(Error::InvalidObservation));
    }
    let d = o.decision.as_mut().unwrap();
    d.count = 2;
    d.candidates[1] = Choice::OrderTrigger { trigger: 0 };
    for selected in choices(&o) {
        assert_eq!(selected, Err(Error::InvalidObservation));
    }
}

#[test]
fn full_pool_policy_versions_reject_previous_support_contract() {
    assert!(matches!(
        LegalRandom::new("legal-random-surprise-v1", RNG_VERSION, 0, 0, 0),
        Err(Error::UnsupportedVersion)
    ));
    assert!(matches!(
        Heuristic::new("heuristic-surprise-v1", 0),
        Err(Error::UnsupportedVersion)
    ));
}
