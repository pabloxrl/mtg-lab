//! Internal boundary for private cast, target and payment continuations.
//!
//! Rule entry points validate legality and preflight generations/storage before
//! these transitions. Only this module mutates the continuation fields. The
//! existing TurnState slots and serialized field names/order remain unchanged.
use super::mana::{Color, ManaCost, ManaError, PaymentDecision};
use super::targets::{Effect, TargetDecision, TargetKind};
use super::turns::{TurnDecision, TurnKind};
use super::*;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(super) struct PendingCast {
    card: Handle,
    sources: Vec<Handle>,
    discard: Option<Handle>,
    mode: Option<u8>,
    effect: Option<super::targets::Effect>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(super) struct Payment {
    actor: Seat,
    id: DecisionId,
    remaining: ManaCost,
    pool: [u32; 6],
}
impl Payment {
    pub(super) fn choices(&self) -> Vec<Color> {
        if let Some(i) = self.remaining.colored.iter().position(|&n| n > 0) {
            if self.pool[i] > 0 {
                vec![Color::ALL[i]]
            } else {
                vec![]
            }
        } else if self.remaining.generic > 0 {
            Color::ALL
                .into_iter()
                .filter(|c| self.pool[c.index()] > 0)
                .collect()
        } else {
            vec![]
        }
    }
    pub(super) fn decision(&self) -> PaymentDecision {
        PaymentDecision {
            id: self.id,
            actor: self.actor,
            choices: self.choices(),
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(super) struct Targeting {
    card: Handle,
    decision: TargetDecision,
    selected: Vec<Handle>,
    destinations: Vec<Handle>,
}

impl PendingCast {
    pub(super) fn card(&self) -> Handle {
        self.card
    }
    pub(super) fn mode(&self) -> Option<u8> {
        self.mode
    }
    pub(super) fn discard(&self) -> Option<Handle> {
        self.discard
    }
    pub(super) fn sources(&self) -> &[Handle] {
        &self.sources
    }
    pub(super) fn effect(&self) -> Option<Effect> {
        self.effect
    }
}
impl Payment {
    pub(super) fn actor(&self) -> Seat {
        self.actor
    }
    pub(super) fn id(&self) -> DecisionId {
        self.id
    }
    pub(super) fn pool(&self) -> [u32; 6] {
        self.pool
    }
    pub(super) fn remaining(&self) -> ManaCost {
        self.remaining
    }
    pub(super) fn is_paid(&self) -> bool {
        self.remaining.generic == 0 && self.remaining.colored == [0; 6]
    }
}
impl Targeting {
    pub(super) fn card(&self) -> Handle {
        self.card
    }
    pub(super) fn decision(&self) -> &TargetDecision {
        &self.decision
    }
    pub(super) fn selected(&self) -> &[Handle] {
        &self.selected
    }
}
impl Game {
    pub(super) fn start_payment(
        &mut self,
        actor: Seat,
        generation: u64,
        remaining: ManaCost,
    ) -> PaymentDecision {
        let payment = Payment {
            actor,
            id: DecisionId {
                scope: self.objects.scope(),
                generation,
            },
            remaining,
            pool: self.turns.mana[seat_index(actor)],
        };
        let decision = payment.decision();
        self.turns.payment = Some(payment);
        self.generation = generation;
        decision
    }
    /// Target selection hands off directly to payment, with no priority window.
    pub(super) fn start_cast_payment(
        &mut self,
        actor: Seat,
        generation: u64,
        card: Handle,
        remaining: ManaCost,
        effect: Option<Effect>,
    ) -> PaymentDecision {
        let decision = self.start_payment(actor, generation, remaining);
        self.turns.casting = Some(PendingCast {
            card,
            sources: vec![],
            discard: None,
            mode: None,
            effect,
        });
        self.turns.targeting = None;
        decision
    }
    pub(super) fn stage_cast_mode(&mut self, mode: u8, generation: u64) {
        self.turns.casting.as_mut().expect("validated cast").mode = Some(mode);
        self.turns
            .payment
            .as_mut()
            .expect("validated payment")
            .id
            .generation = generation;
        self.generation = generation;
    }
    pub(super) fn stage_cast_discard(&mut self, card: Handle, generation: u64) {
        self.turns.casting.as_mut().expect("validated cast").discard = Some(card);
        self.turns
            .payment
            .as_mut()
            .expect("validated payment")
            .id
            .generation = generation;
        self.generation = generation;
    }
    pub(super) fn start_targeting(
        &mut self,
        card: Handle,
        decision: TargetDecision,
        destinations: Vec<Handle>,
    ) {
        self.generation = decision.id.generation;
        self.turns.targeting = Some(Targeting {
            card,
            decision,
            selected: vec![],
            destinations,
        });
    }
    pub(super) fn select_cast_target(&mut self, h: Handle, generation: u64) -> TargetDecision {
        let t = self.turns.targeting.as_mut().expect("validated targets");
        t.selected.push(h);
        if t.decision.kind == TargetKind::BiteSource {
            t.decision.kind = TargetKind::BiteDestination;
            t.decision.choices = t.destinations.clone();
        } else {
            t.decision.kind = TargetKind::Complete;
            t.decision.choices.clear();
        }
        t.decision.id.generation = generation;
        self.generation = generation;
        t.decision.clone()
    }
    /// The caller has checked source legality, pool overflow and generation.
    /// Reservation must precede either staged mutation.
    pub(super) fn stage_cast_mana(
        &mut self,
        land: Handle,
        color: usize,
        value: u32,
        generation: u64,
    ) -> Result<PaymentDecision, StorageError> {
        let cast = self.turns.casting.as_mut().expect("validated cast source");
        cast.sources
            .try_reserve(1)
            .map_err(|_| StorageError::CapacityExceeded)?;
        cast.sources.push(land);
        let p = self.turns.payment.as_mut().expect("validated payment");
        p.pool[color] = value;
        p.id.generation = generation;
        self.generation = generation;
        Ok(p.decision())
    }
    pub(super) fn spend_payment_unit(&mut self, color: Color, generation: u64) -> PaymentDecision {
        let p = self
            .turns
            .payment
            .as_mut()
            .expect("validated payment choice");
        let i = color.index();
        p.pool[i] -= 1;
        if p.remaining.colored[i] > 0 {
            p.remaining.colored[i] -= 1;
        } else {
            p.remaining.generic -= 1;
        }
        p.id.generation = generation;
        self.generation = generation;
        p.decision()
    }
    pub(super) fn cancel_cast_payment(&mut self, actor: Seat, generation: u64) -> TurnDecision {
        self.turns.payment = None;
        self.turns.casting = None;
        self.generation = generation;
        self.set_turn_decision(actor, TurnKind::Priority)
    }
    pub(super) fn cancel_cast_targets(&mut self, actor: Seat, generation: u64) -> TurnDecision {
        self.turns.targeting = None;
        self.generation = generation;
        self.set_turn_decision(actor, TurnKind::Priority)
    }
    pub(super) fn clear_cast_choices(&mut self) {
        self.turns.payment = None;
        self.turns.targeting = None;
        self.turns.casting = None;
    }
    pub(super) fn commit_payment(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> Result<TurnDecision, ManaError> {
        let p = self.validate_payment(actor, id)?;
        if !p.is_paid() {
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
    /// Only finish_cast calls this, after all spell/source/storage preflights.
    /// Its caller joins the payment with taps and the stack move before return.
    pub(super) fn commit_cast_payment(
        &mut self,
        actor: Seat,
        id: DecisionId,
    ) -> (PendingCast, TurnDecision) {
        let cast = self.turns.casting.take().expect("validated cast");
        let decision = self.commit_payment(actor, id).expect("preflighted payment");
        (cast, decision)
    }
}
