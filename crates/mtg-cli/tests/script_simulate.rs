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
fn run_command(command: Command) -> (i32, Vec<Value>, String) {
    run_command_with_timeout(command, Duration::from_secs(10))
}
fn run_game(value: Value) -> (i32, Vec<Value>, String) {
    let input = Input::new(&value);
    // Full debug games are not a shared-runner performance qualification.
    // Independent watchdog review retains all semantic and ten-second signal
    // checks; this outer liveness guard still kills/reaps a hung subprocess.
    run_command_with_timeout(command(&input), Duration::from_secs(60))
}
fn run_command_with_timeout(mut command: Command, timeout: Duration) -> (i32, Vec<Value>, String) {
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
    let until = Instant::now() + timeout;
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
    serde_json::from_str(include_str!("../../../fixtures/simulate/script-v3.json")).unwrap()
}
// Original CR103, 305, 601, 608, 508–510, 104.3a: the ordered green
// opening plays two Forests, casts a Cub, then Growth and an unblocked attack.
// 2/2 +3/+3 deals 5; defending P1 concedes. No replay-derived expectations.
#[test]
fn script_normal_reset_literal_combat_and_exact_consumption() {
    let c = config();
    assert_eq!(c["script"]["records"].as_array().unwrap().len(), 102);
    let (code, r, e) = run_game(c.clone());
    assert_eq!(code, 0, "{e}");
    assert_eq!(r, run_game(c).1);
    assert_eq!(r[1]["status"], "completed");
    assert_eq!(r[1]["winner"], 0);
    assert_eq!(r[1]["decisions"], 101);
    assert_eq!(r[1]["life"], json!([20, 15]));
    assert_eq!(r[1]["hand_counts"], json!([5, 7]));
    assert_eq!(r[1]["library_counts"], json!([31, 31]));
    let zones = r[1]["public_zones"].as_array().unwrap();
    let battlefield = zones.iter().find(|z| z["zone"] == "battlefield").unwrap();
    assert_eq!(battlefield["cards"].as_array().unwrap().len(), 5);
    let cub = battlefield["cards"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["card"] == "bear-cub")
        .unwrap();
    assert_eq!(cub["creature"], json!([5, 5, 0]));
    assert_eq!(
        zones.iter().find(|z| z["zone"] == "graveyard_0").unwrap()["cards"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(r[1]["script_consumed"], 102);
    assert_eq!(r[1]["script_status"], "complete");
    assert_eq!(r[2]["script_consumed"], 102);
    assert_eq!(r[2]["script_remaining"], 0);
    assert_eq!(r[2]["completed"], 1);
    // Script payloads (and their object identities) are never echoed in headers.
    assert!(r[0]["config"]["script"].get("records").is_none());
    assert_eq!(r[0]["script_privacy"], "privileged");
}
fn concession(episode: u64, seat: &str) -> Value {
    json!({"episode":episode,"decision":0,"seat":seat,"record":json!({"version":1,"actor":seat,"decision":"concession","choices":[{"kind":"concede"}]}).to_string()})
}
#[test]
fn script_both_seats_multiple_episodes_concessions_consume_once() {
    let mut c = config();
    c["first_episode"] = json!(7);
    c["episodes"] = json!(2);
    c["script"]["records"] = json!([concession(7, "P0"), concession(8, "P1")]);
    let (code, r, e) = run(c, &[]);
    assert_eq!(code, 0, "{e}");
    assert_eq!(r.len(), 4);
    for (i, winner) in [(1, 1), (2, 0)] {
        assert_eq!(r[i]["decisions"], 0);
        assert_eq!(r[i]["winner"], winner);
        assert_eq!(r[i]["script_consumed"], 1);
        assert_eq!(r[i]["script_status"], "complete");
    }
    assert_eq!(r[3]["completed"], 2);
    assert_eq!(r[3]["wins"], json!([1, 1]));
    assert_eq!(r[3]["script_consumed"], 2);
    assert_eq!(r[3]["script_remaining"], 0);
}
#[test]
fn script_missing_extra_reordered_stale_malformed_illegal_never_fall_back() {
    for mutation in 0..11 {
        let mut c = config();
        let records = c["script"]["records"].as_array_mut().unwrap();
        let (accepted, consumed) = match mutation {
            0 => {
                records.clear();
                (0, 0)
            }
            1 => {
                records.push(concession(0, "P1"));
                (101, 102)
            }
            2 => {
                records.swap(0, 1);
                (0, 0)
            }
            3 => {
                records[0]["record"] = json!("PRIVATE_INVALID_PAYLOAD");
                (0, 0)
            }
            4 => {
                records[0]["seat"] = json!("P1");
                (0, 0)
            }
            5 => {
                records[0]["decision"] = json!(1);
                (0, 0)
            }
            6 => {
                records[0]["episode"] = json!(1);
                (0, 0)
            }
            7 => {
                let mut r: Value =
                    serde_json::from_str(records[4]["record"].as_str().unwrap()).unwrap();
                r["choices"][0]["card"]["incarnation"] = json!(9);
                records[4]["record"] = json!(r.to_string());
                (4, 4)
            }
            8 => {
                let mut r: Value =
                    serde_json::from_str(records[0]["record"].as_str().unwrap()).unwrap();
                r["choices"] = json!([{"kind":"pass"}]);
                records[0]["record"] = json!(r.to_string());
                (0, 0)
            }
            9 => {
                records.pop();
                (101, 101)
            }
            _ => {
                records.remove(4);
                (4, 4)
            }
        };
        let (code, r, e) = run_game(c);
        assert_eq!(code, 3, "mutation {mutation}: {e}");
        assert_eq!(r[1]["decisions"], accepted, "mutation {mutation}");
        assert_eq!(r[1]["script_consumed"], consumed);
        assert_eq!(r[1]["script_status"], "error");
        assert!(!r[1]["caller_error"].is_null());
        let s = r.last().unwrap();
        assert_eq!(s["started"], 1);
        assert_eq!(s["not_started"], 0);
        assert_eq!(s["draws"], 0);
        assert_eq!(s["failed"], 0);
        assert_eq!(s["completed"], u64::from(mutation == 1));
        assert_eq!(s["incomplete"], u64::from(mutation != 1));
        assert!(!format!("{r:?}{e}").contains("PRIVATE_INVALID_PAYLOAD"));
    }
}
#[test]
fn script_bounds_and_explicit_mode_validation() {
    for mutation in 0..7 {
        let mut c = config();
        match mutation {
            0 => c["script"]["max_bytes"] = json!(1),
            1 => c["script"]["max_records"] = json!(101),
            2 => c["script"]["privacy"] = json!("public"),
            3 => c["script"]["version"] = json!(9),
            4 => c["policies"][1] = json!("heuristic-thrill-v1"),
            5 => c["schema_version"] = json!(2),
            _ => {
                c.as_object_mut().unwrap().remove("script");
            }
        }
        let (code, r, e) = run(c, &[]);
        assert_eq!(code, 2, "{e}");
        assert!(r.is_empty());
        let error: Value = serde_json::from_str(&e).unwrap();
        assert_eq!(error["type"], "error");
    }
}
#[test]
fn script_decision_and_work_stops_are_truncated_not_consumed() {
    for work in [false, true] {
        let mut c = config();
        if work {
            c["native"]["max_work_calls"] = json!(1);
        } else {
            c["max_decisions"] = json!(1);
        }
        let (code, r, e) = run(c, &[]);
        assert_eq!(code, 0, "{e}");
        assert_eq!(r[1]["status"], "truncated");
        assert_eq!(r[1]["script_status"], "truncated");
        assert_eq!(r[1]["script_consumed"], u64::from(!work));
        assert_eq!(r[2]["completed"], 0);
        assert_eq!(r[2]["truncated"], 1);
        assert_eq!(r[2]["script_remaining"], if work { 102 } else { 101 });
    }
}

#[test]
fn script_full_combat_both_starting_seats_and_episode_boundaries() {
    fn swap(v: &mut Value) {
        match v {
            Value::String(s) if s == "P0" => *s = "P1".into(),
            Value::String(s) if s == "P1" => *s = "P0".into(),
            Value::Object(m) => {
                if let Some(b) = m.get_mut("birth") {
                    let n = b.as_u64().unwrap();
                    *b = json!(if n < 40 { n + 40 } else { n - 40 });
                }
                for v in m.values_mut() {
                    swap(v);
                }
            }
            Value::Array(a) => {
                for v in a {
                    swap(v);
                }
            }
            _ => (),
        }
    }
    for start in [0, 1] {
        let mut c = config();
        c["game"]["starting_seat"] = json!(start);
        c["episodes"] = json!(2);
        let mut records = c["script"]["records"].as_array().unwrap().clone();
        if start == 1 {
            for e in &mut records {
                e["seat"] = json!(if e["seat"] == "P0" { "P1" } else { "P0" });
                let mut r: Value = serde_json::from_str(e["record"].as_str().unwrap()).unwrap();
                swap(&mut r);
                e["record"] = json!(r.to_string());
            }
        }
        let mut second = records.clone();
        for e in &mut second {
            e["episode"] = json!(1);
        }
        records.extend(second);
        c["script"]["records"] = json!(records);
        let (code, r, e) = run_game(c);
        assert_eq!(code, 0, "{e}");
        for row in &r[1..3] {
            assert_eq!(row["winner"], start);
            assert_eq!(row["decisions"], 101);
            assert_eq!(
                row["life"],
                if start == 0 {
                    json!([20, 15])
                } else {
                    json!([15, 20])
                }
            );
            assert_eq!(row["script_consumed"], 102);
            assert_eq!(row["script_status"], "complete");
        }
        assert_eq!(r[3]["completed"], 2);
        assert_eq!(r[3]["script_consumed"], 204);
        assert_eq!(r[3]["script_remaining"], 0);
    }
}

#[test]
fn script_played_combat_reaches_literal_rules_terminal_without_concession() {
    // Original continuation: first Growth attack deals 5, then eight unblocked
    // 2-power attacks deal 16. CR104.3b/704.5a require a life loss at -1.
    // Explicit cleanup discards retain seven cards; final active player draws
    // the eighth before lethal combat. No concession or policy supplies the end.
    let c: Value = serde_json::from_str(include_str!(
        "../../../fixtures/simulate/script-lethal-v3.json"
    ))
    .unwrap();
    assert_eq!(c["script"]["records"].as_array().unwrap().len(), 434);
    let (code, r, e) = run_game(c);
    assert_eq!(code, 0, "{e}");
    assert_eq!(r[1]["status"], "completed");
    assert_eq!(r[1]["winner"], 0);
    assert_eq!(r[1]["life"], json!([20, -1]));
    assert_eq!(r[1]["decisions"], 434);
    assert_eq!(r[1]["hand_counts"], json!([8, 7]));
    assert_eq!(r[1]["library_counts"], json!([23, 23]));
    assert_eq!(r[1]["script_consumed"], 434);
    assert_eq!(r[1]["script_status"], "complete");
    assert_eq!(r[2]["script_remaining"], 0);
    assert_eq!(r[2]["wins"], json!([1, 0]));
}

#[test]
fn script_input_limits_include_envelopes_and_file_bytes() {
    let mut c = config();
    let bytes = serde_json::to_vec(&c["script"]["records"]).unwrap().len();
    c["script"]["max_bytes"] = json!(bytes);
    assert_eq!(run_game(c.clone()).0, 0);
    c["script"]["max_bytes"] = json!(bytes - 1);
    assert_eq!(run(c.clone(), &[]).0, 2);
    c["script"]["records"][0]["record"] = json!("PRIVATE".repeat(200_000));
    let (code, r, e) = run(c, &[]);
    assert_eq!(code, 2);
    assert!(r.is_empty());
    assert!(!e.contains("PRIVATE"));
}
