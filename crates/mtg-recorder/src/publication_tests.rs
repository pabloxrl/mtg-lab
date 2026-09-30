// GH-164: actual normal resets, seed 164. CR 103.5 keeps and CR 104.3a
// concession: two Keep decisions, then P1 concedes outside policy decisions.
// Decision budget 2 instead truncates after both keeps, with zero reward.
use crate as mtg_recorder;
use mtg_core::{
    episode::{Budget, Clock, Driver, EpisodeResult, Status},
    game::{
        Config,
        policy::{Choice, Submission},
    },
    objects::Seat,
    trajectory::{Limits, PolicyInfo},
};
use mtg_recorder::{
    Backpressure,
    collector::{Run, Storage},
    manifest::{LoadMode, Manifest},
};

use std::num::NonZeroUsize;
const RUN: &str = "a34c952c-723c-44ef-95f9-dcdb066db576";
#[derive(Debug)]
struct Zero;
impl Clock for Zero {
    fn now_ms(&self) -> u64 {
        0
    }
}
fn run(n: u64, truncated: bool) -> Run {
    Run {
        id: RUN.into(),
        config: Config::default(),
        policies: ["script-v1".into(), "script-v2".into()],
        limits: Limits {
            decisions: Some(if truncated { 2 } else { 100 }),
            ..Limits::default()
        },
        first_ordinal: 7,
        started: n,
    }
}
fn storage() -> Storage {
    Storage {
        queue_bytes: 1_000_000,
        max_bytes: 4_000_000,
        backpressure: Backpressure::Block,
    }
}
fn result(run: &Run, ordinal: u64, kind: &str) -> EpisodeResult {
    result_bound(run, ordinal, kind, 164, None)
}
fn result_bound(
    run: &Run,
    ordinal: u64,
    kind: &str,
    master: u64,
    replay: Option<&str>,
) -> EpisodeResult {
    let records = if kind == "failed" { 1 } else { 100 };
    let mut d = Driver::bounded(
        256,
        Budget {
            limits: run.limits.clone(),
            work_quantum: NonZeroUsize::MAX,
            records: NonZeroUsize::new(records).unwrap(),
        },
        Box::new(Zero),
    )
    .unwrap();
    let mut h = run.header(ordinal).unwrap();
    h.restricted_replay = replay.map(str::to_owned);
    d.reset_captured(&run.config, master, ordinal, NonZeroUsize::MAX, &h)
        .unwrap();
    if kind != "incomplete" {
        for seat in [Seat::P0, Seat::P1] {
            let domain = d.observe(seat).unwrap().decision.unwrap();
            let accepted = d.submit_with_policy(
                seat,
                &Submission {
                    schema_version: 1,
                    revision: domain.revision,
                    generation: domain.generation,
                    choices: vec![Choice::Keep],
                },
                &PolicyInfo {
                    checkpoint: Some("caller-checkpoint".into()),
                    ..PolicyInfo::default()
                },
            );
            if kind == "failed" && seat == Seat::P1 {
                assert!(accepted.is_err());
            } else {
                accepted.unwrap();
            }
        }
        if kind == "completed" {
            d.advance(NonZeroUsize::MAX).unwrap();
            d.concede(Seat::P1, d.episode_id().unwrap()).unwrap();
        }
    }
    d.finish().unwrap()
}

use mtg_core::episode::replay::Registry;
use mtg_recorder::publication::{Destination, publish};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "mtg-publication-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::create_dir(path.join("datasets")).unwrap();
        fs::create_dir(path.join("replays")).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
const REPLAY: &str = "45b374c9-a4db-44fe-a681-603c4055708a";
#[test]
fn real_normal_reset_publication_reopens_manifest_jsonl_and_authorized_replay() {
    let temp = Temp::new();
    let run = run(1, false);
    let mut result = result(&run, 7, "completed");
    let mut registry = Registry::default();
    registry.register(REPLAY, &mut result).unwrap();
    publish(
        &run,
        std::slice::from_ref(&result),
        &registry,
        |id, key| id == REPLAY && key.ordinal == 7,
        Destination {
            datasets: &temp.0.join("datasets"),
            replays: &temp.0.join("replays"),
        },
        storage(),
    )
    .expect("validated artifacts must publish");
    let dir = temp.0.join("datasets").join(RUN);
    let manifest = Manifest::parse(
        fs::File::open(dir.join("manifest.json")).unwrap(),
        4_000_000,
    )
    .unwrap();
    let loaded = manifest
        .load_v2(
            "episodes.jsonl",
            fs::File::open(dir.join("episodes.jsonl")).unwrap(),
            &manifest.versions,
            4_000_000,
            LoadMode::CompletedOnly,
        )
        .unwrap();
    let e = &loaded.episodes()[0];
    assert_eq!(
        e.decisions
            .iter()
            .map(|d| (d.index, d.actor))
            .collect::<Vec<_>>(),
        vec![(0, 0), (1, 1)]
    );
    for d in &e.decisions {
        assert_eq!(
            d.choice.submission.choices,
            vec![mtg_recorder::structured::Command::Keep]
        );
        assert_eq!(d.reward, [0, 0]);
        assert_eq!(d.observation.view.life, [20, 20]);
        assert_eq!(d.observation.view.hand_counts, [7, 7]);
        assert_eq!(d.observation.view.library_counts, [33, 33]);
        assert_eq!(d.observation.view.hand.len(), 7);
        assert_eq!(d.observation.view.seat, d.actor);
        let domain = d.observation.decision.as_ref().unwrap();
        assert_eq!(domain.actor, d.actor);
        assert_eq!(
            domain.candidates,
            vec![
                mtg_recorder::structured::Command::Keep,
                mtg_recorder::structured::Command::Mulligan
            ]
        );
        assert_eq!(domain.legal_mask, vec![true, true]);
        assert_eq!(d.choice.policy.log_probability, None);
    }
    assert_eq!(e.footer.as_ref().unwrap().returns, [1, -1]);
    assert_eq!(e.header.restricted_replay.as_deref(), Some(REPLAY));
    assert_eq!(
        e,
        &mtg_recorder::from_core_v2(result.trajectory().unwrap()).unwrap()
    );
    for seat in [0, 1] {
        let rows = loaded.policy_decisions(seat).unwrap().collect::<Vec<_>>();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].actor, seat);
        let text = serde_json::to_string(&rows).unwrap();
        for forbidden in ["restricted_replay", "master", "rng", "history", REPLAY] {
            assert!(!text.contains(forbidden));
        }
    }
    let bytes = super::read_replay(
        &temp.0.join("replays"),
        REPLAY,
        &result,
        |id, key| id == REPLAY && key.ordinal == 7,
        4_000_000,
    )
    .unwrap();
    let game = mtg_core::opening::replay::played::verify(&bytes).unwrap();
    assert_eq!(game.outcome().map(Status::Completed), Some(result.status()));
    assert_eq!(
        normalized(&game.snapshot()),
        normalized(result.privileged_snapshot())
    );
    assert_eq!(
        bytes,
        registry.resolve(REPLAY, &result, |_, _| true).unwrap()
    );
}

#[test]
fn denied_missing_or_wrong_replay_binding_never_reserves_a_run() {
    let t = Temp::new();
    let r = run(1, false);
    let mut a = result(&r, 7, "completed");
    let mut registry = Registry::default();
    registry.register(REPLAY, &mut a).unwrap();
    for (records, grant) in [
        (vec![a.clone()], false),
        (vec![result(&r, 7, "completed")], true),
        (vec![result(&r, 8, "completed")], true),
    ] {
        assert!(
            publish(
                &r,
                &records,
                &registry,
                |_, _| grant,
                Destination {
                    datasets: &t.0.join("datasets"),
                    replays: &t.0.join("replays")
                },
                storage()
            )
            .is_err()
        );
        assert!(!t.0.join("datasets").join(RUN).exists());
    }
    let mut wrong = r.clone();
    wrong.config.starting_seat = 1;
    assert!(
        publish(
            &wrong,
            &[a],
            &registry,
            |_, _| true,
            Destination {
                datasets: &t.0.join("datasets"),
                replays: &t.0.join("replays")
            },
            storage()
        )
        .is_err()
    );
}
#[test]
fn diagnostics_preserve_real_truncation_failure_and_incomplete_accounting() {
    for kind in ["truncated", "failed", "incomplete"] {
        let t = Temp::new();
        let r = run(1, kind == "truncated");
        let a = result(&r, 7, kind);
        assert!(!matches!(a.status(), Status::Completed(_)));
        publish(
            &r,
            &[a],
            &Registry::default(),
            |_, _| false,
            Destination {
                datasets: &t.0.join("datasets"),
                replays: &t.0.join("replays"),
            },
            storage(),
        )
        .unwrap();
        let dir = t.0.join("datasets").join(RUN);
        let m = Manifest::parse(
            fs::File::open(dir.join("manifest.json")).unwrap(),
            4_000_000,
        )
        .unwrap();
        let bytes = fs::read(dir.join("episodes.jsonl")).unwrap();
        assert!(
            m.load_v2(
                "episodes.jsonl",
                &bytes[..],
                &m.versions,
                4_000_000,
                LoadMode::CompletedOnly
            )
            .is_err()
        );
        let loaded = m
            .load_v2(
                "episodes.jsonl",
                &bytes[..],
                &m.versions,
                4_000_000,
                LoadMode::Diagnostic,
            )
            .unwrap();
        assert_eq!(loaded.episodes().len(), usize::from(kind == "truncated"));
        assert!(
            fs::read_dir(t.0.join("replays").join(RUN))
                .unwrap()
                .next()
                .is_none()
        );
        if kind == "truncated" {
            assert_eq!(
                loaded.episodes()[0].footer.as_ref().unwrap().returns,
                [0, 0]
            );
        }
    }
}

fn completed() -> (Run, EpisodeResult, Registry) {
    let r = run(1, false);
    let mut a = result(&r, 7, "completed");
    let mut registry = Registry::default();
    registry.register(REPLAY, &mut a).unwrap();
    (r, a, registry)
}
fn publish_at(
    t: &Temp,
    r: &Run,
    a: &EpisodeResult,
    registry: &Registry,
    hook: &mut crate::FileHook<'_>,
) -> Result<(), crate::Error> {
    super::publish_observed(
        r,
        std::slice::from_ref(a),
        registry,
        |_, _| true,
        Destination {
            datasets: &t.0.join("datasets"),
            replays: &t.0.join("replays"),
        },
        storage(),
        hook,
    )
}
#[test]
fn all_write_sync_link_and_manifest_failures_preserve_diagnostics_without_advertisement() {
    use crate::FileStage::*;
    let (r, a, registry) = completed();
    for name in ["episodes.jsonl", REPLAY, "manifest.json"] {
        for stage in [Create, Write, Sync, Link, DirectorySync] {
            let t = Temp::new();
            let error = publish_at(&t, &r, &a, &registry, &mut |path, boundary| {
                if path.file_name().unwrap() == name && boundary == stage {
                    Err(std::io::Error::other("injected file operation failure"))
                } else {
                    Ok(())
                }
            });
            assert!(error.is_err(), "{name} {stage:?}");
            assert!(
                !t.0.join("datasets")
                    .join(RUN)
                    .join("manifest.json")
                    .exists()
            );
            assert!(t.0.join("datasets").join(RUN).is_dir());
            assert!(
                publish_at(&t, &r, &a, &registry, &mut |_, _| Ok(())).is_err(),
                "reserved run cannot retry"
            );
            if name == "manifest.json" {
                assert!(
                    t.0.join("datasets")
                        .join(RUN)
                        .join("episodes.jsonl")
                        .is_file()
                );
                assert!(t.0.join("replays").join(RUN).join(REPLAY).is_file());
            }
        }
    }
}
#[test]
fn interruptions_at_every_file_boundary_never_advertise_partial_artifacts() {
    use crate::FileStage::*;
    let (r, a, registry) = completed();
    for name in ["episodes.jsonl", REPLAY, "manifest.json"] {
        for stage in [Create, Write, Sync, Link, DirectorySync] {
            let t = Temp::new();
            let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                publish_at(&t, &r, &a, &registry, &mut |path, boundary| {
                    if path.file_name().unwrap() == name && boundary == stage {
                        panic!("simulated process interruption");
                    }
                    Ok(())
                })
                .unwrap();
            }));
            assert!(stopped.is_err());
            let dir = t.0.join("datasets").join(RUN);
            if dir.join("manifest.json").exists() {
                assert_eq!((name, stage), ("manifest.json", DirectorySync));
                let m = Manifest::parse(
                    fs::File::open(dir.join("manifest.json")).unwrap(),
                    4_000_000,
                )
                .unwrap();
                m.load_v2(
                    "episodes.jsonl",
                    fs::File::open(dir.join("episodes.jsonl")).unwrap(),
                    &m.versions,
                    4_000_000,
                    LoadMode::CompletedOnly,
                )
                .unwrap();
                super::read_replay(&t.0.join("replays"), REPLAY, &a, |_, _| true, 4_000_000)
                    .unwrap();
            }
            assert!(publish_at(&t, &r, &a, &registry, &mut |_, _| Ok(())).is_err());
        }
    }
}
#[test]
fn corruption_or_missing_artifacts_before_manifest_are_detected_and_readers_reject_later_damage() {
    let (r, a, registry) = completed();
    for name in ["episodes.jsonl", REPLAY] {
        for missing in [false, true] {
            let t = Temp::new();
            assert!(
                publish_at(&t, &r, &a, &registry, &mut |path, stage| {
                    if path.file_name().unwrap() == name && stage == crate::FileStage::DirectorySync
                    {
                        if missing {
                            fs::remove_file(path)?;
                        } else {
                            fs::write(path, b"corrupt")?;
                        }
                    }
                    Ok(())
                })
                .is_err()
            );
            assert!(
                !t.0.join("datasets")
                    .join(RUN)
                    .join("manifest.json")
                    .exists()
            );
        }
    }
    let t = Temp::new();
    publish_at(&t, &r, &a, &registry, &mut |_, _| Ok(())).unwrap();
    let private = t.0.join("replays");
    assert!(super::read_replay(&private, REPLAY, &a, |_, _| false, 4_000_000).is_err());
    let bytes = super::read_replay(
        &private,
        REPLAY,
        &a,
        |id, key| id == REPLAY && key.ordinal == 7,
        4_000_000,
    )
    .unwrap();
    assert_eq!(bytes, registry.resolve(REPLAY, &a, |_, _| true).unwrap());
    assert!(super::read_replay(&private, REPLAY, &a, |_, _| true, 1).is_err());
    let different = result(&r, 8, "completed");
    assert!(super::read_replay(&private, REPLAY, &different, |_, _| true, 4_000_000).is_err());
    fs::write(private.join(RUN).join(REPLAY), b"corrupt").unwrap();
    assert!(super::read_replay(&private, REPLAY, &a, |_, _| true, 4_000_000).is_err());
    fs::remove_file(private.join(RUN).join(REPLAY)).unwrap();
    assert!(super::read_replay(&private, REPLAY, &a, |_, _| true, 4_000_000).is_err());
    let dir = t.0.join("datasets").join(RUN);
    let m = Manifest::parse(
        fs::File::open(dir.join("manifest.json")).unwrap(),
        4_000_000,
    )
    .unwrap();
    fs::write(dir.join("episodes.jsonl"), b"corrupt").unwrap();
    assert!(
        m.load_v2(
            "episodes.jsonl",
            fs::File::open(dir.join("episodes.jsonl")).unwrap(),
            &m.versions,
            4_000_000,
            LoadMode::CompletedOnly
        )
        .is_err()
    );
}
#[test]
fn prior_runs_are_never_clobbered_and_roots_must_be_separate() {
    let (r, a, registry) = completed();
    let t = Temp::new();
    publish_at(&t, &r, &a, &registry, &mut |_, _| Ok(())).unwrap();
    let manifest = t.0.join("datasets").join(RUN).join("manifest.json");
    let before = fs::read(&manifest).unwrap();
    assert!(publish_at(&t, &r, &a, &registry, &mut |_, _| Ok(())).is_err());
    assert_eq!(fs::read(&manifest).unwrap(), before);
    for replay_root in [
        t.0.clone(),
        t.0.join("datasets"),
        t.0.join("datasets").join(RUN),
    ] {
        assert!(
            publish(
                &r,
                std::slice::from_ref(&a),
                &registry,
                |_, _| true,
                Destination {
                    datasets: &t.0.join("datasets"),
                    replays: &replay_root
                },
                storage()
            )
            .is_err()
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(t.0.join("replays").join(RUN))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(t.0.join("replays").join(RUN).join(REPLAY))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

// Normalize ONLY process-local store/decision namespaces, as the replay contract
// specifies. All game fields, identities, zones, pending work and RNG remain.
fn normalized(bytes: &[u8]) -> serde_json::Value {
    use serde_json::{Value, json};
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
            Value::Array(a) => a.iter_mut().for_each(scrub),
            _ => (),
        }
    }
    let envelope: Value = serde_json::from_slice(bytes).unwrap();
    let mut v: Value = serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
    v["objects"]["id"] = json!(0);
    scrub(&mut v);
    v
}
#[test]
fn failed_manifest_withdrawal_reports_uncertain_outcome_and_never_retries() {
    let (r, a, registry) = completed();
    let t = Temp::new();
    let err = publish_at(&t, &r, &a, &registry, &mut |path, stage| {
        if path.file_name().unwrap() == "manifest.json"
            && matches!(
                stage,
                crate::FileStage::DirectorySync | crate::FileStage::Withdraw
            )
        {
            Err(std::io::Error::other(
                "injected sync plus withdrawal failure",
            ))
        } else {
            Ok(())
        }
    });
    assert!(matches!(err, Err(crate::Error::PublicationUncertain)));
    // An uncertain commit may be visible, but it cannot reference partial data.
    let dir = t.0.join("datasets").join(RUN);
    let m = Manifest::parse(
        fs::File::open(dir.join("manifest.json")).unwrap(),
        4_000_000,
    )
    .unwrap();
    m.load_v2(
        "episodes.jsonl",
        fs::File::open(dir.join("episodes.jsonl")).unwrap(),
        &m.versions,
        4_000_000,
        LoadMode::CompletedOnly,
    )
    .unwrap();
    super::read_replay(&t.0.join("replays"), REPLAY, &a, |_, _| true, 4_000_000).unwrap();
    assert!(publish_at(&t, &r, &a, &registry, &mut |_, _| Ok(())).is_err());
}

#[test]
fn real_no_replace_link_errors_and_dataset_cleanup_errors_preserve_prior_bytes() {
    let (r, a, registry) = completed();
    for name in ["episodes.jsonl", REPLAY, "manifest.json"] {
        let t = Temp::new();
        let mut collided = None;
        let error = publish_at(&t, &r, &a, &registry, &mut |path, stage| {
            if path.file_name().unwrap() == name && stage == crate::FileStage::Link {
                fs::write(path, b"existing owner bytes")?;
                collided = Some(path.to_owned());
            }
            Ok(())
        });
        assert!(
            matches!(error, Err(crate::Error::Io(ref e)) if e.kind() == std::io::ErrorKind::AlreadyExists)
        );
        assert_eq!(
            fs::read(collided.unwrap()).unwrap(),
            b"existing owner bytes"
        );
        let manifest = t.0.join("datasets").join(RUN).join("manifest.json");
        if manifest.exists() {
            assert!(Manifest::parse(fs::File::open(manifest).unwrap(), 4_000_000).is_err());
        }
    }
    let t = Temp::new();
    assert!(
        publish_at(&t, &r, &a, &registry, &mut |path, stage| {
            if stage == crate::FileStage::UnlinkPartial
                && path.file_name().unwrap() == "episodes.jsonl"
            {
                Err(std::io::Error::other("injected dataset cleanup failure"))
            } else {
                Ok(())
            }
        })
        .is_err()
    );
    let dir = t.0.join("datasets").join(RUN);
    assert!(dir.join("episodes.jsonl.partial").exists());
    assert!(dir.join("episodes.jsonl").exists());
    assert!(!dir.join("manifest.json").exists());
}

#[test]
fn directory_reservation_and_parent_sync_failures_never_advertise() {
    let (r, a, registry) = completed();
    for root in ["datasets", "replays"] {
        for stage in [crate::FileStage::Create, crate::FileStage::DirectorySync] {
            let t = Temp::new();
            let reserved = t.0.join(root).join(RUN);
            assert!(
                publish_at(&t, &r, &a, &registry, &mut |path, boundary| {
                    if path == reserved && boundary == stage {
                        Err(std::io::Error::other("injected run reservation failure"))
                    } else {
                        Ok(())
                    }
                })
                .is_err()
            );
            assert!(
                !t.0.join("datasets")
                    .join(RUN)
                    .join("manifest.json")
                    .exists()
            );
            let result = publish_at(&t, &r, &a, &registry, &mut |_, _| Ok(()));
            if root == "datasets" && stage == crate::FileStage::Create {
                result.expect("pre-reservation error has not consumed run ID");
            } else {
                assert!(result.is_err(), "reserved run ID cannot retry");
            }
        }
    }
}

#[test]
fn same_id_with_different_actual_reset_history_is_not_the_registered_artifact() {
    let (r, a, registry) = completed();
    let b = result_bound(&r, 7, "completed", 165, Some(REPLAY));
    assert_ne!(a.privileged_snapshot(), b.privileged_snapshot());
    let t = Temp::new();
    assert!(publish_at(&t, &r, &b, &registry, &mut |_, _| Ok(())).is_err());
    assert!(!t.0.join("datasets").join(RUN).exists());
    publish_at(&t, &r, &a, &registry, &mut |_, _| Ok(())).unwrap();
    assert!(super::read_replay(&t.0.join("replays"), REPLAY, &b, |_, _| true, 4_000_000).is_err());
}
