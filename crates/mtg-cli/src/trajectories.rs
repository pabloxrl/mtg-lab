//! Explicit local dataset selection; no directory discovery or replay resolution.
use mtg_recorder::{
    manifest::{EpisodeStatus, LoadMode, Manifest, RunEnd},
    schema::End,
};
use serde_json::{Value, json};
use std::{ffi::OsString, fs, io::Read, path::Path};
type Error = (i32, String);
const MAX_BYTES: usize = 16 * 1024 * 1024;
fn invalid() -> Error {
    (
        2,
        "trajectory invalid arguments, metadata or data; private detail withheld".into(),
    )
}
// Preserve legacy machine error categories without serializing private I/O details.
fn reader_error(error: mtg_recorder::Error) -> Error {
    let category = match error {
        mtg_recorder::Error::Invalid => "Invalid",
        mtg_recorder::Error::Incomplete => "Incomplete",
        mtg_recorder::Error::Integrity => "Integrity",
        mtg_recorder::Error::Limit => "Limit",
        mtg_recorder::Error::Overflow => "Overflow",
        mtg_recorder::Error::Poisoned => "Poisoned",
        mtg_recorder::Error::PublicationUncertain => "PublicationUncertain",
        mtg_recorder::Error::Io(_) => "Io",
    };
    (
        2,
        format!("trajectory: {category}; private detail withheld"),
    )
}
fn input_error() -> Error {
    (2, "trajectory input must be a readable regular file within the byte limit; private detail withheld".into())
}
fn read(path: &Path, remaining: &mut usize) -> Result<Vec<u8>, Error> {
    // Do not open FIFOs/devices or follow a final-component symlink. On Unix,
    // nonblocking/no-follow also closes the check/open race for these inputs.
    let meta = fs::symlink_metadata(path).map_err(|_| input_error())?;
    if !meta.is_file() || meta.len() > *remaining as u64 {
        return Err(input_error());
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).map_err(|_| input_error())?;
    let opened = file.metadata().map_err(|_| input_error())?;
    if !opened.is_file() || opened.len() > *remaining as u64 {
        return Err(input_error());
    }
    let mut bytes = Vec::new();
    file.take(*remaining as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| input_error())?;
    *remaining = remaining.checked_sub(bytes.len()).ok_or_else(input_error)?;
    Ok(bytes)
}
pub fn validate(args: &[OsString]) -> Result<Value, Error> {
    let path = args.first().ok_or_else(invalid)?;
    let mut format = None;
    let mut manifest = None;
    let mut diagnostic = false;
    let mut budget = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].to_str() {
            Some("--diagnostic") if !diagnostic => diagnostic = true,
            Some("--format") if format.is_none() => {
                i += 1;
                format = Some(args.get(i).and_then(|s| s.to_str()).ok_or_else(invalid)?);
            }
            Some("--manifest") if manifest.is_none() => {
                i += 1;
                manifest = Some(args.get(i).ok_or_else(invalid)?);
            }
            Some("--max-bytes") if budget.is_none() => {
                i += 1;
                let n = args
                    .get(i)
                    .and_then(|s| s.to_str())
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or_else(invalid)?;
                if !(1..=MAX_BYTES).contains(&n) {
                    return Err(invalid());
                }
                budget = Some(n);
            }
            _ => return Err(invalid()),
        }
        i += 1;
    }
    if manifest.is_some() && format.is_some() {
        return Err(invalid());
    }
    let mut remaining = budget.unwrap_or(MAX_BYTES);
    if let Some(manifest_path) = manifest {
        let bytes = read(Path::new(manifest_path), &mut remaining)?;
        let m = Manifest::parse(bytes.as_slice(), bytes.len()).map_err(|_| invalid())?;
        let name = Path::new(path)
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(invalid)?;
        // m.file.name is compared by the strict loader, never joined or opened.
        let data = read(Path::new(path), &mut remaining)?;
        let mode = if diagnostic {
            LoadMode::Diagnostic
        } else {
            LoadMode::CompletedOnly
        };
        // Format compatibility is fixed to v1/v2. Historical provenance strings
        // are declarations to cross-check, not authenticity or current-build pins.
        let (episodes, decisions) = match m.dataset_schema {
            1 => {
                let r = m
                    .load(name, data.as_slice(), &m.versions, data.len(), mode)
                    .map_err(|_| invalid())?;
                (
                    r.episodes().len(),
                    r.episodes()
                        .iter()
                        .map(|e| e.decisions.len())
                        .sum::<usize>(),
                )
            }
            2 => {
                let r = m
                    .load_v2(name, data.as_slice(), &m.versions, data.len(), mode)
                    .map_err(|_| invalid())?;
                (
                    r.episodes().len(),
                    r.episodes()
                        .iter()
                        .map(|e| e.decisions.len())
                        .sum::<usize>(),
                )
            }
            _ => return Err(invalid()),
        };
        let mut completed = 0;
        let mut truncated = 0;
        let mut failed = 0;
        let mut incomplete = 0;
        for e in &m.episodes {
            match &e.status {
                EpisodeStatus::Completed => completed += 1,
                EpisodeStatus::Truncated => truncated += 1,
                EpisodeStatus::Failed(_) => failed += 1,
                EpisodeStatus::Incomplete => incomplete += 1,
            }
        }
        let end = match m.end {
            RunEnd::Completed => "completed",
            RunEnd::Truncated(_) => "truncated",
            RunEnd::Failed(_) => "failed",
        };
        Ok(
            json!({"type":"trajectory_validation","status":if end=="completed" {"valid"} else {"valid_noncompleted"},
            "format":format!("run-manifest-v{}",m.dataset_schema),"episodes":episodes,"decisions":decisions,
            "declared_episodes":m.episodes.len(),"completed":completed,"truncated":truncated,"failed":failed,"incomplete":incomplete,
            "run_end":end,"recording_complete":m.recording_complete}),
        )
    } else {
        let format = format.unwrap_or("scalar-jsonl-v1");
        if !matches!(format, "scalar-jsonl-v1" | "structured-jsonl-v2") {
            return Err(invalid());
        }
        let bytes = read(Path::new(path), &mut remaining)?;
        match format {
            "scalar-jsonl-v1" => {
                // Preserve the original v1 command, including its summary and
                // acceptance of valid stored truncations. Run loading is stricter.
                let episodes =
                    mtg_recorder::read(bytes.as_slice(), bytes.len()).map_err(reader_error)?;
                let decisions: usize = episodes.iter().map(|e| e.decisions.len()).sum();
                Ok(
                    json!({"type":"trajectory_validation","status":"valid","format":format,"episodes":episodes.len(),"decisions":decisions}),
                )
            }
            "structured-jsonl-v2" => {
                let episodes =
                    mtg_recorder::read_v2(bytes.as_slice(), bytes.len()).map_err(|_| invalid())?;
                let mut completed = 0;
                let mut truncated = 0;
                for e in &episodes {
                    match &e.footer.as_ref().ok_or_else(invalid)?.end {
                        End::Completed => completed += 1,
                        End::Truncated(_) => truncated += 1,
                        End::Failed(_) => return Err(invalid()),
                    }
                }
                if truncated > 0 && !diagnostic {
                    return Err(invalid());
                }
                let decisions: usize = episodes.iter().map(|e| e.decisions.len()).sum();
                Ok(
                    json!({"type":"trajectory_validation","status":if truncated==0 {"valid"} else {"valid_noncompleted"},
                    "format":format,"episodes":episodes.len(),"decisions":decisions,"completed":completed,"truncated":truncated}),
                )
            }
            _ => unreachable!(),
        }
    }
}
