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
    outcome: Option<terminal::Outcome>,
    episode: Option<terminal::EpisodeId>,
    turns: turns::TurnState,
    work: VecDeque<Work>,
    #[serde(with = "crate::objects::StoreWire")]
    objects: ObjectStore,
    life: [i64; 2],
    decision: Option<OpeningDecision>,
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
pub(super) fn engine() -> &'static str {
    static ENGINE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    ENGINE.get_or_init(|| {
        // Conservative compatibility: any core-source or pinned data change rejects
        // older artifacts, even if that change could have supported a migration.
        digest(
            concat!(
                include_str!("lib.rs"),
                include_str!("opening.rs"),
                include_str!("objects.rs"),
                include_str!("rng.rs"),
                include_str!("turns.rs"),
                include_str!("mana.rs"),
                include_str!("casting.rs"),
                include_str!("targets.rs"),
                include_str!("combat.rs"),
                include_str!("terminal.rs"),
                include_str!("views.rs"),
                include_str!("card_identities.rs"),
                include_str!("snapshot.rs"),
                include_str!("replay.rs"),
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
        let old = candidate.objects.scope();
        let fresh = ObjectStore::new()
            .map_err(|_| RestoreError::IdentityExhausted)?
            .scope();
        rebase(&mut value, old, fresh)?;
        value["objects"]["id"] = fresh.into();
        let candidate = GameWire::deserialize(value).map_err(|_| RestoreError::Corrupt)?;
        *self = candidate;
        Ok(())
    }
    fn snapshot_work_valid(&self) -> bool {
        if self.mulligans.iter().any(|&n| n > 7)
            || (!self.work.is_empty() && (self.decision.is_some() || self.rng.is_none()))
        {
            return false;
        }
        self.work.iter().all(|w| match w {
            Work::CombatLife(_) | Work::FinishCombat => {
                self.turns
                    .position
                    .is_some_and(|(_, _, step)| step == turns::Step::CombatDamage)
                    && self.turns.decision.is_none()
            }
            Work::Modify(m) => self
                .objects
                .get(m.handle)
                .is_ok_and(|o| o.zone == Zone::Battlefield),
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
