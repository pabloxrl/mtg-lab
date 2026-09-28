//! Original GH-116 / RFC 0002 §8 contract tests. Expectations are handwritten:
//! one synthetic concession episode, one decision, reward [1,-1], ordinal 7.
//! Manifest whole-file checksum independently calculated with Python hashlib.
use mtg_recorder::{Backpressure, Error, Writer, manifest::*, schema::*};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
const FIXTURE: &[u8] = include_bytes!("manifest.json");
const DATA: &[u8] = include_bytes!("episode.jsonl");
fn manifest() -> Manifest {
    Manifest::parse(FIXTURE, 10000).unwrap()
}
fn load(m: &Manifest, data: &[u8], mode: LoadMode) -> Result<LoadedRun, Error> {
    m.load("episode.jsonl", data, &manifest().versions, 10000, mode)
}
fn altered(path: &[&str], value: Value) -> Manifest {
    let mut v: Value = serde_json::from_slice(FIXTURE).unwrap();
    let mut p = &mut v;
    for key in path {
        p = &mut p[*key];
    }
    *p = value;
    serde_json::from_value(v).unwrap()
}
#[test]
fn handwritten_pair_roundtrips() {
    let m = manifest();
    assert_eq!(
        Manifest::parse(m.encode(10000).unwrap().as_slice(), 10000).unwrap(),
        m
    );
    let run = load(&m, DATA, LoadMode::default()).unwrap();
    let expected: Episode = serde_json::from_slice(include_bytes!("episode.json")).unwrap();
    assert_eq!(run.episodes(), &[expected]);
    assert_eq!(run.episodes()[0].footer.as_ref().unwrap().returns, [1, -1]);
    assert_eq!(m.file.bytes, 2318);
}
#[test]
fn every_required_field_and_unknown_fields_rejected() {
    let original: Value = serde_json::from_slice(FIXTURE).unwrap();
    for key in original.as_object().unwrap().keys() {
        let mut v = original.clone();
        v.as_object_mut().unwrap().remove(key);
        assert!(
            Manifest::parse(serde_json::to_vec(&v).unwrap().as_slice(), 10000).is_err(),
            "{key}"
        );
    }
    let mut v = original;
    v["private_seed"] = json!(123);
    assert!(Manifest::parse(serde_json::to_vec(&v).unwrap().as_slice(), 10000).is_err());
}
#[test]
fn incompatible_versions_and_provenance_rejected() {
    let cases = vec![
        (vec!["dataset_schema"], json!(2)),
        (vec!["versions", "schema"], json!(2)),
        (vec!["versions", "observation"], json!(2)),
        (vec!["versions", "engine"], json!("other")),
        (vec!["versions", "rules"], json!("other")),
        (vec!["versions", "cards"], json!("other")),
        (vec!["versions", "action"], json!("other")),
        (vec!["run"], json!("00000000-0000-0000-0000-000000000000")),
        (vec!["config_hash"], json!("d".repeat(64))),
        (vec!["deck_hashes"], json!(["d".repeat(64), "e".repeat(64)])),
        (vec!["policies"], json!(["other", "script-b"])),
        (vec!["seats"], json!([1, 0])),
        (vec!["starting_seat"], json!(1)),
        (vec!["limits", "decisions"], json!(42)),
    ];
    for (path, value) in cases {
        let m = altered(&path, value);
        assert!(load(&m, DATA, LoadMode::Diagnostic).is_err(), "{path:?}");
    }
}
#[test]
fn inventory_checksum_and_completion_are_binding() {
    let cases = vec![
        (vec!["file", "format"], json!(2)),
        (vec!["file", "name"], json!("different.jsonl")),
        (vec!["file", "bytes"], json!(2317)),
        (vec!["file", "sha256"], json!("0".repeat(64))),
        (vec!["file", "episodes"], json!(2)),
        (vec!["file", "decisions"], json!(2)),
        (vec!["recording_complete"], json!(false)),
        (vec!["episodes"], json!([])),
        (
            vec!["episodes"],
            json!([{"ordinal":8,"status":"Completed"}]),
        ),
        (
            vec!["episodes"],
            json!([{"ordinal":7,"status":"Truncated"}]),
        ),
        (
            vec!["capture"],
            json!({"AllEpisodes":{"first_ordinal":7,"count":2}}),
        ),
        (vec!["end"], json!({"Truncated":"budget"})),
    ];
    for (path, value) in cases {
        assert!(
            load(&altered(&path, value), DATA, LoadMode::Diagnostic).is_err(),
            "{path:?}"
        );
    }
    assert!(load(&manifest(), &DATA[..DATA.len() - 1], LoadMode::Diagnostic).is_err());
}
#[test]
fn failed_and_interrupted_runs_are_explicit_diagnostics() {
    for status in [
        EpisodeStatus::Failed("writer error".into()),
        EpisodeStatus::Incomplete,
    ] {
        let mut m = manifest();
        m.episodes.push(DeclaredEpisode {
            ordinal: 8,
            status: status.clone(),
        });
        m.capture = Capture::AllEpisodes {
            first_ordinal: 7,
            count: 2,
        };
        m.recording_complete = false;
        m.end = if matches!(status, EpisodeStatus::Failed(_)) {
            RunEnd::Failed("writer error".into())
        } else {
            RunEnd::Truncated("interrupted".into())
        };
        assert!(matches!(
            load(&m, DATA, LoadMode::default()),
            Err(Error::Incomplete)
        ));
        assert_eq!(
            load(&m, DATA, LoadMode::Diagnostic)
                .unwrap()
                .episodes()
                .len(),
            1
        );
        let mut lie = m.clone();
        lie.end = RunEnd::Completed;
        assert!(load(&lie, DATA, LoadMode::Diagnostic).is_err());
        lie = m;
        lie.recording_complete = true;
        assert!(load(&lie, DATA, LoadMode::Diagnostic).is_err());
    }
}
fn persisted(e: &Episode, m: &mut Manifest) -> Vec<u8> {
    let mut w = Writer::new(Vec::new(), 10000, Backpressure::Block).unwrap();
    w.append(e).unwrap();
    let b = w.finish().unwrap();
    m.file.bytes = b.len() as u64;
    m.file.sha256 = format!("{:x}", Sha256::digest(&b));
    b
}
#[test]
fn truncated_episode_is_not_a_completed_run() {
    let mut e: Episode = serde_json::from_slice(include_bytes!("episode.json")).unwrap();
    let mut m = manifest();
    e.decisions[0].terminated = false;
    e.decisions[0].truncated = true;
    e.decisions[0].reward = [0, 0];
    let f = e.footer.as_mut().unwrap();
    f.end = End::Truncated(Limit::Decisions);
    f.returns = [0, 0];
    f.boundary_reward = [0, 0];
    for v in &mut f.final_observations {
        v.terminal = None;
    }
    m.episodes[0].status = EpisodeStatus::Truncated;
    m.end = RunEnd::Truncated("decision limit".into());
    let b = persisted(&e, &mut m);
    assert!(matches!(
        load(&m, &b, LoadMode::default()),
        Err(Error::Incomplete)
    ));
    assert_eq!(load(&m, &b, LoadMode::Diagnostic).unwrap().episodes(), &[e]);
    m.end = RunEnd::Completed;
    assert!(load(&m, &b, LoadMode::Diagnostic).is_err());
}
#[test]
fn replay_ids_never_enter_policy_rows_and_paths_are_rejected() {
    let mut e: Episode = serde_json::from_slice(include_bytes!("episode.json")).unwrap();
    let mut m = manifest();
    e.header.restricted_replay = Some("01234567-89ab-cdef-0123-456789abcdef".into());
    let b = persisted(&e, &mut m);
    let run = load(&m, &b, LoadMode::default()).unwrap();
    let rows = run.policy_decisions(0).unwrap().collect::<Vec<_>>();
    let encoded = serde_json::to_string(&rows).unwrap();
    assert!(!encoded.contains("01234567-89ab-cdef"));
    assert!(!encoded.contains("restricted_replay"));
    assert_eq!(run.policy_decisions(1).unwrap().count(), 0);
    assert!(run.policy_decisions(2).is_err());
    for reference in [
        "/tmp/private.replay",
        "../private",
        "https://example.invalid/private",
        "{\"seed\":12}",
    ] {
        e.header.restricted_replay = Some(reference.into());
        let b = persisted(&e, &mut m);
        assert!(load(&m, &b, LoadMode::Diagnostic).is_err(), "{reference}");
    }
}
#[test]
fn bounds_and_unsupported_conventions_rejected() {
    assert!(matches!(Manifest::parse(FIXTURE, 10), Err(Error::Limit)));
    assert!(matches!(manifest().encode(10), Err(Error::Limit)));
    assert!(matches!(
        manifest().load(
            "episode.jsonl",
            DATA,
            &manifest().versions,
            10,
            LoadMode::default()
        ),
        Err(Error::Limit)
    ));
    for field in ["reward", "discount", "time", "capture"] {
        let mut v: Value = serde_json::from_slice(FIXTURE).unwrap();
        v[field] = json!("unknown");
        assert!(Manifest::parse(serde_json::to_vec(&v).unwrap().as_slice(), 10000).is_err());
    }
}

#[test]
fn nested_required_fields_duplicates_and_invalid_declarations_rejected() {
    let original: Value = serde_json::from_slice(FIXTURE).unwrap();
    for section in ["versions", "limits", "file"] {
        for key in original[section].as_object().unwrap().keys() {
            let mut v = original.clone();
            v[section].as_object_mut().unwrap().remove(key);
            assert!(
                Manifest::parse(serde_json::to_vec(&v).unwrap().as_slice(), 10000).is_err(),
                "{section}.{key}"
            );
        }
    }
    let text = String::from_utf8(FIXTURE.to_vec()).unwrap().replace(
        "\"dataset_schema\": 1,",
        "\"dataset_schema\": 1, \"dataset_schema\": 1,",
    );
    assert!(Manifest::parse(text.as_bytes(), 10000).is_err());
    for (path, value) in [
        (vec!["run"], json!("not-a-uuid")),
        (vec!["policies"], json!(["", "script-b"])),
        (vec!["config_hash"], json!("bad")),
        (vec!["file", "name"], json!("../episode.jsonl")),
        (vec!["file", "name"], json!("episode.jsonl.partial")),
        (vec!["limits", "decisions"], json!(0)),
        (
            vec!["episodes"],
            json!([{"ordinal":7,"status":"Completed"},{"ordinal":7,"status":"Completed"}]),
        ),
        (
            vec!["episodes"],
            json!([{"ordinal":7,"status":{"Failed":""}}]),
        ),
        (
            vec!["capture"],
            json!({"AllEpisodes":{"first_ordinal":18446744073709551615u64,"count":1}}),
        ),
        (
            vec!["capture"],
            json!({"DeterministicSeedSubset":{"algorithm":"", "config_hash":"a".repeat(64),"population":10}}),
        ),
    ] {
        assert!(altered(&path, value).validate().is_err(), "{path:?}");
    }
}

#[test]
fn declared_seed_subset_is_preserved_but_not_selected_by_loader() {
    let mut m = manifest();
    m.capture = Capture::DeterministicSeedSubset {
        algorithm: "collector-selection-v1".into(),
        config_hash: "d".repeat(64),
        population: 10,
    };
    assert_eq!(
        Manifest::parse(m.encode(10000).unwrap().as_slice(), 10000).unwrap(),
        m
    );
    assert_eq!(
        load(&m, DATA, LoadMode::default())
            .unwrap()
            .episodes()
            .len(),
        1
    );
    m.capture = Capture::DeterministicSeedSubset {
        algorithm: "collector-selection-v1".into(),
        config_hash: "d".repeat(64),
        population: 7,
    };
    assert!(load(&m, DATA, LoadMode::Diagnostic).is_err());
}

#[test]
fn diagnostic_mode_never_salvages_corrupt_or_unsealed_prefixes() {
    let mut m = manifest();
    let line_end = DATA.iter().position(|b| *b == b'\n').unwrap() + 1;
    let prefix = &DATA[..line_end];
    m.file.bytes = prefix.len() as u64;
    m.file.sha256 = format!("{:x}", Sha256::digest(prefix));
    assert!(matches!(
        load(&m, prefix, LoadMode::Diagnostic),
        Err(Error::Incomplete)
    ));
    let mut corrupt = DATA.to_vec();
    let at = corrupt.windows(7).position(|s| s == b"engine-").unwrap();
    corrupt[at] = b'E';
    m.file.bytes = corrupt.len() as u64;
    m.file.sha256 = format!("{:x}", Sha256::digest(&corrupt));
    assert!(matches!(
        load(&m, &corrupt, LoadMode::Diagnostic),
        Err(Error::Integrity)
    ));
}

#[test]
fn empty_sealed_run_and_io_failure_remain_explicit() {
    // Independent SHA-256 of zero episode bytes in the canonical empty-file seal.
    let bytes=b"{\"decisions\":0,\"episodes\":0,\"format\":1,\"kind\":\"seal\",\"sha256\":\"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"}\n";
    let mut m = manifest();
    m.episodes.clear();
    m.capture = Capture::AllEpisodes {
        first_ordinal: 0,
        count: 0,
    };
    m.file.episodes = 0;
    m.file.decisions = 0;
    m.file.bytes = bytes.len() as u64;
    m.file.sha256 = format!("{:x}", Sha256::digest(bytes));
    assert!(
        load(&m, bytes, LoadMode::default())
            .unwrap()
            .episodes()
            .is_empty()
    );
    struct Broken;
    impl std::io::Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("read failed"))
        }
    }
    assert!(matches!(
        m.load(
            "episode.jsonl",
            Broken,
            &m.versions,
            10000,
            LoadMode::default()
        ),
        Err(Error::Io(_))
    ));
}
