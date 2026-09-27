//! Seat-filtered structured core observations. See doc/views.md for schema v1.
use super::*;
use serde::Serialize;

pub const SCHEMA_VERSION: u32 = 1;
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VisibleCard {
    pub card: &'static str,
    pub owner: u8,
    pub controller: u8,
    pub tapped: bool,
    pub creature: Option<[u32; 3]>,
    pub summoning_sick: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PublicZone {
    pub zone: &'static str,
    pub cards: Vec<VisibleCard>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Revelation {
    pub card: &'static str,
    pub owner: u8,
    /// Historical zone only; never refreshed from the current hidden state.
    pub zone_at_reveal: &'static str,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OpeningView {
    pub generation: u64,
    pub kind: &'static str,
    pub count: usize,
    /// Ordered legal rows: keep/mulligan, or card keys for ordered bottoming.
    pub candidates: Vec<&'static str>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PlayerView {
    pub schema_version: u32,
    pub seat: u8,
    pub life: [i64; 2],
    pub hand_counts: [usize; 2],
    pub library_counts: [usize; 2],
    pub hand: Vec<VisibleCard>,
    pub public_zones: Vec<PublicZone>,
    pub remembered: Vec<Revelation>,
    pub starting_seat: u8,
    /// Turn number, active seat and step; absent during opening.
    pub turn: Option<(u64, u8, &'static str)>,
    pub mana: [[u32; 6]; 2],
    pub acting_seat: Option<u8>,
    pub opening: Option<OpeningView>,
    pub terminal: Option<TerminalView>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TerminalView {
    pub winner: Option<u8>,
    pub losses: [Option<&'static str>; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewError {
    Unavailable,
    WrongActor,
    StaleDecision,
    InvalidSelection,
}
fn zone_name(zone: Zone) -> &'static str {
    match zone {
        Zone::Library(Seat::P0) => "library_0",
        Zone::Library(Seat::P1) => "library_1",
        Zone::Hand(Seat::P0) => "hand_0",
        Zone::Hand(Seat::P1) => "hand_1",
        Zone::Graveyard(Seat::P0) => "graveyard_0",
        Zone::Graveyard(Seat::P1) => "graveyard_1",
        Zone::Battlefield => "battlefield",
        Zone::Stack => "stack",
        Zone::Exile => "exile",
    }
}
fn step_name(step: turns::Step) -> &'static str {
    use turns::Step::*;
    match step {
        Upkeep => "upkeep",
        Draw => "draw",
        PrecombatMain => "precombat_main",
        BeginningCombat => "beginning_combat",
        DeclareAttackers => "declare_attackers",
        DeclareBlockers => "declare_blockers",
        CombatDamage => "combat_damage",
        EndCombat => "end_combat",
        PostcombatMain => "postcombat_main",
        End => "end",
        Cleanup => "cleanup",
    }
}
impl Game {
    fn visible_card(&self, h: Handle) -> VisibleCard {
        let o = self.objects.get(h).expect("live zone member");
        VisibleCard {
            card: o.card.identity().key,
            owner: seat_index(o.owner) as u8,
            controller: seat_index(o.controller) as u8,
            tapped: o.tapped,
            creature: self
                .creature_state(h)
                .map(|c| [c.power, c.toughness, c.damage]),
            summoning_sick: o.zone == Zone::Battlefield && self.summoning_sick(h),
        }
    }
    pub(super) fn view_hand(&self, seat: Seat) -> Vec<Handle> {
        let mut cards: Vec<_> = self.objects.in_zone(Zone::Hand(seat)).collect();
        // Stable sorting groups indistinguishable duplicates without exposing
        // storage addresses or the initial deck/shuffle position.
        cards.sort_by_key(|h| self.objects.get(*h).expect("live hand").card.identity().key);
        cards
    }
    pub fn observe(&self, seat: Seat) -> Result<PlayerView, ViewError> {
        if self.turns.payment.is_some() {
            return Err(ViewError::Unavailable);
        }
        self.observe_visible_state(seat)
    }
    // Shared committed-state projection. Only the policy boundary exposes
    // standalone payment choices; provisional pool and cost never enter this view.
    pub(super) fn observe_visible_state(&self, seat: Seat) -> Result<PlayerView, ViewError> {
        if self.rng.is_none()
            || !self.work.is_empty()
            || self.turns.targeting.is_some()
            || self.turns.casting.is_some()
        {
            return Err(ViewError::Unavailable);
        }
        let hand = self.view_hand(seat);
        let opening = self.decision.filter(|d| d.actor == seat).map(|d| {
            let (kind, count, candidates) = match d.kind {
                OpeningKind::KeepOrMulligan => (
                    "keep_or_mulligan",
                    1,
                    d.candidates
                        .iter()
                        .map(|c| match c {
                            OpeningChoice::Keep => "keep",
                            OpeningChoice::Mulligan => "mulligan",
                        })
                        .collect(),
                ),
                OpeningKind::Bottom { count } => (
                    "bottom",
                    count,
                    hand.iter()
                        .map(|h| self.objects.get(*h).expect("live hand").card.identity().key)
                        .collect(),
                ),
            };
            OpeningView {
                generation: d.generation,
                kind,
                count,
                candidates,
            }
        });
        let public_zones = Zone::ALL
            .into_iter()
            .filter(|z| z.is_public())
            .map(|zone| PublicZone {
                zone: zone_name(zone),
                cards: self
                    .objects
                    .in_zone(zone)
                    .map(|h| self.visible_card(h))
                    .collect(),
            })
            .collect();
        let remembered = self
            .objects
            .knowledge(seat)
            .iter()
            .map(|fact| Revelation {
                card: fact.card.identity().key,
                owner: seat_index(fact.owner) as u8,
                zone_at_reveal: zone_name(fact.zone),
            })
            .collect();
        let terminal = self.outcome.map(|o| TerminalView {
            winner: o.winner.map(|s| seat_index(s) as u8),
            losses: o.losses.map(|r| {
                r.map(|r| match r {
                    terminal::LossReason::Life => "life",
                    terminal::LossReason::EmptyDraw => "empty_draw",
                    terminal::LossReason::Concession => "concession",
                })
            }),
        });
        Ok(PlayerView {
            schema_version: SCHEMA_VERSION,
            seat: seat_index(seat) as u8,
            life: self.life,
            hand_counts: [Seat::P0, Seat::P1].map(|s| self.objects.in_zone(Zone::Hand(s)).count()),
            library_counts: [Seat::P0, Seat::P1]
                .map(|s| self.objects.in_zone(Zone::Library(s)).count()),
            hand: hand.into_iter().map(|h| self.visible_card(h)).collect(),
            public_zones,
            remembered,
            starting_seat: seat_index(self.starting) as u8,
            turn: self
                .turn_position()
                .map(|(n, s, step)| (n, seat_index(s) as u8, step_name(step))),
            mana: self.mana(),
            acting_seat: self
                .decision
                .map(|d| d.actor)
                .or_else(|| self.turns.payment.as_ref().map(|p| p.actor))
                .or_else(|| self.turn_decision().map(|d| d.actor))
                .map(|s| seat_index(s) as u8),
            opening,
            terminal,
        })
    }
    /// Game-routed opening command. Rows refer to `observe(actor).opening`.
    /// The trusted caller binds the game and seat; this is not authentication.
    pub fn apply_opening_view(
        &mut self,
        actor: Seat,
        generation: u64,
        rows: &[usize],
    ) -> Result<(), ViewError> {
        if !self.work.is_empty() {
            return Err(ViewError::Unavailable);
        }
        let d = self.decision.ok_or(ViewError::Unavailable)?;
        if actor != d.actor {
            return Err(ViewError::WrongActor);
        }
        if generation != d.generation {
            return Err(ViewError::StaleDecision);
        }
        let selection = match d.kind {
            OpeningKind::KeepOrMulligan => {
                if rows.len() != 1 {
                    return Err(ViewError::InvalidSelection);
                }
                Selection::Choose(d.candidate(rows[0]))
            }
            OpeningKind::Bottom { count } => {
                if rows.len() != count {
                    return Err(ViewError::InvalidSelection);
                }
                let hand = self.view_hand(actor);
                let original = self.bottom_cards().expect("bottom decision");
                let mut candidates = Vec::with_capacity(rows.len());
                for row in rows {
                    let h = hand.get(*row).ok_or(ViewError::InvalidSelection)?;
                    let index = original.iter().position(|x| x == h).expect("same hand");
                    candidates.push(d.candidate(index));
                }
                Selection::Bottom(candidates)
            }
        };
        self.apply(
            actor,
            &OpeningAction {
                decision: d.id,
                selection,
            },
        )
        .map(|_| ())
        .map_err(|_| ViewError::InvalidSelection)
    }
    /// Privileged rules/test hook, never a policy operation or permission grant.
    /// Records a historical identity/zone fact only for the specified viewer.
    pub fn privileged_reveal_to(&mut self, seat: Seat, card: Handle) -> Result<(), StorageError> {
        self.objects.reveal_to(seat, card)
    }
}
#[cfg(test)]
#[path = "views_tests.rs"]
mod tests;
