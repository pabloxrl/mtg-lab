//! Normal ordered-deck reset, real casts through Driver/Run, CR 603/601.2i.
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
fn send(d: &mut Driver, actor: Seat, choices: Vec<C>, q: NonZeroUsize) {
    let dec = d.observe(actor).unwrap().decision.unwrap();
    let s = Submission {
        schema_version: 1,
        revision: dec.revision,
        generation: dec.generation,
        choices,
    };
    if dec.kind == "trigger_order" {
        let before = d.privileged_snapshot();
        let history = d.privileged_history().to_vec();
        let capture = serde_json::to_value(d.trajectory()).unwrap();
        let mut wrong = s.clone();
        wrong.choices.pop();
        assert!(d.submit(actor, &wrong).is_err());
        assert!(
            d.submit(
                if actor == Seat::P0 {
                    Seat::P1
                } else {
                    Seat::P0
                },
                &s
            )
            .is_err()
        );
        assert_eq!(before, d.privileged_snapshot());
        assert_eq!(history, d.privileged_history());
        assert_eq!(capture, serde_json::to_value(d.trajectory()).unwrap());
    }
    d.submit(actor, &s)
        .unwrap_or_else(|e| panic!("{dec:?} {s:?}: {e:?}"));
    if dec.kind == "trigger_order" {
        assert!(d.submit(actor, &s).is_err());
    }
    while d.advance(q).unwrap() == Progress::InternalYield {}
}
#[test]
fn cast_triggers_normal_reset_typed_capture_replay_quantum() {
    let mut history = None;
    let mut trajectory = None;
    for captured in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
            .unwrap();
            let mut order: Vec<String> = [
                "firebrand-archer",
                "crackling-cyclops",
                "firebrand-archer",
                "dragon-fodder",
            ]
            .into_iter()
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
                id: "20500000-1234-4234-8234-123456789abc".into(),
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
                d.reset_captured(&run.config, 205, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 205, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, vec![C::Keep], q);
            send(&mut d, Seat::P1, vec![C::Keep], q);
            let mut landed = 0;
            let mut cast_turn = 0;
            let mut ordered = false;
            let mut resolved = false;
            let mut expired = false;
            for _ in 0..2000 {
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
                let cyclops = bf.iter().find(|c| c.card == "crackling-cyclops");
                if ordered && t.0 == 9 {
                    if o.stack.len() == 3 {
                        assert_eq!(o.view.life, [20, 20]);
                        assert_eq!(cyclops.unwrap().creature.unwrap()[0], 3);
                    }
                    if o.stack.len() == 2 {
                        assert_eq!(o.view.life, [20, 19]);
                    }
                    if o.stack.len() == 1 {
                        assert_eq!(o.view.life, [20, 18]);
                        assert_eq!(bf.iter().filter(|c| c.card == "goblin-token").count(), 0);
                    }
                    if o.stack.is_empty() {
                        assert_eq!(o.view.life, [20, 18]);
                        assert_eq!(cyclops.unwrap().creature.unwrap()[0], 3);
                        assert_eq!(bf.iter().filter(|c| c.card == "goblin-token").count(), 2);
                        resolved = true;
                    }
                }
                if t.0 == 10 && t.2 == "upkeep" {
                    assert_eq!(cyclops.unwrap().creature.unwrap()[0], 0);
                    expired = true;
                    break;
                }
                let choices = match dec.kind {
                    "trigger_order" => {
                        assert_eq!(o.view.life, [20, 20]);
                        assert_eq!(cyclops.unwrap().creature.unwrap()[0], 0);
                        assert_eq!(dec.count, 3);
                        ordered = true;
                        let mut sources = o.pending_triggers.clone();
                        // Choose from source features, not knowledge of queue layout.
                        sources.sort_by_key(|p| {
                            (
                                p.ability.effect == "cyclops_boost",
                                std::cmp::Reverse(p.row),
                            )
                        });
                        sources
                            .iter()
                            .map(|p| C::OrderTrigger { trigger: p.row })
                            .collect()
                    }
                    "priority" if actor == Seat::P0 && t.1 == 0 && t.2 == "precombat_main" => {
                        if landed != t.0 {
                            landed = t.0;
                            vec![C::PlayLand {
                                card: r(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| c.card == "mountain")
                                        .unwrap(),
                                ),
                            }]
                        } else if cast_turn != t.0 && [3, 5, 7, 9].contains(&t.0) {
                            cast_turn = t.0;
                            let key = match t.0 {
                                3 | 7 => "firebrand-archer",
                                5 => "crackling-cyclops",
                                _ => "dragon-fodder",
                            };
                            vec![C::Cast {
                                card: r(
                                    VisibleZone::Hand,
                                    o.view.hand.iter().position(|c| c.card == key).unwrap(),
                                ),
                            }]
                        } else {
                            vec![C::Pass]
                        }
                    }
                    "payment" => {
                        let legal: Vec<_> = dec
                            .candidates
                            .iter()
                            .zip(&dec.legal_mask)
                            .filter_map(|(c, m)| m.then_some(c))
                            .collect();
                        vec![if legal.contains(&&C::FinishPayment) {
                            C::FinishPayment
                        } else if legal.contains(&&C::Pay { color: 3 }) {
                            C::Pay { color: 3 }
                        } else {
                            (*legal
                                .iter()
                                .find(|c| matches!(c, C::TapMana { .. }))
                                .unwrap())
                            .clone()
                        }]
                    }
                    "priority" => vec![C::Pass],
                    "attackers" | "blockers" | "combat_damage" => vec![C::FinishCombat],
                    "cleanup_discard" => vec![C::Discard {
                        card: r(
                            VisibleZone::Hand,
                            o.view
                                .hand
                                .iter()
                                .position(|c| c.card == "mountain")
                                .unwrap(),
                        ),
                    }],
                    _ => panic!("unexpected {}", dec.kind),
                };
                send(&mut d, actor, choices, q);
            }
            assert!(ordered && resolved && expired);
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
                registry.register("cast-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("cast-private", &result, |_, _| true)
                    .unwrap();
                assert_eq!(
                    mtg_core::game::replay::played::verify(bytes)
                        .unwrap()
                        .life(),
                    [20, 18]
                );
                let converted = from_core_v2(result.trajectory().unwrap()).unwrap();
                let ordering = converted
                    .decisions
                    .iter()
                    .position(|r| !r.observation.pending_triggers.is_empty())
                    .unwrap();
                assert_eq!(
                    converted.decisions[ordering].observation.pending_triggers[1]
                        .ability
                        .card,
                    "crackling-cyclops"
                );
                let mut corrupt = converted.clone();
                corrupt.decisions[ordering].observation.pending_triggers[1]
                    .ability
                    .controller = 1;
                assert!(
                    mtg_recorder::structured::validate(&corrupt).is_err(),
                    "source features must agree with the public object"
                );
                let mut corrupt = converted.clone();
                corrupt.decisions[ordering].observation.pending_triggers[1]
                    .ability
                    .source = corrupt.decisions[ordering].observation.pending_triggers[0]
                    .ability
                    .source;
                assert!(mtg_recorder::structured::validate(&corrupt).is_err());
                let mut corrupt = converted.clone();
                let trigger = corrupt
                    .decisions
                    .iter_mut()
                    .flat_map(|r| &mut r.observation.stack)
                    .find_map(|s| s.trigger.as_mut())
                    .unwrap();
                trigger.effect = "unimplemented".into();
                assert!(mtg_recorder::structured::validate(&corrupt).is_err());
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
