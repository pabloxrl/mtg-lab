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
    Bottom { card: VisibleRef },
    Pass,
    PlayLand { card: VisibleRef },
    TapMana { card: VisibleRef },
    Pay { color: u8 },
    FinishPayment,
    CancelPayment,
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
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Observation {
    pub schema_version: u32,
    pub view: views::PlayerView,
    pub decision: Option<Decision>,
    pub unsupported_families: [&'static str; 3],
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
impl Game {
    fn policy_ready(&self) -> Result<(), PolicyError> {
        if self.rng.is_none() || !self.work.is_empty() {
            return Err(PolicyError::Unavailable);
        }
        if self.turns.casting.is_some() || self.turns.targeting.is_some() {
            return Err(PolicyError::UnsupportedSpell);
        }
        if let Some(d) = self.turn_decision() {
            match d.kind {
                turns::TurnKind::Combat(_) => return Err(PolicyError::UnsupportedCombat),
                turns::TurnKind::Discard { .. } => return Err(PolicyError::UnsupportedDecision),
                turns::TurnKind::Priority => (),
            }
        }
        Ok(())
    }
    fn policy_actor_generation(&self) -> Option<(Seat, u64)> {
        self.decision
            .map(|d| (d.actor, d.generation))
            .or_else(|| {
                self.turns
                    .payment
                    .as_ref()
                    .map(|p| (p.actor, p.id.generation))
            })
            .or_else(|| self.turn_decision().map(|d| (d.actor, d.id.generation)))
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
        } else if let Some(p) = self.payment_decision(seat) {
            for color in 0..6 {
                push(
                    Choice::Pay { color },
                    p.choices.contains(&mana::Color::ALL[usize::from(color)]),
                )?;
            }
            push(Choice::FinishPayment, p.choices.is_empty())?;
            push(Choice::CancelPayment, true)?;
            ("payment", 1)
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
        Ok(Some(Decision {
            revision: self.policy_revision,
            generation,
            actor: seat_index(actor) as u8,
            kind,
            count,
            candidates,
            legal_mask,
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
            unsupported_families: ["spell", "combat", "cleanup_discard"],
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
        match &submission.choices[0] {
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
                self.tap_mana(actor, id, h).map(|_| ()).map_err(mana_error)
            }
            Choice::Pay { color } => self
                .choose_payment(actor, id, mana::Color::ALL[usize::from(*color)])
                .map(|_| ())
                .map_err(mana_error),
            Choice::FinishPayment => self
                .finish_payment(actor, id)
                .map(|_| ())
                .map_err(mana_error),
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
