//! Compatibility at the core boundary: CR 103.5 Keep, then CR 104.3a
//! concession. Expected actions/rewards and optional statistics are literal;
//! serialized core data is additionally the pre-refactor compatibility oracle.
use mtg_core::{
    episode::Driver,
    game::{
        Config,
        policy::{Choice, Submission},
    },
    objects::Seat,
    trajectory::{Limits, PolicyInfo},
};
use mtg_recorder::{Backpressure, Error, Writer, collector::Run, from_core_v2, read_v2};
use std::num::NonZeroUsize;

#[test]
fn played_conversion_preserves_bytes_statistics_and_incomplete_error() {
    for value in [
        None,
        Some(-0.0),
        Some(f64::from_bits(1)),
        Some(f64::from_bits(2377159206977889939)),
        Some(f64::MAX),
    ] {
        let run = Run {
            id: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
            config: Config::default(),
            policies: ["script".into(), "script".into()],
            limits: Limits::default(),
            first_ordinal: 0,
            started: 1,
        };
        let mut driver = Driver::new(256).unwrap();
        driver
            .reset_captured(
                &run.config,
                133,
                0,
                NonZeroUsize::MAX,
                &run.header(0).unwrap(),
            )
            .unwrap();
        for seat in [Seat::P0, Seat::P1] {
            let d = driver.observe(seat).unwrap().decision.unwrap();
            driver
                .submit_with_policy(
                    seat,
                    &Submission {
                        revision: d.revision,
                        schema_version: 1,
                        generation: d.generation,
                        choices: vec![Choice::Keep],
                    },
                    &PolicyInfo {
                        value,
                        log_probability: value.map(|_| -0.5),
                        ..PolicyInfo::default()
                    },
                )
                .unwrap();
        }
        driver.advance(NonZeroUsize::MAX).unwrap();
        driver
            .concede(Seat::P1, driver.episode_id().unwrap())
            .unwrap();
        let result = driver.finish().unwrap();
        let original = result.trajectory().unwrap();
        let converted = from_core_v2(original).unwrap();
        let legacy = serde_json::from_value(serde_json::to_value(original).unwrap()).unwrap();
        let encode = |e| {
            let mut w = Writer::new_v2(Vec::new(), 200_000, Backpressure::Block).unwrap();
            w.append_v2(e).unwrap();
            w.finish().unwrap()
        };
        let bytes = encode(&converted);
        assert_eq!(bytes, encode(&legacy));
        let loaded = read_v2(bytes.as_slice(), 200_000).unwrap();
        assert_eq!(loaded[0].footer.as_ref().unwrap().returns, [1, -1]);
        for row in &loaded[0].decisions {
            assert_eq!(
                row.choice.submission.choices,
                [mtg_recorder::structured::Command::Keep]
            );
            assert_eq!(
                row.choice.policy.value.map(f64::to_bits),
                value.map(f64::to_bits)
            );
            assert_eq!(row.choice.policy.log_probability, value.map(|_| -0.5));
            assert_eq!(row.choice.policy.checkpoint, None);
        }
        let mut interrupted = Driver::new(256).unwrap();
        interrupted
            .reset_captured(
                &run.config,
                133,
                0,
                NonZeroUsize::MAX,
                &run.header(0).unwrap(),
            )
            .unwrap();
        let result = interrupted.finish().unwrap();
        assert!(matches!(
            from_core_v2(result.trajectory().unwrap()),
            Err(Error::Incomplete)
        ));
    }
}

#[test]
fn legacy_conversion_preserves_statistics_truncations_and_quarantine() {
    use mtg_core::{game::Game, trajectory as t};
    use mtg_recorder::{from_core, read, schema};
    let run = Run {
        id: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
        config: Config::default(),
        policies: ["script".into(), "script".into()],
        limits: Limits {
            decisions: Some(1),
            turns: Some(2),
            wall_time_ms: Some(3),
        },
        first_ordinal: 0,
        started: 1,
    };
    for (limit, expected) in [
        (t::Limit::Decisions, schema::Limit::Decisions),
        (t::Limit::Turns, schema::Limit::Turns),
        (t::Limit::WallTime, schema::Limit::WallTime),
    ] {
        let mut game = Game::new().unwrap();
        game.reset(&run.config, 133, 0).unwrap();
        let before = t::Frame::capture(&game).unwrap();
        let mut header = run.header(0).unwrap();
        header.versions.schema = 1;
        header.restricted_replay = Some("opaque-replay".into());
        let mut recorder = t::Recorder::new(&header, &before).unwrap();
        assert!(matches!(
            from_core(recorder.episode()),
            Err(Error::Incomplete)
        ));
        let opening = game.observe(Seat::P0).unwrap().opening.unwrap();
        game.apply_opening_view(Seat::P0, opening.generation, &[0])
            .unwrap();
        let after = t::Frame::capture(&game).unwrap();
        recorder
            .append(
                &before,
                &t::Choice {
                    kind: opening.kind.into(),
                    logical_action: 0,
                    micro_choice: 0,
                    candidates: opening
                        .candidates
                        .iter()
                        .enumerate()
                        .map(|(i, name)| t::Candidate {
                            semantic: name.to_string(),
                            features: vec![i as i64],
                        })
                        .collect(),
                    legal_mask: vec![true; opening.candidates.len()],
                    selected: 0,
                    policy: PolicyInfo {
                        checkpoint: Some("checkpoint".into()),
                        log_probability: Some(-0.0),
                        value: Some(f64::from_bits(1)),
                        recurrent_state: Some("state".into()),
                        exploration: Some("settings".into()),
                    },
                },
                &after,
            )
            .unwrap();
        recorder.finish(&after, t::End::Truncated(limit)).unwrap();
        let converted = from_core(recorder.episode()).unwrap();
        let legacy =
            serde_json::from_value(serde_json::to_value(recorder.episode()).unwrap()).unwrap();
        let encode = |e| {
            let mut writer = Writer::new(Vec::new(), 200_000, Backpressure::Block).unwrap();
            writer.append(e).unwrap();
            writer.finish().unwrap()
        };
        let bytes = encode(&converted);
        assert_eq!(bytes, encode(&legacy));
        let loaded = read(bytes.as_slice(), 200_000).unwrap();
        let row = &loaded[0].decisions[0];
        assert_eq!(row.choice.policy.value.unwrap().to_bits(), 1);
        assert_eq!(
            row.choice.policy.log_probability.unwrap().to_bits(),
            (-0.0f64).to_bits()
        );
        assert_eq!(row.choice.policy.checkpoint.as_deref(), Some("checkpoint"));
        assert_eq!(row.choice.policy.recurrent_state.as_deref(), Some("state"));
        assert_eq!(row.choice.policy.exploration.as_deref(), Some("settings"));
        assert_eq!(
            loaded[0].footer.as_ref().unwrap().end,
            schema::End::Truncated(expected)
        );
        assert_eq!(loaded[0].footer.as_ref().unwrap().returns, [0, 0]);
        let mut failed = t::Recorder::new(&header, &before).unwrap();
        failed
            .finish(&before, t::End::Failed("capture failure".into()))
            .unwrap();
        assert!(matches!(
            from_core(failed.episode()),
            Err(Error::Incomplete)
        ));
    }
}

#[test]
fn played_fodder_tokens_survive_typed_conversion_and_jsonl_roundtrip() {
    use mtg_core::game::policy::VisibleZone;
    let run = Run {
        id: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
        config: Config::default(),
        policies: ["fodder-only".into(), "fodder-only".into()],
        limits: Limits::default(),
        first_ordinal: 0,
        started: 1,
    };
    let mut d = Driver::new(256).unwrap();
    d.reset_captured(
        &run.config,
        194,
        0,
        NonZeroUsize::MIN,
        &run.header(0).unwrap(),
    )
    .unwrap();
    let mut seen = false;
    for _ in 0..800 {
        while d.advance(NonZeroUsize::MIN).unwrap() == mtg_core::episode::Progress::InternalYield {}
        let v = d.observe(Seat::P0).unwrap().view;
        let token_count = v
            .public_zones
            .iter()
            .flat_map(|z| &z.cards)
            .filter(|c| c.card == "goblin-token")
            .count();
        if token_count == 2 {
            seen = true;
            break;
        }
        assert_eq!(token_count, 0);
        let seat = if v.acting_seat == Some(0) {
            Seat::P0
        } else {
            Seat::P1
        };
        let o = d.observe(seat).unwrap();
        let decision = o.decision.unwrap();
        let legal: Vec<_> = decision
            .candidates
            .iter()
            .zip(&decision.legal_mask)
            .filter(|(_, mask)| **mask)
            .map(|(c, _)| c.clone())
            .collect();
        let choices = match decision.kind {
            "keep_or_mulligan" => vec![Choice::Keep],
            "priority" => {
                let selected=legal.iter().find(|c|matches!(c,Choice::PlayLand{..}))
                    .or_else(||legal.iter().find(|c|matches!(c,Choice::Cast{card} if o.view.hand[card.row].card=="dragon-fodder")))
                    .cloned().unwrap_or(Choice::Pass);
                vec![selected]
            }
            "payment" => {
                let c = legal
                    .iter()
                    .find(|c| matches!(c, Choice::FinishPayment))
                    .or_else(|| legal.iter().find(|c| matches!(c, Choice::Pay { .. })))
                    .or_else(|| legal.iter().find(|c| matches!(c, Choice::TapMana { .. })))
                    .unwrap()
                    .clone();
                vec![c]
            }
            "attackers" | "blockers" | "combat_damage" => vec![Choice::FinishCombat],
            "cleanup_discard" => o
                .view
                .hand
                .iter()
                .enumerate()
                .take(decision.count)
                .map(|(row, _)| Choice::Discard {
                    card: mtg_core::game::policy::VisibleRef {
                        zone: VisibleZone::Hand,
                        row,
                    },
                })
                .collect(),
            k => panic!("unscripted {k}"),
        };
        d.submit(
            seat,
            &Submission {
                schema_version: 1,
                revision: decision.revision,
                generation: decision.generation,
                choices,
            },
        )
        .unwrap();
    }
    assert!(
        seen,
        "seed 194 must actually resolve Fodder within the bound"
    );
    d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
    let result = d.finish().unwrap();
    let original = result.trajectory().unwrap();
    let converted = from_core_v2(original).unwrap();
    // Wire serializers omit absent optionals; compare against the former
    // serde boundary in its wire type, preserving that documented normalization.
    let legacy: mtg_recorder::structured::Episode =
        serde_json::from_value(serde_json::to_value(original).unwrap()).unwrap();
    assert_eq!(converted, legacy);
    let mut w = Writer::new_v2(Vec::new(), 2_000_000, Backpressure::Block).unwrap();
    w.append_v2(&converted).unwrap();
    let bytes = w.finish().unwrap();
    let loaded = read_v2(bytes.as_slice(), 2_000_000).unwrap();
    assert_eq!(loaded[0], converted);
    let footer = original.footer().unwrap();
    assert_eq!(footer.returns, [1, -1]);
    for observation in &footer.final_observations {
        let tokens: Vec<_> = observation
            .view
            .public_zones
            .iter()
            .flat_map(|z| &z.cards)
            .filter(|c| c.card == "goblin-token")
            .collect();
        assert_eq!(tokens.len(), 2);
        for token in tokens {
            assert_eq!(token.creature, Some([1, 1, 0]));
            assert!(token.summoning_sick);
            assert!(!token.tapped);
        }
    }
}
