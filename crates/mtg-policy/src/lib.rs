//! Native policies consuming only seat-authorized observations.
//!
//! See `doc/random-policy.md` for distributions, content limits and RNG versioning.
use mtg_core::{
    game::policy::{self, Choice, Decision, Observation, Submission},
    rng::{EpisodeRng, Stream},
};

pub const VERSION: &str = "legal-random-haste-v1";
/// Pins SplitMix64 seed derivation, seat domains and unbiased bounded sampling.
pub const RNG_VERSION: &str = "legal-random-rng-v1";
const POLICY_SCHEMA: u32 = 1;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedVersion,
    WrongSeat,
    Unavailable,
    UnsupportedDecision,
    UnsupportedContent,
    InvalidObservation,
}

/// One owned policy stream per episode and seat. Recreate for each episode;
/// the caller separately owns the game, routing, submissions and work limits.
pub struct LegalRandom {
    seat: u8,
    rng: EpisodeRng,
}

impl LegalRandom {
    pub fn new(
        version: &str,
        rng_version: &str,
        master: u64,
        episode: u64,
        seat: u8,
    ) -> Result<Self, Error> {
        if version != VERSION || rng_version != RNG_VERSION {
            return Err(Error::UnsupportedVersion);
        }
        let stream = match seat {
            0 => Stream::PolicySeat0,
            1 => Stream::PolicySeat1,
            _ => return Err(Error::WrongSeat),
        };
        // Deliberately pin the primitive, not its potentially changing default.
        let rng = EpisodeRng::new("splitmix64-v1", master, episode, stream)
            .map_err(|_| Error::UnsupportedVersion)?;
        Ok(Self { seat, rng })
    }

    /// Sample a semantic command from a fresh, authorized observation. Evaluation
    /// consumes only this policy's RNG. Apply through Game::apply_policy, which
    /// rejects stale/wrong-seat commands without changing the game. Re-evaluation
    /// consumes another sample; it is not an idempotent transport operation.
    pub fn choose(&mut self, observation: &Observation) -> Result<Submission, Error> {
        if observation.schema_version != POLICY_SCHEMA || observation.view.schema_version != 1 {
            return Err(Error::UnsupportedVersion);
        }
        if observation.view.seat != self.seat {
            return Err(Error::WrongSeat);
        }
        let d = observation.decision.as_ref().ok_or(Error::Unavailable)?;
        if d.actor != self.seat {
            return Err(Error::WrongSeat);
        }
        validate(d)?;
        validate_content(observation, d)?;
        let choices = if let Some(f) = &d.factored {
            vec![match d.kind {
                "attackers" => {
                    if self.below(2) == 1 {
                        Choice::FinishCombat
                    } else {
                        Choice::SelectAttackers {
                            cards: f
                                .attackers
                                .iter()
                                .copied()
                                .filter(|_| self.below(2) == 1)
                                .collect(),
                        }
                    }
                }
                "blockers" => {
                    if self.below(2) == 1 {
                        Choice::FinishCombat
                    } else {
                        Choice::SelectBlockers {
                            blocks: f
                                .blockers
                                .iter()
                                .filter_map(|b| {
                                    let allowed: Vec<_> = f
                                        .attackers
                                        .iter()
                                        .filter(|a| !f.forbidden_blocks.contains(&(*b, **a)))
                                        .collect();
                                    let n = self.below(allowed.len() as u64 + 1);
                                    (n > 0).then(|| (*b, *allowed[n as usize - 1]))
                                })
                                .collect(),
                        }
                    }
                }
                "combat_damage" => {
                    if let Some(a) = f.damage.iter().find(|a| a.amounts.is_none()) {
                        let mut remaining = a.power;
                        let last = a.blockers.len() - 1;
                        let amounts = a
                            .blockers
                            .iter()
                            .enumerate()
                            .map(|(i, b)| {
                                let n = if i == last {
                                    remaining
                                } else {
                                    self.below(u64::from(remaining) + 1) as u32
                                };
                                remaining -= n;
                                (*b, n)
                            })
                            .collect();
                        Choice::AssignDamage {
                            attacker: a.attacker,
                            amounts,
                        }
                    } else {
                        Choice::FinishCombat
                    }
                }
                _ => unreachable!("validated kind"),
            }]
        } else {
            let mut legal = d
                .candidates
                .iter()
                .zip(&d.legal_mask)
                .filter_map(|(c, legal)| legal.then_some(c))
                .collect::<Vec<_>>();
            (0..d.count)
                .map(|_| {
                    let index = self.below(legal.len() as u64) as usize;
                    legal.remove(index).clone()
                })
                .collect()
        };
        Ok(Submission {
            schema_version: POLICY_SCHEMA,
            revision: d.revision,
            generation: d.generation,
            choices,
        })
    }

    fn below(&mut self, bound: u64) -> u64 {
        bounded(|| self.rng.next_u64(), bound)
    }
}

fn bounded(mut word: impl FnMut() -> u64, bound: u64) -> u64 {
    // [threshold, 2^64) contains a whole number of residue classes.
    let threshold = bound.wrapping_neg() % bound;
    loop {
        let n = word();
        if n >= threshold {
            return n % bound;
        }
    }
}

fn validate(d: &Decision) -> Result<(), Error> {
    if d.candidates.len() != d.legal_mask.len() || d.count == 0 {
        return Err(Error::InvalidObservation);
    }
    let combat = matches!(d.kind, "attackers" | "blockers" | "combat_damage");
    if combat {
        let f = d.factored.as_ref().ok_or(Error::InvalidObservation)?;
        if d.count != 1 || d.candidates != [Choice::FinishCombat] {
            return Err(Error::InvalidObservation);
        }
        let can_finish = d.kind != "combat_damage" || f.damage.iter().all(|a| a.amounts.is_some());
        if d.legal_mask != [can_finish] || f.damage.iter().any(|a| a.blockers.is_empty()) {
            return Err(Error::InvalidObservation);
        }
        return Ok(());
    }
    if !matches!(
        d.kind,
        "keep_or_mulligan"
            | "bottom"
            | "priority"
            | "growth_target"
            | "bite_source"
            | "bite_destination"
            | "targets_complete"
            | "activation_target"
            | "payment"
            | "cleanup_discard"
    ) {
        return Err(Error::UnsupportedDecision);
    }
    if d.factored.is_some()
        || (!matches!(d.kind, "bottom" | "cleanup_discard") && d.count != 1)
        || d.legal_mask.iter().filter(|legal| **legal).count() < d.count
    {
        return Err(Error::InvalidObservation);
    }
    for (c, legal) in d.candidates.iter().zip(&d.legal_mask) {
        if !legal {
            continue;
        }
        let supported = match d.kind {
            "keep_or_mulligan" => matches!(c, Choice::Keep | Choice::Mulligan),
            "bottom" => matches!(c, Choice::Bottom { .. }),
            "cleanup_discard" => matches!(c, Choice::Discard { .. }),
            "priority" => matches!(
                c,
                Choice::Pass
                    | Choice::PlayLand { .. }
                    | Choice::TapMana { .. }
                    | Choice::Cast { .. }
                    | Choice::Activate { .. }
            ),
            "activation_target" => matches!(
                c,
                Choice::Target { .. } | Choice::FinishActivation | Choice::CancelActivation
            ),
            "payment" => matches!(
                c,
                Choice::Pay { .. }
                    | Choice::TapMana { .. }
                    | Choice::FinishPayment
                    | Choice::CancelPayment
            ),
            _ => matches!(
                c,
                Choice::Target { .. } | Choice::FinishTargets | Choice::CancelTargets
            ),
        };
        if !supported {
            return Err(Error::UnsupportedDecision);
        }
    }
    Ok(())
}

// A new core card must not silently expand a versioned policy's supported casts
// or mana sources. Unsupported masked hand cards are deliberately not inspected.
fn validate_content(o: &Observation, d: &Decision) -> Result<(), Error> {
    use policy::VisibleZone;
    for (c, legal) in d.candidates.iter().zip(&d.legal_mask) {
        if !legal {
            continue;
        }
        let (r, supported): (_, &[&str]) = match c {
            Choice::Cast { card } => (
                card,
                &[
                    "bear-cub",
                    "swab-goblin",
                    "giant-growth",
                    "bite-down",
                    "dragon-fodder",
                    "llanowar-elves",
                    "druid-of-the-cowl",
                    "magnigoth-sentry",
                    "axgard-cavalry",
                ],
            ),
            Choice::Activate { card } => (card, &["axgard-cavalry"]),
            Choice::PlayLand { card } => (card, &["forest", "mountain"]),
            Choice::TapMana { card } => (
                card,
                &["forest", "mountain", "llanowar-elves", "druid-of-the-cowl"],
            ),
            _ => continue,
        };
        let cards = match r.zone {
            VisibleZone::Hand => &o.view.hand,
            VisibleZone::Battlefield => {
                &o.view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .ok_or(Error::InvalidObservation)?
                    .cards
            }
        };
        let card = cards.get(r.row).ok_or(Error::InvalidObservation)?;
        if !supported.contains(&card.card) {
            return Err(Error::UnsupportedContent);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_rejects_bias_tail_and_handles_extremes() {
        // For bound3 reject 0; 1..=6 give exactly two instances of each row.
        let mut words = [0, 1, 2, 3, 4, 5, 6].into_iter();
        assert_eq!(
            (0..6)
                .map(|_| bounded(|| words.next().unwrap(), 3))
                .collect::<Vec<_>>(),
            [1, 2, 0, 1, 2, 0]
        );
        assert_eq!(bounded(|| u64::MAX, 1), 0);
        let mut words = [0, 1, u64::MAX].into_iter();
        assert_eq!(bounded(|| words.next().unwrap(), u64::MAX), 1);
        assert_eq!(bounded(|| words.next().unwrap(), u64::MAX), 0);
    }
}

mod heuristic;
pub use heuristic::{HEURISTIC_VERSION, Heuristic};
