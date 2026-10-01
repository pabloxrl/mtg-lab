//! GH-120 composed producer/validator/replay audit. Independent expectations are
//! the retained CR-referenced seed-178 script ledger, not generated golden data.
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
            "mtg-datasets-{}-{}",
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
    let until = Instant::now() + Duration::from_secs(60);
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

const MAX: usize = 16 * 1024 * 1024;
fn config(d: &Dir) -> Value {
    let mut c: Value =
        serde_json::from_str(include_str!("../../../fixtures/simulate/script-v3.json")).unwrap();
    fs::create_dir(d.0.join("data")).unwrap();
    fs::create_dir(d.0.join("private")).unwrap();
    c["capture"] = json!({"dataset_root":d.0.join("data"),"replay_root":d.0.join("private"),
        "authorization":"local-owner-v1","backpressure":"block","max_episodes":4,
        "queue_bytes":MAX,"max_bytes":MAX});
    c
}
fn simulate(d: &Dir, c: &Value) -> (i32, Vec<Value>, String) {
    let p = d.file("config.json", &serde_json::to_vec(c).unwrap());
    run(command(&["simulate", "--config", &p]))
}
fn paths(d: &Dir, rows: &[Value]) -> (String, String, String) {
    let p = rows.iter().find(|r| r["type"] == "publication").unwrap();
    assert_eq!(p["status"], "published");
    let id = p["run_id"].as_str().unwrap();
    (
        d.path(&format!("data/{id}/episodes.jsonl")),
        d.path(&format!("data/{id}/manifest.json")),
        id.into(),
    )
}
fn validate(data: &str, manifest: &str, diagnostic: bool) -> Command {
    let mut c = command(&["trajectories", "validate", data, "--manifest", manifest]);
    if diagnostic {
        c.arg("--diagnostic");
    }
    c
}
fn accounting(rows: &[Value], requested: u64) {
    let s = rows.last().unwrap();
    let n = |k: &str| s[k].as_u64().unwrap();
    assert_eq!(n("requested"), requested);
    assert_eq!(n("started") + n("not_started"), requested);
    assert_eq!(
        n("started"),
        n("completed") + n("truncated") + n("failed") + n("incomplete")
    );
    assert_eq!(
        n("started") as usize,
        rows.iter().filter(|r| r["type"] == "episode").count()
    );
}
fn load(
    data: &str,
    manifest: &str,
) -> mtg_recorder::manifest::LoadedRun<mtg_recorder::structured::Episode> {
    let m = mtg_recorder::manifest::Manifest::parse(&fs::read(manifest).unwrap()[..], MAX).unwrap();
    m.load_v2(
        "episodes.jsonl",
        &fs::read(data).unwrap()[..],
        &m.versions,
        MAX,
        mtg_recorder::manifest::LoadMode::CompletedOnly,
    )
    .unwrap()
}

#[test]
fn simulate_to_validate_and_replay_preserves_literal_script_and_exact_capture() {
    let d = Dir::new();
    let c = config(&d);
    let (code, rows, err) = simulate(&d, &c);
    assert_eq!(code, 0, "{err}");
    assert!(err.is_empty());
    accounting(&rows, 1);
    let e = &rows[1];
    // Original reviewed script: Cub and Growth deal five damage; concession
    // ends the game without another policy decision or life change.
    assert_eq!(e["life"], json!([20, 15]));
    assert_eq!(e["hand_counts"], json!([5, 7]));
    assert_eq!(e["library_counts"], json!([31, 31]));
    assert_eq!(e["decisions"], 101);
    assert_eq!(e["script_consumed"], 102);
    assert_eq!(e["winner"], 0);
    let (data, manifest, id) = paths(&d, &rows);
    assert_eq!(
        success(validate(&data, &manifest, false)),
        vec![json!({
        "schema_version":1,"type":"trajectory_validation","status":"valid",
        "format":"run-manifest-v2","episodes":1,"decisions":101,"declared_episodes":1,
        "completed":1,"truncated":0,"failed":0,"incomplete":0,"run_end":"completed",
        "recording_complete":true})]
    );
    assert_eq!(
        success(command(&[
            "trajectories",
            "validate",
            &data,
            "--format",
            "structured-jsonl-v2"
        ])),
        vec![
            json!({"schema_version":1,"type":"trajectory_validation","status":"valid",
        "format":"structured-jsonl-v2","episodes":1,"decisions":101,"completed":1,"truncated":0})
        ]
    );
    let loaded = load(&data, &manifest);
    let episode = &loaded.episodes()[0];
    let replay_id = episode.header.restricted_replay.as_ref().unwrap();
    let replay = d.path(&format!("private/{id}/{replay_id}"));
    assert_eq!(
        success(command(&["replay", "verify", &replay])),
        vec![json!({
        "schema_version":1,"type":"replay_verification","status":"verified","scope":"played-v1",
        "checkpoint":"terminal","life":[20,15],"outcome":{"winner":"P0","losses":[null,"Concession"]}})]
    );
    let replay_value: Value = serde_json::from_slice(&fs::read(&replay).unwrap()).unwrap();
    for (actual, expected) in replay_value["choices"]
        .as_array()
        .unwrap()
        .iter()
        .zip(c["script"]["records"].as_array().unwrap())
    {
        assert_eq!(
            actual["choice"],
            serde_json::from_str::<Value>(expected["record"].as_str().unwrap()).unwrap()
        );
    }
    assert_eq!(replay_value["choices"].as_array().unwrap().len(), 102);
    compare_owned_reference(&c, &id, episode, &replay);
    let mut off = c.clone();
    off.as_object_mut().unwrap().remove("capture");
    let (code, uncaptured, err) = simulate(&d, &off);
    assert_eq!(code, 0, "{err}");
    assert_eq!(uncaptured[1], rows[1]);
    // Removing private replay bytes must not affect policy dataset validation:
    // an opaque reference is neither a file resolver nor an authority grant.
    fs::remove_file(&replay).unwrap();
    assert_eq!(
        success(validate(&data, &manifest, false))[0]["status"],
        "valid"
    );
    failure(command(&["replay", "verify", &replay]), 2, "replay");
}

// Re-execute literal inputs with the production owner, independently of CLI
// routing, and compare ALL canonical fields to bytes produced in another process.
// This is equality evidence; the literal ledger above is the rules oracle.
fn compare_owned_reference(
    c: &Value,
    id: &str,
    actual: &mtg_recorder::structured::Episode,
    replay: &str,
) {
    use mtg_core::{
        episode::{Budget, Clock, Driver, Progress, RecordContext},
        objects::Seat,
    };
    use std::num::NonZeroUsize;
    #[derive(Debug)]
    struct Zero;
    impl Clock for Zero {
        fn now_ms(&self) -> u64 {
            0
        }
    }
    let run = mtg_recorder::collector::Run {
        id: id.into(),
        config: serde_json::from_value(c["game"].clone()).unwrap(),
        policies: serde_json::from_value(c["policies"].clone()).unwrap(),
        limits: mtg_core::trajectory::Limits {
            decisions: Some(c["max_decisions"].as_u64().unwrap()),
            ..Default::default()
        },
        first_ordinal: 0,
        started: 1,
    };
    let q = NonZeroUsize::new(c["native"]["work_quantum"].as_u64().unwrap() as usize).unwrap();
    let mut owner = Driver::bounded(
        256,
        Budget {
            limits: run.limits.clone(),
            work_quantum: q,
            records: NonZeroUsize::new(c["native"]["max_records"].as_u64().unwrap() as usize)
                .unwrap(),
        },
        Box::new(Zero),
    )
    .unwrap();
    owner
        .reset_captured(&run.config, 178, 0, q, &run.header(0).unwrap())
        .unwrap();
    for entry in c["script"]["records"].as_array().unwrap() {
        for step in 0..10000 {
            if owner.advance(q).unwrap() != Progress::InternalYield {
                break;
            }
            assert!(step < 9999, "bounded settlement");
        }
        let seat: Seat = serde_json::from_value(entry["seat"].clone()).unwrap();
        let record = entry["record"].as_str().unwrap();
        let r: Value = serde_json::from_str(record).unwrap();
        let context = if r["decision"] == "concession" {
            RecordContext::Concession {
                episode: owner.episode_id().unwrap(),
            }
        } else {
            let choice = owner.observe(seat).unwrap().decision.unwrap();
            RecordContext::Decision {
                revision: choice.revision,
                generation: choice.generation,
            }
        };
        owner
            .submit_record(seat, context, record.as_bytes())
            .unwrap();
    }
    let mut result = owner.finish().unwrap();
    let mut registry = mtg_core::episode::replay::Registry::default();
    registry
        .register(
            actual.header.restricted_replay.as_ref().unwrap(),
            &mut result,
        )
        .unwrap();
    assert_eq!(
        *actual,
        mtg_recorder::from_core_v2(result.trajectory().unwrap()).unwrap()
    );
    assert_eq!(
        result.trajectory().unwrap().footer().unwrap().returns,
        [1, -1]
    );
    let game = mtg_core::opening::replay::played::verify(&fs::read(replay).unwrap()).unwrap();
    assert_eq!(
        normalized(&game.snapshot()),
        normalized(result.privileged_snapshot())
    );
}
fn normalized(bytes: &[u8]) -> Value {
    fn scrub(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "scope" || k == "store" {
                        *v = json!(0);
                    } else {
                        scrub(v);
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    scrub(v);
                }
            }
            _ => (),
        }
    }
    let envelope: Value = serde_json::from_slice(bytes).unwrap();
    let mut state: Value = serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
    state["objects"]["id"] = json!(0);
    scrub(&mut state);
    state
}

#[test]
fn composed_diagnostics_reject_completed_claim_and_account_never_started_requests() {
    for mode in ["missing", "illegal", "decisions", "work", "records"] {
        let d = Dir::new();
        let mut c = config(&d);
        c["episodes"] = json!(3);
        match mode {
            "missing" => {
                c["script"]["records"].as_array_mut().unwrap().truncate(4);
            }
            "illegal" => {
                let mut record: Value =
                    serde_json::from_str(c["script"]["records"][4]["record"].as_str().unwrap())
                        .unwrap();
                record["choices"][0]["card"]["incarnation"] = json!(2);
                c["script"]["records"][4]["record"] = json!(record.to_string());
            }
            "decisions" => c["max_decisions"] = json!(4),
            "work" => c["native"]["max_work_calls"] = json!(1),
            "records" => c["native"]["max_records"] = json!(1),
            _ => unreachable!(),
        }
        let (code, rows, err) = simulate(&d, &c);
        assert_eq!(
            code,
            if ["missing", "illegal", "records"].contains(&mode) {
                3
            } else {
                0
            },
            "{mode}: {err}"
        );
        accounting(&rows, 3);
        assert_eq!(rows.last().unwrap()["started"], 1);
        assert_eq!(rows.last().unwrap()["not_started"], 2);
        assert_eq!(rows.last().unwrap()["completed"], 0);
        if ["missing", "illegal"].contains(&mode) {
            // Four accepted opening/upkeep choices; rejected land leaves the
            // seven-card hand, life, history and script cursor unchanged.
            assert_eq!(rows[1]["decisions"], 4);
            assert_eq!(rows[1]["script_consumed"], 4);
            assert_eq!(rows[1]["life"], json!([20, 20]));
            assert_eq!(rows[1]["hand_counts"], json!([7, 7]));
            assert_eq!(rows[1]["owner_status"], "incomplete");
        }
        let (data, manifest, id) = paths(&d, &rows);
        failure(validate(&data, &manifest, false), 2, "trajectory");
        let summary = success(validate(&data, &manifest, true));
        assert_eq!(summary[0]["status"], "valid_noncompleted");
        assert_eq!(summary[0]["declared_episodes"], 1);
        assert_eq!(summary[0]["completed"], 0);
        let status = rows[1]["owner_status"].as_str().unwrap();
        assert_eq!(summary[0][status], 1);
        assert_eq!(
            fs::read_dir(d.0.join("private").join(id)).unwrap().count(),
            0
        );
    }
}

#[test]
fn composed_writer_failure_cannot_be_validated_as_completed_capture() {
    let d = Dir::new();
    let mut c = config(&d);
    // Three explicit concessions are rules outcomes, never writer failures.
    let template = c["script"]["records"][0].clone();
    c["script"]["records"] = json!(
        (0..3)
            .map(|episode| {
                let mut e = template.clone();
                e["episode"] = json!(episode);
                e["decision"] = json!(0);
                e["seat"] = json!("P0");
                e["record"] = json!(
                    json!({"version":1,"actor":"P0",
            "decision":"concession","choices":[{"kind":"concede"}]})
                    .to_string()
                );
                e
            })
            .collect::<Vec<_>>()
    );
    c["episodes"] = json!(3);
    c["capture"]["max_bytes"] = json!(1);
    let (code, rows, err) = simulate(&d, &c);
    assert_eq!(code, 3, "{err}");
    accounting(&rows, 3);
    assert_eq!(rows.last().unwrap()["completed"], 3);
    assert_eq!(rows.last().unwrap()["failed"], 0);
    assert_eq!(rows.last().unwrap()["publication"], "failed");
    let id = rows[4]["run_id"].as_str().unwrap();
    let data = d.path(&format!("data/{id}/episodes.jsonl"));
    let manifest = d.path(&format!("data/{id}/manifest.json"));
    assert!(!PathBuf::from(&manifest).exists());
    failure(validate(&data, &manifest, true), 2, "trajectory");
}

#[test]
fn native_captured_games_feed_both_verifiers_from_both_starting_seats() {
    for seat in 0..2 {
        let d = Dir::new();
        let capture = config(&d)["capture"].clone();
        let mut c: Value =
            serde_json::from_str(include_str!("../../../fixtures/simulate/native-v2.json"))
                .unwrap();
        c["episodes"] = json!(1);
        c["game"]["starting_seat"] = json!(seat);
        c["capture"] = capture;
        let (code, rows, err) = simulate(&d, &c);
        assert_eq!(code, 0, "{err}");
        accounting(&rows, 1);
        assert_eq!(rows[1]["status"], "completed");
        let (data, manifest, id) = paths(&d, &rows);
        let summary = success(validate(&data, &manifest, false));
        assert_eq!(summary[0]["completed"], 1);
        assert_eq!(summary[0]["decisions"], rows[1]["decisions"]);
        let loaded = load(&data, &manifest);
        let replay = d.path(&format!(
            "private/{id}/{}",
            loaded.episodes()[0]
                .header
                .restricted_replay
                .as_ref()
                .unwrap()
        ));
        let verified = success(command(&["replay", "verify", &replay]));
        assert_eq!(verified[0]["life"], rows[1]["life"]);
        assert_eq!(verified[0]["status"], "verified");
        c.as_object_mut().unwrap().remove("capture");
        let (code, off, err) = simulate(&d, &c);
        assert_eq!(code, 0, "{err}");
        assert_eq!(rows[1], off[1]);
    }
}

#[test]
fn captured_deadline_and_sigterm_leave_no_false_completed_artifacts() {
    use std::io::{BufRead, BufReader};
    for signal in [false, true] {
        let d = Dir::new();
        let capture = config(&d)["capture"].clone();
        let mut c: Value =
            serde_json::from_str(include_str!("../../../fixtures/simulate/native-v2.json"))
                .unwrap();
        c["capture"] = capture;
        c["episodes"] = json!(100);
        c["capture"]["max_episodes"] = json!(100);
        let (code, rows) = if signal {
            let p = d.file("config.json", &serde_json::to_vec(&c).unwrap());
            let mut child = command(&["simulate", "--config", &p]).spawn().unwrap();
            let stdout = child.stdout.take().unwrap();
            let mut stderr = child.stderr.take().unwrap();
            let errors = thread::spawn(move || {
                let mut s = String::new();
                stderr.read_to_string(&mut s).unwrap();
                s
            });
            let (tx, rx) = std::sync::mpsc::channel();
            let output = thread::spawn(move || {
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
                    panic!("SIGTERM timeout");
                }
                thread::sleep(Duration::from_millis(5));
            };
            assert!(errors.join().unwrap().is_empty());
            (status.code().unwrap(), output.join().unwrap())
        } else {
            c["deadline_ms"] = json!(1);
            let (code, rows, err) = simulate(&d, &c);
            assert!(err.is_empty());
            (code, rows)
        };
        assert_eq!(code, if signal { 143 } else { 4 });
        accounting(&rows, 100);
        let summary = rows.last().unwrap();
        assert!(summary["not_started"].as_u64().unwrap() > 0);
        assert_eq!(summary["publication"], "failed");
        let p = rows.iter().find(|r| r["type"] == "publication").unwrap();
        assert_eq!(p["reason"], "interrupted");
        let id = p["run_id"].as_str().unwrap();
        let data = d.path(&format!("data/{id}/episodes.jsonl"));
        let manifest = d.path(&format!("data/{id}/manifest.json"));
        assert!(!PathBuf::from(&manifest).exists());
        failure(validate(&data, &manifest, true), 2, "trajectory");
    }
}
