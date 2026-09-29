//! Canonical structured played-policy trajectories. See doc/trajectories.md.
use super::{
    CaptureSelection, DiscountConvention, End, EpisodeKey, Error, Header, PolicyInfo,
    RewardConvention, Versions, hash, uuid,
};
use crate::game::policy::{self, Choice as Command, Observation, Submission};
use crate::objects::Seat;
use crate::opening::{Game, terminal::EpisodeId};
use serde::Serialize;

pub const SCHEMA_VERSION: u32 = 2;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ActionStatus {
    Continuing,
    Committed,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Choice {
    pub submission: Submission,
    pub logical_action: u64,
    pub micro_choice: u64,
    pub status: ActionStatus,
    pub policy: PolicyInfo,
}
/// Only authorized views, with a nonserialized episode identity guard.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    episode: EpisodeId,
    views: [Observation; 2],
}
impl Frame {
    pub(crate) fn owned(game: &Game, revision: u64) -> Result<Self, Error> {
        // Capacity governs accepted input. A resulting decision may exceed it;
        // retaining its full domain must not silently truncate or lose the action.
        let mut frame = Self::capture(game, usize::MAX)?;
        for view in &mut frame.views {
            if let Some(d) = &mut view.decision {
                d.revision = revision;
            }
        }
        Ok(frame)
    }
    pub(crate) fn status_after(&self, submission: &Submission) -> ActionStatus {
        match submission.choices.first() {
            Some(Command::CancelPayment | Command::CancelTargets) => ActionStatus::Cancelled,
            _ if self.views.iter().any(|v| v.pending.is_some()) => ActionStatus::Continuing,
            Some(
                Command::SelectAttackers { .. }
                | Command::SelectBlockers { .. }
                | Command::AssignDamage { .. },
            ) if self
                .views
                .iter()
                .any(|v| v.decision.as_ref().is_some_and(|d| d.factored.is_some())) =>
            {
                ActionStatus::Continuing
            }
            _ => ActionStatus::Committed,
        }
    }
    pub fn capture(game: &Game, capacity: usize) -> Result<Self, Error> {
        Ok(Self {
            episode: game.episode_id().ok_or(Error::Unavailable)?,
            views: [
                game.policy_observe(Seat::P0, capacity)
                    .map_err(|_| Error::Unavailable)?,
                game.policy_observe(Seat::P1, capacity)
                    .map_err(|_| Error::Unavailable)?,
            ],
        })
    }
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Decision {
    pub episode: EpisodeKey,
    pub index: usize,
    pub seat_index: usize,
    pub actor: u8,
    pub observation: Observation,
    pub choice: Choice,
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
    pub cancelled_actions: usize,
    pub final_observations: [Observation; 2],
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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Transition {
    pub episode: EpisodeKey,
    pub seat: u8,
    pub seat_index: usize,
    pub decision_index: usize,
    pub next_decision: Option<usize>,
    pub observation: Observation,
    pub next_observation: Observation,
    pub choice: Choice,
    pub reward: i8,
    pub decisions_elapsed: usize,
    pub logical_actions_elapsed: usize,
    pub cancelled_actions_elapsed: usize,
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
    pub final_observation: Observation,
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
    match frame.views[0].view.terminal.as_ref().and_then(|t| t.winner) {
        Some(0) => [1, -1],
        Some(1) => [-1, 1],
        _ => [0, 0],
    }
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
    pub(crate) fn validate_header(header: &Header, starting_seat: u8) -> Result<(), Error> {
        let v = &header.versions;
        if v.schema != SCHEMA_VERSION
            || v.observation != policy::SCHEMA_VERSION
            || [&v.engine, &v.rules, &v.cards, &v.action]
                .iter()
                .any(|s| s.trim().is_empty())
            || !uuid(&header.id.run)
            || !header.deck_hashes.iter().all(|s| hash(s))
            || !hash(&header.config_hash)
            || header.policies.iter().any(|s| s.trim().is_empty())
            || header.starting_seat > 1
            || header.starting_seat != starting_seat
            || header
                .restricted_replay
                .as_ref()
                .is_some_and(|s| s.trim().is_empty())
        {
            return Err(Error::InvalidHeader);
        }
        Ok(())
    }
    // Only the owned driver may cross a real internal-work boundary. This is
    // deliberately not a public way for a producer to omit policy decisions.
    pub(crate) fn settle(&mut self, frame: &Frame) -> Result<(), Error> {
        if self.episode.footer.is_some() {
            return Err(Error::AlreadyEnded);
        }
        if self.last.episode != frame.episode
            || self.last.views.iter().any(|v| v.decision.is_some())
        {
            return if &self.last == frame {
                Ok(())
            } else {
                Err(Error::Discontinuity)
            };
        }
        self.last = frame.clone();
        if let Some(last) = self.episode.decisions.last_mut() {
            last.next_actor = frame.views[0].view.acting_seat;
        }
        Ok(())
    }
    pub fn new(header: &Header, initial: &Frame) -> Result<Self, Error> {
        Self::validate_header(header, initial.views[0].view.starting_seat)?;
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
    /// Retain the captured policy domain and complete accepted semantic submission.
    /// Validation is transactional; it never applies a game action or consumes RNG.
    pub fn append(&mut self, before: &Frame, choice: &Choice, after: &Frame) -> Result<(), Error> {
        if self.episode.footer.is_some() {
            return Err(Error::AlreadyEnded);
        }
        if before != &self.last || before.episode != after.episode {
            return Err(Error::Discontinuity);
        }
        let actor = before.views[0]
            .view
            .acting_seat
            .ok_or(Error::InvalidChoice)?;
        if actor > 1 || before.views[0].view.terminal.is_some() {
            return Err(Error::InvalidChoice);
        }
        let p = &choice.policy;
        let sequence_ok = match self.episode.decisions.last() {
            None => choice.logical_action == 0 && choice.micro_choice == 0,
            Some(d) => match d.choice.status {
                ActionStatus::Continuing => {
                    d.actor == actor
                        && choice.logical_action == d.choice.logical_action
                        && d.choice.micro_choice.checked_add(1) == Some(choice.micro_choice)
                }
                _ => {
                    d.choice.logical_action.checked_add(1) == Some(choice.logical_action)
                        && choice.micro_choice == 0
                }
            },
        };
        let observation = &before.views[actor as usize];
        if !sequence_ok
            || !valid_submission(observation, &choice.submission)
            || choice.status != action_status(observation, &choice.submission)
            || before == after
            || p.log_probability.is_some_and(|x| !x.is_finite() || x > 0.0)
            || p.value.is_some_and(|x| !x.is_finite())
        {
            return Err(Error::InvalidChoice);
        }
        if let Some(next) = after.views.iter().find_map(|v| v.decision.as_ref()) {
            let previous = observation.decision.as_ref().ok_or(Error::InvalidChoice)?;
            if (next.revision, next.generation) <= (previous.revision, previous.generation) {
                return Err(Error::Discontinuity);
            }
        }
        let terminated = after.views[0].view.terminal.is_some();
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
            reward,
            next_actor: after.views[0].view.acting_seat,
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
        let terminal = final_frame.views[0].view.terminal.is_some();
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
            cancelled_actions: cancellation_count(&self.episode.decisions),
            final_observations: frame.views.clone(),
        });
        self.last = frame.clone();
    }
    /// An owning collector's capture failure invalidates the whole sample,
    /// including a terminal footer. Retain all captured rows for diagnostics.
    pub(crate) fn quarantine(&mut self, reason: String) {
        self.close(&self.last.clone(), End::Failed(reason), [0, 0]);
    }
    pub fn episode(&self) -> &Episode {
        &self.episode
    }
}
impl Episode {
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
                    cancelled_actions_elapsed: cancellation_count(interval),
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

fn cancellation_count(decisions: &[Decision]) -> usize {
    decisions
        .iter()
        .filter(|d| d.choice.status == ActionStatus::Cancelled)
        .count()
}
fn action_status(o: &Observation, s: &Submission) -> ActionStatus {
    match s.choices.first() {
        Some(Command::CancelPayment | Command::CancelTargets) => ActionStatus::Cancelled,
        Some(
            Command::Cast { .. }
            | Command::Target { .. }
            | Command::FinishTargets
            | Command::Pay { .. }
            | Command::SelectAttackers { .. }
            | Command::SelectBlockers { .. }
            | Command::AssignDamage { .. },
        ) => ActionStatus::Continuing,
        Some(Command::TapMana { .. }) if o.pending.is_some() => ActionStatus::Continuing,
        _ => ActionStatus::Committed,
    }
}
fn unique<T: PartialEq>(items: &[T]) -> bool {
    items
        .iter()
        .enumerate()
        .all(|(i, x)| !items[..i].contains(x))
}
/// Validate only the authorized input domain. The producer must submit to the
/// engine first; this storage API neither drives nor replays a game.
fn valid_submission(o: &Observation, s: &Submission) -> bool {
    let Some(d) = &o.decision else {
        return false;
    };
    if s.schema_version != policy::SCHEMA_VERSION
        || s.revision != d.revision
        || s.generation != d.generation
        || s.choices.len() != d.count
    {
        return false;
    }
    if let (Some(f), [command]) = (&d.factored, s.choices.as_slice()) {
        match command {
            Command::SelectAttackers { cards } if d.kind == "attackers" => {
                return unique(cards) && cards.iter().all(|r| f.attackers.contains(r));
            }
            Command::SelectBlockers { blocks } if d.kind == "blockers" => {
                return unique(&blocks.iter().map(|(b, _)| b).collect::<Vec<_>>())
                    && blocks
                        .iter()
                        .all(|(b, a)| f.blockers.contains(b) && f.attackers.contains(a));
            }
            Command::AssignDamage { attacker, amounts } if d.kind == "combat_damage" => {
                let Some(a) = f.damage.iter().find(|a| a.attacker == *attacker) else {
                    return false;
                };
                return unique(&amounts.iter().map(|(b, _)| b).collect::<Vec<_>>())
                    && amounts.iter().all(|(b, _)| a.blockers.contains(b))
                    && amounts
                        .iter()
                        .try_fold(0u32, |sum, (_, n)| sum.checked_add(*n))
                        == Some(a.power);
            }
            _ => (),
        }
    }
    unique(&s.choices)
        && s.choices.iter().all(|c| {
            d.candidates
                .iter()
                .enumerate()
                .any(|(i, candidate)| candidate == c && d.legal_mask.get(i) == Some(&true))
        })
}
