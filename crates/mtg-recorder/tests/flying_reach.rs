//! GH-196 normal-reset played Sentry, CR 601/509/510/702.17. Literal pinned 3G 4/4.
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
    .unwrap_or_else(|e| panic!("{seat:?} {:?} {} {c:?}: {e:?}", o.view.turn, t.kind));
    for _ in 0..1000 {
        if d.advance(q).unwrap() != Progress::InternalYield {
            return;
        }
    }
    panic!("quantum bound");
}
fn reference(zone: VisibleZone, row: usize) -> VisibleRef {
    VisibleRef { zone, row }
}
#[test]
fn flying_reach_normal_reset_sentry_cast_combat_capture_replay() {
    let mut expected_history = None;
    let mut expected_trajectory = None;
    for captured in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
            .unwrap();
            let mut order: Vec<String> = [
                "magnigoth-sentry",
                "bear-cub",
                "forest",
                "forest",
                "forest",
                "forest",
                "forest",
            ]
            .into_iter()
            .map(String::from)
            .collect();
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
                id: "19600000-1234-4234-8234-123456789abc".into(),
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
                d.reset_captured(&run.config, 196, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 196, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, C::Keep, q);
            send(&mut d, Seat::P1, C::Keep, q);
            let mut landed = [0, 0];
            let mut cast = [false, false];
            let mut attacked = false;
            let mut blocked = false;
            let mut checked = false;
            for _ in 0..1000 {
                let v = d.observe(Seat::P0).unwrap().view;
                let actor = if v.acting_seat == Some(0) {
                    Seat::P0
                } else {
                    Seat::P1
                };
                let seat = usize::from(actor == Seat::P1);
                let o = d.observe(actor).unwrap();
                let t = o.view.turn.unwrap();
                let dec = o.decision.unwrap();
                let bf = &o
                    .view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .unwrap()
                    .cards;
                if t.0 == 9 && t.2 == "combat_damage" && dec.kind == "priority" {
                    let sentry = bf.iter().find(|c| c.card == "magnigoth-sentry").unwrap();
                    assert_eq!(sentry.creature, Some([4, 4, 2]));
                    assert!(!bf.iter().any(|c| c.card == "bear-cub"));
                    checked = true;
                    break;
                }
                let choice = match dec.kind {
                    "priority" if t.2 == "precombat_main" && t.1 == seat as u8 => {
                        if landed[seat] != t.0
                            && ((seat == 0 && t.0 <= 7) || (seat == 1 && t.0 <= 4))
                        {
                            landed[seat] = t.0;
                            C::PlayLand {
                                card: reference(
                                    VisibleZone::Hand,
                                    o.view.hand.iter().position(|c| c.card == "forest").unwrap(),
                                ),
                            }
                        } else if !cast[seat] && t.0 == if seat == 0 { 7 } else { 4 } {
                            cast[seat] = true;
                            C::Cast {
                                card: reference(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| {
                                            c.card
                                                == if seat == 0 {
                                                    "magnigoth-sentry"
                                                } else {
                                                    "bear-cub"
                                                }
                                        })
                                        .unwrap(),
                                ),
                            }
                        } else {
                            C::Pass
                        }
                    }
                    "priority" => C::Pass,
                    "payment" => {
                        // Only scripted Forest mana and green spending; do not choose arbitrary spells/targets.
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
                                .find_map(|(c, m)| match c {
                                    C::TapMana { card } if *m && bf[card.row].card == "forest" => {
                                        Some(c.clone())
                                    }
                                    _ => None,
                                })
                                .expect("scripted Forest source")
                        }
                    }
                    "attackers" if t.0 == 9 && !attacked => {
                        attacked = true;
                        C::SelectAttackers {
                            cards: vec![reference(
                                VisibleZone::Battlefield,
                                bf.iter()
                                    .position(|c| c.card == "magnigoth-sentry")
                                    .unwrap(),
                            )],
                        }
                    }
                    "blockers" if t.0 == 9 && !blocked => {
                        blocked = true;
                        C::SelectBlockers {
                            blocks: vec![(
                                reference(
                                    VisibleZone::Battlefield,
                                    bf.iter().position(|c| c.card == "bear-cub").unwrap(),
                                ),
                                reference(
                                    VisibleZone::Battlefield,
                                    bf.iter()
                                        .position(|c| c.card == "magnigoth-sentry")
                                        .unwrap(),
                                ),
                            )],
                        }
                    }
                    "attackers" | "blockers" | "combat_damage" => C::FinishCombat,
                    "cleanup_discard" => C::Discard {
                        card: reference(
                            VisibleZone::Hand,
                            o.view.hand.iter().position(|c| c.card == "forest").unwrap(),
                        ),
                    },
                    _ => panic!("unexpected scripted decision {}", dec.kind),
                };
                send(&mut d, actor, choice, q);
            }
            assert!(checked && attacked && blocked && cast == [true, true]);
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
            let mut result = d.finish().unwrap();
            let history = result.privileged_history().to_vec();
            if let Some(ref e) = expected_history {
                assert_eq!(&history, e);
            } else {
                expected_history = Some(history);
            }
            if captured {
                let mut registry = mtg_core::episode::replay::Registry::default();
                registry.register("sentry-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("sentry-private", &result, |_, _| true)
                    .unwrap();
                let replayed = mtg_core::game::replay::played::verify(bytes).unwrap();
                assert_eq!(replayed.life(), [20, 20]);
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
                if let Some(ref e) = expected_trajectory {
                    assert_eq!(&value, e);
                } else {
                    expected_trajectory = Some(value);
                }
            }
        }
    }
}
