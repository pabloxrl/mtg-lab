//! Pending abilities are owned independently of their source incarnation.
use super::*;

/// The scoped cast and ETB abilities; source handles retain incarnation.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerKind {
    Pyromancer {
        target: Option<Seat>,
    },
    Archer,
    Cyclops,
    #[cfg(test)]
    Synthetic {
        tag: u32,
    },
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
    pub(super) fn cast_triggers(&self, actor: Seat, card: CardId) -> Vec<PendingTrigger> {
        if super::card_definitions::definition(card)
            .creature_base()
            .is_some()
        {
            return vec![];
        }
        self.objects
            .in_zone(Zone::Battlefield)
            .filter_map(|source| {
                let o = self.objects.get(source).expect("battlefield");
                if o.controller != actor {
                    return None;
                }
                let kind = super::card_definitions::definition(o.card).cast_trigger()?;
                Some(PendingTrigger {
                    source,
                    card: o.card,
                    controller: actor,
                    kind,
                })
            })
            .collect()
    }

    pub(super) fn prepare_trigger_resolution(
        &mut self,
        a: TriggeredAbility,
    ) -> Result<(), turns::TurnError> {
        use turns::TurnError;
        let mut change = None;
        let mut life = None;
        match a.declaration.kind {
            TriggerKind::Pyromancer { target } => {
                let target = target.ok_or(TurnError::UnsupportedStack)?;
                let mut next = self.life;
                let i = seat_index(target);
                next[i] = next[i].checked_sub(2).ok_or(TurnError::EffectOverflow)?;
                life = Some(next);
            }
            TriggerKind::Archer => {
                let mut next = self.life;
                let i = seat_index(turns::opponent(a.declaration.controller));
                next[i] = next[i].checked_sub(1).ok_or(TurnError::EffectOverflow)?;
                life = Some(next);
            }
            TriggerKind::Cyclops => {
                let h = a.declaration.source;
                if let Some(state) = self.creature_state(h).filter(|_| self.haste_target(h)) {
                    state
                        .power
                        .checked_add(3)
                        .ok_or(TurnError::EffectOverflow)?;
                    let old = self
                        .turns
                        .modifications
                        .iter()
                        .find(|m| m.handle == h)
                        .copied()
                        .unwrap_or(targets::Modification {
                            handle: h,
                            boost: 0,
                            power_boost: 0,
                            damage: 0,
                        });
                    change = Some(targets::Modification {
                        power_boost: old
                            .power_boost
                            .checked_add(3)
                            .ok_or(TurnError::EffectOverflow)?,
                        ..old
                    });
                    self.turns
                        .modifications
                        .try_reserve(1)
                        .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
                }
            }
            #[cfg(test)]
            TriggerKind::Synthetic { .. } => {}
        }
        self.objects
            .prepare_removals(1)
            .map_err(TurnError::Storage)?;
        if let Some(life) = life {
            self.work.push_back(Work::TriggerLife(life));
        }
        if let Some(change) = change {
            self.work.push_back(Work::Modify(change));
        }
        self.work.push_back(Work::RemoveAbility(a.object));
        Ok(())
    }
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
        self.turns
            .trigger_placement
            .try_reserve(order.len())
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        self.generation = generation;
        self.turns.decision = None;
        self.work
            .extend(order.iter().copied().map(Work::PlaceTrigger));
        self.work.push_back(Work::TriggerBoundary);
        Ok(self.resume(quantum))
    }

    /// Required CR 603.3d target; no pass/cancel or absent target is legal.
    pub fn target_trigger_quantum(
        &mut self,
        actor: Seat,
        id: DecisionId,
        target: Seat,
        quantum: NonZeroUsize,
    ) -> Result<Progress, turns::TurnError> {
        use turns::TurnError;
        let d = self.turn_decision().ok_or(TurnError::NotReady)?;
        if d.actor != actor {
            return Err(TurnError::Invalid(ApplyError::WrongActor));
        }
        if d.id != id {
            return Err(TurnError::Invalid(ApplyError::StaleDecision));
        }
        if d.kind != turns::TurnKind::TriggerTarget {
            return Err(TurnError::Invalid(ApplyError::WrongKind));
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(TurnError::Invalid(ApplyError::DecisionExhausted))?;
        self.work
            .try_reserve(self.turns.trigger_placement.len() + 1)
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        let row = self.turns.trigger_placement[0];
        self.turns.pending_triggers[row]
            .as_mut()
            .expect("targeted trigger")
            .kind = TriggerKind::Pyromancer {
            target: Some(target),
        };
        self.generation = generation;
        self.turns.decision = None;
        self.work.extend(
            self.turns
                .trigger_placement
                .drain(..)
                .map(Work::PlaceTrigger),
        );
        self.work.push_back(Work::TriggerBoundary);
        Ok(self.resume(quantum))
    }

    pub(super) fn place_trigger(&mut self, row: usize) {
        let pending = self.turns.pending_triggers[row]
            .as_ref()
            .expect("reserved trigger");
        if matches!(pending.kind, TriggerKind::Pyromancer { target: None }) {
            let controller = pending.controller;
            self.turns.trigger_placement.push(row);
            while let Some(work) = self.work.pop_front() {
                match work {
                    Work::PlaceTrigger(row) => self.turns.trigger_placement.push(row),
                    Work::TriggerBoundary => {}
                    _ => unreachable!("placement batch"),
                }
            }
            self.set_turn_decision(controller, turns::TurnKind::TriggerTarget);
            return;
        }
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
