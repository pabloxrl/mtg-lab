//! Atomic vanilla creature casts. Public inspections here remain privileged.
use super::mana::{ManaCost, ManaError, PaymentDecision, mana_color};
use super::turns::{Step, TurnDecision, TurnError, TurnKind};
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastError {
    Mana(ManaError),
    IllegalSpell,
    NoCast,
    Storage(StorageError),
}
pub(super) use super::cast_state::PendingCast;
pub(super) fn cost(card: CardId) -> Option<ManaCost> {
    super::card_definitions::definition(card).cost()
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
            let i = mana_color(self.objects.get(h).expect("source").card)
                .unwrap()
                .index();
            available[i] += 1;
        }
        self.objects
            .in_zone(Zone::Hand(actor))
            .filter(|h| {
                cost(self.objects.get(*h).expect("hand").card).is_some_and(|c| {
                    let card = self.objects.get(*h).unwrap().card;
                    let timing = if super::card_definitions::definition(card).discard_draw() {
                        self.objects
                            .in_zone(Zone::Hand(actor))
                            .any(|other| other != *h)
                    } else if super::targets::instant(card) {
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
                .is_ok_and(|o| super::targets::targeted(o.card))
        {
            return Err(CastError::IllegalSpell);
        }
        let generation = self.next_mana_generation().map_err(CastError::Mana)?;
        Ok(self.start_cast_payment(
            actor,
            generation,
            card,
            cost(self.objects.get(card).expect("candidate").card).expect("supported"),
            None,
        ))
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
            .filter(|h| self.usable_mana_source(actor, *h) && !cast.sources().contains(h))
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
        let i = mana_color(self.objects.get(land).expect("source").card)
            .unwrap()
            .index();
        let value = self.turns.payment.as_ref().unwrap().pool()[i]
            .checked_add(1)
            .ok_or(CastError::Mana(ManaError::Overflow))?;
        let generation = self.next_mana_generation().map_err(CastError::Mana)?;
        self.stage_cast_mana(land, i, value, generation)
            .map_err(CastError::Storage)
    }
    /// Commit the paid spell and chosen activations together; never expose the
    /// payment layer's intermediate priority. The caster then retains priority.
    pub fn finish_cast(&mut self, actor: Seat, id: DecisionId) -> Result<TurnDecision, CastError> {
        let p = self.validate_payment(actor, id).map_err(CastError::Mana)?;
        let cast = self.turns.casting.as_ref().ok_or(CastError::NoCast)?;
        if !p.is_paid() {
            return Err(CastError::Mana(ManaError::IllegalPayment));
        }
        self.next_mana_generation().map_err(CastError::Mana)?;
        let card = cast.card();
        let discard = cast.discard();
        if self.cast_discard_pending()
            || discard.is_some_and(|h| {
                h == card
                    || !self
                        .objects
                        .get(h)
                        .is_ok_and(|o| o.zone == Zone::Hand(actor))
            })
        {
            return Err(CastError::IllegalSpell);
        }
        if !self
            .objects
            .get(card)
            .is_ok_and(|o| o.zone == Zone::Hand(actor))
            || cast.effect().is_some_and(|e| {
                self.legal_effect_targets(actor, e)
                    != match e {
                        super::targets::Effect::Growth(_) => 1,
                        super::targets::Effect::Bite(_, _) => 2,
                    }
            })
        {
            return Err(CastError::IllegalSpell);
        }
        for &source in cast.sources() {
            if !self.usable_mana_source(actor, source) {
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
        if let Some(h) = discard {
            // Both public moves append knowledge. Separate zone preflights
            // alone would each reserve only one entry at the same old length.
            self.objects
                .reserve_knowledge(Zone::Stack, 2)
                .map_err(CastError::Storage)?;
            let owner = self.objects.get(h).expect("validated discard").owner;
            self.objects
                .prepare_moves(&[h], Zone::Graveyard(owner))
                .map_err(CastError::Storage)?;
        }
        let (cast, d) = self.commit_cast_payment(actor, id);
        if let Some(h) = discard {
            let owner = self.objects.get(h).unwrap().owner;
            self.objects
                .move_to(h, Zone::Graveyard(owner))
                .expect("preflighted discard");
        }
        for &h in cast.sources() {
            self.objects.get_mut(h).expect("reserved source").tapped = true;
        }
        let h = self
            .objects
            .move_to(card, Zone::Stack)
            .expect("preflighted spell");
        self.turns.stack.push(h);
        if let Some(effect) = cast.effect() {
            self.turns.effects.push((h, effect));
        }
        Ok(d)
    }
    /// Private additional-cost choice. Nothing changes zones until finish_cast.
    pub fn choose_cast_discard(
        &mut self,
        actor: Seat,
        id: DecisionId,
        cards: &[Handle],
    ) -> Result<PaymentDecision, CastError> {
        self.validate_payment(actor, id).map_err(CastError::Mana)?;
        let cast = self.turns.casting.as_ref().ok_or(CastError::NoCast)?;
        if !super::card_definitions::definition(
            self.objects
                .get(cast.card())
                .map_err(CastError::Storage)?
                .card,
        )
        .discard_draw()
            || cast.discard().is_some()
            || cards.len() != 1
            || cards[0] == cast.card()
            || !self
                .objects
                .get(cards[0])
                .is_ok_and(|o| o.zone == Zone::Hand(actor))
        {
            return Err(CastError::IllegalSpell);
        }
        let generation = self.next_mana_generation().map_err(CastError::Mana)?;
        self.stage_cast_discard(cards[0], generation);
        Ok(self.payment_decision(actor).expect("pending payment"))
    }
    pub(super) fn cast_discard_pending(&self) -> bool {
        self.turns.casting.as_ref().is_some_and(|c| {
            c.discard().is_none()
                && self
                    .objects
                    .get(c.card())
                    .is_ok_and(|o| super::card_definitions::definition(o.card).discard_draw())
        })
    }
    pub fn summoning_sick(&self, h: Handle) -> bool {
        self.objects
            .get(h)
            .is_ok_and(|o| o.zone == Zone::Battlefield)
            && self.turns.sick.contains(&h)
            && !self.has_haste(h)
    }
    pub(super) fn queue_stack_pass(
        &mut self,
        actor: Seat,
        generation: u64,
    ) -> Result<(), TurnError> {
        // Raw storage placement is not an executable spell.
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
        // Reserve bounded spell work before accepting the pass.
        self.work
            .try_reserve(6)
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        if !self.turns.passed {
            self.work.push_back(Work::Priority {
                actor: super::turns::opponent(actor),
                passed: true,
                terminal: false,
            });
        } else {
            let effect = self
                .turns
                .effects
                .iter()
                .find(|(spell, _)| *spell == h)
                .map(|(_, e)| *e);
            let mut failed_draw = None;
            let resolution = if let Some(a) =
                self.turns.abilities.iter().find(|a| a.object == h).copied()
            {
                self.objects
                    .prepare_removals(1)
                    .map_err(TurnError::Storage)?;
                let legal = a.target.is_some_and(|t| self.haste_target(t));
                if a.power && self.haste_target(a.source) {
                    let old = self
                        .turns
                        .modifications
                        .iter()
                        .find(|m| m.handle == a.source)
                        .copied()
                        .unwrap_or(super::targets::Modification {
                            handle: a.source,
                            boost: 0,
                            power_boost: 0,
                            damage: 0,
                        });
                    let power_boost = old
                        .power_boost
                        .checked_add(1)
                        .filter(|b| {
                            b.checked_add(old.boost)
                                .and_then(|b| b.checked_add(5))
                                .is_some()
                        })
                        .ok_or(TurnError::EffectOverflow)?;
                    self.turns
                        .modifications
                        .try_reserve(1)
                        .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
                    self.work
                        .push_back(Work::Modify(super::targets::Modification {
                            power_boost,
                            ..old
                        }));
                } else if a.invoker && legal {
                    let target = a.target.unwrap();
                    let old = self
                        .turns
                        .modifications
                        .iter()
                        .find(|m| m.handle == target)
                        .copied()
                        .unwrap_or(super::targets::Modification {
                            handle: target,
                            boost: 0,
                            power_boost: 0,
                            damage: 0,
                        });
                    let (power, toughness) =
                        super::card_definitions::definition(self.objects.get(target).unwrap().card)
                            .creature_base()
                            .unwrap();
                    let boost = old
                        .boost
                        .checked_add(5)
                        .filter(|b| {
                            b.checked_add(power)
                                .and_then(|n| n.checked_add(old.power_boost))
                                .is_some()
                                && b.checked_add(toughness).is_some()
                        })
                        .ok_or(TurnError::EffectOverflow)?;
                    self.turns
                        .modifications
                        .try_reserve(1)
                        .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
                    self.turns
                        .trample
                        .try_reserve(1)
                        .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
                    self.work
                        .push_back(Work::Modify(super::targets::Modification { boost, ..old }));
                    self.work.push_back(Work::GrantTrample(target));
                } else if legal {
                    self.turns
                        .haste
                        .try_reserve(1)
                        .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
                    self.work.push_back(Work::GrantHaste(a.target.unwrap()));
                }
                self.work.push_back(Work::RemoveAbility(h));
                Some(super::targets::Resolution {
                    spell: self.objects.get(h).unwrap().card,
                    legal_targets: usize::from(legal),
                    resolved: a.power || legal,
                })
            } else if let Some(effect) = effect {
                let plan = self.prepare_effect(h, effect)?;
                if let Some(change) = plan.change {
                    self.work.push_back(Work::Modify(change));
                }
                for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
                    for &handle in &plan.moves[i] {
                        self.work.push_back(Work::SpellMove {
                            handle,
                            zone: Zone::Graveyard(seat),
                            controller: None,
                        });
                    }
                }
                Some(plan.resolution)
            } else if super::card_definitions::definition(self.objects.get(h).unwrap().card)
                .discard_draw()
            {
                let o = *self.objects.get(h).unwrap();
                let draws: Vec<_> = self
                    .objects
                    .in_zone(Zone::Library(o.controller))
                    .take(2)
                    .collect();
                self.objects
                    .prepare_moves(&draws, Zone::Hand(o.controller))
                    .map_err(TurnError::Storage)?;
                self.objects
                    .prepare_moves(&[h], Zone::Graveyard(o.owner))
                    .map_err(TurnError::Storage)?;
                if draws.len() < 2 {
                    failed_draw = Some(o.controller);
                }
                for handle in draws {
                    self.work.push_back(Work::SpellMove {
                        handle,
                        zone: Zone::Hand(o.controller),
                        controller: None,
                    });
                }
                self.work.push_back(Work::SpellMove {
                    handle: h,
                    zone: Zone::Graveyard(o.owner),
                    controller: None,
                });
                Some(super::targets::Resolution {
                    spell: o.card,
                    legal_targets: 0,
                    resolved: true,
                })
            } else if matches!(
                super::card_definitions::definition(self.objects.get(h).unwrap().card),
                super::card_definitions::Definition::TokenSorcery { .. }
            ) {
                let o = *self.objects.get(h).unwrap();
                self.objects
                    .reserve_knowledge(Zone::Battlefield, 3)
                    .map_err(TurnError::Storage)?;
                self.objects
                    .prepare_allocations(2, Zone::Battlefield)
                    .map_err(TurnError::Storage)?;
                self.objects
                    .prepare_moves(&[h], Zone::Graveyard(o.owner))
                    .map_err(TurnError::Storage)?;
                self.turns
                    .sick
                    .try_reserve(2)
                    .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
                self.work.push_back(Work::CreateGoblins {
                    controller: o.controller,
                });
                self.work.push_back(Work::SpellMove {
                    handle: h,
                    zone: Zone::Graveyard(o.owner),
                    controller: None,
                });
                Some(super::targets::Resolution {
                    spell: o.card,
                    legal_targets: 0,
                    resolved: true,
                })
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
                self.work.push_back(Work::SpellMove {
                    handle: h,
                    zone: Zone::Battlefield,
                    controller: Some(controller),
                });
                None
            };
            self.work.push_back(Work::FinishSpell {
                spell: h,
                resolution,
            });
            if failed_draw.is_some() {
                self.work
                    .push_back(Work::Turn(super::turns::TurnWork::Ready {
                        actor: self.turns.position.expect("turn").1,
                        kind: TurnKind::Priority,
                        failed_draw,
                    }));
            } else {
                self.work.push_back(Work::Priority {
                    actor: self.turns.position.expect("turn").1,
                    passed: false,
                    terminal: true,
                });
            }
        }
        self.turns.decision = None;
        self.generation = generation;
        Ok(())
    }
}
#[cfg(test)]
#[path = "casting_tests.rs"]
mod tests;
