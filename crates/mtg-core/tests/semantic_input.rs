//! GH-176: original CR 103/104.3a/117/305/400.7/601/608/508–510/514
//! expectations. Normal ordered frozen-green reset, seed 176, ordinal 0.
//! The script extends the independently authored GH-113 Cub/Growth combat case.
use mtg_core::{
    episode::{Budget, Clock, Driver, Error, Failure, Progress, RecordContext, Status},
    game::{
        Config, DeckConfig, actions, policy,
        terminal::{LossReason, Outcome},
    },
    objects::{Seat, Zone},
    trajectory::{EpisodeKey, Header, Limits, PolicyInfo, Versions},
};
use serde_json::{Value, json};
use std::num::NonZeroUsize;
const CAP: usize = 256;
#[derive(Debug)]
struct Zero;
impl Clock for Zero {
    fn now_ms(&self) -> u64 {
        0
    }
}
fn config() -> Config {
    // Literal top seven: Cub, Cub, Growth, Forest x4. Complete frozen green deck.
    let order = [
        ("bear-cub", 2),
        ("giant-growth", 1),
        ("forest", 16),
        ("bear-cub", 2),
        ("giant-growth", 2),
        ("llanowar-elves", 3),
        ("druid-of-the-cowl", 2),
        ("magnigoth-sentry", 2),
        ("tajuru-pathwarden", 2),
        ("thornweald-archer", 3),
        ("bite-down", 3),
        ("wildheart-invoker", 2),
    ]
    .into_iter()
    .flat_map(|(k, n)| vec![k.to_owned(); n])
    .collect::<Vec<_>>();
    assert_eq!(order.len(), 40);
    Config {
        seats: vec![
            DeckConfig {
                deck: "green".into(),
                order: Some(order)
            };
            2
        ],
        ..Config::default()
    }
}
fn object(s: Seat, birth: u64, key: &str, zone: Zone, incarnation: u64) -> Value {
    json!({"birth":birth,"card":key,"owner":s,"zone":zone,"incarnation":incarnation})
}
fn hand(s: Seat, n: u64, key: &str) -> Value {
    object(
        s,
        n + if s == Seat::P0 { 0 } else { 40 },
        key,
        Zone::Hand(s),
        1,
    )
}
fn permanent(s: Seat, n: u64, key: &str) -> Value {
    object(
        s,
        n + if s == Seat::P0 { 0 } else { 40 },
        key,
        Zone::Battlefield,
        if key == "forest" { 2 } else { 3 },
    )
}
fn record(s: Seat, kind: &str, choices: Value) -> Value {
    json!({"version":1,"actor":s,"decision":kind,"choices":choices})
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

fn context(d: &Driver, s: Seat) -> RecordContext {
    let v = d.observe(s).unwrap().decision.unwrap();
    RecordContext::Decision {
        revision: v.revision,
        generation: v.generation,
    }
}
fn settle(d: &mut Driver) {
    for _ in 0..300 {
        if d.advance(NonZeroUsize::MIN).unwrap() != Progress::InternalYield {
            return;
        }
    }
    panic!("settlement bound");
}
fn fresh(capture: bool, bounded: bool, limits: Limits, records: usize) -> Driver {
    let mut d = if bounded {
        Driver::bounded(
            CAP,
            Budget {
                limits: limits.clone(),
                work_quantum: NonZeroUsize::MIN,
                records: NonZeroUsize::new(records).unwrap(),
            },
            Box::new(Zero),
        )
        .unwrap()
    } else {
        Driver::new(CAP).unwrap()
    };
    if capture {
        let mut h = header();
        h.limits = limits;
        d.reset_captured(&config(), 176, 0, NonZeroUsize::MIN, &h)
            .unwrap();
    } else {
        d.reset(&config(), 176, 0, NonZeroUsize::MIN).unwrap();
    }
    d
}
fn unchanged(d: &mut Driver, seat: Seat, ctx: RecordContext, bytes: &[u8]) {
    // Full owner debug includes Game/RNG, revision, recorder, status, counts,
    // pending work, history and finalization; not merely the public view.
    let before = format!("{d:?}");
    assert!(d.submit_record(seat, ctx, bytes).is_err());
    assert_eq!(format!("{d:?}"), before);
}
struct Script {
    d: Driver,
    canonical: Driver,
    records: Vec<Value>,
}
impl Script {
    fn new(capture: bool, bounded: bool) -> Self {
        let mut d = fresh(capture, bounded, Limits::default(), 1000);
        let mut canonical = fresh(capture, bounded, Limits::default(), 1000);
        settle(&mut d);
        settle(&mut canonical);
        // Advance through a previous episode so owner and core revisions differ.
        // An implementation forwarding the decoded core revision must fail.
        for driver in [&mut d, &mut canonical] {
            driver.finish().unwrap();
            if capture {
                driver
                    .reset_captured(&config(), 176, 0, NonZeroUsize::MIN, &header())
                    .unwrap();
            } else {
                driver.reset(&config(), 176, 0, NonZeroUsize::MIN).unwrap();
            }
            settle(driver);
        }
        Self {
            d,
            canonical,
            records: vec![],
        }
    }
    fn send(&mut self, s: Seat, kind: &str, choices: Value) {
        let r = record(s, kind, choices);
        let bytes = serde_json::to_vec(&r).unwrap();
        let ctx = context(&self.d, s);
        unchanged(
            &mut self.d,
            if s == Seat::P0 { Seat::P1 } else { Seat::P0 },
            ctx,
            &bytes,
        );
        let mut bad = r.clone();
        bad["choices"] = json!([]);
        unchanged(&mut self.d, s, ctx, &serde_json::to_vec(&bad).unwrap());
        let RecordContext::Decision {
            revision,
            generation,
        } = ctx
        else {
            panic!()
        };
        for stale in [
            RecordContext::Decision {
                revision: revision - 1,
                generation,
            },
            RecordContext::Decision {
                revision,
                generation: generation + 1,
            },
        ] {
            unchanged(&mut self.d, s, stale, &bytes);
        }
        // Baseline uses delivered ordinary submit, with decoded choices. The
        // independent oracles are handwritten records and literal checkpoints.
        let mut g = mtg_core::game::Game::new().unwrap();
        g.restore(&self.canonical.privileged_snapshot()).unwrap();
        let actions::Decoded::Decision { mut submission, .. } =
            actions::decode(&g, &bytes, CAP).unwrap()
        else {
            panic!()
        };
        let input = self.canonical.observe(s).unwrap();
        let decision = input.decision.as_ref().unwrap();
        submission.revision = decision.revision;
        submission.generation = decision.generation;
        self.canonical.submit(s, &submission).unwrap();
        self.d.submit_record(s, ctx, &bytes).unwrap();
        self.records.push(r);
        assert_eq!(
            self.d.privileged_history(),
            self.canonical.privileged_history()
        );
        assert_eq!(
            self.d
                .privileged_history()
                .iter()
                .map(|b| serde_json::from_slice::<Value>(b).unwrap())
                .collect::<Vec<_>>(),
            self.records
        );
        assert_eq!(self.d.trajectory(), self.canonical.trajectory());
        if let Some(e) = self.d.trajectory() {
            let row = e.decisions().last().unwrap();
            assert_eq!(row.observation, input);
            assert_eq!(row.choice.submission, submission);
            assert_eq!(row.choice.policy, PolicyInfo::default());
        }
        settle(&mut self.d);
        settle(&mut self.canonical);
        assert_eq!(
            normalized(&self.d.privileged_snapshot()),
            normalized(&self.canonical.privileged_snapshot())
        );
    }
    fn one(&mut self, s: Seat, k: &str, c: Value) {
        self.send(s, k, json!([c]));
    }
    fn idle(&mut self) {
        let (s, d) = [Seat::P0, Seat::P1]
            .into_iter()
            .find_map(|s| self.d.observe(s).unwrap().decision.map(|d| (s, d)))
            .unwrap();
        let c = match d.kind {
            "priority" => json!({"kind":"pass"}),
            "attackers" | "blockers" | "combat_damage" => json!({"kind":"finish_combat"}),
            _ => panic!("unscripted {}", d.kind),
        };
        self.one(s, d.kind, c);
    }
    fn pair(&mut self) {
        self.idle();
        self.idle();
    }
    fn until(&mut self, t: u64, step: &str) {
        for _ in 0..300 {
            if self
                .d
                .observe(Seat::P0)
                .unwrap()
                .view
                .turn
                .is_some_and(|p| p.0 == t && p.2 == step)
            {
                return;
            }
            self.idle();
        }
        panic!("script bound");
    }
    fn cast(&mut self, s: Seat, n: u64) {
        self.one(
            s,
            "priority",
            json!({"kind":"cast","card":hand(s,n,"bear-cub")}),
        );
        for land in [3, 4] {
            self.one(
                s,
                "payment",
                json!({"kind":"tap_mana","card":permanent(s,land,"forest")}),
            );
        }
        for _ in 0..2 {
            self.one(s, "payment", json!({"kind":"pay","color":4}));
        }
        self.one(s, "payment", json!({"kind":"finish_payment"}));
        self.pair();
    }
    fn cub(&self) -> [u32; 3] {
        let v = self.d.observe(Seat::P0).unwrap().view;
        let c = v
            .public_zones
            .iter()
            .find(|z| z.zone == "battlefield")
            .unwrap()
            .cards
            .iter()
            .find(|c| c.card == "bear-cub" && c.controller == 0)
            .unwrap();
        c.creature.unwrap()
    }
}
#[test]
fn records_played_spell_combat_capture_and_owner_modes() {
    let mut results = vec![];
    for capture in [false, true] {
        for bounded in [false, true] {
            let mut g = Script::new(capture, bounded);
            for s in [Seat::P0, Seat::P1] {
                g.one(s, "keep_or_mulligan", json!({"kind":"keep"}));
            }
            assert_eq!(g.d.observe(Seat::P0).unwrap().view.hand_counts, [7, 7]);
            assert_eq!(g.d.observe(Seat::P0).unwrap().view.library_counts, [33, 33]);
            for t in 1..=4 {
                let s = if t % 2 == 1 { Seat::P0 } else { Seat::P1 };
                g.until(t, "precombat_main");
                g.one(
                    s,
                    "priority",
                    json!({"kind":"play_land","card":hand(s,if t<=2{3}else{4},"forest")}),
                );
                if t >= 3 {
                    g.cast(s, 0);
                }
            }
            g.until(6, "precombat_main");
            g.cast(Seat::P1, 1);
            g.until(7, "precombat_main");
            g.one(
                Seat::P0,
                "priority",
                json!({"kind":"cast","card":hand(Seat::P0,2,"giant-growth")}),
            );
            g.one(
                Seat::P0,
                "growth_target",
                json!({"kind":"target","card":permanent(Seat::P0,0,"bear-cub")}),
            );
            g.one(
                Seat::P0,
                "targets_complete",
                json!({"kind":"finish_targets"}),
            );
            g.one(
                Seat::P0,
                "payment",
                json!({"kind":"tap_mana","card":permanent(Seat::P0,3,"forest")}),
            );
            g.one(Seat::P0, "payment", json!({"kind":"pay","color":4}));
            g.one(Seat::P0, "payment", json!({"kind":"finish_payment"}));
            g.pair();
            assert_eq!(g.cub(), [5, 5, 0]);
            g.until(7, "declare_attackers");
            g.one(
                Seat::P0,
                "attackers",
                json!({"kind":"select_attackers","cards":[permanent(Seat::P0,0,"bear-cub")]}),
            );
            g.one(Seat::P0, "attackers", json!({"kind":"finish_combat"}));
            g.pair();
            g.one(Seat::P1,"blockers",json!({"kind":"select_blockers","blocks":[[permanent(Seat::P1,0,"bear-cub"),permanent(Seat::P0,0,"bear-cub")],[permanent(Seat::P1,1,"bear-cub"),permanent(Seat::P0,0,"bear-cub")]]}));
            g.one(Seat::P1, "blockers", json!({"kind":"finish_combat"}));
            g.pair();
            g.one(Seat::P0,"combat_damage",json!({"kind":"assign_damage","attacker":permanent(Seat::P0,0,"bear-cub"),"amounts":[[permanent(Seat::P1,0,"bear-cub"),3],[permanent(Seat::P1,1,"bear-cub"),2]]}));
            g.one(Seat::P0, "combat_damage", json!({"kind":"finish_combat"}));
            assert_eq!(g.cub(), [5, 5, 4]);
            let v = g.d.observe(Seat::P0).unwrap().view;
            assert_eq!(v.life, [20, 20]);
            assert_eq!(
                v.public_zones
                    .iter()
                    .find(|z| z.zone == "graveyard_1")
                    .unwrap()
                    .cards
                    .len(),
                2
            );
            g.until(8, "upkeep");
            assert_eq!(g.cub(), [2, 2, 0]);
            let bytes =
                serde_json::to_vec(&record(Seat::P0, "concession", json!([{"kind":"concede"}])))
                    .unwrap();
            let ctx = RecordContext::Concession {
                episode: g.d.episode_id().unwrap(),
            };
            unchanged(&mut g.d, Seat::P1, ctx, &bytes);
            g.d.submit_record(Seat::P0, ctx, &bytes).unwrap();
            g.canonical
                .concede(Seat::P0, g.canonical.episode_id().unwrap())
                .unwrap();
            unchanged(&mut g.d, Seat::P0, ctx, &bytes);
            assert_eq!(g.d.trajectory(), g.canonical.trajectory());
            assert_eq!(g.d.privileged_history(), g.canonical.privileged_history());
            let result = g.d.finish().unwrap();
            assert_eq!(
                result.status(),
                Status::Completed(Outcome {
                    winner: Some(Seat::P1),
                    losses: [Some(LossReason::Concession), None]
                })
            );
            assert_eq!(result.accepted_decisions() as usize, g.records.len());
            if let Some(e) = result.trajectory() {
                assert_eq!(e.footer().unwrap().returns, [-1, 1]);
                assert_eq!(e.footer().unwrap().boundary_reward, [-1, 1]);
                assert_eq!(e.footer().unwrap().decisions, g.records.len());
                assert_eq!(e.seat(Seat::P0).unwrap().total_return, -1);
                assert_eq!(e.seat(Seat::P1).unwrap().total_return, 1);
            }
            assert_eq!(g.d.accounting().completed, 1);
            assert!(g.d.finish().is_err());
            results.push((
                normalized(result.privileged_snapshot()),
                result.privileged_history().to_vec(),
            ));
        }
    }
    for r in &results {
        assert_eq!(r, &results[0]);
    }
}

#[test]
fn records_rejections_recover_without_owner_mutation() {
    for capture in [false, true] {
        for bounded in [false, true] {
            let mut d = fresh(capture, bounded, Limits::default(), 1000);
            let keep = serde_json::to_vec(&record(
                Seat::P0,
                "keep_or_mulligan",
                json!([{"kind":"keep"}]),
            ))
            .unwrap();
            let fake = RecordContext::Decision {
                revision: 1,
                generation: 0,
            };
            unchanged(&mut d, Seat::P0, fake, &keep);
            let concede =
                serde_json::to_vec(&record(Seat::P1, "concession", json!([{"kind":"concede"}])))
                    .unwrap();
            // Partial reset cannot concede, even if it has assigned an episode token.
            if let Some(episode) = d.episode_id() {
                unchanged(
                    &mut d,
                    Seat::P1,
                    RecordContext::Concession { episode },
                    &concede,
                );
            }
            settle(&mut d);
            let old_episode = d.episode_id().unwrap();
            let old_context = context(&d, Seat::P0);
            for bad in [
                b"".to_vec(),
                b"{".to_vec(),
                b"{}".to_vec(),
                br#"{"version":1,"actor":"P0","decision":"keep_or_mulligan"}"#.to_vec(),
            ] {
                unchanged(&mut d, Seat::P0, old_context, &bad);
            }
            for change in 0..5 {
                let mut r = record(Seat::P0, "keep_or_mulligan", json!([{"kind":"keep"}]));
                match change {
                    0 => r["version"] = json!(99),
                    1 => r["actor"] = json!(Seat::P1),
                    2 => r["decision"] = json!("priority"),
                    3 => r["choices"] = json!([]),
                    _ => r["choices"] = json!([{"kind":"pass"}]),
                };
                unchanged(
                    &mut d,
                    Seat::P0,
                    old_context,
                    &serde_json::to_vec(&r).unwrap(),
                );
            }
            // Valid recovery through mulligan proves rejects did not poison pending work/RNG.
            let mull = serde_json::to_vec(&record(
                Seat::P0,
                "keep_or_mulligan",
                json!([{"kind":"mulligan"}]),
            ))
            .unwrap();
            d.submit_record(Seat::P0, old_context, &mull).unwrap();
            settle(&mut d);
            unchanged(&mut d, Seat::P0, old_context, &keep);
            let k1 = serde_json::to_vec(&record(
                Seat::P1,
                "keep_or_mulligan",
                json!([{"kind":"keep"}]),
            ))
            .unwrap();
            let ctx = context(&d, Seat::P1);
            d.submit_record(Seat::P1, ctx, &k1).unwrap();
            settle(&mut d);
            // Mulligan hand is shuffled: the legal Bottom candidate supplies the
            // choice, while malformed/stale-reference negatives above/below remain literal.
            let obs = d.observe(Seat::P0).unwrap();
            let dec = obs.decision.unwrap();
            assert_eq!(dec.kind, "bottom");
            let sub = policy::Submission {
                schema_version: 1,
                revision: dec.revision,
                generation: dec.generation,
                choices: vec![dec.candidates[0].clone()],
            };
            d.submit(Seat::P0, &sub).unwrap();
            settle(&mut d);
            let ctx = context(&d, Seat::P0);
            d.submit_record(Seat::P0, ctx, &keep).unwrap();
            let cctx = RecordContext::Concession {
                episode: old_episode,
            };
            unchanged(&mut d, Seat::P1, cctx, &concede); // opening-to-turn internal boundary
            settle(&mut d);
            d.submit_record(Seat::P1, cctx, &concede).unwrap();
            unchanged(&mut d, Seat::P1, cctx, &concede);
            let r = d.finish().unwrap();
            assert_eq!(r.accepted_decisions(), 4);
            assert_eq!(d.accounting().completed, 1);
            unchanged(&mut d, Seat::P1, cctx, &concede);
            d.reset(&config(), 176, 0, NonZeroUsize::MIN).unwrap();
            settle(&mut d);
            unchanged(&mut d, Seat::P0, old_context, &keep);
            unchanged(&mut d, Seat::P1, cctx, &concede);
            let ctx = context(&d, Seat::P0);
            d.submit_record(Seat::P0, ctx, &keep).unwrap();
        }
    }
}
#[test]
fn records_object_corruptions_and_illegal_land_are_transactional() {
    for capture in [false, true] {
        for bounded in [false, true] {
            let mut g = Script::new(capture, bounded);
            for s in [Seat::P0, Seat::P1] {
                g.one(s, "keep_or_mulligan", json!({"kind":"keep"}));
            }
            g.until(1, "precombat_main");
            let valid = record(
                Seat::P0,
                "priority",
                json!([{"kind":"play_land","card":hand(Seat::P0,3,"forest")}]),
            );
            let ctx = context(&g.d, Seat::P0);
            for field in ["birth", "incarnation", "card", "owner", "zone"] {
                let mut bad = valid.clone();
                bad["choices"][0]["card"][field] = match field {
                    "birth" => json!(400),
                    "incarnation" => json!(2),
                    "card" => json!("bear-cub"),
                    "owner" => json!(Seat::P1),
                    _ => json!(Zone::Battlefield),
                };
                unchanged(&mut g.d, Seat::P0, ctx, &serde_json::to_vec(&bad).unwrap());
            }
            g.one(Seat::P0, "priority", valid["choices"][0].clone());
            let ctx = context(&g.d, Seat::P0);
            unchanged(
                &mut g.d,
                Seat::P0,
                ctx,
                &serde_json::to_vec(&valid).unwrap(),
            ); // departed hand incarnation
            let second = record(
                Seat::P0,
                "priority",
                json!([{"kind":"play_land","card":hand(Seat::P0,4,"forest")}]),
            );
            unchanged(
                &mut g.d,
                Seat::P0,
                ctx,
                &serde_json::to_vec(&second).unwrap(),
            ); // one land per turn
            g.one(Seat::P0, "priority", json!({"kind":"pass"}));
        }
    }
}
#[test]
fn records_limits_and_record_exhaustion_follow_owner_finalization() {
    for capture in [false, true] {
        for record_limit in [false, true] {
            let limits = Limits {
                decisions: if record_limit { None } else { Some(1) },
                ..Limits::default()
            };
            let mut d = fresh(capture, true, limits, if record_limit { 1 } else { 1000 });
            settle(&mut d);
            let keep = serde_json::to_vec(&record(
                Seat::P0,
                "keep_or_mulligan",
                json!([{"kind":"keep"}]),
            ))
            .unwrap();
            let ctx = context(&d, Seat::P0);
            d.submit_record(Seat::P0, ctx, &keep).unwrap();
            let keep1 = serde_json::to_vec(&record(
                Seat::P1,
                "keep_or_mulligan",
                json!([{"kind":"keep"}]),
            ))
            .unwrap();
            let ctx = context(&d, Seat::P1);
            let snapshot = d.privileged_snapshot();
            let history = d.privileged_history().to_vec();
            if record_limit {
                assert_eq!(
                    d.submit_record(Seat::P1, ctx, &keep1),
                    Err(Error::RecordCapacity)
                );
                assert_eq!(d.status(), Some(Status::Failed(Failure::RecordCapacity)));
            } else {
                assert_eq!(
                    d.status(),
                    Some(Status::Truncated(mtg_core::trajectory::Limit::Decisions))
                );
            }
            // Limit/quarantine finalization is an explicit owner effect, never an
            // accepted input. Subsequent rejects preserve the entire finalized owner.
            assert_eq!(d.privileged_snapshot(), snapshot);
            assert_eq!(d.privileged_history(), history);
            unchanged(&mut d, Seat::P1, ctx, &keep1);
            let concession =
                serde_json::to_vec(&record(Seat::P1, "concession", json!([{"kind":"concede"}])))
                    .unwrap();
            let ctx = RecordContext::Concession {
                episode: d.episode_id().unwrap(),
            };
            unchanged(&mut d, Seat::P1, ctx, &concession);
            let result = d.finish().unwrap();
            assert_eq!(result.accepted_decisions(), 1);
            assert_eq!(d.accounting().failed + d.accounting().truncated, 1);
            assert!(d.finish().is_err());
        }
    }
}

#[test]
fn records_concession_either_seat_requires_matching_live_context() {
    for capture in [false, true] {
        for bounded in [false, true] {
            for seat in [Seat::P0, Seat::P1] {
                let mut d = fresh(capture, bounded, Limits::default(), 1000);
                settle(&mut d);
                let bytes =
                    serde_json::to_vec(&record(seat, "concession", json!([{"kind":"concede"}])))
                        .unwrap();
                let decision = context(&d, Seat::P0);
                unchanged(&mut d, seat, decision, &bytes);
                let mut other = fresh(false, false, Limits::default(), 1000);
                settle(&mut other);
                unchanged(
                    &mut d,
                    seat,
                    RecordContext::Concession {
                        episode: other.episode_id().unwrap(),
                    },
                    &bytes,
                );
                let ctx = RecordContext::Concession {
                    episode: d.episode_id().unwrap(),
                };
                let keep = serde_json::to_vec(&record(
                    Seat::P0,
                    "keep_or_mulligan",
                    json!([{"kind":"keep"}]),
                ))
                .unwrap();
                unchanged(&mut d, Seat::P0, ctx, &keep);
                let mut extra: Value = serde_json::from_slice(&bytes).unwrap();
                extra["choices"][0]["index"] = json!(0);
                unchanged(&mut d, seat, ctx, &serde_json::to_vec(&extra).unwrap());
                d.submit_record(seat, ctx, &bytes).unwrap();
                unchanged(&mut d, seat, ctx, &bytes);
                let r = d.finish().unwrap();
                assert_eq!(r.accepted_decisions(), 0);
                assert_eq!(r.privileged_history().len(), 1);
                assert_eq!(
                    serde_json::from_slice::<Value>(&r.privileged_history()[0]).unwrap(),
                    serde_json::from_slice::<Value>(&bytes).unwrap()
                );
                if let Some(e) = r.trajectory() {
                    let rewards = if seat == Seat::P0 { [-1, 1] } else { [1, -1] };
                    assert_eq!(e.footer().unwrap().returns, rewards);
                    assert_eq!(e.footer().unwrap().boundary_reward, rewards);
                    assert!(e.decisions().is_empty());
                    for (index, s) in [Seat::P0, Seat::P1].into_iter().enumerate() {
                        let seq = e.seat(s).unwrap();
                        assert_eq!(seq.unassigned_reward, rewards[index]);
                        assert_eq!(seq.total_return, rewards[index]);
                        assert!(seq.transitions.is_empty());
                    }
                }
                assert_eq!(d.accounting().completed, 1);
                assert!(d.finish().is_err());
            }
        }
    }
}
