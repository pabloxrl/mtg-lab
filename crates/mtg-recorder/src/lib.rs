//! Bounded JSONL persistence for canonical scalar trajectories.
//!
//! See `doc/trajectory-jsonl.md`. Publication validates replays through the core;
//! the recorder does not implement game rules or consume game RNG streams.
pub mod collector;
pub mod manifest;
pub mod publication;
pub mod schema;
pub mod structured;
use schema::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum Error {
    Invalid,
    Incomplete,
    Integrity,
    Limit,
    Overflow,
    Poisoned,
    /// A published link could not be durably withdrawn after a sync failure.
    /// Inspect retained artifacts; never retry the reserved run ID automatically.
    PublicationUncertain,
    Io(std::io::Error),
}
#[derive(Clone, Copy, Default)]
pub enum Backpressure {
    #[default]
    Block,
    Fail,
}
#[derive(Clone, Debug, Default)]
pub struct Metrics {
    pub write_wait: Duration,
    pub batches: u64,
    pub bytes: u64,
    pub buffer_high_water: usize,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Record<E = Episode> {
    Episode {
        episode: Box<E>,
    },
    Seal {
        format: u32,
        episodes: u64,
        decisions: u64,
        sha256: String,
    },
}
#[derive(Default)]
struct Inventory {
    last: Option<EpisodeKey>,
    episodes: u64,
    decisions: u64,
}
impl Inventory {
    fn add<E: Durable>(&mut self, e: &E) -> Result<(), Error> {
        if self.last.as_ref().is_some_and(|last| {
            last.run != e.metadata().0.id.run || last.ordinal >= e.metadata().0.id.ordinal
        }) {
            return Err(Error::Invalid);
        }
        self.episodes = self.episodes.checked_add(1).ok_or(Error::Limit)?;
        self.decisions = self
            .decisions
            .checked_add(e.decision_count() as u64)
            .ok_or(Error::Limit)?;
        self.last = Some(e.metadata().0.id.clone());
        Ok(())
    }
}
/// An episode-sized bounded queue. Oversized episodes fail explicitly; callers
/// must enforce an episode budget. `finish` is mandatory, including for zero episodes.
pub struct Writer<W> {
    format: u32,
    sink: W,
    capacity: usize,
    mode: Backpressure,
    pending: Vec<u8>,
    digest: Sha256,
    inventory: Inventory,
    poisoned: bool,
    metrics: Metrics,
}
impl<W: Write> Writer<W> {
    pub fn new(sink: W, capacity: usize, mode: Backpressure) -> Result<Self, Error> {
        if capacity == 0 {
            return Err(Error::Limit);
        }
        Ok(Self {
            format: 1,
            sink,
            capacity,
            mode,
            pending: Vec::new(),
            digest: Sha256::new(),
            inventory: Inventory::default(),
            poisoned: false,
            metrics: Metrics::default(),
        })
    }
    /// Explicit v2 stream, including an empty dataset; never inferred from rows.
    pub fn new_v2(sink: W, capacity: usize, mode: Backpressure) -> Result<Self, Error> {
        let mut writer = Self::new(sink, capacity, mode)?;
        writer.format = 2;
        Ok(writer)
    }
    pub fn append_v2(&mut self, episode: &structured::Episode) -> Result<(), Error> {
        self.append_record(episode)
    }
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
    pub fn append(&mut self, episode: &Episode) -> Result<(), Error> {
        self.append_record(episode)
    }
    fn append_record<E: Durable>(&mut self, episode: &E) -> Result<(), Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        let result = self.append_inner(episode);
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    fn append_inner<E: Durable>(&mut self, episode: &E) -> Result<(), Error> {
        if self.format != E::FORMAT {
            return Err(Error::Invalid);
        }
        episode.validate_record()?;
        #[derive(Serialize)]
        struct Row<'a, E> {
            kind: &'static str,
            episode: &'a E,
        }
        let line = canonical(
            &Row {
                kind: "episode",
                episode,
            },
            self.capacity,
        )?;
        if line.len() > self.capacity - self.pending.len() {
            match self.mode {
                Backpressure::Fail => return Err(Error::Overflow),
                Backpressure::Block => self.flush_batch()?,
            }
        }
        self.inventory.add(episode)?;
        self.digest.update(&line);
        self.pending.extend_from_slice(&line);
        self.metrics.buffer_high_water = self.metrics.buffer_high_water.max(self.pending.len());
        Ok(())
    }
    fn flush_batch(&mut self) -> Result<(), Error> {
        if self.pending.is_empty() {
            return Ok(());
        }
        let start = Instant::now();
        let result = self.sink.write_all(&self.pending);
        self.metrics.write_wait += start.elapsed();
        result.map_err(Error::Io)?;
        self.metrics.bytes += self.pending.len() as u64;
        self.metrics.batches += 1;
        self.pending.clear();
        Ok(())
    }
    /// Explicit drain permits producers using Fail mode to make queue space.
    pub fn flush(&mut self) -> Result<(), Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        let result = self.flush_batch();
        if result.is_err() {
            self.poisoned = true;
        }
        result
    }
    pub fn finish(self) -> Result<W, Error> {
        self.finish_with_metrics().map(|(sink, _)| sink)
    }
    pub fn finish_with_metrics(mut self) -> Result<(W, Metrics), Error> {
        if self.poisoned {
            return Err(Error::Poisoned);
        }
        self.flush_batch()?;
        let seal: Record = Record::Seal {
            format: self.format,
            episodes: self.inventory.episodes,
            decisions: self.inventory.decisions,
            sha256: format!("{:x}", self.digest.finalize()),
        };
        // Fixed-size seal, separate from the caller's episode/batch capacity.
        let bytes = canonical(&seal, 512)?;
        let start = Instant::now();
        self.sink.write_all(&bytes).map_err(Error::Io)?;
        self.sink.flush().map_err(Error::Io)?;
        self.metrics.write_wait += start.elapsed();
        self.metrics.bytes += bytes.len() as u64;
        Ok((self.sink, self.metrics))
    }
}
struct Limited {
    bytes: Vec<u8>,
    limit: usize,
    exceeded: bool,
}
impl Write for Limited {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.limit - self.bytes.len() {
            self.exceeded = true;
            return Err(std::io::Error::other("JSONL record exceeds byte budget"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn canonical(value: &impl Serialize, limit: usize) -> Result<Vec<u8>, Error> {
    // Bound the first serialization before materializing the sorted JSON tree.
    let mut raw = Limited {
        bytes: Vec::new(),
        limit: limit.saturating_sub(1),
        exceeded: false,
    };
    if serde_json::to_writer(&mut raw, value).is_err() {
        return Err(if raw.exceeded {
            Error::Limit
        } else {
            Error::Invalid
        });
    }
    let sorted: serde_json::Value =
        serde_json::from_slice(&raw.bytes).map_err(|_| Error::Invalid)?;
    let mut bytes = serde_json::to_vec(&sorted).map_err(|_| Error::Invalid)?;
    bytes.push(b'\n');
    if bytes.len() > limit {
        return Err(Error::Limit);
    }
    Ok(bytes)
}
/// Copies only the canonical data, never game state, seeds or capability handles.
pub fn from_core(episode: &mtg_core::trajectory::Episode) -> Result<Episode, Error> {
    let e = serde_json::from_value(serde_json::to_value(episode).map_err(|_| Error::Invalid)?)
        .map_err(|_| Error::Invalid)?;
    validate(&e)?;
    Ok(e)
}
fn nonempty(s: &str) -> bool {
    !s.trim().is_empty()
}
fn hash(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
fn view(v: &PlayerView, seat: u8, starting: u8) -> bool {
    v.schema_version == 1
        && v.seat == seat
        && seat < 2
        && v.starting_seat == starting
        && v.acting_seat.is_none_or(|s| s < 2)
        && v.turn.as_ref().is_none_or(|(n, s, step)| {
            *n > 0
                && *s < 2
                && [
                    "upkeep",
                    "draw",
                    "precombat_main",
                    "beginning_combat",
                    "declare_attackers",
                    "declare_blockers",
                    "combat_damage",
                    "end_combat",
                    "postcombat_main",
                    "end",
                    "cleanup",
                ]
                .contains(&step.as_str())
        })
        && v.hand.len() == v.hand_counts[seat as usize]
        && v.hand.iter().all(|c| card(c) && c.owner == seat)
        && v.public_zones.iter().all(|z| {
            [
                "graveyard_0",
                "graveyard_1",
                "battlefield",
                "stack",
                "exile",
            ]
            .contains(&z.zone.as_str())
                && z.cards.iter().all(card)
        })
        && v.remembered.iter().all(|r| {
            r.owner < 2
                && nonempty(&r.card)
                && [
                    "library_0",
                    "library_1",
                    "hand_0",
                    "hand_1",
                    "graveyard_0",
                    "graveyard_1",
                    "battlefield",
                    "stack",
                    "exile",
                ]
                .contains(&r.zone_at_reveal.as_str())
        })
        && v.opening.as_ref().is_none_or(|o| {
            v.acting_seat == Some(seat)
                && ["keep_or_mulligan", "bottom"].contains(&o.kind.as_str())
                && o.candidates.iter().all(|c| nonempty(c))
        })
        && v.terminal.as_ref().is_none_or(|t| {
            v.acting_seat.is_none()
                && t.winner.is_none_or(|s| s < 2)
                && t.losses.iter().all(|l| {
                    l.as_ref()
                        .is_none_or(|s| ["life", "empty_draw", "concession"].contains(&s.as_str()))
                })
        })
}
fn card(c: &VisibleCard) -> bool {
    c.owner < 2 && c.controller < 2 && nonempty(&c.card)
}
/// Structural/temporal/reward validation, not a replay verifier or authenticity check.
pub fn validate(e: &Episode) -> Result<(), Error> {
    let h = &e.header;
    let v = &h.versions;
    if v.schema != 1
        || v.observation != 1
        || ![&v.engine, &v.rules, &v.cards, &v.action]
            .iter()
            .all(|s| nonempty(s))
        || !uuid(&h.id.run)
        || h.starting_seat > 1
        || !h.deck_hashes.iter().all(|s| hash(s))
        || !hash(&h.config_hash)
        || !h.policies.iter().all(|s| nonempty(s))
        || h.restricted_replay.as_ref().is_some_and(|s| !nonempty(s))
    {
        return Err(Error::Invalid);
    }
    let f = e
        .footer
        .as_ref()
        .filter(|f| f.complete && !matches!(f.end, End::Failed(_)))
        .ok_or(Error::Incomplete)?;
    if f.decisions != e.decisions.len()
        || !f
            .final_observations
            .iter()
            .enumerate()
            .all(|(s, v)| view(v, s as u8, h.starting_seat))
    {
        return Err(Error::Invalid);
    }
    let terminal = &f.final_observations[0].terminal;
    if terminal != &f.final_observations[1].terminal {
        return Err(Error::Invalid);
    }
    let completed = matches!(f.end, End::Completed);
    if completed != terminal.is_some() {
        return Err(Error::Invalid);
    }
    let expected = match terminal.as_ref().and_then(|t| t.winner) {
        Some(0) => [1, -1],
        Some(1) => [-1, 1],
        _ => [0, 0],
    };
    if f.returns != expected {
        return Err(Error::Invalid);
    }
    if let Some(t) = terminal {
        let valid = match t.winner {
            Some(0) => t.losses[0].is_none() && t.losses[1].is_some(),
            Some(1) => t.losses[1].is_none() && t.losses[0].is_some(),
            None => t.losses.iter().all(Option::is_some),
            _ => false,
        };
        if !valid {
            return Err(Error::Invalid);
        }
    }
    let mut seats = [0, 0];
    let mut actions = 0;
    let mut sums = [
        i64::from(f.boundary_reward[0]),
        i64::from(f.boundary_reward[1]),
    ];
    if f.boundary_reward != [0, 0] && f.boundary_reward != expected {
        return Err(Error::Invalid);
    }
    for (i, d) in e.decisions.iter().enumerate() {
        let c = &d.choice;
        let last = i + 1 == e.decisions.len();
        let sequence = if i == 0 {
            c.logical_action == 0 && c.micro_choice == 0
        } else {
            let prev = &e.decisions[i - 1].choice;
            (c.logical_action == prev.logical_action
                && prev.micro_choice.checked_add(1) == Some(c.micro_choice))
                || (prev.logical_action.checked_add(1) == Some(c.logical_action)
                    && c.micro_choice == 0)
        };
        if d.actor > 1
            || d.episode != h.id
            || d.index != i
            || d.seat_index != seats[d.actor as usize]
            || !view(&d.observation, d.actor, h.starting_seat)
            || d.observation.acting_seat != Some(d.actor)
            || d.observation.terminal.is_some()
            || !sequence
            || !nonempty(&c.kind)
            || c.candidates.is_empty()
            || c.legal_mask.len() != c.candidates.len()
            || c.legal_mask.get(c.selected) != Some(&true)
            || c.candidates.iter().any(|c| !nonempty(&c.semantic))
            || c.candidates[c.selected].semantic != d.action
            || c.policy
                .log_probability
                .is_some_and(|p| !p.is_finite() || p > 0.0)
            || c.policy.value.is_some_and(|p| !p.is_finite())
            || d.terminated != (last && completed)
            || d.truncated != (last && !completed)
            || d.next_actor
                != if last {
                    None
                } else {
                    Some(e.decisions[i + 1].actor)
                }
            || (d.reward != [0, 0] && !(last && completed && d.reward == expected))
        {
            return Err(Error::Invalid);
        }
        if c.micro_choice == 0 {
            actions += 1;
        }
        seats[d.actor as usize] += 1;
        for (s, sum) in sums.iter_mut().enumerate() {
            *sum += i64::from(d.reward[s]);
        }
    }
    if actions != f.logical_actions || sums != expected.map(i64::from) {
        return Err(Error::Invalid);
    }
    Ok(())
}
/// Loads only a complete, canonical, sealed file. The explicit byte budget bounds
/// input and deserialization memory; no unbounded line or identity set is used.
pub fn read(source: impl Read, max_bytes: usize) -> Result<Vec<Episode>, Error> {
    read_typed(source, max_bytes)
}
/// Explicit structured schema v2 reader. V1 and unknown versions are rejected.
pub fn read_v2(source: impl Read, max_bytes: usize) -> Result<Vec<structured::Episode>, Error> {
    read_typed(source, max_bytes)
}
fn read_typed<E: Durable>(source: impl Read, max_bytes: usize) -> Result<Vec<E>, Error> {
    let mut bytes = Vec::new();
    source
        .take((max_bytes as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(Error::Io)?;
    if bytes.len() > max_bytes {
        return Err(Error::Limit);
    }
    if bytes.is_empty() || !bytes.ends_with(b"\n") {
        return Err(Error::Incomplete);
    }
    let mut episodes = vec![];
    let mut inventory = Inventory::default();
    let mut digest = Sha256::new();
    let mut sealed = false;
    for line in bytes.split_inclusive(|&b| b == b'\n') {
        if sealed {
            return Err(Error::Invalid);
        }
        let record: Record<E> = serde_json::from_slice(line).map_err(|_| Error::Invalid)?;
        // Reject unknown/missing/duplicate fields and noncanonical representations.
        if canonical(&record, max_bytes)? != line {
            return Err(Error::Invalid);
        }
        match record {
            Record::Episode { episode } => {
                episode.validate_record()?;
                inventory.add(episode.as_ref())?;
                digest.update(line);
                episodes.push(*episode);
            }
            Record::Seal {
                format,
                episodes,
                decisions,
                sha256,
            } => {
                if format != E::FORMAT {
                    return Err(Error::Invalid);
                }
                if episodes != inventory.episodes
                    || decisions != inventory.decisions
                    || sha256 != format!("{:x}", digest.clone().finalize())
                {
                    return Err(Error::Integrity);
                }
                sealed = true;
            }
        }
    }
    if !sealed {
        return Err(Error::Incomplete);
    }
    Ok(episodes)
}
#[cfg(test)]
mod jsonl_contract;

/// Writes a single finalized JSONL dataset without replacing an existing target.
pub fn write_file(
    path: &std::path::Path,
    episodes: &[Episode],
    capacity: usize,
    mode: Backpressure,
) -> Result<(), Error> {
    write_file_typed(path, episodes, capacity, mode)
}
/// Same no-replace publication protocol as v1, with an explicit v2 seal.
pub fn write_file_v2(
    path: &std::path::Path,
    episodes: &[structured::Episode],
    capacity: usize,
    mode: Backpressure,
) -> Result<(), Error> {
    write_file_typed(path, episodes, capacity, mode)
}
fn write_file_typed<E: Durable>(
    path: &std::path::Path,
    episodes: &[E],
    capacity: usize,
    mode: Backpressure,
) -> Result<(), Error> {
    write_file_observed(path, episodes, capacity, mode, &mut |_, _| Ok(()))
}
// Publication fault boundary shared with the existing single-file writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FileStage {
    Create,
    Write,
    Sync,
    Link,
    UnlinkPartial,
    DirectorySync,
    Withdraw,
}
pub(crate) type FileHook<'a> = dyn FnMut(&std::path::Path, FileStage) -> std::io::Result<()> + 'a;
pub(crate) fn write_file_observed<E: Durable>(
    path: &std::path::Path,
    episodes: &[E],
    capacity: usize,
    mode: Backpressure,
    hook: &mut FileHook<'_>,
) -> Result<(), Error> {
    use std::fs::{self, OpenOptions};
    // The sibling fragment and final name are on the same filesystem. A hard
    // link publishes atomically and fails if the final name already exists.
    let mut partial_name = path.as_os_str().to_owned();
    partial_name.push(".partial");
    let partial = std::path::PathBuf::from(partial_name);
    if path.try_exists().map_err(Error::Io)? {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "dataset exists",
        )));
    }
    hook(path, FileStage::Create).map_err(Error::Io)?;
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)
        .map_err(Error::Io)?;
    hook(path, FileStage::Write).map_err(Error::Io)?;
    let mut writer = Writer::new(file, capacity, mode)?;
    writer.format = E::FORMAT;
    for episode in episodes {
        writer.append_record(episode)?;
    }
    let file = writer.finish()?;
    hook(path, FileStage::Sync).map_err(Error::Io)?;
    file.sync_all().map_err(Error::Io)?;
    hook(path, FileStage::Link).map_err(Error::Io)?;
    fs::hard_link(&partial, path).map_err(Error::Io)?;
    hook(path, FileStage::UnlinkPartial).map_err(Error::Io)?;
    fs::remove_file(&partial).map_err(Error::Io)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."));
    hook(path, FileStage::DirectorySync).map_err(Error::Io)?;
    std::fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(Error::Io)?;
    Ok(())
}

// One serializer, queue, integrity reader and publisher; schema-specific checks
// are the only version-dependent behavior.
pub(crate) trait Durable: Serialize + serde::de::DeserializeOwned {
    const FORMAT: u32;
    fn validate_record(&self) -> Result<(), Error>;
    fn metadata(
        &self,
    ) -> (
        &Header,
        &RewardConvention,
        &DiscountConvention,
        Option<&End>,
    );
    fn decision_count(&self) -> usize;
}
impl Durable for Episode {
    const FORMAT: u32 = 1;
    fn validate_record(&self) -> Result<(), Error> {
        validate(self)
    }
    fn metadata(
        &self,
    ) -> (
        &Header,
        &RewardConvention,
        &DiscountConvention,
        Option<&End>,
    ) {
        (
            &self.header,
            &self.reward_convention,
            &self.discount_convention,
            self.footer.as_ref().map(|f| &f.end),
        )
    }
    fn decision_count(&self) -> usize {
        self.decisions.len()
    }
}
impl Durable for structured::Episode {
    const FORMAT: u32 = 2;
    fn validate_record(&self) -> Result<(), Error> {
        structured::validate(self)
    }
    fn metadata(
        &self,
    ) -> (
        &Header,
        &RewardConvention,
        &DiscountConvention,
        Option<&End>,
    ) {
        (
            &self.header,
            &self.reward_convention,
            &self.discount_convention,
            self.footer.as_ref().map(|f| &f.end),
        )
    }
    fn decision_count(&self) -> usize {
        self.decisions.len()
    }
}
/// Copies only canonical owned v2 policy data. Missing statistics stay absent.
pub fn from_core_v2(
    episode: &mtg_core::trajectory::v2::Episode,
) -> Result<structured::Episode, Error> {
    let e = serde_json::from_value(serde_json::to_value(episode).map_err(|_| Error::Invalid)?)
        .map_err(|_| Error::Invalid)?;
    structured::validate(&e)?;
    Ok(e)
}
