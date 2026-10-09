//! RFC0002 §§3/8/9: actual frozen decks, native decisions, canonical capture.
//! Reproducibility is a transport oracle, not a rules oracle: losses are checked
//! separately against CR704.5a/b. No synthetic state or alternative executor.
use mtg_core::{
    episode::{Driver, EpisodeResult, Progress, Status},
    game::{Config, DeckConfig},
    objects::Seat,
    trajectory::Limits,
};
use mtg_policy::{HEURISTIC_VERSION, Heuristic, LegalRandom, RNG_VERSION, VERSION};
use mtg_recorder::{
    Backpressure,
    collector::{Run, Storage},
    from_core_v2, read_v2,
};
use std::{collections::BTreeSet, num::NonZeroUsize};

fn play(
    run: &Run,
    random: bool,
    capture: bool,
    q: NonZeroUsize,
    seen: &mut BTreeSet<String>,
) -> EpisodeResult {
    let mut d = Driver::instrumented(
        1024,
        if capture {
            mtg_core::metrics::Mode::Counters
        } else {
            mtg_core::metrics::Mode::Off
        },
    )
    .unwrap();
    if capture {
        d.reset_captured(&run.config, 42, 7, q, &run.header(7).unwrap())
            .unwrap();
    } else {
        d.reset(&run.config, 42, 7, q).unwrap();
    }
    let mut ps = [0, 1].map(|s| LegalRandom::new(VERSION, RNG_VERSION, 42, 7, s).unwrap());
    let hs = [0, 1].map(|s| Heuristic::new(HEURISTIC_VERSION, s).unwrap());
    for _ in 0..20000 {
        if d.status().is_some() {
            break;
        }
        let mut p = d.advance(q).unwrap();
        while p == Progress::InternalYield {
            p = d.advance(q).unwrap();
        }
        if d.status().is_some() {
            break;
        }
        let a = d.observe(Seat::P0).unwrap().view.acting_seat.unwrap();
        let actor = if a == 0 { Seat::P0 } else { Seat::P1 };
        let o = d.observe(actor).unwrap();
        let dec = o.decision.as_ref().unwrap();
        seen.insert(dec.kind.into());
        let s = if random {
            ps[a as usize].choose(&o)
        } else {
            hs[a as usize].choose(&o)
        }
        .unwrap_or_else(|e| panic!("{} {:?}: {e:?}", dec.kind, run.config));
        // Check every encountered family once per game, including history/capture.
        if !seen.contains(&format!("rejected:{}", dec.kind)) {
            let snapshot = d.privileged_snapshot();
            // A strategy need not voluntarily reach every family (the heuristic
            // never mulligans or floats activation mana). Both must still choose
            // valid actions in every actual reached continuation. Restore the
            // authoritative Game solely to validate each independent proposal.
            for probe_random in [false, true] {
                let mut probe = mtg_core::game::Game::new().unwrap();
                probe.restore(&snapshot).unwrap();
                let input = probe.policy_observe(actor, 1024).unwrap();
                let proposed = if probe_random {
                    LegalRandom::new(VERSION, RNG_VERSION, 208, 7, a)
                        .unwrap()
                        .choose(&input)
                        .unwrap()
                } else {
                    hs[a as usize].choose(&input).unwrap()
                };
                probe.apply_policy(actor, &proposed, 1024).unwrap();
            }
            let history = d.privileged_history().to_vec();
            let trajectory = serde_json::to_value(d.trajectory()).unwrap();
            let mut bad = s.clone();
            bad.choices.clear();
            assert!(d.submit(actor, &bad).is_err());
            bad.choices = vec![mtg_core::game::policy::Choice::Spell];
            assert!(d.submit(actor, &bad).is_err());
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
            assert_eq!(snapshot, d.privileged_snapshot());
            assert_eq!(history, d.privileged_history());
            assert_eq!(trajectory, serde_json::to_value(d.trajectory()).unwrap());
            seen.insert(format!("rejected:{}", dec.kind));
        }
        d.submit(actor, &s).unwrap();
        let snapshot = d.privileged_snapshot();
        assert!(d.submit(actor, &s).is_err());
        assert_eq!(snapshot, d.privileged_snapshot());
    }
    let result = d.finish().unwrap();
    assert!(
        matches!(result.status(), Status::Completed(_)),
        "{:?}",
        run.config
    );
    if capture {
        // Complement the independently hand-counted tiny traces with the full
        // existing normal-game matrix; retain all replay and rules assertions.
        let c = d.metrics().unwrap();
        let trajectory = result.trajectory().unwrap();
        let footer = trajectory.footer().unwrap();
        assert_eq!(c.decisions, footer.decisions as u64);
        assert_eq!(c.logical_actions, footer.logical_actions as u64);
        assert_eq!(c.cancelled_actions, footer.cancelled_actions as u64);
        assert_eq!(
            c.committed_actions,
            trajectory
                .decisions()
                .iter()
                .filter(|d| d.choice.status == mtg_core::trajectory::v2::ActionStatus::Committed)
                .count() as u64
        );
        assert_eq!(
            (c.completed, c.rules_completed, c.failed, c.truncated),
            (1, 1, 0, 0)
        );
    } else {
        assert!(d.metrics().is_none());
    }
    let view = &result.final_observations().unwrap()[0].view;
    let v = serde_json::to_value(view).unwrap();
    let mut losses = 0;
    for seat in 0..2 {
        match v["terminal"]["losses"][seat].as_str() {
            Some("life") => {
                assert!(view.life[seat] <= 0);
                losses += 1;
            }
            Some("empty_draw") => {
                assert_eq!(view.library_counts[seat], 0);
                assert_eq!(view.turn.unwrap().1, seat as u8);
                assert_eq!(view.turn.unwrap().2, "draw");
                losses += 1;
            }
            None => (),
            other => panic!("native policies cannot concede: {other:?}"),
        }
    }
    assert!(losses > 0);
    result
}

#[test]
fn full_pool_native_games_replay_capture_quantum_and_rejections() {
    let mut all = BTreeSet::new();
    for random in [false, true] {
        for decks in [
            ["red", "green"],
            ["green", "red"],
            ["red", "red"],
            ["green", "green"],
        ] {
            for start in [0, 1] {
                let id = if random { VERSION } else { HEURISTIC_VERSION };
                let run = Run {
                    id: "20800000-1234-4234-8234-123456789abc".into(),
                    config: Config {
                        starting_seat: start,
                        seats: decks
                            .map(|deck| DeckConfig {
                                deck: deck.into(),
                                order: None,
                            })
                            .to_vec(),
                        ..Config::default()
                    },
                    policies: [id.into(), id.into()],
                    limits: Limits::default(),
                    first_ordinal: 7,
                    started: 1,
                };
                let mut seen = BTreeSet::new();
                let mut captured = play(&run, random, true, NonZeroUsize::MAX, &mut seen);
                let repeat = play(&run, random, true, NonZeroUsize::MIN, &mut BTreeSet::new());
                let off = play(&run, random, false, NonZeroUsize::MAX, &mut BTreeSet::new());
                assert_eq!(captured.privileged_history(), repeat.privileged_history());
                assert_eq!(captured.privileged_history(), off.privileged_history());
                assert_eq!(captured.final_observations(), repeat.final_observations());
                assert_eq!(captured.final_observations(), off.final_observations());
                assert_eq!(
                    serde_json::to_value(captured.trajectory()).unwrap(),
                    serde_json::to_value(repeat.trajectory()).unwrap()
                );
                let mut registry = mtg_core::episode::replay::Registry::default();
                registry
                    .register("20800000-1234-4234-8234-123456789abd", &mut captured)
                    .unwrap();
                let bytes = registry
                    .resolve("20800000-1234-4234-8234-123456789abd", &captured, |_, _| {
                        true
                    })
                    .unwrap();
                let replay = mtg_core::game::replay::played::verify(bytes).unwrap();
                assert_eq!(
                    replay.life(),
                    captured.final_observations().unwrap()[0].view.life
                );
                let expected = from_core_v2(captured.trajectory().unwrap()).unwrap();
                let bundle = run
                    .persist(
                        &[captured],
                        Storage {
                            queue_bytes: 64_000_000,
                            max_bytes: 64_000_000,
                            backpressure: Backpressure::Block,
                        },
                    )
                    .unwrap();
                assert_eq!(read_v2(bundle.bytes(), 64_000_000).unwrap(), vec![expected]);
                println!(
                    "{id} {decks:?} start={start} decisions={} kinds={seen:?}",
                    repeat.accepted_decisions()
                );
                all.extend(seen);
            }
        }
    }
    for kind in [
        "keep_or_mulligan",
        "bottom",
        "priority",
        "payment",
        "activation_payment",
        "activation_target",
        "growth_target",
        "bite_source",
        "bite_destination",
        "targets_complete",
        "cast_mode",
        "cast_discard",
        "trigger_order",
        "trigger_target",
        "attackers",
        "blockers",
        "combat_damage",
        "cleanup_discard",
    ] {
        assert!(
            all.contains(kind),
            "missing actual played decision {kind}: {all:?}"
        );
    }
}
