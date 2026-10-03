//! GH-198 normal-reset Tajuru, CR 601/510/702.19/702.20. Pinned 4G 5/4.
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
fn trample_normal_reset_two_cubs_capture_replay() {
    let mut expected_history = None;
    let mut expected_trajectory = None;
    for captured in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
            .unwrap();
            let mut order: Vec<String> = [
                "tajuru-pathwarden",
                "bear-cub",
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
                id: "19800000-1234-4234-8234-123456789abc".into(),
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
                d.reset_captured(&run.config, 198, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 198, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, C::Keep, q);
            send(&mut d, Seat::P1, C::Keep, q);
            let mut landed = [0, 0];
            let mut cast = [0, 0];
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
                if t.0 == 11 && t.2 == "combat_damage" && dec.kind == "priority" {
                    assert!(
                        !bf.iter()
                            .any(|c| c.card == "tajuru-pathwarden" || c.card == "bear-cub")
                    );
                    assert_eq!(o.view.life, [20, 19]);
                    checked = true;
                    break;
                }
                let choice = match dec.kind {
                    "priority" if t.2 == "precombat_main" && t.1 == seat as u8 => {
                        if landed[seat] != t.0
                            && ((seat == 0 && t.0 <= 9) || (seat == 1 && t.0 <= 4))
                        {
                            landed[seat] = t.0;
                            C::PlayLand {
                                card: reference(
                                    VisibleZone::Hand,
                                    o.view.hand.iter().position(|c| c.card == "forest").unwrap(),
                                ),
                            }
                        } else if cast[seat] < if seat == 0 { 1 } else { 2 }
                            && t.0 == if seat == 0 { 9 } else { 4 + 2 * cast[seat] }
                        {
                            cast[seat] += 1;
                            C::Cast {
                                card: reference(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| {
                                            c.card
                                                == if seat == 0 {
                                                    "tajuru-pathwarden"
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
                    "attackers" if t.0 == 11 && !attacked => {
                        attacked = true;
                        C::SelectAttackers {
                            cards: vec![reference(
                                VisibleZone::Battlefield,
                                bf.iter()
                                    .position(|c| c.card == "tajuru-pathwarden")
                                    .unwrap(),
                            )],
                        }
                    }
                    "blockers" if t.0 == 11 && !blocked => {
                        blocked = true;
                        C::SelectBlockers {
                            blocks: bf
                                .iter()
                                .enumerate()
                                .filter(|(_, c)| c.card == "bear-cub")
                                .map(|(i, _)| {
                                    (
                                        reference(VisibleZone::Battlefield, i),
                                        reference(
                                            VisibleZone::Battlefield,
                                            bf.iter()
                                                .position(|c| c.card == "tajuru-pathwarden")
                                                .unwrap(),
                                        ),
                                    )
                                })
                                .collect(),
                        }
                    }
                    "combat_damage"
                        if t.0 == 11
                            && dec.factored.as_ref().unwrap().damage[0].amounts.is_none() =>
                    {
                        let a = &dec.factored.as_ref().unwrap().damage[0];
                        assert_eq!(a.power, 5);
                        assert_eq!(a.trample_lethal, Some(vec![2, 2]));
                        assert!(!bf[a.attacker.row].tapped);
                        // Illegal trample must preserve state, RNG, history and captured transitions.
                        let before = d.privileged_snapshot();
                        let history = d.privileged_history().to_vec();
                        let captured_before = serde_json::to_value(d.trajectory()).unwrap();
                        let request = Submission {
                            schema_version: 1,
                            revision: dec.revision,
                            generation: dec.generation,
                            choices: vec![C::AssignDamage {
                                attacker: a.attacker,
                                amounts: vec![(a.blockers[0], 1)],
                            }],
                        };
                        assert!(d.submit(actor, &request).is_err());
                        assert_eq!(d.privileged_snapshot(), before);
                        assert_eq!(d.privileged_history(), history);
                        assert_eq!(
                            serde_json::to_value(d.trajectory()).unwrap(),
                            captured_before
                        );
                        C::AssignDamage {
                            attacker: a.attacker,
                            amounts: a.blockers.iter().map(|b| (*b, 2)).collect(),
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
            assert!(checked && attacked && blocked && cast == [1, 2]);
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
                registry.register("tajuru-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("tajuru-private", &result, |_, _| true)
                    .unwrap();
                let replayed = mtg_core::game::replay::played::verify(bytes).unwrap();
                assert_eq!(replayed.life(), [20, 19]);
                let converted = from_core_v2(result.trajectory().unwrap()).unwrap();
                // Durable validators must reject insufficient lethal, excessive totals,
                // duplicate recipients and malformed trample domains independently.
                for mutation in 0..4 {
                    let mut bad = converted.clone();
                    let row = bad
                        .decisions
                        .iter_mut()
                        .find(|r| {
                            matches!(
                                r.choice.submission.choices[0],
                                mtg_recorder::structured::Command::AssignDamage { .. }
                            )
                        })
                        .unwrap();
                    if mutation == 3 {
                        row.observation
                            .decision
                            .as_mut()
                            .unwrap()
                            .factored
                            .as_mut()
                            .unwrap()
                            .damage[0]
                            .trample_lethal = Some(vec![2]);
                    } else if let mtg_recorder::structured::Command::AssignDamage {
                        amounts, ..
                    } = &mut row.choice.submission.choices[0]
                    {
                        match mutation {
                            0 => amounts[0].1 = 1,
                            1 => amounts[0].1 = 4,
                            _ => amounts.push(amounts[0]),
                        }
                    }
                    assert!(mtg_recorder::structured::validate(&bad).is_err());
                }

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
