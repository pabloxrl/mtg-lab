//! Versioned, seat-routed structured choices without privileged engine handles.
use super::*;
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisibleZone {
    Hand,
    Battlefield,
}
/// Row in the authorized observation, valid only with its decision generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisibleRef {
    pub zone: VisibleZone,
    pub row: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Choice {
    Keep,
    Mulligan,
    Bottom {
        card: VisibleRef,
    },
    Pass,
    PlayLand {
        card: VisibleRef,
    },
    Activate {
        card: VisibleRef,
    },
    FinishActivation,
    CancelActivation,
    TapMana {
        card: VisibleRef,
    },
    Pay {
        color: u8,
    },
    FinishPayment,
    CancelPayment,
    Cast {
        card: VisibleRef,
    },
    Target {
        card: VisibleRef,
    },
    FinishTargets,
    CancelTargets,
    SelectAttackers {
        cards: Vec<VisibleRef>,
    },
    SelectBlockers {
        blocks: Vec<(VisibleRef, VisibleRef)>,
    },
    AssignDamage {
        attacker: VisibleRef,
        amounts: Vec<(VisibleRef, u32)>,
    },
    FinishCombat,
    Discard {
        card: VisibleRef,
    },
    Spell,
    Combat,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Submission {
    pub revision: u64,
    pub schema_version: u32,
    pub generation: u64,
    pub choices: Vec<Choice>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Decision {
    pub revision: u64,
    pub generation: u64,
    pub actor: u8,
    pub kind: &'static str,
    pub count: usize,
    pub candidates: Vec<Choice>,
    pub legal_mask: Vec<bool>,
    pub factored: Option<CombatChoices>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub schema_version: u32,
    pub view: views::PlayerView,
    pub decision: Option<Decision>,
    pub pending: Option<PendingSpell>,
    pub stack: Vec<StackSpell>,
    pub combat: Vec<CombatAttack>,
    pub unsupported_families: [&'static str; 0],
}
/// Factored domains, not a table of every subset/map/integer composition.
/// Present only for the actor at a combat decision. All rows use the current view.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CombatChoices {
    pub attackers: Vec<VisibleRef>,
    pub blockers: Vec<VisibleRef>,
    pub selected: Vec<VisibleRef>,
    pub blocks: Vec<(VisibleRef, VisibleRef)>,
    /// Pairs excluded from the attacker/blocker Cartesian product; not whole maps.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub forbidden_blocks: Vec<(VisibleRef, VisibleRef)>,
    pub damage: Vec<DamageAllocation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DamageAllocation {
    pub attacker: VisibleRef,
    pub power: u32,
    pub blockers: Vec<VisibleRef>,
    /// Lethal damage needed per blocker; remainder may go to defender only when all are met.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trample_lethal: Option<Vec<u32>>,
    pub amounts: Option<Vec<(VisibleRef, u32)>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CombatAttack {
    pub attacker: VisibleRef,
    pub blocked: bool,
    pub blockers: Vec<VisibleRef>,
}
/// Actor-only provisional state. References use the accompanying visible rows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PendingSpell {
    pub card: VisibleRef,
    pub targets: Vec<Option<VisibleRef>>,
    pub sources: Vec<Option<VisibleRef>>,
    pub pool: Option<[u32; 6]>,
    pub remaining: Option<mana::ManaCost>,
}
/// Bottom-to-top committed stack. A departed target is null, never rebound to
/// another object or looked up in a hidden zone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StackSpell {
    pub row: usize,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub ability: bool,
    pub targets: Vec<Option<VisibleRef>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyError {
    UnsupportedVersion,
    Unavailable,
    WrongActor,
    StaleDecision,
    InvalidSelection,
    CapacityExceeded,
    UnsupportedSpell,
    UnsupportedCombat,
    UnsupportedDecision,
}
fn opening_error(error: ApplyError) -> PolicyError {
    match error {
        ApplyError::DecisionExhausted => PolicyError::CapacityExceeded,
        ApplyError::WrongActor => PolicyError::WrongActor,
        ApplyError::StaleDecision | ApplyError::StaleCandidate => PolicyError::StaleDecision,
        ApplyError::NoDecision | ApplyError::WorkPending => PolicyError::Unavailable,
        _ => PolicyError::InvalidSelection,
    }
}
fn turn_error(error: turns::TurnError) -> PolicyError {
    use turns::TurnError::*;
    match error {
        Invalid(e) => opening_error(e),
        Storage(StorageError::CapacityExceeded | StorageError::IdentityExhausted)
        | Draw(DrawError::Storage(
            StorageError::CapacityExceeded | StorageError::IdentityExhausted,
        ))
        | EffectOverflow
        | TurnExhausted => PolicyError::CapacityExceeded,
        UnsupportedStack => PolicyError::UnsupportedSpell,
        UnsupportedCombat => PolicyError::UnsupportedCombat,
        NotReady | AlreadyStarted => PolicyError::Unavailable,
        _ => PolicyError::InvalidSelection,
    }
}
fn mana_error(error: mana::ManaError) -> PolicyError {
    match error {
        mana::ManaError::Turn(e) => turn_error(e),
        mana::ManaError::Overflow => PolicyError::CapacityExceeded,
        _ => PolicyError::InvalidSelection,
    }
}
fn cast_error(error: casting::CastError) -> PolicyError {
    match error {
        casting::CastError::Mana(e) => mana_error(e),
        casting::CastError::Storage(
            StorageError::CapacityExceeded | StorageError::IdentityExhausted,
        ) => PolicyError::CapacityExceeded,
        _ => PolicyError::InvalidSelection,
    }
}
fn target_error(error: targets::TargetError) -> PolicyError {
    match error {
        targets::TargetError::Cast(e) => cast_error(e),
        targets::TargetError::Invalid(e) => opening_error(e),
        targets::TargetError::CapacityExceeded { .. } => PolicyError::CapacityExceeded,
        _ => PolicyError::InvalidSelection,
    }
}
fn combat_error(error: combat::CombatError) -> PolicyError {
    match error {
        combat::CombatError::Invalid(e) => opening_error(e),
        combat::CombatError::Turn(e) => turn_error(e),
        combat::CombatError::CapacityExceeded { .. } => PolicyError::CapacityExceeded,
        combat::CombatError::NotReady => PolicyError::Unavailable,
        _ => PolicyError::InvalidSelection,
    }
}
impl Game {
    fn policy_battlefield_ref(&self, h: Handle) -> Option<VisibleRef> {
        self.objects
            .in_zone(Zone::Battlefield)
            .position(|x| x == h)
            .map(|row| VisibleRef {
                zone: VisibleZone::Battlefield,
                row,
            })
    }
    fn policy_pending(&self, seat: Seat) -> Option<PendingSpell> {
        if let Some(p) = self.turns.activation.as_ref().filter(|p| p.actor == seat) {
            return Some(PendingSpell {
                card: self.policy_battlefield_ref(p.source)?,
                targets: p
                    .target
                    .into_iter()
                    .map(|h| self.policy_battlefield_ref(h))
                    .collect(),
                sources: vec![self.policy_battlefield_ref(p.source)],
                pool: None,
                remaining: None,
            });
        }
        let (card, targets, sources) = if let Some(t) = self
            .turns
            .targeting
            .as_ref()
            .filter(|t| t.decision().actor == seat)
        {
            (t.card(), t.selected().to_vec(), vec![])
        } else {
            let c = self.turns.casting.as_ref().filter(|_| {
                self.turns
                    .payment
                    .as_ref()
                    .is_some_and(|p| p.actor() == seat)
            })?;
            let targets = match c.effect() {
                Some(targets::Effect::Growth(a)) => vec![a],
                Some(targets::Effect::Bite(a, b)) => vec![a, b],
                None => vec![],
            };
            (c.card(), targets, c.sources().to_vec())
        };
        Some(PendingSpell {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: self.view_hand(seat).iter().position(|h| *h == card)?,
            },
            targets: targets
                .into_iter()
                .map(|h| self.policy_battlefield_ref(h))
                .collect(),
            sources: sources
                .into_iter()
                .map(|h| self.policy_battlefield_ref(h))
                .collect(),
            pool: self.turns.payment.as_ref().map(|p| p.pool()),
            remaining: self.turns.payment.as_ref().map(|p| p.remaining()),
        })
    }

    fn policy_ready(&self) -> Result<(), PolicyError> {
        if self.rng.is_none() || !self.work.is_empty() {
            return Err(PolicyError::Unavailable);
        }
        Ok(())
    }
    fn policy_actor_generation(&self) -> Option<(Seat, u64)> {
        self.turns
            .activation
            .as_ref()
            .map(|p| (p.actor, p.id.generation))
            .or_else(|| {
                self.decision
                    .map(|d| (d.actor, d.generation))
                    .or_else(|| {
                        self.turns
                            .targeting
                            .as_ref()
                            .map(|t| (t.decision().actor, t.decision().id.generation))
                    })
                    .or_else(|| {
                        self.turns
                            .payment
                            .as_ref()
                            .map(|p| (p.actor(), p.id().generation))
                    })
                    .or_else(|| self.turn_decision().map(|d| (d.actor, d.id.generation)))
            })
    }
    fn policy_decision(
        &self,
        seat: Seat,
        capacity: usize,
    ) -> Result<Option<Decision>, PolicyError> {
        let Some((actor, generation)) = self.policy_actor_generation() else {
            return Ok(None);
        };
        if actor != seat {
            return Ok(None);
        }
        let mut factored = None;
        let mut candidates = Vec::new();
        let mut legal_mask = Vec::new();
        let mut push = |choice, legal| -> Result<(), PolicyError> {
            if candidates.len() >= capacity {
                return Err(PolicyError::CapacityExceeded);
            }
            candidates
                .try_reserve(1)
                .map_err(|_| PolicyError::CapacityExceeded)?;
            legal_mask
                .try_reserve(1)
                .map_err(|_| PolicyError::CapacityExceeded)?;
            candidates.push(choice);
            legal_mask.push(legal);
            Ok(())
        };
        let (kind, count) = if let Some(d) = self.decision {
            match d.kind {
                OpeningKind::KeepOrMulligan => {
                    for c in d.candidates {
                        push(
                            match c {
                                OpeningChoice::Keep => Choice::Keep,
                                OpeningChoice::Mulligan => Choice::Mulligan,
                            },
                            true,
                        )?;
                    }
                    ("keep_or_mulligan", 1)
                }
                OpeningKind::Bottom { count } => {
                    for row in 0..self.view_hand(seat).len() {
                        push(
                            Choice::Bottom {
                                card: VisibleRef {
                                    zone: VisibleZone::Hand,
                                    row,
                                },
                            },
                            true,
                        )?;
                    }
                    ("bottom", count)
                }
            }
        } else if let Some(p) = self.turns.activation.as_ref() {
            if !p.power && !(p.invoker && p.target.is_some()) {
                for (row, h) in self.objects.in_zone(Zone::Battlefield).enumerate() {
                    push(
                        Choice::Target {
                            card: VisibleRef {
                                zone: VisibleZone::Battlefield,
                                row,
                            },
                        },
                        self.haste_target(h),
                    )?;
                }
            } else {
                for color in 0..6 {
                    push(
                        Choice::Pay { color },
                        self.activation_payment_allowed(p, color as usize),
                    )?;
                }
            }
            push(
                Choice::FinishActivation,
                if p.power || p.invoker {
                    p.paid
                } else {
                    p.target.is_some()
                },
            )?;
            push(Choice::CancelActivation, true)?;
            (
                if p.power || (p.invoker && p.target.is_some()) {
                    "activation_payment"
                } else {
                    "activation_target"
                },
                1,
            )
        } else if let Some(t) = self.target_decision(seat) {
            for (row, h) in self.objects.in_zone(Zone::Battlefield).enumerate() {
                push(
                    Choice::Target {
                        card: VisibleRef {
                            zone: VisibleZone::Battlefield,
                            row,
                        },
                    },
                    t.choices.contains(&h),
                )?;
            }
            push(
                Choice::FinishTargets,
                t.kind == targets::TargetKind::Complete,
            )?;
            push(Choice::CancelTargets, true)?;
            (
                match t.kind {
                    targets::TargetKind::Growth => "growth_target",
                    targets::TargetKind::BiteSource => "bite_source",
                    targets::TargetKind::BiteDestination => "bite_destination",
                    targets::TargetKind::Complete => "targets_complete",
                },
                1,
            )
        } else if let Some(p) = self.payment_decision(seat) {
            for color in 0..6 {
                push(
                    Choice::Pay { color },
                    p.choices.contains(&mana::Color::ALL[usize::from(color)]),
                )?;
            }
            let is_paid = self.turns.payment.as_ref().expect("payment").is_paid();
            push(Choice::FinishPayment, is_paid)?;
            if self.turns.casting.is_some() {
                let sources = self.cast_mana_sources(seat);
                for (row, h) in self.objects.in_zone(Zone::Battlefield).enumerate() {
                    push(
                        Choice::TapMana {
                            card: VisibleRef {
                                zone: VisibleZone::Battlefield,
                                row,
                            },
                        },
                        sources.contains(&h),
                    )?;
                }
            }
            push(Choice::CancelPayment, true)?;
            ("payment", 1)
        } else if let Some(d) = self
            .turn_decision()
            .filter(|d| matches!(d.kind, turns::TurnKind::Combat(_)))
        {
            let c = self.combat_decision(seat, capacity).map_err(combat_error)?;
            let reference = |h| {
                self.policy_battlefield_ref(h)
                    .expect("live public combat object")
            };
            let damage = c
                .damage
                .iter()
                .map(|a| DamageAllocation {
                    attacker: reference(a.attacker),
                    power: a.power,
                    trample_lethal: a.trample_lethal.clone(),
                    blockers: a.blockers.iter().map(|h| reference(*h)).collect(),
                    amounts: self
                        .turns
                        .combat
                        .assignments
                        .iter()
                        .find(|(h, _)| *h == a.attacker)
                        .map(|(_, amounts)| {
                            amounts.iter().map(|(h, n)| (reference(*h), *n)).collect()
                        }),
                })
                .collect::<Vec<_>>();
            push(
                Choice::FinishCombat,
                damage.iter().all(|d| d.amounts.is_some()),
            )?;
            factored = Some(CombatChoices {
                attackers: c.attackers.into_iter().map(reference).collect(),
                blockers: c.blockers.into_iter().map(reference).collect(),
                selected: c.selected.into_iter().map(reference).collect(),
                blocks: c
                    .blocks
                    .into_iter()
                    .map(|(b, a)| (reference(b), reference(a)))
                    .collect(),
                forbidden_blocks: c
                    .forbidden_blocks
                    .into_iter()
                    .map(|(b, a)| (reference(b), reference(a)))
                    .collect(),
                damage,
            });
            (
                match d.kind {
                    turns::TurnKind::Combat(combat::CombatKind::Attackers) => "attackers",
                    turns::TurnKind::Combat(combat::CombatKind::Blockers) => "blockers",
                    _ => "combat_damage",
                },
                1,
            )
        } else if let Some(turns::TurnDecision {
            kind: turns::TurnKind::Discard { count },
            ..
        }) = self.turn_decision()
        {
            for row in 0..self.view_hand(seat).len() {
                push(
                    Choice::Discard {
                        card: VisibleRef {
                            zone: VisibleZone::Hand,
                            row,
                        },
                    },
                    true,
                )?;
            }
            ("cleanup_discard", count)
        } else {
            push(Choice::Pass, true)?;
            let lands = self.land_candidates(seat);
            for (row, h) in self.view_hand(seat).into_iter().enumerate() {
                push(
                    Choice::PlayLand {
                        card: VisibleRef {
                            zone: VisibleZone::Hand,
                            row,
                        },
                    },
                    lands.contains(&h),
                )?;
            }
            let spells = self.cast_candidates(seat);
            for (row, h) in self.view_hand(seat).into_iter().enumerate() {
                if casting::cost(self.objects.get(h).expect("hand").card).is_some() {
                    push(
                        Choice::Cast {
                            card: VisibleRef {
                                zone: VisibleZone::Hand,
                                row,
                            },
                        },
                        spells.contains(&h),
                    )?;
                }
            }
            let activations = self.activation_candidates(seat);
            for (row, h) in self.objects.in_zone(Zone::Battlefield).enumerate() {
                if card_definitions::definition(self.objects.get(h).unwrap().card)
                    .haste_activation()
                    || card_definitions::definition(self.objects.get(h).unwrap().card)
                        .power_activation()
                    || card_definitions::definition(self.objects.get(h).unwrap().card)
                        .invoker_activation()
                {
                    push(
                        Choice::Activate {
                            card: VisibleRef {
                                zone: VisibleZone::Battlefield,
                                row,
                            },
                        },
                        activations.contains(&h),
                    )?;
                }
            }
            let sources = self.mana_sources(seat);
            for (row, h) in self.objects.in_zone(Zone::Battlefield).enumerate() {
                push(
                    Choice::TapMana {
                        card: VisibleRef {
                            zone: VisibleZone::Battlefield,
                            row,
                        },
                    },
                    sources.contains(&h),
                )?;
            }
            ("priority", 1)
        };
        // Bound all factored domain and provisional rows as well as flat rows.
        // No enumeration of subsets, maps or damage amounts is needed.
        let mut needed = candidates.len();
        if let Some(c) = &factored {
            for len in [
                c.attackers.len(),
                c.blockers.len(),
                c.selected.len(),
                c.blocks.len(),
                c.forbidden_blocks.len(),
            ] {
                needed = needed
                    .checked_add(len)
                    .ok_or(PolicyError::CapacityExceeded)?;
            }
            for d in &c.damage {
                needed = needed
                    .checked_add(1)
                    .and_then(|n| n.checked_add(d.blockers.len()))
                    .and_then(|n| n.checked_add(d.amounts.as_ref().map_or(0, Vec::len)))
                    .and_then(|n| n.checked_add(d.trample_lethal.as_ref().map_or(0, Vec::len)))
                    .ok_or(PolicyError::CapacityExceeded)?;
            }
        }
        if needed > capacity {
            return Err(PolicyError::CapacityExceeded);
        }
        Ok(Some(Decision {
            revision: self.policy_revision,
            generation,
            actor: seat_index(actor) as u8,
            kind,
            count,
            candidates,
            legal_mask,
            factored,
        }))
    }
    /// The caller binds the game and authorized seat. Capacity bounds the entire
    /// candidate table (including masked rows); overflow never truncates it.
    pub fn policy_observe(&self, seat: Seat, capacity: usize) -> Result<Observation, PolicyError> {
        self.policy_ready()?;
        let decision = self.policy_decision(seat, capacity)?;
        let view = self
            .observe_visible_state(seat)
            .map_err(|_| PolicyError::Unavailable)?;
        Ok(Observation {
            schema_version: SCHEMA_VERSION,
            view,
            decision,
            pending: self.policy_pending(seat),
            stack: self
                .turns
                .stack
                .iter()
                .enumerate()
                .map(|(row, h)| StackSpell {
                    row,
                    ability: self.turns.abilities.iter().any(|a| a.object == *h),
                    targets: self
                        .stack_targets(*h)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|h| self.policy_battlefield_ref(h))
                        .collect(),
                })
                .collect(),
            combat: self
                .combat()
                .into_iter()
                .map(|a| CombatAttack {
                    attacker: self
                        .policy_battlefield_ref(a.creature)
                        .expect("live attacker"),
                    blocked: a.blocked,
                    blockers: a
                        .blockers
                        .into_iter()
                        .map(|h| self.policy_battlefield_ref(h).expect("live blocker"))
                        .collect(),
                })
                .collect(),
            unsupported_families: [],
        })
    }
    /// Validate semantic choices against the current authorized table before
    /// resolving any reference. No policy can submit an object handle or cost.
    pub fn apply_policy(
        &mut self,
        actor: Seat,
        submission: &Submission,
        capacity: usize,
    ) -> Result<(), PolicyError> {
        if submission.schema_version != SCHEMA_VERSION {
            return Err(PolicyError::UnsupportedVersion);
        }
        self.policy_ready()?;
        let (expected, generation) = self
            .policy_actor_generation()
            .ok_or(PolicyError::Unavailable)?;
        if actor != expected {
            return Err(PolicyError::WrongActor);
        }
        if submission.revision != self.policy_revision || submission.generation != generation {
            return Err(PolicyError::StaleDecision);
        }
        for choice in &submission.choices {
            match choice {
                Choice::Spell => return Err(PolicyError::UnsupportedSpell),
                Choice::Combat => return Err(PolicyError::UnsupportedCombat),
                _ => (),
            }
        }
        let d = self
            .policy_decision(actor, capacity)?
            .ok_or(PolicyError::Unavailable)?;
        if submission.choices.len() != d.count {
            return Err(PolicyError::InvalidSelection);
        }
        let id = DecisionId {
            scope: self.objects.scope(),
            generation,
        };
        if let Some(c) = &d.factored {
            let handle = |r: &VisibleRef| {
                self.objects
                    .in_zone(Zone::Battlefield)
                    .nth(r.row)
                    .expect("validated combat row")
            };
            match &submission.choices[0] {
                Choice::SelectAttackers { cards } if d.kind == "attackers" => {
                    if !cards.iter().all(|r| c.attackers.contains(r)) {
                        return Err(PolicyError::InvalidSelection);
                    }
                    let hs = cards.iter().map(handle).collect::<Vec<_>>();
                    return self
                        .select_attackers(actor, id, &hs)
                        .map(|_| ())
                        .map_err(combat_error);
                }
                Choice::SelectBlockers { blocks } if d.kind == "blockers" => {
                    if !blocks.iter().all(|(b, a)| {
                        c.blockers.contains(b)
                            && c.attackers.contains(a)
                            && !c.forbidden_blocks.contains(&(*b, *a))
                    }) {
                        return Err(PolicyError::InvalidSelection);
                    }
                    let hs = blocks
                        .iter()
                        .map(|(b, a)| (handle(b), handle(a)))
                        .collect::<Vec<_>>();
                    return self
                        .select_blockers(actor, id, &hs)
                        .map(|_| ())
                        .map_err(combat_error);
                }
                Choice::AssignDamage { attacker, amounts } if d.kind == "combat_damage" => {
                    let allocation = c
                        .damage
                        .iter()
                        .find(|a| a.attacker == *attacker)
                        .ok_or(PolicyError::InvalidSelection)?;
                    if !amounts.iter().all(|(b, _)| allocation.blockers.contains(b)) {
                        return Err(PolicyError::InvalidSelection);
                    }
                    let h = handle(attacker);
                    let hs = amounts
                        .iter()
                        .map(|(b, n)| (handle(b), *n))
                        .collect::<Vec<_>>();
                    return self
                        .assign_combat_damage(actor, id, h, &hs)
                        .map(|_| ())
                        .map_err(combat_error);
                }
                _ => (),
            }
        }
        let mut rows = Vec::new();
        for choice in &submission.choices {
            let row = d
                .candidates
                .iter()
                .position(|c| c == choice)
                .ok_or(PolicyError::InvalidSelection)?;
            if !d.legal_mask[row] || rows.contains(&row) {
                return Err(PolicyError::InvalidSelection);
            }
            rows.push(row);
        }
        if let Some(opening) = self.decision {
            let selection = match opening.kind {
                OpeningKind::KeepOrMulligan => Selection::Choose(opening.candidate(rows[0])),
                OpeningKind::Bottom { .. } => {
                    let hand = self.view_hand(actor);
                    let original = self.bottom_cards().expect("validated bottom decision");
                    Selection::Bottom(
                        rows.into_iter()
                            .map(|row| {
                                opening.candidate(
                                    original
                                        .iter()
                                        .position(|h| *h == hand[row])
                                        .expect("same visible hand"),
                                )
                            })
                            .collect(),
                    )
                }
            };
            return self
                .apply(
                    actor,
                    &OpeningAction {
                        decision: opening.id,
                        selection,
                    },
                )
                .map(|_| ())
                .map_err(opening_error);
        }
        let id = DecisionId {
            scope: self.objects.scope(),
            generation,
        };
        if d.kind == "cleanup_discard" {
            let hand = self.view_hand(actor);
            let original = self.discard_cards().expect("discard decision");
            let selection = turns::TurnSelection::Discard(
                rows.into_iter()
                    .map(|row| CandidateId {
                        decision: id,
                        index: original
                            .iter()
                            .position(|h| *h == hand[row])
                            .expect("same hand"),
                    })
                    .collect(),
            );
            return self
                .apply_turn(
                    actor,
                    &turns::TurnAction {
                        decision: id,
                        selection,
                    },
                )
                .map(|_| ())
                .map_err(turn_error);
        }
        match &submission.choices[0] {
            Choice::Activate { card } => {
                let h = self
                    .objects
                    .in_zone(Zone::Battlefield)
                    .nth(card.row)
                    .unwrap();
                self.begin_activation(actor, id, h).map_err(turn_error)
            }
            Choice::FinishActivation => self
                .finish_activation(actor, id)
                .map(|_| ())
                .map_err(turn_error),
            Choice::CancelActivation => self
                .cancel_activation(actor, id)
                .map(|_| ())
                .map_err(turn_error),
            Choice::Target { card } if self.turns.activation.is_some() => {
                let h = self
                    .objects
                    .in_zone(Zone::Battlefield)
                    .nth(card.row)
                    .unwrap();
                self.choose_activation_target(actor, id, h)
                    .map_err(turn_error)
            }
            Choice::FinishCombat => self
                .finish_combat(actor, id)
                .map(|_| ())
                .map_err(combat_error),
            Choice::Pass => self
                .apply_turn(
                    actor,
                    &turns::TurnAction {
                        decision: id,
                        selection: turns::TurnSelection::Pass(CandidateId {
                            decision: id,
                            index: 0,
                        }),
                    },
                )
                .map(|_| ())
                .map_err(turn_error),
            Choice::PlayLand { card } => {
                let h = self.view_hand(actor)[card.row];
                self.play_land(actor, id, h).map(|_| ()).map_err(mana_error)
            }
            Choice::TapMana { card } => {
                let h = self
                    .objects
                    .in_zone(Zone::Battlefield)
                    .nth(card.row)
                    .expect("validated visible row");
                if self.turns.casting.is_some() {
                    self.cast_tap_mana(actor, id, h)
                        .map(|_| ())
                        .map_err(cast_error)
                } else {
                    self.tap_mana(actor, id, h).map(|_| ()).map_err(mana_error)
                }
            }
            Choice::Pay { color } if self.turns.activation.is_some() => self
                .pay_activation(actor, id, mana::Color::ALL[usize::from(*color)])
                .map_err(turn_error),
            Choice::Pay { color } => self
                .choose_payment(actor, id, mana::Color::ALL[usize::from(*color)])
                .map(|_| ())
                .map_err(mana_error),
            Choice::FinishPayment => {
                if self.turns.casting.is_some() {
                    self.finish_cast(actor, id).map(|_| ()).map_err(cast_error)
                } else {
                    self.finish_payment(actor, id)
                        .map(|_| ())
                        .map_err(mana_error)
                }
            }
            Choice::Cast { card } => {
                let h = self.view_hand(actor)[card.row];
                if targets::instant(self.objects.get(h).expect("hand").card) {
                    self.begin_targeted_cast(actor, id, h, capacity)
                        .map(|_| ())
                        .map_err(target_error)
                } else {
                    self.begin_cast(actor, id, h)
                        .map(|_| ())
                        .map_err(cast_error)
                }
            }
            Choice::Target { card } => {
                let h = self
                    .objects
                    .in_zone(Zone::Battlefield)
                    .nth(card.row)
                    .expect("validated target row");
                self.choose_target(actor, id, h)
                    .map(|_| ())
                    .map_err(target_error)
            }
            Choice::FinishTargets => self
                .finish_targets(actor, id)
                .map(|_| ())
                .map_err(target_error),
            Choice::CancelTargets => self
                .cancel_targets(actor, id)
                .map(|_| ())
                .map_err(target_error),
            Choice::CancelPayment => self
                .cancel_payment(actor, id)
                .map(|_| ())
                .map_err(mana_error),
            _ => Err(PolicyError::InvalidSelection),
        }
    }
}
#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
