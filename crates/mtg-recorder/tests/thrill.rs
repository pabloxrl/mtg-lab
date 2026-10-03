//! Real ordered-deck Thrill play. Pinned Oracle and CR 601.2h, 121, 608.
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
fn thrill_normal_reset_discard_draw_capture_replay() {
    let mut history = None;
    let mut trajectory = None;
    for captured in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
            .unwrap();
            let mut order: Vec<String> = [
                "thrill-of-possibility",
                "mountain",
                "mountain",
                "mountain",
                "mountain",
                "mountain",
                "mountain",
                "shivan-dragon",
                "swab-goblin",
                "dragon-fodder",
            ]
            .into_iter()
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
                id: "20200000-1234-4234-8234-123456789abc".into(),
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
                d.reset_captured(&run.config, 202, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 202, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, C::Keep, q);
            send(&mut d, Seat::P1, C::Keep, q);
            let mut landed = 0;
            let mut cast = false;
            let mut committed = false;
            let mut checked = false;
            for _ in 0..200 {
                let actor = if d.observe(Seat::P0).unwrap().view.acting_seat == Some(0) {
                    Seat::P0
                } else {
                    Seat::P1
                };
                let o = d.observe(actor).unwrap();
                let t = o.view.turn.unwrap();
                let dec = o.decision.as_ref().unwrap();
                let legal: Vec<_> = dec
                    .candidates
                    .iter()
                    .zip(&dec.legal_mask)
                    .filter(|(_, m)| **m)
                    .map(|(c, _)| c.clone())
                    .collect();
                if committed && o.stack.is_empty() {
                    let own = d.observe(Seat::P0).unwrap();
                    let mut hand: Vec<_> = own.view.hand.iter().map(|c| c.card).collect();
                    hand.sort();
                    assert_eq!(
                        hand,
                        [
                            "dragon-fodder",
                            "mountain",
                            "mountain",
                            "mountain",
                            "shivan-dragon",
                            "swab-goblin"
                        ]
                    );
                    let grave: Vec<_> = own
                        .view
                        .public_zones
                        .iter()
                        .filter(|z| z.zone == "graveyard_0")
                        .flat_map(|z| z.cards.iter().map(|c| c.card))
                        .collect();
                    assert_eq!(grave, ["mountain", "thrill-of-possibility"]);
                    checked = true;
                    break;
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
                        } else if t.0 == 3 && !cast {
                            cast = true;
                            C::Cast {
                                card: r(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| c.card == "thrill-of-possibility")
                                        .unwrap(),
                                ),
                            }
                        } else {
                            C::Pass
                        }
                    }
                    "cast_discard" => {
                        let before = d.privileged_snapshot();
                        let h = d.privileged_history().to_vec();
                        let tr = serde_json::to_value(d.trajectory()).unwrap();
                        for seat in [Seat::P0, Seat::P1] {
                            assert!(
                                d.submit(
                                    seat,
                                    &Submission {
                                        schema_version: 1,
                                        revision: dec.revision,
                                        generation: dec.generation,
                                        choices: vec![C::FinishPayment]
                                    }
                                )
                                .is_err()
                            );
                        }
                        assert_eq!(before, d.privileged_snapshot());
                        assert_eq!(h, d.privileged_history());
                        assert_eq!(tr, serde_json::to_value(d.trajectory()).unwrap());
                        assert!(d.observe(Seat::P1).unwrap().pending.is_none());
                        C::Discard {
                            card: r(
                                VisibleZone::Hand,
                                o.view
                                    .hand
                                    .iter()
                                    .position(|c| c.card == "mountain")
                                    .unwrap(),
                            ),
                        }
                    }
                    "payment" => {
                        if legal.contains(&C::FinishPayment) {
                            committed = true;
                            C::FinishPayment
                        } else if legal.contains(&C::Pay { color: 3 }) {
                            C::Pay { color: 3 }
                        } else {
                            legal
                                .into_iter()
                                .find(|c| matches!(c, C::TapMana { .. }))
                                .unwrap()
                        }
                    }
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
                    "priority" => C::Pass,
                    "attackers" | "blockers" | "combat_damage" => C::FinishCombat,
                    _ => panic!("unexpected {}", dec.kind),
                };
                send(&mut d, actor, c, q);
                if committed && !d.observe(Seat::P0).unwrap().stack.is_empty() {
                    let own = d.observe(Seat::P0).unwrap();
                    assert_eq!(own.view.hand.len(), 4);
                    assert_eq!(own.view.mana[0], [0; 6]);
                    assert_eq!(
                        own.view
                            .public_zones
                            .iter()
                            .filter(|z| z.zone == "graveyard_0")
                            .flat_map(|z| z.cards.iter().map(|c| c.card))
                            .collect::<Vec<_>>(),
                        ["mountain"]
                    );
                }
            }
            assert!(checked && cast && committed);
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
            let mut result = d.finish().unwrap();
            let h = result.privileged_history().to_vec();
            if let Some(e) = &history {
                assert_eq!(&h, e)
            } else {
                history = Some(h)
            }
            if captured {
                let mut registry = mtg_core::episode::replay::Registry::default();
                registry.register("thrill-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("thrill-private", &result, |_, _| true)
                    .unwrap();
                assert_eq!(
                    mtg_core::game::replay::played::verify(bytes)
                        .unwrap()
                        .life(),
                    [20, 20]
                );
                let converted = from_core_v2(result.trajectory().unwrap()).unwrap();
                assert!(converted.decisions.iter().any(|d| {
                    d.observation
                        .decision
                        .as_ref()
                        .is_some_and(|d| d.kind == "cast_discard")
                }));
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
                    assert_eq!(&value, e)
                } else {
                    trajectory = Some(value)
                }
            }
        }
    }
}
