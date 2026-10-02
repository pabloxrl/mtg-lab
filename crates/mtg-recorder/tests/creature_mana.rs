//! Real normal-reset Driver/Run, typed conversion, JSONL and replay. CR 601/605.
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
fn creature_mana_typed_roundtrip_and_quantum_equivalence() {
    let mut previous = None;
    for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
        let manifest: serde_json::Value = serde_json::from_str(include_str!(
            "../../../data/cards/foundations_micro_v1.json"
        ))
        .unwrap();
        let mut order: Vec<String> = [
            "forest",
            "llanowar-elves",
            "bear-cub",
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
            id: "19500000-1234-4234-8234-123456789abc".into(),
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
        d.reset_captured(&run.config, 195, 0, q, &run.header(0).unwrap())
            .unwrap();
        while d.advance(q).unwrap() == Progress::InternalYield {}
        send(&mut d, Seat::P0, C::Keep, q);
        send(&mut d, Seat::P1, C::Keep, q);
        let mut elf_cast = false;
        let mut cub_cast = false;
        let mut activated = false;
        let mut landed = false;
        for _ in 0..180 {
            let view = d.observe(Seat::P0).unwrap().view;
            let actor = if view.acting_seat == Some(0) {
                Seat::P0
            } else {
                Seat::P1
            };
            let o = d.observe(actor).unwrap();
            let decision = o.decision.unwrap();
            let turn = o.view.turn.unwrap().0;
            let choice = match decision.kind {
                "priority" if actor == Seat::P0 && o.view.turn.unwrap().2 == "precombat_main" => {
                    if !landed {
                        landed = true;
                        C::PlayLand {
                            card: reference(
                                VisibleZone::Hand,
                                o.view.hand.iter().position(|c| c.card == "forest").unwrap(),
                            ),
                        }
                    } else if !elf_cast {
                        elf_cast = true;
                        C::Cast {
                            card: reference(
                                VisibleZone::Hand,
                                o.view
                                    .hand
                                    .iter()
                                    .position(|c| c.card == "llanowar-elves")
                                    .unwrap(),
                            ),
                        }
                    } else if turn == 3 && !cub_cast {
                        cub_cast = true;
                        C::Cast {
                            card: reference(
                                VisibleZone::Hand,
                                o.view
                                    .hand
                                    .iter()
                                    .position(|c| c.card == "bear-cub")
                                    .unwrap(),
                            ),
                        }
                    } else if turn == 5 && !activated {
                        activated = true;
                        C::TapMana {
                            card: reference(VisibleZone::Battlefield, 1),
                        }
                    } else if activated {
                        break;
                    } else {
                        C::Pass
                    }
                }
                "priority" => C::Pass,
                "payment" => {
                    // Exact ordinal script within each cast; no automatic source choice.
                    let count = d
                        .trajectory()
                        .unwrap()
                        .decisions()
                        .iter()
                        .rev()
                        .take_while(|r| {
                            r.choice.status == mtg_core::trajectory::v2::ActionStatus::Continuing
                        })
                        .count();
                    if count == if cub_cast { 5 } else { 3 } {
                        C::FinishPayment
                    } else {
                        match count {
                            1 => C::TapMana {
                                card: reference(
                                    VisibleZone::Battlefield,
                                    if cub_cast { 1 } else { 0 },
                                ),
                            },
                            2 | 4 => C::Pay { color: 4 },
                            3 => C::TapMana {
                                card: reference(VisibleZone::Battlefield, 0),
                            },
                            _ => panic!("payment script"),
                        }
                    }
                }
                "attackers" | "blockers" | "combat_damage" => C::FinishCombat,
                "cleanup_discard" => {
                    let row = o.view.hand.iter().position(|c| c.card == "forest").unwrap();
                    assert_eq!(decision.count, 1);
                    C::Discard {
                        card: reference(VisibleZone::Hand, row),
                    }
                }
                _ => panic!("unexpected choice"),
            };
            send(&mut d, actor, choice, q);
        }
        assert!(activated && cub_cast);
        d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
        let result = d.finish().unwrap();
        let converted = from_core_v2(result.trajectory().unwrap()).unwrap();
        let mut w = Writer::new_v2(Vec::new(), 4_000_000, Backpressure::Block).unwrap();
        w.append_v2(&converted).unwrap();
        let bytes = w.finish().unwrap();
        let loaded = read_v2(bytes.as_slice(), 4_000_000).unwrap();
        assert_eq!(loaded, vec![converted.clone()]);
        assert_eq!(converted.footer.as_ref().unwrap().returns, [1, -1]);
        let value = serde_json::to_value(&converted).unwrap();
        if let Some(prior) = previous {
            assert_eq!(value, prior);
        }
        previous = Some(value);
    }
}
