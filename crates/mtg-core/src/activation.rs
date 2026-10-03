//! Nonmana activations. The stack owns an independent ability object;
//! source and target handles identify incarnations, never replacement cards.
use super::turns::{TurnDecision, TurnError, TurnKind};
use super::*;
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(super) struct PendingActivation {
    pub source: Handle,
    pub actor: Seat,
    pub target: Option<Handle>,
    pub paid: bool,
    pub reserved: [u32; 6],
    pub power: bool,
    pub invoker: bool,
    pub id: DecisionId,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub(super) struct Ability {
    pub object: Handle,
    pub source: Handle,
    pub target: Option<Handle>,
    pub power: bool,
    pub invoker: bool,
}
impl Game {
    pub fn has_haste(&self, h: Handle) -> bool {
        self.objects
            .get(h)
            .is_ok_and(|o| o.zone == Zone::Battlefield)
            && self.turns.haste.contains(&h)
    }
    pub(super) fn haste_target(&self, h: Handle) -> bool {
        self.objects.get(h).is_ok_and(|o| {
            o.zone == Zone::Battlefield
                && card_definitions::definition(o.card)
                    .creature_base()
                    .is_some()
        })
    }
    pub(super) fn usable_activation_source(&self, actor: Seat, h: Handle) -> bool {
        self.objects.get(h).is_ok_and(|o| {
            o.zone == Zone::Battlefield
                && o.controller == actor
                && ((card_definitions::definition(o.card).haste_activation()
                    && !o.tapped
                    && !self.summoning_sick(h))
                    || (card_definitions::definition(o.card).power_activation()
                        && self.turns.mana[seat_index(actor)][3] > 0)
                    || (card_definitions::definition(o.card).invoker_activation()
                        && self.turns.mana[seat_index(actor)]
                            .iter()
                            .map(|n| u64::from(*n))
                            .sum::<u64>()
                            >= 8))
        })
    }
    pub fn activation_candidates(&self, actor: Seat) -> Vec<Handle> {
        if !self
            .turn_decision()
            .is_some_and(|d| d.actor == actor && d.kind == TurnKind::Priority)
        {
            return vec![];
        }
        self.objects
            .in_zone(Zone::Battlefield)
            .filter(|h| self.usable_activation_source(actor, *h))
            .collect()
    }
    pub fn begin_activation(
        &mut self,
        actor: Seat,
        id: DecisionId,
        source: Handle,
    ) -> Result<(), TurnError> {
        self.mana_priority(actor, id)
            .map_err(|_| TurnError::Invalid(ApplyError::IllegalCandidate))?;
        if !self.activation_candidates(actor).contains(&source) {
            return Err(TurnError::Invalid(ApplyError::IllegalCandidate));
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(TurnError::Invalid(ApplyError::DecisionExhausted))?;
        self.turns.activation = Some(PendingActivation {
            source,
            actor,
            target: None,
            paid: false,
            reserved: [0; 6],
            invoker: card_definitions::definition(self.objects.get(source).unwrap().card)
                .invoker_activation(),
            power: card_definitions::definition(self.objects.get(source).unwrap().card)
                .power_activation(),
            id: DecisionId {
                scope: self.objects.scope(),
                generation,
            },
        });
        self.generation = generation;
        self.turns.decision = None;
        Ok(())
    }
    fn validate_activation(
        &self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<&PendingActivation, TurnError> {
        if self.outcome.is_some() || !self.work.is_empty() {
            return Err(TurnError::NotReady);
        }
        let p = self.turns.activation.as_ref().ok_or(TurnError::NotReady)?;
        if p.actor != actor {
            return Err(TurnError::Invalid(ApplyError::WrongActor));
        }
        if p.id != id {
            return Err(TurnError::Invalid(ApplyError::StaleDecision));
        }
        Ok(p)
    }
    pub fn choose_activation_target(
        &mut self,
        actor: Seat,
        id: DecisionId,
        target: Handle,
    ) -> Result<(), TurnError> {
        let p = self.validate_activation(actor, id)?;
        if p.power || (p.invoker && p.target.is_some()) || !self.haste_target(target) {
            return Err(TurnError::Invalid(ApplyError::IllegalCandidate));
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(TurnError::Invalid(ApplyError::DecisionExhausted))?;
        let p = self.turns.activation.as_mut().unwrap();
        p.target = Some(target);
        p.id.generation = generation;
        self.generation = generation;
        Ok(())
    }
    pub fn pay_activation(
        &mut self,
        actor: Seat,
        id: DecisionId,
        color: mana::Color,
    ) -> Result<(), TurnError> {
        let p = self.validate_activation(actor, id)?;
        if !self.activation_payment_allowed(p, color as usize) {
            return Err(TurnError::Invalid(ApplyError::IllegalCandidate));
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(TurnError::Invalid(ApplyError::DecisionExhausted))?;
        let p = self.turns.activation.as_mut().unwrap();
        if p.invoker {
            p.reserved[color as usize] += 1;
            p.paid = p.reserved.iter().sum::<u32>() == 8;
        } else {
            p.paid = true;
        }
        p.id.generation = generation;
        self.generation = generation;
        Ok(())
    }
    pub(super) fn activation_payment_allowed(&self, p: &PendingActivation, color: usize) -> bool {
        !p.paid
            && if p.invoker {
                p.target.is_some()
                    && self.turns.mana[seat_index(p.actor)][color] > p.reserved[color]
            } else {
                p.power && color == 3 && self.turns.mana[seat_index(p.actor)][3] > 0
            }
    }
    pub fn cancel_activation(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<TurnDecision, TurnError> {
        self.validate_activation(actor, id)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(TurnError::Invalid(ApplyError::DecisionExhausted))?;
        self.turns.activation = None;
        self.generation = generation;
        Ok(self.set_turn_decision(actor, TurnKind::Priority))
    }
    pub fn finish_activation(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<TurnDecision, TurnError> {
        let p = self.validate_activation(actor, id)?;
        let source = p.source;
        let target = p.target;
        let power = p.power;
        let invoker = p.invoker;
        let reserved = p.reserved;
        if (power && (!p.paid || target.is_some()))
            || (!power && target.is_none())
            || (invoker && !p.paid)
        {
            return Err(TurnError::Invalid(ApplyError::WrongCardinality));
        }
        if (invoker
            && reserved
                .iter()
                .zip(self.turns.mana[seat_index(actor)])
                .any(|(r, m)| *r > m))
            || !self.usable_activation_source(actor, source)
            || target.is_some_and(|h| !self.haste_target(h))
        {
            return Err(TurnError::Invalid(ApplyError::IllegalCandidate));
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(TurnError::Invalid(ApplyError::DecisionExhausted))?;
        let card = self.objects.get(source).unwrap().card;
        self.turns
            .stack
            .try_reserve(1)
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        self.turns
            .abilities
            .try_reserve(1)
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        self.objects
            .prepare_allocations(1, Zone::Stack)
            .map_err(TurnError::Storage)?;
        let object = self
            .objects
            .allocate(card, actor, Zone::Stack)
            .map_err(TurnError::Storage)?;
        if invoker {
            for (m, r) in self.turns.mana[seat_index(actor)].iter_mut().zip(reserved) {
                *m -= r;
            }
        } else if power {
            self.turns.mana[seat_index(actor)][3] -= 1;
        } else {
            self.objects.get_mut(source).unwrap().tapped = true;
        }
        self.turns.stack.push(object);
        self.turns.abilities.push(Ability {
            object,
            source,
            target,
            power,
            invoker,
        });
        self.turns.activation = None;
        self.turns.passed = false;
        self.generation = generation;
        Ok(self.set_turn_decision(actor, TurnKind::Priority))
    }
}
