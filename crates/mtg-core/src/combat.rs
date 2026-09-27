//! Vanilla combat declarations and simultaneous damage. Inspection is privileged.
use super::turns::{TurnDecision, TurnError};
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatKind {
    Attackers,
    Blockers,
    Damage,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attack {
    pub creature: Handle,
    pub blocked: bool,
    pub blockers: Vec<Handle>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DamageChoice {
    pub attacker: Handle,
    pub power: u32,
    pub blockers: Vec<Handle>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CombatDecision {
    pub id: DecisionId,
    pub actor: Seat,
    pub kind: CombatKind,
    pub attackers: Vec<Handle>,
    pub blockers: Vec<Handle>,
    pub selected: Vec<Handle>,
    pub blocks: Vec<(Handle, Handle)>,
    pub damage: Vec<DamageChoice>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatError {
    Invalid(ApplyError),
    NotReady,
    IllegalAttacker,
    IllegalBlocker,
    IllegalDamage,
    MissingDamage,
    CapacityExceeded { needed: usize, capacity: usize },
    Turn(TurnError),
}
#[derive(Clone, Debug, Default)]
pub(super) struct CombatState {
    pub(super) attacks: Vec<Attack>,
    selected: Vec<Handle>,
    blocks: Vec<(Handle, Handle)>,
    assignments: Vec<(Handle, Vec<(Handle, u32)>)>,
}
fn vanilla(card: CardId) -> bool {
    matches!(card.identity().key, "bear-cub" | "swab-goblin")
}
impl Game {
    fn combat_live(&self, h: Handle, controller: Seat) -> bool {
        self.objects.get(h).is_ok_and(|o| {
            o.zone == Zone::Battlefield && o.controller == controller && vanilla(o.card)
        })
    }
    pub(super) fn supported_combat(&self) -> bool {
        self.objects.in_zone(Zone::Battlefield).all(|h| {
            let c = self.objects.get(h).unwrap().card;
            vanilla(c) || super::mana::basic_color(c).is_some()
        })
    }
    pub(super) fn has_combat_creature(&self, active: Seat) -> bool {
        self.objects
            .in_zone(Zone::Battlefield)
            .any(|h| self.combat_live(h, active))
    }
    /// Current committed combat only, excluding departed/changed-controller
    /// objects. Blocked status is remembered independently of surviving blockers.
    pub fn combat(&self) -> Vec<Attack> {
        let Some((_, active, _)) = self.turns.position else {
            return vec![];
        };
        self.turns
            .combat
            .attacks
            .iter()
            .filter(|a| self.combat_live(a.creature, active))
            .map(|a| Attack {
                creature: a.creature,
                blocked: a.blocked,
                blockers: a
                    .blockers
                    .iter()
                    .copied()
                    .filter(|b| self.combat_live(*b, super::turns::opponent(active)))
                    .collect(),
            })
            .collect()
    }
    fn combat_guard(
        &self,
        actor: Seat,
        id: DecisionId,
        kind: CombatKind,
    ) -> Result<(), CombatError> {
        let d = self.turn_decision().ok_or(CombatError::NotReady)?;
        if actor != d.actor {
            return Err(CombatError::Invalid(ApplyError::WrongActor));
        }
        if id != d.id {
            return Err(CombatError::Invalid(ApplyError::StaleDecision));
        }
        if d.kind != super::turns::TurnKind::Combat(kind) {
            return Err(CombatError::NotReady);
        }
        Ok(())
    }
    fn combat_generation(&self) -> Result<u64, CombatError> {
        self.generation
            .checked_add(1)
            .ok_or(CombatError::Invalid(ApplyError::DecisionExhausted))
    }
    fn combat_update(&mut self, actor: Seat, generation: u64, kind: CombatKind) -> TurnDecision {
        self.generation = generation;
        self.set_turn_decision(actor, super::turns::TurnKind::Combat(kind))
    }
    pub fn combat_decision(
        &self,
        actor: Seat,
        capacity: usize,
    ) -> Result<CombatDecision, CombatError> {
        let d = self.turn_decision().ok_or(CombatError::NotReady)?;
        let super::turns::TurnKind::Combat(kind) = d.kind else {
            return Err(CombatError::NotReady);
        };
        self.combat_guard(actor, d.id, kind)?;
        let active = self.turns.position.unwrap().1;
        let attacks = self.combat();
        let attackers = if kind == CombatKind::Attackers {
            self.objects
                .in_zone(Zone::Battlefield)
                .filter(|h| {
                    self.combat_live(*h, active)
                        && !self.objects.get(*h).unwrap().tapped
                        && !self.summoning_sick(*h)
                })
                .collect()
        } else {
            attacks.iter().map(|a| a.creature).collect::<Vec<_>>()
        };
        let blockers = if kind == CombatKind::Blockers {
            self.objects
                .in_zone(Zone::Battlefield)
                .filter(|h| self.combat_live(*h, actor) && !self.objects.get(*h).unwrap().tapped)
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let damage = if kind == CombatKind::Damage {
            attacks
                .iter()
                .filter(|a| a.blockers.len() > 1)
                .map(|a| DamageChoice {
                    attacker: a.creature,
                    power: self.creature_state(a.creature).unwrap().power,
                    blockers: a.blockers.clone(),
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        let needed = attackers
            .len()
            .max(blockers.len())
            .max(damage.iter().map(|d| d.blockers.len()).max().unwrap_or(0));
        if needed > capacity {
            return Err(CombatError::CapacityExceeded { needed, capacity });
        }
        Ok(CombatDecision {
            id: d.id,
            actor,
            kind,
            attackers,
            blockers,
            selected: self.turns.combat.selected.clone(),
            blocks: self.turns.combat.blocks.clone(),
            damage,
        })
    }
    fn validate_attackers(&self, actor: Seat, cards: &[Handle]) -> Result<(), CombatError> {
        for (i, h) in cards.iter().enumerate() {
            if cards[..i].contains(h) {
                return Err(CombatError::Invalid(ApplyError::DuplicateCandidate));
            }
            if !self.combat_live(*h, actor)
                || self.objects.get(*h).unwrap().tapped
                || self.summoning_sick(*h)
            {
                return Err(CombatError::IllegalAttacker);
            }
        }
        Ok(())
    }
    /// Replace the provisional subset. No powerset is generated and nothing taps
    /// until finish. An empty subset is an explicit legal declaration.
    pub fn select_attackers(
        &mut self,
        actor: Seat,
        id: DecisionId,
        cards: &[Handle],
    ) -> Result<TurnDecision, CombatError> {
        self.combat_guard(actor, id, CombatKind::Attackers)?;
        self.validate_attackers(actor, cards)?;
        let generation = self.combat_generation()?;
        self.turns.combat.selected = cards.to_vec();
        Ok(self.combat_update(actor, generation, CombatKind::Attackers))
    }
    fn validate_blocks(&self, actor: Seat, blocks: &[(Handle, Handle)]) -> Result<(), CombatError> {
        let attacks = self.combat();
        for (i, (b, a)) in blocks.iter().enumerate() {
            if blocks[..i].iter().any(|(other, _)| other == b) {
                return Err(CombatError::Invalid(ApplyError::DuplicateCandidate));
            }
            if !self.combat_live(*b, actor)
                || self.objects.get(*b).unwrap().tapped
                || !attacks.iter().any(|x| x.creature == *a)
            {
                return Err(CombatError::IllegalBlocker);
            }
        }
        Ok(())
    }
    /// Replace a provisional blocker-to-attacker map, with at most one attacker
    /// per blocker. Many blockers may choose the same attacker; no ordering.
    pub fn select_blockers(
        &mut self,
        actor: Seat,
        id: DecisionId,
        blocks: &[(Handle, Handle)],
    ) -> Result<TurnDecision, CombatError> {
        self.combat_guard(actor, id, CombatKind::Blockers)?;
        self.validate_blocks(actor, blocks)?;
        let generation = self.combat_generation()?;
        self.turns.combat.blocks = blocks.to_vec();
        Ok(self.combat_update(actor, generation, CombatKind::Blockers))
    }
    fn validate_damage(
        &self,
        attacker: Handle,
        amounts: &[(Handle, u32)],
    ) -> Result<(), CombatError> {
        let attacks = self.combat();
        let a = attacks
            .iter()
            .find(|a| a.creature == attacker)
            .ok_or(CombatError::IllegalDamage)?;
        if a.blockers.len() < 2 {
            return Err(CombatError::IllegalDamage);
        }
        let mut total = 0u64;
        for (i, (b, n)) in amounts.iter().enumerate() {
            if !a.blockers.contains(b) || amounts[..i].iter().any(|(other, _)| other == b) {
                return Err(CombatError::IllegalDamage);
            }
            total = total
                .checked_add(u64::from(*n))
                .ok_or(CombatError::IllegalDamage)?;
        }
        if total != u64::from(self.creature_state(attacker).unwrap().power) {
            return Err(CombatError::IllegalDamage);
        }
        Ok(())
    }
    /// Assign one multiply-blocked attacker's entire power. Omitted recipients
    /// get zero; any division is legal, with no obsolete lethal-first ordering.
    pub fn assign_combat_damage(
        &mut self,
        actor: Seat,
        id: DecisionId,
        attacker: Handle,
        amounts: &[(Handle, u32)],
    ) -> Result<TurnDecision, CombatError> {
        self.combat_guard(actor, id, CombatKind::Damage)?;
        self.validate_damage(attacker, amounts)?;
        let generation = self.combat_generation()?;
        self.turns
            .combat
            .assignments
            .retain(|(h, _)| *h != attacker);
        self.turns
            .combat
            .assignments
            .push((attacker, amounts.to_vec()));
        Ok(self.combat_update(actor, generation, CombatKind::Damage))
    }
    pub fn finish_combat(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<TurnDecision, CombatError> {
        let d = self.turn_decision().ok_or(CombatError::NotReady)?;
        let super::turns::TurnKind::Combat(kind) = d.kind else {
            return Err(CombatError::NotReady);
        };
        self.combat_guard(actor, id, kind)?;
        let generation = self.combat_generation()?;
        match kind {
            CombatKind::Attackers => {
                self.validate_attackers(actor, &self.turns.combat.selected)?;
                let attacks = self
                    .turns
                    .combat
                    .selected
                    .iter()
                    .map(|h| Attack {
                        creature: *h,
                        blocked: false,
                        blockers: vec![],
                    })
                    .collect();
                for h in &self.turns.combat.selected {
                    self.objects.get_mut(*h).unwrap().tapped = true;
                }
                self.turns.combat.attacks = attacks;
                self.turns.combat.selected.clear();
            }
            CombatKind::Blockers => {
                self.validate_blocks(actor, &self.turns.combat.blocks)?;
                let mut attacks = self.combat();
                for (b, h) in &self.turns.combat.blocks {
                    let a = attacks.iter_mut().find(|a| a.creature == *h).unwrap();
                    a.blocked = true;
                    a.blockers.push(*b);
                }
                self.turns.combat.attacks = attacks;
                self.turns.combat.blocks.clear();
            }
            CombatKind::Damage => self.deal_combat_damage()?,
        }
        self.generation = generation;
        self.turns.passed = false;
        Ok(self.set_turn_decision(
            self.turns.position.unwrap().1,
            super::turns::TurnKind::Priority,
        ))
    }
    fn deal_combat_damage(&mut self) -> Result<(), CombatError> {
        use super::targets::Modification;
        let mut changes = self.turns.modifications.clone();
        let mut life = self.life;
        let active = self.turns.position.unwrap().1;
        let defender = seat_index(super::turns::opponent(active));
        let overflow = CombatError::Turn(TurnError::EffectOverflow);
        let mut mark = |h: Handle, amount: u32| -> Result<(), CombatError> {
            if let Some(m) = changes.iter_mut().find(|m| m.handle == h) {
                m.damage = m.damage.checked_add(amount).ok_or(overflow)?;
            } else {
                changes.push(Modification {
                    handle: h,
                    boost: 0,
                    damage: amount,
                });
            }
            Ok(())
        };
        // Gather every assignment and all damage before any SBA or mutation.
        for a in self.combat() {
            let power = self.creature_state(a.creature).unwrap().power;
            if !a.blocked {
                life[defender] = life[defender]
                    .checked_sub(i64::from(power))
                    .ok_or(overflow)?;
            } else if a.blockers.len() == 1 {
                mark(a.blockers[0], power)?;
            } else if a.blockers.len() > 1 {
                let amounts = &self
                    .turns
                    .combat
                    .assignments
                    .iter()
                    .find(|(h, _)| *h == a.creature)
                    .ok_or(CombatError::MissingDamage)?
                    .1;
                self.validate_damage(a.creature, amounts)?;
                for (h, n) in amounts {
                    mark(*h, *n)?;
                }
            } // blocked with no remaining blocker deals no damage (510.1c).
            for b in a.blockers {
                mark(a.creature, self.creature_state(b).unwrap().power)?;
            }
        }
        let mut dead = [vec![], vec![]];
        for m in &changes {
            if let Ok(o) = self.objects.get(m.handle)
                && o.zone == Zone::Battlefield
                && self
                    .creature_state(m.handle)
                    .is_some_and(|c| m.damage >= c.toughness)
            {
                dead[seat_index(o.owner)].push(m.handle);
            }
        }
        for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            self.objects
                .prepare_moves(&dead[i], Zone::Graveyard(seat))
                .map_err(|e| CombatError::Turn(TurnError::Storage(e)))?;
        }
        self.life = life;
        self.turns.modifications = changes;
        for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            for h in &dead[i] {
                self.objects
                    .move_to(*h, Zone::Graveyard(seat))
                    .expect("preflighted combat deaths");
            }
        }
        self.turns.modifications.retain(|m| {
            self.objects
                .get(m.handle)
                .is_ok_and(|o| o.zone == Zone::Battlefield)
        });
        self.turns.combat.assignments.clear();
        Ok(())
    }
}
#[cfg(test)]
#[path = "combat_tests.rs"]
mod tests;
