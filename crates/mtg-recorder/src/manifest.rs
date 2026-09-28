//! Versioned scalar run metadata above the canonical JSONL format.
use crate::{Error, schema::*};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub dataset_schema: u32,
    pub run: String,
    pub versions: Versions,
    pub deck_hashes: [String; 2],
    pub config_hash: String,
    pub policies: [String; 2],
    pub seats: [u8; 2],
    pub starting_seat: u8,
    pub limits: Limits,
    pub reward: RewardConvention,
    pub discount: DiscountConvention,
    pub time: TimeConvention,
    pub capture: Capture,
    pub episodes: Vec<DeclaredEpisode>,
    pub end: RunEnd,
    pub recording_complete: bool,
    pub file: FileInventory,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeConvention {
    DecisionAndLogicalAction,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Capture {
    AllEpisodes {
        first_ordinal: u64,
        count: u64,
    },
    /// Declaration only: collector owns selection before observing outcomes.
    DeterministicSeedSubset {
        algorithm: String,
        config_hash: String,
        population: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredEpisode {
    pub ordinal: u64,
    pub status: EpisodeStatus,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum EpisodeStatus {
    Completed,
    Truncated,
    Failed(String),
    Incomplete,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum RunEnd {
    Completed,
    Truncated(String),
    Failed(String),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileInventory {
    pub name: String,
    pub format: u32,
    pub bytes: u64,
    pub sha256: String,
    pub episodes: u64,
    pub decisions: u64,
}
#[derive(Clone, Copy, Default)]
pub enum LoadMode {
    #[default]
    CompletedOnly,
    Diagnostic,
}
pub struct LoadedRun {
    episodes: Vec<Episode>,
}
impl LoadedRun {
    /// Authorized offline dataset access; contains both seats' observations.
    pub fn episodes(&self) -> &[Episode] {
        &self.episodes
    }
    /// Seat authorization is the caller's responsibility. No replay resolution.
    pub fn policy_decisions(&self, seat: u8) -> Result<impl Iterator<Item = &Decision>, Error> {
        if seat > 1 {
            return Err(Error::Invalid);
        }
        Ok(self
            .episodes
            .iter()
            .flat_map(|e| &e.decisions)
            .filter(move |d| d.actor == seat))
    }
}
impl Manifest {
    /// Parse bounded JSON with every field explicit (including nullable limits).
    /// This validates declarations; `load` additionally binds them to file bytes.
    pub fn parse(source: impl Read, max_bytes: usize) -> Result<Self, Error> {
        let bytes = bounded(source, max_bytes)?;
        let manifest: Self = serde_json::from_slice(&bytes).map_err(|_| Error::Invalid)?;
        let input: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| Error::Invalid)?;
        if input != serde_json::to_value(&manifest).map_err(|_| Error::Invalid)? {
            return Err(Error::Invalid);
        }
        manifest.validate()?;
        Ok(manifest)
    }
    pub fn encode(&self, max_bytes: usize) -> Result<Vec<u8>, Error> {
        self.validate()?;
        crate::canonical(self, max_bytes)
    }
    /// Check metadata without reading or resolving any artifact path.
    pub fn validate(&self) -> Result<(), Error> {
        let v = &self.versions;
        if self.dataset_schema != 1
            || v.schema != 1
            || v.observation != 1
            || ![&v.engine, &v.rules, &v.cards, &v.action]
                .iter()
                .all(|s| crate::nonempty(s))
            || !crate::uuid(&self.run)
            || self.seats != [0, 1]
            || self.starting_seat > 1
            || !self.deck_hashes.iter().all(|s| crate::hash(s))
            || !crate::hash(&self.config_hash)
            || !self.policies.iter().all(|s| crate::nonempty(s))
            || self.file.format != 1
            || !crate::hash(&self.file.sha256)
            || !safe_name(&self.file.name)
            || [
                self.limits.decisions,
                self.limits.turns,
                self.limits.wall_time_ms,
            ]
            .contains(&Some(0))
        {
            return Err(Error::Invalid);
        }
        if self
            .episodes
            .windows(2)
            .any(|w| w[0].ordinal >= w[1].ordinal)
        {
            return Err(Error::Invalid);
        }
        match &self.capture {
            Capture::AllEpisodes {
                first_ordinal,
                count,
            } => {
                if *count != self.episodes.len() as u64
                    || self
                        .episodes
                        .iter()
                        .enumerate()
                        .any(|(i, e)| first_ordinal.checked_add(i as u64) != Some(e.ordinal))
                {
                    return Err(Error::Invalid);
                }
            }
            Capture::DeterministicSeedSubset {
                algorithm,
                config_hash,
                population,
            } => {
                if !crate::nonempty(algorithm)
                    || !crate::hash(config_hash)
                    || self.episodes.iter().any(|e| e.ordinal >= *population)
                {
                    return Err(Error::Invalid);
                }
            }
        }
        let mut stored = 0u64;
        let mut unfinished = false;
        let mut truncated = false;
        let mut failed = false;
        for e in &self.episodes {
            match &e.status {
                EpisodeStatus::Completed => stored += 1,
                EpisodeStatus::Truncated => {
                    stored += 1;
                    truncated = true;
                }
                EpisodeStatus::Failed(reason) => {
                    if !crate::nonempty(reason) {
                        return Err(Error::Invalid);
                    }
                    failed = true;
                    unfinished = true;
                }
                EpisodeStatus::Incomplete => unfinished = true,
            }
        }
        let end_valid = match &self.end {
            RunEnd::Completed => !unfinished && !truncated,
            RunEnd::Truncated(reason) => {
                crate::nonempty(reason) && !failed && (unfinished || truncated)
            }
            RunEnd::Failed(reason) => crate::nonempty(reason) && failed,
        };
        if !end_valid || self.recording_complete == unfinished || self.file.episodes != stored {
            return Err(Error::Invalid);
        }
        Ok(())
    }
    /// Load a single caller-opened sealed JSONL file. Expected versions are the
    /// consumer's compatibility contract, not versions inferred from the input.
    /// Diagnostic mode exposes valid stored episodes only, never corrupt prefixes.
    pub fn load(
        &self,
        name: &str,
        source: impl Read,
        expected: &Versions,
        max_bytes: usize,
        mode: LoadMode,
    ) -> Result<LoadedRun, Error> {
        self.validate()?;
        if &self.versions != expected || self.file.name != name {
            return Err(Error::Invalid);
        }
        if self.file.bytes > max_bytes as u64 {
            return Err(Error::Limit);
        }
        let bytes = bounded(source, max_bytes)?;
        if bytes.len() as u64 != self.file.bytes
            || format!("{:x}", Sha256::digest(&bytes)) != self.file.sha256
        {
            return Err(Error::Integrity);
        }
        let episodes = crate::read(bytes.as_slice(), max_bytes)?;
        let declarations: Vec<_> = self
            .episodes
            .iter()
            .filter(|e| {
                matches!(
                    e.status,
                    EpisodeStatus::Completed | EpisodeStatus::Truncated
                )
            })
            .collect();
        if episodes.len() != declarations.len() {
            return Err(Error::Integrity);
        }
        let mut decisions = 0u64;
        for (episode, declaration) in episodes.iter().zip(declarations) {
            let h = &episode.header;
            let footer = episode.footer.as_ref().ok_or(Error::Incomplete)?;
            if h.id.run != self.run
                || h.id.ordinal != declaration.ordinal
                || h.versions != self.versions
                || h.deck_hashes != self.deck_hashes
                || h.config_hash != self.config_hash
                || h.policies != self.policies
                || h.starting_seat != self.starting_seat
                || h.limits != self.limits
                || episode.reward_convention != self.reward
                || episode.discount_convention != self.discount
                || h.restricted_replay
                    .as_ref()
                    .is_some_and(|id| !crate::uuid(id))
                || !matches!(
                    (&declaration.status, &footer.end),
                    (EpisodeStatus::Completed, End::Completed)
                        | (EpisodeStatus::Truncated, End::Truncated(_))
                )
            {
                return Err(Error::Invalid);
            }
            decisions = decisions
                .checked_add(episode.decisions.len() as u64)
                .ok_or(Error::Limit)?;
        }
        if decisions != self.file.decisions {
            return Err(Error::Integrity);
        }
        if matches!(mode, LoadMode::CompletedOnly) && !matches!(self.end, RunEnd::Completed) {
            return Err(Error::Incomplete);
        }
        Ok(LoadedRun { episodes })
    }
}
fn bounded(source: impl Read, limit: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    source
        .take((limit as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(Error::Io)?;
    if bytes.len() > limit {
        return Err(Error::Limit);
    }
    Ok(bytes)
}
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        && !name.ends_with(".partial")
}
