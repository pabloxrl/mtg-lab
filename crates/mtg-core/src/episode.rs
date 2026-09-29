//! Owned scalar execution; privileged history is separate from policy views.
use crate::trajectory::{self, Header, PolicyInfo, v2};
use crate::{
    game::{self, Config, Game, Progress as CoreProgress, actions, policy, terminal},
    objects::{Seat, StorageError},
};
use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidBudget,
    Stopped,
    RecordCapacity,
    Storage(StorageError),
    Capture(trajectory::Error),
    RevisionExhausted,
    NotStarted,
    Finalized,
    ActiveEpisode,
    Ended,
    Reset(game::ResetError),
    Action(actions::ActionError),
    Policy(policy::PolicyError),
    Concede(terminal::ConcedeError),
    Turn(game::turns::TurnError),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Completed(terminal::Outcome),
    Incomplete,
    Truncated(trajectory::Limit),
    Failed(Failure),
}
/// External monotonic milliseconds; the rules core never reads a wall clock.
pub trait Clock: std::fmt::Debug {
    fn now_ms(&self) -> u64;
}
#[derive(Clone, Debug)]
pub struct Budget {
    pub limits: trajectory::Limits,
    pub work_quantum: NonZeroUsize,
    pub records: NonZeroUsize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Recording,
    RecordCapacity,
    Capacity,
    Clock,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Accounting {
    pub started: u64,
    pub completed: u64,
    pub truncated: u64,
    pub failed: u64,
    pub incomplete: u64,
}
/// A ready policy boundary includes opening, payment, targets and combat choices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    InternalYield,
    Ready,
    Terminal(terminal::Outcome),
    Stopped(Status),
}
#[derive(Clone, Debug)]
pub struct Inputs {
    pub config: Config,
    pub master: u64,
    pub ordinal: u64,
}
#[derive(Clone, Debug)]
pub struct EpisodeResult {
    inputs: Inputs,
    status: Status,
    snapshot: Vec<u8>,
    history: Vec<Vec<u8>>,
    trajectory: Option<v2::Episode>,
    capture_requested: bool,
    final_observations: Option<[policy::Observation; 2]>,
    budget: Option<Budget>,
    accepted_decisions: u64,
}
impl EpisodeResult {
    pub fn budget(&self) -> Option<&Budget> {
        self.budget.as_ref()
    }
    pub fn accepted_decisions(&self) -> u64 {
        self.accepted_decisions
    }
    pub fn final_observations(&self) -> Option<&[policy::Observation; 2]> {
        self.final_observations.as_ref()
    }
    pub fn trajectory(&self) -> Option<&v2::Episode> {
        self.trajectory.as_ref()
    }
    pub fn capture_requested(&self) -> bool {
        self.capture_requested
    }
    pub fn privileged_inputs(&self) -> &Inputs {
        &self.inputs
    }
    pub fn status(&self) -> Status {
        self.status
    }
    pub fn privileged_snapshot(&self) -> &[u8] {
        &self.snapshot
    }
    pub fn privileged_history(&self) -> &[Vec<u8>] {
        &self.history
    }
}
/// Sole owner of a game. No mutable game, restore, or unrecorded apply API escapes.
///
/// ```compile_fail
/// use mtg_core::episode::Driver;
/// let mut driver = Driver::new(256).unwrap();
/// let game = &mut driver.game; // Private: actions cannot bypass history.
/// ```
#[derive(Debug)]
pub struct Driver {
    game: Game,
    capacity: usize,
    inputs: Option<Inputs>,
    history: Vec<Vec<u8>>,
    finalized: bool,
    revision: u64,
    progress: CoreProgress,
    capture_header: Option<Header>,
    recorder: Option<v2::Recorder>,
    capture_error: Option<trajectory::Error>,
    budget: Option<Budget>,
    clock: Option<Box<dyn Clock>>,
    started_ms: u64,
    last_ms: u64,
    decisions: u64,
    status: Option<Status>,
    accounting: Accounting,
}
impl Driver {
    pub fn bounded(capacity: usize, budget: Budget, clock: Box<dyn Clock>) -> Result<Self, Error> {
        if capacity == 0
            || [
                budget.limits.decisions,
                budget.limits.turns,
                budget.limits.wall_time_ms,
            ]
            .contains(&Some(0))
        {
            return Err(Error::InvalidBudget);
        }
        let mut driver = Self::new(capacity).map_err(Error::Storage)?;
        driver.budget = Some(budget);
        driver.clock = Some(clock);
        Ok(driver)
    }
    pub fn accounting(&self) -> Accounting {
        self.accounting
    }
    pub fn status(&self) -> Option<Status> {
        self.status
    }
    fn quantum(&self, requested: NonZeroUsize) -> NonZeroUsize {
        self.budget
            .as_ref()
            .map_or(requested, |b| requested.min(b.work_quantum))
    }
    fn account(&mut self, status: Status) {
        if self.status.is_some() {
            return;
        }
        self.status = Some(status);
        match status {
            Status::Completed(_) => self.accounting.completed += 1,
            Status::Truncated(_) => self.accounting.truncated += 1,
            Status::Failed(_) => self.accounting.failed += 1,
            Status::Incomplete => self.accounting.incomplete += 1,
        }
    }
    fn fail(&mut self, failure: Failure) {
        if let Some(recorder) = &mut self.recorder {
            recorder.quarantine(format!("owned collector: {failure:?}"));
        }
        self.account(Status::Failed(failure));
    }
    // Recording errors outrank rules completion. No failed capture is exposed as
    // a training sample, even if its underlying game reached a terminal state.
    fn capture_checked(&mut self, result: Result<(), Error>) -> Result<(), Error> {
        if let Err(Error::Capture(e)) = &result {
            self.capture_error = Some(e.clone());
            self.fail(Failure::Recording);
        }
        result
    }
    fn check_boundary(&mut self) -> Result<(), Error> {
        if self.status.is_some() {
            return Ok(());
        }
        if let Some(outcome) = self.game.outcome() {
            self.account(Status::Completed(outcome));
            return Ok(());
        }
        if self.budget.is_some() && self.progress != CoreProgress::InternalYield {
            for seat in [Seat::P0, Seat::P1] {
                if self.game.policy_observe(seat, self.capacity)
                    == Err(policy::PolicyError::CapacityExceeded)
                {
                    self.fail(Failure::Capacity);
                    return Err(Error::Policy(policy::PolicyError::CapacityExceeded));
                }
            }
        }
        let Some(b) = &self.budget else {
            return Ok(());
        };
        let now = self.clock.as_ref().unwrap().now_ms();
        if now < self.last_ms {
            self.fail(Failure::Clock);
            return Err(Error::Stopped);
        }
        self.last_ms = now;
        let limit = if b.limits.decisions.is_some_and(|n| self.decisions >= n) {
            Some(trajectory::Limit::Decisions)
        } else if b.limits.turns.is_some_and(|n| {
            self.game
                .turn_position()
                .is_some_and(|(turn, _, _)| turn > n)
        }) {
            Some(trajectory::Limit::Turns)
        } else if b
            .limits
            .wall_time_ms
            .is_some_and(|n| now - self.started_ms >= n)
        {
            Some(trajectory::Limit::WallTime)
        } else {
            None
        };
        if let Some(limit) = limit {
            // Partial reset has no authorized observation yet. Keep its actual
            // snapshot, without advancing to manufacture a final frame.
            if self.recorder.is_some() && self.progress != CoreProgress::InternalYield {
                let result = (|| {
                    let frame = self.frame()?;
                    self.recorder
                        .as_mut()
                        .unwrap()
                        .finish(&frame, trajectory::End::Truncated(limit))
                        .map_err(Error::Capture)
                })();
                self.capture_checked(result)?;
            }
            self.account(Status::Truncated(limit));
        }
        Ok(())
    }
    fn before_input(&mut self) -> Result<(), Error> {
        self.active()?;
        self.check_boundary()?;
        self.active()
    }
    fn record_slot(&mut self) -> Result<(), Error> {
        if self
            .budget
            .as_ref()
            .is_some_and(|b| self.history.len() >= b.records.get())
        {
            self.fail(Failure::RecordCapacity);
            return Err(Error::RecordCapacity);
        }
        Ok(())
    }
    pub fn new(capacity: usize) -> Result<Self, StorageError> {
        Ok(Self {
            game: Game::new()?,
            capacity,
            inputs: None,
            history: vec![],
            finalized: false,
            revision: 0,
            progress: CoreProgress::NotStarted,
            capture_header: None,
            recorder: None,
            capture_error: None,
            budget: None,
            clock: None,
            started_ms: 0,
            last_ms: 0,
            decisions: 0,
            status: None,
            accounting: Accounting::default(),
        })
    }
    /// Enable complete in-memory capture for this reset. Header provenance and
    /// policy identity are caller declarations; run provenance is a separate layer.
    pub fn reset_captured(
        &mut self,
        config: &Config,
        master: u64,
        ordinal: u64,
        quantum: NonZeroUsize,
        header: &Header,
    ) -> Result<Progress, Error> {
        v2::Recorder::validate_header(header, config.starting_seat).map_err(Error::Capture)?;
        if header.id.ordinal != ordinal {
            return Err(Error::Capture(trajectory::Error::InvalidHeader));
        }
        if self
            .budget
            .as_ref()
            .is_some_and(|b| b.limits != header.limits)
        {
            return Err(Error::InvalidBudget);
        }
        self.reset_owned(config, master, ordinal, quantum, Some(header.clone()))
    }
    /// Immutable diagnostic view; unfinished episodes have no training reader.
    pub fn trajectory(&self) -> Option<&v2::Episode> {
        self.recorder.as_ref().map(v2::Recorder::episode)
    }
    fn frame(&self) -> Result<v2::Frame, Error> {
        v2::Frame::owned(&self.game, self.revision).map_err(Error::Capture)
    }
    fn refresh_capture(&mut self) -> Result<(), Error> {
        if self.capture_header.is_none() || self.progress == CoreProgress::InternalYield {
            return Ok(());
        }
        let frame = self.frame()?;
        match &mut self.recorder {
            Some(r) => r.settle(&frame).map_err(Error::Capture)?,
            None => {
                self.recorder = Some(
                    v2::Recorder::new(self.capture_header.as_ref().unwrap(), &frame)
                        .map_err(Error::Capture)?,
                )
            }
        }
        Ok(())
    }
    pub fn submit_with_policy(
        &mut self,
        seat: Seat,
        submission: &policy::Submission,
        info: &PolicyInfo,
    ) -> Result<(), Error> {
        self.before_input()?;
        // Pending core work has no policy frame. This is a rejected request,
        // not a recorder fault; preserve the resumable episode in both modes.
        if self.progress == CoreProgress::InternalYield {
            return Err(Error::Policy(policy::PolicyError::Unavailable));
        }
        if info
            .log_probability
            .is_some_and(|x| !x.is_finite() || x > 0.0)
            || info.value.is_some_and(|x| !x.is_finite())
        {
            return Err(Error::Capture(trajectory::Error::InvalidChoice));
        }
        let before = if self.capture_header.is_some() {
            match self.frame() {
                Ok(frame) => Some(frame),
                Err(error) => return self.capture_checked(Err(error)),
            }
        } else {
            None
        };
        let accepted = self.submit_core(seat, submission);
        if self.budget.is_some()
            && matches!(
                accepted,
                Err(Error::Policy(policy::PolicyError::CapacityExceeded))
            )
        {
            self.fail(Failure::Capacity);
        }
        accepted?;
        self.decisions += 1;
        if let Some(before) = before {
            // Core acceptance is the execution receipt, not the recorder's domain
            // validator. Preserve an explicit poisoned owner if an invariant fails.
            let captured = (|| {
                let after = self.frame()?;
                let recorder = self
                    .recorder
                    .as_mut()
                    .ok_or(Error::Capture(trajectory::Error::Unavailable))?;
                let (logical_action, micro_choice) =
                    recorder.episode().decisions().last().map_or((0, 0), |d| {
                        if d.choice.status == v2::ActionStatus::Continuing {
                            (d.choice.logical_action, d.choice.micro_choice + 1)
                        } else {
                            (d.choice.logical_action + 1, 0)
                        }
                    });
                let choice = v2::Choice {
                    submission: submission.clone(),
                    logical_action,
                    micro_choice,
                    status: after.status_after(submission),
                    policy: info.clone(),
                };
                recorder
                    .append(&before, &choice, &after)
                    .map_err(Error::Capture)
            })();
            self.capture_checked(captured)?;
        }
        self.check_boundary()?;
        Ok(())
    }
    /// Start a fresh normal-reset game. Finish the previous episode first, even
    /// if incomplete. Invalid configuration preserves the entire previous episode.
    pub fn reset(
        &mut self,
        config: &Config,
        master: u64,
        ordinal: u64,
        quantum: NonZeroUsize,
    ) -> Result<Progress, Error> {
        self.reset_owned(config, master, ordinal, quantum, None)
    }
    fn reset_owned(
        &mut self,
        config: &Config,
        master: u64,
        ordinal: u64,
        quantum: NonZeroUsize,
        header: Option<Header>,
    ) -> Result<Progress, Error> {
        if self.inputs.is_some() && !self.finalized {
            return Err(Error::ActiveEpisode);
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(Error::RevisionExhausted)?;
        // A new owner also permits explicit abandonment during an internal yield;
        // the saved incomplete result keeps the exact unfinished state.
        let mut game = Game::new().map_err(Error::Storage)?;
        let started_ms = self.clock.as_ref().map_or(0, |c| c.now_ms());
        let progress = game
            .reset_quantum(config, master, ordinal, self.quantum(quantum))
            .map_err(Error::Reset)?;
        self.game = game;
        self.inputs = Some(Inputs {
            config: config.clone(),
            master,
            ordinal,
        });
        self.history.clear();
        self.capture_header = header;
        self.recorder = None;
        self.capture_error = None;
        self.finalized = false;
        self.revision = revision;
        self.progress = progress;
        self.started_ms = started_ms;
        self.last_ms = started_ms;
        self.decisions = 0;
        self.status = None;
        self.accounting.started += 1;
        // Capture must exist before a reset-time budget stop is finalized.
        let capture = self.refresh_capture();
        self.capture_checked(capture)?;
        self.check_boundary()?;
        Ok(self.boundary())
    }
    fn boundary(&self) -> Progress {
        if let Some(status @ (Status::Truncated(_) | Status::Failed(_))) = self.status {
            return Progress::Stopped(status);
        }
        match self.progress {
            CoreProgress::Terminal(outcome) => Progress::Terminal(outcome),
            CoreProgress::InternalYield => Progress::InternalYield,
            CoreProgress::OpeningComplete if self.game.turn_position().is_none() => {
                Progress::InternalYield
            }
            _ => Progress::Ready,
        }
    }
    fn active(&self) -> Result<(), Error> {
        if self.inputs.is_none() {
            return Err(Error::NotStarted);
        }
        if self.finalized {
            return Err(Error::Finalized);
        }
        if matches!(self.status, Some(Status::Truncated(_) | Status::Failed(_))) {
            return Err(Error::Stopped);
        }
        if let Some(error) = &self.capture_error {
            return Err(Error::Capture(error.clone()));
        }
        if self.game.outcome().is_some() {
            return Err(Error::Ended);
        }
        Ok(())
    }
    /// Perform at most one core work quantum. OpeningComplete is an internal
    /// boundary; the next advance starts turns. Submission itself is synchronous.
    pub fn advance(&mut self, quantum: NonZeroUsize) -> Result<Progress, Error> {
        self.active()?;
        self.check_boundary()?;
        if self.status.is_some() {
            return Ok(self.boundary());
        }
        let quantum = self.quantum(quantum);
        self.progress = if self.progress == CoreProgress::OpeningComplete
            && self.game.turn_position().is_none()
        {
            match self.game.start_turns_quantum(quantum) {
                Ok(progress) => progress,
                Err(error) => {
                    if self.budget.is_some() {
                        self.fail(Failure::Capacity);
                    }
                    return Err(Error::Turn(error));
                }
            }
        } else {
            self.game.resume(quantum)
        };
        let result = self.refresh_capture();
        self.capture_checked(result)?;
        self.check_boundary()?;
        Ok(self.boundary())
    }
    /// Caller supplies its authorized seat; transport authentication is external.
    /// Revisions are driver-local and reject delayed submissions across reset.
    pub fn observe(&self, seat: Seat) -> Result<policy::Observation, Error> {
        let mut observation = self
            .game
            .policy_observe(seat, self.capacity)
            .map_err(Error::Policy)?;
        if let Some(d) = &mut observation.decision {
            d.revision = self.revision;
        }
        Ok(observation)
    }
    /// Validate and encode before mutation; append exactly once after acceptance.
    pub fn submit(&mut self, seat: Seat, submission: &policy::Submission) -> Result<(), Error> {
        self.submit_with_policy(seat, submission, &PolicyInfo::default())
    }
    fn submit_core(&mut self, seat: Seat, submission: &policy::Submission) -> Result<(), Error> {
        self.active()?;
        if submission.schema_version != policy::SCHEMA_VERSION {
            return Err(Error::Policy(policy::PolicyError::UnsupportedVersion));
        }
        if submission.revision != self.revision {
            return Err(Error::Policy(policy::PolicyError::StaleDecision));
        }
        let d = self
            .game
            .policy_observe(seat, self.capacity)
            .map_err(Error::Policy)?
            .decision
            .ok_or(Error::Policy(policy::PolicyError::WrongActor))?;
        let mut core = submission.clone();
        core.revision = d.revision;
        let record =
            actions::encode(&self.game, seat, &core, self.capacity).map_err(Error::Action)?;
        self.record_slot()?;
        self.game
            .apply_policy(seat, &core, self.capacity)
            .map_err(Error::Policy)?;
        self.history.push(record);
        // apply_policy drains its work synchronously; this only inspects the boundary.
        self.progress = self.game.resume(NonZeroUsize::MIN);
        Ok(())
    }
    pub fn concede(&mut self, seat: Seat, episode: terminal::EpisodeId) -> Result<(), Error> {
        self.before_input()?;
        if self.game.episode_id() != Some(episode) {
            return Err(Error::Concede(terminal::ConcedeError::StaleEpisode));
        }
        // Semantic history starts from a fully executed normal reset. A terminal
        // result during a yield could not be reproduced by that action stream.
        if self.boundary() == Progress::InternalYield {
            return Err(Error::Concede(terminal::ConcedeError::SettlementPending));
        }
        let record = actions::encode_concession(&self.game, seat).map_err(Error::Action)?;
        self.record_slot()?;
        let outcome = self.game.concede(seat, episode).map_err(Error::Concede)?;
        self.history.push(record);
        self.progress = CoreProgress::Terminal(outcome);
        let captured = (|| {
            if self.capture_header.is_some() {
                let frame = self.frame()?;
                self.recorder
                    .as_mut()
                    .ok_or(Error::Capture(trajectory::Error::Unavailable))?
                    .finish(&frame, trajectory::End::Completed)
                    .map_err(Error::Capture)?;
            }
            Ok(())
        })();
        self.capture_checked(captured)?;
        self.check_boundary()?;
        Ok(())
    }
    pub fn episode_id(&self) -> Option<terminal::EpisodeId> {
        self.game.episode_id()
    }
    /// Full private state and RNG; never a policy input.
    pub fn privileged_snapshot(&self) -> Vec<u8> {
        self.game.snapshot()
    }
    /// Privileged semantic records, including private opening choices.
    pub fn privileged_history(&self) -> &[Vec<u8>] {
        &self.history
    }
    /// Seal exactly once without inventing a rules result for unfinished work.
    /// The returned buffers and actual reset inputs survive every subsequent reset.
    pub fn finish(&mut self) -> Result<EpisodeResult, Error> {
        let inputs = self.inputs.clone().ok_or(Error::NotStarted)?;
        if self.finalized {
            return Err(Error::Finalized);
        }
        self.check_boundary()?;
        if self.status.is_none() {
            self.account(Status::Incomplete);
        }
        let final_observations = match (
            self.game.policy_observe(Seat::P0, usize::MAX),
            self.game.policy_observe(Seat::P1, usize::MAX),
        ) {
            (Ok(mut a), Ok(mut b)) => {
                for o in [&mut a, &mut b] {
                    if let Some(d) = &mut o.decision {
                        d.revision = self.revision;
                    }
                }
                Some([a, b])
            }
            _ => None,
        };
        let result = EpisodeResult {
            inputs,
            status: self.status.unwrap(),
            budget: self.budget.clone(),
            accepted_decisions: self.decisions,
            final_observations,
            snapshot: self.game.snapshot(),
            history: self.history.clone(),
            trajectory: self.trajectory().cloned(),
            capture_requested: self.capture_header.is_some(),
        };
        self.finalized = true;
        Ok(result)
    }
}

// This constructor exists only in unit-test binaries to exercise unreachable
// component-edge positions. Production capture can start only at normal reset.
#[cfg(test)]
impl Driver {
    pub(crate) fn synthetic_capture_test(mut game: Game, header: Header) -> Self {
        let progress = game.resume(NonZeroUsize::MIN);
        let mut d = Self {
            game,
            capacity: 256,
            inputs: Some(Inputs {
                config: Config::default(),
                master: 0,
                ordinal: 0,
            }),
            history: vec![],
            finalized: false,
            revision: 1,
            progress,
            capture_header: Some(header),
            recorder: None,
            capture_error: None,
            budget: None,
            clock: None,
            started_ms: 0,
            last_ms: 0,
            decisions: 0,
            status: None,
            accounting: Accounting::default(),
        };
        d.refresh_capture().unwrap();
        d
    }
}

#[cfg(test)]
#[path = "episode_budget_tests.rs"]
mod budget_tests;
