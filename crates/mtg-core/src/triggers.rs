//! Pending abilities are owned independently of their source incarnation.
use super::*;

/// No production trigger detection is enabled by the placement contract.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerKind {
    #[cfg(test)]
    Synthetic { tag: u32 },
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub(super) struct PendingTrigger {
    pub source: Handle,
    pub card: CardId,
    pub controller: Seat,
    pub kind: TriggerKind,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub(super) struct TriggeredAbility {
    pub object: Handle,
    pub declaration: PendingTrigger,
}

impl Game {
    pub(super) fn trigger_candidates(&self, actor: Seat) -> Vec<usize> {
        self.turns
            .pending_triggers
            .iter()
            .enumerate()
            .filter_map(|(i, p)| p.as_ref().filter(|p| p.controller == actor).map(|_| i))
            .collect()
    }

    /// Submit exactly this controller's complete bottom-to-top permutation.
    /// CR 603.3b/101.4: active controller places first, then nonactive controller.
    pub fn order_triggers_quantum(
        &mut self,
        actor: Seat,
        id: DecisionId,
        order: &[usize],
        quantum: NonZeroUsize,
    ) -> Result<Progress, turns::TurnError> {
        use turns::TurnError;
        let d = self.turn_decision().ok_or(TurnError::NotReady)?;
        let invalid = TurnError::Invalid;
        if d.actor != actor {
            return Err(invalid(ApplyError::WrongActor));
        }
        if d.id != id {
            return Err(invalid(ApplyError::StaleDecision));
        }
        if d.kind != turns::TurnKind::TriggerOrder {
            return Err(invalid(ApplyError::WrongKind));
        }
        let candidates = self.trigger_candidates(actor);
        if order.len() != candidates.len() {
            return Err(invalid(ApplyError::WrongCardinality));
        }
        for (i, row) in order.iter().enumerate() {
            if !candidates.contains(row) {
                return Err(invalid(ApplyError::IllegalCandidate));
            }
            if order[..i].contains(row) {
                return Err(invalid(ApplyError::DuplicateCandidate));
            }
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(invalid(ApplyError::DecisionExhausted))?;
        self.objects
            .prepare_allocations(order.len(), Zone::Stack)
            .map_err(TurnError::Storage)?;
        self.turns
            .stack
            .try_reserve(order.len())
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        self.turns
            .triggered
            .try_reserve(order.len())
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        self.work
            .try_reserve(order.len() + 1)
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        self.generation = generation;
        self.turns.decision = None;
        self.work
            .extend(order.iter().copied().map(Work::PlaceTrigger));
        self.work.push_back(Work::TriggerBoundary);
        Ok(self.resume(quantum))
    }

    // TriggerKind is deliberately uninhabited outside tests until a real trigger
    // source is delivered. Keep the shared placement algorithm compiled here.
    #[allow(unreachable_code, unused_variables)]
    pub(super) fn place_trigger(&mut self, row: usize) {
        let declaration = self.turns.pending_triggers[row]
            .take()
            .expect("reserved trigger");
        let object = self
            .objects
            .allocate(declaration.card, declaration.controller, Zone::Stack)
            .expect("preflighted trigger allocation");
        self.turns.stack.push(object);
        self.turns.triggered.push(TriggeredAbility {
            object,
            declaration,
        });
    }

    pub(super) fn trigger_boundary(&mut self) -> turns::TurnDecision {
        let active = self.turns.position.expect("trigger turn").1;
        for actor in [active, turns::opponent(active)] {
            if !self.trigger_candidates(actor).is_empty() {
                return self.set_turn_decision(actor, turns::TurnKind::TriggerOrder);
            }
        }
        self.turns.pending_triggers.clear();
        let (actor, passed) = self.turns.trigger_return.take().expect("placement return");
        self.turns.passed = passed;
        self.set_turn_decision(actor, turns::TurnKind::Priority)
    }
}

#[cfg(test)]
impl Game {
    pub(super) fn inject_triggers(&mut self, triggers: Vec<PendingTrigger>) {
        self.turns.pending_triggers = triggers.into_iter().map(Some).collect();
        self.turns.decision = None;
        self.work.push_back(Work::Priority {
            actor: self.turns.position.unwrap().1,
            passed: false,
            terminal: true,
        });
    }
}
