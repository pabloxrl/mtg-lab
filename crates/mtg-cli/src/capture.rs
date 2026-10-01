//! CLI ownership and bounds only; canonical recording/publication live in libraries.
use crate::simulate::{Config as Simulation, Stop};
use mtg_core::{
    episode::{EpisodeResult, replay::Registry},
    trajectory::Limits,
};
use mtg_recorder::{
    Backpressure, FileHook, FileStage,
    collector::{Run, Storage},
    publication::{self, Destination},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::{self, Read},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub dataset_root: PathBuf,
    pub replay_root: PathBuf,
    pub authorization: String,
    pub max_episodes: u64,
    pub backpressure: QueueMode,
    pub queue_bytes: usize,
    /// Per JSONL, manifest, or replay artifact, not total disk/RSS usage.
    pub max_bytes: usize,
}
#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum QueueMode {
    Block,
    Fail,
}
impl Config {
    pub fn validate(&self, episodes: u64) -> Result<(), String> {
        if self.authorization != "local-owner-v1"
            || self.max_episodes == 0
            || episodes > self.max_episodes
            || usize::try_from(self.max_episodes).is_err()
            || self.queue_bytes == 0
            || self.max_bytes == 0
        {
            return Err("invalid capture authorization or bounds".into());
        }
        self.roots()
            .map(|_| ())
            .map_err(|_| "capture requires existing disjoint directory roots".into())
    }
    fn roots(&self) -> io::Result<(PathBuf, PathBuf)> {
        let a = std::fs::canonicalize(&self.dataset_root)?;
        let b = std::fs::canonicalize(&self.replay_root)?;
        if !a.is_dir() || !b.is_dir() || a.starts_with(&b) || b.starts_with(&a) {
            return Err(io::Error::other("invalid capture roots"));
        }
        Ok((a, b))
    }
}
fn id() -> io::Result<String> {
    // OS randomness is independent of game/policy RNG. Linux is the delivered host.
    let mut b = [0u8; 16];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut b)?;
    b[6] = (b[6] & 15) | 64;
    b[8] = (b[8] & 63) | 128;
    let h: String = b.iter().map(|n| format!("{n:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &h[..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..]
    ))
}
pub struct Session {
    pub run: Run,
    pub results: Vec<EpisodeResult>,
    datasets: PathBuf,
    replays: PathBuf,
}
impl Session {
    pub fn new(c: &Simulation, config: &Config) -> io::Result<Self> {
        config.validate(c.episodes).map_err(io::Error::other)?;
        let (datasets, replays) = config.roots()?;
        Ok(Self {
            run: Run {
                id: id()?,
                config: c.game.clone(),
                policies: c.policies.clone(),
                limits: Limits {
                    decisions: Some(c.max_decisions),
                    wall_time_ms: c.deadline_ms,
                    turns: None,
                },
                first_ordinal: c.first_episode,
                started: 0,
            },
            results: vec![],
            datasets,
            replays,
        })
    }
    pub fn publish(
        &mut self,
        c: &Config,
        started: u64,
        control: &mut impl FnMut() -> Option<Stop>,
        hook: &mut FileHook<'_>,
    ) -> (Value, Option<Stop>, bool) {
        self.run.started = started;
        let mut stop = control();
        let mut registry = Registry::default();
        let mut grants = Vec::new();
        let result = (|| {
            if stop.is_some() {
                return Err(mtg_recorder::Error::Incomplete);
            }
            for result in &mut self.results {
                let replay = id().map_err(mtg_recorder::Error::Io)?;
                if let mtg_core::episode::replay::Availability::Available(id) = registry
                    .register(&replay, result)
                    .map_err(|_| mtg_recorder::Error::Invalid)?
                {
                    grants.push((id, result.trajectory().unwrap().header().id.clone()));
                }
                stop = control();
                if stop.is_some() {
                    return Err(mtg_recorder::Error::Incomplete);
                }
            }
            publication::publish_observed(
                &self.run,
                &self.results,
                &registry,
                |id, key| {
                    grants
                        .iter()
                        .any(|(grant, episode)| grant == id && episode == key)
                },
                Destination {
                    datasets: &self.datasets,
                    replays: &self.replays,
                },
                Storage {
                    queue_bytes: c.queue_bytes,
                    max_bytes: c.max_bytes,
                    backpressure: match c.backpressure {
                        QueueMode::Block => Backpressure::Block,
                        QueueMode::Fail => Backpressure::Fail,
                    },
                },
                &mut |path, stage| {
                    // A stop must not itself prevent cleanup of a published link.
                    if stage != FileStage::Withdraw {
                        stop = stop.or_else(&mut *control);
                        if stop.is_some() {
                            return Err(io::Error::other("publication interrupted"));
                        }
                    }
                    hook(path, stage)
                },
            )
        })();
        let (status, reason) = match &result {
            Ok(()) => ("published", "manifest_committed"),
            Err(mtg_recorder::Error::PublicationUncertain) => ("uncertain", "commit_uncertain"),
            Err(_) if stop.is_some() => ("failed", "interrupted"),
            Err(_) => ("failed", "publication_error"),
        };
        (
            json!({"type":"publication","run_id":self.run.id,"status":status,"reason":reason,"started":started,"retained_results":self.results.len()}),
            stop,
            result.is_err(),
        )
    }
}

#[cfg(test)]
#[path = "capture_tests.rs"]
mod tests;
