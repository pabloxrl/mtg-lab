//! Owned scalar execution; privileged history is separate from policy views.
use crate::trajectory::{self, Header, PolicyInfo, v2};
use crate::{
    game::{self, Config, Game, Progress as CoreProgress, actions, policy, terminal},
    objects::{Seat, StorageError},
};
use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
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
}
/// A ready policy boundary includes opening, payment, targets and combat choices.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    InternalYield,
    Ready,
    Terminal(terminal::Outcome),
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
}
impl EpisodeResult {
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
}
impl Driver {
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
        let progress = self.reset(config, master, ordinal, quantum)?;
        self.capture_header = Some(header.clone());
        self.refresh_capture()?;
        Ok(progress)
    }
    /// Immutable diagnostic view; unfinished episodes have no training reader.
    pub fn trajectory(&self) -> Option<&v2::Episode> {
        self.recorder.as_ref().map(v2::Recorder::episode)
    }
    fn frame(&self) -> Result<v2::Frame, Error> {
        v2::Frame::owned(&self.game, self.revision).map_err(Error::Capture)
    }
    fn refresh_capture(&mut self) -> Result<(), Error> {
        if self.capture_header.is_none() || self.boundary() == Progress::InternalYield {
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
        self.active()?;
        if info
            .log_probability
            .is_some_and(|x| !x.is_finite() || x > 0.0)
            || info.value.is_some_and(|x| !x.is_finite())
        {
            return Err(Error::Capture(trajectory::Error::InvalidChoice));
        }
        let before = if self.capture_header.is_some() {
            Some(self.frame()?)
        } else {
            None
        };
        self.submit_core(seat, submission)?;
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
            if let Err(Error::Capture(error)) = &captured {
                self.capture_error = Some(error.clone());
            }
            captured?;
        }
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
        let progress = game
            .reset_quantum(config, master, ordinal, quantum)
            .map_err(Error::Reset)?;
        self.game = game;
        self.inputs = Some(Inputs {
            config: config.clone(),
            master,
            ordinal,
        });
        self.history.clear();
        self.capture_header = None;
        self.recorder = None;
        self.capture_error = None;
        self.finalized = false;
        self.revision = revision;
        self.progress = progress;
        Ok(self.boundary())
    }
    fn boundary(&self) -> Progress {
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
        self.progress = if self.progress == CoreProgress::OpeningComplete
            && self.game.turn_position().is_none()
        {
            self.game
                .start_turns_quantum(quantum)
                .map_err(Error::Turn)?
        } else {
            self.game.resume(quantum)
        };
        self.refresh_capture()?;
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
        self.game
            .apply_policy(seat, &core, self.capacity)
            .map_err(Error::Policy)?;
        self.history.push(record);
        // apply_policy drains its work synchronously; this only inspects the boundary.
        self.progress = self.game.resume(NonZeroUsize::MIN);
        Ok(())
    }
    pub fn concede(&mut self, seat: Seat, episode: terminal::EpisodeId) -> Result<(), Error> {
        self.active()?;
        if self.game.episode_id() != Some(episode) {
            return Err(Error::Concede(terminal::ConcedeError::StaleEpisode));
        }
        // Semantic history starts from a fully executed normal reset. A terminal
        // result during a yield could not be reproduced by that action stream.
        if self.boundary() == Progress::InternalYield {
            return Err(Error::Concede(terminal::ConcedeError::SettlementPending));
        }
        let record = actions::encode_concession(&self.game, seat).map_err(Error::Action)?;
        let outcome = self.game.concede(seat, episode).map_err(Error::Concede)?;
        self.history.push(record);
        self.progress = CoreProgress::Terminal(outcome);
        if self.capture_header.is_some() {
            let frame = self.frame()?;
            self.recorder
                .as_mut()
                .ok_or(Error::Capture(trajectory::Error::Unavailable))?
                .finish(&frame, trajectory::End::Completed)
                .map_err(Error::Capture)?;
        }
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
        let inputs = self.inputs.as_ref().ok_or(Error::NotStarted)?;
        if self.finalized {
            return Err(Error::Finalized);
        }
        if let Some(error) = &self.capture_error {
            return Err(Error::Capture(error.clone()));
        }
        let result = EpisodeResult {
            inputs: inputs.clone(),
            status: self
                .game
                .outcome()
                .map_or(Status::Incomplete, Status::Completed),
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
        };
        d.refresh_capture().unwrap();
        d
    }
}
