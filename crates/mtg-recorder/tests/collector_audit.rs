//! GH-154 independent normal-reset ledger. CR 103.5, 103.8a, 104.3c,
//! 117.3/117.4, 504, 508.8 and 514.1; doc/views.md and policy-decisions.md.
//! No game output supplies hand, domain, mask, step, count, reward or boundary
//! expectations. Only opaque decision revision/generation tokens are bound from
//! the current endpoint; they are not rules state or an expected-result oracle.
use mtg_core::{
    episode::{Driver, EpisodeResult, Progress, replay::Registry},
    game::{
        Config, DeckConfig,
        policy::{Choice, Submission, VisibleRef, VisibleZone},
    },
    objects::Seat,
    trajectory::Limits,
};
use mtg_recorder::{
    Backpressure,
    collector::{Run, Storage},
    manifest::{LoadMode, Manifest},
    publication::{self, Destination},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    num::NonZeroUsize,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
const RUN: &str = "ab345678-1234-4234-8234-123456789abc";
const REPLAY: &str = "cd345678-1234-4234-8234-123456789abc";
const MAX: usize = 100_000_000;
fn order() -> Vec<String> {
    [
        ("forest", 16),
        ("bear-cub", 4),
        ("giant-growth", 3),
        ("bite-down", 3),
        ("llanowar-elves", 3),
        ("druid-of-the-cowl", 2),
        ("magnigoth-sentry", 2),
        ("tajuru-pathwarden", 2),
        ("thornweald-archer", 3),
        ("wildheart-invoker", 2),
    ]
    .into_iter()
    .flat_map(|(s, n)| vec![s.to_owned(); n])
    .collect()
}
fn run(starting: u8) -> Run {
    Run {
        id: RUN.into(),
        config: Config {
            starting_seat: starting,
            seats: vec![
                DeckConfig {
                    deck: "green".into(),
                    order: Some(order())
                };
                2
            ],
            ..Config::default()
        },
        policies: ["literal-ledger".into(), "literal-ledger".into()],
        limits: Limits::default(),
        first_ordinal: 0,
        started: 1,
    }
}
fn seat(s: usize) -> Seat {
    if s == 0 { Seat::P0 } else { Seat::P1 }
}
fn hand(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Hand,
        row,
    }
}
// Frozen card manifest: Cub is 2/2 and Sentry is 4/4. The documented M1
// view exposes these implemented characteristics even outside the battlefield.
fn card(key: &str, s: usize) -> Value {
    json!({"card":key,"owner":s,"controller":s,"tapped":false,
        "creature":match key {"bear-cub"=>json!([2,2,0]),"magnigoth-sentry"=>json!([4,4,0]),_=>Value::Null},"summoning_sick":false})
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
fn ready(d: &mut Driver) {
    for _ in 0..1000 {
        if d.advance(NonZeroUsize::MAX).unwrap() != Progress::InternalYield {
            return;
        }
    }
    panic!("work bound")
}
struct Ledger {
    start: usize,
    hands: [Vec<String>; 2],
    drawn: [usize; 2],
    graves: [Vec<Value>; 2],
    remembered: Vec<Value>,
    rows: Vec<Value>,
    submissions: Vec<Submission>,
}
impl Ledger {
    fn new(start: usize) -> Self {
        Self {
            start,
            hands: [order()[..7].to_vec(), order()[..7].to_vec()],
            drawn: [7; 2],
            graves: Default::default(),
            remembered: vec![],
            rows: vec![],
            submissions: vec![],
        }
    }
    fn draw(&mut self, s: usize) {
        self.hands[s].push(order()[self.drawn[s]].clone());
        self.drawn[s] += 1;
        self.hands[s].sort();
    }
    fn discard(&mut self, s: usize) {
        let key = self.hands[s].remove(0);
        self.graves[s].push(card(&key, s));
        self.remembered
            .push(json!({"card":key,"owner":s,"zone_at_reveal":format!("graveyard_{s}")}));
    }
    fn observation(
        &self,
        s: usize,
        turn: Option<(u64, usize, &str)>,
        actor: Option<usize>,
        kind: &str,
        (revision, generation): (u64, u64),
        terminal: bool,
    ) -> Value {
        let opening = kind == "keep_or_mulligan";
        let own = actor == Some(s) && !terminal;
        let mut candidates = vec![];
        let mut mask = vec![];
        if own {
            if opening {
                candidates = vec![json!({"kind":"keep"}), json!({"kind":"mulligan"})];
                mask = vec![true, true];
            } else if kind == "cleanup_discard" {
                for row in 0..self.hands[s].len() {
                    candidates.push(json!({"kind":"discard","card":{"zone":"hand","row":row}}));
                    mask.push(true);
                }
            } else {
                candidates.push(json!({"kind":"pass"}));
                mask.push(true);
                for (row, key) in self.hands[s].iter().enumerate() {
                    candidates.push(json!({"kind":"play_land","card":{"zone":"hand","row":row}}));
                    mask.push(
                        key == "forest"
                            && turn.is_some_and(|(_, a, p)| {
                                a == s && (p == "precombat_main" || p == "postcombat_main")
                            }),
                    );
                }
                for (row, key) in self.hands[s].iter().enumerate() {
                    if ["bear-cub", "giant-growth", "bite-down"].contains(&key.as_str()) {
                        candidates.push(json!({"kind":"cast","card":{"zone":"hand","row":row}}));
                        mask.push(false);
                    }
                }
            }
        }
        let losses = if self.start == 0 {
            json!([null, "empty_draw"])
        } else {
            json!(["empty_draw", null])
        };
        json!({"schema_version":1,"view":{"schema_version":1,"seat":s,"life":[20,20],
            "hand_counts":[self.hands[0].len(),self.hands[1].len()],"library_counts":[40-self.drawn[0],40-self.drawn[1]],
            "hand":self.hands[s].iter().map(|k|card(k,s)).collect::<Vec<_>>(),
            "public_zones":[{"zone":"graveyard_0","cards":self.graves[0]},{"zone":"graveyard_1","cards":self.graves[1]},
              {"zone":"battlefield","cards":[]},{"zone":"stack","cards":[]},{"zone":"exile","cards":[]}],
            "remembered":self.remembered,"starting_seat":self.start,"turn":turn,"mana":([[0;6];2]),"acting_seat":actor,
            "opening":if opening&&own {json!({"generation":generation,"kind":kind,"count":1,"candidates":["keep","mulligan"]})} else {Value::Null},
            "terminal":if terminal {json!({"winner":self.start,"losses":losses})}else{Value::Null}},
            "decision":if own {json!({"revision":revision,"generation":generation,"actor":s,"kind":kind,"count":1,"candidates":candidates,"legal_mask":mask,"factored":null})}else{Value::Null},
            "pending":null,"stack":[],"combat":[],"unsupported_families":[]})
    }
    fn send(
        &mut self,
        on: &mut Driver,
        off: &mut Driver,
        s: usize,
        turn: Option<(u64, usize, &str)>,
        kind: &str,
        choice: Choice,
    ) {
        let actual = on.observe(seat(s)).unwrap();
        let d = actual.decision.as_ref().unwrap();
        let expected = self.observation(s, turn, Some(s), kind, (d.revision, d.generation), false);
        assert_eq!(
            serde_json::to_value(&actual).unwrap(),
            expected,
            "independent row {}",
            self.rows.len()
        );
        assert_eq!(actual, off.observe(seat(s)).unwrap());
        assert_eq!(
            serde_json::to_value(on.observe(seat(1 - s)).unwrap()).unwrap(),
            self.observation(
                1 - s,
                turn,
                Some(s),
                kind,
                (d.revision, d.generation),
                false
            )
        );
        let sub = Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![choice],
        };
        on.submit(seat(s), &sub).unwrap();
        off.submit(seat(s), &sub).unwrap();
        assert_eq!(
            normalized(&on.privileged_snapshot()),
            normalized(&off.privileged_snapshot())
        );
        self.rows.push(expected);
        self.submissions.push(sub);
    }
}
fn played(start: usize) -> (Run, EpisodeResult, Ledger) {
    let r = run(start as u8);
    let mut on = Driver::new(256).unwrap();
    let mut off = Driver::new(256).unwrap();
    on.reset_captured(&r.config, 154, 0, NonZeroUsize::MAX, &r.header(0).unwrap())
        .unwrap();
    off.reset(&r.config, 154, 0, NonZeroUsize::MAX).unwrap();
    ready(&mut on);
    ready(&mut off);
    let mut l = Ledger::new(start);
    for s in [start, 1 - start] {
        l.send(&mut on, &mut off, s, None, "keep_or_mulligan", Choice::Keep);
    }
    ready(&mut on);
    ready(&mut off);
    for turn in 1..=68 {
        let active = if turn % 2 == 1 { start } else { 1 - start };
        for step in [
            "upkeep",
            "draw",
            "precombat_main",
            "beginning_combat",
            "declare_attackers",
            "end_combat",
            "postcombat_main",
            "end",
        ] {
            if turn == 1 && step == "draw" {
                continue;
            }
            if step == "draw" {
                l.draw(active);
            }
            for s in [active, 1 - active] {
                l.send(
                    &mut on,
                    &mut off,
                    s,
                    Some((turn, active, step)),
                    "priority",
                    Choice::Pass,
                );
            }
            if turn == 68 {
                break;
            }
        }
        if turn == 68 {
            break;
        }
        if turn > 1 {
            l.send(
                &mut on,
                &mut off,
                active,
                Some((turn, active, "cleanup")),
                "cleanup_discard",
                Choice::Discard { card: hand(0) },
            );
            l.discard(active);
        }
    }
    assert_eq!(l.rows.len(), 1140); // 2 keeps + 14 passes + 66*(16 passes+discard) + 2 passes.
    // GH-20: terminal polling is not another transition or reward event.
    let ended = on.trajectory().unwrap().clone();
    let snapshot = on.privileged_snapshot();
    let history = on.privileged_history().to_vec();
    let outcome = match on.status().unwrap() {
        mtg_core::episode::Status::Completed(outcome) => outcome,
        other => panic!("expected rules completion, got {other:?}"),
    };
    for quantum in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
        for _ in 0..3 {
            // doc/episode-driver.md: advancement after terminal is rejected.
            assert_eq!(on.advance(quantum), Err(mtg_core::episode::Error::Ended));
            assert_eq!(
                on.status(),
                Some(mtg_core::episode::Status::Completed(outcome))
            );
            assert_eq!(on.trajectory(), Some(&ended));
            assert_eq!(on.privileged_snapshot(), snapshot);
            assert_eq!(on.privileged_history(), history);
        }
    }
    let result = on.finish().unwrap();
    assert!(on.finish().is_err());
    assert!(off.finish().unwrap().trajectory().is_none());
    assert_eq!(
        normalized(result.privileged_snapshot()),
        normalized(&off.privileged_snapshot())
    );
    let e = result.trajectory().unwrap();
    assert_eq!(e.decisions().len(), 1140);
    let rewards = if start == 0 { [1, -1] } else { [-1, 1] };
    let mut counts = [0, 0];
    for (i, d) in e.decisions().iter().enumerate() {
        let actor = l.rows[i]["view"]["seat"].as_u64().unwrap() as usize;
        assert_eq!(
            (d.index, d.actor, d.seat_index),
            (i, actor as u8, counts[actor])
        );
        counts[actor] += 1;
        assert_eq!(serde_json::to_value(&d.observation).unwrap(), l.rows[i]);
        assert_eq!(d.choice.submission, l.submissions[i]);
        assert_eq!(
            (d.choice.logical_action, d.choice.micro_choice),
            (i as u64, 0)
        );
        assert_eq!(
            d.choice.status,
            mtg_core::trajectory::v2::ActionStatus::Committed
        );
        assert_eq!(d.choice.policy, Default::default());
        assert_eq!(d.terminated, i == 1139);
        assert!(!d.truncated);
        assert_eq!(d.reward, if i == 1139 { rewards } else { [0, 0] });
        assert_eq!(
            d.next_actor,
            l.rows
                .get(i + 1)
                .map(|v| v["view"]["seat"].as_u64().unwrap() as u8)
        );
    }
    let f = e.footer().unwrap();
    assert_eq!(f.returns, rewards);
    assert_eq!(f.boundary_reward, [0, 0]);
    assert_eq!(f.logical_actions, 1140);
    assert_eq!(f.cancelled_actions, 0);
    assert_eq!(f.end, mtg_core::trajectory::End::Completed);
    assert!(f.complete);
    assert_eq!(f.decisions, 1140);
    for (s, reward) in rewards.iter().enumerate() {
        assert_eq!(
            serde_json::to_value(&f.final_observations[s]).unwrap(),
            l.observation(s, Some((68, 1 - start, "draw")), None, "", (0, 0), true)
        );
        let seq = e.seat(seat(s)).unwrap();
        assert_eq!(seq.total_return, *reward);
        assert_eq!(seq.unassigned_reward, 0);
        for (i, t) in seq.transitions.iter().enumerate() {
            let next = seq.transitions.get(i + 1);
            let end = next.map_or(1140, |n| n.decision_index);
            assert_eq!(t.next_decision, next.map(|n| n.decision_index));
            assert_eq!(t.decisions_elapsed, end - t.decision_index);
            assert_eq!(t.logical_actions_elapsed, end - t.decision_index);
            assert_eq!(
                serde_json::to_value(&t.next_observation).unwrap(),
                next.map_or_else(
                    || serde_json::to_value(&f.final_observations[s]).unwrap(),
                    |n| l.rows[n.decision_index].clone()
                )
            );
            assert_eq!(t.reward, if next.is_none() { rewards[s] } else { 0 });
            assert_eq!(t.terminated, next.is_none());
            assert!(!t.truncated);
        }
    }
    let owned = e.clone();
    on.reset(&r.config, 155, 1, NonZeroUsize::MAX).unwrap();
    assert!(on.submit(seat(start), &l.submissions[0]).is_err());
    assert_eq!(result.trajectory(), Some(&owned));
    (r, result, l)
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static N: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "mtg-audit-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        for name in ["data", "private"] {
            fs::create_dir(p.join(name)).unwrap();
        }
        Self(p)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn storage() -> Storage {
    Storage {
        queue_bytes: MAX,
        max_bytes: MAX,
        backpressure: Backpressure::Block,
    }
}
fn publish_check(r: &Run, result: &mut EpisodeResult) -> (Temp, Manifest, Vec<u8>) {
    let t = Temp::new();
    let mut registry = Registry::default();
    registry.register(REPLAY, result).unwrap();
    publication::publish(
        r,
        std::slice::from_ref(result),
        &registry,
        |id, key| id == REPLAY && key.run == RUN,
        Destination {
            datasets: &t.0.join("data"),
            replays: &t.0.join("private"),
        },
        storage(),
    )
    .unwrap();
    let dir = t.0.join("data").join(RUN);
    let m = Manifest::parse(fs::File::open(dir.join("manifest.json")).unwrap(), MAX).unwrap();
    let bytes = fs::read(dir.join("episodes.jsonl")).unwrap();
    assert_eq!(m.file.bytes, bytes.len() as u64);
    assert_eq!(m.file.sha256, format!("{:x}", Sha256::digest(&bytes)));
    assert_eq!(m.file.decisions, result.accepted_decisions());
    assert!(m.recording_complete);
    let data = m
        .load_v2(
            "episodes.jsonl",
            bytes.as_slice(),
            &m.versions,
            MAX,
            LoadMode::CompletedOnly,
        )
        .unwrap();
    assert_eq!(
        data.episodes(),
        &[mtg_recorder::from_core_v2(result.trajectory().unwrap()).unwrap()]
    );
    assert!(
        publication::read_replay(&t.0.join("private"), REPLAY, result, |_, _| false, MAX).is_err()
    );
    let replay = publication::read_replay(
        &t.0.join("private"),
        REPLAY,
        result,
        |id, key| id == REPLAY && key.ordinal == 0,
        MAX,
    )
    .unwrap();
    let game = mtg_core::opening::replay::played::verify(&replay).unwrap();
    assert_eq!(
        normalized(&game.snapshot()),
        normalized(result.privileged_snapshot())
    );
    (t, m, bytes)
}
#[test]
fn both_starting_seats_full_independent_ledger_publish_reload_and_replay() {
    for start in 0..2 {
        let (r, mut result, ledger) = played(start);
        let (_t, m, bytes) = publish_check(&r, &mut result);
        let data = m
            .load_v2(
                "episodes.jsonl",
                bytes.as_slice(),
                &m.versions,
                MAX,
                LoadMode::CompletedOnly,
            )
            .unwrap();
        let episode = &data.episodes()[0];
        let observations = episode
            .decisions
            .iter()
            .map(|d| serde_json::to_value(&d.observation).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(observations, ledger.rows);
        // Prove the independent ledger detects omitted/extra decisions and wrong
        // checkpoints, rather than accepting a regenerated implementation trace.
        let mut missing = observations.clone();
        missing.remove(7);
        assert_ne!(missing, ledger.rows);
        let mut extra = observations.clone();
        extra.push(extra[0].clone());
        assert_ne!(extra, ledger.rows);
        let mut wrong = observations.clone();
        wrong[7]["view"]["life"][0] = json!(19);
        assert_ne!(wrong, ledger.rows);
        wrong = observations.clone();
        wrong[7]["decision"]["legal_mask"][0] = json!(false);
        assert_ne!(wrong, ledger.rows);
        for duplicate in [false, true] {
            let mut corrupt = episode.clone();
            if duplicate {
                corrupt.decisions.insert(7, corrupt.decisions[7].clone());
            } else {
                corrupt.decisions.remove(7);
            }
            let mut writer =
                mtg_recorder::Writer::new_v2(Vec::new(), MAX, Backpressure::Block).unwrap();
            assert!(writer.append_v2(&corrupt).is_err());
        }
        let mut bad_versions = m.versions.clone();
        bad_versions.observation = 99;
        assert!(
            m.load_v2(
                "episodes.jsonl",
                bytes.as_slice(),
                &bad_versions,
                MAX,
                LoadMode::CompletedOnly
            )
            .is_err()
        );
        let mut corrupt = bytes.clone();
        corrupt[0] = b'!';
        assert!(
            m.load_v2(
                "episodes.jsonl",
                corrupt.as_slice(),
                &m.versions,
                MAX,
                LoadMode::CompletedOnly
            )
            .is_err()
        );
    }
}

// Catalog rules-terminal-rewards-once-regression; RFC 0002 B036/B037.
// A normal-reset CR 104.3c win, not an edited terminal test position.
#[test]
fn restored_terminal_polling_preserves_once_only_persisted_rewards() {
    for start in 0..2 {
        let (r, mut result, _) = played(start);
        let mut restored = mtg_core::game::Game::new().unwrap();
        restored.restore(result.privileged_snapshot()).unwrap();
        let expected = mtg_core::game::terminal::Outcome {
            winner: Some(seat(start)),
            losses: if start == 0 {
                [None, Some(mtg_core::game::terminal::LossReason::EmptyDraw)]
            } else {
                [Some(mtg_core::game::terminal::LossReason::EmptyDraw), None]
            },
        };
        let snapshot = restored.snapshot();
        let views = [Seat::P0, Seat::P1].map(|s| restored.policy_observe(s, 256).unwrap());
        assert_eq!(&views, result.final_observations().unwrap());
        for quantum in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
            for _ in 0..3 {
                assert_eq!(restored.outcome(), Some(expected));
                assert_eq!(
                    restored.resume(quantum),
                    mtg_core::game::Progress::Terminal(expected)
                );
                assert_eq!(restored.snapshot(), snapshot);
                for (s, view) in views.iter().enumerate() {
                    assert_eq!(&restored.policy_observe(seat(s), 256).unwrap(), view);
                    assert!(view.decision.is_none());
                }
                assert_eq!(
                    restored.concede(seat(start), restored.episode_id().unwrap()),
                    Err(mtg_core::game::terminal::ConcedeError::AlreadyEnded)
                );
            }
        }
        let (_temp, manifest, bytes) = publish_check(&r, &mut result);
        for _ in 0..3 {
            let data = manifest
                .load_v2(
                    "episodes.jsonl",
                    bytes.as_slice(),
                    &manifest.versions,
                    MAX,
                    LoadMode::CompletedOnly,
                )
                .unwrap();
            let episode = &data.episodes()[0];
            assert_eq!(episode.decisions.len(), 1140);
            // A syntactically valid second terminal credit must be rejected by
            // semantic validation even before checksums become relevant.
            let mut duplicate = episode.clone();
            duplicate.footer.as_mut().unwrap().boundary_reward =
                if start == 0 { [1, -1] } else { [-1, 1] };
            assert!(mtg_recorder::structured::validate(&duplicate).is_err());
            for s in 0..2 {
                let reward = if s == start { 1 } else { -1 };
                assert_eq!(
                    episode
                        .decisions
                        .iter()
                        .map(|d| i64::from(d.reward[s]))
                        .sum::<i64>(),
                    reward
                );
                assert_eq!(episode.footer.as_ref().unwrap().boundary_reward[s], 0);
                assert_eq!(
                    i64::from(episode.footer.as_ref().unwrap().returns[s]),
                    reward
                );
                let sequence = result.trajectory().unwrap().seat(seat(s)).unwrap();
                assert_eq!(
                    sequence
                        .transitions
                        .iter()
                        .map(|t| i64::from(t.reward))
                        .sum::<i64>(),
                    reward
                );
                assert_eq!(sequence.unassigned_reward, 0);
            }
        }
    }
}

// DRL-005: reopen actual published bytes in another process. The parent grants
// replay export separately; no default trajectory reader resolves private IDs.
#[test]
fn canonical_trajectory_and_authorized_replay_reload_in_fresh_process() {
    const CHILD: &str = "MTG_GH20_RELOAD_ROOT";
    if let Some(root) = std::env::var_os(CHILD) {
        let literal: Vec<mtg_recorder::structured::Episode> =
            serde_json::from_str(include_str!("structured.json")).unwrap();
        assert_eq!(
            mtg_recorder::read_v2(include_bytes!("structured.jsonl").as_slice(), MAX).unwrap(),
            literal
        );
        let root = PathBuf::from(root);
        let manifest = Manifest::parse(
            fs::File::open(root.join("data").join(RUN).join("manifest.json")).unwrap(),
            MAX,
        )
        .unwrap();
        let bytes = fs::read(root.join("data").join(RUN).join("episodes.jsonl")).unwrap();
        let data = manifest
            .load_v2(
                "episodes.jsonl",
                bytes.as_slice(),
                &manifest.versions,
                MAX,
                LoadMode::CompletedOnly,
            )
            .unwrap();
        let game = mtg_core::opening::replay::played::verify(
            &fs::read(root.join("authorized-replay")).unwrap(),
        )
        .unwrap();
        fs::write(
            root.join("reloaded.json"),
            serde_json::to_vec(
                &json!({"episodes":data.episodes(),"state":normalized(&game.snapshot())}),
            )
            .unwrap(),
        )
        .unwrap();
        return;
    }
    for start in 0..2 {
        let (r, mut result, _) = played(start);
        let (temp, _, _) = publish_check(&r, &mut result);
        let replay = publication::read_replay(
            &temp.0.join("private"),
            REPLAY,
            &result,
            |id, key| id == REPLAY && key.run == RUN,
            MAX,
        )
        .unwrap();
        fs::write(temp.0.join("authorized-replay"), replay).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "canonical_trajectory_and_authorized_replay_reload_in_fresh_process",
                "--nocapture",
            ])
            .env(CHILD, &temp.0)
            .env_remove("DISPLAY")
            .stdin(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("fresh-process reload exceeded 30 seconds");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let loaded: Value =
            serde_json::from_slice(&fs::read(temp.0.join("reloaded.json")).unwrap()).unwrap();
        assert_eq!(
            loaded["episodes"],
            json!([mtg_recorder::from_core_v2(result.trajectory().unwrap()).unwrap()])
        );
        assert_eq!(loaded["state"], normalized(result.privileged_snapshot()));
    }
}
// Re-execute the original real combat script and every original assertion;
// no replacement engine, synthetic state, output snapshot oracle, or skipped test.
#[path = "../../mtg-core/tests/capture.rs"]
mod capture;
#[test]
fn played_target_payment_cancel_and_combat_survive_publication() {
    let mut r = run(0);
    r.config = capture::config();
    let mut result = capture::played_combat(r.header(0).unwrap());
    let (_temp, _manifest, _bytes) = publish_check(&r, &mut result);
}

#[test]
fn zero_decision_and_nonacting_rewards_survive_publication_for_both_losers() {
    for loser in 0..2 {
        for decisions in 0..=1 {
            let r = run(0);
            let mut d = Driver::new(256).unwrap();
            d.reset_captured(&r.config, 154, 0, NonZeroUsize::MAX, &r.header(0).unwrap())
                .unwrap();
            ready(&mut d);
            if decisions == 1 {
                let domain = d.observe(Seat::P0).unwrap().decision.unwrap();
                d.submit(
                    Seat::P0,
                    &Submission {
                        schema_version: 1,
                        revision: domain.revision,
                        generation: domain.generation,
                        choices: vec![Choice::Keep],
                    },
                )
                .unwrap();
            }
            d.concede(seat(loser), d.episode_id().unwrap()).unwrap();
            let mut result = d.finish().unwrap();
            assert!(d.finish().is_err());
            let (_temp, m, bytes) = publish_check(&r, &mut result);
            let data = m
                .load_v2(
                    "episodes.jsonl",
                    bytes.as_slice(),
                    &m.versions,
                    MAX,
                    LoadMode::CompletedOnly,
                )
                .unwrap();
            let e = &data.episodes()[0];
            assert_eq!(e.decisions.len(), decisions);
            let rewards = if loser == 0 { [-1, 1] } else { [1, -1] };
            assert_eq!(e.footer.as_ref().unwrap().returns, rewards);
            assert_eq!(e.footer.as_ref().unwrap().boundary_reward, rewards);
            for (s, reward) in rewards.into_iter().enumerate() {
                let seq = result.trajectory().unwrap().seat(seat(s)).unwrap();
                assert_eq!(seq.total_return, reward);
                assert_eq!(
                    seq.unassigned_reward,
                    if s == 0 && decisions == 1 { 0 } else { reward }
                );
                assert_eq!(
                    seq.transitions.iter().map(|t| t.reward).sum::<i8>() + seq.unassigned_reward,
                    reward
                );
            }
        }
    }
}

#[test]
fn ordered_mulligan_choices_survive_aggregate_publication_for_both_starting_seats() {
    // CR 103.5: two mulligans leave five cards, with two explicitly ordered
    // bottom selections. The policy contract exposes bottoming before the next
    // keep/mulligan prompt. No shuffled card identities are used as an oracle.
    for start in 0..2 {
        let r = run(start as u8);
        let mut d = Driver::new(256).unwrap();
        d.reset_captured(&r.config, 117, 0, NonZeroUsize::MAX, &r.header(0).unwrap())
            .unwrap();
        let script = [
            (start, "keep_or_mulligan", vec![Choice::Mulligan]),
            (1 - start, "keep_or_mulligan", vec![Choice::Keep]),
            (start, "bottom", vec![Choice::Bottom { card: hand(0) }]),
            (start, "keep_or_mulligan", vec![Choice::Mulligan]),
            (
                start,
                "bottom",
                vec![
                    Choice::Bottom { card: hand(5) },
                    Choice::Bottom { card: hand(1) },
                ],
            ),
            (start, "keep_or_mulligan", vec![Choice::Keep]),
        ];
        let mut submissions = Vec::new();
        for (i, (actor, kind, choices)) in script.iter().enumerate() {
            ready(&mut d);
            let domain = d.observe(seat(*actor)).unwrap().decision.unwrap();
            assert_eq!(domain.kind, *kind);
            assert_eq!(domain.actor, *actor as u8);
            assert_eq!(domain.count, if i == 4 { 2 } else { 1 });
            if *kind == "bottom" {
                assert_eq!(
                    domain.candidates,
                    (0..7)
                        .map(|row| Choice::Bottom { card: hand(row) })
                        .collect::<Vec<_>>()
                );
                assert_eq!(domain.legal_mask, vec![true; 7]);
            }
            let submission = Submission {
                schema_version: 1,
                revision: domain.revision,
                generation: domain.generation,
                choices: choices.clone(),
            };
            if i == 4 {
                let before = d.privileged_snapshot();
                let mut missing = submission.clone();
                missing.choices.pop();
                assert!(d.submit(seat(*actor), &missing).is_err());
                assert_eq!(d.privileged_snapshot(), before);
                assert_eq!(d.trajectory().unwrap().decisions().len(), 4);
            }
            d.submit(seat(*actor), &submission).unwrap();
            submissions.push(submission);
        }
        ready(&mut d);
        let mut counts = [7, 7];
        counts[start] = 5;
        assert_eq!(d.observe(seat(start)).unwrap().view.hand_counts, counts);
        d.concede(seat(1 - start), d.episode_id().unwrap()).unwrap();
        let mut result = d.finish().unwrap();
        let owned = result.trajectory().unwrap().clone();
        d.reset(&r.config, 118, 1, NonZeroUsize::MAX).unwrap();
        assert_eq!(result.trajectory(), Some(&owned));
        let (_temp, m, bytes) = publish_check(&r, &mut result);
        let data = m
            .load_v2(
                "episodes.jsonl",
                bytes.as_slice(),
                &m.versions,
                MAX,
                LoadMode::CompletedOnly,
            )
            .unwrap();
        let e = &data.episodes()[0];
        assert_eq!(e.decisions.len(), 6);
        for (i, row) in e.decisions.iter().enumerate() {
            assert_eq!(row.actor, script[i].0 as u8);
            assert_eq!(row.index, i);
            assert_eq!(
                serde_json::to_value(&row.choice.submission).unwrap(),
                serde_json::to_value(&submissions[i]).unwrap()
            );
            assert_eq!(row.choice.logical_action, i as u64);
            assert_eq!(row.choice.micro_choice, 0);
            assert_eq!(
                serde_json::to_value(&row.choice.policy).unwrap(),
                json!({}) // doc/trajectory-jsonl.md: absent v2 statistics omit keys.
            );
            assert_eq!(row.reward, [0, 0]);
            // External concession marks the last global row's boundary;
            // its reward remains solely in the footer (doc/trajectories.md).
            assert_eq!(row.terminated, i == 5);
            assert!(!row.truncated);
        }
        let rewards = if start == 0 { [1, -1] } else { [-1, 1] };
        assert_eq!(e.footer.as_ref().unwrap().returns, rewards);
        assert_eq!(e.footer.as_ref().unwrap().boundary_reward, rewards);
        assert_eq!(e.footer.as_ref().unwrap().decisions, 6);
    }
}
