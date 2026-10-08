//! Literal choices derive from doc/heuristic-policy.md, specified before code.
//! Synthetic observations test strategy, not rules reachability or playing strength.
use mtg_core::{
    game::{
        Config, Game,
        mana::ManaCost,
        policy::{
            Choice, CombatChoices, DamageAllocation, Observation, PendingSpell, StackSpell,
            VisibleRef, VisibleZone,
        },
        views::VisibleCard,
    },
    objects::Seat,
};
use mtg_policy::{Error, HEURISTIC_VERSION, Heuristic};
fn h(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Hand,
        row,
    }
}
fn b(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Battlefield,
        row,
    }
}
fn card(name: &'static str, controller: u8, creature: Option<[u32; 3]>) -> VisibleCard {
    VisibleCard {
        haste: false,
        trample: name == "tajuru-pathwarden",
        card: name,
        owner: controller,
        controller,
        tapped: false,
        creature,
        summoning_sick: false,
    }
}
fn obs(kind: &'static str, choices: Vec<Choice>) -> Observation {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 0, 0).unwrap();
    let mut o = g.policy_observe(Seat::P0, 256).unwrap();
    o.view.hand = vec![
        card("forest", 0, None),
        card("bear-cub", 0, None),
        card("giant-growth", 0, None),
        card("bite-down", 0, None),
        card("llanowar-elves", 0, None),
        card("mountain", 0, None),
    ];
    o.view
        .public_zones
        .iter_mut()
        .find(|z| z.zone == "battlefield")
        .unwrap()
        .cards = vec![
        card("bear-cub", 0, Some([2, 2, 0])),
        card("bear-cub", 1, Some([5, 5, 0])),
        card("bear-cub", 1, Some([2, 2, 1])),
        card("bear-cub", 0, Some([5, 5, 0])),
        card("mountain", 0, None),
        card("forest", 0, None),
    ];
    let d = o.decision.as_mut().unwrap();
    d.kind = kind;
    d.candidates = choices;
    d.legal_mask = vec![true; d.candidates.len()];
    d.count = 1;
    d.factored = None;
    o
}
fn choose(o: &Observation) -> Vec<Choice> {
    Heuristic::new(HEURISTIC_VERSION, 0)
        .unwrap()
        .choose(o)
        .unwrap()
        .choices
}
fn expect(kind: &'static str, choices: Vec<Choice>, expected: Choice) {
    assert_eq!(choose(&obs(kind, choices)), vec![expected]);
}
#[test]
fn heuristic_land_creature_response_and_mask_ties() {
    let choices = vec![
        Choice::Pass,
        Choice::Cast { card: h(1) },
        Choice::PlayLand { card: h(0) },
        Choice::PlayLand { card: h(5) },
    ];
    let mut o = obs("priority", choices);
    assert_eq!(choose(&o), vec![Choice::PlayLand { card: h(0) }]);
    o.decision.as_mut().unwrap().legal_mask = vec![true, true, false, false];
    assert_eq!(choose(&o), vec![Choice::Cast { card: h(1) }]);
    let mut o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(2) }]);
    assert_eq!(choose(&o), vec![Choice::Pass]);
    o.stack.push(StackSpell {
        trigger: None,
        mode: None,
        ability: false,
        row: 0,
        targets: vec![Some(b(0))],
    });
    assert_eq!(choose(&o), vec![Choice::Cast { card: h(2) }]);
    expect(
        "priority",
        vec![Choice::Pass, Choice::Cast { card: h(3) }],
        Choice::Cast { card: h(3) },
    );
}
#[test]
fn heuristic_opening_discard_and_bottom_values() {
    expect(
        "keep_or_mulligan",
        vec![Choice::Mulligan, Choice::Keep],
        Choice::Keep,
    );
    for kind in ["bottom", "cleanup_discard"] {
        let c = |r| {
            if kind == "bottom" {
                Choice::Bottom { card: h(r) }
            } else {
                Choice::Discard { card: h(r) }
            }
        };
        let mut o = obs(kind, (0..6).map(c).collect());
        o.decision.as_mut().unwrap().count = 3;
        assert_eq!(choose(&o), vec![c(4), c(0), c(5)]);
    }
}
#[test]
fn heuristic_target_scores_and_cancellation() {
    for kind in ["growth_target", "bite_source"] {
        expect(
            kind,
            vec![
                Choice::Target { card: b(1) },
                Choice::Target { card: b(0) },
                Choice::CancelTargets,
                Choice::Target { card: b(3) },
            ],
            Choice::Target { card: b(3) },
        );
        expect(
            kind,
            vec![Choice::Target { card: b(1) }, Choice::CancelTargets],
            Choice::CancelTargets,
        );
    }
    let mut o = obs(
        "bite_destination",
        vec![
            Choice::Target { card: b(1) },
            Choice::Target { card: b(2) },
            Choice::CancelTargets,
        ],
    );
    o.pending = Some(PendingSpell {
        mode: None,
        card: h(3),
        targets: vec![Some(b(0))],
        sources: vec![],
        pool: None,
        remaining: None,
    });
    assert_eq!(choose(&o), vec![Choice::Target { card: b(2) }]);
    expect(
        "targets_complete",
        vec![Choice::CancelTargets, Choice::FinishTargets],
        Choice::FinishTargets,
    );
}
#[test]
fn heuristic_payment_prefers_progress_and_needed_color() {
    expect(
        "payment",
        vec![
            Choice::CancelPayment,
            Choice::Pay { color: 4 },
            Choice::FinishPayment,
        ],
        Choice::FinishPayment,
    );
    expect(
        "payment",
        vec![
            Choice::CancelPayment,
            Choice::TapMana { card: b(5) },
            Choice::Pay { color: 4 },
        ],
        Choice::Pay { color: 4 },
    );
    let mut o = obs(
        "payment",
        vec![
            Choice::CancelPayment,
            Choice::TapMana { card: b(4) },
            Choice::TapMana { card: b(5) },
        ],
    );
    o.pending = Some(PendingSpell {
        mode: None,
        card: h(1),
        targets: vec![],
        sources: vec![],
        pool: Some([0; 6]),
        remaining: Some(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
        }),
    });
    assert_eq!(choose(&o), vec![Choice::TapMana { card: b(5) }]);
    expect(
        "payment",
        vec![Choice::CancelPayment],
        Choice::CancelPayment,
    );
}
fn combat(kind: &'static str) -> Observation {
    let mut o = obs(kind, vec![Choice::FinishCombat]);
    o.decision.as_mut().unwrap().factored = Some(CombatChoices {
        forbidden_blocks: vec![],
        attackers: vec![b(0), b(3)],
        blockers: vec![b(1), b(2)],
        selected: vec![],
        blocks: vec![],
        damage: vec![],
    });
    o
}
#[test]
fn heuristic_attack_block_allocate_then_finish() {
    let mut o = combat("attackers");
    assert_eq!(
        choose(&o),
        vec![Choice::SelectAttackers {
            cards: vec![b(0), b(3)]
        }]
    );
    o.decision
        .as_mut()
        .unwrap()
        .factored
        .as_mut()
        .unwrap()
        .selected = vec![b(0), b(3)];
    assert_eq!(choose(&o), vec![Choice::FinishCombat]);
    let mut o = combat("blockers");
    let blocks = vec![(b(1), b(3)), (b(2), b(0))];
    assert_eq!(
        choose(&o),
        vec![Choice::SelectBlockers {
            blocks: blocks.clone()
        }]
    );
    o.decision
        .as_mut()
        .unwrap()
        .factored
        .as_mut()
        .unwrap()
        .blocks = blocks;
    assert_eq!(choose(&o), vec![Choice::FinishCombat]);
    let mut o = combat("combat_damage");
    let d = o.decision.as_mut().unwrap();
    d.legal_mask = vec![false];
    d.factored.as_mut().unwrap().damage = vec![DamageAllocation {
        trample_lethal: None,
        attacker: b(3),
        power: 5,
        blockers: vec![b(2), b(1)],
        amounts: None,
    }];
    let amounts = vec![(b(2), 1), (b(1), 4)];
    assert_eq!(
        choose(&o),
        vec![Choice::AssignDamage {
            attacker: b(3),
            amounts: amounts.clone()
        }]
    );
    let d = o.decision.as_mut().unwrap();
    d.legal_mask = vec![true];
    d.factored.as_mut().unwrap().damage[0].amounts = Some(amounts);
    assert_eq!(choose(&o), vec![Choice::FinishCombat]);
}
#[test]
fn heuristic_errors_never_default_to_pass() {
    assert!(matches!(
        Heuristic::new("future", 0),
        Err(Error::UnsupportedVersion)
    ));
    assert!(matches!(
        Heuristic::new(HEURISTIC_VERSION, 2),
        Err(Error::WrongSeat)
    ));
    let p = Heuristic::new(HEURISTIC_VERSION, 0).unwrap();
    let mut o = obs("future", vec![Choice::Pass]);
    assert_eq!(p.choose(&o), Err(Error::UnsupportedDecision));
    o = obs("priority", vec![Choice::Pass, Choice::Spell]);
    assert_eq!(p.choose(&o), Err(Error::UnsupportedDecision));
    o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(4) }]);
    // GH-197 enables Cavalry; retain strict rejection with unimplemented Shivan.
    o.view.hand[4].card = "viashino-pyromancer";
    assert_eq!(p.choose(&o), Err(Error::UnsupportedContent));
    o = obs("priority", vec![Choice::Pass]);
    o.decision.as_mut().unwrap().legal_mask.clear();
    assert_eq!(p.choose(&o), Err(Error::InvalidObservation));
    o = obs("priority", vec![Choice::Pass]);
    o.view.seat = 1;
    assert_eq!(p.choose(&o), Err(Error::WrongSeat));
    o.view.seat = 0;
    o.schema_version = 2;
    assert_eq!(p.choose(&o), Err(Error::UnsupportedVersion));
    o.schema_version = 1;
    o.decision = None;
    assert_eq!(p.choose(&o), Err(Error::Unavailable));
}

#[test]
fn heuristic_fodder_uses_vanilla_development_priority() {
    // GH-194 extends the supported domain: Fodder develops two vanilla 1/1s.
    // The declared strategy ranks this alongside the existing vanilla casts.
    let mut o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(1) }]);
    o.view.hand[1].card = "dragon-fodder";
    assert_eq!(choose(&o), vec![Choice::Cast { card: h(1) }]);
}

#[test]
fn token_policy_domain_requires_new_versions() {
    assert!(Heuristic::new("heuristic-m1-v1", 0).is_err());
    assert_ne!(mtg_policy::VERSION, "legal-random-m1-v1");
}

#[test]
fn creature_mana_policy_casts_and_pays_with_both_sources() {
    for key in ["llanowar-elves", "druid-of-the-cowl"] {
        let mut o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(1) }]);
        o.view.hand[1].card = key;
        assert_eq!(choose(&o), vec![Choice::Cast { card: h(1) }]);
        let mut o = obs(
            "payment",
            vec![Choice::CancelPayment, Choice::TapMana { card: b(0) }],
        );
        o.view
            .public_zones
            .iter_mut()
            .find(|z| z.zone == "battlefield")
            .unwrap()
            .cards[0]
            .card = key;
        assert_eq!(choose(&o), vec![Choice::TapMana { card: b(0) }]);
    }
}

#[test]
fn creature_mana_domain_requires_new_policy_versions() {
    assert!(Heuristic::new("heuristic-tokens-v1", 0).is_err());
    assert_ne!(mtg_policy::VERSION, "legal-random-tokens-v1");
}

#[test]
fn flying_reach_heuristic_excludes_illegal_pair_before_ranking() {
    let mut o = combat("blockers");
    o.decision
        .as_mut()
        .unwrap()
        .factored
        .as_mut()
        .unwrap()
        .forbidden_blocks = vec![(b(1), b(3))];
    assert_eq!(
        choose(&o),
        vec![Choice::SelectBlockers {
            blocks: vec![(b(1), b(0)), (b(2), b(3))]
        }]
    );
}

#[test]
fn cavalry_policy_cast_activation_target_finish_and_version_contract() {
    assert!(Heuristic::new("heuristic-reach-v1", 0).is_err());
    assert!(
        mtg_policy::LegalRandom::new("legal-random-reach-v1", mtg_policy::RNG_VERSION, 0, 0, 0)
            .is_err()
    );
    let mut o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(1) }]);
    o.view.hand[1].card = "axgard-cavalry";
    assert_eq!(choose(&o), vec![Choice::Cast { card: h(1) }]);
    let mut o = obs(
        "priority",
        vec![Choice::Pass, Choice::Activate { card: b(0) }],
    );
    o.view
        .public_zones
        .iter_mut()
        .find(|z| z.zone == "battlefield")
        .unwrap()
        .cards[0]
        .card = "axgard-cavalry";
    assert_eq!(choose(&o), vec![Choice::Activate { card: b(0) }]);
    let mut random =
        mtg_policy::LegalRandom::new(mtg_policy::VERSION, mtg_policy::RNG_VERSION, 197, 0, 0)
            .unwrap();
    assert!(random.choose(&o).is_ok());
    expect(
        "activation_target",
        vec![
            Choice::Target { card: b(0) },
            Choice::Target { card: b(1) },
            Choice::CancelActivation,
        ],
        Choice::Target { card: b(0) },
    );
    expect(
        "activation_target",
        vec![
            Choice::Target { card: b(0) },
            Choice::FinishActivation,
            Choice::CancelActivation,
        ],
        Choice::FinishActivation,
    );
}

#[test]
fn heuristic_trample_assigns_marked_lethal_and_remainder_to_defender() {
    let mut o = combat("combat_damage");
    let d = o.decision.as_mut().unwrap();
    d.legal_mask = vec![false];
    d.factored.as_mut().unwrap().damage = vec![DamageAllocation {
        attacker: b(3),
        power: 5,
        blockers: vec![b(2), b(1)],
        amounts: None,
        trample_lethal: Some(vec![1, 2]),
    }];
    assert_eq!(
        choose(&o),
        vec![Choice::AssignDamage {
            attacker: b(3),
            amounts: vec![(b(2), 1), (b(1), 2)]
        }]
    );
}

#[test]
fn heuristic_deathtouch_assigns_one_each_and_five_to_defender() {
    let mut o = combat("combat_damage");
    let d = o.decision.as_mut().unwrap();
    d.legal_mask = vec![false];
    d.factored.as_mut().unwrap().damage = vec![DamageAllocation {
        attacker: b(3),
        power: 7,
        blockers: vec![b(2), b(1)],
        amounts: None,
        trample_lethal: Some(vec![1, 1]),
    }];
    assert_eq!(
        choose(&o),
        vec![Choice::AssignDamage {
            attacker: b(3),
            amounts: vec![(b(2), 1), (b(1), 1)]
        }]
    );
}

#[test]
fn shivan_native_policy_cast_activate_pay_finish_and_version_contract() {
    assert!(Heuristic::new("heuristic-deathtouch-v1", 0).is_err());
    assert!(
        mtg_policy::LegalRandom::new(
            "legal-random-deathtouch-v1",
            mtg_policy::RNG_VERSION,
            0,
            0,
            0
        )
        .is_err()
    );
    let mut o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(1) }]);
    o.view.hand[1].card = "shivan-dragon";
    assert_eq!(choose(&o), vec![Choice::Cast { card: h(1) }]);
    let mut o = obs(
        "priority",
        vec![Choice::Pass, Choice::Activate { card: b(0) }],
    );
    o.view
        .public_zones
        .iter_mut()
        .find(|z| z.zone == "battlefield")
        .unwrap()
        .cards[0]
        .card = "shivan-dragon";
    assert_eq!(choose(&o), vec![Choice::Activate { card: b(0) }]);
    let mut random =
        mtg_policy::LegalRandom::new(mtg_policy::VERSION, mtg_policy::RNG_VERSION, 200, 0, 0)
            .unwrap();
    assert!(random.choose(&o).is_ok());
    for (choices, want) in [
        (
            vec![Choice::Pay { color: 3 }, Choice::CancelActivation],
            Choice::Pay { color: 3 },
        ),
        (
            vec![Choice::FinishActivation, Choice::CancelActivation],
            Choice::FinishActivation,
        ),
    ] {
        let o = obs("activation_payment", choices);
        assert_eq!(choose(&o), vec![want]);
        assert!(random.choose(&o).is_ok());
    }
}

#[test]
fn invoker_native_policy_cast_activate_pay_finish_and_version_contract() {
    assert!(Heuristic::new("heuristic-shivan-v1", 0).is_err());
    assert!(
        mtg_policy::LegalRandom::new("legal-random-shivan-v1", mtg_policy::RNG_VERSION, 0, 0, 0)
            .is_err()
    );
    let mut o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(1) }]);
    o.view.hand[1].card = "wildheart-invoker";
    assert_eq!(choose(&o), vec![Choice::Cast { card: h(1) }]);
    let mut o = obs(
        "priority",
        vec![Choice::Pass, Choice::Activate { card: b(0) }],
    );
    o.view
        .public_zones
        .iter_mut()
        .find(|z| z.zone == "battlefield")
        .unwrap()
        .cards[0]
        .card = "wildheart-invoker";
    assert_eq!(choose(&o), vec![Choice::Activate { card: b(0) }]);
    let mut random =
        mtg_policy::LegalRandom::new(mtg_policy::VERSION, mtg_policy::RNG_VERSION, 200, 0, 0)
            .unwrap();
    assert!(random.choose(&o).is_ok());
    for (choices, want) in [
        (
            vec![Choice::Pay { color: 4 }, Choice::CancelActivation],
            Choice::Pay { color: 4 },
        ),
        (
            vec![Choice::FinishActivation, Choice::CancelActivation],
            Choice::FinishActivation,
        ),
    ] {
        let o = obs("activation_payment", choices);
        assert_eq!(choose(&o), vec![want]);
        assert!(random.choose(&o).is_ok());
    }
}

#[test]
fn thrill_policy_cast_discard_payment_and_previous_version_rejection() {
    assert!(Heuristic::new("heuristic-invoker-v1", 0).is_err());
    assert!(
        mtg_policy::LegalRandom::new("legal-random-invoker-v1", mtg_policy::RNG_VERSION, 0, 0, 0)
            .is_err()
    );
    let mut o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(1) }]);
    o.view.hand[1].card = "thrill-of-possibility";
    assert_eq!(choose(&o), vec![Choice::Cast { card: h(1) }]);
    let mut random =
        mtg_policy::LegalRandom::new(mtg_policy::VERSION, mtg_policy::RNG_VERSION, 202, 0, 0)
            .unwrap();
    assert!(random.choose(&o).is_ok());
    let o = obs(
        "cast_discard",
        vec![Choice::Discard { card: h(0) }, Choice::CancelPayment],
    );
    assert_eq!(choose(&o), vec![Choice::Discard { card: h(0) }]);
    assert!(random.choose(&o).is_ok());
}

#[test]
fn surprise_policy_modes_and_previous_version_rejection() {
    assert!(Heuristic::new("heuristic-thrill-v1", 0).is_err());
    let mut o = obs("priority", vec![Choice::Pass, Choice::Cast { card: h(1) }]);
    o.view.hand[1].card = "goblin-surprise";
    assert_eq!(choose(&o), vec![Choice::Cast { card: h(1) }]);
    let o = obs(
        "cast_mode",
        vec![
            Choice::Mode { mode: 0 },
            Choice::Mode { mode: 1 },
            Choice::CancelPayment,
        ],
    );
    assert_eq!(choose(&o), vec![Choice::Mode { mode: 1 }]);
    let mut random =
        mtg_policy::LegalRandom::new(mtg_policy::VERSION, mtg_policy::RNG_VERSION, 203, 0, 0)
            .unwrap();
    let mut seen = [false; 3];
    for _ in 0..100 {
        let c = random.choose(&o).unwrap();
        match c.choices[0] {
            Choice::Mode { mode } => seen[mode as usize] = true,
            Choice::CancelPayment => seen[2] = true,
            _ => panic!("unexpected"),
        }
    }
    assert_eq!(seen, [true; 3]);
}
