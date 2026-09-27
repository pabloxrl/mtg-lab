//! Privileged complete M1 replay. See `doc/replay.md`.
use super::*;
use std::collections::BTreeMap;

pub const VERSION: u32 = 1;
const FORMAT: &str = "mtg-core-played-replay";
// The fixed M1 pool has 80 cards and factored choices, not subset enumeration.
const CAPACITY: usize = 256;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    version: u32,
    action_version: u32,
    engine: String,
    rules: String,
    cards: String,
    rng: String,
    shuffle: String,
    config_sha256: String,
    config: Config,
    master: u64,
    episode: u64,
    initial: Value,
    choices: Vec<Record>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    choice: actions::Record,
    after: Value,
}
// Historical references retain the witnessed incarnation after a zone change.
// This map is private execution scratch: its handle keys never enter artifacts.
#[derive(Default)]
struct Checkpoints {
    identities: BTreeMap<String, Value>,
}
impl Checkpoints {
    fn capture(&mut self, g: &Game) -> Value {
        let zones: Vec<_> = Zone::ALL.into_iter().map(|zone| {
            let objects: Vec<_> = g.objects.in_zone(zone).map(|h| {
                let o = g.objects.get(h).expect("live object");
                let (birth, incarnation) = g.objects.semantic_identity(h).expect("live identity");
                let identity = serde_json::json!({"birth":birth,"incarnation":incarnation,"card":o.card.identity().key,"owner":o.owner,"zone":zone});
                self.identities.insert(serde_json::to_string(&h).expect("handle JSON"),identity.clone());
                let creature = g.creature_state(h).map(|c| serde_json::json!({"power":c.power,"toughness":c.toughness,"damage":c.damage}));
                serde_json::json!({"identity":identity,"controller":o.controller,"tapped":o.tapped,"creature":creature})
            }).collect();
            serde_json::json!({"zone":zone,"objects":objects})
        }).collect();
        let mut turns = serde_json::to_value(&g.turns).expect("turn state JSON");
        self.normalize(&mut turns);
        // CardId is a compact internal ordinal; the artifact names the card.
        if let Some(r) = g.turns.last_resolution {
            turns["last_resolution"]["spell"] = r.spell.identity().key.into();
        }
        serde_json::json!({
            "life":g.life,"opening":checkpoint(g),"rng":g.rng,
            "zones":zones,"turns":turns,"outcome":g.outcome
        })
    }
    fn normalize(&self, value: &mut Value) {
        match value {
            Value::Object(m) if m.contains_key("store") && m.contains_key("slot") => {
                let h: Handle = serde_json::from_value(value.clone()).expect("core handle");
                *value = self
                    .identities
                    .get(&serde_json::to_string(&h).expect("handle JSON"))
                    .expect("all referenced incarnations witnessed at decision boundaries")
                    .clone();
            }
            Value::Object(m) => {
                // Decision IDs scope live commands, but are not semantic replay state.
                if m.get("id").is_some_and(|v| v.get("scope").is_some()) {
                    m.remove("id");
                }
                for v in m.values_mut() {
                    self.normalize(v);
                }
            }
            Value::Array(a) => {
                for v in a {
                    self.normalize(v);
                }
            }
            _ => (),
        }
    }
}
fn advance(g: &mut Game) -> Result<(), ReplayError> {
    if g.outcome.is_none() && g.decision.is_none() && g.turns.position.is_none() {
        g.start_turns().map_err(ReplayError::Turn)?;
    }
    Ok(())
}
fn apply_record(g: &mut Game, choice: &actions::Record, index: usize) -> Result<(), ReplayError> {
    if g.outcome.is_some() {
        return Err(ReplayError::UnconsumedChoice { index });
    }
    actions::apply(
        g,
        &serde_json::to_vec(choice).expect("action JSON"),
        CAPACITY,
    )
    .map_err(|reason| ReplayError::SemanticChoice { index, reason })?;
    advance(g)
}
fn finished(g: &Game, index: usize) -> Result<(), ReplayError> {
    if g.outcome.is_none() {
        Err(ReplayError::MissingChoice { index })
    } else {
        Ok(())
    }
}
/// Execute an entire normal-reset game, with no default choices. Opening
/// completion enters the first upkeep automatically; internal work is drained.
/// Artifacts contain both seats' secrets and must not be used as seat exports.
pub fn record(
    config: &Config,
    master: u64,
    episode: u64,
    choices: &[actions::Record],
) -> Result<Vec<u8>, ReplayError> {
    let mut g = start(config, master, episode)?;
    let mut checkpoints = Checkpoints::default();
    let initial = checkpoints.capture(&g);
    let mut records = Vec::with_capacity(choices.len());
    for (index, choice) in choices.iter().enumerate() {
        apply_record(&mut g, choice, index)?;
        records.push(Record {
            choice: choice.clone(),
            after: checkpoints.capture(&g),
        });
    }
    finished(&g, choices.len())?;
    let e = Envelope {
        format: FORMAT.into(),
        version: VERSION,
        action_version: actions::ACTION_VERSION,
        engine: snapshot::engine().into(),
        rules: rules(),
        cards: cards_pin(),
        rng: super::super::VERSION.into(),
        shuffle: SHUFFLE_VERSION.into(),
        config_sha256: config_hash(config),
        config: config.clone(),
        master,
        episode,
        initial,
        choices: records,
    };
    Ok(serde_json::to_vec(&e).expect("replay JSON"))
}
/// Re-execute in a fresh game and reject the first differing semantic field,
/// incompatible pin, invalid action, or incomplete/extra choice stream.
/// This detects corruption, not coordinated forgery of actions and checkpoints.
pub fn verify(bytes: &[u8]) -> Result<Game, ReplayError> {
    let e: Envelope = serde_json::from_slice(bytes).map_err(|_| ReplayError::Malformed)?;
    for (field, valid) in [
        ("format", e.format == FORMAT),
        ("version", e.version == VERSION),
        (
            "action_version",
            e.action_version == actions::ACTION_VERSION,
        ),
        ("engine", e.engine == snapshot::engine()),
        ("rules", e.rules == rules()),
        ("cards", e.cards == cards_pin()),
        ("rng", e.rng == super::super::VERSION),
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
    let mut checkpoints = Checkpoints::default();
    compare_value(&e.initial, &checkpoints.capture(&g), 0)?;
    for (index, r) in e.choices.iter().enumerate() {
        apply_record(&mut g, &r.choice, index)?;
        compare_value(&r.after, &checkpoints.capture(&g), index + 1)?;
    }
    finished(&g, e.choices.len())?;
    Ok(g)
}
fn compare_value(expected: &Value, actual: &Value, checkpoint: usize) -> Result<(), ReplayError> {
    if let Some((path, expected, actual)) = difference(expected, actual, String::new()) {
        return Err(ReplayError::Divergence {
            checkpoint,
            path,
            expected,
            actual,
        });
    }
    Ok(())
}
