//! B021 oracle: sample accepted decisions 2 and 4 of five; capacity one drops exactly one.
use mtg_core::{
    episode::{Budget, Clock, Driver, Failure, Progress, Status},
    game::{
        Config,
        policy::{self, Choice},
    },
    metrics::{Mode, TraceConfig},
    objects::Seat,
    trajectory::{EpisodeKey, Header, Limits, Versions},
};
use std::num::{NonZeroU64, NonZeroUsize};
fn ready(d: &mut Driver) {
    while d.advance(NonZeroUsize::MAX).unwrap() == Progress::InternalYield {}
}
fn send(d: &mut Driver, seat: Seat, choice: Choice) {
    let decision = d.observe(seat).unwrap().decision.unwrap();
    d.submit(
        seat,
        &policy::Submission {
            schema_version: 1,
            revision: decision.revision,
            generation: decision.generation,
            choices: vec![choice.clone()],
        },
    )
    .unwrap_or_else(|error| {
        panic!(
            "{seat:?} {choice:?} at {:?}: {error:?}",
            d.observe(seat).unwrap().view.turn
        )
    });
}

fn header() -> Header {
    Header {
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
        policies: ["ledger".into(), "ledger".into()],
        starting_seat: 0,
        limits: Limits::default(),
        restricted_replay: None,
    }
}

fn run(mode: Mode, capture: bool) -> (Driver, mtg_core::episode::EpisodeResult) {
    let mut d = Driver::instrumented(256, mode).unwrap();
    d.set_trace_config(TraceConfig {
        every: NonZeroU64::new(2).unwrap(),
        capacity: 1,
    })
    .unwrap();
    if capture {
        d.reset_captured(&Config::default(), 215, 0, NonZeroUsize::MAX, &header())
            .unwrap();
    } else {
        d.reset(&Config::default(), 215, 0, NonZeroUsize::MAX)
            .unwrap();
    }
    send(&mut d, Seat::P0, Choice::Keep);
    send(&mut d, Seat::P1, Choice::Keep);
    ready(&mut d);
    for seat in [Seat::P0, Seat::P1, Seat::P0] {
        send(&mut d, seat, Choice::Pass);
        ready(&mut d);
    }
    d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
    let result = d.finish().unwrap();
    (d, result)
}
#[test]
fn fixed_selection_exact_drop_and_public_privacy() {
    let (a, _) = run(Mode::SampledTrace, false);
    let (b, _) = run(Mode::SampledTrace, true);
    let trace = a.diagnostic_trace().expect("sampled checkpoints");
    assert_eq!(trace, b.diagnostic_trace().unwrap());
    assert_eq!(trace.records.len(), 1);
    assert_eq!(trace.records[0].decision, 2);
    assert_eq!(trace.dropped, 1);
    assert!(!trace.overflowed);
    let value = serde_json::to_value(trace).unwrap();
    assert_eq!(value["schema_version"], 1);
    let keys: Vec<_> = value["records"][0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["decision", "rules_work_units"]);
}
#[test]
fn four_modes_capture_independence_and_existing_replay_format() {
    let (_, baseline) = run(Mode::Off, false);
    for mode in [
        Mode::Off,
        Mode::Counters,
        Mode::SampledTrace,
        Mode::FullReplay,
    ] {
        for capture in [false, true] {
            let (d, r) = run(mode, capture);
            assert_eq!(r.privileged_history(), baseline.privileged_history());
            assert_eq!(r.status(), baseline.status());
            assert_eq!(r.accepted_decisions(), 5);
            assert_eq!(r.capture_requested(), capture);
            assert_eq!(r.trajectory().is_some(), capture);
            assert_eq!(d.metrics().is_some(), mode != Mode::Off);
            let replay = r.privileged_replay(4_000_000).unwrap();
            assert_eq!(replay.is_some(), mode == Mode::FullReplay);
            if let Some(bytes) = replay {
                let game = mtg_core::opening::replay::played::verify(&bytes).unwrap();
                assert_eq!(game.outcome().map(Status::Completed), Some(r.status()));
                let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                assert_eq!(v["choices"].as_array().unwrap().len(), 6); // five decisions + concession
                assert!(r.privileged_replay(1).is_err());
            }
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
#[test]
fn promised_replay_overflow_is_failed_and_unavailable() {
    let mut d = Driver::bounded_instrumented(
        256,
        Budget {
            limits: Limits::default(),
            work_quantum: NonZeroUsize::MAX,
            records: NonZeroUsize::MIN,
        },
        Box::new(Zero),
        Mode::FullReplay,
    )
    .unwrap();
    d.reset(&Config::default(), 215, 0, NonZeroUsize::MAX)
        .unwrap();
    send(&mut d, Seat::P0, Choice::Keep);
    let decision = d.observe(Seat::P1).unwrap().decision.unwrap();
    let before = d.privileged_snapshot();
    assert!(
        d.submit(
            Seat::P1,
            &policy::Submission {
                schema_version: 1,
                revision: decision.revision,
                generation: decision.generation,
                choices: vec![Choice::Keep]
            }
        )
        .is_err()
    );
    assert_eq!(d.privileged_snapshot(), before);
    let r = d.finish().unwrap();
    assert_eq!(r.status(), Status::Failed(Failure::RecordCapacity));
    assert!(r.privileged_replay(4_000_000).is_err());
}
#[test]
fn trace_configuration_is_bounded_and_immutable_after_start() {
    let mut d = Driver::instrumented(256, Mode::SampledTrace).unwrap();
    assert!(
        d.set_trace_config(TraceConfig {
            every: NonZeroU64::MIN,
            capacity: usize::MAX
        })
        .is_err()
    );
    d.reset(&Config::default(), 215, 0, NonZeroUsize::MAX)
        .unwrap();
    assert!(d.set_trace_config(TraceConfig::default()).is_err());
}

#[test]
fn zero_capacity_rejection_and_reset_have_literal_accounting() {
    let mut d = Driver::instrumented(256, Mode::SampledTrace).unwrap();
    d.set_trace_config(TraceConfig {
        every: NonZeroU64::MIN,
        capacity: 0,
    })
    .unwrap();
    d.reset(&Config::default(), 215, 0, NonZeroUsize::MAX)
        .unwrap();
    let decision = d.observe(Seat::P0).unwrap().decision.unwrap();
    let stale = policy::Submission {
        schema_version: 1,
        revision: decision.revision + 1,
        generation: decision.generation,
        choices: vec![Choice::Keep],
    };
    let snapshot = d.privileged_snapshot();
    assert!(d.submit(Seat::P0, &stale).is_err());
    assert_eq!(d.privileged_snapshot(), snapshot);
    assert_eq!(d.diagnostic_trace().unwrap().dropped, 0);
    send(&mut d, Seat::P0, Choice::Keep);
    assert_eq!(d.diagnostic_trace().unwrap().dropped, 1);
    assert!(d.diagnostic_trace().unwrap().records.is_empty());
    d.finish().unwrap();
    d.reset(&Config::default(), 215, 1, NonZeroUsize::MAX)
        .unwrap();
    assert_eq!(d.diagnostic_trace().unwrap().dropped, 0);
    send(&mut d, Seat::P0, Choice::Keep);
    assert_eq!(d.diagnostic_trace().unwrap().dropped, 1);
}
