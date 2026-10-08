//! Independent CR 103, 117, 305, 601, 608, 508–510, 514 expectations.
//! Full frozen green decks; directed library order, normal reset (no synthetic state).
use mtg_core::{
    objects::{Seat, Zone},
    opening::{
        actions,
        replay::{ReplayError, played},
        turns::{Step, TurnKind},
        *,
    },
};
use serde_json::{Value, json};
const CAP: usize = 256;
static REPLAY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn config() -> Config {
    let order = [
        ("bear-cub", 2),
        ("giant-growth", 1),
        ("bite-down", 1),
        ("forest", 16),
        ("bear-cub", 2),
        ("giant-growth", 2),
        ("bite-down", 2),
        ("llanowar-elves", 3),
        ("druid-of-the-cowl", 2),
        ("magnigoth-sentry", 2),
        ("tajuru-pathwarden", 2),
        ("thornweald-archer", 3),
        ("wildheart-invoker", 2),
    ]
    .into_iter()
    .flat_map(|(k, n)| vec![k.to_owned(); n])
    .collect();
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
fn reference(s: Seat, n: u64, key: &str, zone: Zone, incarnation: u64) -> Value {
    json!({"birth":n+if s==Seat::P0 {0} else {40},"card":key,"owner":s,"zone":zone,"incarnation":incarnation})
}
fn hand(s: Seat, n: u64, key: &str) -> Value {
    reference(s, n, key, Zone::Hand(s), 1)
}
fn permanent(s: Seat, n: u64, key: &str) -> Value {
    reference(
        s,
        n,
        key,
        Zone::Battlefield,
        if key == "forest" { 2 } else { 3 },
    )
}
struct Script {
    g: Game,
    choices: Vec<actions::Record>,
    marks: Vec<(&'static str, usize)>,
}
impl Script {
    fn new() -> Self {
        let mut g = Game::new().unwrap();
        g.reset(&config(), 114, 0).unwrap();
        Self {
            g,
            choices: vec![],
            marks: vec![],
        }
    }
    fn send(&mut self, s: Seat, kind: &str, cs: Value) {
        let v = json!({"version":1,"actor":s,"decision":kind,"choices":cs});
        let b = serde_json::to_vec(&v).unwrap();
        actions::apply(&mut self.g, &b, CAP)
            .unwrap_or_else(|e| panic!("{e:?} {v} at {:?}", self.g.turn_position()));
        self.choices.push(serde_json::from_value(v).unwrap());
        if self.g.decision().is_none()
            && self.g.turn_position().is_none()
            && self.g.outcome().is_none()
        {
            self.g.start_turns().unwrap();
        }
    }
    fn one(&mut self, s: Seat, k: &str, c: Value) {
        self.send(s, k, json!([c]));
    }
    fn mark(&mut self, n: &'static str) {
        self.marks.push((n, self.choices.len() - 1));
    }
    fn pass(&mut self) {
        let s = self.g.turn_decision().unwrap().actor;
        self.one(s, "priority", json!({"kind":"pass"}));
    }
    fn pair(&mut self) {
        self.pass();
        self.pass();
    }
    fn idle(&mut self) {
        let d = self.g.turn_decision().unwrap();
        match d.kind {
            TurnKind::TriggerOrder | TurnKind::TriggerTarget => {
                panic!("no trigger sources in this script")
            }
            TurnKind::Priority => self.pass(),
            TurnKind::Combat(_) => self.one(
                d.actor,
                self.g
                    .policy_observe(d.actor, CAP)
                    .unwrap()
                    .decision
                    .unwrap()
                    .kind,
                json!({"kind":"finish_combat"}),
            ),
            TurnKind::Discard { count } => {
                let hs = self
                    .g
                    .objects()
                    .in_zone(Zone::Hand(d.actor))
                    .take(count)
                    .collect::<Vec<_>>();
                let obs = self.g.policy_observe(d.actor, CAP).unwrap();
                let dec = obs.decision.unwrap();
                let cs = dec
                    .candidates
                    .into_iter()
                    .filter(|c| matches!(c, policy::Choice::Discard { .. }))
                    .take(hs.len())
                    .collect();
                let sub = policy::Submission {
                    schema_version: policy::SCHEMA_VERSION,
                    revision: dec.revision,
                    generation: dec.generation,
                    choices: cs,
                };
                let bytes = actions::encode(&self.g, d.actor, &sub, CAP).unwrap();
                let r: actions::Record = serde_json::from_slice(&bytes).unwrap();
                actions::apply(&mut self.g, &bytes, CAP).unwrap();
                self.choices.push(r);
            }
        }
    }
    fn until(&mut self, t: u64, step: Step) {
        for _ in 0..500 {
            if self
                .g
                .turn_position()
                .is_some_and(|p| p.0 == t && p.2 == step)
            {
                return;
            }
            self.idle();
        }
        panic!("script bound");
    }
    fn pay(&mut self, s: Seat, lands: &[u64]) {
        for &n in lands {
            self.one(
                s,
                "payment",
                json!({"kind":"tap_mana","card":permanent(s,n,"forest")}),
            );
        }
        for _ in lands {
            self.one(s, "payment", json!({"kind":"pay","color":4}));
        }
        self.one(s, "payment", json!({"kind":"finish_payment"}));
    }
    fn cub(&mut self, s: Seat) {
        self.one(
            s,
            "priority",
            json!({"kind":"cast","card":hand(s,0,"bear-cub")}),
        );
        self.pay(s, &[4, 5]);
        self.pair();
    }
    fn instant(&mut self, s: Seat, n: u64, key: &str, targets: &[(Seat, u64)]) {
        self.one(s, "priority", json!({"kind":"cast","card":hand(s,n,key)}));
        for &(owner, birth) in targets {
            let kind = self
                .g
                .policy_observe(s, CAP)
                .unwrap()
                .decision
                .unwrap()
                .kind;
            self.one(
                s,
                kind,
                json!({"kind":"target","card":permanent(owner,birth,"bear-cub")}),
            );
        }
        self.mark("targets");
        self.one(s, "targets_complete", json!({"kind":"finish_targets"}));
    }
}
fn script() -> Script {
    let mut s = Script::new();
    for seat in [Seat::P0, Seat::P1] {
        s.one(seat, "keep_or_mulligan", json!({"kind":"keep"}));
    }
    for turn in 1..=4 {
        let seat = if turn % 2 == 1 { Seat::P0 } else { Seat::P1 };
        s.until(turn, Step::PrecombatMain);
        s.one(
            seat,
            "priority",
            json!({"kind":"play_land","card":hand(seat,if turn<=2{4}else{5},"forest")}),
        );
        if turn >= 3 {
            s.cub(seat);
        }
    }
    s.until(5, Step::PrecombatMain);
    s.one(
        Seat::P0,
        "priority",
        json!({"kind":"play_land","card":hand(Seat::P0,6,"forest")}),
    );
    // Let P1 untap, then cast Bite during P1's main. P1 responds with Growth.
    s.until(6, Step::PrecombatMain);
    s.pass();
    s.instant(Seat::P0, 3, "bite-down", &[(Seat::P0, 0), (Seat::P1, 0)]);
    s.pay(Seat::P0, &[4, 5]);
    s.pass();
    s.instant(Seat::P1, 2, "giant-growth", &[(Seat::P1, 0)]);
    s.pay(Seat::P1, &[4]);
    s.mark("response");
    s.pair();
    s.mark("growth");
    s.pair();
    s.mark("bite");
    let cub =
        s.g.objects()
            .in_zone(Zone::Battlefield)
            .find(|h| {
                let o = s.g.objects().get(*h).unwrap();
                o.owner == Seat::P1 && o.card.identity().key == "bear-cub"
            })
            .unwrap();
    let c = s.g.creature_state(cub).unwrap();
    assert_eq!((c.power, c.toughness, c.damage), (5, 5, 2));
    s.until(7, Step::Upkeep);
    s.mark("cleanup");
    let c = s.g.creature_state(cub).unwrap();
    assert_eq!((c.power, c.toughness, c.damage), (2, 2, 0));
    // Ten unblocked Cub attacks: exact 2 damage each, P1 loses at zero (CR 510/704).
    for hit in 1..=10 {
        let turn = 5 + hit * 2;
        s.until(turn, Step::DeclareAttackers);
        s.one(
            Seat::P0,
            "attackers",
            json!({"kind":"select_attackers","cards":[permanent(Seat::P0,0,"bear-cub")]}),
        );
        s.one(Seat::P0, "attackers", json!({"kind":"finish_combat"}));
        s.pair();
        s.one(Seat::P1, "blockers", json!({"kind":"finish_combat"}));
        s.mark("combat");
        for _ in 0..20 {
            if s.g.life()[1] == 20 - 2 * hit as i64 {
                break;
            }
            s.idle();
        }
        assert_eq!(s.g.life(), [20, 20 - 2 * hit as i64]);
    }
    assert_eq!(s.g.outcome().unwrap().winner, Some(Seat::P0));
    s.mark("terminal");
    s
}
fn recording() -> (Value, Script) {
    let s = script();
    let b = played::record(&config(), 114, 0, &s.choices).unwrap();
    (serde_json::from_slice(&b).unwrap(), s)
}
#[test]
fn played_replay_normal_reset_independent_checkpoints() {
    let _guard = REPLAY_TEST_LOCK.lock().unwrap();
    let (v, s) = recording();
    let bytes = serde_json::to_vec(&v).unwrap();
    let g = played::verify(&bytes).unwrap();
    assert_eq!(g.life(), [20, 0]);
    assert_eq!(g.outcome(), s.g.outcome());
    for (name, i) in s.marks {
        let c = &v["choices"][i]["after"];
        match name {
            "response" => {
                assert_eq!(c["turns"]["stack"][0]["card"], "bite-down");
                assert_eq!(c["turns"]["stack"][1]["card"], "giant-growth");
                assert_eq!(c["turns"]["decision"]["actor"], "P1");
                assert_eq!(c["turns"]["mana"], json!(([[0u32; 6]; 2])));
                assert_eq!(
                    c["turns"]["effects"][0][1]["Bite"][0],
                    permanent(Seat::P0, 0, "bear-cub")
                );
                assert_eq!(
                    c["turns"]["effects"][0][1]["Bite"][1],
                    permanent(Seat::P1, 0, "bear-cub")
                );
            }
            "combat" => {
                assert_eq!(
                    c["turns"]["combat"]["attacks"][0]["creature"],
                    permanent(Seat::P0, 0, "bear-cub")
                );
                assert_eq!(c["turns"]["combat"]["attacks"][0]["blocked"], false);
                assert_eq!(c["turns"]["combat"]["attacks"][0]["blockers"], json!([]));
            }
            "growth" => assert_eq!(c["turns"]["modifications"][0]["boost"], 3),
            "bite" => assert_eq!(c["turns"]["modifications"][0]["damage"], 2),
            "cleanup" => {
                assert_eq!(c["turns"]["position"], json!([7, "P0", "Upkeep"]));
                assert_eq!(c["turns"]["modifications"], json!([]));
            }
            "terminal" => {
                assert_eq!(c["life"], json!([20, 0]));
                assert_eq!(c["outcome"]["losses"], json!([null, "Life"]));
            }
            _ => (),
        }
    }
    assert_eq!(
        bytes,
        serde_json::to_vec(
            &serde_json::from_slice::<Value>(
                &played::record(&config(), 114, 0, &s.choices).unwrap()
            )
            .unwrap()
        )
        .unwrap()
    );
}
#[test]
fn played_replay_strict_corruption() {
    let _guard = REPLAY_TEST_LOCK.lock().unwrap();
    let (v, s) = recording();
    let verify = |v: &Value| played::verify(&serde_json::to_vec(v).unwrap());
    let mut bad = v.clone();
    bad["choices"].as_array_mut().unwrap().pop();
    assert!(matches!(
        verify(&bad),
        Err(ReplayError::MissingChoice { .. })
    ));
    bad = v.clone();
    let extra = bad["choices"][0].clone();
    bad["choices"].as_array_mut().unwrap().push(extra);
    assert!(matches!(
        verify(&bad),
        Err(ReplayError::UnconsumedChoice { .. })
    ));
    for field in [
        "engine",
        "rules",
        "cards",
        "rng",
        "shuffle",
        "config_sha256",
    ] {
        bad = v.clone();
        bad[field] = json!("wrong");
        assert!(matches!(
            verify(&bad),
            Err(ReplayError::Incompatible { .. })
        ));
    }
    for field in ["version", "action_version"] {
        bad = v.clone();
        bad[field] = json!(999);
        assert!(matches!(
            verify(&bad),
            Err(ReplayError::Incompatible { .. })
        ));
    }
    for path in ["/life/0", "/turns/decision/actor"] {
        bad = v.clone();
        let after = &mut bad["choices"][2]["after"];
        *after.pointer_mut(path).unwrap() = if path == "/life/0" {
            json!(19)
        } else {
            json!("P0")
        };
        assert!(
            matches!(verify(&bad),Err(ReplayError::Divergence{checkpoint:3,path:p,..}) if p==path)
        );
    }
    let i = s.marks.iter().find(|(n, _)| *n == "targets").unwrap().1;
    bad = v.clone();
    bad["choices"][i]["after"]["turns"]["targeting"]["selected"][0]["birth"] = json!(1);
    assert!(matches!(verify(&bad),Err(ReplayError::Divergence{checkpoint,..}) if checkpoint==i+1));
    bad = v.clone();
    bad["initial"]["rng"]["state"] = json!(0);
    assert!(matches!(
        verify(&bad),
        Err(ReplayError::Divergence { checkpoint: 0, .. })
    ));
    bad = v.clone();
    bad["initial"]["zones"][0]["objects"][0]["identity"]["birth"] = json!(999);
    assert!(matches!(
        verify(&bad),
        Err(ReplayError::Divergence { checkpoint: 0, .. })
    ));
    // Swapping a legal same-name Cub changes identity, even if the final winner would agree.
    let cast = v["choices"]
        .as_array()
        .unwrap()
        .iter()
        .position(|r| r["choice"]["choices"][0]["kind"] == "cast")
        .unwrap();
    bad = v.clone();
    bad["choices"][cast]["choice"]["choices"][0]["card"]["birth"] = json!(1);
    assert!(
        matches!(verify(&bad),Err(ReplayError::Divergence{checkpoint,..}) if checkpoint==cast+1)
    );
    bad = v.clone();
    bad["choices"][cast]["choice"]["choices"][0]["card"]["incarnation"] = json!(999);
    assert!(matches!(verify(&bad),Err(ReplayError::SemanticChoice{index,..}) if index==cast));
    for field in ["master", "episode"] {
        bad = v.clone();
        bad[field] = json!(999);
        assert!(matches!(
            verify(&bad),
            Err(ReplayError::Divergence { checkpoint: 0, .. })
        ));
    }
    bad = v.clone();
    bad["choices"][0]["choice"]["choices"][0]["index"] = json!(0);
    assert!(matches!(verify(&bad), Err(ReplayError::Malformed)));
    bad = v.clone();
    bad["choices"][1]["after"]
        .as_object_mut()
        .unwrap()
        .remove("outcome");
    assert!(matches!(
        verify(&bad),
        Err(ReplayError::Divergence { checkpoint: 2, .. })
    ));
    bad = v.clone();
    bad["choices"][0].as_object_mut().unwrap().remove("after");
    assert!(verify(&bad).is_err());
    assert!(matches!(
        played::record(&config(), 114, 0, &s.choices[..2]),
        Err(ReplayError::MissingChoice { index: 2 })
    ));
}

#[test]
fn played_replay_fresh_process_and_quantum_suffixes() {
    let _guard = REPLAY_TEST_LOCK.lock().unwrap();
    use std::{
        num::NonZeroUsize,
        process::{Command, Stdio},
    };
    const INPUT: &str = "MTG_PLAYED_REPLAY_TEST";
    if let Ok(path) = std::env::var(INPUT) {
        let bytes = std::fs::read(path).unwrap();
        let scalar = played::verify(&bytes).unwrap();
        assert_eq!(scalar.life(), [20, 0]);
        let v: Value = serde_json::from_slice(&bytes).unwrap();
        let config: Config = serde_json::from_value(v["config"].clone()).unwrap();
        let mut saved_work = std::collections::BTreeSet::new();
        for budget in [1, 3, 17] {
            let q = NonZeroUsize::new(budget).unwrap();
            let mut g = Game::new().unwrap();
            g.reset(&config, 114, 0).unwrap();
            let mut comparison = Game::new().unwrap();
            comparison.reset(&config, 114, 0).unwrap();
            for (i, r) in v["choices"].as_array().unwrap().iter().enumerate() {
                let b = serde_json::to_vec(&r["choice"]).unwrap();
                actions::apply(&mut comparison, &b, CAP).unwrap();
                let decoded = actions::decode(&g, &b, CAP).unwrap();
                let progress = match decoded {
                    actions::Decoded::Decision {
                        actor,
                        ref submission,
                    } if submission.choices == [policy::Choice::Pass] => {
                        let d = g.turn_decision().unwrap();
                        Some(
                            g.apply_turn_quantum(
                                actor,
                                &turns::TurnAction {
                                    decision: d.id,
                                    selection: turns::TurnSelection::Pass(d.candidate(0)),
                                },
                                q,
                            )
                            .unwrap(),
                        )
                    }
                    actions::Decoded::Decision {
                        actor,
                        ref submission,
                    } if submission.choices == [policy::Choice::FinishCombat] => Some(
                        g.finish_combat_quantum(actor, g.turn_decision().unwrap().id, q)
                            .unwrap(),
                    ),
                    actions::Decoded::Decision {
                        actor,
                        ref submission,
                    } if submission
                        .choices
                        .iter()
                        .all(|c| matches!(c, policy::Choice::Discard { .. })) =>
                    {
                        let d = g.turn_decision().unwrap();
                        let original = g.discard_cards().unwrap();
                        let mut hand = original.clone();
                        hand.sort_by_key(|h| g.objects().get(*h).unwrap().card.identity().key);
                        let cards = submission
                            .choices
                            .iter()
                            .map(|c| {
                                let policy::Choice::Discard { card } = c else {
                                    panic!()
                                };
                                d.candidate(
                                    original.iter().position(|h| *h == hand[card.row]).unwrap(),
                                )
                            })
                            .collect();
                        Some(
                            g.apply_turn_quantum(
                                actor,
                                &turns::TurnAction {
                                    decision: d.id,
                                    selection: turns::TurnSelection::Discard(cards),
                                },
                                q,
                            )
                            .unwrap(),
                        )
                    }
                    _ => {
                        actions::apply(&mut g, &b, CAP).unwrap();
                        None
                    }
                };
                if let Some(mut p) = progress {
                    let mut yields = 0;
                    while p == Progress::InternalYield {
                        yields += 1;
                        assert!(yields < 1000);
                        let state = normalized(&g);
                        let first = &state["work"][0];
                        let family = first
                            .as_object()
                            .map(|o| o.keys().next().unwrap().clone())
                            .unwrap_or_else(|| first.to_string());
                        if budget == 1 && saved_work.insert(family.clone()) {
                            let suffix = v["choices"].as_array().unwrap()[i + 1..]
                                .iter()
                                .map(|r| r["choice"].clone())
                                .collect::<Vec<_>>();
                            let request = json!({"snapshot":String::from_utf8(g.snapshot()).unwrap(),"suffix":suffix,"expected":normalized(&scalar),"yielded":true});
                            fresh_snapshot_suffix(&request, &family);
                        }
                        let save = g.snapshot();
                        let before = normalized(&g);
                        g.restore(&save).unwrap();
                        assert_eq!(normalized(&g), before);
                        p = g.resume(q);
                    }
                }
                if comparison.decision().is_none()
                    && comparison.turn_position().is_none()
                    && comparison.outcome().is_none()
                {
                    comparison.start_turns().unwrap();
                    let mut p = g.start_turns_quantum(q).unwrap();
                    while p == Progress::InternalYield {
                        let save = g.snapshot();
                        let before = normalized(&g);
                        g.restore(&save).unwrap();
                        assert_eq!(normalized(&g), before);
                        p = g.resume(q);
                    }
                }
                assert_eq!(
                    normalized(&g),
                    normalized(&comparison),
                    "choice {i} budget {budget}"
                );
            }
            assert_eq!(g.outcome(), scalar.outcome());
        }
        return;
    }
    let (v, _) = recording();
    let path = std::env::temp_dir().join(format!("played-replay-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(&v).unwrap()).unwrap();
    let result = Command::new("timeout")
        .args(["240"])
        .arg(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "played_replay_fresh_process_and_quantum_suffixes",
            "--nocapture",
        ])
        .env(INPUT, &path)
        .env_remove("DISPLAY")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed; 0 failed"));
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
fn normalized(g: &Game) -> Value {
    fn scrub(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for key in ["scope", "store"] {
                    if m.contains_key(key) {
                        m.insert(key.into(), json!(0));
                    }
                }
                for v in m.values_mut() {
                    scrub(v)
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
    let envelope: Value = serde_json::from_slice(&g.snapshot()).unwrap();
    let mut value: Value = serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
    value["objects"]["id"] = json!(0);
    scrub(&mut value);
    value
}

// GH-19: fresh-process snapshots at every distinct pending choice and response
// boundary. Literal outcomes come from script()'s CR/card ledger, while full
// state equality is an additional metamorphic check (not a rules oracle).
#[test]
fn snapshot_played_pending_choices_fresh_process() {
    let _guard = REPLAY_TEST_LOCK.lock().unwrap();
    const INPUT: &str = "MTG_PENDING_SNAPSHOT_TEST";
    if let Ok(path) = std::env::var(INPUT) {
        let request: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let mut g = Game::new().unwrap();
        g.restore(request["snapshot"].as_str().unwrap().as_bytes())
            .unwrap();
        let expected: Value = request["expected"].clone();
        if request["yielded"] == true {
            let mut progress = g.resume(std::num::NonZeroUsize::new(1).unwrap());
            let mut count = 0;
            while progress == Progress::InternalYield {
                count += 1;
                assert!(count < 1000);
                progress = g.resume(std::num::NonZeroUsize::new(1).unwrap());
            }
        }
        for r in request["suffix"].as_array().unwrap() {
            let bytes = serde_json::to_vec(r).unwrap();
            // Semantic records reacquire the new process's handles and decision.
            actions::apply(&mut g, &bytes, CAP).unwrap();
            if g.decision().is_none() && g.turn_position().is_none() && g.outcome().is_none() {
                g.start_turns().unwrap();
            }
        }
        assert_eq!(g.life(), [20, 0]);
        assert_eq!(g.outcome().unwrap().winner, Some(Seat::P0));
        assert_eq!(
            normalized(&g),
            expected,
            "complete state, RNG and ordered zones"
        );
        return;
    }
    let s = script();
    let mut g = Game::new().unwrap();
    g.reset(&config(), 114, 0).unwrap();
    let mut seen = std::collections::BTreeSet::new();
    let path = std::env::temp_dir().join(format!("pending-snapshot-{}.json", std::process::id()));
    for (i, r) in s.choices.iter().enumerate() {
        let bytes = serde_json::to_vec(r).unwrap();
        let actions::Decoded::Decision { actor, submission } =
            actions::decode(&g, &bytes, CAP).unwrap()
        else {
            panic!()
        };
        let view = g.policy_observe(actor, CAP).unwrap();
        let kind = view.decision.as_ref().unwrap().kind;
        // Each payment microchoice and each response stack depth gets a save.
        let key = format!(
            "{kind}/{:?}/{}",
            submission.choices,
            g.objects().in_zone(Zone::Stack).count()
        );
        let snapshot = g.snapshot();
        let before = normalized(&g);
        g.restore(&snapshot).unwrap();
        assert_eq!(normalized(&g), before);
        let unchanged = g.snapshot();
        assert_eq!(
            g.apply_policy(actor, &submission, CAP),
            Err(policy::PolicyError::StaleDecision)
        );
        assert_eq!(g.snapshot(), unchanged);
        let actions::Decoded::Decision {
            submission: fresh, ..
        } = actions::decode(&g, &bytes, CAP).unwrap()
        else {
            panic!()
        };
        let wrong = if actor == Seat::P0 {
            Seat::P1
        } else {
            Seat::P0
        };
        assert_eq!(
            g.apply_policy(wrong, &fresh, CAP),
            Err(policy::PolicyError::WrongActor)
        );
        assert_eq!(g.snapshot(), unchanged);
        if seen.insert(key) {
            let request = json!({"snapshot":String::from_utf8(snapshot).unwrap(),"suffix":&s.choices[i..],"expected":normalized(&s.g)});
            std::fs::write(&path, serde_json::to_vec(&request).unwrap()).unwrap();
            let output = std::process::Command::new("timeout")
                .arg("120")
                .arg(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "snapshot_played_pending_choices_fresh_process",
                    "--nocapture",
                ])
                .env(INPUT, &path)
                .env_remove("DISPLAY")
                .stdin(std::process::Stdio::null())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "choice {i} {kind}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
        }
        actions::apply(&mut g, &bytes, CAP).unwrap();
        if g.decision().is_none() && g.turn_position().is_none() && g.outcome().is_none() {
            g.start_turns().unwrap();
        }
    }
    std::fs::remove_file(path).unwrap();
    for kind in [
        "bite_source/",
        "bite_destination/",
        "targets_complete/",
        "payment/",
        "attackers/",
        "blockers/",
        "cleanup_discard/",
    ] {
        assert!(
            seen.iter().any(|s| s.starts_with(kind)),
            "missing {kind}: {seen:?}"
        );
    }
    assert_eq!(normalized(&g), normalized(&s.g));
}

fn fresh_snapshot_suffix(request: &Value, context: &str) {
    let path = std::env::temp_dir().join(format!("yield-snapshot-{}.json", std::process::id()));
    std::fs::write(&path, serde_json::to_vec(request).unwrap()).unwrap();
    let output = std::process::Command::new("timeout")
        .arg("120")
        .arg(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "snapshot_played_pending_choices_fresh_process",
            "--nocapture",
        ])
        .env("MTG_PENDING_SNAPSHOT_TEST", &path)
        .env_remove("DISPLAY")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        output.status.success(),
        "{context}: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
}
