//! GH-181: real library-produced played datasets; synthetic corruptions are
//! explicitly separate. RFC0002 §8/9 requires strict complete loading and privacy.
use mtg_core::{
    episode::{Budget, Clock, Driver},
    game::{
        Config, DeckConfig,
        policy::{Choice, Submission},
    },
    objects::Seat,
    trajectory::Limits,
};
use mtg_recorder::{
    Backpressure,
    collector::{Bundle, Run, Storage},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
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

const MAX: usize = 16 * 1024 * 1024;
#[derive(Debug)]
struct Zero;
impl Clock for Zero {
    fn now_ms(&self) -> u64 {
        0
    }
}
// CR103 keeps, CR117 two upkeep passes, CR103.8a first draw skipped,
// CR305 one Forest, CR104.3a concession. Five policy decisions per completed
// episode; concession is an external event, not a policy decision.
fn bundle(kinds: &[&str]) -> Bundle {
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
    let r = Run {
        id: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
        config: Config {
            seats: vec![
                DeckConfig {
                    deck: "green".into(),
                    order: Some(order)
                };
                2
            ],
            ..Config::default()
        },
        policies: ["script-v1".into(), "script-v1".into()],
        limits: Limits {
            decisions: Some(6),
            ..Limits::default()
        },
        first_ordinal: 7,
        started: kinds.len() as u64,
    };
    let results: Vec<_> = kinds
        .iter()
        .enumerate()
        .map(|(i, kind)| {
            let mut d = Driver::bounded(
                256,
                Budget {
                    limits: r.limits.clone(),
                    work_quantum: NonZeroUsize::MAX,
                    records: NonZeroUsize::new(if *kind == "failed" { 1 } else { 100 }).unwrap(),
                },
                Box::new(Zero),
            )
            .unwrap();
            let mut h = r.header(7 + i as u64).unwrap();
            if *kind == "completed" {
                h.restricted_replay = Some("00000000-0000-4000-8000-000000000181".into());
            }
            d.reset_captured(&r.config, 181, 7 + i as u64, NonZeroUsize::MAX, &h)
                .unwrap();
            if *kind != "incomplete" {
                for seat in [Seat::P0, Seat::P1] {
                    let domain = d.observe(seat).unwrap().decision.unwrap();
                    let accepted = d.submit(
                        seat,
                        &Submission {
                            schema_version: 1,
                            revision: domain.revision,
                            generation: domain.generation,
                            choices: vec![Choice::Keep],
                        },
                    );
                    if *kind == "failed" && seat == Seat::P1 {
                        assert!(accepted.is_err());
                    } else {
                        accepted.unwrap();
                    }
                }
            }
            if *kind == "completed" || *kind == "truncated" {
                d.advance(NonZeroUsize::MAX).unwrap();
                for (index, seat) in [Seat::P0, Seat::P1, Seat::P0].into_iter().enumerate() {
                    let domain = d.observe(seat).unwrap().decision.unwrap();
                    let choice = if index == 2 {
                        domain
                            .candidates
                            .iter()
                            .find(|c| matches!(c, Choice::PlayLand { .. }))
                            .unwrap()
                            .clone()
                    } else {
                        Choice::Pass
                    };
                    d.submit(
                        seat,
                        &Submission {
                            schema_version: 1,
                            revision: domain.revision,
                            generation: domain.generation,
                            choices: vec![choice],
                        },
                    )
                    .unwrap();
                    d.advance(NonZeroUsize::MAX).unwrap();
                }
                if *kind == "completed" {
                    d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
                }
                if *kind == "truncated" {
                    let domain = d.observe(Seat::P0).unwrap().decision.unwrap();
                    d.submit(
                        Seat::P0,
                        &Submission {
                            schema_version: 1,
                            revision: domain.revision,
                            generation: domain.generation,
                            choices: vec![Choice::Pass],
                        },
                    )
                    .unwrap();
                }
                // The explicit six-decision limit truncates this live game.
            }
            let result = d.finish().unwrap();
            if *kind == "completed" {
                assert_eq!(result.accepted_decisions(), 5);
                let v = serde_json::to_value(result.trajectory().unwrap()).unwrap();
                assert_eq!(v["footer"]["returns"], json!([1, -1]));
                assert_eq!(
                    v["footer"]["final_observations"][0]["view"]["life"],
                    json!([20, 20])
                );
            }
            result
        })
        .collect();
    r.persist(
        &results,
        Storage {
            queue_bytes: MAX,
            max_bytes: MAX,
            backpressure: Backpressure::Block,
        },
    )
    .unwrap()
}
fn files(d: &Dir, b: &Bundle) -> (String, String) {
    (
        d.file("episodes.jsonl", b.bytes()),
        d.file("manifest.json", &b.manifest().encode(MAX).unwrap()),
    )
}
fn v2(path: &str) -> Command {
    command(&[
        "trajectories",
        "validate",
        path,
        "--format",
        "structured-jsonl-v2",
    ])
}
fn manifest(path: &str, m: &str, diagnostic: bool) -> Command {
    let mut c = command(&["trajectories", "validate", path, "--manifest", m]);
    if diagnostic {
        c.arg("--diagnostic");
    }
    c
}
#[test]
fn real_played_v2_and_manifest_have_independently_counted_summaries() {
    let b = bundle(&["completed", "completed"]);
    let d = Dir::new();
    let (p, m) = files(&d, &b);
    assert_eq!(
        success(v2(&p)),
        vec![
            json!({"schema_version":1,"type":"trajectory_validation","status":"valid",
        "format":"structured-jsonl-v2","episodes":2,"decisions":10,"completed":2,"truncated":0})
        ]
    );
    assert_eq!(
        success(manifest(&p, &m, false)),
        vec![
            json!({"schema_version":1,"type":"trajectory_validation","status":"valid",
        "format":"run-manifest-v2","episodes":2,"decisions":10,"declared_episodes":2,
        "completed":2,"truncated":0,"failed":0,"incomplete":0,"run_end":"completed","recording_complete":true})
        ]
    );
    // Opaque replay ID deliberately has no corresponding file or registry grant.
    assert_eq!(fs::read_dir(&d.0).unwrap().count(), 2);
    let out = d.path("summary.json");
    let mut c = v2(&p);
    c.args(["--output", &out]);
    let (code, rows, err) = run(c);
    assert_eq!(code, 0, "{err}");
    assert!(rows.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(out).unwrap()).unwrap()["decisions"],
        10
    );
}
#[test]
fn v1_fixture_and_explicit_v1_and_manifest_remain_supported() {
    let d = Dir::new();
    let p = d.file(
        "episode.jsonl",
        include_bytes!("../../mtg-recorder/tests/episode.jsonl"),
    );
    let m = d.file(
        "manifest.json",
        include_bytes!("../../mtg-recorder/tests/manifest.json"),
    );
    let expected = json!({"schema_version":1,"type":"trajectory_validation","status":"valid","format":"scalar-jsonl-v1","episodes":1,"decisions":1});
    assert_eq!(
        success(command(&["trajectories", "validate", &p])),
        vec![expected.clone()]
    );
    assert_eq!(
        success(command(&[
            "trajectories",
            "validate",
            &p,
            "--format",
            "scalar-jsonl-v1"
        ])),
        vec![expected]
    );
    assert_eq!(
        success(manifest(&p, &m, false))[0]["format"],
        "run-manifest-v1"
    );
}
#[test]
fn diagnostic_reports_failed_and_incomplete_runs_without_completed_prefix_claim() {
    for kind in ["failed", "incomplete", "truncated"] {
        let b = bundle(&["completed", kind]);
        let d = Dir::new();
        let (p, m) = files(&d, &b);
        failure(manifest(&p, &m, false), 2, "trajectory");
        let rows = success(manifest(&p, &m, true));
        let row = &rows[0];
        assert_eq!(row["status"], "valid_noncompleted");
        assert_eq!(row["declared_episodes"], 2);
        assert_eq!(row["completed"], 1);
        assert_ne!(row["run_end"], "completed");
        assert_eq!(row[kind], 1);
        assert_eq!(row["episodes"], if kind == "truncated" { 2 } else { 1 });
        assert_eq!(row["decisions"], if kind == "truncated" { 11 } else { 5 });
        assert_eq!(row["recording_complete"], kind == "truncated");
        if kind == "truncated" {
            failure(v2(&p), 2, "trajectory");
            let mut c = v2(&p);
            c.arg("--diagnostic");
            assert_eq!(success(c)[0]["status"], "valid_noncompleted");
        }
    }
}
fn reject(c: Command) {
    let (code, rows, err) = run(c);
    assert_eq!(code, 2, "{err}");
    assert!(rows.is_empty());
    let e: Value = serde_json::from_str(&err).unwrap();
    assert_eq!(e["schema_version"], 1);
    assert!(e["message"].as_str().unwrap().contains("trajectory"), "{e}");
    for secret in ["PRIVATE", "forest", "a34c952c"] {
        assert!(!err.contains(secret), "{err}");
    }
}
// Synthetic fault injection below mutates real played bytes. Re-sealing uses
// independently specified JSONL SHA256/count rules, never reader-derived output.
fn reseal(rows: &[Value], format: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut decisions = 0;
    for row in rows {
        serde_json::to_writer(&mut bytes, row).unwrap();
        bytes.push(b'\n');
        decisions += row["episode"]["decisions"].as_array().unwrap().len();
    }
    let seal = json!({"kind":"seal","format":format,"episodes":rows.len(),"decisions":decisions,"sha256":format!("{:x}",Sha256::digest(&bytes))});
    serde_json::to_writer(&mut bytes, &seal).unwrap();
    bytes.push(b'\n');
    bytes
}
fn episode_rows(b: &Bundle) -> Vec<Value> {
    b.bytes()
        .split(|c| *c == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice::<Value>(l).unwrap())
        .filter(|v| v["kind"] == "episode")
        .collect()
}
#[test]
fn synthetic_version_duplicate_missing_and_corrupt_data_never_salvage() {
    let b = bundle(&["completed", "completed"]);
    let original = episode_rows(&b);
    let d = Dir::new();
    let mut cases = vec![
        Vec::new(),
        b"PRIVATE malformed".to_vec(),
        b.bytes()[..b.bytes().len() - 10].to_vec(),
    ];
    for field in ["schema", "observation"] {
        for bad in [Value::Null, json!(9)] {
            let mut rows = original.clone();
            rows[1]["episode"]["header"]["versions"][field] = bad;
            cases.push(reseal(&rows, 2));
        }
    }
    let mut rows = original.clone();
    rows[1] = rows[0].clone();
    cases.push(reseal(&rows, 2));
    cases.push(reseal(&original, 1));
    cases.push(reseal(&original, 99));
    let mut rows = original.clone();
    rows[1]["episode"]["header"]["versions"]
        .as_object_mut()
        .unwrap()
        .remove("schema");
    cases.push(reseal(&rows, 2));
    let mut rows = original.clone();
    rows[1]["episode"]["header"]["versions"]["schema"] = json!(1);
    cases.push(reseal(&rows, 2));
    let mut rows = original.clone();
    rows[0]["episode"]["footer"] = Value::Null;
    cases.push(reseal(&rows, 2));
    let mut rows = original.clone();
    rows[0]["episode"]["PRIVATE"] = json!("PRIVATE");
    cases.push(reseal(&rows, 2));
    let mut bad = b.bytes().to_vec();
    bad[15] ^= 1;
    cases.push(bad);
    for bytes in cases {
        let p = d.file("PRIVATE.jsonl", &bytes);
        reject(v2(&p));
        let mut c = v2(&p);
        c.arg("--diagnostic");
        reject(c);
    }
    let p = d.file("PRIVATE.jsonl", b.bytes());
    reject(command(&["trajectories", "validate", &p]));
    reject(command(&[
        "trajectories",
        "validate",
        &p,
        "--format",
        "PRIVATE",
    ]));
}
#[test]
fn synthetic_manifest_mismatches_and_private_reasons_are_redacted() {
    let b = bundle(&["completed", "completed"]);
    let d = Dir::new();
    let (p, m) = files(&d, &b);
    let original = serde_json::to_value(b.manifest()).unwrap();
    for (pointer, value) in [
        ("/dataset_schema", json!(99)),
        ("/versions/schema", json!(1)),
        ("/file/sha256", json!("0".repeat(64))),
        ("/file/bytes", json!(1)),
        ("/file/decisions", json!(9)),
        ("/policies/0", json!("PRIVATE")),
        ("/config_hash", json!("0".repeat(64))),
        ("/file/name", json!("../PRIVATE")),
        ("/file/name", json!("PRIVATE.partial")),
        ("/file/name", json!("other.jsonl")),
        ("/episodes/1/ordinal", json!(7)),
        ("/episodes/1/ordinal", json!(9)),
        ("/episodes/1/status", json!("Truncated")),
    ] {
        let mut v = original.clone();
        *v.pointer_mut(pointer).unwrap() = value;
        fs::write(&m, serde_json::to_vec(&v).unwrap()).unwrap();
        reject(manifest(&p, &m, true));
    }
    let mut v = original.clone();
    v.as_object_mut().unwrap().remove("dataset_schema");
    fs::write(&m, serde_json::to_vec(&v).unwrap()).unwrap();
    reject(manifest(&p, &m, true));
    // Missing second episode with a valid fresh seal and updated file hash/count:
    // declarations still require two; reader must not bless the stored prefix.
    let bytes = reseal(&episode_rows(&b)[..1], 2);
    let mut v = original.clone();
    v["file"]["bytes"] = json!(bytes.len());
    v["file"]["sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    v["file"]["decisions"] = json!(5);
    fs::write(&p, bytes).unwrap();
    fs::write(&m, serde_json::to_vec(&v).unwrap()).unwrap();
    reject(manifest(&p, &m, true));
    let b = bundle(&["completed", "failed"]);
    let (p, m) = files(&d, &b);
    let mut v = serde_json::to_value(b.manifest()).unwrap();
    v["end"] = json!({"Failed":"PRIVATE_REASON"});
    v["episodes"][1]["status"] = json!({"Failed":"PRIVATE_REASON"});
    fs::write(&m, serde_json::to_vec(&v).unwrap()).unwrap();
    let rows = success(manifest(&p, &m, true));
    assert!(!serde_json::to_string(&rows).unwrap().contains("PRIVATE"));
}
#[test]
fn bounded_explicit_file_selection_and_options_fail_safely() {
    let b = bundle(&["completed"]);
    let d = Dir::new();
    let (p, m) = files(&d, &b);
    for limit in ["0", "1", "16777217", "PRIVATE"] {
        let mut c = v2(&p);
        c.args(["--max-bytes", limit]);
        reject(c);
    }
    let exact = b.bytes().len().to_string();
    let mut c = v2(&p);
    c.args(["--max-bytes", &exact]);
    success(c);
    let mut c = manifest(&p, &m, false);
    c.args(["--max-bytes", &exact]);
    reject(c); // combined budget
    for path in [d.path("PRIVATE-missing"), d.0.to_str().unwrap().into()] {
        reject(v2(&path));
    }
    let huge = d.path("PRIVATE-huge");
    fs::File::create(&huge)
        .unwrap()
        .set_len(MAX as u64 + 1)
        .unwrap();
    reject(v2(&huge));
    let mut c = manifest(&p, &m, false);
    c.args(["--format", "structured-jsonl-v2"]);
    reject(c);
    let mut c = v2(&p);
    c.args(["--format", "structured-jsonl-v2"]);
    reject(c);
    let mut c = v2(&p);
    c.arg("--privileged");
    reject(c);
    #[cfg(unix)]
    {
        let link = d.path("PRIVATE-link");
        std::os::unix::fs::symlink(&p, &link).unwrap();
        reject(v2(&link));
    }
}

#[test]
fn library_played_inputs_match_literal_accounting_before_cli() {
    for kind in ["completed", "truncated", "failed", "incomplete"] {
        let b = bundle(&["completed", kind]);
        let m = b.manifest();
        let expected_stored = if matches!(kind, "completed" | "truncated") {
            2
        } else {
            1
        };
        assert_eq!(m.file.episodes, expected_stored);
        assert_eq!(
            m.file.decisions,
            match kind {
                "completed" => 10,
                "truncated" => 11,
                _ => 5,
            }
        );
        let loaded = m
            .load_v2(
                &m.file.name,
                b.bytes(),
                &m.versions,
                MAX,
                mtg_recorder::manifest::LoadMode::Diagnostic,
            )
            .unwrap();
        assert_eq!(
            loaded.episodes()[0]
                .footer
                .as_ref()
                .unwrap()
                .final_observations[0]
                .view
                .hand
                .len(),
            6
        );
        use mtg_recorder::manifest::EpisodeStatus;
        match kind {
            "completed" => assert_eq!(m.episodes[1].status, EpisodeStatus::Completed),
            "truncated" => assert_eq!(m.episodes[1].status, EpisodeStatus::Truncated),
            "incomplete" => assert_eq!(m.episodes[1].status, EpisodeStatus::Incomplete),
            "failed" => assert!(matches!(m.episodes[1].status, EpisodeStatus::Failed(_))),
            _ => unreachable!(),
        }
    }
}

#[test]
fn diagnostic_empty_failed_run_is_metadata_not_a_completed_dataset() {
    let b = bundle(&["failed"]);
    let d = Dir::new();
    let (p, m) = files(&d, &b);
    failure(manifest(&p, &m, false), 2, "trajectory");
    assert_eq!(
        success(manifest(&p, &m, true)),
        vec![json!({"schema_version":1,"type":"trajectory_validation",
        "status":"valid_noncompleted","format":"run-manifest-v2","episodes":0,"decisions":0,
        "declared_episodes":1,"completed":0,"truncated":0,"failed":1,"incomplete":0,
        "run_end":"failed","recording_complete":false})]
    );
}
#[test]
fn synthetic_seal_count_duplicate_version_and_unsealed_manifest_reject() {
    let b = bundle(&["completed"]);
    let d = Dir::new();
    let (p, m) = files(&d, &b);
    let rows = episode_rows(&b);
    let bytes = reseal(&rows, 2);
    let split = bytes
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == b'\n')
        .map(|(i, _)| i)
        .next()
        .unwrap()
        + 1;
    let seal: Value = serde_json::from_slice(&bytes[split..]).unwrap();
    let mut corruptions = vec![bytes[..split].to_vec()]; // whole episode, absent seal
    for field in ["episodes", "decisions", "sha256", "format"] {
        let mut v = seal.clone();
        v.as_object_mut().unwrap().remove(field);
        let mut bad = bytes[..split].to_vec();
        serde_json::to_writer(&mut bad, &v).unwrap();
        bad.push(b'\n');
        corruptions.push(bad);
    }
    for field in ["episodes", "decisions"] {
        let mut v = seal.clone();
        v[field] = json!(0);
        let mut bad = bytes[..split].to_vec();
        serde_json::to_writer(&mut bad, &v).unwrap();
        bad.push(b'\n');
        corruptions.push(bad);
    }
    let text = String::from_utf8(bytes.clone()).unwrap();
    corruptions.push(
        text.replacen("\"format\":2", "\"format\":2,\"format\":2", 1)
            .into_bytes(),
    );
    corruptions.push(
        text.replacen("\"schema\":2", "\"schema\":2,\"schema\":2", 1)
            .into_bytes(),
    );
    for bad in corruptions {
        fs::write(&p, &bad).unwrap();
        let mut c = v2(&p);
        c.arg("--diagnostic");
        reject(c);
        // Whole-file checksum matches the corrupt bytes; inner reader must still reject.
        let mut v = serde_json::to_value(b.manifest()).unwrap();
        v["file"]["bytes"] = json!(bad.len());
        v["file"]["sha256"] = json!(format!("{:x}", Sha256::digest(&bad)));
        fs::write(&m, serde_json::to_vec(&v).unwrap()).unwrap();
        reject(manifest(&p, &m, true));
    }
    let encoded = String::from_utf8(b.manifest().encode(MAX).unwrap()).unwrap();
    let duplicate = encoded.replacen(
        "\"dataset_schema\":2",
        "\"dataset_schema\":2,\"dataset_schema\":2",
        1,
    );
    fs::write(&p, b.bytes()).unwrap();
    fs::write(&m, duplicate).unwrap();
    reject(manifest(&p, &m, true));
}
