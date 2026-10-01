//! Local publication. See `doc/collector-publication.md` for failure/retry scope.
use crate::{
    Error, FileHook, FileStage,
    collector::{Run, Storage},
    manifest::LoadMode,
};
use mtg_core::{
    episode::{EpisodeResult, Status, replay::Registry},
    trajectory::EpisodeKey,
};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// Existing, separately managed roots. Neither may contain the other.
pub struct Destination<'a> {
    pub datasets: &'a Path,
    pub replays: &'a Path,
}
/// Validate actual owned results and authorized registry bytes before reserving
/// either run directory. Completed episodes require registered opaque replay IDs.
/// Noncompleted episodes retain diagnostic accounting and have no replay.
pub fn publish(
    run: &Run,
    results: &[EpisodeResult],
    registry: &Registry,
    authorize: impl FnMut(&str, &EpisodeKey) -> bool,
    destination: Destination<'_>,
    storage: Storage,
) -> Result<(), Error> {
    publish_observed(
        run,
        results,
        registry,
        authorize,
        destination,
        storage,
        &mut |_, _| Ok(()),
    )
}
/// Publish with a caller cancellation/fault boundary before each filesystem stage.
/// Hooks must allow withdrawal cleanup; no hard I/O deadline is promised.
pub fn publish_observed(
    run: &Run,
    results: &[EpisodeResult],
    registry: &Registry,
    mut authorize: impl FnMut(&str, &EpisodeKey) -> bool,
    destination: Destination<'_>,
    storage: Storage,
    hook: &mut FileHook<'_>,
) -> Result<(), Error> {
    let bundle = run.persist(results, storage)?;
    let m = bundle.manifest();
    let loaded = m.load_v2(
        &m.file.name,
        bundle.bytes(),
        &m.versions,
        storage.max_bytes,
        LoadMode::Diagnostic,
    )?;
    let mut replays = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    for result in results {
        if matches!(result.status(), Status::Completed(_)) {
            let id = result
                .trajectory()
                .and_then(|t| t.header().restricted_replay.as_deref())
                .ok_or(Error::Incomplete)?;
            if !crate::uuid(id) || !ids.insert(id) {
                return Err(Error::Invalid);
            }
            let bytes = registry
                .resolve(id, result, &mut authorize)
                .map_err(|_| Error::Invalid)?;
            if bytes.len() > storage.max_bytes {
                return Err(Error::Limit);
            }
            replays.push((id, bytes));
        }
    }
    let manifest_bytes = m.encode(storage.max_bytes)?;
    let (public, private) = roots(&destination)?;
    let public_run = public.join(&run.id);
    let private_run = private.join(&run.id);
    // Check both first, but create_dir remains the exclusive reservation. A race
    // or later failure may leave a reserved empty directory; it is never reused.
    if public_run.try_exists().map_err(Error::Io)? || private_run.try_exists().map_err(Error::Io)? {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "run reserved",
        )));
    }
    for (directory, parent) in [(&public_run, &public), (&private_run, &private)] {
        hook(directory, FileStage::Create).map_err(Error::Io)?;
        private_directory(directory)?;
        hook(directory, FileStage::DirectorySync).map_err(Error::Io)?;
        sync_dir(parent)?;
    }
    let dataset = public_run.join(&m.file.name);
    crate::write_file_observed(
        &dataset,
        loaded.episodes(),
        storage.queue_bytes,
        storage.backpressure,
        hook,
    )?;
    for (id, bytes) in &replays {
        write_artifact(&private_run.join(id), bytes, hook)?;
    }
    // Reopen actual storage. Never use in-memory checksums as proof of disk bytes.
    let disk = bounded(&dataset, storage.max_bytes)?;
    if disk != bundle.bytes() {
        return Err(Error::Integrity);
    }
    m.load_v2(
        &m.file.name,
        &disk[..],
        &m.versions,
        storage.max_bytes,
        LoadMode::Diagnostic,
    )?;
    for (id, bytes) in &replays {
        if bounded(&private_run.join(id), storage.max_bytes)? != *bytes {
            return Err(Error::Integrity);
        }
    }
    // Retained partial manifest is diagnostic intent, never an advertisement.
    // The only advertised boundary is the final manifest hard link.
    let manifest = public_run.join("manifest.json");
    write_artifact(&manifest, &manifest_bytes, hook)
}
fn roots(destination: &Destination<'_>) -> Result<(PathBuf, PathBuf), Error> {
    let public = fs::canonicalize(destination.datasets).map_err(Error::Io)?;
    let private = fs::canonicalize(destination.replays).map_err(Error::Io)?;
    if public.starts_with(&private)
        || private.starts_with(&public)
        || !public.is_dir()
        || !private.is_dir()
    {
        return Err(Error::Invalid);
    }
    Ok((public, private))
}
fn private_directory(path: &Path) -> Result<(), Error> {
    let mut builder = fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path).map_err(Error::Io)
}
fn sync_dir(path: &Path) -> Result<(), Error> {
    fs::File::open(path)
        .and_then(|f| f.sync_all())
        .map_err(Error::Io)
}
fn write_artifact(path: &Path, bytes: &[u8], hook: &mut FileHook<'_>) -> Result<(), Error> {
    let mut name = path.as_os_str().to_owned();
    name.push(".partial");
    let partial = PathBuf::from(name);
    hook(path, FileStage::Create).map_err(Error::Io)?;
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&partial).map_err(Error::Io)?;
    hook(path, FileStage::Write).map_err(Error::Io)?;
    file.write_all(bytes).map_err(Error::Io)?;
    hook(path, FileStage::Flush).map_err(Error::Io)?;
    file.flush().map_err(Error::Io)?;
    hook(path, FileStage::Sync).map_err(Error::Io)?;
    file.sync_all().map_err(Error::Io)?;
    // Readback precedes publication even for the manifest itself.
    if bounded(&partial, bytes.len())? != bytes {
        return Err(Error::Integrity);
    }
    hook(path, FileStage::Link).map_err(Error::Io)?;
    fs::hard_link(&partial, path).map_err(Error::Io)?;
    // On a returned post-link error, withdraw ONLY the link we just created.
    // Keep partial bytes for diagnosis. A process crash here can leave a valid
    // final manifest, since all artifacts have already been validated/synced.
    let finish = (|| {
        hook(path, FileStage::DirectorySync).map_err(Error::Io)?;
        sync_dir(path.parent().ok_or(Error::Invalid)?)
    })();
    if let Err(error) = finish {
        hook(path, FileStage::Withdraw).map_err(|_| Error::PublicationUncertain)?;
        fs::remove_file(path).map_err(|_| Error::PublicationUncertain)?;
        sync_dir(path.parent().ok_or(Error::Invalid)?).map_err(|_| Error::PublicationUncertain)?;
        return Err(error);
    }
    // Retain .partial as a recovery alias: no fallible cleanup after commit.
    Ok(())
}
fn bounded(path: &Path, max: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(Error::Io)?
        .take((max as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(Error::Io)?;
    if bytes.len() > max {
        return Err(Error::Limit);
    }
    Ok(bytes)
}
/// Separately authorized disk read. The trusted owner retains the actual result;
/// re-registration reconstructs its replay and verifies full config/history/state
/// binding. No registry lifetime or grant is persisted or inferred from an ID.
/// Denial happens before any filesystem access. Policy loaders never call this.
pub fn read_replay(
    root: &Path,
    id: &str,
    result: &EpisodeResult,
    authorize: impl FnOnce(&str, &EpisodeKey) -> bool,
    max_bytes: usize,
) -> Result<Vec<u8>, Error> {
    let h = result.trajectory().ok_or(Error::Incomplete)?.header();
    if !crate::uuid(id) || !crate::uuid(&h.id.run) || h.restricted_replay.as_deref() != Some(id) {
        return Err(Error::Invalid);
    }
    if !authorize(id, &h.id) {
        return Err(Error::Invalid);
    }
    let mut registry = Registry::default();
    let mut owned = result.clone();
    registry
        .register(id, &mut owned)
        .map_err(|_| Error::Invalid)?;
    let expected = registry
        .resolve(id, &owned, |_, _| true)
        .map_err(|_| Error::Invalid)?;
    if expected.len() > max_bytes {
        return Err(Error::Limit);
    }
    let bytes = bounded(&root.join(&h.id.run).join(id), max_bytes)?;
    if bytes != expected {
        return Err(Error::Integrity);
    }
    Ok(bytes)
}

#[cfg(test)]
#[path = "publication_tests.rs"]
mod tests;
