//! Basic land actions and private, transactional mana-payment continuations.
//! Pool indices follow the fixture contract: W, U, B, R, G, C.
use super::turns::{Step, TurnDecision, TurnError, TurnKind};
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colorless,
}
impl Color {
    pub const ALL: [Self; 6] = [
        Self::White,
        Self::Blue,
        Self::Black,
        Self::Red,
        Self::Green,
        Self::Colorless,
    ];
    pub fn index(self) -> usize {
        self as usize
    }
}
/// Exact symbols (including C) and a generic amount. Rules code supplies costs;
/// this is not permission for a policy to invent a spell's cost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManaCost {
    pub colored: [u32; 6],
    pub generic: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentDecision {
    pub id: DecisionId,
    pub actor: Seat,
    pub choices: Vec<Color>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManaError {
    Turn(TurnError),
    IllegalLand,
    IllegalSource,
    InsufficientMana,
    IllegalPayment,
    PaymentPending,
    NoPayment,
    Overflow,
}
#[derive(Clone, Debug)]
pub(super) struct Payment {
    actor: Seat,
    id: DecisionId,
    remaining: ManaCost,
    pool: [u32; 6],
}
impl Payment {
    fn choices(&self) -> Vec<Color> {
        if let Some(i) = self.remaining.colored.iter().position(|&n| n > 0) {
            vec![Color::ALL[i]]
        } else if self.remaining.generic > 0 {
            Color::ALL
                .into_iter()
                .filter(|c| self.pool[c.index()] > 0)
                .collect()
        } else {
            vec![]
        }
    }
    fn decision(&self) -> PaymentDecision {
        PaymentDecision {
            id: self.id,
            actor: self.actor,
            choices: self.choices(),
        }
    }
}
fn basic_color(card: CardId) -> Option<Color> {
    match card.identity().key {
        "forest" => Some(Color::Green),
        "mountain" => Some(Color::Red),
        _ => None,
    }
}
fn invalid(e: ApplyError) -> ManaError {
    ManaError::Turn(TurnError::Invalid(e))
}
impl Game {
    fn mana_priority(&self, actor: Seat, id: DecisionId) -> Result<(), ManaError> {
        if self.turns.payment.is_some() {
            return Err(ManaError::PaymentPending);
        }
        let d = self
            .turn_decision()
            .ok_or(ManaError::Turn(TurnError::NotReady))?;
        if d.actor != actor {
            return Err(invalid(ApplyError::WrongActor));
        }
        if d.id != id {
            return Err(invalid(ApplyError::StaleDecision));
        }
        if d.kind != TurnKind::Priority {
            return Err(invalid(ApplyError::WrongKind));
        }
        Ok(())
    }
    fn next_mana_generation(&self) -> Result<u64, ManaError> {
        self.generation
            .checked_add(1)
            .ok_or(invalid(ApplyError::DecisionExhausted))
    }
    fn has_priority(&self, actor: Seat) -> bool {
        self.turn_decision()
            .is_some_and(|d| d.actor == actor && d.kind == TurnKind::Priority)
    }
    pub fn land_candidates(&self, actor: Seat) -> Vec<Handle> {
        if !self.has_priority(actor)
            || self.turns.land_used
            || !self.turns.position.is_some_and(|(_, active, step)| {
                active == actor && matches!(step, Step::PrecombatMain | Step::PostcombatMain)
            })
            || self.objects.in_zone(Zone::Stack).next().is_some()
        {
            return vec![];
        }
        self.objects
            .in_zone(Zone::Hand(actor))
            .filter(|&h| basic_color(self.objects.get(h).expect("live hand").card).is_some())
            .collect()
    }
    pub fn mana_sources(&self, actor: Seat) -> Vec<Handle> {
        if !self.has_priority(actor) {
            return vec![];
        }
        self.objects
            .in_zone(Zone::Battlefield)
            .filter(|&h| {
                let o = self.objects.get(h).expect("live permanent");
                o.controller == actor && !o.tapped && basic_color(o.card).is_some()
            })
            .collect()
    }
    pub fn play_land(
        &mut self,
        actor: Seat,
        id: DecisionId,
        land: Handle,
    ) -> Result<TurnDecision, ManaError> {
        self.mana_priority(actor, id)?;
        if !self.land_candidates(actor).contains(&land) {
            return Err(ManaError::IllegalLand);
        }
        let generation = self.next_mana_generation()?;
        self.objects
            .move_to(land, Zone::Battlefield)
            .map_err(|e| ManaError::Turn(TurnError::Storage(e)))?;
        self.turns.land_used = true;
        self.turns.passed = false;
        self.generation = generation;
        Ok(self.set_turn_decision(actor, TurnKind::Priority))
    }
    pub fn tap_mana(
        &mut self,
        actor: Seat,
        id: DecisionId,
        land: Handle,
    ) -> Result<TurnDecision, ManaError> {
        self.mana_priority(actor, id)?;
        if !self.mana_sources(actor).contains(&land) {
            return Err(ManaError::IllegalSource);
        }
        let color = basic_color(self.objects.get(land).expect("validated source").card)
            .unwrap()
            .index();
        let value = self.turns.mana[seat_index(actor)][color]
            .checked_add(1)
            .ok_or(ManaError::Overflow)?;
        let generation = self.next_mana_generation()?;
        self.objects.get_mut(land).expect("validated source").tapped = true;
        self.turns.mana[seat_index(actor)][color] = value;
        self.turns.passed = false;
        self.generation = generation;
        Ok(self.set_turn_decision(actor, TurnKind::Priority))
    }
    /// Engine integration boundary: reserve an affordable cost from already floated
    /// mana. The game is locked until explicit finish/cancel; no opponent priority
    /// or provisional pool mutation. Casting supplies the cost in the next layer.
    pub fn begin_payment(
        &mut self,
        actor: Seat,
        id: DecisionId,
        cost: ManaCost,
    ) -> Result<PaymentDecision, ManaError> {
        self.mana_priority(actor, id)?;
        let pool = self.turns.mana[seat_index(actor)];
        if (0..6).any(|i| cost.colored[i] > pool[i])
            || (0..6)
                .map(|i| u64::from(pool[i]) - u64::from(cost.colored[i]))
                .sum::<u64>()
                < u64::from(cost.generic)
        {
            return Err(ManaError::InsufficientMana);
        }
        let generation = self.next_mana_generation()?;
        self.generation = generation;
        let p = Payment {
            actor,
            id: DecisionId {
                scope: self.objects.scope(),
                generation,
            },
            remaining: cost,
            pool,
        };
        let d = p.decision();
        self.turns.payment = Some(p);
        Ok(d)
    }
    pub fn payment_decision(&self, actor: Seat) -> Option<PaymentDecision> {
        self.turns
            .payment
            .as_ref()
            .filter(|p| p.actor == actor)
            .map(Payment::decision)
    }
    fn validate_payment(&self, actor: Seat, id: DecisionId) -> Result<&Payment, ManaError> {
        let p = self.turns.payment.as_ref().ok_or(ManaError::NoPayment)?;
        if actor != p.actor {
            return Err(invalid(ApplyError::WrongActor));
        }
        if id != p.id {
            return Err(invalid(ApplyError::StaleDecision));
        }
        Ok(p)
    }
    /// Choose one unit; exact symbols are paid first, then any generic colors.
    /// This canonical order preserves every final legal payment without flattening
    /// all combinations into a single candidate list. Even the last unit stays
    /// provisional until finish_payment.
    pub fn choose_payment(
        &mut self,
        actor: Seat,
        id: DecisionId,
        color: Color,
    ) -> Result<PaymentDecision, ManaError> {
        let p = self.validate_payment(actor, id)?;
        if !p.choices().contains(&color) {
            return Err(ManaError::IllegalPayment);
        }
        let generation = self.next_mana_generation()?;
        let p = self.turns.payment.as_mut().unwrap();
        let i = color.index();
        p.pool[i] -= 1;
        if p.remaining.colored[i] > 0 {
            p.remaining.colored[i] -= 1;
        } else {
            p.remaining.generic -= 1;
        }
        p.id.generation = generation;
        self.generation = generation;
        Ok(p.decision())
    }
    /// Discard provisional choices. Previously activated mana abilities remain real.
    pub fn cancel_payment(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<TurnDecision, ManaError> {
        self.validate_payment(actor, id)?;
        let generation = self.next_mana_generation()?;
        self.turns.payment = None;
        self.generation = generation;
        Ok(self.set_turn_decision(actor, TurnKind::Priority))
    }
    /// Commit the complete payment exactly once. No response window is introduced.
    /// The casting layer must join this with its own preflighted commit, without
    /// returning an intermediate priority decision to a policy.
    pub fn finish_payment(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<TurnDecision, ManaError> {
        let p = self.validate_payment(actor, id)?;
        if p.remaining.generic > 0 || p.remaining.colored.iter().any(|&n| n > 0) {
            return Err(ManaError::IllegalPayment);
        }
        let pool = p.pool;
        let generation = self.next_mana_generation()?;
        self.turns.mana[seat_index(actor)] = pool;
        self.turns.payment = None;
        self.turns.passed = false;
        self.generation = generation;
        Ok(self.set_turn_decision(actor, TurnKind::Priority))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ready() -> Game {
        let mut g = Game::new().unwrap();
        g.reset(&Config::default(), 42, 9).unwrap();
        while let Some(d) = g.decision() {
            g.apply(
                d.actor,
                &OpeningAction {
                    decision: d.id,
                    selection: Selection::Choose(d.candidate(0)),
                },
            )
            .unwrap();
        }
        g.start_turns().unwrap();
        g
    }
    #[test]
    fn mana_payment_two_units_are_private_until_commit() {
        // Declared synthetic pool; CR 107.4: 1G can use G plus R or another G.
        let mut g = ready();
        g.turns.mana[0] = [0, 0, 0, 1, 2, 0];
        let d = g.turn_decision().unwrap();
        let p = g.begin_payment(
            Seat::P0,
            d.id,
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
            },
        );
        assert!(p.is_ok(), "1G is payable from RGG under CR 107.4");
        let p = p.unwrap();
        assert_eq!(p.choices, vec![Color::Green]);
        g.choose_payment(Seat::P0, p.id, Color::Green).unwrap();
        let before = format!("{g:?}");
        assert_eq!(
            g.choose_payment(Seat::P0, p.id, Color::Red),
            Err(invalid(ApplyError::StaleDecision))
        );
        assert_eq!(format!("{g:?}"), before);
        let p = g.payment_decision(Seat::P0).unwrap();
        assert_eq!(p.choices, vec![Color::Red, Color::Green]);
        assert_eq!(g.mana()[0], [0, 0, 0, 1, 2, 0]);
        g.choose_payment(Seat::P0, p.id, Color::Red).unwrap();
        let p = g.payment_decision(Seat::P0).unwrap();
        g.finish_payment(Seat::P0, p.id).unwrap();
        assert_eq!(g.mana()[0], [0, 0, 0, 0, 1, 0]);
    }
    #[test]
    fn mana_synthetic_illegal_sources_stack_and_overflow() {
        let mut g = ready();
        let forest = CardId::from_key("forest").unwrap();
        let cub = CardId::from_key("bear-cub").unwrap();
        let own = g
            .objects
            .allocate(forest, Seat::P0, Zone::Battlefield)
            .unwrap();
        let enemy = g
            .objects
            .allocate(forest, Seat::P1, Zone::Battlefield)
            .unwrap();
        let creature = g
            .objects
            .allocate(cub, Seat::P0, Zone::Battlefield)
            .unwrap();
        let hand = g
            .objects
            .allocate(forest, Seat::P0, Zone::Hand(Seat::P0))
            .unwrap();
        let d = g.turn_decision().unwrap();
        for h in [enemy, creature, hand] {
            let before = format!("{g:?}");
            assert_eq!(g.tap_mana(Seat::P0, d.id, h), Err(ManaError::IllegalSource));
            assert_eq!(format!("{g:?}"), before);
        }
        g.turns.mana[0][4] = u32::MAX;
        let before = format!("{g:?}");
        assert_eq!(g.tap_mana(Seat::P0, d.id, own), Err(ManaError::Overflow));
        assert_eq!(format!("{g:?}"), before);
        g.turns.mana[0][4] = 0;
        g.generation = u64::MAX;
        let before = format!("{g:?}");
        assert_eq!(
            g.tap_mana(Seat::P0, d.id, own),
            Err(invalid(ApplyError::DecisionExhausted))
        );
        assert_eq!(format!("{g:?}"), before);
        g.generation = d.id.generation;
        g.turns.position = Some((1, Seat::P0, Step::PrecombatMain));
        let stack = g.objects.allocate(cub, Seat::P0, Zone::Stack).unwrap();
        let before = format!("{g:?}");
        assert_eq!(
            g.play_land(Seat::P0, d.id, hand),
            Err(ManaError::IllegalLand)
        );
        assert_eq!(format!("{g:?}"), before);
        // A mana ability itself never uses or resolves the existing stack.
        g.tap_mana(Seat::P0, d.id, own).unwrap();
        assert!(g.objects.get(stack).is_ok());
        assert_eq!(g.mana()[0][4], 1);
        let d = g.turn_decision().unwrap();
        let p = g
            .begin_payment(
                Seat::P0,
                d.id,
                ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                },
            )
            .unwrap();
        g.generation = u64::MAX;
        let before = format!("{g:?}");
        assert!(g.choose_payment(Seat::P0, p.id, Color::Green).is_err());
        assert!(g.cancel_payment(Seat::P0, p.id).is_err());
        assert_eq!(format!("{g:?}"), before);
    }
    // Independent Cartesian spent-vector enumerator, not the engine's color-first
    // decision traversal: CR 107.4 requires each colored amount plus exact total.
    fn expected(pool: [u32; 6], cost: ManaCost) -> std::collections::BTreeSet<[u32; 6]> {
        let mut out = std::collections::BTreeSet::new();
        if cost.generic + cost.colored.iter().sum::<u32>() > pool.iter().sum::<u32>() {
            return out;
        }
        for code in 0..729 {
            let mut n = code;
            let mut spent = [0; 6];
            for x in &mut spent {
                *x = n % 3;
                n /= 3;
            }
            if (0..6).all(|i| spent[i] <= pool[i] && spent[i] >= cost.colored[i])
                && spent.iter().sum::<u32>() == cost.generic + cost.colored.iter().sum::<u32>()
            {
                out.insert(std::array::from_fn(|i| pool[i] - spent[i]));
            }
        }
        out
    }
    fn traverse(g: &mut Game, out: &mut std::collections::BTreeSet<[u32; 6]>) {
        let d = g.payment_decision(Seat::P0).unwrap();
        if d.choices.is_empty() {
            g.finish_payment(Seat::P0, d.id).unwrap();
            out.insert(g.mana()[0]);
            return;
        }
        for c in d.choices {
            let saved = g.turns.clone();
            let generation = g.generation;
            g.choose_payment(Seat::P0, d.id, c).unwrap();
            traverse(g, out);
            g.turns = saved;
            g.generation = generation;
        }
    }
    #[test]
    fn mana_all_small_payment_combinations_independent_enumerator() {
        let mut g = ready();
        let base = g.turns.clone();
        let generation = g.generation;
        for code in 0..729 {
            let mut n = code;
            let mut pool = [0; 6];
            for x in &mut pool {
                *x = n % 3;
                n /= 3;
            }
            if pool.iter().sum::<u32>() > 4 {
                continue;
            }
            for mask in 0..729 {
                let mut digits = mask;
                let colored = std::array::from_fn(|_| {
                    let n = digits % 3;
                    digits /= 3;
                    n
                });
                if colored.iter().sum::<u32>() > 4 {
                    continue;
                }
                for generic in 0..=4 {
                    let cost = ManaCost { colored, generic };
                    let want = expected(pool, cost);
                    g.turns = base.clone();
                    g.generation = generation;
                    g.turns.mana[0] = pool;
                    let before = format!("{g:?}");
                    let d = g.turn_decision().unwrap();
                    let result = g.begin_payment(Seat::P0, d.id, cost);
                    if want.is_empty() {
                        assert_eq!(result, Err(ManaError::InsufficientMana));
                        assert_eq!(format!("{g:?}"), before);
                    } else {
                        assert!(result.is_ok(), "pool={pool:?} cost={cost:?}");
                        let mut actual = std::collections::BTreeSet::new();
                        traverse(&mut g, &mut actual);
                        assert_eq!(actual, want, "pool={pool:?} cost={cost:?}");
                    }
                }
            }
        }
    }
}
