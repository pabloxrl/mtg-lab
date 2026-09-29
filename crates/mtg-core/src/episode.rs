//! Owned scalar execution; privileged history is separate from policy views.
use crate::{
    game::{self, Config, Game, Progress as CoreProgress, actions, policy, terminal},
    objects::{Seat, StorageError},
};
use std::num::NonZeroUsize;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Storage(StorageError),
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
}
impl EpisodeResult {
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
        })
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
        let record = actions::encode_concession(&self.game, seat).map_err(Error::Action)?;
        let outcome = self.game.concede(seat, episode).map_err(Error::Concede)?;
        self.history.push(record);
        self.progress = CoreProgress::Terminal(outcome);
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
        let result = EpisodeResult {
            inputs: inputs.clone(),
            status: self
                .game
                .outcome()
                .map_or(Status::Incomplete, Status::Completed),
            snapshot: self.game.snapshot(),
            history: self.history.clone(),
        };
        self.finalized = true;
        Ok(result)
    }
}
