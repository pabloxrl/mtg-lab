use super::*;
use mtg_core::{
    episode::Progress,
    trajectory::{EpisodeKey, Header, Limits, Versions},
};
use serde_json::{Value, json};
use std::num::NonZeroUsize;
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
fn normalized(bytes: &[u8]) -> Value {
    fn scrub(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "scope" || k == "store" {
                        *v = json!(0)
                    } else {
                        scrub(v)
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    scrub(v)
                }
            }
            _ => (),
        }
    }
    let e: Value = serde_json::from_slice(bytes).unwrap();
    let mut v: Value = serde_json::from_str(e["payload"].as_str().unwrap()).unwrap();
    v["objects"]["id"] = json!(0);
    scrub(&mut v);
    v
}

fn config() -> crate::simulate::Config {
    serde_json::from_str(include_str!("../../../fixtures/simulate/script-v3.json")).unwrap()
}
fn settle(d: &mut Driver) {
    for _ in 0..1000 {
        if d.advance(NonZeroUsize::MIN).unwrap() != Progress::InternalYield {
            return;
        }
    }
    panic!("bounded settlement");
}
fn unchanged(c: &Config, cursor: usize, d: &mut Driver) {
    let before = format!("{d:?}"); // Game, RNG, revision, pending work, history, capture, status, accounting.
    let mut index = cursor;
    assert!(submit(c, &mut index, 0, d).is_err());
    assert_eq!(index, cursor);
    assert_eq!(format!("{d:?}"), before);
}
#[test]
fn routing_rejects_preserve_full_owner_and_capture_then_valid_script_recovers() {
    let mut results = vec![];
    for capture in [false, true] {
        let c = config();
        let script = c.script.unwrap();
        let mut d = Driver::new(256).unwrap();
        if capture {
            d.reset_captured(&c.game, 178, 0, NonZeroUsize::MIN, &header())
                .unwrap();
        } else {
            d.reset(&c.game, 178, 0, NonZeroUsize::MIN).unwrap();
        }
        let mut cursor = 0;
        settle(&mut d);
        for position in 0..102 {
            if position > 0 {
                unchanged(&script, position - 1, &mut d);
            }
            for mutation in 0..6 {
                let mut bad: Config =
                    serde_json::from_value(serde_json::to_value(&script).unwrap()).unwrap();
                match mutation {
                    0 => bad.records.truncate(position),
                    1 => bad.records[position].decision += 1,
                    2 => bad.records[position].episode += 1,
                    3 => bad.records[position].record = "PRIVATE malformed".into(),
                    4 => {
                        bad.records[position].seat = if bad.records[position].seat == Seat::P0 {
                            Seat::P1
                        } else {
                            Seat::P0
                        }
                    }
                    _ => {
                        let mut r: Value =
                            serde_json::from_str(&bad.records[position].record).unwrap();
                        r["choices"] = json!([]);
                        bad.records[position].record = r.to_string();
                    }
                }
                unchanged(&bad, cursor, &mut d);
            }
            // Original first land is hand Forest birth 2, incarnation 1. Wrong
            // incarnation and a second land in the same turn must do nothing.
            if position == 4 || position == 5 {
                let mut bad: Config =
                    serde_json::from_value(serde_json::to_value(&script).unwrap()).unwrap();
                let mut r: Value = serde_json::from_str(&script.records[4].record).unwrap();
                if position == 4 {
                    r["choices"][0]["card"]["incarnation"] = json!(2);
                } else {
                    r["choices"][0]["card"]["birth"] = json!(3);
                }
                bad.records[position].record = r.to_string();
                unchanged(&bad, cursor, &mut d);
            }
            // Concession follows 101 choices and does not increment decisions.
            if position == 101 {
                let v = d.observe(Seat::P0).unwrap().view;
                assert_eq!(v.life, [20, 15]);
                assert_eq!(v.library_counts, [31, 31]);
            }
            submit(&script, &mut cursor, 0, &mut d).unwrap();
            assert_eq!(cursor, position + 1);
            assert_eq!(d.privileged_history().len(), cursor);
            let canonical: Record =
                serde_json::from_slice(&d.privileged_history()[position]).unwrap();
            let literal: Record = serde_json::from_str(&script.records[position].record).unwrap();
            assert_eq!(canonical, literal);
            if position < 101 {
                settle(&mut d);
            }
        }
        unchanged(&script, 0, &mut d); // replaying stale input cannot revive a terminal game
        let r = d.finish().unwrap();
        assert_eq!(r.accepted_decisions(), 101);
        assert_eq!(d.accounting().completed, 1);
        if let Some(e) = r.trajectory() {
            assert_eq!(e.decisions().len(), 101);
            assert_eq!(e.footer().unwrap().returns, [1, -1]);
        }
        results.push((
            normalized(r.privileged_snapshot()),
            r.privileged_history().to_vec(),
        ));
    }
    assert_eq!(results[0], results[1]);
}

#[test]
fn script_control_stops_account_remaining_input_and_never_start_next_episode() {
    for (stop, code, status) in [
        (crate::simulate::Stop::Deadline, 4, "truncated"),
        (crate::simulate::Stop::Sigint, 130, "incomplete"),
        (crate::simulate::Stop::Sigterm, 143, "incomplete"),
    ] {
        let mut c = config();
        // A full reset quantum makes poll 3 occur after the first accepted
        // choice, rather than during opening internal work.
        c.native.as_mut().unwrap().work_quantum = NonZeroUsize::new(1000).unwrap();
        c.episodes = 2;
        let mut polls = 0;
        let mut bytes = vec![];
        let code_actual = crate::native::run(&c, &mut bytes, || {
            polls += 1;
            (polls == 3).then_some(stop)
        })
        .unwrap();
        assert_eq!(code_actual, code);
        let rows: Vec<Value> = String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1]["status"], status);
        assert_eq!(rows[1]["script_status"], status);
        assert_eq!(rows[1]["script_consumed"], 1);
        assert_eq!(rows[2]["not_started"], 1);
        assert_eq!(rows[2]["script_remaining"], 101);
        assert_eq!(rows[2]["completed"], 0);
    }
}
