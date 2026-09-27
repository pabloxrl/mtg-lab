#![doc = include_str!("../../../doc/replay.md")]
use super::*;
use serde::{Deserialize, Serialize};

pub const REPLAY_VERSION: u32 = 1;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardRef {
    pub card: String,
    /// Zero-based occurrence of this card identity in the current hand.
    pub occurrence: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Keep {},
    Mulligan {},
    Bottom { cards: Vec<CardRef> },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub actor: Seat,
    pub action: Action,
}
#[derive(Debug, PartialEq, Eq)]
pub enum ReplayError {
    Malformed,
    Turn(turns::TurnError),
    SemanticChoice {
        index: usize,
        reason: actions::ActionError,
    },
    Incompatible {
        field: String,
    },
    InvalidConfig(ResetError),
    Storage(StorageError),
    InvalidChoice {
        index: usize,
        reason: ApplyError,
    },
    MissingChoice {
        index: usize,
    },
    UnconsumedChoice {
        index: usize,
    },
    Divergence {
        checkpoint: usize,
        path: String,
        expected: String,
        actual: String,
    },
}

use serde_json::Value;
use sha2::{Digest, Sha256};
const FORMAT: &str = "mtg-core-opening-replay";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    engine: String,
    rules: String,
    cards: String,
    rng: String,
    shuffle: String,
    config_sha256: String,
    config: Config,
    master: u64,
    episode: u64,
    initial: Checkpoint,
    choices: Vec<Record>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    choice: Choice,
    after: Checkpoint,
}
/// Semantic opening state, without process-local handles or candidate indices.
/// Private declarations and RNG are included to detect a changed action even
/// before the next seat declares or any publicly observable effect occurs.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Checkpoint {
    life: [i64; 2],
    hands: [Vec<String>; 2],
    libraries: [Vec<String>; 2],
    starting: Seat,
    kept: [bool; 2],
    declarations: [Option<OpeningChoice>; 2],
    mulligans: [usize; 2],
    needs_bottom: [bool; 2],
    decision: Value,
    rng: Value,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn rules() -> String {
    digest(include_bytes!("../../../data/rules/cr-2026-09-25.json"))
}
fn cards_pin() -> String {
    digest(include_bytes!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
}
fn config_hash(c: &Config) -> String {
    digest(&serde_json::to_vec(c).expect("config JSON"))
}
fn checkpoint(g: &Game) -> Checkpoint {
    let zone = |z| {
        g.objects
            .in_zone(z)
            .map(|h| {
                g.objects
                    .get(h)
                    .expect("live zone card")
                    .card
                    .identity()
                    .key
                    .to_owned()
            })
            .collect()
    };
    Checkpoint {
        life: g.life,
        hands: [zone(Zone::Hand(Seat::P0)), zone(Zone::Hand(Seat::P1))],
        libraries: [zone(Zone::Library(Seat::P0)), zone(Zone::Library(Seat::P1))],
        starting: g.starting,
        kept: g.kept,
        declarations: g.declarations,
        mulligans: g.mulligans,
        needs_bottom: g.needs_bottom,
        decision: serde_json::to_value(g.decision.map(|d| (d.actor, d.kind)))
            .expect("decision JSON"),
        rng: serde_json::to_value(&g.rng).expect("RNG JSON"),
    }
}
fn start(config: &Config, master: u64, episode: u64) -> Result<Game, ReplayError> {
    let mut g = Game::new().map_err(ReplayError::Storage)?;
    g.reset(config, master, episode)
        .map_err(ReplayError::InvalidConfig)?;
    Ok(g)
}
fn apply(g: &mut Game, c: &Choice, index: usize) -> Result<(), ReplayError> {
    let d = g.decision.ok_or(ReplayError::UnconsumedChoice { index })?;
    let error = |reason| ReplayError::InvalidChoice { index, reason };
    if c.actor != d.actor {
        return Err(error(ApplyError::WrongActor));
    }
    let selection = match (&c.action, d.kind) {
        (Action::Keep {}, OpeningKind::KeepOrMulligan) => Selection::Choose(d.candidate(0)),
        (Action::Mulligan {}, OpeningKind::KeepOrMulligan) => Selection::Choose(d.candidate(1)),
        (Action::Bottom { cards }, OpeningKind::Bottom { .. }) => {
            let hand = g.bottom_cards().expect("bottom decision");
            let mut selected = Vec::with_capacity(cards.len());
            for card in cards {
                let position = hand
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| {
                        g.objects.get(**h).expect("hand").card.identity().key == card.card
                    })
                    .nth(card.occurrence)
                    .map(|(i, _)| i)
                    .ok_or_else(|| error(ApplyError::IllegalCandidate))?;
                selected.push(d.candidate(position));
            }
            Selection::Bottom(selected)
        }
        _ => return Err(error(ApplyError::WrongKind)),
    };
    g.apply(
        c.actor,
        &OpeningAction {
            decision: d.id,
            selection,
        },
    )
    .map_err(error)?;
    Ok(())
}
fn complete(g: &Game, index: usize) -> Result<(), ReplayError> {
    if g.decision.is_some() {
        Err(ReplayError::MissingChoice { index })
    } else {
        Ok(())
    }
}
/// Run a complete opening script and capture an owned privileged replay.
/// Incomplete/extra/illegal choices fail; no decisions are supplied by default.
pub fn record(
    config: &Config,
    master: u64,
    episode: u64,
    choices: &[Choice],
) -> Result<Vec<u8>, ReplayError> {
    let mut g = start(config, master, episode)?;
    let initial = checkpoint(&g);
    let mut records = Vec::with_capacity(choices.len());
    for (index, choice) in choices.iter().enumerate() {
        apply(&mut g, choice, index)?;
        records.push(Record {
            choice: choice.clone(),
            after: checkpoint(&g),
        });
    }
    complete(&g, choices.len())?;
    let envelope = Envelope {
        format: FORMAT.into(),
        version: REPLAY_VERSION,
        engine: snapshot::engine().into(),
        rules: rules(),
        cards: cards_pin(),
        rng: VERSION.into(),
        shuffle: SHUFFLE_VERSION.into(),
        config_sha256: config_hash(config),
        config: config.clone(),
        master,
        episode,
        initial,
        choices: records,
    };
    Ok(serde_json::to_vec(&envelope).expect("replay JSON"))
}
/// Replay through the core and report the first mismatch. Success returns a
/// fresh game at the end of opening; no caller-owned game is mutated on error.
/// This detects corruption, not adversarial rewriting of both actions and all
/// expected checkpoints. Authenticate artifacts at the storage boundary.
pub fn verify(bytes: &[u8]) -> Result<Game, ReplayError> {
    let e: Envelope = serde_json::from_slice(bytes).map_err(|_| ReplayError::Malformed)?;
    for (field, valid) in [
        ("format", e.format == FORMAT),
        ("version", e.version == REPLAY_VERSION),
        ("engine", e.engine == snapshot::engine()),
        ("rules", e.rules == rules()),
        ("cards", e.cards == cards_pin()),
        ("rng", e.rng == VERSION),
        ("shuffle", e.shuffle == SHUFFLE_VERSION),
        ("config_sha256", e.config_sha256 == config_hash(&e.config)),
    ] {
        if !valid {
            return Err(ReplayError::Incompatible {
                field: field.into(),
            });
        }
    }
    let mut g = start(&e.config, e.master, e.episode)?;
    compare(&e.initial, &checkpoint(&g), 0)?;
    for (index, record) in e.choices.iter().enumerate() {
        apply(&mut g, &record.choice, index)?;
        compare(&record.after, &checkpoint(&g), index + 1)?;
    }
    complete(&g, e.choices.len())?;
    Ok(g)
}
fn compare(
    expected: &Checkpoint,
    actual: &Checkpoint,
    checkpoint: usize,
) -> Result<(), ReplayError> {
    let a = serde_json::to_value(expected).expect("checkpoint JSON");
    let b = serde_json::to_value(actual).expect("checkpoint JSON");
    if let Some((path, expected, actual)) = difference(&a, &b, String::new()) {
        return Err(ReplayError::Divergence {
            checkpoint,
            path,
            expected,
            actual,
        });
    }
    Ok(())
}
fn difference(a: &Value, b: &Value, path: String) -> Option<(String, String, String)> {
    if a == b {
        return None;
    }
    match (a, b) {
        (Value::Object(a), Value::Object(b)) if a.keys().eq(b.keys()) => {
            for (key, a) in a {
                if let Some(d) = difference(a, &b[key], format!("{path}/{key}")) {
                    return Some(d);
                }
            }
        }
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => {
            for (i, (a, b)) in a.iter().zip(b).enumerate() {
                if let Some(d) = difference(a, b, format!("{path}/{i}")) {
                    return Some(d);
                }
            }
        }
        _ => {}
    }
    Some((path, a.to_string(), b.to_string()))
}

#[path = "played_replay.rs"]
pub mod played;
