//! Fixed strategy contract in doc/heuristic-policy.md; no game or RNG access.
use crate::{Error, validate, validate_content};
use mtg_core::game::{
    policy::{Choice, Observation, Submission, VisibleRef, VisibleZone},
    views::VisibleCard,
};
pub const HEURISTIC_VERSION: &str = "heuristic-deathtouch-v1";
pub struct Heuristic {
    seat: u8,
}
impl Heuristic {
    pub fn new(version: &str, seat: u8) -> Result<Self, Error> {
        if version != HEURISTIC_VERSION {
            return Err(Error::UnsupportedVersion);
        }
        if seat > 1 {
            return Err(Error::WrongSeat);
        }
        Ok(Self { seat })
    }
    /// Idempotent for the same input. Engine application still checks freshness.
    pub fn choose(&self, o: &Observation) -> Result<Submission, Error> {
        if o.schema_version != 1 || o.view.schema_version != 1 {
            return Err(Error::UnsupportedVersion);
        }
        if o.view.seat != self.seat {
            return Err(Error::WrongSeat);
        }
        let d = o.decision.as_ref().ok_or(Error::Unavailable)?;
        if d.actor != self.seat {
            return Err(Error::WrongSeat);
        }
        validate(d)?;
        validate_content(o, d)?;
        let choices = if let Some(f) = &d.factored {
            vec![match d.kind {
                "attackers" => {
                    if f.selected == f.attackers {
                        Choice::FinishCombat
                    } else {
                        Choice::SelectAttackers {
                            cards: f.attackers.clone(),
                        }
                    }
                }
                "blockers" => {
                    let mut remaining = f
                        .attackers
                        .iter()
                        .map(|r| Ok((*r, stats(o, *r)?)))
                        .collect::<Result<Vec<_>, Error>>()?;
                    // Stable sort preserves domain order for equal stats.
                    remaining.sort_by_key(|a| std::cmp::Reverse(a.1));
                    let blocks = f
                        .blockers
                        .iter()
                        .filter_map(|b| {
                            let i = remaining
                                .iter()
                                .position(|(a, _)| !f.forbidden_blocks.contains(&(*b, *a)))?;
                            Some((*b, remaining.remove(i).0))
                        })
                        .collect::<Vec<_>>();
                    if blocks == f.blocks {
                        Choice::FinishCombat
                    } else {
                        Choice::SelectBlockers { blocks }
                    }
                }
                "combat_damage" => {
                    if let Some(a) = f.damage.iter().find(|a| a.amounts.is_none()) {
                        let mut left = a.power;
                        let mut amounts = Vec::with_capacity(a.blockers.len());
                        for (i, b) in a.blockers.iter().enumerate() {
                            let n = if let Some(lethal) = &a.trample_lethal {
                                left.min(lethal[i])
                            } else if i + 1 == a.blockers.len() {
                                left
                            } else {
                                left.min(stats(o, *b)?.1)
                            };
                            left -= n;
                            amounts.push((*b, n));
                        }
                        Choice::AssignDamage {
                            attacker: a.attacker,
                            amounts,
                        }
                    } else {
                        Choice::FinishCombat
                    }
                }
                _ => return Err(Error::UnsupportedDecision),
            }]
        } else {
            let mut ranked = d
                .candidates
                .iter()
                .zip(&d.legal_mask)
                .filter(|(_, legal)| **legal)
                .map(|(c, _)| Ok((c, self.score(o, d.kind, c)?)))
                .collect::<Result<Vec<_>, Error>>()?;
            ranked.sort_by_key(|a| std::cmp::Reverse(a.1));
            ranked
                .into_iter()
                .take(d.count)
                .map(|(c, _)| c.clone())
                .collect()
        };
        Ok(Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices,
        })
    }
    fn score(&self, o: &Observation, kind: &str, c: &Choice) -> Result<(i64, i64, i64), Error> {
        let plain = |n| Ok((n, 0, 0));
        match c {
            Choice::Keep => plain(1),
            Choice::Mulligan => plain(0),
            Choice::Bottom { card: r } | Choice::Discard { card: r } => {
                plain(-retention(card(o, *r)?.card))
            }
            Choice::Pass => plain(0),
            Choice::PlayLand { .. } => plain(40),
            Choice::Cast { card: r } => {
                let bf = o
                    .view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .ok_or(Error::InvalidObservation)?;
                let own = bf
                    .cards
                    .iter()
                    .any(|c| c.controller == self.seat && c.creature.is_some());
                let enemy = bf
                    .cards
                    .iter()
                    .any(|c| c.controller != self.seat && c.creature.is_some());
                plain(match card(o, *r)?.card {
                    "bear-cub" | "swab-goblin" | "dragon-fodder" | "llanowar-elves"
                    | "druid-of-the-cowl" | "magnigoth-sentry" | "axgard-cavalry"
                    | "tajuru-pathwarden" | "thornweald-archer" => 30,
                    "bite-down" if own && enemy => 20,
                    "giant-growth" if own && (!o.stack.is_empty() || !o.combat.is_empty()) => 10,
                    "bite-down" | "giant-growth" => -1,
                    _ => return Err(Error::UnsupportedContent),
                })
            }
            Choice::Activate { .. } => plain(5),
            Choice::FinishPayment | Choice::FinishTargets | Choice::FinishActivation => plain(30),
            Choice::Pay { .. } => plain(20),
            Choice::TapMana { card: r } => {
                if kind == "priority" {
                    return plain(-2);
                }
                let color = match card(o, *r)?.card {
                    "forest" | "llanowar-elves" | "druid-of-the-cowl" => 4,
                    "mountain" => 3,
                    _ => return Err(Error::UnsupportedContent),
                };
                let owed = o
                    .pending
                    .as_ref()
                    .and_then(|p| p.remaining.as_ref())
                    .is_some_and(|r| r.colored[color] > 0);
                Ok((10, i64::from(owed), 0))
            }
            Choice::CancelPayment | Choice::CancelTargets | Choice::CancelActivation => plain(0),
            Choice::Target { card: r } => {
                let target = card(o, *r)?;
                let (power, toughness) = stats(o, *r)?;
                if kind == "bite_destination" {
                    if target.controller == self.seat {
                        return plain(-1);
                    }
                    let source = o
                        .pending
                        .as_ref()
                        .and_then(|p| p.targets.first())
                        .and_then(|r| *r)
                        .ok_or(Error::InvalidObservation)?;
                    let lethal = stats(o, source)?.0 >= toughness;
                    Ok((
                        1 + i64::from(lethal),
                        i64::from(power),
                        i64::from(toughness),
                    ))
                } else {
                    if target.controller != self.seat {
                        return plain(-1);
                    }
                    Ok((1, i64::from(power), i64::from(toughness)))
                }
            }
            _ => Err(Error::UnsupportedDecision),
        }
    }
}
fn card(o: &Observation, r: VisibleRef) -> Result<&VisibleCard, Error> {
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
    cards.get(r.row).ok_or(Error::InvalidObservation)
}
fn stats(o: &Observation, r: VisibleRef) -> Result<(u32, u32), Error> {
    let [p, t, d] = card(o, r)?.creature.ok_or(Error::InvalidObservation)?;
    Ok((p, t.saturating_sub(d)))
}
fn retention(name: &str) -> i64 {
    match name {
        "forest" | "mountain" => 1,
        "giant-growth" => 2,
        "bite-down" => 3,
        "bear-cub" | "swab-goblin" => 4,
        _ => 0,
    }
}
