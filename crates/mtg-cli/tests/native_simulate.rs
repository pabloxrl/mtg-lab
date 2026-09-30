//! RFC0002 §9 and owned Driver budget contract; independent boundary counts.
use serde_json::{Value, json};
use std::{
    fs,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Input(PathBuf);
impl Input {
    fn new(value: &Value) -> Self {
        let p = std::env::temp_dir().join(format!(
            "mtg-simulate-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&p, serde_json::to_vec(value).unwrap()).unwrap();
        Self(p)
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn legacy_config() -> Value {
    json!({"schema_version":1, "game":mtg_core::opening::Config::default(), "policies":["pass-v1","pass-v1"],"master_seed":42,"first_episode":0,"episodes":2,"max_decisions":3000,"deadline_ms":null})
}
fn command(input: &Input) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mtg"));
    c.arg("simulate")
        .arg("--config")
        .arg(&input.0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env("TERM", "dumb");
    c
}
fn run(value: Value, extra: &[&str]) -> (i32, Vec<Value>, String) {
    let input = Input::new(&value);
    let mut c = command(&input);
    c.args(extra);
    run_command(c)
}
fn run_command(mut command: Command) -> (i32, Vec<Value>, String) {
    let mut child = command.spawn().unwrap();
    let out = child.stdout.take();
    let mut err = child.stderr.take().unwrap();
    let reader = thread::spawn(move || {
        let mut s = String::new();
        if let Some(mut out) = out {
            out.read_to_string(&mut s).unwrap();
        }
        s
    });
    let errors = thread::spawn(move || {
        let mut s = String::new();
        err.read_to_string(&mut s).unwrap();
        s
    });
    let until = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if Instant::now() > until {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("headless subprocess deadline exceeded")
        }
        thread::sleep(Duration::from_millis(5));
    };
    let output = reader.join().unwrap();
    assert!(!output.contains('\u{1b}'));
    (
        status.code().unwrap(),
        output
            .lines()
            .map(|s| serde_json::from_str(s).expect("versioned JSONL stdout"))
            .collect(),
        errors.join().unwrap(),
    )
}

fn config() -> Value {
    let mut c = legacy_config();
    c["schema_version"] = json!(2);
    c["policies"] = json!(["heuristic-m1-v1", "heuristic-m1-v1"]);
    c["native"] = json!({"policy_seed":42,"rng_version":"legal-random-rng-v1","work_quantum":64,"max_work_calls":100000,"max_records":10000});
    c["episodes"] = json!(1);
    for i in 0..2 {
        c["game"]["seats"][i]["deck"] = json!("green");
    }
    c
}
#[test]
fn native_one_decision_cannot_be_a_rules_outcome() {
    // CR103: one opening choice cannot cause life loss or rules completion.
    // Random may choose keep or mulligan; neither is a win or draw.
    for start in [0, 1] {
        for policy in ["heuristic-m1-v1", "legal-random-m1-v1"] {
            let mut c = config();
            c["game"]["starting_seat"] = json!(start);
            c["policies"] = json!([policy, policy]);
            c["max_decisions"] = json!(1);
            let (code, r, e) = run(c.clone(), &[]);
            assert_eq!(code, 0, "{e}");
            assert_eq!(r, run(c, &[]).1);
            assert_eq!(r[1]["status"], "truncated");
            assert_eq!(r[1]["reason"], "decision_limit");
            assert_eq!(r[1]["decisions"], 1);
            assert_eq!(r[1]["life"], json!([20, 20]));
            assert_eq!(r[1]["winner"], Value::Null);
            assert_eq!(r[2]["truncated"], 1);
            assert_eq!(r[2]["draws"], 0);
        }
    }
}
#[test]
fn native_explicit_work_and_record_limits_account_exactly() {
    let mut c = config();
    c["native"]["work_quantum"] = json!(1);
    c["native"]["max_work_calls"] = json!(1);
    c["episodes"] = json!(3);
    let (code, r, e) = run(c, &[]);
    assert_eq!(code, 0, "{e}");
    for row in &r[1..4] {
        assert_eq!(row["status"], "incomplete");
        assert_eq!(row["reason"], "work_limit");
        assert_eq!(row["work_calls"], 1); // reset only; no resume or choice
        assert_eq!(row["decisions"], 0);
        assert_eq!(row["life"], Value::Null); // partial reset has no view
    }
    assert_eq!(r[4]["incomplete"], 3);
    let mut c = config();
    c["native"]["max_records"] = json!(1);
    c["episodes"] = json!(3);
    let (code, r, e) = run(c, &[]);
    assert_eq!(code, 3, "{e}");
    assert_eq!(r[1]["status"], "failed");
    assert_eq!(r[1]["reason"], "record_capacity");
    assert_eq!(r[1]["decisions"], 1);
    assert_eq!(r[2]["failed"], 1);
    assert_eq!(r[2]["not_started"], 2);
    assert_eq!(r[2]["draws"], 0);
}
#[test]
fn native_bad_versions_policies_bounds_and_overflow_fail_without_output() {
    let mut bad = vec![];
    for (key, val) in [
        ("schema_version", json!(3)),
        (
            "policies",
            json!(["missing-private-token", "heuristic-m1-v1"]),
        ),
        ("episodes", json!(0)),
        ("max_decisions", json!(0)),
    ] {
        let mut c = config();
        c[key] = val;
        bad.push(c);
    }
    for key in [
        "work_quantum",
        "max_work_calls",
        "max_records",
        "policy_seed",
        "rng_version",
    ] {
        let mut c = config();
        c["native"].as_object_mut().unwrap().remove(key);
        bad.push(c);
    }
    for key in ["work_quantum", "max_work_calls", "max_records"] {
        let mut c = config();
        c["native"][key] = json!(0);
        bad.push(c);
    }
    let mut c = config();
    c["native"]["rng_version"] = json!("unknown");
    bad.push(c);
    let mut c = config();
    c["episodes"] = json!(u64::MAX);
    c["first_episode"] = json!(2);
    bad.push(c);
    let mut c = config();
    c["schema_version"] = json!(1);
    bad.push(c);
    let mut c = config();
    c.as_object_mut().unwrap().remove("native");
    bad.push(c);
    for c in bad {
        let (code, r, e) = run(c, &[]);
        assert_eq!(code, 2);
        assert!(r.is_empty());
        assert_eq!(serde_json::from_str::<Value>(&e).unwrap()["type"], "error");
        assert!(!e.contains("missing-private-token"));
    }
}

fn direct(c: &Value) -> Value {
    use mtg_core::{
        objects::Seat,
        opening::{Game, actions},
    };
    use mtg_policy::{HEURISTIC_VERSION, Heuristic, LegalRandom, RNG_VERSION, VERSION};
    use sha2::{Digest, Sha256};
    let mut g = Game::new().unwrap();
    let gc = serde_json::from_value(c["game"].clone()).unwrap();
    let episode = c["first_episode"].as_u64().unwrap();
    g.reset(&gc, c["master_seed"].as_u64().unwrap(), episode)
        .unwrap();
    let mut random = [0, 1].map(|s| {
        LegalRandom::new(
            VERSION,
            RNG_VERSION,
            c["native"]["policy_seed"].as_u64().unwrap(),
            episode,
            s,
        )
        .unwrap()
    });
    let heuristic = [0, 1].map(|s| Heuristic::new(HEURISTIC_VERSION, s).unwrap());
    let mut history = vec![];
    for _ in 0..c["max_decisions"].as_u64().unwrap() {
        if g.outcome().is_some() {
            break;
        }
        let (seat, o) = [Seat::P0, Seat::P1]
            .into_iter()
            .find_map(|s| {
                g.policy_observe(s, 256)
                    .ok()
                    .filter(|o| o.decision.is_some())
                    .map(|o| (s, o))
            })
            .unwrap();
        let i = usize::from(seat == Seat::P1);
        let s = if c["policies"][i] == VERSION {
            random[i].choose(&o).unwrap()
        } else {
            heuristic[i].choose(&o).unwrap()
        };
        history.push(actions::encode(&g, seat, &s, 256).unwrap());
        g.apply_policy(seat, &s, 256).unwrap();
        if g.decision().is_none() && g.turn_position().is_none() {
            g.start_turns().unwrap();
        }
    }
    if c["max_decisions"].as_u64().unwrap() > 10000 {
        let records = history
            .iter()
            .map(|b| serde_json::from_slice::<actions::Record>(b).unwrap())
            .collect::<Vec<_>>();
        let kinds = if c["policies"] == json!([HEURISTIC_VERSION, HEURISTIC_VERSION]) {
            vec!["play_land", "cast", "select_attackers"]
        } else {
            // LegalRandom may legally pass/cancel/decline attacks. Independent
            // policy-contract review preserves this seed and all comparisons.
            vec!["play_land", "cast"]
        };
        for kind in kinds {
            assert!(
                records.iter().flat_map(|r| &r.choices).any(|choice| {
                    let value = serde_json::to_value(choice).unwrap();
                    value["kind"] == kind
                        && (kind != "select_attackers"
                            || !value["cards"].as_array().unwrap().is_empty())
                }),
                "real played game must exercise {kind}"
            );
        }
    }
    let v = g.observe(Seat::P0).unwrap();
    json!({"history_sha256":format!("{:x}",Sha256::digest(serde_json::to_vec(&history).unwrap())),"decisions":history.len(),"life":v.life,"turn":v.turn.map(|t|t.0),"turn_position":v.turn,"hand_counts":v.hand_counts,"library_counts":v.library_counts,"public_zones":v.public_zones,"terminal":v.terminal})
}
#[test]
fn native_real_games_repeat_and_match_direct_libraries_with_rules_checkpoints() {
    for start in [0, 1] {
        for policies in [
            ["heuristic-m1-v1", "heuristic-m1-v1"],
            ["legal-random-m1-v1", "legal-random-m1-v1"],
            ["heuristic-m1-v1", "legal-random-m1-v1"],
        ] {
            let mut c = config();
            c["policies"] = json!(policies);
            c["game"]["starting_seat"] = json!(start);
            c["first_episode"] = json!(7);
            c["max_decisions"] = json!(20000);
            c["native"]["max_records"] = json!(20000);
            let (code, r, e) = run(c.clone(), &[]);
            assert_eq!(code, 0, "{e}");
            assert_eq!(r, run(c.clone(), &[]).1);
            let expected = direct(&c);
            for (key, v) in expected.as_object().unwrap() {
                assert_eq!(&r[1][key], v, "{key}");
            }
            assert_eq!(r[1]["status"], "completed");
            let terminal = &r[1]["terminal"];
            // Independent CR704.5a/b oracle: each declared loss must have the
            // corresponding public predicate. No generated winner constant.
            let mut losers = 0;
            for seat in 0..2 {
                match terminal["losses"][seat].as_str() {
                    Some("life") => {
                        assert!(r[1]["life"][seat].as_i64().unwrap() <= 0);
                        losers += 1;
                    }
                    Some("empty_draw") => {
                        assert_eq!(r[1]["library_counts"][seat], 0);
                        assert_eq!(r[1]["turn_position"][1], seat);
                        assert_eq!(r[1]["turn_position"][2], "draw");
                        losers += 1;
                    }
                    None => {}
                    other => panic!("unexpected loss {other:?}"),
                }
            }
            assert!(losers > 0);
            if losers == 1 {
                let winner = r[1]["winner"].as_u64().unwrap() as usize;
                assert_eq!(terminal["losses"][winner], Value::Null);
                assert_eq!(r[2]["wins"][winner], 1);
                assert_eq!(r[2]["draws"], 0);
            } else {
                assert_eq!(r[2]["draws"], 1);
            }
            assert_eq!(r[2]["started"], 1);
            assert_eq!(r[2]["completed"], 1);
            assert_eq!(r[2]["incomplete"], 0);
            assert_eq!(r[2]["failed"], 0);
        }
    }
}
#[test]
fn native_stdout_failure_is_a_caller_error() {
    let input = Input::new(&config());
    let mut c = command(&input);
    c.stdout(
        fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap(),
    );
    let (code, r, e) = run_command(c);
    assert_eq!(code, 3);
    assert!(r.is_empty());
    let e: Value = serde_json::from_str(&e).unwrap();
    assert_eq!(e["type"], "error");
    assert_eq!(e["message"], "simulation output failed");
}
#[test]
fn native_signals_have_bounded_complete_accounting() {
    use std::io::{BufRead, BufReader};
    for (signal, code) in [("-TERM", 143), ("-INT", 130)] {
        let mut c = config();
        c["episodes"] = json!(u64::MAX);
        c["first_episode"] = json!(0);
        let input = Input::new(&c);
        let mut child = command(&input).spawn().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let reader = thread::spawn(move || {
            let mut rows = vec![];
            for line in BufReader::new(stdout).lines() {
                rows.push(serde_json::from_str::<Value>(&line.unwrap()).unwrap());
                if rows.len() == 1 {
                    tx.send(()).unwrap();
                }
            }
            rows
        });
        if rx.recv_timeout(Duration::from_secs(10)).is_err() {
            let _ = child.kill();
            let _ = child.wait();
            panic!("no run header");
        }
        assert!(
            Command::new("kill")
                .args([signal, &child.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        let until = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(s) = child.try_wait().unwrap() {
                break s;
            }
            if Instant::now() > until {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("signal timeout");
            }
            thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(status.code(), Some(code));
        let r = reader.join().unwrap();
        let s = r.last().unwrap();
        let n = |k: &str| s[k].as_u64().unwrap();
        assert_eq!(
            n("started"),
            n("completed") + n("truncated") + n("failed") + n("incomplete")
        );
        assert_eq!(n("started") + n("not_started"), u64::MAX);
        assert_eq!(n("started"), r.len() as u64 - 2);
        assert!(n("not_started") > 0);
    }
}

#[test]
fn native_literal_first_land_checkpoint_both_starting_seats() {
    // Explicit normal-reset orders, no synthetic Game. CR103 keeps seven;
    // CR103.8a skips starter draw. Two keeps + two upkeep priority passes +
    // one land play = five decisions. CR305 moves one Forest to battlefield.
    let order = [
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
    .flat_map(|(card, n)| vec![card; n])
    .collect::<Vec<_>>();
    for start in [0, 1] {
        let mut c = config();
        c["game"]["starting_seat"] = json!(start);
        c["max_decisions"] = json!(5);
        for seat in 0..2 {
            c["game"]["seats"][seat]["order"] = json!(order);
        }
        let (code, r, e) = run(c.clone(), &[]);
        assert_eq!(code, 0, "{e}");
        let row = &r[1];
        assert_eq!(row["status"], "truncated");
        assert_eq!(row["decisions"], 5);
        assert_eq!(row["turn"], 1);
        assert_eq!(row["life"], json!([20, 20]));
        assert_eq!(row["library_counts"], json!([33, 33]));
        let mut hands = [7, 7];
        hands[start] = 6;
        assert_eq!(row["hand_counts"], json!(hands));
        let zones = row["public_zones"].as_array().unwrap();
        let battlefield = zones.iter().find(|z| z["zone"] == "battlefield").unwrap();
        assert_eq!(battlefield["cards"].as_array().unwrap().len(), 1);
        // CR302.6 concerns creatures only; doc/views.md defines creature
        // sickness. Independent expectation review is retained with evidence.
        assert_eq!(
            battlefield["cards"][0],
            json!({"card":"forest","controller":start,"owner":start,"tapped":false,"creature":null,"summoning_sick":false})
        );
        for zone in zones.iter().filter(|z| z["zone"] != "battlefield") {
            assert!(zone["cards"].as_array().unwrap().is_empty());
        }
        for (key, v) in direct(&c).as_object().unwrap() {
            assert_eq!(&row[key], v, "{key}");
        }
    }
}

#[test]
fn native_configuration_never_silently_runs_as_passive_benchmark() {
    let mut value = config();
    value["max_decisions"] = json!(1);
    let input = Input::new(&value);
    let mut c = Command::new(env!("CARGO_BIN_EXE_mtg"));
    c.args(["bench", "--workload", "scalar-pass-v1", "--config"])
        .arg(&input.0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY");
    let (code, r, e) = run_command(c);
    assert_eq!(code, 2);
    assert!(r.is_empty());
    assert_eq!(serde_json::from_str::<Value>(&e).unwrap()["type"], "error");
}
