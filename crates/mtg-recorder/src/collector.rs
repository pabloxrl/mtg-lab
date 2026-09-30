//! Persistence boundary for actual owned scalar results; no publication.
use crate::{Backpressure, Error, Metrics, manifest::Manifest};
use mtg_core::{
    episode::EpisodeResult,
    game::Config,
    trajectory::{Header, Limits},
};
use std::io::Write;

#[derive(Clone, Debug)]
pub struct Run {
    pub id: String,
    pub config: Config,
    /// Trusted caller identities; policy execution cannot establish these names.
    pub policies: [String; 2],
    pub limits: Limits,
    pub first_ordinal: u64,
    pub started: u64,
}
#[derive(Clone, Copy)]
pub struct Storage {
    pub queue_bytes: usize,
    pub max_bytes: usize,
    pub backpressure: Backpressure,
}
#[derive(Debug)]
pub struct Bundle {
    manifest: Manifest,
    bytes: Vec<u8>,
    metrics: Metrics,
}
impl Bundle {
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
    pub fn into_parts(self) -> (Manifest, Vec<u8>, Metrics) {
        (self.manifest, self.bytes, self.metrics)
    }
}
fn hash(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}
fn json(value: &impl serde::Serialize) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|_| Error::Invalid)
}
impl Run {
    /// Derived capture header. Use before reset; persistence independently checks
    /// it against the immutable actual inputs. No seed or replay access escapes.
    pub fn header(&self, ordinal: u64) -> Result<Header, Error> {
        use mtg_core::{
            game::{policy, snapshot},
            trajectory::{EpisodeKey, Versions},
        };
        if !crate::uuid(&self.id)
            || self.config.seats.len() != 2
            || self.config.starting_seat > 1
            || !self.policies.iter().all(|p| crate::nonempty(p))
            || [
                self.limits.decisions,
                self.limits.turns,
                self.limits.wall_time_ms,
            ]
            .contains(&Some(0))
        {
            return Err(Error::Invalid);
        }
        let pool = include_bytes!("../../../data/cards/foundations_micro_v1.json");
        let catalog: serde_json::Value =
            serde_json::from_slice(pool).map_err(|_| Error::Invalid)?;
        let mut deck_hashes = [String::new(), String::new()];
        for (i, seat) in self.config.seats.iter().enumerate() {
            let deck = catalog["decks"]
                .as_array()
                .ok_or(Error::Invalid)?
                .iter()
                .find(|d| d["id"].as_str() == Some(&seat.deck))
                .ok_or(Error::Invalid)?;
            deck_hashes[i] = hash(&json(deck)?);
        }
        Ok(Header {
            id: EpisodeKey {
                run: self.id.clone(),
                ordinal,
            },
            versions: Versions {
                schema: mtg_core::trajectory::v2::SCHEMA_VERSION,
                engine: snapshot::engine().into(),
                rules: hash(include_bytes!("../../../data/rules/cr-2026-09-25.json")),
                cards: hash(pool),
                action: format!("policy-v{}", policy::SCHEMA_VERSION),
                observation: policy::SCHEMA_VERSION,
            },
            deck_hashes,
            config_hash: hash(&json(&self.config)?),
            policies: self.policies.clone(),
            starting_seat: self.config.starting_seat,
            limits: self.limits.clone(),
            restricted_replay: None,
        })
    }
    pub fn persist(&self, results: &[EpisodeResult], storage: Storage) -> Result<Bundle, Error> {
        self.persist_with_sink(results, storage, Vec::new())
    }
    /// The sink is empty caller-owned scratch, never an advertised dataset.
    /// Its byte view must reflect actual writes. Validate after seal AND flush;
    /// errors return no bundle and leave every borrowed result intact. Sink I/O
    /// deadlines are caller-owned. This function performs no durable publication.
    pub fn persist_with_sink<W: Write + AsRef<[u8]>>(
        &self,
        results: &[EpisodeResult],
        storage: Storage,
        sink: W,
    ) -> Result<Bundle, Error> {
        use crate::{manifest::*, schema::*};
        use mtg_core::episode::Status;
        if self.started != results.len() as u64 || !sink.as_ref().is_empty() {
            return Err(Error::Invalid);
        }
        let h = self.header(self.first_ordinal)?;
        let header: crate::schema::Header =
            serde_json::from_slice(&json(&h)?).map_err(|_| Error::Invalid)?;
        let mut manifest = Manifest {
            dataset_schema: 2,
            run: self.id.clone(),
            versions: header.versions,
            deck_hashes: header.deck_hashes,
            config_hash: header.config_hash,
            policies: self.policies.clone(),
            seats: [0, 1],
            starting_seat: self.config.starting_seat,
            limits: header.limits,
            reward: RewardConvention::SparseZeroSumTerminal,
            discount: DiscountConvention::UndiscountedEpisodic,
            time: TimeConvention::DecisionAndLogicalAction,
            capture: Capture::AllEpisodes {
                first_ordinal: self.first_ordinal,
                count: self.started,
            },
            episodes: Vec::new(),
            end: RunEnd::Completed,
            recording_complete: true,
            file: FileInventory {
                name: "episodes.jsonl".into(),
                format: 2,
                bytes: 0,
                sha256: String::new(),
                episodes: 0,
                decisions: 0,
            },
        };
        let mut writer = crate::Writer::new_v2(
            Bounded {
                sink,
                limit: storage.max_bytes,
                written: 0,
            },
            storage.queue_bytes,
            storage.backpressure,
        )?;
        for (index, result) in results.iter().enumerate() {
            let ordinal = self
                .first_ordinal
                .checked_add(index as u64)
                .ok_or(Error::Limit)?;
            let input = result.privileged_inputs();
            let actual_limits = result
                .budget()
                .map(|b| b.limits.clone())
                .unwrap_or_default();
            if input.ordinal != ordinal
                || json(&input.config)? != json(&self.config)?
                || actual_limits != self.limits
                || !result.capture_requested()
            {
                return Err(Error::Invalid);
            }
            let mut expected = self.header(ordinal)?;
            // Opaque linkage only; publication separately validates authorization,
            // actual result/history binding and replay bytes before advertising.
            if let Some(id) = result
                .trajectory()
                .and_then(|t| t.header().restricted_replay.as_ref())
            {
                if !crate::uuid(id) || !matches!(result.status(), Status::Completed(_)) {
                    return Err(Error::Invalid);
                }
                expected.restricted_replay = Some(id.clone());
            }
            if result.trajectory().is_some_and(|t| t.header() != &expected) {
                return Err(Error::Invalid);
            }
            let status = match result.status() {
                Status::Completed(_) => EpisodeStatus::Completed,
                // Internal-work budget stops have no final policy frame and cannot
                // satisfy the manifest's stored-Truncated contract. Account the
                // missing recording explicitly; keep the actual stop in the result.
                Status::Truncated(_) if result.final_observations().is_none() => {
                    EpisodeStatus::Incomplete
                }
                Status::Truncated(_) => EpisodeStatus::Truncated,
                Status::Failed(reason) => EpisodeStatus::Failed(format!("{reason:?}")),
                Status::Incomplete => EpisodeStatus::Incomplete,
            };
            match &status {
                EpisodeStatus::Completed | EpisodeStatus::Truncated => {
                    let episode =
                        crate::from_core_v2(result.trajectory().ok_or(Error::Incomplete)?)?;
                    let footer = episode.footer.as_ref().ok_or(Error::Incomplete)?;
                    let expected_end = match result.status() {
                        Status::Completed(_) => End::Completed,
                        Status::Truncated(reason) => serde_json::from_slice(&json(
                            &mtg_core::trajectory::End::Truncated(reason),
                        )?)
                        .map_err(|_| Error::Invalid)?,
                        _ => unreachable!(),
                    };
                    if footer.end != expected_end
                        || episode.decisions.len() as u64 != result.accepted_decisions()
                        || serde_json::to_value(&footer.final_observations)
                            .map_err(|_| Error::Invalid)?
                            != serde_json::to_value(
                                result.final_observations().ok_or(Error::Incomplete)?,
                            )
                            .map_err(|_| Error::Invalid)?
                    {
                        return Err(Error::Invalid);
                    }
                    writer.append_v2(&episode)?;
                    manifest.file.episodes += 1;
                    manifest.file.decisions = manifest
                        .file
                        .decisions
                        .checked_add(result.accepted_decisions())
                        .ok_or(Error::Limit)?;
                    if matches!(status, EpisodeStatus::Truncated)
                        && !matches!(manifest.end, RunEnd::Failed(_))
                    {
                        manifest.end = RunEnd::Truncated("episode limit".into());
                    }
                }
                EpisodeStatus::Failed(reason) => {
                    manifest.recording_complete = false;
                    manifest.end = RunEnd::Failed(reason.clone());
                }
                EpisodeStatus::Incomplete => {
                    manifest.recording_complete = false;
                    if !matches!(manifest.end, RunEnd::Failed(_)) {
                        manifest.end = RunEnd::Truncated(match result.status() {
                            Status::Truncated(reason) => {
                                format!("{reason:?}: recording incomplete during internal work")
                            }
                            _ => "unfinished episode".into(),
                        });
                    }
                }
            }
            manifest.episodes.push(DeclaredEpisode { ordinal, status });
        }
        let (sink, metrics) = writer.finish_with_metrics()?;
        let bytes = sink.sink.as_ref();
        if bytes.len() > storage.max_bytes {
            return Err(Error::Limit);
        }
        manifest.file.bytes = bytes.len() as u64;
        manifest.file.sha256 = hash(bytes);
        // Even diagnostic runs must be fully sealed and structurally valid.
        manifest.load_v2(
            &manifest.file.name,
            bytes,
            &manifest.versions,
            storage.max_bytes,
            LoadMode::Diagnostic,
        )?;
        Ok(Bundle {
            manifest,
            bytes: bytes.to_vec(),
            metrics,
        })
    }
}
struct Bounded<W> {
    sink: W,
    limit: usize,
    written: usize,
}
impl<W: Write> Write for Bounded<W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit.saturating_sub(self.written) {
            return Err(std::io::Error::other("sealed dataset byte budget exceeded"));
        }
        let n = self.sink.write(bytes)?;
        self.written += n;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.sink.flush()
    }
}
