//! GH-180: strict local-file routing, privacy and replay completion.
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

// Hand-authored normal-reset trace: CR 103 keeps, CR 117 upkeep passes,
// CR 103.8a skips the first draw, CR 305 land play, CR 104.3a concession.
// Full frozen green deck, lands first. No fixture-state mutation or policy oracle.
fn played() -> Vec<u8> {
    use mtg_core::opening::{DeckConfig, actions};
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
    .flat_map(|(k, n)| vec![k.to_string(); n])
    .collect();
    let config = Config {
        seats: vec![
            DeckConfig {
                deck: "green".into(),
                order: Some(order)
            };
            2
        ],
        ..Config::default()
    };
    let records: Vec<actions::Record> = serde_json::from_value(json!([
        {"version":1,"actor":"P0","decision":"keep_or_mulligan","choices":[{"kind":"keep"}]},
        {"version":1,"actor":"P1","decision":"keep_or_mulligan","choices":[{"kind":"keep"}]},
        {"version":1,"actor":"P0","decision":"priority","choices":[{"kind":"pass"}]},
        {"version":1,"actor":"P1","decision":"priority","choices":[{"kind":"pass"}]},
        {"version":1,"actor":"P0","decision":"priority","choices":[{"kind":"play_land","card":{"birth":0,"card":"forest","owner":"P0","zone":{"Hand":"P0"},"incarnation":1}}]},
        {"version":1,"actor":"P1","decision":"concession","choices":[{"kind":"concede"}]}
    ])).unwrap();
    let bytes = replay::played::record(&config, 180, 0, &records).unwrap();
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    // Literal independent checkpoints: no life change, land leaves P0's hand,
    // opponent concedes without changing life, winner P0 (not a life-loss win).
    assert_eq!(v["initial"]["life"], json!([20, 20]));
    assert_eq!(
        v["choices"][4]["after"]["opening"]["hands"][0]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        v["choices"][5]["after"]["outcome"],
        json!({"winner":"P0","losses":[null,"Concession"]})
    );
    bytes
}
fn opening() -> Vec<u8> {
    replay::record(
        &Config::default(),
        180,
        0,
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
fn rejected(bytes: &[u8], kind: &str) {
    let d = Dir::new();
    let path = d.file("PRIVATE_PATH.json", bytes);
    let (code, rows, err) = run(command(&["replay", "verify", &path]));
    assert_eq!(code, 2, "{err}");
    assert!(rows.is_empty());
    let e: Value = serde_json::from_str(&err).unwrap();
    assert_eq!(
        e,
        json!({"schema_version":1,"type":"error","exit_code":2,
        "message":format!("replay {kind}; privileged detail withheld")})
    );
    for secret in [
        "PRIVATE_PATH",
        "PRIVATE_PAYLOAD",
        "PRIVATE_HAND",
        "PRIVATE_FIELD",
    ] {
        assert!(!err.contains(secret));
    }
}
#[test]
fn played_and_opening_verify_with_literal_public_summary() {
    let d = Dir::new();
    let path = d.file("played.json", &played());
    let rows = success(command(&["replay", "verify", &path]));
    assert_eq!(
        rows,
        vec![json!({"schema_version":1,"type":"replay_verification",
        "status":"verified","scope":"played-v1","checkpoint":"terminal",
        "life":[20,20],"outcome":{"winner":"P0","losses":[null,"Concession"]}})]
    );
    let output = d.path("summary.jsonl");
    let (code, stdout, err) = run(command(&["replay", "verify", &path, "--output", &output]));
    assert_eq!(code, 0, "{err}");
    assert!(stdout.is_empty() && err.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(output).unwrap()).unwrap(),
        rows[0]
    );
    let path = d.file("opening.json", &opening());
    assert_eq!(
        success(command(&["replay", "verify", &path])),
        vec![json!({"schema_version":1,
        "type":"replay_verification","status":"verified","scope":"opening-v1"})]
    );
}
#[test]
fn routing_rejects_unsupported_missing_and_duplicate_discriminators() {
    for bytes in [played(), opening()] {
        let original: Value = serde_json::from_slice(&bytes).unwrap();
        for (field, value) in [("format", json!("PRIVATE_PAYLOAD")), ("version", json!(2))] {
            let mut v = original.clone();
            v[field] = value;
            rejected(&serde_json::to_vec(&v).unwrap(), "Incompatible");
        }
        for field in ["format", "version"] {
            let mut v = original.clone();
            v.as_object_mut().unwrap().remove(field);
            rejected(&serde_json::to_vec(&v).unwrap(), "Malformed");
            let duplicate = format!(
                "{{\"{field}\":{},{}",
                original[field],
                &String::from_utf8(bytes.clone()).unwrap()[1..]
            );
            rejected(duplicate.as_bytes(), "Malformed");
        }
        let mut v = original.clone();
        v["format"] = json!(if original["format"] == "mtg-core-opening-replay" {
            "mtg-core-played-replay"
        } else {
            "mtg-core-opening-replay"
        });
        rejected(&serde_json::to_vec(&v).unwrap(), "Malformed");
    }
}
#[test]
fn history_and_semantic_corruptions_fail_without_private_details() {
    let original: Value = serde_json::from_slice(&played()).unwrap();
    let mut v = original.clone();
    v["choices"].as_array_mut().unwrap().pop();
    rejected(&serde_json::to_vec(&v).unwrap(), "MissingChoice"); // land isn't terminal
    let mut v = original.clone();
    v["choices"]
        .as_array_mut()
        .unwrap()
        .push(original["choices"][5].clone());
    rejected(&serde_json::to_vec(&v).unwrap(), "UnconsumedChoice"); // action after concession
    let mut v = original.clone();
    v["choices"][4]["choice"]["choices"][0]["card"]["incarnation"] = json!(99);
    rejected(&serde_json::to_vec(&v).unwrap(), "SemanticChoice"); // stale land incarnation
    for pointer in ["/initial/life/0", "/choices/4/after/life/1"] {
        let mut v = original.clone();
        *v.pointer_mut(pointer).unwrap() = json!(19);
        rejected(&serde_json::to_vec(&v).unwrap(), "Divergence"); // no damage or costs occurred
    }
    let mut v = original;
    v["initial"]["PRIVATE_FIELD"] = json!("PRIVATE_PAYLOAD");
    rejected(&serde_json::to_vec(&v).unwrap(), "Divergence");
    let mut v: Value = serde_json::from_slice(&opening()).unwrap();
    v["initial"]["hands"][1][0] = json!("PRIVATE_HAND");
    rejected(&serde_json::to_vec(&v).unwrap(), "Divergence");
}
#[test]
fn bounded_malformed_input_and_inspection_do_not_enable_privilege() {
    for bytes in [b"{".as_slice(), b"{}", b"null", b"\xff", b"[]"] {
        rejected(bytes, "Malformed");
    }
    let bytes = played();
    rejected(&bytes[..bytes.len() - 1], "Malformed");
    let d = Dir::new();
    let path = d.path("PRIVATE_PATH.json");
    let f = fs::File::create(&path).unwrap();
    f.set_len(16 * 1024 * 1024 + 1).unwrap();
    failure(command(&["replay", "verify", &path]), 2, "byte limit");
    fs::remove_file(&path).unwrap();
    failure(command(&["replay", "verify", &path]), 2, "input");
    failure(
        command(&["replay", "verify", d.0.to_str().unwrap()]),
        2,
        "regular file",
    );
    let path = d.file("played.json", &bytes);
    failure(
        command(&[
            "replay", "inspect", &path, "--seat", "0", "--format", "jsonl",
        ]),
        2,
        "opening-v1 only",
    );
    failure(
        command(&["replay", "verify", &path, "--privileged"]),
        2,
        "usage:",
    );
}
