//! Authoritative game state and shared decision identity.
//!
//! Rule modules remain children so Game internals stay private to the engine.
//! Opening setup and owned work execution live in their own modules.
use crate::objects::{CardId, Handle, ObjectStore, Seat, StorageError, Zone};
use crate::rng::{EpisodeRng, Stream, VERSION};
use std::{collections::VecDeque, num::NonZeroUsize};

#[path = "activation.rs"]
pub mod activation;
#[path = "card_definitions.rs"]
mod card_definitions;
#[path = "cast_state.rs"]
mod cast_state;
#[path = "casting.rs"]
pub mod casting;
#[path = "combat.rs"]
pub mod combat;
#[path = "mana.rs"]
pub mod mana;
#[path = "opening.rs"]
pub mod opening;
#[path = "policy.rs"]
pub mod policy;
#[path = "targets.rs"]
pub mod targets;
#[path = "terminal.rs"]
pub mod terminal;
#[path = "triggers.rs"]
pub mod triggers;
#[path = "turns.rs"]
pub mod turns;
#[path = "views.rs"]
pub mod views;
#[path = "work.rs"]
mod work;
pub use opening::{
    Config, DeckConfig, DrawError, FORMAT, OpeningAction, OpeningChoice, OpeningDecision,
    OpeningKind, ResetError, SHUFFLE_VERSION, Selection,
};
pub use work::Progress;
use work::Work;
#[cfg(test)]
use work::shuffle_word;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecisionId {
    scope: u64,
    pub generation: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateId {
    pub decision: DecisionId,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyError {
    NoDecision,
    WorkPending,
    WrongActor,
    StaleDecision,
    StaleCandidate,
    WrongKind,
    IllegalCandidate,
    WrongCardinality,
    DuplicateCandidate,
    InvalidOrder,
    UnexpectedOrder,
    DecisionExhausted,
}

#[derive(serde::Serialize, Debug)]
pub struct Game {
    // Destination-local policy timeline; never loaded from a snapshot.
    #[serde(skip)]
    policy_revision: u64,
    outcome: Option<terminal::Outcome>,
    episode: Option<terminal::EpisodeId>,
    turns: turns::TurnState,
    work: VecDeque<Work>,
    objects: ObjectStore,
    life: [i64; 2],
    decision: Option<OpeningDecision>,
    rng: Option<EpisodeRng>,
    generation: u64,
    starting: Seat,
    kept: [bool; 2],
    declarations: [Option<OpeningChoice>; 2],
    mulligans: [usize; 2],
    orders: [Option<Vec<Handle>>; 2],
    needs_bottom: [bool; 2],
}
impl Game {
    pub fn new() -> Result<Self, StorageError> {
        Ok(Self {
            policy_revision: 0,
            outcome: None,
            episode: None,
            turns: turns::TurnState::default(),
            work: VecDeque::new(),
            objects: ObjectStore::new()?,
            life: [0; 2],
            decision: None,
            rng: None,
            generation: 0,
            starting: Seat::P0,
            kept: [false; 2],
            declarations: [None; 2],
            mulligans: [0; 2],
            orders: [None, None],
            needs_bottom: [false; 2],
        })
    }
    pub fn objects(&self) -> &ObjectStore {
        &self.objects
    }
    pub fn life(&self) -> [i64; 2] {
        self.life
    }
}

fn seat_index(seat: Seat) -> usize {
    match seat {
        Seat::P0 => 0,
        Seat::P1 => 1,
    }
}
fn validate_candidate(
    candidate: CandidateId,
    decision: DecisionId,
    count: usize,
) -> Result<(), ApplyError> {
    if candidate.decision != decision {
        return Err(ApplyError::StaleCandidate);
    }
    if candidate.index >= count {
        return Err(ApplyError::IllegalCandidate);
    }
    Ok(())
}

#[cfg(test)]
#[path = "targets_tests.rs"]
mod targets_tests;

#[path = "snapshot.rs"]
pub mod snapshot;

#[path = "replay.rs"]
pub mod replay;

#[cfg(test)]
#[path = "trajectory_rules_tests.rs"]
mod trajectory_rules_tests;

#[path = "actions.rs"]
pub mod actions;

#[cfg(test)]
#[path = "instant_reference_tests.rs"]
mod instant_reference_tests;

#[cfg(test)]
#[path = "trajectory_v2_tests.rs"]
mod trajectory_v2_tests;

#[cfg(test)]
#[path = "episode_capture_tests.rs"]
mod episode_capture_tests;

#[cfg(test)]
#[path = "cast_boundary_tests.rs"]
mod cast_boundary_tests;

#[cfg(test)]
#[path = "card_definition_tests.rs"]
mod card_definition_tests;

#[cfg(test)]
#[path = "token_tests.rs"]
mod token_tests;

#[cfg(test)]
#[path = "creature_mana_reference_tests.rs"]
mod creature_mana_reference_tests;

#[cfg(test)]
#[path = "haste_tests.rs"]
mod haste_tests;

#[cfg(test)]
#[path = "shivan_tests.rs"]
mod shivan_tests;

#[cfg(test)]
#[path = "invoker_tests.rs"]
mod invoker_tests;

#[cfg(test)]
#[path = "cast_trigger_tests.rs"]
mod cast_trigger_tests;
#[cfg(test)]
#[path = "trigger_tests.rs"]
mod trigger_tests;

#[cfg(test)]
#[path = "etb_trigger_tests.rs"]
mod etb_trigger_tests;

#[cfg(test)]
#[path = "cleanup_trigger_tests.rs"]
mod cleanup_trigger_tests;

#[cfg(test)]
#[path = "m2_cost_reference_tests.rs"]
mod m2_cost_reference_tests;
