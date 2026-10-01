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

fn captured(c: &mut Value, root: &std::path::Path) {
    fs::create_dir_all(root.join("datasets")).unwrap();
    fs::create_dir_all(root.join("private")).unwrap();
    c["capture"] = json!({"dataset_root":root.join("datasets"),"replay_root":root.join("private"),"authorization":"local-owner-v1","backpressure":"fail","max_episodes":4,"queue_bytes":16777216,"max_bytes":16777216});
}
#[test]
fn captured_script_publishes_canonical_manifest_and_independent_ledger() {
    let root = std::env::temp_dir().join(format!("capture-positive-{}", std::process::id()));
    let mut c = config();
    captured(&mut c, &root);
    let (code, rows, errors) = run_game(c);
    assert_eq!(code, 0, "{errors}");
    assert_eq!(rows[1]["life"], json!([20, 15]));
    assert_eq!(rows[1]["decisions"], 101);
    assert_eq!(rows[1]["winner"], 0);
    let p = rows.iter().find(|r| r["type"] == "publication").unwrap();
    assert_eq!(p["status"], "published");
    let id = p["run_id"].as_str().unwrap();
    let bytes = fs::read(root.join("datasets").join(id).join("manifest.json")).unwrap();
    let m = mtg_recorder::manifest::Manifest::parse(&bytes[..], 16777216).unwrap();
    let data = fs::read(root.join("datasets").join(id).join("episodes.jsonl")).unwrap();
    let loaded = m
        .load_v2(
            "episodes.jsonl",
            &data[..],
            &m.versions,
            16777216,
            mtg_recorder::manifest::LoadMode::CompletedOnly,
        )
        .unwrap();
    assert_eq!(loaded.episodes().len(), 1);
    assert_eq!(loaded.episodes()[0].decisions.len(), 101);
    let replay = loaded.episodes()[0]
        .header
        .restricted_replay
        .as_ref()
        .unwrap();
    // Deliberate local owner file read. ID knowledge never invokes a resolver.
    let game = mtg_core::opening::replay::played::verify(
        &fs::read(root.join("private").join(id).join(replay)).unwrap(),
    )
    .unwrap();
    assert_eq!(
        game.outcome().unwrap().winner,
        Some(mtg_core::objects::Seat::P0)
    );
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn captured_storage_failure_does_not_reclassify_completed_game() {
    let root = std::env::temp_dir().join(format!("capture-limit-{}", std::process::id()));
    let mut c = config();
    captured(&mut c, &root);
    c["capture"]["max_bytes"] = json!(1);
    let (code, rows, _) = run_game(c);
    assert_eq!(code, 3);
    assert_eq!(rows[1]["status"], "completed");
    let p = rows.iter().find(|r| r["type"] == "publication").unwrap();
    assert_eq!(p["status"], "failed");
    let summary = rows.last().unwrap();
    assert_eq!(summary["completed"], 1);
    assert_eq!(summary["failed"], 0);
    assert_eq!(fs::read_dir(root.join("datasets")).unwrap().count(), 0);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn invalid_capture_configuration_has_no_output_or_storage_mutation() {
    let root = std::env::temp_dir().join(format!("capture-invalid-{}", std::process::id()));
    for mutation in 0..8 {
        let mut c = config();
        captured(&mut c, &root);
        match mutation {
            0 => c["capture"]["replay_root"] = c["capture"]["dataset_root"].clone(),
            1 => c["capture"]["replay_root"] = json!(root),
            2 => c["capture"]["dataset_root"] = json!(root.join("missing")),
            3 => c["capture"]["authorization"] = json!("deny"),
            4 => c["capture"]["queue_bytes"] = json!(0),
            5 => c["capture"]["max_bytes"] = json!(0),
            6 => c["capture"]["max_episodes"] = json!(0),
            _ => c["episodes"] = json!(5),
        }
        let output = root.join("output.jsonl");
        let (code, rows, err) = run(c, &["--output", output.to_str().unwrap()]);
        assert_eq!(code, 2);
        assert!(rows.is_empty());
        assert!(!output.exists());
        assert!(!err.contains(root.to_str().unwrap()));
        assert_eq!(fs::read_dir(root.join("datasets")).unwrap().count(), 0);
        assert_eq!(fs::read_dir(root.join("private")).unwrap().count(), 0);
    }
    // Canonicalization rejects aliases as well as lexical overlap.
    #[cfg(unix)]
    {
        let mut c = config();
        captured(&mut c, &root);
        std::os::unix::fs::symlink(root.join("datasets"), root.join("alias")).unwrap();
        c["capture"]["replay_root"] = json!(root.join("alias"));
        assert_eq!(run_game(c).0, 2);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn captured_native_both_seats_publish_and_match_uncaptured_game_rows() {
    for seat in 0..2 {
        let root =
            std::env::temp_dir().join(format!("capture-native-{}-{seat}", std::process::id()));
        let mut c: Value =
            serde_json::from_str(include_str!("../../../fixtures/simulate/native-v2.json"))
                .unwrap();
        c["episodes"] = json!(1);
        c["game"]["starting_seat"] = json!(seat);
        let (off_code, off, _) = run_game(c.clone());
        assert_eq!(off_code, 0);
        captured(&mut c, &root);
        // Full games can exceed the script fixture's byte budget.
        c["capture"]["max_bytes"] = json!(67108864);
        c["capture"]["queue_bytes"] = json!(67108864);
        let (code, on, errors) = run_game(c);
        assert_eq!(code, 0, "{errors}");
        assert_eq!(on[1], off[1]);
        assert_eq!(on[1]["status"], "completed");
        let p = on.iter().find(|r| r["type"] == "publication").unwrap();
        assert_eq!(p["status"], "published");
        let id = p["run_id"].as_str().unwrap();
        let m = mtg_recorder::manifest::Manifest::parse(
            &fs::read(root.join("datasets").join(id).join("manifest.json")).unwrap()[..],
            67108864,
        )
        .unwrap();
        let data = fs::read(root.join("datasets").join(id).join("episodes.jsonl")).unwrap();
        let loaded = m
            .load_v2(
                "episodes.jsonl",
                &data[..],
                &m.versions,
                67108864,
                mtg_recorder::manifest::LoadMode::CompletedOnly,
            )
            .unwrap();
        assert_eq!(
            loaded.episodes()[0].decisions.len() as u64,
            on[1]["decisions"].as_u64().unwrap()
        );
        let replay = loaded.episodes()[0]
            .header
            .restricted_replay
            .as_ref()
            .unwrap();
        mtg_core::opening::replay::played::verify(
            &fs::read(root.join("private").join(id).join(replay)).unwrap(),
        )
        .unwrap();
        assert!(
            !serde_json::to_string(&on)
                .unwrap()
                .contains(root.to_str().unwrap())
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn captured_os_signals_preserve_requested_accounting() {
    use std::io::{BufRead, BufReader};
    for (signal, code) in [("-TERM", 143), ("-INT", 130)] {
        let root =
            std::env::temp_dir().join(format!("capture-signal-{}-{signal}", std::process::id()));
        let mut c: Value =
            serde_json::from_str(include_str!("../../../fixtures/simulate/native-v2.json"))
                .unwrap();
        captured(&mut c, &root);
        c["capture"]["max_episodes"] = json!(100);
        c["episodes"] = json!(100);
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
        assert_eq!(n("started") + n("not_started"), 100);
        assert_eq!(n("started"), r.len() as u64 - 3);
        assert!(n("not_started") > 0);
        assert_eq!(s["publication"], "failed");
        fs::remove_dir_all(root).unwrap();
    }
}
