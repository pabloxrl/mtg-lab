//! Injected recorder failures isolate the ownership boundary, not played-game acceptance.
use super::*;
use crate::trajectory::{EpisodeKey, Limits, Versions};
thread_local! {
    pub(super) static CAPTURE_FRAMES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

// SYS-DATA-005, RFC B037: disabled capture must not build trajectory-only
// frames. Thread-local instrumentation cannot be disturbed by parallel tests.
#[test]
fn disabled_capture_never_materializes_trajectory_frames() {
    for capture in [false, true] {
        CAPTURE_FRAMES.with(|count| count.set(0));
        let mut d = if capture {
            driver(false)
        } else {
            let mut d = Driver::new(256).unwrap();
            d.reset(&Config::default(), 161, 0, NonZeroUsize::MAX)
                .unwrap();
            d
        };
        for i in 0..6 {
            while d.advance(NonZeroUsize::MAX).unwrap() == Progress::InternalYield {}
            let (seat, decision) = [Seat::P0, Seat::P1]
                .into_iter()
                .find_map(|s| d.observe(s).unwrap().decision.map(|decision| (s, decision)))
                .unwrap();
            d.submit(
                seat,
                &policy::Submission {
                    schema_version: 1,
                    revision: decision.revision,
                    generation: decision.generation,
                    choices: vec![if i < 2 {
                        policy::Choice::Keep
                    } else {
                        policy::Choice::Pass
                    }],
                },
            )
            .unwrap();
        }
        d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
        let result = d.finish().unwrap();
        assert_eq!(result.trajectory().is_some(), capture);
        assert_eq!(result.accepted_decisions(), 6);
        d.reset(&Config::default(), 162, 1, NonZeroUsize::MAX)
            .unwrap();
        let count = CAPTURE_FRAMES.with(|count| count.get());
        if capture {
            assert!(count > 0, "positive control must exercise the frame hook");
        } else {
            assert_eq!(count, 0, "disabled capture must not build extra frames");
        }
    }
}
#[derive(Debug)]
struct Zero;
impl Clock for Zero {
    fn now_ms(&self) -> u64 {
        0
    }
}
fn driver(bounded: bool) -> Driver {
    let mut d = if bounded {
        Driver::bounded(
            256,
            Budget {
                limits: Limits::default(),
                work_quantum: NonZeroUsize::new(1000).unwrap(),
                records: NonZeroUsize::new(100).unwrap(),
            },
            Box::new(Zero),
        )
        .unwrap()
    } else {
        Driver::new(256).unwrap()
    };
    let h = Header {
        id: EpisodeKey {
            run: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
            ordinal: 0,
        },
        versions: Versions {
            schema: 2,
            engine: "test".into(),
            rules: "cr-20260925".into(),
            cards: "pool-v1".into(),
            action: "policy-v1".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["test".into(), "test".into()],
        starting_seat: 0,
        limits: Limits::default(),
        restricted_replay: None,
    };
    d.reset_captured(&Config::default(), 161, 0, NonZeroUsize::MAX, &h)
        .unwrap();
    d
}
#[test]
fn injected_append_and_terminal_finish_failures_account_once_and_quarantine() {
    for terminal in [false, true] {
        let mut d = driver(true);
        let frame = d.frame().unwrap();
        // Close the real recorder early to force its existing AlreadyEnded error.
        d.recorder
            .as_mut()
            .unwrap()
            .finish(&frame, trajectory::End::Failed("injected".into()))
            .unwrap();
        let error = if terminal {
            d.concede(Seat::P1, d.episode_id().unwrap())
        } else {
            let v = d.observe(Seat::P0).unwrap().decision.unwrap();
            d.submit(
                Seat::P0,
                &policy::Submission {
                    schema_version: 1,
                    revision: v.revision,
                    generation: v.generation,
                    choices: vec![policy::Choice::Keep],
                },
            )
        };
        assert_eq!(error, Err(Error::Capture(trajectory::Error::AlreadyEnded)));
        assert_eq!(d.privileged_history().len(), 1);
        assert!(d.advance(NonZeroUsize::MIN).is_err());
        let r = d.finish().unwrap();
        assert_eq!(r.status(), Status::Failed(Failure::Recording));
        assert_eq!(r.privileged_history().len(), 1);
        assert!(!r.trajectory().unwrap().footer().unwrap().complete);
        assert!(r.trajectory().unwrap().seat(Seat::P0).is_err());
        assert_eq!(d.accounting().failed, 1);
        assert_eq!(d.accounting().completed, 0);
        assert!(d.finish().is_err());
        d.reset(&Config::default(), 161, 1, NonZeroUsize::MIN)
            .unwrap();
        d.finish().unwrap();
        assert_eq!(d.accounting().started, 2);
        assert_eq!(d.accounting().failed, 1);
        assert_eq!(d.accounting().incomplete, 1);
    }
}
#[test]
fn injected_settlement_failure_is_not_retryable() {
    let mut d = driver(true);
    let frame = d.frame().unwrap();
    d.recorder
        .as_mut()
        .unwrap()
        .finish(&frame, trajectory::End::Failed("injected".into()))
        .unwrap();
    assert_eq!(
        d.advance(NonZeroUsize::MIN),
        Err(Error::Capture(trajectory::Error::AlreadyEnded))
    );
    assert!(d.advance(NonZeroUsize::MIN).is_err());
    assert_eq!(
        d.finish().unwrap().status(),
        Status::Failed(Failure::Recording)
    );
    assert_eq!(d.accounting().failed, 1);
}

#[test]
fn unbounded_capture_failure_still_has_exactly_one_owned_failed_result() {
    let mut d = driver(false);
    let frame = d.frame().unwrap();
    d.recorder
        .as_mut()
        .unwrap()
        .finish(&frame, trajectory::End::Failed("injected".into()))
        .unwrap();
    assert_eq!(
        d.concede(Seat::P1, d.episode_id().unwrap()),
        Err(Error::Capture(trajectory::Error::AlreadyEnded))
    );
    assert_eq!(
        d.finish().unwrap().status(),
        Status::Failed(Failure::Recording)
    );
    assert_eq!(d.accounting().started, 1);
    assert_eq!(d.accounting().failed, 1);
    assert_eq!(d.accounting().completed, 0);
    assert!(d.finish().is_err());
}

#[test]
fn metric_modes_do_not_materialize_extra_capture_frames() {
    let header = driver(false).capture_header.unwrap();
    for mode in [Mode::Off, Mode::Counters] {
        for capture in [false, true] {
            CAPTURE_FRAMES.with(|n| n.set(0));
            let mut d = Driver::instrumented(256, mode).unwrap();
            if capture {
                d.reset_captured(&Config::default(), 214, 0, NonZeroUsize::MAX, &header)
                    .unwrap();
            } else {
                d.reset(&Config::default(), 214, 0, NonZeroUsize::MAX)
                    .unwrap();
            }
            let v = d.observe(Seat::P0).unwrap().decision.unwrap();
            d.submit(
                Seat::P0,
                &policy::Submission {
                    schema_version: 1,
                    revision: v.revision,
                    generation: v.generation,
                    choices: vec![policy::Choice::Keep],
                },
            )
            .unwrap();
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
            d.finish().unwrap();
            let frames = CAPTURE_FRAMES.with(|n| n.get());
            assert_eq!(frames > 0, capture);
            assert_eq!(d.metrics().is_some(), mode == Mode::Counters);
        }
    }
}

#[test]
fn metric_recording_failure_outranks_concession_completion() {
    // Test-only fault injection at the real recorder boundary, not a substitute
    // for normal-reset played acceptance. RFC B021/B037 require quarantine.
    let header = driver(false).capture_header.unwrap();
    let mut d = Driver::instrumented(256, Mode::Counters).unwrap();
    d.reset_captured(&Config::default(), 214, 0, NonZeroUsize::MAX, &header)
        .unwrap();
    let frame = d.frame().unwrap();
    d.recorder
        .as_mut()
        .unwrap()
        .finish(
            &frame,
            trajectory::End::Failed("PRIVATE_RECORDING_SENTINEL".into()),
        )
        .unwrap();
    assert!(d.concede(Seat::P1, d.episode_id().unwrap()).is_err());
    d.finish().unwrap();
    let c = d.metrics().unwrap();
    assert_eq!(
        (
            c.started,
            c.failed,
            c.completed,
            c.rules_completed,
            c.recording_failures
        ),
        (1, 1, 0, 0, 1)
    );
    assert!(
        !serde_json::to_string(&c.report())
            .unwrap()
            .contains("PRIVATE_RECORDING_SENTINEL")
    );
    assert!(d.finish().is_err());
    assert_eq!(d.metrics().unwrap().failed, 1);
}
