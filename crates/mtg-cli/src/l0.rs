//! Offline, privileged full-corpus replay. No generation or reference processes.
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs,
    path::{Component, Path},
    time::Instant,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    version: String,
    status: String,
    final_admitted_games: usize,
    semantic_schema: String,
    game_count: usize,
    qualification_sha256: String,
    artifacts: BTreeMap<String, String>,
    games: Vec<Entry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    seed: u64,
    input: String,
    expected: String,
    trace: String,
    source_cursors: Vec<u64>,
    actions: usize,
    checkpoints: usize,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read(base: &Path, name: &str) -> Result<Vec<u8>, String> {
    let path = Path::new(name);
    if path
        .components()
        .any(|p| !matches!(p, Component::Normal(_)))
        || name.is_empty()
    {
        return Err(format!("/artifacts/{name}: invalid local path"));
    }
    let resolved = base
        .join(path)
        .canonicalize()
        .map_err(|e| format!("/artifacts/{name}: {e}"))?;
    if !resolved.starts_with(base) || !resolved.is_file() {
        return Err(format!("/artifacts/{name}: unavailable regular local file"));
    }
    fs::read(resolved).map_err(|e| format!("/artifacts/{name}: {e}"))
}
fn difference(expected: &Value, actual: &Value, path: &str) -> Option<String> {
    match (expected, actual) {
        (Value::Object(a), Value::Object(b)) => {
            for k in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                let p = format!("{path}/{k}");
                match (a.get(k), b.get(k)) {
                    (Some(x), Some(y)) => {
                        if let Some(d) = difference(x, y, &p) {
                            return Some(d);
                        }
                    }
                    _ => return Some(format!("{p}: missing/extra field")),
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                if let Some(d) = difference(x, y, &format!("{path}/{i}")) {
                    return Some(d);
                }
            }
            if a.len() != b.len() {
                return Some(format!("{path}: length {} != {}", a.len(), b.len()));
            }
        }
        _ if expected != actual => {
            return Some(format!("{path}: expected {expected}, actual {actual}"));
        }
        _ => {}
    }
    None
}
fn cursor(error: &str) -> Option<usize> {
    for prefix in ["/points/", "/play/"] {
        if let Some((_, rest)) = error.split_once(prefix)
            && let Ok(n) = rest.split(['/', ' ', ':']).next()?.parse()
        {
            return Some(n);
        }
    }
    None
}
fn replay(input: &Value, expected: &Value, entry: &Entry) -> Result<(), String> {
    let mut actual = std::panic::catch_unwind(|| mtg_core::game::replay_admission(input))
        .map_err(|_| "/replay: engine panic".to_string())??;
    let case = &input["cases"][0];
    for (field, tape) in [("consumed_play", &case["play"])] {
        if let Some(d) = difference(tape, &actual[field], field) {
            return Err(d);
        }
    }
    for name in ["chance", "choices"] {
        if let Some(d) = difference(
            &case[name],
            &actual["opening"][format!("consumed_{name}")],
            name,
        ) {
            return Err(d);
        }
    }
    if actual["points"].as_array().map(Vec::len) != Some(entry.checkpoints)
        || actual["consumed_play"].as_array().map(Vec::len) != Some(entry.actions)
        || actual["points"]
            .as_array()
            .and_then(|v| v.last())
            .is_none_or(|p| p["outcome"].is_null())
    {
        return Err("/completeness: actions/checkpoints/outcome".into());
    }
    for key in [
        "policy_capture",
        "stale_candidates_rejected",
        "continuation_rejections",
        "stops",
    ] {
        actual.as_object_mut().unwrap().remove(key);
    }
    fn canonical_handle(item: &mut Value, stores: &mut BTreeMap<u64, usize>) {
        if let Some(store) = item["raw_source_handle"]["store"].as_u64() {
            let next = stores.len();
            item["raw_source_handle"]["store"] = json!(*stores.entry(store).or_insert(next));
        }
    }
    let mut stores = BTreeMap::new();
    // Arena nonces differ between processes and between games. Semantic
    // occurrence, birth, incarnation, slot and generation remain compared.
    if let Some(points) = actual["points"].as_array_mut() {
        for point in points {
            for item in point["stack"].as_array_mut().into_iter().flatten() {
                canonical_handle(item, &mut stores);
            }
        }
    }
    for item in actual["trigger_provenance"]
        .as_array_mut()
        .into_iter()
        .flatten()
    {
        canonical_handle(item, &mut stores);
    }
    difference(expected, &actual, "").map_or(Ok(()), Err)
}
fn context(entry: &Entry) -> Value {
    json!({"id":entry.id,"seed":entry.seed,"trace":entry.trace,"cursor":null,"source_cursor":null})
}
fn run(args: &[OsString], receipt: &mut Value) -> Result<(), String> {
    if args.len() != 3 {
        return Err("usage: mtg l0 MANIFEST.json MANIFEST_SHA256".into());
    }
    if cfg!(debug_assertions) {
        return Err("/build: release runner required".into());
    }
    let runner =
        fs::read(std::env::current_exe().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    receipt["runner_sha256"] = json!(digest(&runner));
    let path = Path::new(&args[1]);
    if !fs::metadata(path)
        .map_err(|e| format!("/manifest: {e}"))?
        .is_file()
    {
        return Err("/manifest: regular local file required".into());
    }
    let bytes = fs::read(path).map_err(|e| format!("/manifest: {e}"))?;
    let hash = digest(&bytes);
    receipt["corpus_sha256"] = json!(hash);
    if args[2].to_str() != Some(hash.as_str()) {
        return Err("/manifest/checksum".into());
    }
    let manifest: Manifest =
        serde_json::from_slice(&bytes).map_err(|e| format!("/manifest: {e}"))?;
    receipt["expected"] = json!(manifest.game_count);
    receipt["corpus_version"] = json!(manifest.version);
    receipt["status"] = json!(manifest.status);
    if manifest.schema_version != 1
        || manifest.semantic_schema != "admission-v10-extended-v1"
        || manifest.status != "provisional-native-patched-xmage"
        || manifest.final_admitted_games != 0
        || manifest.games.len() != manifest.game_count
        || manifest.game_count == 0
    {
        return Err("/manifest/schema/count/scope".into());
    }
    let base = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    // Read and verify every listed artifact, including raw source and adjudication evidence.
    // Inputs are read again with their checksums before use; concurrent replacement cannot pass.
    let mut artifact_bytes = bytes.len() as u64;
    for (name, hash) in &manifest.artifacts {
        receipt["failure_context"] = manifest
            .games
            .iter()
            .find(|entry| [&entry.input, &entry.expected, &entry.trace].contains(&name))
            .map_or(
                json!({"trace":name,"seed":null,"cursor":null,"source_cursor":null}),
                context,
            );
        let data = read(&base, name)?;
        if digest(&data) != *hash {
            return Err(format!("/artifacts/{name}/checksum"));
        }
        artifact_bytes += data.len() as u64;
    }
    if manifest.artifacts.get("qualification.json") != Some(&manifest.qualification_sha256) {
        return Err("/qualification/checksum".into());
    }
    let mut ids = BTreeSet::new();
    for entry in &manifest.games {
        receipt["failure_context"] = context(entry);
        if !ids.insert(&entry.id)
            || entry.source_cursors.len() != entry.actions
            || entry.actions != entry.checkpoints
        {
            return Err("/manifest/identity/checkpoints".into());
        }
        for name in [&entry.input, &entry.expected, &entry.trace] {
            if !manifest.artifacts.contains_key(name) {
                return Err(format!("/artifacts/{name}: unlisted"));
            }
        }
    }
    receipt["artifact_bytes"] = json!(artifact_bytes);
    let mut latencies = vec![];
    let mut replay_seconds = 0.0;
    for entry in &manifest.games {
        let start = Instant::now();
        let result = (|| {
            let load = |name: &str| -> Result<Value, String> {
                let data = read(&base, name)?;
                if digest(&data) != manifest.artifacts[name] {
                    return Err(format!("/artifacts/{name}/checksum"));
                }
                serde_json::from_slice(&data).map_err(|e| format!("/artifacts/{name}: {e}"))
            };
            let input = load(&entry.input)?;
            let expected = load(&entry.expected)?;
            let loaded = Instant::now();
            let result = replay(&input, &expected, entry);
            replay_seconds += loaded.elapsed().as_secs_f64();
            result
        })();
        let elapsed = start.elapsed().as_secs_f64();
        latencies.push(elapsed);
        receipt["games"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":entry.id,"elapsed_seconds":elapsed,"passed":result.is_ok()}));
        match result {
            Ok(()) => {
                receipt["successful"] = json!(receipt["successful"].as_u64().unwrap() + 1);
                receipt["actions"] =
                    json!(receipt["actions"].as_u64().unwrap() + entry.actions as u64);
                receipt["checkpoints"] =
                    json!(receipt["checkpoints"].as_u64().unwrap() + entry.checkpoints as u64);
            }
            Err(error) => {
                receipt["failed"] = json!(receipt["failed"].as_u64().unwrap() + 1);
                let at = cursor(&error);
                receipt["errors"].as_array_mut().unwrap().push(json!({"id":entry.id,"seed":entry.seed,"trace":entry.trace,"cursor":at,"source_cursor":at.and_then(|n|entry.source_cursors.get(n)),"first_field":error}));
            }
        }
    }
    receipt["loaded_replay_comparison_seconds"] = json!(replay_seconds);
    receipt["loaded_games_per_second"] =
        json!(receipt["successful"].as_u64().unwrap() as f64 / replay_seconds);
    receipt["warmup_policy"] = json!(
        "none; loaded replay throughput excludes artifact load and parsing, not engine warmup"
    );
    latencies.sort_by(f64::total_cmp);
    receipt["p50_game_seconds"] = json!(latencies[(latencies.len() - 1) / 2]);
    receipt["p95_game_seconds"] = json!(latencies[(latencies.len() * 95).div_ceil(100) - 1]);
    Ok(())
}
pub fn execute(args: &[OsString]) -> i32 {
    let warm = args.last().is_some_and(|a| a == "--warm");
    let args = if warm { &args[..args.len() - 1] } else { args };
    let start = Instant::now();
    let mut receipt = json!({"schema_version":1,"kind":"native-l0","full_comparable_run":false,
        "cold_boundary":"in-process only; use scripts/run_l0.py for complete process boundary",
        "successful":0,"failed":0,"expected":null,"actions":0,"checkpoints":0,"errors":[],"games":[],
        "concurrency":1,"build_profile":if cfg!(debug_assertions){"debug"}else{"release"},"compilation_seconds":null,
        "instrumentation":"exact semantic reports including cleanup and damage quantum checks; excludes recorder and invalid-action diagnostic probes",
        "runner_sha256":null,"corpus_sha256":null,"source_sha256":env!("MTG_BENCH_SOURCE"),
        "rustc":env!("MTG_BENCH_RUSTC"),"target":env!("MTG_BENCH_TARGET")});
    if let Err(error) = run(args, &mut receipt) {
        let mut detail = receipt.get("failure_context").cloned().unwrap_or_else(
            || json!({"seed":null,"trace":null,"cursor":null,"source_cursor":null}),
        );
        detail["first_field"] = json!(error);
        receipt["errors"].as_array_mut().unwrap().push(detail);
    }
    receipt.as_object_mut().unwrap().remove("failure_context");
    receipt["in_process_seconds"] = json!(start.elapsed().as_secs_f64());
    receipt["peak_rss_kib"] = std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmHWM:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|n| n.parse::<u64>().ok())
        })
        .map_or(Value::Null, |n| json!(n));
    receipt["hardware"] = json!({"os":std::env::consts::OS,"arch":std::env::consts::ARCH,
        "available_parallelism":std::thread::available_parallelism().ok().map(|n|n.get())});
    let ok = receipt["errors"].as_array().unwrap().is_empty();
    receipt["complete"] = json!(ok);
    receipt["unattempted"] = receipt["expected"]
        .as_u64()
        .map(|n| {
            json!(n - receipt["successful"].as_u64().unwrap() - receipt["failed"].as_u64().unwrap())
        })
        .unwrap_or(Value::Null);
    if warm {
        use std::io::Write;
        println!("{}", json!({"phase":"cold_complete","receipt":receipt}));
        if std::io::stdout().flush().is_err() {
            return 1;
        }
        let mut warm_receipt = receipt.clone();
        for key in ["successful", "failed", "actions", "checkpoints"] {
            warm_receipt[key] = json!(0);
        }
        warm_receipt["errors"] = json!([]);
        warm_receipt["games"] = json!([]);
        for key in [
            "in_process_seconds",
            "complete",
            "unattempted",
            "peak_rss_kib",
        ] {
            warm_receipt.as_object_mut().unwrap().remove(key);
        }
        let warm_start = Instant::now();
        if let Err(error) = run(args, &mut warm_receipt) {
            warm_receipt["errors"]
                .as_array_mut()
                .unwrap()
                .push(json!({"first_field":error}));
        }
        let seconds = warm_start.elapsed().as_secs_f64();
        warm_receipt["elapsed_seconds"] = json!(seconds);
        warm_receipt["games_per_second"] =
            json!(warm_receipt["successful"].as_u64().unwrap() as f64 / seconds);
        warm_receipt["boundary"] = json!(
            "second full corpus load, integrity, replay and comparison in the same process; no discarded warmup"
        );
        let warm_ok = warm_receipt["errors"].as_array().unwrap().is_empty();
        warm_receipt["complete"] = json!(warm_ok);
        warm_receipt
            .as_object_mut()
            .unwrap()
            .remove("failure_context");
        warm_receipt["unattempted"] = warm_receipt["expected"]
            .as_u64()
            .map(|n| {
                json!(
                    n - warm_receipt["successful"].as_u64().unwrap()
                        - warm_receipt["failed"].as_u64().unwrap()
                )
            })
            .unwrap_or(Value::Null);
        receipt["warm"] = warm_receipt;
        receipt["complete"] = json!(ok && warm_ok);
        println!("{receipt}");
        return if ok && warm_ok { 0 } else { 1 };
    }
    println!("{receipt}");
    if ok { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_comparison_preserves_identities_order_and_missing_fields() {
        let expected = json!({"library":["copy/1", "copy/2"], "stack":[{"incarnation":2,"effect":3}], "life":[20,19]});
        for (changed, field) in [
            (
                json!({"library":["copy/2","copy/1"],"stack":[{"incarnation":2,"effect":3}],"life":[20,19]}),
                "/library/0",
            ),
            (
                json!({"library":["copy/1","copy/2"],"stack":[{"incarnation":3,"effect":3}],"life":[20,19]}),
                "/stack/0/incarnation",
            ),
            (
                json!({"library":["copy/1","copy/2"],"stack":[{"incarnation":2}],"life":[20,19]}),
                "/stack/0/effect",
            ),
        ] {
            assert!(
                difference(&expected, &changed, "")
                    .unwrap()
                    .starts_with(field)
            );
        }
        assert_eq!(difference(&expected, &expected, ""), None);
    }
}
