//! Original CLI contract: RFC 0002 §9; CR 103.8a, 504.1, 704.5b.
//! Forty cards minus seven kept leaves 33 draws; passive play loses on
//! the nonstarter's 34th draw (turn 68), not when a library merely becomes empty.
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
fn config() -> Value {
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
#[test]
fn simulate_headless_reproducible_rules_completion_both_starters_and_mirrors() {
    for decks in [["red", "green"], ["red", "red"], ["green", "green"]] {
        for start in [0, 1] {
            let mut c = config();
            c["game"]["starting_seat"] = json!(start);
            for i in 0..2 {
                c["game"]["seats"][i]["deck"] = json!(decks[i]);
            }
            let (code, rows, err) = run(c.clone(), &[]);
            assert_eq!(code, 0, "{err}");
            assert_eq!(
                rows,
                run(c.clone(), &[]).1,
                "same seed/config must reproduce all bytes/records"
            );
            assert_eq!(rows[0]["type"], "run");
            assert_eq!(rows[0]["config"], c);
            assert_eq!(rows[0]["workers"], 1);
            assert_eq!(rows[0]["instrumentation"], "summary-v1");
            for field in [
                "engine_sha256",
                "rules_sha256",
                "cards_sha256",
                "config_sha256",
            ] {
                assert_eq!(rows[0][field].as_str().unwrap().len(), 64);
            }
            for (i, row) in rows[1..3].iter().enumerate() {
                assert_eq!(row["type"], "episode");
                assert_eq!(row["episode"], i);
                assert_eq!(row["status"], "completed");
                assert_eq!(row["turn"], 68);
                assert_eq!(row["life"], json!([20, 20]));
                assert_eq!(row["winner"], start);
                assert_eq!(row["reason"], "empty_draw");
                assert!(row["decisions"].as_u64().unwrap() > 2);
            }
            let s = rows.last().unwrap();
            assert_eq!(s["completed"], 2);
            assert_eq!(s["truncated"], 0);
            assert_eq!(s["failed"], 0);
            assert_eq!(s["unfinished"], 0);
            assert_eq!(s["not_started"], 0);
            assert_eq!(s["draws"], 0);
            assert_eq!(s["wins"][start as usize], 2);
        }
    }
}
#[test]
fn simulate_decision_limit_is_not_a_draw_or_win() {
    let mut c = config();
    c["max_decisions"] = json!(1);
    c["first_episode"] = json!(9);
    let (code, r, e) = run(c, &[]);
    assert_eq!(code, 0, "{e}");
    for (i, row) in r[1..3].iter().enumerate() {
        assert_eq!(row["episode"], 9 + i);
        assert_eq!(row["status"], "truncated");
        assert_eq!(row["reason"], "decision_limit");
        assert_eq!(row["decisions"], 1);
        assert_eq!(row["winner"], Value::Null);
    }
    let s = r.last().unwrap();
    assert_eq!(s["started"], 2);
    assert_eq!(s["truncated"], 2);
    assert_eq!(s["completed"], 0);
    assert_eq!(s["draws"], 0);
    assert_eq!(s["wins"], json!([0, 0]));
}
#[test]
fn simulate_invalid_configs_fail_explicitly_without_prompt() {
    let mut invalid = vec![];
    for (key, value) in [
        ("episodes", json!(0)),
        ("max_decisions", json!(0)),
        ("policies", json!(["human", "pass-v1"])),
        ("schema_version", json!(99)),
        ("surprise", json!(true)),
        ("deadline_ms", json!(0)),
    ] {
        let mut c = config();
        c[key] = value;
        invalid.push(c);
    }
    let mut c = config();
    c.as_object_mut().unwrap().remove("master_seed");
    invalid.push(c);
    let mut c = config();
    c["first_episode"] = json!(u64::MAX);
    invalid.push(c);
    let mut c = config();
    c["game"]["seats"][0]["deck"] = json!("unsupported");
    invalid.push(c);
    for c in invalid {
        let (code, rows, e) = run(c, &[]);
        assert_eq!(code, 2);
        assert!(rows.is_empty());
        let e: Value = serde_json::from_str(e.trim()).expect("structured validation error");
        assert_eq!(e["type"], "error");
        assert_eq!(e["schema_version"], 1);
    }
    let (code, _, e) = run(config(), &["--policy", "human"]);
    assert_eq!(code, 2);
    assert!(serde_json::from_str::<Value>(e.trim()).is_ok());
}
#[test]
fn simulate_output_path_and_write_failures() {
    let output = Input::new(&json!({}));
    fs::remove_file(&output.0).unwrap();
    let (code, r, e) = run(config(), &["--output", output.0.to_str().unwrap()]);
    assert_eq!(code, 0, "{e}");
    assert!(r.is_empty());
    let text = fs::read_to_string(&output.0).unwrap();
    assert_eq!(text.lines().count(), 4);
    // Refuse overwriting an existing result, and propagate a real write failure.
    assert_eq!(
        run(config(), &["--output", output.0.to_str().unwrap()]).0,
        3
    );
    assert_eq!(
        run(
            config(),
            &["--output", "/no-such-mtg-directory/result.jsonl"]
        )
        .0,
        3
    );
}
#[test]
fn simulate_sigterm_accounts_every_requested_episode_with_deadline() {
    use std::io::{BufRead, BufReader};
    let mut c = config();
    c["episodes"] = json!(1_000_000);
    let input = Input::new(&c);
    let mut child = command(&input).spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = thread::spawn(move || {
        let mut rows = vec![];
        for line in BufReader::new(stdout).lines() {
            let v: Value = serde_json::from_str(&line.unwrap()).unwrap();
            rows.push(v);
            if rows.len() == 1 {
                tx.send(()).unwrap();
            }
        }
        rows
    });
    if rx.recv_timeout(Duration::from_secs(10)).is_err() {
        let _ = child.kill();
        let _ = child.wait();
        panic!("no flushed run header before deadline");
    }
    assert!(
        Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
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
            panic!("SIGTERM summary deadline exceeded")
        }
        thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(status.code(), Some(143));
    let r = reader.join().unwrap();
    let s = r.last().unwrap();
    assert_eq!(s["type"], "summary");
    assert_eq!(s["reason"], "sigterm");
    let n = |k: &str| s[k].as_u64().unwrap();
    assert_eq!(
        n("started"),
        n("completed") + n("truncated") + n("failed") + n("unfinished")
    );
    assert_eq!(n("started") + n("not_started"), 1_000_000);
    assert!(n("not_started") > 0);
    assert_eq!(r.len() as u64 - 2, n("started"));
}

#[test]
fn simulate_wall_deadline_finishes_with_complete_accounting() {
    let mut c = config();
    c["episodes"] = json!(1_000_000);
    c["deadline_ms"] = json!(1);
    let (code, r, e) = run(c, &[]);
    assert_eq!(code, 4, "{e}");
    let s = r.last().unwrap();
    assert_eq!(s["reason"], "deadline");
    let n = |k: &str| s[k].as_u64().unwrap();
    assert_eq!(n("started") + n("not_started"), 1_000_000);
    assert_eq!(n("started"), n("completed") + n("truncated"));
    assert_eq!(n("failed"), 0);
    assert_eq!(n("unfinished"), 0);
    assert!(n("not_started") > 0);
}
#[test]
fn simulate_missing_malformed_config_and_broken_stdout_are_explicit() {
    let input = Input::new(&config());
    fs::write(&input.0, b"{broken").unwrap();
    let (code, _, errors) = run_command(command(&input));
    assert_eq!(code, 2);
    assert!(serde_json::from_str::<Value>(&errors).is_ok());
    fs::remove_file(&input.0).unwrap();
    let (code, _, errors) = run_command(command(&input));
    assert_eq!(code, 2);
    assert!(serde_json::from_str::<Value>(&errors).is_ok());
    let input = Input::new(&config());
    let mut c = command(&input);
    c.stdout(
        fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap(),
    );
    let (code, _, errors) = run_command(c);
    assert_eq!(code, 3);
    assert!(serde_json::from_str::<Value>(&errors).is_ok());
}
