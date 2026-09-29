//! Injected recorder failures isolate the ownership boundary, not played-game acceptance.
use super::*;
use crate::trajectory::{EpisodeKey, Limits, Versions};
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
