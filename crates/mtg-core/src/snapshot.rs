//! Privileged, integrity-checked snapshots. See `doc/snapshot.md`.
use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SNAPSHOT_VERSION: u32 = 1;
const MAGIC: &str = "mtg-core-snapshot";

#[derive(Debug, PartialEq, Eq)]
pub enum RestoreError {
    Corrupt,
    UnsupportedVersion,
    IncompatibleEngine,
    IdentityExhausted,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    engine: String,
    sha256: String,
    payload: String,
}

// Remote deserialization keeps unchecked Game/ObjectStore constructors private.
#[derive(Deserialize)]
#[serde(remote = "Game", deny_unknown_fields)]
struct GameWire {
    #[serde(skip)]
    policy_revision: u64,
    #[serde(skip)]
    metric_work: Option<(u64, bool)>,
    #[serde(deserialize_with = "Option::deserialize")]
    outcome: Option<terminal::Outcome>,
    #[serde(deserialize_with = "Option::deserialize")]
    episode: Option<terminal::EpisodeId>,
    turns: turns::TurnState,
    work: VecDeque<Work>,
    #[serde(with = "crate::objects::StoreWire")]
    objects: ObjectStore,
    life: [i64; 2],
    #[serde(deserialize_with = "Option::deserialize")]
    decision: Option<OpeningDecision>,
    #[serde(deserialize_with = "Option::deserialize")]
    rng: Option<EpisodeRng>,
    generation: u64,
    starting: Seat,
    kept: [bool; 2],
    declarations: [Option<OpeningChoice>; 2],
    mulligans: [usize; 2],
    orders: [Option<Vec<Handle>>; 2],
    needs_bottom: [bool; 2],
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
/// Conservative source/data compatibility identity used by snapshots and run provenance.
pub fn engine() -> &'static str {
    static ENGINE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    ENGINE.get_or_init(|| {
        // Conservative compatibility: any core-source or pinned data change rejects
        // older artifacts, even if that change could have supported a migration.
        digest(
            concat!(
                include_str!("lib.rs"),
                include_str!("opening.rs"),
                include_str!("game.rs"),
                include_str!("work.rs"),
                include_str!("objects.rs"),
                include_str!("rng.rs"),
                include_str!("turns.rs"),
                include_str!("triggers.rs"),
                include_str!("mana.rs"),
                include_str!("casting.rs"),
                include_str!("activation.rs"),
                include_str!("cast_state.rs"),
                include_str!("targets.rs"),
                include_str!("combat.rs"),
                include_str!("terminal.rs"),
                include_str!("views.rs"),
                include_str!("policy.rs"),
                include_str!("card_identities.rs"),
                include_str!("card_definitions.rs"),
                include_str!("snapshot.rs"),
                include_str!("replay.rs"),
                include_str!("played_replay.rs"),
                include_str!("actions.rs"),
                include_str!("../../../data/cards/foundations_micro_v1.json"),
                include_str!("../../../data/rules/cr-2026-09-25.json")
            )
            .as_bytes(),
        )
    })
}

impl Game {
    /// Owned full-information bytes, including RNG and pending private work.
    /// Never send this artifact through a seat's observation channel.
    pub fn snapshot(&self) -> Vec<u8> {
        let payload = serde_json::to_string(self).expect("core state is JSON representable");
        serde_json::to_vec(&Envelope {
            format: MAGIC.into(),
            version: SNAPSHOT_VERSION,
            engine: engine().into(),
            sha256: digest(payload.as_bytes()),
            payload,
        })
        .expect("snapshot envelope is JSON representable")
    }
    /// Load an engine-produced artifact transactionally. Checksums detect
    /// accidental corruption, not forgery; only load trusted privileged saves.
    /// Success creates a fresh capability scope: reacquire handles/decision IDs.
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), RestoreError> {
        let envelope: Envelope =
            serde_json::from_slice(bytes).map_err(|_| RestoreError::Corrupt)?;
        if envelope.format != MAGIC {
            return Err(RestoreError::Corrupt);
        }
        if envelope.version != SNAPSHOT_VERSION {
            return Err(RestoreError::UnsupportedVersion);
        }
        if envelope.engine != engine() {
            return Err(RestoreError::IncompatibleEngine);
        }
        if digest(envelope.payload.as_bytes()) != envelope.sha256 {
            return Err(RestoreError::Corrupt);
        }
        let mut value: serde_json::Value =
            serde_json::from_str(&envelope.payload).map_err(|_| RestoreError::Corrupt)?;
        // Parse and validate before allocating a fresh identity or replacing self.
        let candidate = GameWire::deserialize(&value).map_err(|_| RestoreError::Corrupt)?;
        if !candidate.objects.snapshot_valid() || !candidate.snapshot_work_valid() {
            return Err(RestoreError::Corrupt);
        }
        let revision = self
            .policy_revision
            .checked_add(1)
            .ok_or(RestoreError::IdentityExhausted)?;
        let old = candidate.objects.scope();
        let fresh = ObjectStore::new()
            .map_err(|_| RestoreError::IdentityExhausted)?
            .scope();
        rebase(&mut value, old, fresh)?;
        value["objects"]["id"] = fresh.into();
        let mut candidate = GameWire::deserialize(value).map_err(|_| RestoreError::Corrupt)?;
        candidate.policy_revision = revision;
        *self = candidate;
        Ok(())
    }
    fn snapshot_work_valid(&self) -> bool {
        if self.mulligans.iter().any(|&n| n > 7)
            || (!self.work.is_empty() && (self.decision.is_some() || self.rng.is_none()))
        {
            return false;
        }
        let target_decision = self
            .turns
            .decision
            .filter(|d| d.kind == turns::TurnKind::TriggerTarget);
        if target_decision.is_some() != !self.turns.trigger_placement.is_empty() {
            return false;
        }
        if let Some(d) = target_decision {
            let rows = &self.turns.trigger_placement;
            let Some(Some(p)) = self.turns.pending_triggers.get(rows[0]) else {
                return false;
            };
            let Some((_, active, _)) = self.turns.position else {
                return false;
            };
            let next_actor = [active, turns::opponent(active)]
                .into_iter()
                .find(|&s| !self.trigger_candidates(s).is_empty());
            if !self.work.is_empty()
                || self.turns.trigger_return.is_none()
                || next_actor != Some(d.actor)
                || p.controller != d.actor
                || !matches!(
                    p.kind,
                    super::triggers::TriggerKind::Pyromancer { target: None }
                )
                || rows.len() != self.trigger_candidates(d.actor).len()
                || rows.iter().enumerate().any(|(i, r)| {
                    rows[..i].contains(r)
                        || !self
                            .turns
                            .pending_triggers
                            .get(*r)
                            .is_some_and(|p| p.as_ref().is_some_and(|p| p.controller == d.actor))
                })
            {
                return false;
            }
        }
        if self.turns.triggered.iter().any(|a| {
            matches!(
                a.declaration.kind,
                super::triggers::TriggerKind::Pyromancer { target: None }
            )
        }) {
            return false;
        }
        if let Some(d) = self
            .turns
            .decision
            .filter(|d| d.kind == turns::TurnKind::TriggerOrder)
        {
            let Some((_, active, _)) = self.turns.position else {
                return false;
            };
            let actor = [active, turns::opponent(active)]
                .into_iter()
                .find(|&actor| !self.trigger_candidates(actor).is_empty());
            if actor != Some(d.actor) || self.turns.trigger_return.is_none() {
                return false;
            }
        }
        let mut placement_rows = Vec::new();
        let mut boundary = false;
        for w in &self.work {
            match w {
                Work::PlaceTrigger(row) => {
                    if boundary || placement_rows.contains(row) {
                        return false;
                    }
                    placement_rows.push(*row);
                }
                Work::TriggerBoundary => {
                    if boundary {
                        return false;
                    }
                    boundary = true;
                }
                _ if boundary || !placement_rows.is_empty() => return false,
                _ => {}
            }
        }
        if !placement_rows.is_empty() && !boundary {
            return false;
        }
        if boundary && self.work.len() != placement_rows.len() + 1 {
            return false;
        }
        if let Some(first) = placement_rows.first() {
            let Some(Some(p)) = self.turns.pending_triggers.get(*first) else {
                return false;
            };
            let controller = p.controller;
            let expected = self.trigger_candidates(controller);
            if expected.len() != placement_rows.len()
                || !placement_rows.iter().all(|r| expected.contains(r))
            {
                return false;
            }
        }
        self.work.iter().all(|w| match w {
            Work::PlaceTrigger(row) => {
                self.turns.decision.is_none()
                    && self.turns.trigger_return.is_some()
                    && self
                        .turns
                        .pending_triggers
                        .get(*row)
                        .is_some_and(Option::is_some)
            }
            Work::TriggerBoundary => {
                self.turns.decision.is_none() && self.turns.trigger_return.is_some()
            }
            Work::Turn(w) => {
                use turns::TurnWork;
                self.turns.decision.is_none()
                    && match w {
                        TurnWork::Boundary(turn, _, _) => *turn > 0,
                        TurnWork::CleanupEnd { next_turn, active } => {
                            self.turns.position.is_some_and(|(turn, seat, step)| {
                                turn.checked_add(1) == Some(*next_turn)
                                    && seat == *active
                                    && matches!(step, turns::Step::End | turns::Step::Cleanup)
                            })
                        }
                        TurnWork::Move(h, zone) => {
                            self.objects.get(*h).is_ok()
                                && matches!(zone, Zone::Hand(_) | Zone::Graveyard(_))
                        }
                        TurnWork::Untap(h) => self
                            .objects
                            .get(*h)
                            .is_ok_and(|o| o.zone == Zone::Battlefield),
                        TurnWork::Wake(index) => *index < self.turns.sick.len(),
                        TurnWork::ExpireTrample => !self.turns.trample.is_empty(),
                        TurnWork::ExpireHaste => !self.turns.haste.is_empty(),
                        TurnWork::Expire => !self.turns.modifications.is_empty(),
                        TurnWork::Ready { .. } => self.turns.position.is_some(),
                    }
            }
            Work::CombatLife(_) | Work::FinishCombat => {
                self.turns
                    .position
                    .is_some_and(|(_, _, step)| step == turns::Step::CombatDamage)
                    && self.turns.decision.is_none()
            }
            Work::GrantHaste(h) | Work::GrantTrample(h) => self.haste_target(*h),
            Work::RemoveAbility(h) => {
                self.turns.abilities.iter().any(|a| a.object == *h)
                    || self.turns.triggered.iter().any(|a| a.object == *h)
            }
            Work::TriggerLife(_) => {
                self.turns.decision.is_none()
                    && self
                        .turns
                        .triggered
                        .iter()
                        .any(|a| matches!(a.declaration.kind, triggers::TriggerKind::Archer))
            }
            Work::Modify(m) => self
                .objects
                .get(m.handle)
                .is_ok_and(|o| o.zone == Zone::Battlefield),
            Work::CreateGoblins { .. } => {
                self.turns.position.is_some() && self.turns.decision.is_none()
            }
            Work::SpellMove {
                handle,
                zone,
                controller,
            } => {
                self.objects.get(*handle).is_ok()
                    && matches!(
                        (zone, controller),
                        (Zone::Battlefield, Some(_)) | (Zone::Graveyard(_), None)
                    )
            }
            Work::FinishSpell { spell, .. } => self.turns.stack.last() == Some(spell),
            Work::Priority { .. } => self.turns.position.is_some() && self.turns.decision.is_none(),
            Work::Reset {
                seat,
                size,
                position,
                phase,
                ..
            } => {
                *seat < 2
                    && (1..=40).contains(size)
                    && *phase <= 2
                    && *position < if *phase == 2 { 7 } else { 40 }
            }
            Work::Redraw {
                old,
                shuffled,
                current,
                size,
                position,
                phase,
                ..
            } => {
                old.len() == 40
                    && shuffled.len() == 40
                    && current.len() <= 40
                    && (1..=40).contains(size)
                    && *phase <= 3
                    && *position < if *phase == 3 { 7 } else { 40 }
            }
            Work::Bottom {
                cards, position, ..
            } => !cards.is_empty() && cards.len() <= 7 && *position < cards.len(),
            Work::Advance => true,
        })
    }
}
fn rebase(value: &mut serde_json::Value, old: u64, new: u64) -> Result<(), RestoreError> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                if key == "store" || key == "scope" {
                    if value.as_u64() != Some(old) {
                        return Err(RestoreError::Corrupt);
                    }
                    *value = new.into();
                } else {
                    rebase(value, old, new)?;
                }
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                rebase(value, old, new)?;
            }
        }
        _ => {}
    }
    Ok(())
}
pub(super) fn opening_candidates<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<&'static [OpeningChoice], D::Error> {
    let v = Vec::<OpeningChoice>::deserialize(d)?;
    match v.as_slice() {
        [] => Ok(&[]),
        [OpeningChoice::Keep, OpeningChoice::Mulligan] => {
            Ok(&[OpeningChoice::Keep, OpeningChoice::Mulligan])
        }
        _ => Err(serde::de::Error::custom("invalid opening candidates")),
    }
}
pub(super) mod decks {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        decks: &[[CardId; 40]; 2],
        s: S,
    ) -> Result<S::Ok, S::Error> {
        [&decks[0][..], &decks[1][..]].serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<Box<[[CardId; 40]; 2]>, D::Error> {
        let rows = Vec::<Vec<CardId>>::deserialize(d)?;
        let rows = rows
            .into_iter()
            .map(|row| {
                row.try_into()
                    .map_err(|_| serde::de::Error::custom("deck length"))
            })
            .collect::<Result<Vec<[CardId; 40]>, D::Error>>()?;
        Ok(Box::new(
            rows.try_into()
                .map_err(|_| serde::de::Error::custom("seat count"))?,
        ))
    }
}
