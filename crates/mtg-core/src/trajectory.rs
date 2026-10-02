//! Canonical owned in-memory trajectories. See doc/trajectories.md.
use crate::objects::Seat;
use crate::opening::{Game, terminal::EpisodeId, views::PlayerView};
use serde::Serialize;

pub const SCHEMA_VERSION: u32 = 1;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Versions {
    pub schema: u32,
    pub engine: String,
    pub rules: String,
    pub cards: String,
    pub action: String,
    pub observation: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EpisodeKey {
    /// Collector-generated globally unique run UUID; never a seed.
    pub run: String,
    pub ordinal: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Header {
    pub id: EpisodeKey,
    pub versions: Versions,
    pub deck_hashes: [String; 2],
    pub config_hash: String,
    pub policies: [String; 2],
    pub starting_seat: u8,
    pub limits: Limits,
    /// Opaque reference to separately permissioned seeds/replay configuration.
    pub restricted_replay: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Limits {
    pub decisions: Option<u64>,
    pub turns: Option<u64>,
    pub wall_time_ms: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Candidate {
    pub semantic: String,
    pub features: Vec<i64>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PolicyInfo {
    pub checkpoint: Option<String>,
    pub log_probability: Option<f64>,
    pub value: Option<f64>,
    pub recurrent_state: Option<String>,
    pub exploration: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Choice {
    pub kind: String,
    pub logical_action: u64,
    pub micro_choice: u64,
    pub candidates: Vec<Candidate>,
    pub legal_mask: Vec<bool>,
    pub selected: usize,
    pub policy: PolicyInfo,
}
/// Only authorized views, with a nonserialized episode identity guard.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    episode: EpisodeId,
    views: [PlayerView; 2],
}
impl Frame {
    pub fn capture(game: &Game) -> Result<Self, Error> {
        Ok(Self {
            episode: game.episode_id().ok_or(Error::Unavailable)?,
            views: [
                game.observe(Seat::P0).map_err(|_| Error::Unavailable)?,
                game.observe(Seat::P1).map_err(|_| Error::Unavailable)?,
            ],
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum End {
    Completed,
    Truncated(Limit),
    Failed(String),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Limit {
    Decisions,
    Turns,
    WallTime,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Decision {
    pub episode: EpisodeKey,
    pub index: usize,
    pub seat_index: usize,
    pub actor: u8,
    pub observation: PlayerView,
    pub choice: Choice,
    pub action: String,
    pub reward: [i8; 2],
    pub next_actor: Option<u8>,
    pub terminated: bool,
    pub truncated: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Footer {
    pub end: End,
    pub complete: bool,
    pub returns: [i8; 2],
    /// Reward from an ending outside a recorded policy action (e.g. concession).
    pub boundary_reward: [i8; 2],
    pub decisions: usize,
    pub logical_actions: usize,
    pub final_observations: [PlayerView; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum RewardConvention {
    SparseZeroSumTerminal,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum DiscountConvention {
    UndiscountedEpisodic,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum CaptureSelection {
    AllDecisions,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Episode {
    reward_convention: RewardConvention,
    discount_convention: DiscountConvention,
    capture_selection: CaptureSelection,
    header: Header,
    decisions: Vec<Decision>,
    footer: Option<Footer>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Recorder {
    episode: Episode,
    last: Frame,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Unavailable,
    InvalidHeader,
    InvalidChoice,
    Discontinuity,
    AlreadyEnded,
    InvalidEnd,
    Quarantined,
    MissingProbability,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Transition {
    pub episode: EpisodeKey,
    pub seat: u8,
    pub seat_index: usize,
    pub decision_index: usize,
    pub next_decision: Option<usize>,
    pub observation: PlayerView,
    pub next_observation: PlayerView,
    pub choice: Choice,
    pub reward: i8,
    pub decisions_elapsed: usize,
    pub logical_actions_elapsed: usize,
    pub terminated: bool,
    pub truncated: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SeatSequence {
    pub versions: Versions,
    pub discount_convention: DiscountConvention,
    pub episode: EpisodeKey,
    pub seat: u8,
    pub transitions: Vec<Transition>,
    pub final_observation: PlayerView,
    /// Sole reward-bearing record if this seat made no decisions; otherwise zero.
    pub unassigned_reward: i8,
    pub total_return: i8,
    pub end: End,
}
fn seat_number(seat: Seat) -> usize {
    match seat {
        Seat::P0 => 0,
        Seat::P1 => 1,
    }
}
fn rewards(frame: &Frame) -> [i8; 2] {
    match frame.views[0].terminal.as_ref().and_then(|t| t.winner) {
        Some(0) => [1, -1],
        Some(1) => [-1, 1],
        _ => [0, 0],
    }
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
fn action_count(decisions: &[Decision]) -> usize {
    decisions
        .iter()
        .enumerate()
        .filter(|(i, d)| {
            *i == 0 || decisions[*i - 1].choice.logical_action != d.choice.logical_action
        })
        .count()
}
impl Recorder {
    pub fn new(header: &Header, initial: &Frame) -> Result<Self, Error> {
        let v = &header.versions;
        if v.schema != SCHEMA_VERSION
            || v.observation != crate::opening::views::SCHEMA_VERSION
            || [&v.engine, &v.rules, &v.cards, &v.action]
                .iter()
                .any(|s| s.trim().is_empty())
            || !uuid(&header.id.run)
            || !header.deck_hashes.iter().all(|s| hash(s))
            || !hash(&header.config_hash)
            || header.policies.iter().any(|s| s.trim().is_empty())
            || header.starting_seat > 1
            || header.starting_seat != initial.views[0].starting_seat
            || header
                .restricted_replay
                .as_ref()
                .is_some_and(|s| s.trim().is_empty())
        {
            return Err(Error::InvalidHeader);
        }
        Ok(Self {
            episode: Episode {
                reward_convention: RewardConvention::SparseZeroSumTerminal,
                discount_convention: DiscountConvention::UndiscountedEpisodic,
                capture_selection: CaptureSelection::AllDecisions,
                header: header.clone(),
                decisions: vec![],
                footer: None,
            },
            last: initial.clone(),
        })
    }
    /// Producer supplies the exact action-time table and chosen semantic action.
    /// Validation is transactional; it never applies a game action or consumes RNG.
    pub fn append(&mut self, before: &Frame, choice: &Choice, after: &Frame) -> Result<(), Error> {
        if self.episode.footer.is_some() {
            return Err(Error::AlreadyEnded);
        }
        if before != &self.last || before.episode != after.episode {
            return Err(Error::Discontinuity);
        }
        let actor = before.views[0].acting_seat.ok_or(Error::InvalidChoice)?;
        if actor > 1 || before.views[0].terminal.is_some() {
            return Err(Error::InvalidChoice);
        }
        let p = &choice.policy;
        let sequence_ok = match self.episode.decisions.last() {
            None => choice.logical_action == 0 && choice.micro_choice == 0,
            Some(d) => {
                (choice.logical_action == d.choice.logical_action
                    && d.choice.micro_choice.checked_add(1) == Some(choice.micro_choice))
                    || (d.choice.logical_action.checked_add(1) == Some(choice.logical_action)
                        && choice.micro_choice == 0)
            }
        };
        if !sequence_ok
            || choice.kind.trim().is_empty()
            || choice.candidates.is_empty()
            || choice.candidates.len() != choice.legal_mask.len()
            || choice.legal_mask.get(choice.selected) != Some(&true)
            || choice
                .candidates
                .iter()
                .any(|c| c.semantic.trim().is_empty())
            || p.log_probability.is_some_and(|x| !x.is_finite() || x > 0.0)
            || p.value.is_some_and(|x| !x.is_finite())
        {
            return Err(Error::InvalidChoice);
        }
        let terminated = after.views[0].terminal.is_some();
        let reward = if terminated { rewards(after) } else { [0, 0] };
        self.episode.decisions.push(Decision {
            episode: self.episode.header.id.clone(),
            index: self.episode.decisions.len(),
            seat_index: self
                .episode
                .decisions
                .iter()
                .filter(|d| d.actor == actor)
                .count(),
            actor,
            observation: before.views[actor as usize].clone(),
            choice: choice.clone(),
            action: choice.candidates[choice.selected].semantic.clone(),
            reward,
            next_actor: after.views[0].acting_seat,
            terminated,
            truncated: false,
        });
        self.last = after.clone();
        if terminated {
            self.close(after, End::Completed, [0, 0]);
        }
        Ok(())
    }
    /// End outside a recorded action. Completed requires a real rules outcome;
    /// truncation requires a live, unchanged boundary. Failure is quarantined.
    pub fn finish(&mut self, final_frame: &Frame, end: End) -> Result<(), Error> {
        if self.episode.footer.is_some() {
            return Err(Error::AlreadyEnded);
        }
        if self.last.episode != final_frame.episode {
            return Err(Error::Discontinuity);
        }
        let terminal = final_frame.views[0].terminal.is_some();
        match &end {
            End::Completed if !terminal => return Err(Error::InvalidEnd),
            End::Truncated(_) if terminal || final_frame != &self.last => {
                return Err(Error::InvalidEnd);
            }
            End::Failed(reason) if reason.trim().is_empty() => return Err(Error::InvalidEnd),
            _ => (),
        }
        let boundary_reward = if matches!(end, End::Completed) {
            rewards(final_frame)
        } else {
            [0, 0]
        };
        self.close(final_frame, end, boundary_reward);
        Ok(())
    }
    fn close(&mut self, frame: &Frame, end: End, boundary_reward: [i8; 2]) {
        // A stop after the last action is a boundary on that global transition.
        if let Some(last) = self.episode.decisions.last_mut() {
            last.truncated = matches!(end, End::Truncated(_));
            last.terminated = matches!(end, End::Completed);
            last.next_actor = None;
        }
        self.episode.footer = Some(Footer {
            returns: if matches!(end, End::Completed) {
                rewards(frame)
            } else {
                [0, 0]
            },
            complete: !matches!(end, End::Failed(_)),
            end,
            boundary_reward,
            decisions: self.episode.decisions.len(),
            logical_actions: action_count(&self.episode.decisions),
            final_observations: frame.views.clone(),
        });
        self.last = frame.clone();
    }
    pub fn episode(&self) -> &Episode {
        &self.episode
    }
}
impl Episode {
    /// Immutable serialized convention; storage adapters must preserve it.
    pub fn reward_convention(&self) -> &RewardConvention {
        &self.reward_convention
    }
    pub fn discount_convention(&self) -> &DiscountConvention {
        &self.discount_convention
    }
    pub fn capture_selection(&self) -> &CaptureSelection {
        &self.capture_selection
    }
    pub fn header(&self) -> &Header {
        &self.header
    }
    pub fn decisions(&self) -> &[Decision] {
        &self.decisions
    }
    /// Diagnostic access is explicit; a footer alone is not training eligibility.
    pub fn footer(&self) -> Option<&Footer> {
        self.footer.as_ref()
    }
    pub fn seat(&self, seat: Seat) -> Result<SeatSequence, Error> {
        let f = self
            .footer
            .as_ref()
            .filter(|f| f.complete && !matches!(f.end, End::Failed(_)))
            .ok_or(Error::Quarantined)?;
        let s = seat_number(seat);
        let rows: Vec<_> = self
            .decisions
            .iter()
            .filter(|d| d.actor as usize == s)
            .collect();
        let transitions = rows
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let next = rows.get(i + 1);
                let end = next.map_or(self.decisions.len(), |d| d.index);
                let interval = &self.decisions[d.index..end];
                Transition {
                    episode: self.header.id.clone(),
                    seat: s as u8,
                    seat_index: i,
                    decision_index: d.index,
                    next_decision: next.map(|d| d.index),
                    observation: d.observation.clone(),
                    next_observation: next.map_or_else(
                        || f.final_observations[s].clone(),
                        |d| d.observation.clone(),
                    ),
                    choice: d.choice.clone(),
                    reward: interval.iter().map(|d| d.reward[s]).sum::<i8>()
                        + if next.is_none() {
                            f.boundary_reward[s]
                        } else {
                            0
                        },
                    decisions_elapsed: interval.len(),
                    logical_actions_elapsed: action_count(interval),
                    terminated: next.is_none() && matches!(f.end, End::Completed),
                    truncated: next.is_none() && matches!(f.end, End::Truncated(_)),
                }
            })
            .collect();
        Ok(SeatSequence {
            versions: self.header.versions.clone(),
            discount_convention: self.discount_convention.clone(),
            episode: self.header.id.clone(),
            seat: s as u8,
            transitions,
            final_observation: f.final_observations[s].clone(),
            unassigned_reward: if rows.is_empty() { f.returns[s] } else { 0 },
            total_return: f.returns[s],
            end: f.end.clone(),
        })
    }
    pub fn require_probabilities(&self) -> Result<(), Error> {
        if self.footer.as_ref().is_none_or(|f| !f.complete) {
            return Err(Error::Quarantined);
        }
        if self
            .decisions
            .iter()
            .any(|d| d.choice.policy.log_probability.is_none())
        {
            return Err(Error::MissingProbability);
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "trajectory_tests.rs"]
mod tests;

pub mod v2;
