//! Normal ordered-deck reset, real casts through Driver/Run, CR 603.2/603.3d/113.7a.
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
    if matches!(dec.kind, "trigger_order" | "trigger_target") {
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
    if matches!(dec.kind, "trigger_order" | "trigger_target") {
        assert!(d.submit(actor, &s).is_err());
    }
    while d.advance(q).unwrap() == Progress::InternalYield {}
}
#[test]
fn pyromancer_normal_reset_typed_capture_replay_quantum() {
    let mut history = None;
    let mut trajectory = None;
    for captured in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let manifest: serde_json::Value = serde_json::from_str(include_str!(
                "../../../data/cards/foundations_micro_v1.json"
            ))
            .unwrap();
            let mut order: Vec<String> = ["viashino-pyromancer", "viashino-pyromancer"]
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
                id: "20600000-1234-4234-8234-123456789abc".into(),
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
                d.reset_captured(&run.config, 206, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 206, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            send(&mut d, Seat::P0, vec![C::Keep], q);
            send(&mut d, Seat::P1, vec![C::Keep], q);
            let mut landed = 0;
            let mut cast_turn = 0;
            let mut targets = 0;
            let mut complete = false;
            for _ in 0..1000 {
                let actor = if d.observe(Seat::P0).unwrap().view.acting_seat == Some(0) {
                    Seat::P0
                } else {
                    Seat::P1
                };
                let o = d.observe(actor).unwrap();
                let t = o.view.turn.unwrap();
                let dec = o.decision.as_ref().unwrap();
                if targets == 2 && o.stack.is_empty() && dec.kind == "priority" {
                    assert_eq!(o.view.life, [18, 18]);
                    complete = true;
                    break;
                }
                let choices = match dec.kind {
                    "trigger_order" => {
                        assert_eq!(o.view.life, if targets == 0 { [20, 20] } else { [18, 20] });
                        assert_eq!(o.pending_triggers[0].ability.effect, "pyromancer_damage");
                        dec.candidates.clone()
                    }
                    "trigger_target" => {
                        assert_eq!(
                            dec.candidates,
                            vec![C::TargetPlayer { seat: 0 }, C::TargetPlayer { seat: 1 }]
                        );
                        assert!(o.pending_triggers[0].selecting_target);
                        let seat = targets;
                        targets += 1;
                        vec![C::TargetPlayer { seat }]
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
                        } else if cast_turn != t.0 && [3, 5].contains(&t.0) {
                            cast_turn = t.0;
                            vec![C::Cast {
                                card: r(
                                    VisibleZone::Hand,
                                    o.view
                                        .hand
                                        .iter()
                                        .position(|c| c.card == "viashino-pyromancer")
                                        .unwrap(),
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
                    "cleanup_discard" => dec.candidates.iter().take(dec.count).cloned().collect(),
                    _ => panic!("unexpected {}", dec.kind),
                };
                send(&mut d, actor, choices, q);
            }
            assert!(complete);
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
                registry.register("etb-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("etb-private", &result, |_, _| true)
                    .unwrap();
                assert_eq!(
                    mtg_core::game::replay::played::verify(bytes)
                        .unwrap()
                        .life(),
                    [18, 18]
                );
                let converted = from_core_v2(result.trajectory().unwrap()).unwrap();
                assert_eq!(
                    converted
                        .decisions
                        .iter()
                        .filter(|r| r
                            .observation
                            .decision
                            .as_ref()
                            .is_some_and(|d| d.kind == "trigger_target"))
                        .count(),
                    2
                );
                let mut corrupt = converted.clone();
                let trigger = corrupt
                    .decisions
                    .iter_mut()
                    .flat_map(|r| &mut r.observation.stack)
                    .find_map(|s| s.trigger.as_mut())
                    .unwrap();
                trigger.target_player = None;
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
