//! Atomic vanilla creature casts. Public inspections here remain privileged.
use super::mana::{ManaCost, ManaError, Payment, PaymentDecision, basic_color};
use super::turns::{Step, TurnDecision, TurnError, TurnKind};
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastError {
    Mana(ManaError),
    IllegalSpell,
    NoCast,
    Storage(StorageError),
}
#[derive(Clone, Debug)]
pub(super) struct PendingCast {
    pub(super) card: Handle,
    pub(super) sources: Vec<Handle>,
    pub(super) effect: Option<super::targets::Effect>,
}
pub(super) fn cost(card: CardId) -> Option<ManaCost> {
    let mut colored = [0; 6];
    match card.identity().key {
        "bear-cub" | "giant-growth" | "bite-down" => colored[4] = 1,
        "swab-goblin" => colored[3] = 1,
        _ => return None,
    }
    Some(ManaCost {
        colored,
        generic: if card.identity().key == "giant-growth" {
            0
        } else {
            1
        },
    })
}
impl Game {
    /// Legal spell handles for the current priority decision. Timing and total
    /// available pool/basic-land resources are checked, without auto-paying.
    pub fn cast_candidates(&self, actor: Seat) -> Vec<Handle> {
        if !self
            .turn_decision()
            .is_some_and(|d| d.actor == actor && d.kind == TurnKind::Priority)
        {
            return vec![];
        }
        let mut available = self.turns.mana[seat_index(actor)].map(u64::from);
        for h in self.mana_sources(actor) {
            let i = basic_color(self.objects.get(h).expect("source").card)
                .unwrap()
                .index();
            available[i] += 1;
        }
        self.objects
            .in_zone(Zone::Hand(actor))
            .filter(|h| {
                cost(self.objects.get(*h).expect("hand").card).is_some_and(|c| {
                    let card = self.objects.get(*h).unwrap().card;
                    let timing = if super::targets::instant(card) {
                        self.has_targets(actor, card)
                    } else {
                        self.turns.position.is_some_and(|(_, active, step)| {
                            active == actor
                                && matches!(step, Step::PrecombatMain | Step::PostcombatMain)
                        }) && self.objects.in_zone(Zone::Stack).next().is_none()
                    };
                    timing
                        && (0..6).all(|i| available[i] >= u64::from(c.colored[i]))
                        && available.iter().sum::<u64>()
                            >= u64::from(c.generic)
                                + c.colored.iter().map(|n| u64::from(*n)).sum::<u64>()
                })
            })
            .collect()
    }
    pub fn begin_cast(
        &mut self,
        actor: Seat,
        id: DecisionId,
        card: Handle,
    ) -> Result<PaymentDecision, CastError> {
        self.mana_priority(actor, id).map_err(CastError::Mana)?;
        if !self.cast_candidates(actor).contains(&card)
            || self
                .objects
                .get(card)
                .is_ok_and(|o| super::targets::instant(o.card))
        {
            return Err(CastError::IllegalSpell);
        }
        let generation = self.next_mana_generation().map_err(CastError::Mana)?;
        let payment = Payment {
            actor,
            id: DecisionId {
                scope: self.objects.scope(),
                generation,
            },
            remaining: cost(self.objects.get(card).expect("candidate").card).expect("supported"),
            pool: self.turns.mana[seat_index(actor)],
        };
        let d = payment.decision();
        self.turns.payment = Some(payment);
        self.turns.casting = Some(PendingCast {
            card,
            sources: vec![],
            effect: None,
        });
        self.generation = generation;
        Ok(d)
    }
    /// Available basic mana abilities during a cast. Activations are staged
    /// with the entire logical action, so cancellation/rejection leaks no taps.
    pub fn cast_mana_sources(&self, actor: Seat) -> Vec<Handle> {
        let Some(cast) = self.turns.casting.as_ref() else {
            return vec![];
        };
        if self.payment_decision(actor).is_none() {
            return vec![];
        }
        self.objects
            .in_zone(Zone::Battlefield)
            .filter(|h| {
                let o = self.objects.get(*h).expect("battlefield");
                o.controller == actor
                    && !o.tapped
                    && basic_color(o.card).is_some()
                    && !cast.sources.contains(h)
            })
            .collect()
    }
    pub fn cast_tap_mana(
        &mut self,
        actor: Seat,
        id: DecisionId,
        land: Handle,
    ) -> Result<PaymentDecision, CastError> {
        self.validate_payment(actor, id).map_err(CastError::Mana)?;
        if !self.cast_mana_sources(actor).contains(&land) {
            return Err(CastError::Mana(ManaError::IllegalSource));
        }
        let i = basic_color(self.objects.get(land).expect("source").card)
            .unwrap()
            .index();
        let value = self.turns.payment.as_ref().unwrap().pool[i]
            .checked_add(1)
            .ok_or(CastError::Mana(ManaError::Overflow))?;
        let generation = self.next_mana_generation().map_err(CastError::Mana)?;
        let cast = self.turns.casting.as_mut().expect("cast source");
        cast.sources
            .try_reserve(1)
            .map_err(|_| CastError::Storage(StorageError::CapacityExceeded))?;
        cast.sources.push(land);
        let p = self.turns.payment.as_mut().unwrap();
        p.pool[i] = value;
        p.id.generation = generation;
        self.generation = generation;
        Ok(p.decision())
    }
    /// Commit the paid spell and chosen activations together; never expose the
    /// payment layer's intermediate priority. The caster then retains priority.
    pub fn finish_cast(&mut self, actor: Seat, id: DecisionId) -> Result<TurnDecision, CastError> {
        let p = self.validate_payment(actor, id).map_err(CastError::Mana)?;
        let cast = self.turns.casting.as_ref().ok_or(CastError::NoCast)?;
        if p.remaining.generic > 0 || p.remaining.colored.iter().any(|n| *n > 0) {
            return Err(CastError::Mana(ManaError::IllegalPayment));
        }
        self.next_mana_generation().map_err(CastError::Mana)?;
        let card = cast.card;
        if !self
            .objects
            .get(card)
            .is_ok_and(|o| o.zone == Zone::Hand(actor))
            || cast.effect.is_some_and(|e| {
                self.legal_effect_targets(actor, e)
                    != match e {
                        super::targets::Effect::Growth(_) => 1,
                        super::targets::Effect::Bite(_, _) => 2,
                    }
            })
        {
            return Err(CastError::IllegalSpell);
        }
        for &source in &cast.sources {
            if !self
                .objects
                .get(source)
                .is_ok_and(|o| o.zone == Zone::Battlefield && o.controller == actor && !o.tapped)
            {
                return Err(CastError::Mana(ManaError::IllegalSource));
            }
        }
        self.turns
            .effects
            .try_reserve(1)
            .map_err(|_| CastError::Storage(StorageError::CapacityExceeded))?;
        // Storage preflight occurs before any spend, tap, generation or move.
        self.objects
            .prepare_moves(&[card], Zone::Stack)
            .map_err(CastError::Storage)?;
        self.turns
            .stack
            .try_reserve(1)
            .map_err(|_| CastError::Storage(StorageError::CapacityExceeded))?;
        let cast = self.turns.casting.take().expect("validated");
        let d = self.commit_payment(actor, id).expect("preflighted payment");
        for h in cast.sources {
            self.objects.get_mut(h).expect("reserved source").tapped = true;
        }
        let h = self
            .objects
            .move_to(card, Zone::Stack)
            .expect("preflighted spell");
        self.turns.stack.push(h);
        if let Some(effect) = cast.effect {
            self.turns.effects.push((h, effect));
        }
        Ok(d)
    }
    pub fn summoning_sick(&self, h: Handle) -> bool {
        self.objects
            .get(h)
            .is_ok_and(|o| o.zone == Zone::Battlefield)
            && self.turns.sick.contains(&h)
    }
    pub(super) fn pass_stack(
        &mut self,
        actor: Seat,
        generation: u64,
    ) -> Result<super::turns::TurnProgress, TurnError> {
        // Raw storage placement is not an executable spell. Preserve the explicit
        // unsupported-state rejection used by storage-only synthetic callers.
        if !self
            .objects
            .in_zone(Zone::Stack)
            .eq(self.turns.stack.iter().copied())
        {
            return Err(TurnError::UnsupportedStack);
        }
        let h = *self.turns.stack.last().ok_or(TurnError::UnsupportedStack)?;
        if cost(self.objects.get(h).map_err(TurnError::Storage)?.card).is_none() {
            return Err(TurnError::UnsupportedStack);
        }
        if !self.turns.passed {
            self.turns.passed = true;
            self.generation = generation;
            let next = if actor == Seat::P0 {
                Seat::P1
            } else {
                Seat::P0
            };
            return Ok(super::turns::TurnProgress::Decision(
                self.set_turn_decision(next, TurnKind::Priority),
            ));
        }
        if let Some((_, effect)) = self
            .turns
            .effects
            .iter()
            .find(|(spell, _)| *spell == h)
            .copied()
        {
            self.resolve_effect(h, effect)?;
        } else {
            if super::targets::instant(self.objects.get(h).unwrap().card) {
                return Err(TurnError::UnsupportedStack);
            }
            self.objects
                .prepare_moves(&[h], Zone::Battlefield)
                .map_err(TurnError::Storage)?;
            self.turns
                .sick
                .try_reserve(1)
                .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
            let controller = self.objects.get(h).expect("spell").controller;
            let permanent = self
                .objects
                .move_to(h, Zone::Battlefield)
                .expect("preflighted resolution");
            self.objects
                .get_mut(permanent)
                .expect("permanent")
                .controller = controller;
            self.turns.sick.push(permanent);
        }
        self.turns.stack.pop();
        self.turns.passed = false;
        self.generation = generation;
        let active = self.turns.position.expect("turn").1;
        if let Some(result) = self.settle_terminal(None) {
            return Ok(super::turns::TurnProgress::Terminal(result));
        }
        Ok(super::turns::TurnProgress::Decision(
            self.set_turn_decision(active, TurnKind::Priority),
        ))
    }
}
#[cfg(test)]
#[path = "casting_tests.rs"]
mod tests;
