//! RFC R0002-B039/B041: closed stdin, no display, strict machine results.
//! Expected opening counts/life: CR 103.4/103.5. Passive loss: CR 103.8a,
//! 504.1/704.5b. Conformance uses the existing independent CR 117.3d fixture.
use mtg_core::{
    objects::Seat,
    opening::{
        Config,
        replay::{self, Action, Choice},
    },
};
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
struct Dir(PathBuf);
impl Dir {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mtg-tools-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn file(&self, name: &str, bytes: &[u8]) -> String {
        let p = self.0.join(name);
        fs::write(&p, bytes).unwrap();
        p.to_str().unwrap().into()
    }
    fn path(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().into()
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn command(args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_mtg"));
    c.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env("TERM", "dumb");
    c
}
fn run(mut c: Command) -> (i32, Vec<Value>, String) {
    let mut p = c.spawn().unwrap();
    let mut out = p.stdout.take().unwrap();
    let mut err = p.stderr.take().unwrap();
    let o = thread::spawn(move || {
        let mut s = String::new();
        out.read_to_string(&mut s).unwrap();
        s
    });
    let e = thread::spawn(move || {
        let mut s = String::new();
        err.read_to_string(&mut s).unwrap();
        s
    });
    let until = Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(s) = p.try_wait().unwrap() {
            break s;
        }
        if Instant::now() > until {
            p.kill().unwrap();
            p.wait().unwrap();
            panic!("headless command hung");
        }
        thread::sleep(Duration::from_millis(5));
    };
    let out = o.join().unwrap();
    let err = e.join().unwrap();
    assert!(!out.contains('\u{1b}') && !err.contains('\u{1b}'));
    let rows = out
        .lines()
        .map(|l| serde_json::from_str(l).expect("JSONL stdout"))
        .collect();
    (status.code().unwrap(), rows, err)
}
fn success(c: Command) -> Vec<Value> {
    let (code, rows, err) = run(c);
    assert_eq!(code, 0, "{err}");
    assert!(err.is_empty(), "{err}");
    assert!(!rows.is_empty());
    for r in &rows {
        assert_eq!(r["schema_version"], 1);
    }
    rows
}
fn failure(c: Command, code: i32, needle: &str) {
    let (actual, rows, err) = run(c);
    assert_eq!(actual, code, "{err}");
    assert!(rows.is_empty(), "{rows:?}");
    let e: Value = serde_json::from_str(&err).expect("structured stderr");
    assert_eq!(e["schema_version"], 1);
    assert_eq!(e["exit_code"], code);
    assert!(e["message"].as_str().unwrap().contains(needle), "{e}");
}
fn recording() -> Vec<u8> {
    replay::record(
        &Config::default(),
        42,
        9,
        &[
            Choice {
                actor: Seat::P0,
                action: Action::Keep {},
            },
            Choice {
                actor: Seat::P1,
                action: Action::Keep {},
            },
        ],
    )
    .unwrap()
}
#[test]
fn replay_verify_and_seat_filtered_inspect() {
    let d = Dir::new();
    let p = d.file("opening.json", &recording());
    let r = success(command(&["replay", "verify", &p]));
    assert_eq!(r[0]["status"], "verified");
    assert_eq!(r[0]["scope"], "opening-v1");
    for seat in ["0", "1"] {
        let r = success(command(&[
            "replay", "inspect", &p, "--seat", seat, "--format", "jsonl",
        ]));
        assert_eq!(r.len(), 1);
        let v = &r[0]["observation"];
        assert_eq!(v["seat"], seat.parse::<u8>().unwrap());
        assert_eq!(v["life"], json!([20, 20]));
        assert_eq!(v["hand_counts"], json!([7, 7]));
        assert_eq!(v["library_counts"], json!([33, 33]));
        assert_eq!(v["hand"].as_array().unwrap().len(), 7);
        assert!(v.get("libraries").is_none() && v.get("rng").is_none());
        // Default red/green: the other seat's distinct cards never enter this view.
        let text = serde_json::to_string(&r).unwrap();
        assert!(!text.contains(if seat == "0" { "forest" } else { "mountain" }));
    }
}
#[test]
fn replay_errors_are_explicit_and_inspection_diagnostics_are_redacted() {
    let d = Dir::new();
    let mut r: Value = serde_json::from_slice(&recording()).unwrap();
    r["choices"].as_array_mut().unwrap().pop();
    let p = d.file("missing.json", &serde_json::to_vec(&r).unwrap());
    failure(command(&["replay", "verify", &p]), 2, "MissingChoice");
    let p = d.file("malformed.json", b"{}");
    failure(command(&["replay", "verify", &p]), 2, "Malformed");
    let mut r: Value = serde_json::from_slice(&recording()).unwrap();
    r["initial"]["hands"][1][0] = json!("PRIVATE_SENTINEL");
    let p = d.file("tampered.json", &serde_json::to_vec(&r).unwrap());
    let (code, rows, err) = run(command(&[
        "replay", "inspect", &p, "--seat", "0", "--format", "jsonl",
    ]));
    assert_eq!(code, 2);
    assert!(rows.is_empty());
    assert!(!err.contains("PRIVATE_SENTINEL"));
    assert!(err.contains("Divergence"));
    failure(
        command(&["replay", "inspect", &p, "--seat", "2", "--format", "jsonl"]),
        2,
        "seat",
    );
}
#[test]
fn trajectories_validate_seal_and_reject_corruption() {
    let d = Dir::new();
    let bytes = include_bytes!("../../mtg-recorder/tests/episode.jsonl");
    let p = d.file("good.jsonl", bytes);
    let r = success(command(&["trajectories", "validate", &p]));
    assert_eq!(r[0]["status"], "valid");
    assert_eq!(r[0]["episodes"], 1);
    // Independently approved correction: the handwritten fixture has one action.
    assert_eq!(r[0]["decisions"], 1);
    let mut lines: Vec<Value> = std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    lines[1]["decisions"] = json!(2);
    let corrupt = format!("{}\n{}\n", lines[0], lines[1]);
    let p = d.file("count-tampered.jsonl", corrupt.as_bytes());
    failure(command(&["trajectories", "validate", &p]), 2, "Integrity");
    // SHA-256 of the empty byte string is an independent published constant.
    let empty = b"{\"decisions\":0,\"episodes\":0,\"format\":1,\"kind\":\"seal\",\"sha256\":\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"}\n";
    let p = d.file("empty.jsonl", empty);
    let r = success(command(&["trajectories", "validate", &p]));
    assert_eq!(r[0]["episodes"], 0);
    assert_eq!(r[0]["decisions"], 0);
    let first = bytes.iter().position(|b| *b == b'\n').unwrap();
    let p = d.file("incomplete.jsonl", &bytes[..=first]);
    failure(command(&["trajectories", "validate", &p]), 2, "Incomplete");
    let p = d.file("bad.jsonl", b"{}\n");
    failure(command(&["trajectories", "validate", &p]), 2, "Invalid");
}
fn comparison(d: &Dir) -> (String, String) {
    let fixture: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/scenarios/priority-pass.json"
    ))
    .unwrap();
    let state = fixture["setup"]["state"].clone();
    let snap = json!({"state":state,"rng":{"algorithm":"synthetic-test","state_hex":"00"},"decision":{"id":"pass-0","actor":0,"kind":"priority","candidates":["pass"]},"private_information":{"views":[{"seat":0,"observation":{"own_hand":[],"opponent_hand_count":0,"library_counts":[0,0]}},{"seat":1,"observation":{"own_hand":[],"opponent_hand_count":0,"library_counts":[0,0]}}]}});
    let actual = json!({"checkpoint_version":1,"fixture_id":"schema/priority-pass","fixture_revision":fixture["provenance"]["fixture_revision"],"initial":state,"consumed_script":[{"id":"pass-0","actor":0,"kind":"pass","source":null,"choices":[{"id":"pass-choice","actor":0,"kind":"pass","values":[]}]}],"checkpoints":[{"name":"priority-p1","after":"pass-0","kind":"decision","state":{"priority":1}}],"invalid_results":[{"action":fixture["invalid_actions"][0]["action"],"at":"initial","error":"wrong_actor","before":snap,"after":snap}]});
    (
        d.file("fixture.json", &serde_json::to_vec(&fixture).unwrap()),
        d.file("actual.json", &serde_json::to_vec(&actual).unwrap()),
    )
}
fn conformance(f: &str, a: &str, artifacts: &str) -> Command {
    command(&[
        "conformance",
        "--suite",
        "checkpoints-v1",
        "--fixture",
        f,
        "--actual",
        a,
        "--artifacts",
        artifacts,
    ])
}
#[test]
fn conformance_compares_actual_checkpoints_and_preserves_failure_artifacts() {
    let d = Dir::new();
    let (f, a) = comparison(&d);
    let r = success(conformance(&f, &a, &d.path("pass")));
    assert_eq!(r[0]["status"], "pass");
    assert_eq!(r[0]["scope"], "supplied-checkpoints-only");
    let mut actual: Value = serde_json::from_slice(&fs::read(&a).unwrap()).unwrap();
    actual["checkpoints"][0]["state"]["priority"] = json!(0);
    fs::write(&a, serde_json::to_vec(&actual).unwrap()).unwrap();
    let (code, r, err) = run(conformance(&f, &a, &d.path("fail")));
    assert_eq!(code, 1, "{err}");
    assert!(err.is_empty());
    assert_eq!(r[0]["schema_version"], 1);
    assert_eq!(r[0]["status"], "mismatch");
    assert_eq!(r[0]["path"], "/priority");
    assert!(d.0.join("fail/diff.json").is_file());
    actual["consumed_script"] = json!([]);
    fs::write(&a, serde_json::to_vec(&actual).unwrap()).unwrap();
    let (code, r, _) = run(conformance(&f, &a, &d.path("unresolved")));
    assert_eq!(code, 1);
    assert_eq!(r[0]["checkpoint"], "script");
}
#[test]
fn conformance_missing_dependencies_and_malformed_input_fail_without_prompt() {
    let d = Dir::new();
    let (f, a) = comparison(&d);
    let mut c = conformance(&f, &a, &d.path("missing"));
    c.env("PATH", "");
    failure(c, 3, "python3");
    let a = d.file("bad.json", b"{}");
    failure(conformance(&f, &a, &d.path("bad")), 2, "invalid");
    failure(
        command(&[
            "conformance",
            "--suite",
            "all",
            "--references",
            "xmage,forge",
        ]),
        2,
        "unsupported",
    );
}
#[test]
fn benchmark_smoke_runs_scalar_production_and_keeps_truncation_distinct() {
    let d = Dir::new();
    let mut c: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/simulate/pass-v1.json")).unwrap();
    c["episodes"] = json!(1);
    let p = d.file("config.json", &serde_json::to_vec(&c).unwrap());
    let r = success(command(&[
        "bench",
        "--workload",
        "scalar-pass-v1",
        "--config",
        &p,
    ]));
    assert_eq!(r.len(), 1);
    assert_eq!(r[0]["qualification"], "smoke-only");
    assert_eq!(r[0]["summary"]["completed"], 1);
    assert_eq!(r[0]["summary"]["wins"], json!([1, 0]));
    assert_eq!(r[0]["summary"]["truncated"], 0);
    assert!(r[0]["elapsed_ns"].as_u64().unwrap() > 0);
    c["max_decisions"] = json!(1);
    let p = d.file("short.json", &serde_json::to_vec(&c).unwrap());
    let r = success(command(&[
        "bench",
        "--workload",
        "scalar-pass-v1",
        "--config",
        &p,
    ]));
    assert_eq!(r[0]["summary"]["completed"], 0);
    assert_eq!(r[0]["summary"]["truncated"], 1);
    c["policies"] = json!(["human", "pass-v1"]);
    let p = d.file("human.json", &serde_json::to_vec(&c).unwrap());
    failure(
        command(&["bench", "--workload", "scalar-pass-v1", "--config", &p]),
        2,
        "policies",
    );
}
#[test]
fn files_usage_and_output_failures_are_structured() {
    let d = Dir::new();
    failure(
        command(&["replay", "verify", &d.path("absent")]),
        2,
        "input",
    );
    failure(
        command(&["replay", "verify", d.0.to_str().unwrap()]),
        2,
        "regular",
    );
    failure(command(&[]), 2, "usage");
    let p = d.file("opening.json", &recording());
    let out = d.path("result.jsonl");
    let (code, rows, err) = run(command(&["replay", "verify", &p, "--output", &out]));
    assert_eq!(code, 0, "{err}");
    assert!(rows.is_empty());
    assert!(err.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&out).unwrap()).unwrap()["status"],
        "verified"
    );
    failure(
        command(&["replay", "verify", &p, "--output", &out]),
        3,
        "output",
    );
}

#[test]
fn input_limits_and_benchmark_configuration_fail_explicitly() {
    let d = Dir::new();
    let p = d.path("oversize.json");
    fs::File::create(&p)
        .unwrap()
        .set_len(16 * 1024 * 1024 + 1)
        .unwrap();
    failure(command(&["replay", "verify", &p]), 2, "byte limit");
    failure(command(&["trajectories", "validate", &p]), 2, "byte limit");
    let p = d.file("malformed-config.json", b"{");
    failure(
        command(&["bench", "--workload", "scalar-pass-v1", "--config", &p]),
        2,
        "EOF",
    );
    let mut config: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/simulate/pass-v1.json")).unwrap();
    config["episodes"] = json!(101);
    let p = d.file("budget.json", &serde_json::to_vec(&config).unwrap());
    failure(
        command(&["bench", "--workload", "scalar-pass-v1", "--config", &p]),
        2,
        "smoke limits",
    );
    failure(
        command(&[
            "bench",
            "--workload",
            "foundations_micro_v1",
            "--config",
            &p,
        ]),
        2,
        "unsupported",
    );
}

#[test]
fn conformance_deadline_kills_unresponsive_dependency() {
    use std::os::unix::fs::PermissionsExt;
    let d = Dir::new();
    let (f, a) = comparison(&d);
    // A deliberately wedged process tests supervision only, never agreement.
    // exec replaces the shell, so there is no orphan sleep child.
    let python = d.file("python3", b"#!/bin/sh\nexec /bin/sleep 30\n");
    fs::set_permissions(&python, fs::Permissions::from_mode(0o700)).unwrap();
    let mut c = conformance(&f, &a, &d.path("timeout"));
    c.env("PATH", &d.0);
    failure(c, 4, "timed out");
}
