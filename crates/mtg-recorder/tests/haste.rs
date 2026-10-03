//! GH-197 normal-reset Cavalry gives freshly cast Swab haste. CR 602/702.10/611.2.
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
fn haste_normal_reset_cavalry_activation_capture_replay() {
    played(None);
}

#[test]
fn haste_pending_concession_terminal_capture_and_replay() {
    played(Some(false));
    played(Some(true));
}

fn played(concede_stage: Option<bool>) {
    let mut expected_history = None;
    let mut expected_trajectory = None;
    for captured in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
            .unwrap();
            let mut order: Vec<String> = [
                "axgard-cavalry",
                "swab-goblin",
                "mountain",
                "mountain",
                "mountain",
                "mountain",
                "mountain",
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
                id: "19700000-1234-4234-8234-123456789abc".into(),
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
                d.reset_captured(&run.config, 197, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 197, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, C::Keep, q);
            send(&mut d, Seat::P1, C::Keep, q);
            let mut landed = [0, 0];
            let mut cast = [false, false];
            let mut activated = false;
            let mut selected = false;
            let mut attacked = false;

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
                if dec.kind == "activation_target" && concede_stage == Some(selected) {
                    checked = true;
                    break;
                }
                if t.0 == 6 && t.2 == "upkeep" {
                    assert_eq!(o.view.life, [20, 18]);
                    assert!(!bf.iter().find(|c| c.card == "swab-goblin").unwrap().haste);
                    let swab = bf.iter().find(|c| c.card == "swab-goblin").unwrap();
                    assert!(swab.summoning_sick, "temporary haste expires at cleanup");
                    checked = true;
                    break;
                }
                let choice = match dec.kind {
                    "priority" if t.2 == "precombat_main" && t.1 == seat as u8 => {
                        if seat == 0 && landed[seat] != t.0 && t.0 <= 5 {
                            landed[seat] = t.0;
                            C::PlayLand {
                                card: reference(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| c.card == "mountain")
                                        .unwrap(),
                                ),
                            }
                        } else if seat == 0 && ((t.0 == 3 && !cast[0]) || (t.0 == 5 && !cast[1])) {
                            let index = usize::from(t.0 == 5);
                            cast[index] = true;
                            C::Cast {
                                card: reference(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| {
                                            c.card
                                                == if index == 0 {
                                                    "axgard-cavalry"
                                                } else {
                                                    "swab-goblin"
                                                }
                                        })
                                        .unwrap(),
                                ),
                            }
                        } else if seat == 0
                            && t.0 == 5
                            && !activated
                            && bf.iter().any(|c| c.card == "swab-goblin")
                        {
                            activated = true;
                            assert!(
                                bf.iter()
                                    .find(|c| c.card == "swab-goblin")
                                    .unwrap()
                                    .summoning_sick
                            );
                            C::Activate {
                                card: reference(
                                    VisibleZone::Battlefield,
                                    bf.iter().position(|c| c.card == "axgard-cavalry").unwrap(),
                                ),
                            }
                        } else {
                            C::Pass
                        }
                    }
                    "activation_target" => {
                        if selected {
                            C::FinishActivation
                        } else {
                            selected = true;
                            C::Target {
                                card: reference(
                                    VisibleZone::Battlefield,
                                    bf.iter().position(|c| c.card == "swab-goblin").unwrap(),
                                ),
                            }
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
                            .any(|(c, m)| *m && *c == C::Pay { color: 3 })
                        {
                            C::Pay { color: 3 }
                        } else {
                            dec.candidates
                                .iter()
                                .zip(&dec.legal_mask)
                                .find_map(|(c, m)| match c {
                                    C::TapMana { card }
                                        if *m && bf[card.row].card == "mountain" =>
                                    {
                                        Some(c.clone())
                                    }
                                    _ => None,
                                })
                                .expect("scripted Forest source")
                        }
                    }
                    "attackers" if t.0 == 5 && !attacked => {
                        attacked = true;
                        let row = bf.iter().position(|c| c.card == "swab-goblin").unwrap();
                        assert!(!bf[row].summoning_sick);
                        assert!(bf[row].haste);
                        C::SelectAttackers {
                            cards: vec![reference(VisibleZone::Battlefield, row)],
                        }
                    }
                    "attackers" | "blockers" | "combat_damage" => C::FinishCombat,
                    "cleanup_discard" => C::Discard {
                        card: reference(
                            VisibleZone::Hand,
                            o.view
                                .hand
                                .iter()
                                .position(|c| c.card == "mountain")
                                .unwrap(),
                        ),
                    },
                    _ => panic!("unexpected scripted decision {}", dec.kind),
                };
                if dec.kind == "activation_target" {
                    let before = d.privileged_snapshot();
                    let history = d.privileged_history().to_vec();
                    let capture = serde_json::to_value(d.trajectory()).unwrap();
                    let land = bf.iter().position(|c| c.card == "mountain").unwrap();
                    let bad = Submission {
                        schema_version: 1,
                        revision: dec.revision,
                        generation: dec.generation,
                        choices: vec![C::Target {
                            card: reference(VisibleZone::Battlefield, land),
                        }],
                    };
                    assert!(d.submit(actor, &bad).is_err());
                    assert_eq!(d.privileged_snapshot(), before);
                    assert_eq!(d.privileged_history(), history);
                    assert_eq!(serde_json::to_value(d.trajectory()).unwrap(), capture);
                }
                send(&mut d, actor, choice, q);
            }
            assert!(checked && activated && cast == [true, true]);
            if let Some(stage) = concede_stage {
                assert_eq!(selected, stage);
                assert!(!attacked);
            } else {
                assert!(attacked && selected);
            }
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
            let before = d.privileged_snapshot();
            let history = d.privileged_history().to_vec();
            assert!(
                d.submit(
                    Seat::P0,
                    &Submission {
                        schema_version: 1,
                        revision: 0,
                        generation: 0,
                        choices: vec![C::FinishActivation]
                    }
                )
                .is_err()
            );
            assert_eq!(d.privileged_snapshot(), before);
            assert_eq!(d.privileged_history(), history);
            let mut result = d.finish().unwrap();
            for o in result.final_observations().unwrap() {
                assert!(o.decision.is_none());
                assert!(o.pending.is_none());
                assert_eq!(o.view.acting_seat, None);
                assert!(o.view.terminal.is_some());
            }
            let history = result.privileged_history().to_vec();
            if let Some(ref e) = expected_history {
                assert_eq!(&history, e);
            } else {
                expected_history = Some(history);
            }
            if captured {
                let mut registry = mtg_core::episode::replay::Registry::default();
                registry.register("cavalry-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("cavalry-private", &result, |_, _| true)
                    .unwrap();
                let replayed = mtg_core::game::replay::played::verify(bytes).unwrap();
                assert_eq!(
                    replayed.life(),
                    if concede_stage.is_some() {
                        [20, 20]
                    } else {
                        [20, 18]
                    }
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
                if let Some(ref e) = expected_trajectory {
                    assert_eq!(&value, e);
                } else {
                    expected_trajectory = Some(value);
                }
            }
        }
    }
}
