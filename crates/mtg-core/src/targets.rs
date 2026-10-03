//! Private factored target selection and the M1 Growth/Bite effects.
use super::card_definitions::{InstantEffect, definition};
use super::casting::CastError;
use super::mana::PaymentDecision;
use super::turns::{TurnDecision, TurnError};
use super::*;
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetKind {
    Growth,
    BiteSource,
    BiteDestination,
    Complete,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TargetDecision {
    pub id: DecisionId,
    pub actor: Seat,
    pub kind: TargetKind,
    pub choices: Vec<Handle>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetError {
    Cast(CastError),
    Invalid(ApplyError),
    NoTargets,
    MissingTargets,
    IllegalTarget,
    CapacityExceeded { needed: usize, capacity: usize },
}
pub(super) use super::cast_state::Targeting;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatureState {
    pub power: u32,
    pub toughness: u32,
    pub damage: u32,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub spell: CardId,
    pub legal_targets: usize,
    pub resolved: bool,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub(super) enum Effect {
    Growth(Handle),
    Bite(Handle, Handle),
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug)]
pub(super) struct Modification {
    pub(super) handle: Handle,
    pub(super) boost: u32,
    pub(super) power_boost: u32,
    pub(super) damage: u32,
}
pub(super) fn instant(card: CardId) -> bool {
    definition(card).instant_effect().is_some()
}
fn base(card: CardId) -> Option<(u32, u32)> {
    definition(card).creature_base()
}
pub(super) struct PreparedEffect {
    pub change: Option<Modification>,
    pub moves: [Vec<Handle>; 2],
    pub resolution: Resolution,
}
impl Game {
    pub fn creature_state(&self, h: Handle) -> Option<CreatureState> {
        if self.turns.abilities.iter().any(|a| a.object == h) {
            return None;
        }
        let o = self.objects.get(h).ok()?;
        let (power, toughness) = base(o.card)?;
        let m = self
            .turns
            .modifications
            .iter()
            .find(|m| m.handle == h && o.zone == Zone::Battlefield);
        let (boost, damage) = m.map_or((0, 0), |m| (m.boost, m.damage));
        Some(CreatureState {
            power: power + boost + m.map_or(0, |m| m.power_boost),
            toughness: toughness + boost,
            damage,
        })
    }
    fn creature(&self, h: Handle) -> bool {
        self.objects
            .get(h)
            .is_ok_and(|o| o.zone == Zone::Battlefield && base(o.card).is_some())
    }
    pub(super) fn target_lists(&self, actor: Seat, card: CardId) -> (Vec<Handle>, Vec<Handle>) {
        let creatures = self
            .objects
            .in_zone(Zone::Battlefield)
            .filter(|h| self.creature(*h));
        if definition(card).instant_effect() == Some(InstantEffect::Growth) {
            (creatures.collect(), vec![])
        } else {
            creatures.partition(|h| self.objects.get(*h).unwrap().controller == actor)
        }
    }
    pub(super) fn has_targets(&self, actor: Seat, card: CardId) -> bool {
        let (a, b) = self.target_lists(actor, card);
        !a.is_empty()
            && (definition(card).instant_effect() == Some(InstantEffect::Growth) || !b.is_empty())
    }
    pub fn begin_targeted_cast(
        &mut self,
        actor: Seat,
        id: DecisionId,
        card: Handle,
        capacity: usize,
    ) -> Result<TargetDecision, TargetError> {
        self.mana_priority(actor, id)
            .map_err(|e| TargetError::Cast(CastError::Mana(e)))?;
        if !self.cast_candidates(actor).contains(&card)
            || !instant(
                self.objects
                    .get(card)
                    .map_err(|_| TargetError::IllegalTarget)?
                    .card,
            )
        {
            return Err(TargetError::Cast(CastError::IllegalSpell));
        }
        let c = self.objects.get(card).unwrap().card;
        let (choices, destinations) = self.target_lists(actor, c);
        let needed = choices.len().max(destinations.len());
        if needed > capacity {
            return Err(TargetError::CapacityExceeded { needed, capacity });
        }
        let generation = self
            .next_mana_generation()
            .map_err(|e| TargetError::Cast(CastError::Mana(e)))?;
        let d = TargetDecision {
            id: DecisionId {
                scope: self.objects.scope(),
                generation,
            },
            actor,
            kind: if definition(c).instant_effect() == Some(InstantEffect::Growth) {
                TargetKind::Growth
            } else {
                TargetKind::BiteSource
            },
            choices,
        };
        self.start_targeting(card, d.clone(), destinations);
        Ok(d)
    }
    pub fn target_decision(&self, actor: Seat) -> Option<TargetDecision> {
        self.turns
            .targeting
            .as_ref()
            .filter(|t| t.decision().actor == actor)
            .map(|t| t.decision().clone())
    }
    fn validate_targets(&self, actor: Seat, id: DecisionId) -> Result<&Targeting, TargetError> {
        let t = self
            .turns
            .targeting
            .as_ref()
            .ok_or(TargetError::NoTargets)?;
        if actor != t.decision().actor {
            return Err(TargetError::Invalid(ApplyError::WrongActor));
        }
        if id != t.decision().id {
            return Err(TargetError::Invalid(ApplyError::StaleDecision));
        }
        Ok(t)
    }
    fn legal_target(&self, actor: Seat, kind: TargetKind, h: Handle) -> bool {
        self.creature(h)
            && match kind {
                TargetKind::Growth => true,
                TargetKind::BiteSource => self.objects.get(h).unwrap().controller == actor,
                TargetKind::BiteDestination => self.objects.get(h).unwrap().controller != actor,
                TargetKind::Complete => false,
            }
    }
    pub fn choose_target(
        &mut self,
        actor: Seat,
        id: DecisionId,
        h: Handle,
    ) -> Result<TargetDecision, TargetError> {
        let t = self.validate_targets(actor, id)?;
        if !t.decision().choices.contains(&h) || !self.legal_target(actor, t.decision().kind, h) {
            return Err(TargetError::IllegalTarget);
        }
        let generation = self
            .next_mana_generation()
            .map_err(|e| TargetError::Cast(CastError::Mana(e)))?;
        Ok(self.select_cast_target(h, generation))
    }
    pub fn cancel_targets(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<TurnDecision, TargetError> {
        self.validate_targets(actor, id)?;
        let generation = self
            .next_mana_generation()
            .map_err(|e| TargetError::Cast(CastError::Mana(e)))?;
        Ok(self.cancel_cast_targets(actor, generation))
    }
    pub fn finish_targets(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<PaymentDecision, TargetError> {
        let t = self.validate_targets(actor, id)?;
        if t.decision().kind != TargetKind::Complete {
            return Err(TargetError::MissingTargets);
        }
        let card = t.card();
        let c = self
            .objects
            .get(card)
            .map_err(|_| TargetError::IllegalTarget)?;
        if c.zone != Zone::Hand(actor) {
            return Err(TargetError::IllegalTarget);
        }
        let effect = match t.selected() {
            [a] => Effect::Growth(*a),
            [a, b] => Effect::Bite(*a, *b),
            _ => return Err(TargetError::MissingTargets),
        };
        if self.legal_effect_targets(actor, effect) != t.selected().len() {
            return Err(TargetError::IllegalTarget);
        }
        let remaining =
            super::casting::cost(c.card).ok_or(TargetError::Cast(CastError::IllegalSpell))?;
        let generation = self
            .next_mana_generation()
            .map_err(|e| TargetError::Cast(CastError::Mana(e)))?;
        Ok(self.start_cast_payment(actor, generation, card, remaining, Some(effect)))
    }
    pub fn stack_targets(&self, h: Handle) -> Option<Vec<Handle>> {
        if let Some(a) = self.turns.abilities.iter().find(|a| a.object == h) {
            return Some(a.target.into_iter().collect());
        }
        self.turns
            .effects
            .iter()
            .find(|(spell, _)| *spell == h)
            .map(|(_, e)| match e {
                Effect::Growth(a) => vec![*a],
                Effect::Bite(a, b) => vec![*a, *b],
            })
    }
    pub fn last_resolution(&self) -> Option<Resolution> {
        self.turns.last_resolution
    }
    pub(super) fn legal_effect_targets(&self, actor: Seat, e: Effect) -> usize {
        match e {
            Effect::Growth(a) => usize::from(self.legal_target(actor, TargetKind::Growth, a)),
            Effect::Bite(a, b) => {
                usize::from(self.legal_target(actor, TargetKind::BiteSource, a))
                    + usize::from(self.legal_target(actor, TargetKind::BiteDestination, b))
            }
        }
    }
    pub(super) fn prepare_effect(
        &mut self,
        h: Handle,
        e: Effect,
    ) -> Result<PreparedEffect, TurnError> {
        let o = *self.objects.get(h).map_err(TurnError::Storage)?;
        let legal = self.legal_effect_targets(o.controller, e);
        let mut change = None;
        let mut deathtouch_damage = false;
        match e {
            Effect::Growth(a) if legal == 1 => {
                let old = self
                    .turns
                    .modifications
                    .iter()
                    .find(|m| m.handle == a)
                    .copied()
                    .unwrap_or(Modification {
                        handle: a,
                        power_boost: 0,
                        boost: 0,
                        damage: 0,
                    });
                let boost = old
                    .boost
                    .checked_add(3)
                    .filter(|b| {
                        let (power, toughness) = base(self.objects.get(a).unwrap().card).unwrap();
                        b.checked_add(old.power_boost)
                            .and_then(|b| b.checked_add(power))
                            .is_some()
                            && b.checked_add(toughness).is_some()
                    })
                    .ok_or(TurnError::EffectOverflow)?;
                change = Some(Modification { boost, ..old });
            }
            Effect::Bite(a, b) if legal == 2 => {
                let old = self
                    .turns
                    .modifications
                    .iter()
                    .find(|m| m.handle == b)
                    .copied()
                    .unwrap_or(Modification {
                        handle: b,
                        power_boost: 0,
                        boost: 0,
                        damage: 0,
                    });
                let power = self.creature_state(a).unwrap().power;
                deathtouch_damage = power > 0 && self.has_deathtouch(a);
                let damage = old
                    .damage
                    .checked_add(power)
                    .ok_or(TurnError::EffectOverflow)?;
                change = Some(Modification { damage, ..old });
            }
            _ => {}
        }
        // Preflight all zone moves and numeric/resource bounds before effects,
        // death, or spell-zone changes become observable (CR 608 / 704.5g).
        let dead = change
            .filter(|m| {
                let (_, t) = base(self.objects.get(m.handle).unwrap().card).unwrap();
                m.damage >= t + m.boost || deathtouch_damage
            })
            .map(|m| m.handle);
        let mut moves = [vec![], vec![]];
        moves[seat_index(o.owner)].push(h);
        if let Some(dead) = dead {
            moves[seat_index(self.objects.get(dead).unwrap().owner)].push(dead);
        }
        for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            self.objects
                .prepare_moves(&moves[i], Zone::Graveyard(seat))
                .map_err(TurnError::Storage)?;
        }
        self.objects
            .prepare_removals(usize::from(dead.is_some()))
            .map_err(TurnError::Storage)?;
        self.turns
            .modifications
            .try_reserve(1)
            .map_err(|_| TurnError::Storage(StorageError::CapacityExceeded))?;
        Ok(PreparedEffect {
            change,
            moves,
            resolution: Resolution {
                spell: o.card,
                legal_targets: legal,
                resolved: legal > 0,
            },
        })
    }
}
