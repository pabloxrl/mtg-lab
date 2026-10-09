//! Real ordered-deck reset and played Invoker. Pinned 2GG 4/3; eight generic grants +5/+5 and trample; CR 602/611.2.
use mtg_core::{
    episode::{Driver, Progress},
    game::{
        Config, DeckConfig,
        policy::{Choice as C, Submission, VisibleRef, VisibleZone},
    },
    objects::Seat,
    trajectory::Limits,
};
use mtg_recorder::{Backpressure, Writer, collector::Run, from_core_v2, read_v2};
use std::num::NonZeroUsize;
fn r(zone: VisibleZone, row: usize) -> VisibleRef {
    VisibleRef { zone, row }
}
fn send(d: &mut Driver, seat: Seat, c: C, q: NonZeroUsize) {
    let o = d.observe(seat).unwrap();
    let t = o.decision.unwrap();
    d.submit(
        seat,
        &Submission {
            schema_version: 1,
            revision: t.revision,
            generation: t.generation,
            choices: vec![c.clone()],
        },
    )
    .unwrap_or_else(|e| panic!("{t:?} {c:?}: {e:?}"));
    for _ in 0..1000 {
        if d.advance(q).unwrap() != Progress::InternalYield {
            return;
        }
    }
    panic!("work bound");
}
#[test]
fn invoker_normal_reset_paid_activation_capture_replay() {
    played_activation(false);
}

#[test]
fn activation_mana_invoker_normal_reset_payment_sources_capture_replay() {
    played_activation(true);
}

fn played_activation(during_payment: bool) {
    let mut history = None;
    let mut trajectory = None;
    for captured in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
            .unwrap();
            let mut order: Vec<String> = std::iter::once("wildheart-invoker")
                .chain(std::iter::repeat_n("forest", 16))
                .map(String::from)
                .collect();
            if during_payment {
                order.insert(1, "bear-cub".into());
            }
            for c in manifest["decks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["id"] == "green")
                .unwrap()["cards"]
                .as_array()
                .unwrap()
            {
                let key = c["card_id"].as_str().unwrap();
                for _ in order.iter().filter(|s| s.as_str() == key).count()
                    ..c["copies"].as_u64().unwrap() as usize
                {
                    order.push(key.into());
                }
            }
            let run = Run {
                id: "20100000-1234-4234-8234-123456789abc".into(),
                config: Config {
                    seats: vec![
                        DeckConfig {
                            deck: "green".into(),
                            order: Some(order)
                        };
                        2
                    ],
                    ..Config::default()
                },
                policies: ["script".into(), "script".into()],
                limits: Limits::default(),
                first_ordinal: 0,
                started: 1,
            };
            let mut d = Driver::new(256).unwrap();
            if captured {
                d.reset_captured(&run.config, 201, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 201, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, C::Keep, q);
            send(&mut d, Seat::P1, C::Keep, q);
            let mut landed = 0;
            let mut cast = false;
            let mut cub_cast = false;
            let mut activations = 0;
            let mut paid = 0;
            let mut boosted = false;
            let mut checked = false;
            for _ in 0..3000 {
                let actor = if d.observe(Seat::P0).unwrap().view.acting_seat == Some(0) {
                    Seat::P0
                } else {
                    Seat::P1
                };
                let o = d.observe(actor).unwrap();
                let t = o.view.turn.unwrap();
                let dec = o.decision.as_ref().unwrap();
                if during_payment && matches!(dec.kind, "activation_target" | "activation_payment")
                {
                    for random in [false, true] {
                        let mut probe = mtg_core::game::Game::new().unwrap();
                        probe.restore(&d.privileged_snapshot()).unwrap();
                        let input = probe.policy_observe(actor, 256).unwrap();
                        let proposed = if random {
                            mtg_policy::LegalRandom::new(
                                mtg_policy::VERSION,
                                mtg_policy::RNG_VERSION,
                                254,
                                0,
                                0,
                            )
                            .unwrap()
                            .choose(&input)
                            .unwrap()
                        } else {
                            mtg_policy::Heuristic::new(mtg_policy::HEURISTIC_VERSION, 0)
                                .unwrap()
                                .choose(&input)
                                .unwrap()
                        };
                        probe.apply_policy(actor, &proposed, 256).unwrap();
                    }
                }
                let bf = &o
                    .view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .unwrap()
                    .cards;
                if let Some(dragon) = bf.iter().find(|c| {
                    c.card
                        == if during_payment {
                            "bear-cub"
                        } else {
                            "wildheart-invoker"
                        }
                }) {
                    if activations == 1 && o.stack.is_empty() && t.0 == 23 {
                        let stats = dragon.creature.as_ref().unwrap();
                        assert_eq!(
                            (stats[0], stats[1]),
                            if during_payment { (7, 7) } else { (9, 8) }
                        );
                        assert_eq!(dragon.summoning_sick, !during_payment);
                        for seat in [Seat::P0, Seat::P1] {
                            let view = serde_json::to_value(d.observe(seat).unwrap()).unwrap();
                            let creature = view["view"]["public_zones"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .flat_map(|z| z["cards"].as_array().unwrap())
                                .find(|c| {
                                    c["card"]
                                        == if during_payment {
                                            "bear-cub"
                                        } else {
                                            "wildheart-invoker"
                                        }
                                })
                                .unwrap();
                            assert_eq!(
                                creature["trample"], true,
                                "CR 611.2: granted trample is public before combat"
                            );
                        }
                        boosted = true;
                    }
                    if t.0 == 24 && t.2 == "upkeep" {
                        let stats = dragon.creature.as_ref().unwrap();
                        assert_eq!(
                            (stats[0], stats[1]),
                            if during_payment { (2, 2) } else { (4, 3) }
                        );
                        let value = serde_json::to_value(dragon).unwrap();
                        assert!(
                            !value["trample"].as_bool().unwrap_or(false),
                            "CR 514.2: cleanup expires granted trample"
                        );
                        checked = true;
                        break;
                    }
                }
                let c = match dec.kind {
                    "priority" if actor == Seat::P0 && t.1 == 0 && t.2 == "precombat_main" => {
                        if landed != t.0 {
                            landed = t.0;
                            C::PlayLand {
                                card: r(
                                    VisibleZone::Hand,
                                    o.view.hand.iter().position(|c| c.card == "forest").unwrap(),
                                ),
                            }
                        } else if during_payment && t.0 == 3 && !cub_cast {
                            cub_cast = true;
                            C::Cast {
                                card: r(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| c.card == "bear-cub")
                                        .unwrap(),
                                ),
                            }
                        } else if t.0 == 23 && !cast {
                            cast = true;
                            C::Cast {
                                card: r(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| c.card == "wildheart-invoker")
                                        .unwrap(),
                                ),
                            }
                        } else if t.0 == 23
                            && bf.iter().any(|c| c.card == "wildheart-invoker")
                            && activations < 1
                        {
                            if !during_payment && o.view.mana[0][4] < 8 {
                                C::TapMana {
                                    card: r(
                                        VisibleZone::Battlefield,
                                        bf.iter()
                                            .position(|c| {
                                                c.card == "forest" && c.controller == 0 && !c.tapped
                                            })
                                            .unwrap(),
                                    ),
                                }
                            } else {
                                paid = 0;
                                C::Activate {
                                    card: r(
                                        VisibleZone::Battlefield,
                                        bf.iter()
                                            .position(|c| c.card == "wildheart-invoker")
                                            .unwrap(),
                                    ),
                                }
                            }
                        } else {
                            C::Pass
                        }
                    }
                    "activation_target" => C::Target {
                        card: r(
                            VisibleZone::Battlefield,
                            bf.iter()
                                .position(|c| {
                                    c.card
                                        == if during_payment {
                                            "bear-cub"
                                        } else {
                                            "wildheart-invoker"
                                        }
                                })
                                .unwrap(),
                        ),
                    },
                    "activation_payment" => {
                        let before = d.privileged_snapshot();
                        let h = d.privileged_history().to_vec();
                        let tr = serde_json::to_value(d.trajectory()).unwrap();
                        assert!(
                            d.submit(
                                actor,
                                &Submission {
                                    schema_version: 1,
                                    revision: dec.revision,
                                    generation: dec.generation,
                                    choices: vec![C::Target {
                                        card: r(VisibleZone::Battlefield, 0)
                                    }]
                                }
                            )
                            .is_err()
                        );
                        assert_eq!(before, d.privileged_snapshot());
                        assert_eq!(h, d.privileged_history());
                        assert_eq!(tr, serde_json::to_value(d.trajectory()).unwrap());
                        if during_payment
                            && !dec
                                .candidates
                                .iter()
                                .zip(&dec.legal_mask)
                                .any(|(c, legal)| {
                                    *legal
                                        && (*c == C::Pay { color: 4 } || *c == C::FinishActivation)
                                })
                        {
                            assert!(o.stack.is_empty());
                            assert_eq!(o.view.mana[0][4], 0);
                            dec.candidates
                                .iter()
                                .zip(&dec.legal_mask)
                                .find_map(|(c, legal)| {
                                    (*legal && matches!(c, C::TapMana { .. })).then(|| c.clone())
                                })
                                .expect("explicit payment source")
                        } else if paid == 8 {
                            activations += 1;
                            C::FinishActivation
                        } else {
                            paid += 1;
                            C::Pay { color: 4 }
                        }
                    }
                    "payment" => {
                        if dec
                            .candidates
                            .iter()
                            .zip(&dec.legal_mask)
                            .any(|(c, m)| *m && *c == C::FinishPayment)
                        {
                            C::FinishPayment
                        } else if dec
                            .candidates
                            .iter()
                            .zip(&dec.legal_mask)
                            .any(|(c, m)| *m && *c == C::Pay { color: 4 })
                        {
                            C::Pay { color: 4 }
                        } else {
                            dec.candidates
                                .iter()
                                .zip(&dec.legal_mask)
                                .find_map(|(c, m)| {
                                    if *m && matches!(c, C::TapMana { .. }) {
                                        Some(c.clone())
                                    } else {
                                        None
                                    }
                                })
                                .unwrap()
                        }
                    }
                    "priority" => C::Pass,
                    "attackers" | "blockers" | "combat_damage" => C::FinishCombat,
                    "cleanup_discard" => C::Discard {
                        card: r(
                            VisibleZone::Hand,
                            o.view.hand.iter().position(|c| c.card == "forest").unwrap(),
                        ),
                    },
                    _ => panic!("unexpected {}", dec.kind),
                };
                send(&mut d, actor, c, q);
            }
            assert!(checked && boosted && cast && activations == 1);
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
            let mut result = d.finish().unwrap();
            let h = result.privileged_history().to_vec();
            if let Some(e) = &history {
                assert_eq!(&h, e);
            } else {
                history = Some(h);
            }
            if captured {
                let mut registry = mtg_core::episode::replay::Registry::default();
                registry.register("invoker-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("invoker-private", &result, |_, _| true)
                    .unwrap();
                assert_eq!(
                    mtg_core::game::replay::played::verify(bytes)
                        .unwrap()
                        .life(),
                    [20, 20]
                );
                let converted = from_core_v2(result.trajectory().unwrap()).unwrap();
                if during_payment {
                    let taps: Vec<_> = converted
                        .decisions
                        .iter()
                        .filter(|decision| {
                            decision
                                .observation
                                .decision
                                .as_ref()
                                .is_some_and(|d| d.kind == "activation_payment")
                                && decision.choice.submission.choices.iter().any(|c| {
                                    matches!(c, mtg_recorder::structured::Command::TapMana { .. })
                                })
                        })
                        .collect();
                    assert_eq!(taps.len(), 8);
                    assert!(
                        taps.iter().all(|d| d.choice.status
                            == mtg_recorder::structured::ActionStatus::Continuing)
                    );
                }

                assert!(converted.decisions.iter().any(|decision| {
                    decision
                        .observation
                        .view
                        .public_zones
                        .iter()
                        .flat_map(|zone| &zone.cards)
                        .any(|card| {
                            card.card
                                == if during_payment {
                                    "bear-cub"
                                } else {
                                    "wildheart-invoker"
                                }
                                && card.trample
                                && card.creature
                                    == Some(if during_payment { [7, 7, 0] } else { [9, 8, 0] })
                        })
                }));
                assert!(
                    converted
                        .footer
                        .as_ref()
                        .unwrap()
                        .final_observations
                        .iter()
                        .all(|observation| observation
                            .view
                            .public_zones
                            .iter()
                            .flat_map(|zone| &zone.cards)
                            .filter(|card| card.card
                                == if during_payment {
                                    "bear-cub"
                                } else {
                                    "wildheart-invoker"
                                })
                            .all(|card| !card.trample))
                );
                let mut w = Writer::new_v2(Vec::new(), 4_000_000, Backpressure::Block).unwrap();
                w.append_v2(&converted).unwrap();
                let bytes = w.finish().unwrap();
                assert_eq!(
                    read_v2(bytes.as_slice(), 4_000_000).unwrap(),
                    vec![converted.clone()]
                );
                assert_eq!(converted.footer.as_ref().unwrap().returns, [1, -1]);
                let value = serde_json::to_value(converted).unwrap();
                if let Some(e) = &trajectory {
                    assert_eq!(&value, e);
                } else {
                    trajectory = Some(value);
                }
            }
        }
    }
}
