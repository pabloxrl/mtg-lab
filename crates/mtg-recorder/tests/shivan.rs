//! Real ordered-deck reset and played Shivan. Pinned 4RR 5/5 flying; CR 602/611.2.
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
fn shivan_normal_reset_paid_activation_capture_replay() {
    let mut history = None;
    let mut trajectory = None;
    for captured in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
            .unwrap();
            let mut order: Vec<String> = std::iter::once("shivan-dragon")
                .chain(std::iter::repeat_n("mountain", 16))
                .map(String::from)
                .collect();
            for c in manifest["decks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["id"] == "red")
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
                id: "20000000-1234-4234-8234-123456789abc".into(),
                config: Config {
                    seats: vec![
                        DeckConfig {
                            deck: "red".into(),
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
                d.reset_captured(&run.config, 200, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 200, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, C::Keep, q);
            send(&mut d, Seat::P1, C::Keep, q);
            let mut landed = 0;
            let mut cast = false;
            let mut activations = 0;
            let mut paid = false;
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
                let bf = &o
                    .view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .unwrap()
                    .cards;
                if let Some(dragon) = bf.iter().find(|c| c.card == "shivan-dragon") {
                    if activations == 2 && o.stack.is_empty() && t.0 == 15 {
                        let stats = dragon.creature.as_ref().unwrap();
                        assert_eq!((stats[0], stats[1]), (7, 5));
                        assert!(dragon.summoning_sick);
                        boosted = true;
                    }
                    if t.0 == 16 && t.2 == "upkeep" {
                        let stats = dragon.creature.as_ref().unwrap();
                        assert_eq!((stats[0], stats[1]), (5, 5));
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
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| c.card == "mountain")
                                        .unwrap(),
                                ),
                            }
                        } else if t.0 == 15 && !cast {
                            cast = true;
                            C::Cast {
                                card: r(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| c.card == "shivan-dragon")
                                        .unwrap(),
                                ),
                            }
                        } else if t.0 == 15
                            && bf.iter().any(|c| c.card == "shivan-dragon")
                            && activations < 2
                        {
                            if o.view.mana[0][3] == 0 {
                                C::TapMana {
                                    card: r(
                                        VisibleZone::Battlefield,
                                        bf.iter()
                                            .position(|c| {
                                                c.card == "mountain"
                                                    && c.controller == 0
                                                    && !c.tapped
                                            })
                                            .unwrap(),
                                    ),
                                }
                            } else {
                                paid = false;
                                C::Activate {
                                    card: r(
                                        VisibleZone::Battlefield,
                                        bf.iter().position(|c| c.card == "shivan-dragon").unwrap(),
                                    ),
                                }
                            }
                        } else {
                            C::Pass
                        }
                    }
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
                        if paid {
                            activations += 1;
                            C::FinishActivation
                        } else {
                            paid = true;
                            C::Pay { color: 3 }
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
                            .any(|(c, m)| *m && *c == C::Pay { color: 3 })
                        {
                            C::Pay { color: 3 }
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
                            o.view
                                .hand
                                .iter()
                                .position(|c| c.card == "mountain")
                                .unwrap(),
                        ),
                    },
                    _ => panic!("unexpected {}", dec.kind),
                };
                send(&mut d, actor, c, q);
            }
            assert!(checked && boosted && cast && activations == 2);
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
                registry.register("shivan-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("shivan-private", &result, |_, _| true)
                    .unwrap();
                assert_eq!(
                    mtg_core::game::replay::played::verify(bytes)
                        .unwrap()
                        .life(),
                    [20, 20]
                );
                let converted = from_core_v2(result.trajectory().unwrap()).unwrap();
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
