//! Rules results, indexed by persistent seat. No recorder or external budgets.
use super::*;
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LossReason {
    Life,
    EmptyDraw,
    Concession,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub winner: Option<Seat>,
    pub losses: [Option<LossReason>; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConcedeError {
    SettlementPending,
    NotStarted,
    AlreadyEnded,
    StaleEpisode,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpisodeId(pub(super) DecisionId);
impl Game {
    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }
    pub fn episode_id(&self) -> Option<EpisodeId> {
        self.episode
    }
    /// Concession does not require priority. The episode token rejects delayed
    /// or foreign commands after reset; transport authentication is external.
    pub fn concede(&mut self, seat: Seat, episode: EpisodeId) -> Result<Outcome, ConcedeError> {
        if self.work.iter().any(|w| {
            matches!(
                w,
                Work::Modify(_)
                    | Work::SpellMove { .. }
                    | Work::FinishSpell { .. }
                    | Work::Priority { .. }
            )
        }) {
            return Err(ConcedeError::SettlementPending);
        }
        let current = self.episode_id().ok_or(ConcedeError::NotStarted)?;
        if current != episode {
            return Err(ConcedeError::StaleEpisode);
        }
        if self.outcome.is_some() {
            return Err(ConcedeError::AlreadyEnded);
        }
        let mut losses = [None; 2];
        losses[seat_index(seat)] = Some(LossReason::Concession);
        Ok(self.finish_outcome(losses))
    }
    fn finish_outcome(&mut self, losses: [Option<LossReason>; 2]) -> Outcome {
        let result = Outcome {
            winner: match losses.map(|r| r.is_some()) {
                [true, false] => Some(Seat::P1),
                [false, true] => Some(Seat::P0),
                [true, true] => None,
                [false, false] => unreachable!("terminal needs a loss"),
            },
            losses,
        };
        self.outcome = Some(result);
        self.decision = None;
        self.work.clear();
        self.turns.decision = None;
        self.turns.payment = None;
        self.turns.targeting = None;
        self.turns.casting = None;
        result
    }
    /// All simultaneous loss conditions are collected before choosing a winner.
    /// Called at settled rules boundaries, never during an effect/payment.
    pub(super) fn settle_terminal(&mut self, failed_draw: Option<Seat>) -> Option<Outcome> {
        if self.outcome.is_some() {
            return self.outcome;
        }
        let losses = [Seat::P0, Seat::P1].map(|seat| {
            if self.life[seat_index(seat)] <= 0 {
                Some(LossReason::Life)
            } else if failed_draw == Some(seat) {
                Some(LossReason::EmptyDraw)
            } else {
                None
            }
        });
        if losses.iter().any(Option::is_some) {
            Some(self.finish_outcome(losses))
        } else {
            None
        }
    }
}
#[cfg(test)]
#[path = "terminal_tests.rs"]
mod tests;
