use super::*;
use mtg_recorder::manifest::{LoadMode, Manifest};
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Root(PathBuf);
impl Root {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mtg-capture-unit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(p.join("data")).unwrap();
        fs::create_dir_all(p.join("replay")).unwrap();
        Self(p)
    }
    fn config(&self, script: bool) -> Simulation {
        let mut v: Value = serde_json::from_str(if script {
            include_str!("../../../fixtures/simulate/script-v3.json")
        } else {
            include_str!("../../../fixtures/simulate/native-v2.json")
        })
        .unwrap();
        v["episodes"] = json!(1);
        v["capture"] = json!({"dataset_root":self.0.join("data"),"replay_root":self.0.join("replay"),"authorization":"local-owner-v1","backpressure":"fail","max_episodes":4,"queue_bytes":67108864,"max_bytes":67108864});
        serde_json::from_value(v).unwrap()
    }
}
impl Drop for Root {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn rows(bytes: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect()
}
fn short(c: &mut Simulation) {
    let s = c.script.as_mut().unwrap();
    s.records.truncate(1);
    s.records[0].seat = mtg_core::objects::Seat::P0;
    s.records[0].record =
        json!({"version":1,"actor":"P0","decision":"concession","choices":[{"kind":"concede"}]})
            .to_string();
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

#[test]
fn exact_owned_rows_replay_authority_and_capture_on_off_full_state_rng() {
    for script in [true, false] {
        let root = Root::new();
        let mut c = root.config(script);
        if script {
            c.capture.as_mut().unwrap().backpressure = QueueMode::Block;
        }
        let mut owned = vec![];
        let mut out = vec![];
        assert_eq!(
            crate::native::run_observed(&c, &mut out, || None, &mut |_, _| Ok(()), &mut |r| owned
                .push(r.clone()))
            .unwrap(),
            0
        );
        let r = rows(&out);
        let run = r.iter().find(|r| r["type"] == "publication").unwrap()["run_id"]
            .as_str()
            .unwrap();
        let path = root.0.join("data").join(run);
        let m =
            Manifest::parse(&fs::read(path.join("manifest.json")).unwrap()[..], 67108864).unwrap();
        let data = fs::read(path.join("episodes.jsonl")).unwrap();
        let loaded = m
            .load_v2(
                "episodes.jsonl",
                &data[..],
                &m.versions,
                67108864,
                LoadMode::CompletedOnly,
            )
            .unwrap();
        let replay_id = loaded.episodes()[0]
            .header
            .restricted_replay
            .as_ref()
            .unwrap();
        let mut registry = Registry::default();
        registry.register(replay_id, &mut owned[0]).unwrap();
        assert_eq!(
            serde_json::to_value(&loaded.episodes()[0]).unwrap(),
            serde_json::to_value(
                mtg_recorder::from_core_v2(owned[0].trajectory().unwrap()).unwrap()
            )
            .unwrap()
        );
        assert!(
            registry
                .resolve(replay_id, &owned[0], |_, _| false)
                .is_err()
        );
        assert!(
            publication::read_replay(
                &root.0.join("missing"),
                replay_id,
                &owned[0],
                |_, _| false,
                67108864
            )
            .is_err()
        );
        let bytes = publication::read_replay(
            &root.0.join("replay"),
            replay_id,
            &owned[0],
            |id, key| id == replay_id && key.run == run,
            67108864,
        )
        .unwrap();
        let game = mtg_core::opening::replay::played::verify(&bytes).unwrap();
        assert_eq!(
            normalized(&game.snapshot()),
            normalized(owned[0].privileged_snapshot())
        );
        if script {
            let t = owned[0].trajectory().unwrap();
            assert_eq!(t.decisions().len(), 101);
            assert_eq!(t.footer().unwrap().returns, [1, -1]);
            let view = &owned[0].final_observations().unwrap()[0].view;
            assert_eq!(view.life, [20, 15]);
            assert_eq!(view.hand_counts, [5, 7]);
            assert_eq!(view.library_counts, [31, 31]);
            for (actual, expected) in owned[0]
                .privileged_history()
                .iter()
                .zip(&c.script.as_ref().unwrap().records)
            {
                assert_eq!(
                    serde_json::from_slice::<Value>(actual).unwrap(),
                    serde_json::from_str::<Value>(&expected.record).unwrap()
                );
            }
        }
        c.capture = None;
        let mut off = vec![];
        crate::native::run_observed(&c, &mut vec![], || None, &mut |_, _| Ok(()), &mut |r| {
            off.push(r.clone())
        })
        .unwrap();
        assert_eq!(owned[0].status(), off[0].status());
        assert_eq!(owned[0].privileged_history(), off[0].privileged_history());
        assert_eq!(
            normalized(owned[0].privileged_snapshot()),
            normalized(off[0].privileged_snapshot())
        );
        // Independently declared corruption, followed by absence; never bless disk bytes.
        let replay_path = root.0.join("replay").join(run).join(replay_id);
        fs::write(&replay_path, b"corrupt").unwrap();
        assert!(
            publication::read_replay(
                &root.0.join("replay"),
                replay_id,
                &owned[0],
                |_, _| true,
                67108864
            )
            .is_err()
        );
        fs::remove_file(&replay_path).unwrap();
        assert!(
            publication::read_replay(
                &root.0.join("replay"),
                replay_id,
                &owned[0],
                |_, _| true,
                67108864
            )
            .is_err()
        );
        let mut corrupt = data;
        corrupt[0] = b'!';
        assert!(
            m.load_v2(
                "episodes.jsonl",
                &corrupt[..],
                &m.versions,
                67108864,
                LoadMode::CompletedOnly
            )
            .is_err()
        );
    }
}

#[test]
fn publication_fault_boundaries_keep_game_outcome_and_uncertainty_separate() {
    for stage in [
        FileStage::Create,
        FileStage::Write,
        FileStage::Sync,
        FileStage::Link,
        FileStage::UnlinkPartial,
        FileStage::DirectorySync,
        FileStage::Withdraw,
    ] {
        let root = Root::new();
        let mut c = root.config(true);
        short(&mut c);
        let mut out = vec![];
        let mut hit = false;
        let code = crate::native::run_observed(
            &c,
            &mut out,
            || None,
            &mut |path, at| {
                let fault = if stage == FileStage::Withdraw {
                    path.ends_with("manifest.json")
                        && [FileStage::DirectorySync, FileStage::Withdraw].contains(&at)
                } else {
                    stage == at
                };
                if fault {
                    hit = true;
                    Err(io::Error::other("synthetic PRIVATE path fault"))
                } else {
                    Ok(())
                }
            },
            &mut |_| {},
        )
        .unwrap();
        assert!(hit);
        assert_eq!(code, 3);
        let r = rows(&out);
        assert_eq!(r[1]["status"], "completed");
        assert_eq!(r[1]["winner"], 1);
        assert_eq!(r.last().unwrap()["completed"], 1);
        assert_eq!(r.last().unwrap()["failed"], 0);
        let p = &r[2];
        assert_eq!(
            p["status"],
            if stage == FileStage::Withdraw {
                "uncertain"
            } else {
                "failed"
            }
        );
        assert!(!String::from_utf8_lossy(&out).contains("PRIVATE"));
        let manifest = root
            .0
            .join("data")
            .join(p["run_id"].as_str().unwrap())
            .join("manifest.json");
        assert_eq!(manifest.exists(), stage == FileStage::Withdraw);
    }
}

#[test]
fn captured_limits_and_game_stops_preserve_diagnostics_and_not_started() {
    for mode in 0..6 {
        let root = Root::new();
        let mut c = root.config(true);
        c.episodes = 2;
        c.native.as_mut().unwrap().work_quantum = std::num::NonZeroUsize::new(1000).unwrap();
        match mode {
            0 => c.max_decisions = 1,
            1 => c.native.as_mut().unwrap().max_records = std::num::NonZeroUsize::MIN,
            2 => c.native.as_mut().unwrap().max_work_calls = 2,
            _ => (),
        }
        let stop = match mode {
            3 => Some(Stop::Deadline),
            4 => Some(Stop::Sigint),
            5 => Some(Stop::Sigterm),
            _ => None,
        };
        let mut polls = 0;
        let mut out = vec![];
        crate::native::run(&c, &mut out, || {
            polls += 1;
            if polls == 3 { stop } else { None }
        })
        .unwrap();
        let r = rows(&out);
        assert_eq!(r.last().unwrap()["started"], 1);
        assert_eq!(r.last().unwrap()["not_started"], 1);
        assert_eq!(r.last().unwrap()["completed"], 0);
        let p = r.iter().find(|r| r["type"] == "publication").unwrap();
        assert_eq!(p["status"], "published");
        let dir = root.0.join("data").join(p["run_id"].as_str().unwrap());
        let m =
            Manifest::parse(&fs::read(dir.join("manifest.json")).unwrap()[..], 67108864).unwrap();
        let bytes = fs::read(dir.join("episodes.jsonl")).unwrap();
        assert!(
            m.load_v2(
                "episodes.jsonl",
                &bytes[..],
                &m.versions,
                67108864,
                LoadMode::CompletedOnly
            )
            .is_err()
        );
        m.load_v2(
            "episodes.jsonl",
            &bytes[..],
            &m.versions,
            67108864,
            LoadMode::Diagnostic,
        )
        .unwrap();
        assert_eq!(m.episodes.len(), 1);
    }
}

#[test]
fn publication_deadline_and_interrupt_before_during_and_after_commit() {
    for stop in [Stop::Deadline, Stop::Sigint, Stop::Sigterm] {
        for boundary in [
            FileStage::Create,
            FileStage::Write,
            FileStage::Link,
            FileStage::DirectorySync,
        ] {
            let root = Root::new();
            let mut c = root.config(true);
            short(&mut c);
            let stopped = std::cell::Cell::new(false);
            let mut out = vec![];
            let code = crate::native::run_observed(
                &c,
                &mut out,
                || stopped.get().then_some(stop),
                &mut |path, stage| {
                    if path.ends_with("manifest.json") && stage == boundary {
                        stopped.set(true);
                    }
                    Ok(())
                },
                &mut |_| {},
            )
            .unwrap();
            let r = rows(&out);
            // The last successful directory sync commits; later control must not
            // falsely withdraw or misreport a durable artifact.
            if boundary == FileStage::DirectorySync {
                assert_eq!(r[2]["status"], "published");
                assert_eq!(code, 0);
            } else {
                assert_eq!(r[2]["status"], "failed");
                assert_eq!(code, stop.code());
            }
            assert_eq!(r.last().unwrap()["completed"], 1);
            assert_eq!(r.last().unwrap()["incomplete"], 0);
        }
        let root = Root::new();
        let c = root.config(true);
        let mut out = vec![];
        assert_eq!(
            crate::native::run(&c, &mut out, || Some(stop)).unwrap(),
            stop.code()
        );
        let r = rows(&out);
        assert_eq!(r.last().unwrap()["not_started"], 1);
        assert_eq!(r.last().unwrap()["started"], 0);
        assert_eq!(r[1]["status"], "failed");
    }
}

#[test]
fn publication_flush_failure_reaches_cli_accounting() {
    let root = Root::new();
    let mut c = root.config(true);
    short(&mut c);
    let mut hit = false;
    let mut out = vec![];
    let code = crate::native::run_observed(
        &c,
        &mut out,
        || None,
        &mut |_, stage| {
            if format!("{stage:?}") == "Flush" {
                hit = true;
                Err(io::Error::other("synthetic flush failure"))
            } else {
                Ok(())
            }
        },
        &mut |_| {},
    )
    .unwrap();
    assert!(
        hit,
        "publisher must expose its flush boundary to cancellation/fault observation"
    );
    assert_eq!(code, 3);
    let r = rows(&out);
    assert_eq!(r[2]["status"], "failed");
    assert_eq!(r.last().unwrap()["completed"], 1);
}

#[test]
fn queue_overflow_and_existing_file_conflicts_do_not_clobber_or_claim_publication() {
    for queue in [true, false] {
        let root = Root::new();
        let mut c = root.config(true);
        short(&mut c);
        if queue {
            c.capture.as_mut().unwrap().queue_bytes = 1;
        }
        let mut out = vec![];
        let mut sentinel = None;
        let code = crate::native::run_observed(
            &c,
            &mut out,
            || None,
            &mut |path, stage| {
                if !queue && path.ends_with("episodes.jsonl") && stage == FileStage::Link {
                    fs::write(path, b"sentinel").unwrap();
                    sentinel = Some(path.to_path_buf());
                }
                Ok(())
            },
            &mut |_| {},
        )
        .unwrap();
        assert_eq!(code, 3);
        let r = rows(&out);
        assert_eq!(r[2]["status"], "failed");
        assert_eq!(r.last().unwrap()["completed"], 1);
        if let Some(path) = sentinel {
            assert_eq!(fs::read(path).unwrap(), b"sentinel");
        }
        let dir = root.0.join("data").join(r[2]["run_id"].as_str().unwrap());
        assert!(!dir.join("manifest.json").exists());
    }
}

#[test]
fn consecutive_captured_episodes_keep_distinct_opaque_ids_and_exact_counts() {
    let root = Root::new();
    let mut c = root.config(true);
    short(&mut c);
    c.episodes = 2;
    let script = c.script.as_mut().unwrap();
    let mut second = serde_json::to_value(&script.records[0]).unwrap();
    second["episode"] = json!(1);
    script.records.push(serde_json::from_value(second).unwrap());
    let mut out = vec![];
    assert_eq!(crate::native::run(&c, &mut out, || None).unwrap(), 0);
    let r = rows(&out);
    let p = &r[3];
    assert_eq!(p["status"], "published");
    assert_eq!(p["started"], 2);
    assert_eq!(r[4]["not_started"], 0);
    assert_eq!(r[4]["completed"], 2);
    let dir = root.0.join("data").join(p["run_id"].as_str().unwrap());
    let m = Manifest::parse(&fs::read(dir.join("manifest.json")).unwrap()[..], 67108864).unwrap();
    let bytes = fs::read(dir.join("episodes.jsonl")).unwrap();
    let data = m
        .load_v2(
            "episodes.jsonl",
            &bytes[..],
            &m.versions,
            67108864,
            LoadMode::CompletedOnly,
        )
        .unwrap();
    assert_eq!(data.episodes().len(), 2);
    assert_eq!(data.episodes()[0].header.id.ordinal, 0);
    assert_eq!(data.episodes()[1].header.id.ordinal, 1);
    assert_ne!(
        data.episodes()[0].header.restricted_replay,
        data.episodes()[1].header.restricted_replay
    );
    assert_eq!(data.episodes()[0].decisions.len(), 0);
    assert_eq!(data.episodes()[1].decisions.len(), 0);
}
