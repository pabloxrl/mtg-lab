//! Real normal-reset turns/discards, no synthetic state injection. CR 103.8a/514.
use mtg_core::{
    episode::{Driver, Progress},
    game::{
        Config,
        policy::{Choice, Submission},
    },
    objects::Seat,
    trajectory::Limits,
};
use mtg_recorder::{Backpressure, Writer, collector::Run, from_core_v2, read_v2};
use std::num::NonZeroUsize;
#[test]
fn cleanup_normal_reset_both_seats_capture_replay_and_quantum() {
    let mut expected_history = None;
    let mut expected_trajectory = None;
    for capture in [false, true] {
        for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            let run = Run {
                id: "20700000-1234-4234-8234-123456789abc".into(),
                config: Config::default(),
                policies: ["script".into(), "script".into()],
                limits: Limits::default(),
                first_ordinal: 0,
                started: 1,
            };
            let mut d = Driver::new(256).unwrap();
            if capture {
                d.reset_captured(&run.config, 207, 0, q, &run.header(0).unwrap())
                    .unwrap();
            } else {
                d.reset(&run.config, 207, 0, q).unwrap();
            }
            while d.advance(q).unwrap() == Progress::InternalYield {}
            let mut discards = [0; 2];
            let mut finished = false;
            for _ in 0..200 {
                let actor = if d.observe(Seat::P0).unwrap().view.acting_seat == Some(0) {
                    Seat::P0
                } else {
                    Seat::P1
                };
                let o = d.observe(actor).unwrap();
                if o.view.turn.as_ref().is_some_and(|t| t.0 == 4) {
                    assert_eq!(o.view.turn.as_ref().unwrap().2, "upkeep");
                    assert_eq!(o.view.hand_counts, [7, 7]);
                    finished = true;
                    break;
                }
                let dec = o.decision.as_ref().unwrap();
                let choices = match dec.kind {
                    "keep_or_mulligan" => vec![Choice::Keep],
                    "priority" => vec![Choice::Pass],
                    "cleanup_discard" => {
                        assert_eq!(dec.count, 1);
                        let i = if actor == Seat::P0 { 0 } else { 1 };
                        discards[i] += 1;
                        assert_eq!(o.view.hand_counts[i], 8);
                        assert_eq!(o.view.turn.as_ref().unwrap().2, "cleanup");
                        assert!(
                            d.observe(if i == 0 { Seat::P1 } else { Seat::P0 })
                                .unwrap()
                                .decision
                                .is_none()
                        );
                        vec![dec.candidates[0].clone()]
                    }
                    _ => panic!("unexpected {}", dec.kind),
                };
                let sub = Submission {
                    schema_version: 1,
                    revision: dec.revision,
                    generation: dec.generation,
                    choices,
                };
                if dec.kind == "cleanup_discard" {
                    let state = d.privileged_snapshot();
                    let history = d.privileged_history().to_vec();
                    let rows = serde_json::to_value(d.trajectory()).unwrap();
                    let mut bad = sub.clone();
                    bad.choices.clear();
                    assert!(d.submit(actor, &bad).is_err());
                    assert!(
                        d.submit(
                            if actor == Seat::P0 {
                                Seat::P1
                            } else {
                                Seat::P0
                            },
                            &sub
                        )
                        .is_err()
                    );
                    assert_eq!(d.privileged_snapshot(), state);
                    assert_eq!(d.privileged_history(), history);
                    assert_eq!(serde_json::to_value(d.trajectory()).unwrap(), rows);
                }
                d.submit(actor, &sub).unwrap();
                while d.advance(q).unwrap() == Progress::InternalYield {}
            }
            assert!(finished);
            assert_eq!(discards, [1, 1]);
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
            let mut result = d.finish().unwrap();
            if let Some(h) = &expected_history {
                assert_eq!(result.privileged_history(), h);
            } else {
                expected_history = Some(result.privileged_history().to_vec());
            }
            if capture {
                let mut registry = mtg_core::episode::replay::Registry::default();
                registry.register("cleanup-private", &mut result).unwrap();
                let bytes = registry
                    .resolve("cleanup-private", &result, |_, _| true)
                    .unwrap();
                let replay = mtg_core::game::replay::played::verify(bytes).unwrap();
                assert_eq!(replay.turn_position().unwrap().0, 4);
                assert_eq!(replay.outcome().unwrap().winner, Some(Seat::P0));
                let converted = from_core_v2(result.trajectory().unwrap()).unwrap();
                assert_eq!(
                    converted
                        .decisions
                        .iter()
                        .filter(|r| r
                            .observation
                            .decision
                            .as_ref()
                            .is_some_and(|d| d.kind == "cleanup_discard"))
                        .count(),
                    2
                );
                let mut w = Writer::new_v2(Vec::new(), 4_000_000, Backpressure::Block).unwrap();
                w.append_v2(&converted).unwrap();
                assert_eq!(
                    read_v2(w.finish().unwrap().as_slice(), 4_000_000).unwrap(),
                    vec![converted.clone()]
                );
                let value = serde_json::to_value(converted).unwrap();
                if let Some(e) = &expected_trajectory {
                    assert_eq!(&value, e);
                } else {
                    expected_trajectory = Some(value);
                }
            }
        }
    }
}
