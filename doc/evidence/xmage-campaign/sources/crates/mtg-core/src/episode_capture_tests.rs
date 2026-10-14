//! Explicit SYNTHETIC component edge only: nine cards at cleanup (CR 514.1).
//! The real normal-reset played collector acceptance is tests/capture.rs.
use super::*;
use crate::{
    episode::Driver,
    trajectory::{EpisodeKey, Header, Limits, Versions},
};
use policy::{Choice as C, Submission, VisibleRef, VisibleZone};
fn send(g: &mut Game, s: Seat, c: C) {
    let d = g.policy_observe(s, 256).unwrap().decision.unwrap();
    g.apply_policy(
        s,
        &Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![c],
        },
        256,
    )
    .unwrap()
}
#[test]
fn owned_capture_preserves_two_ordered_discards_and_rejects_missing_second() {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 160, 0).unwrap();
    send(&mut g, Seat::P0, C::Keep);
    send(&mut g, Seat::P1, C::Keep);
    g.start_turns().unwrap();
    for _ in 0..2 {
        g.objects
            .allocate(
                CardId::from_key("forest").unwrap(),
                Seat::P0,
                Zone::Hand(Seat::P0),
            )
            .unwrap();
    }
    g.turns.position = Some((1, Seat::P0, turns::Step::End));
    send(&mut g, Seat::P0, C::Pass);
    send(&mut g, Seat::P1, C::Pass);
    let h = Header {
        id: EpisodeKey {
            run: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
            ordinal: 0,
        },
        versions: Versions {
            schema: 2,
            engine: "synthetic-component".into(),
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
    let mut driver = Driver::synthetic_capture_test(g, h);
    let input = driver.observe(Seat::P0).unwrap();
    let d = input.decision.as_ref().unwrap();
    assert_eq!((d.kind, d.count), ("cleanup_discard", 2));
    assert_eq!(
        d.candidates,
        (0..9)
            .map(|row| C::Discard {
                card: VisibleRef {
                    zone: VisibleZone::Hand,
                    row
                }
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(d.legal_mask, vec![true; 9]);
    let mut sub = Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation,
        choices: vec![C::Discard {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: 8,
            },
        }],
    };
    let snapshot = driver.privileged_snapshot();
    assert!(driver.submit(Seat::P0, &sub).is_err());
    assert_eq!(driver.privileged_snapshot(), snapshot);
    assert!(driver.trajectory().unwrap().decisions().is_empty());
    assert!(driver.privileged_history().is_empty());
    sub.choices.push(C::Discard {
        card: VisibleRef {
            zone: VisibleZone::Hand,
            row: 0,
        },
    });
    driver.submit(Seat::P0, &sub).unwrap();
    let row = &driver.trajectory().unwrap().decisions()[0];
    assert_eq!(row.observation, input);
    assert_eq!(row.choice.submission, sub);
    assert_eq!(driver.observe(Seat::P0).unwrap().view.hand_counts, [7, 7]);
    driver
        .concede(Seat::P0, driver.episode_id().unwrap())
        .unwrap();
    let r = driver.finish().unwrap();
    let e = r.trajectory().unwrap();
    assert_eq!(e.seat(Seat::P0).unwrap().transitions[0].reward, -1);
    assert_eq!(e.seat(Seat::P1).unwrap().unassigned_reward, 1);
}
