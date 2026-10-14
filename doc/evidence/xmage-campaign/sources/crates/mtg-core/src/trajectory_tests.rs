//! Authored ledger from RFC 0002 B036/B037, DRL-001/004/006/007/008.
//! Synthetic observation markers do not claim reachable Magic positions.
use super::*;
use crate::opening::{Config, views::TerminalView};
fn frame() -> Frame {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 0).unwrap();
    Frame::capture(&g).unwrap()
}
fn header() -> Header {
    Header {
        id: EpisodeKey {
            run: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
            ordinal: 7,
        },
        versions: Versions {
            schema: 1,
            engine: "engine-v1".into(),
            rules: "cr-20260925".into(),
            cards: "pool-v1".into(),
            action: "test-ledger-v1".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["script-a-v1".into(), "script-b-v1".into()],
        starting_seat: 0,
        limits: Limits::default(),
        restricted_replay: Some("opaque-replay-id".into()),
    }
}
fn marked(base: &Frame, index: usize, actor: Option<u8>, terminal: bool) -> Frame {
    let mut f = base.clone();
    for (s, v) in f.views.iter_mut().enumerate() {
        v.life = [20 + index as i64, 20];
        v.acting_seat = actor;
        v.opening = None;
        v.hand[0].card = if s == 0 {
            "forest"
        } else {
            "opponent-private-sentinel"
        };
        v.terminal = terminal.then_some(TerminalView {
            winner: Some(0),
            losses: [None, Some("life")],
        });
    }
    f
}
fn choice(index: usize, logical: u64, micro: u64) -> Choice {
    Choice {
        kind: "scripted".into(),
        logical_action: logical,
        micro_choice: micro,
        candidates: vec![
            Candidate {
                semantic: format!("legal-{index}"),
                features: vec![index as i64, 9],
            },
            Candidate {
                semantic: "illegal".into(),
                features: vec![-1, 4],
            },
        ],
        legal_mask: vec![true, false],
        selected: 0,
        policy: PolicyInfo::default(),
    }
}
#[test]
fn trajectory_handwritten_links_rewards_and_microchoice_durations() {
    // Logical actions: decisions 0,1,2 form action 0; 3/4/5 each a new action.
    // Half-open intervals include source decision, exclude destination/final event.
    let base = frame();
    let actors = [0, 0, 1, 1, 0, 1];
    let frames: Vec<_> = actors
        .iter()
        .enumerate()
        .map(|(i, &a)| marked(&base, i, Some(a), false))
        .chain([marked(&base, 6, None, true)])
        .collect();
    let mut r = Recorder::new(&header(), &frames[0]).unwrap();
    for i in 0..6 {
        r.append(
            &frames[i],
            &choice(
                i,
                if i < 3 { 0 } else { (i - 2) as u64 },
                if i < 3 { i as u64 } else { 0 },
            ),
            &frames[i + 1],
        )
        .unwrap();
    }
    assert_eq!(
        r.episode().decisions().len(),
        6,
        "every selected decision must be retained"
    );
    for d in &r.episode().decisions()[..5] {
        assert_eq!(d.reward, [0, 0]);
        assert!(!d.terminated);
    }
    assert_eq!(r.episode().decisions()[5].reward, [1, -1]);
    for (seat, links, durations, actions, rewards) in [
        (
            Seat::P0,
            vec![Some(1), Some(4), None],
            vec![1, 3, 2],
            vec![1, 2, 2],
            vec![0, 0, 1],
        ),
        (
            Seat::P1,
            vec![Some(3), Some(5), None],
            vec![1, 2, 1],
            vec![1, 2, 1],
            vec![0, 0, -1],
        ),
    ] {
        let seq = r.episode().seat(seat).unwrap();
        assert_eq!(
            seq.transitions
                .iter()
                .map(|t| t.next_decision)
                .collect::<Vec<_>>(),
            links
        );
        assert_eq!(
            seq.transitions
                .iter()
                .map(|t| t.decisions_elapsed)
                .collect::<Vec<_>>(),
            durations
        );
        assert_eq!(
            seq.transitions
                .iter()
                .map(|t| t.logical_actions_elapsed)
                .collect::<Vec<_>>(),
            actions
        );
        assert_eq!(
            seq.transitions.iter().map(|t| t.reward).collect::<Vec<_>>(),
            rewards
        );
        assert_eq!(seq.unassigned_reward, 0);
        for (i, t) in seq.transitions.iter().enumerate() {
            assert_eq!(t.seat_index, i);
            assert_eq!(t.next_observation.seat, seq.seat);
            assert_eq!(
                t.next_observation.life[0],
                20 + t.next_decision.unwrap_or(6) as i64
            );
        }
        assert_eq!(
            r.episode().seat(seat).unwrap(),
            seq,
            "reads never consume or duplicate rewards"
        );
    }
    let f = r.episode().footer().unwrap();
    assert_eq!(
        (f.decisions, f.logical_actions, f.returns, f.boundary_reward),
        (6, 4, [1, -1], [0, 0])
    );
    assert_eq!(
        r.finish(&frames[6], End::Completed),
        Err(Error::AlreadyEnded)
    );
    assert_eq!(
        r.append(&frames[5], &choice(5, 3, 0), &frames[6]),
        Err(Error::AlreadyEnded)
    );
}
#[test]
fn trajectory_zero_decision_seat_final_credit_and_owned_reset() {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 0).unwrap();
    let mut before = Frame::capture(&g).unwrap();
    let mut h = header();
    let mut c = choice(0, 0, 0);
    let mut r = Recorder::new(&h, &before).unwrap();
    g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
    let final_frame = Frame::capture(&g).unwrap();
    r.append(&before, &c, &final_frame).unwrap();
    assert!(
        r.episode().footer().is_some(),
        "terminal action must close the episode"
    );
    let saved = r.episode().clone();
    c.candidates[0].features.fill(999);
    c.legal_mask.fill(false);
    h.policies[0].clear();
    before.views[0].hand.clear();
    g.reset(&Config::default(), 99, 1).unwrap();
    assert_eq!(r.episode(), &saved);
    let seq = r.episode().seat(Seat::P1).unwrap();
    assert!(seq.transitions.is_empty());
    assert_eq!(seq.unassigned_reward, -1);
    assert_eq!(seq.total_return, -1);
    assert_eq!(
        seq.final_observation.terminal.as_ref().unwrap().winner,
        Some(0)
    );
    assert_ne!(seq.final_observation, g.observe(Seat::P1).unwrap());
    assert_eq!(
        r.episode().require_probabilities(),
        Err(Error::MissingProbability)
    );
}
#[test]
fn trajectory_truncation_failure_and_incomplete_quarantine() {
    for limit in [Limit::Decisions, Limit::Turns, Limit::WallTime] {
        let f = frame();
        let mut r = Recorder::new(&header(), &f).unwrap();
        assert_eq!(r.episode().seat(Seat::P0), Err(Error::Quarantined));
        r.finish(&f, End::Truncated(limit)).unwrap();
        assert!(
            r.episode().footer().is_some(),
            "external stop must retain a distinct footer"
        );
        let e = r.episode().footer().unwrap();
        assert_eq!(e.returns, [0, 0]);
        assert_eq!(e.end, End::Truncated(limit));
        assert!(e.complete);
        assert_eq!(e.final_observations[0].terminal, None);
    }
    let f = frame();
    let mut r = Recorder::new(&header(), &f).unwrap();
    r.finish(&f, End::Failed("encoding_overflow".into()))
        .unwrap();
    assert_eq!(r.episode().seat(Seat::P0), Err(Error::Quarantined));
    assert!(!r.episode().footer().unwrap().complete);
}
#[test]
fn trajectory_reject_invalid_choice_continuity_versions_and_false_terminal() {
    let f = frame();
    let c = choice(0, 0, 0);
    let mut r = Recorder::new(&header(), &f).unwrap();
    let mut bad = c.clone();
    bad.selected = 1;
    assert_eq!(r.append(&f, &bad, &f), Err(Error::InvalidChoice));
    assert!(r.episode().decisions().is_empty());
    assert_eq!(r.finish(&f, End::Completed), Err(Error::InvalidEnd));
    let mut h = header();
    h.versions.schema = 99;
    assert_eq!(Recorder::new(&h, &f), Err(Error::InvalidHeader));
    let foreign = frame();
    assert_eq!(r.append(&f, &c, &foreign), Err(Error::Discontinuity));
}

#[test]
fn trajectory_both_seats_external_concession_and_empty_episode() {
    for winner in [0, 1] {
        for decisions in [0, 1] {
            let mut g = Game::new().unwrap();
            g.reset(&Config::default(), 42, 0).unwrap();
            let f = Frame::capture(&g).unwrap();
            let mut r = Recorder::new(&header(), &f).unwrap();
            if decisions == 1 {
                let v = g.observe(Seat::P0).unwrap().opening.unwrap();
                g.apply_opening_view(Seat::P0, v.generation, &[0]).unwrap();
                r.append(&f, &choice(0, 0, 0), &Frame::capture(&g).unwrap())
                    .unwrap();
            }
            let loser = if winner == 0 { Seat::P1 } else { Seat::P0 };
            g.concede(loser, g.episode_id().unwrap()).unwrap();
            r.finish(&Frame::capture(&g).unwrap(), End::Completed)
                .unwrap();
            let expected = if winner == 0 { [1, -1] } else { [-1, 1] };
            assert_eq!(r.episode().footer().unwrap().boundary_reward, expected);
            for seat in [Seat::P0, Seat::P1] {
                let s = r.episode().seat(seat).unwrap();
                assert_eq!(
                    s.transitions.iter().map(|t| t.reward as i32).sum::<i32>()
                        + s.unassigned_reward as i32,
                    expected[seat_number(seat)] as i32
                );
                assert_eq!(s.total_return, expected[seat_number(seat)]);
                assert_eq!(s.final_observation.seat, seat_number(seat) as u8);
            }
        }
    }
}
#[test]
fn trajectory_header_and_choice_validation_is_transactional() {
    let f = frame();
    let h = header();
    let mut invalid = vec![];
    for field in 0..9 {
        let mut x = h.clone();
        match field {
            0 => x.id.run = "not-unique".into(),
            1 => x.versions.engine.clear(),
            2 => x.versions.rules.clear(),
            3 => x.versions.cards.clear(),
            4 => x.versions.action.clear(),
            5 => x.versions.observation = 999,
            6 => x.deck_hashes[0].clear(),
            7 => x.config_hash.clear(),
            _ => x.policies[1].clear(),
        }
        invalid.push(x);
    }
    for x in invalid {
        assert_eq!(Recorder::new(&x, &f), Err(Error::InvalidHeader));
    }
    let mut r = Recorder::new(&h, &f).unwrap();
    let before = r.clone();
    for field in 0..9 {
        let mut c = choice(0, 0, 0);
        match field {
            0 => c.legal_mask.clear(),
            1 => c.selected = 20,
            2 => c.legal_mask.fill(false),
            3 => c.candidates.clear(),
            4 => c.logical_action = 2,
            5 => c.micro_choice = 1,
            6 => c.policy.log_probability = Some(f64::NAN),
            7 => c.policy.value = Some(f64::INFINITY),
            _ => c.candidates[0].semantic.clear(),
        }
        assert_eq!(r.append(&f, &c, &f), Err(Error::InvalidChoice));
        assert_eq!(r, before);
    }
    let mut stale = f.clone();
    stale.views[0].life[0] -= 1;
    assert_eq!(
        r.append(&stale, &choice(0, 0, 0), &f),
        Err(Error::Discontinuity)
    );
    assert_eq!(r, before);
}
#[test]
fn trajectory_action_time_data_optional_statistics_and_privacy() {
    let base = frame();
    let first = marked(&base, 0, Some(0), false);
    let second = marked(&base, 1, Some(1), false);
    let last = marked(&base, 2, None, true);
    let mut r = Recorder::new(&header(), &first).unwrap();
    let mut c = choice(0, 0, 0);
    c.policy = PolicyInfo {
        checkpoint: Some("policy-v8".into()),
        log_probability: Some(-0.25),
        value: Some(0.75),
        recurrent_state: Some("seat0-state0".into()),
        exploration: Some("temperature=1".into()),
    };
    r.append(&first, &c, &second).unwrap();
    c.candidates.reverse();
    c.legal_mask.reverse();
    c.selected = 1;
    c.logical_action = 1;
    c.policy.log_probability = Some(-0.5);
    r.append(&second, &c, &last).unwrap();
    assert_eq!(r.episode().require_probabilities(), Ok(()));
    let s = r.episode().seat(Seat::P0).unwrap();
    let t = &s.transitions[0];
    assert_eq!(t.choice.candidates[0].features, vec![0, 9]);
    assert_eq!(t.choice.legal_mask, vec![true, false]);
    assert_eq!(t.choice.policy.log_probability, Some(-0.25));
    assert_eq!(t.choice.policy.value, Some(0.75));
    let encoded = serde_json::to_string(&s).unwrap();
    assert!(
        !encoded.contains("opponent-private-sentinel"),
        "P1 sentinel cannot enter P0 sequence"
    );
    assert!(!encoded.contains("opaque-replay-id"));
    assert!(!encoded.contains("script-b-v1"));
    assert_eq!(s.transitions[0].next_observation.hand[0].card, "forest");
}
#[test]
fn trajectory_truncated_transition_bootstrap_and_failed_completed_recording() {
    let f = frame();
    let next = marked(&f, 1, Some(0), false);
    let mut r = Recorder::new(&header(), &f).unwrap();
    r.append(&f, &choice(0, 0, 0), &next).unwrap();
    r.finish(&next, End::Truncated(Limit::Decisions)).unwrap();
    let seq = r.episode().seat(Seat::P0).unwrap();
    let t = &seq.transitions[0];
    assert!(t.truncated);
    assert!(!t.terminated);
    assert_eq!(t.next_observation, next.views[0]);
    let value = 0.75;
    let target = t.reward as f64 + if t.terminated { 0.0 } else { value };
    assert_eq!(target, 0.75);
    let final_frame = marked(&f, 2, None, true);
    let mut failed = Recorder::new(&header(), &f).unwrap();
    failed
        .finish(&final_frame, End::Failed("sink_rejected_record".into()))
        .unwrap();
    assert_eq!(
        failed.episode().footer().unwrap().final_observations[0]
            .terminal
            .as_ref()
            .unwrap()
            .winner,
        Some(0)
    );
    assert_eq!(failed.episode().seat(Seat::P0), Err(Error::Quarantined));
}
#[test]
fn trajectory_normal_opening_capture_on_off_and_reset_rejection() {
    use crate::opening::turns::{TurnAction, TurnSelection};
    let mut captured = Game::new().unwrap();
    let mut plain = Game::new().unwrap();
    captured.reset(&Config::default(), 773, 21).unwrap();
    plain.reset(&Config::default(), 773, 21).unwrap();
    let mut previous = Frame::capture(&captured).unwrap();
    let mut r = Recorder::new(&header(), &previous).unwrap();
    for i in 0..2 {
        let d = captured.decision().unwrap();
        let seat = d.actor;
        let view = captured.observe(seat).unwrap();
        let opening = view.opening.unwrap();
        let c = Choice {
            kind: opening.kind.into(),
            candidates: opening
                .candidates
                .iter()
                .map(|s| Candidate {
                    semantic: s.to_string(),
                    features: vec![if *s == "keep" { 0 } else { 1 }],
                })
                .collect(),
            legal_mask: vec![true; opening.candidates.len()],
            ..choice(i, i as u64, 0)
        };
        captured
            .apply_opening_view(seat, opening.generation, &[0])
            .unwrap();
        let p = plain.observe(seat).unwrap().opening.unwrap();
        plain.apply_opening_view(seat, p.generation, &[0]).unwrap();
        if i == 1 {
            captured.start_turns().unwrap();
            plain.start_turns().unwrap();
        }
        let after = Frame::capture(&captured).unwrap();
        r.append(&previous, &c, &after).unwrap();
        previous = after;
    }
    for i in 2..6 {
        for g in [&mut captured, &mut plain] {
            let d = g.turn_decision().unwrap();
            g.apply_turn(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Pass(d.candidate(0)),
                },
            )
            .unwrap();
        }
        let after = Frame::capture(&captured).unwrap();
        let mut c = choice(i, i as u64, 0);
        c.kind = "priority".into();
        c.candidates = vec![Candidate {
            semantic: "pass".into(),
            features: vec![0],
        }];
        c.legal_mask = vec![true];
        r.append(&previous, &c, &after).unwrap();
        previous = after;
    }
    for s in [Seat::P0, Seat::P1] {
        assert_eq!(captured.observe(s).unwrap(), plain.observe(s).unwrap());
    }
    // Separate Game allocations necessarily have distinct capability scopes.
    // Compare every semantic state field, including RNG, not namespace/checksum.
    fn semantic(g: &Game) -> serde_json::Value {
        let envelope: serde_json::Value = serde_json::from_slice(&g.snapshot()).unwrap();
        let mut payload: serde_json::Value =
            serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
        fn scopes(v: &mut serde_json::Value) {
            match v {
                serde_json::Value::Object(o) => {
                    for (k, v) in o {
                        if k == "store" || k == "scope" {
                            *v = 0.into();
                        } else {
                            scopes(v);
                        }
                    }
                }
                serde_json::Value::Array(a) => {
                    for v in a {
                        scopes(v);
                    }
                }
                _ => (),
            }
        }
        scopes(&mut payload);
        payload["objects"]["id"] = 0.into();
        payload
    }
    assert_eq!(semantic(&captured), semantic(&plain));
    let unchanged = r.clone();
    captured.reset(&Config::default(), 88, 22).unwrap();
    let reset = Frame::capture(&captured).unwrap();
    assert_eq!(
        r.finish(&reset, End::Truncated(Limit::Decisions)),
        Err(Error::Discontinuity)
    );
    assert_eq!(r, unchanged);
    r.finish(&previous, End::Truncated(Limit::Decisions))
        .unwrap();
    assert_eq!(r.episode().footer().unwrap().decisions, 6);
}
#[test]
fn trajectory_reward_discount_and_capture_conventions_are_explicit() {
    let f = frame();
    let mut r = Recorder::new(&header(), &f).unwrap();
    let json = serde_json::to_value(r.episode()).unwrap();
    assert_eq!(json["reward_convention"], "SparseZeroSumTerminal");
    assert_eq!(json["discount_convention"], "UndiscountedEpisodic");
    assert_eq!(json["capture_selection"], "AllDecisions");
    r.finish(&f, End::Truncated(Limit::Decisions)).unwrap();
    let seq = serde_json::to_value(r.episode().seat(Seat::P0).unwrap()).unwrap();
    assert_eq!(seq["discount_convention"], "UndiscountedEpisodic");
    assert_eq!(seq["versions"]["action"], "test-ledger-v1");
}
