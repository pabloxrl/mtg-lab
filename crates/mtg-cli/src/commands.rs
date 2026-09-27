//! Thin unattended clients of existing replay, recorder and comparator contracts.
use crate::simulate;
use mtg_core::{objects::Seat, opening::replay};
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::Path,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};
type Error = (i32, String);
const INPUT_LIMIT: usize = 16 * 1024 * 1024;
const USAGE: &str = "usage: mtg replay verify FILE | replay inspect FILE --seat 0|1 --format jsonl | trajectories validate FILE | conformance --suite checkpoints-v1 --fixture FILE --actual FILE --artifacts NEW_DIR | bench --workload scalar-pass-v1 --config FILE; optional trailing --output NEW_FILE; unsupported suites/references fail";
fn invalid(message: impl Into<String>) -> Error {
    (2, message.into())
}
fn io_error(e: impl std::fmt::Display) -> Error {
    (3, e.to_string())
}
fn read(path: &std::ffi::OsStr, limit: usize) -> Result<Vec<u8>, Error> {
    let meta = fs::metadata(path).map_err(|e| invalid(format!("input: {e}")))?;
    if !meta.is_file() || meta.len() > limit as u64 {
        return Err(invalid(
            "input must be a regular file within the byte limit",
        ));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| invalid(format!("input: {e}")))?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| invalid(format!("input: {e}")))?;
    if bytes.len() > limit {
        return Err(invalid("input exceeds byte limit"));
    }
    Ok(bytes)
}
fn stopped(signal: &AtomicUsize) -> Option<simulate::Stop> {
    match signal.load(Ordering::Relaxed) {
        n if n == signal_hook::consts::SIGTERM as usize => Some(simulate::Stop::Sigterm),
        n if n == signal_hook::consts::SIGINT as usize => Some(simulate::Stop::Sigint),
        _ => None,
    }
}
fn replay_error(e: replay::ReplayError, inspect: bool) -> Error {
    if inspect {
        // Replay divergence includes privileged expected/actual values. Never
        // forward those into a player-visible inspection diagnostic.
        let kind = match e {
            replay::ReplayError::Divergence { .. } => "Divergence",
            replay::ReplayError::MissingChoice { .. } => "MissingChoice",
            replay::ReplayError::UnconsumedChoice { .. } => "UnconsumedChoice",
            replay::ReplayError::InvalidChoice { .. } => "InvalidChoice",
            replay::ReplayError::Incompatible { .. } => "Incompatible",
            replay::ReplayError::InvalidConfig(_) => "InvalidConfig",
            replay::ReplayError::Storage(_) => "Storage",
            replay::ReplayError::Malformed => "Malformed",
        };
        invalid(format!("replay {kind}; privileged detail withheld"))
    } else {
        invalid(format!("replay: {e:?}"))
    }
}
pub fn execute(args: &[OsString], signal: &AtomicUsize) -> Result<i32, Error> {
    let (args, output) = if args.len() >= 2 && args[args.len() - 2] == "--output" {
        (&args[..args.len() - 2], Some(&args[args.len() - 1]))
    } else {
        (args, None)
    };
    let is = |i: usize, value: &str| args.get(i).is_some_and(|s| s == value);
    let (code, mut value) = if is(0, "replay") && (is(1, "verify") || is(1, "inspect")) {
        let inspect = is(1, "inspect");
        if (!inspect && args.len() != 3)
            || (inspect
                && (args.len() != 7 || !is(3, "--seat") || !is(5, "--format") || !is(6, "jsonl")))
        {
            return Err(invalid(USAGE));
        }
        let seat = if inspect {
            match args[4].to_str() {
                Some("0") => Seat::P0,
                Some("1") => Seat::P1,
                _ => return Err(invalid("seat must be 0 or 1")),
            }
        } else {
            Seat::P0
        };
        let game =
            replay::verify(&read(&args[2], INPUT_LIMIT)?).map_err(|e| replay_error(e, inspect))?;
        let value = if inspect {
            let view = game
                .observe(seat)
                .map_err(|_| invalid("replay observation unavailable"))?;
            json!({"type":"replay_inspection","scope":"opening-v1","checkpoint":"final","observation":view})
        } else {
            json!({"type":"replay_verification","status":"verified","scope":"opening-v1"})
        };
        (0, value)
    } else if args.len() == 3 && is(0, "trajectories") && is(1, "validate") {
        let bytes = read(&args[2], INPUT_LIMIT)?;
        let episodes = mtg_recorder::read(bytes.as_slice(), INPUT_LIMIT)
            .map_err(|e| invalid(format!("trajectory: {e:?}")))?;
        let decisions: usize = episodes.iter().map(|e| e.decisions.len()).sum();
        (
            0,
            json!({"type":"trajectory_validation","status":"valid","format":"scalar-jsonl-v1","episodes":episodes.len(),"decisions":decisions}),
        )
    } else if args.len() == 5
        && is(0, "bench")
        && is(1, "--workload")
        && is(2, "scalar-pass-v1")
        && is(3, "--config")
    {
        benchmark(&args[4], signal)?
    } else if args.len() == 9
        && is(0, "conformance")
        && is(1, "--suite")
        && is(2, "checkpoints-v1")
        && is(3, "--fixture")
        && is(5, "--actual")
        && is(7, "--artifacts")
    {
        conformance(&args[4], &args[6], &args[8], signal)?
    } else {
        return Err(invalid(USAGE));
    };
    value["schema_version"] = json!(1);
    let mut sink: Box<dyn Write> = match output {
        Some(path) => Box::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|e| io_error(format!("output: {e}")))?,
        ),
        None => Box::new(io::stdout().lock()),
    };
    serde_json::to_writer(&mut sink, &value).map_err(io_error)?;
    sink.write_all(b"\n")
        .and_then(|_| sink.flush())
        .map_err(io_error)?;
    Ok(code)
}
fn benchmark(path: &std::ffi::OsStr, signal: &AtomicUsize) -> Result<(i32, Value), Error> {
    let c: simulate::Config =
        serde_json::from_slice(&read(path, 1_048_576)?).map_err(|e| invalid(e.to_string()))?;
    c.validate().map_err(invalid)?;
    if c.episodes > 100 || c.max_decisions > 10_000 {
        return Err(invalid(
            "scalar smoke limits: at most 100 episodes and 10000 decisions per episode",
        ));
    }
    // Includes reset, policy, game work, observations used by simulation, and
    // summary JSON encoding. No terminal rendering occurs in this window.
    let start = Instant::now();
    let mut bytes = Vec::new();
    let code = simulate::run(
        &c,
        &mut bytes,
        || {
            stopped(signal).or_else(|| {
                let limit = c.deadline_ms.unwrap_or(10_000).min(10_000);
                (start.elapsed().as_millis() >= limit as u128).then_some(simulate::Stop::Deadline)
            })
        },
        simulate::step,
    )
    .map_err(io_error)?;
    let elapsed = start.elapsed().as_nanos();
    let mut rows = bytes
        .split(|b| *b == b'\n')
        .filter(|b| !b.is_empty())
        .map(serde_json::from_slice::<Value>);
    let run = rows
        .next()
        .ok_or_else(|| io_error("missing run header"))?
        .map_err(io_error)?;
    let summary = rows
        .next_back()
        .ok_or_else(|| io_error("missing summary"))?
        .map_err(io_error)?;
    Ok((
        code,
        json!({"type":"benchmark","workload":"scalar-pass-v1","qualification":"smoke-only","measurement":"reset-policy-core-and-summary-encoding","elapsed_ns":elapsed,"workers":1,"run":run,"summary":summary,"max_wall_ms":10_000,"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"debug_assertions":cfg!(debug_assertions)}),
    ))
}
fn conformance(
    fixture: &std::ffi::OsStr,
    actual: &std::ffi::OsStr,
    artifacts: &std::ffi::OsStr,
    signal: &AtomicUsize,
) -> Result<(i32, Value), Error> {
    // The comparator is a repository tool, not an engine or a fabricated
    // reference. Its existing strict choice checks and artifacts are retained.
    read(fixture, 1_048_576)?;
    read(actual, 1_048_576)?;
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/checkpoints.py");
    if !script.is_file() {
        return Err(io_error("missing repository checkpoint tooling"));
    }
    let mut child = Command::new("python3")
        .arg("-B")
        .arg(script)
        .arg(fixture)
        .arg(actual)
        .arg("--artifacts")
        .arg(artifacts)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .spawn()
        .map_err(|e| io_error(format!("python3 checkpoint dependency: {e}")))?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    fn capture(stream: impl Read) -> io::Result<Vec<u8>> {
        let mut b = Vec::new();
        stream.take(1_048_577).read_to_end(&mut b)?;
        Ok(b)
    }
    let out = thread::spawn(move || capture(stdout));
    let err = thread::spawn(move || capture(stderr));
    let until = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(s) = child.try_wait().map_err(io_error)? {
            break Ok(s);
        }
        if stopped(signal).is_some() || Instant::now() >= until {
            let _ = child.kill();
            let _ = child.wait();
            break Err((4,"conformance interrupted or timed out after at most 10 seconds; artifacts may be incomplete".into()));
        }
        thread::sleep(Duration::from_millis(5));
    };
    let bytes = out
        .join()
        .map_err(|_| io_error("comparator reader failed"))?
        .map_err(io_error)?;
    let errors = err
        .join()
        .map_err(|_| io_error("comparator reader failed"))?
        .map_err(io_error)?;
    let status = status?;
    if bytes.len() > 1_048_576 || errors.len() > 1_048_576 {
        return Err(io_error("comparator output exceeds byte limit"));
    }
    let mut value: Value = serde_json::from_slice(&bytes).map_err(|_| {
        io_error(
            "checkpoint tooling failed to produce JSON; check repository and Python dependencies",
        )
    })?;
    let code = status.code().unwrap_or(3);
    if code != 0 && code != 1 {
        return Err((
            if code == 2 { 2 } else { 3 },
            format!(
                "invalid conformance input or artifacts: {}",
                value["message"]
            ),
        ));
    }
    if !errors.is_empty()
        || (code == 0 && value["status"] != "pass")
        || (code == 1 && value["status"] != "mismatch")
    {
        return Err(io_error("inconsistent checkpoint comparator result"));
    }
    value["type"] = json!("conformance");
    value["scope"] = json!("supplied-checkpoints-only");
    Ok((code, value))
}
