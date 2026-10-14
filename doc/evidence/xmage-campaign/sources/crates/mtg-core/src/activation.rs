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
    pub sources: Vec<Handle>,
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
                    || card_definitions::definition(o.card).power_activation()
                    || card_definitions::definition(o.card).invoker_activation())
        })
    }
    pub fn activation_candidates(&self, actor: Seat) -> Vec<Handle> {
        if !self
            .turn_decision()
            .is_some_and(|d| d.actor == actor && d.kind == TurnKind::Priority)
        {
            return vec![];
        }
        let mut available = self.turns.mana[seat_index(actor)].map(u64::from);
        for h in self.mana_sources(actor) {
            let color = mana::mana_color(self.objects.get(h).unwrap().card).unwrap();
            available[color.index()] += 1;
        }
        self.objects
            .in_zone(Zone::Battlefield)
            .filter(|h| {
                let definition = card_definitions::definition(self.objects.get(*h).unwrap().card);
                self.usable_activation_source(actor, *h)
                    && (!definition.power_activation() || available[3] > 0)
                    && (!definition.invoker_activation() || available.iter().sum::<u64>() >= 8)
            })
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
            sources: vec![],
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
        p.reserved[color as usize] += 1;
        p.paid = p.reserved.iter().sum::<u32>() == if p.invoker { 8 } else { 1 };
        p.id.generation = generation;
        self.generation = generation;
        Ok(())
    }
    /// Mana abilities resolve immediately within the private transaction. The
    /// resulting mana and tap costs become public together with the activation.
    pub fn activation_mana_sources(&self, actor: Seat) -> Vec<Handle> {
        let Some(p) = self
            .turns
            .activation
            .as_ref()
            .filter(|p| p.actor == actor && (p.power || (p.invoker && p.target.is_some())))
        else {
            return vec![];
        };
        self.objects
            .in_zone(Zone::Battlefield)
            .filter(|h| self.usable_mana_source(actor, *h) && !p.sources.contains(h))
            .collect()
    }
    pub fn activation_tap_mana(
        &mut self,
        actor: Seat,
        id: DecisionId,
        source: Handle,
    ) -> Result<(), TurnError> {
        let p = self.validate_activation(actor, id)?;
        if !self.activation_mana_sources(actor).contains(&source) {
            return Err(TurnError::Invalid(ApplyError::IllegalCandidate));
        }
        let color = mana::mana_color(self.objects.get(source).unwrap().card)
            .unwrap()
            .index();
        self.activation_pool(p)?[color]
            .checked_add(1)
            .ok_or(TurnError::EffectOverflow)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(TurnError::Invalid(ApplyError::DecisionExhausted))?;
        let p = self.turns.activation.as_mut().unwrap();
        p.sources
            .try_reserve(1)
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        p.sources.push(source);
        p.id.generation = generation;
        self.generation = generation;
        Ok(())
    }
    /// Recompute against live public resources to revalidate every reserved
    /// incarnation. Wider arithmetic permits spending before generating mana at
    /// the pool limit, while the resulting pool itself must fit its contract.
    pub(super) fn activation_pool(&self, p: &PendingActivation) -> Result<[u32; 6], TurnError> {
        let mut pool = self.turns.mana[seat_index(p.actor)].map(u64::from);
        for (i, h) in p.sources.iter().enumerate() {
            if p.sources[..i].contains(h) || !self.usable_mana_source(p.actor, *h) {
                return Err(TurnError::Invalid(ApplyError::IllegalCandidate));
            }
            let color = mana::mana_color(self.objects.get(*h).unwrap().card)
                .unwrap()
                .index();
            pool[color] += 1;
        }
        let mut remaining = [0; 6];
        for i in 0..6 {
            remaining[i] = u32::try_from(
                pool[i]
                    .checked_sub(u64::from(p.reserved[i]))
                    .ok_or(TurnError::Invalid(ApplyError::IllegalCandidate))?,
            )
            .map_err(|_| TurnError::EffectOverflow)?;
        }
        Ok(remaining)
    }
    pub(super) fn activation_payment_allowed(&self, p: &PendingActivation, color: usize) -> bool {
        !p.paid
            && (p.power && color == 3 || p.invoker && p.target.is_some())
            && self.activation_pool(p).is_ok_and(|pool| pool[color] > 0)
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
        let pool = self.activation_pool(p)?;
        if (power && (!p.paid || target.is_some()))
            || (!power && target.is_none())
            || (invoker && !p.paid)
        {
            return Err(TurnError::Invalid(ApplyError::WrongCardinality));
        }
        if !self.usable_activation_source(actor, source)
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
        let pending = self.turns.activation.take().unwrap();
        for h in pending.sources {
            self.objects.get_mut(h).unwrap().tapped = true;
        }
        self.turns.mana[seat_index(actor)] = pool;
        if !power && !invoker {
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
