//! CR 103 reset prefix, test-only privileged client. Literal oracle is external.
use super::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema_version: u32,
    family: String,
    pins: BTreeMap<String, String>,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    starter: u8,
    decks: Vec<Deck>,
    chance: Vec<Event>,
    choices: Vec<Value>,
    stop: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Deck {
    deck: String,
    occurrences: Vec<String>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Event {
    sequence: usize,
    kind: String,
    actor: usize,
    before: Vec<String>,
    after: Vec<String>,
}
fn require(ok: bool, path: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(format!("first divergence: {path}"))
    }
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-reset.json"
    ))
    .unwrap()
}
fn validate(doc: Value) -> Result<Envelope, String> {
    let doc: Envelope =
        serde_json::from_value(doc).map_err(|e| format!("first divergence: /envelope {e}"))?;
    require(doc.schema_version == 1, "/schema_version")?;
    require(doc.family == "reset", "/family")?;
    let pins: BTreeMap<_, _> = [
        (
            "data/cards/foundations_micro_v1.json",
            include_bytes!("../../../data/cards/foundations_micro_v1.json").as_slice(),
        ),
        (
            "data/rules/cr-2026-09-25.json",
            include_bytes!("../../../data/rules/cr-2026-09-25.json").as_slice(),
        ),
        (
            "references/xmage/pins.json",
            include_bytes!("../../../references/xmage/pins.json").as_slice(),
        ),
    ]
    .into_iter()
    .map(|(p, b)| (p.to_string(), format!("{:x}", Sha256::digest(b))))
    .collect();
    require(doc.pins == pins, "/pins")?;
    require(!doc.cases.is_empty(), "/cases empty")?;
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let mut ids = BTreeSet::new();
    for case in &doc.cases {
        require(
            !case.id.is_empty() && ids.insert(case.id.clone()),
            "/case id",
        )?;
        require(case.starter < 2, "/starter")?;
        require(
            case.stop == "first_declaration",
            "/stop unsupported callback",
        )?;
        require(
            case.choices.is_empty(),
            "/choices unsupported mulligan/ongoing callback",
        )?;
        require(case.decks.len() == 2, "/decks")?;
        require(case.chance.len() == 2, "/chance length")?;
        for (seat, deck) in case.decks.iter().enumerate() {
            let frozen = manifest["decks"]
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["id"].as_str() == Some(deck.deck.as_str()))
                .ok_or_else(|| "first divergence: /deck unknown".to_string())?;
            let mut canonical = Vec::new();
            for entry in frozen["cards"].as_array().unwrap() {
                for copy in 0..entry["copies"].as_u64().unwrap() {
                    canonical.push(format!(
                        "{seat}/{}/{copy}",
                        entry["card_id"].as_str().unwrap()
                    ));
                }
            }
            require(deck.occurrences == canonical, "/deck occurrences")?;
            let event = &case.chance[seat];
            require(event.sequence == seat, "/chance sequence")?;
            require(event.actor == seat, "/chance actor")?;
            require(event.kind == "initial_shuffle", "/chance kind")?;
            require(event.before == canonical, "/chance before")?;
            let mut sorted = event.after.clone();
            sorted.sort();
            canonical.sort();
            require(sorted == canonical, "/chance permutation")?;
        }
    }
    Ok(doc)
}
struct Tape<'a> {
    case: &'a Case,
    births: BTreeMap<u64, String>,
    handles: BTreeMap<String, Handle>,
    used: Vec<Event>,
}
impl Tape<'_> {
    fn identities(&self, objects: &ObjectStore, seat: Seat) -> Vec<String> {
        objects
            .in_zone(Zone::Library(seat))
            .map(|h| self.births[&objects.semantic_identity(h).unwrap().0].clone())
            .collect()
    }
}
impl opening::ResetChance for Tape<'_> {
    fn created(&mut self, objects: &ObjectStore, seat: Seat, index: usize, handle: Handle) {
        let occurrence = &self.case.decks[seat_index(seat)].occurrences[index];
        let (birth, incarnation) = objects.semantic_identity(handle).unwrap();
        assert_eq!(incarnation, 0);
        assert_eq!(
            objects.get(handle).unwrap().card.identity().key,
            occurrence.split('/').nth(1).unwrap()
        );
        assert!(self.births.insert(birth, occurrence.clone()).is_none());
        assert!(self.handles.insert(occurrence.clone(), handle).is_none());
    }
    fn shuffle(&mut self, objects: &mut ObjectStore, seat: Seat) {
        let event = &self.case.chance[self.used.len()];
        assert_eq!(
            event.actor,
            seat_index(seat),
            "first divergence: runtime shuffle actor"
        );
        let before = self.identities(objects, seat);
        assert_eq!(
            before, event.before,
            "first divergence: runtime shuffle multiset/order"
        );
        let order: Vec<_> = event.after.iter().map(|id| self.handles[id]).collect();
        objects.reorder(Zone::Library(seat), &order);
        self.used.push(Event {
            sequence: self.used.len(),
            kind: "initial_shuffle".into(),
            actor: seat_index(seat),
            before,
            after: self.identities(objects, seat),
        });
    }
}
fn execute(g: &mut Game, case: &Case) -> Value {
    let config = Config {
        starting_seat: case.starter,
        seats: case
            .decks
            .iter()
            .map(|d| DeckConfig::new(&d.deck))
            .collect(),
        ..Config::default()
    };
    let mut tape = Tape {
        case,
        births: BTreeMap::new(),
        handles: BTreeMap::new(),
        used: vec![],
    };
    let decision = g
        .reset_with_occurrence_chance(&config, 0, 0, &mut tape)
        .unwrap();
    assert_eq!(decision.kind, OpeningKind::KeepOrMulligan);
    assert_eq!(g.outcome(), None);
    assert_eq!(g.objects.slot_count(), 80);
    assert_eq!(tape.used.len(), case.chance.len());
    assert_eq!(
        g.rng.as_ref().unwrap(),
        &EpisodeRng::new(VERSION, 0, 0, Stream::Environment).unwrap()
    );
    let mut hands = vec![];
    let mut libraries = vec![];
    let mut objects = serde_json::Map::new();
    for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
        for (zone, name) in [(Zone::Hand(seat), "hand"), (Zone::Library(seat), "library")] {
            let mut ids = vec![];
            for h in g.objects.in_zone(zone) {
                let (birth, incarnation) = g.objects.semantic_identity(h).unwrap();
                assert_eq!(incarnation, u64::from(name == "hand"));
                let id = tape.births[&birth].clone();
                let copy: usize = id.split('/').nth(2).unwrap().parse().unwrap();
                let object = g.objects.get(h).unwrap();
                assert_eq!(object.owner, seat);
                objects.insert(
                    id.clone(),
                    json!({"seat":i,"card":object.card.identity().key,"copy":copy,"zone":name}),
                );
                ids.push(id);
            }
            if name == "hand" {
                ids.sort();
                hands.push(ids);
            } else {
                libraries.push(ids);
            }
        }
    }
    json!({"completion":"prefix","boundary":"first_declaration","starting_seat":seat_index(g.starting),"turn_active_seat":g.turn_position().map(|(_,seat,_)| seat_index(seat)),"declaration_seat":seat_index(decision.actor),"life":g.life(),"hand":hands,"library":libraries,"occurrences":objects,"consumed_chance":tape.used,"consumed_choices":[]})
}
fn run(g: &mut Game, case: &Value) -> Value {
    let mut doc = fixture();
    doc["cases"] = json!([case]);
    execute(g, &validate(doc).unwrap().cases[0])
}
fn consume(g: &mut Game, doc: Value) -> Result<BTreeMap<String, Value>, String> {
    let doc = validate(doc)?;
    Ok(doc
        .cases
        .iter()
        .map(|case| (case.id.clone(), execute(g, case)))
        .collect())
}
#[test]
fn full_pool_reset_literal_checkpoints() {
    let doc = fixture();
    validate(doc.clone()).unwrap();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-reset-expectations.json"
    ))
    .unwrap();
    let mut g = Game::new().unwrap();
    let mut points = BTreeMap::new();
    let mut repeated = BTreeMap::new();
    let mut stale_handles_rejected = 0;
    for case in doc["cases"].as_array().unwrap() {
        let actual = run(&mut g, case);
        assert_eq!(
            actual["library"],
            expected[case["id"].as_str().unwrap()]["library"],
            "creation-bound occurrence positions"
        );
        assert_eq!(actual, expected[case["id"].as_str().unwrap()]);
        let handles: Vec<_> = [
            Zone::Hand(Seat::P0),
            Zone::Hand(Seat::P1),
            Zone::Library(Seat::P0),
            Zone::Library(Seat::P1),
        ]
        .into_iter()
        .flat_map(|zone| g.objects.in_zone(zone))
        .collect();
        assert_eq!(handles.len(), 80);
        let again = run(&mut g, case);
        assert_eq!(actual, again);
        for h in handles {
            assert_eq!(g.objects.get(h), Err(StorageError::InvalidHandle));
            stale_handles_rejected += 1;
        }
        let id = case["id"].as_str().unwrap().to_string();
        points.insert(id.clone(), actual);
        repeated.insert(id, again);
    }
    let negatives: Value = match std::env::var("MTG_FULL_POOL_NEGATIVES") {
        Ok(path) => serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap(),
        Err(std::env::VarError::NotPresent) => serde_json::from_str(include_str!(
            "../../../fixtures/reference/full-pool-reset-negative-inputs.json"
        ))
        .unwrap(),
        Err(error) => panic!("invalid negative-input path: {error}"),
    };
    let mut rejections = BTreeMap::new();
    for (name, bad) in negatives.as_object().unwrap() {
        let before = format!("{g:?}");
        let error = consume(&mut g, bad.clone()).unwrap_err();
        assert!(error.starts_with("first divergence:"));
        assert_eq!(
            format!("{g:?}"),
            before,
            "invalid envelope must preserve game/RNG"
        );
        rejections.insert(name, error);
    }
    assert_eq!(rejections.len(), 19);
    if let Ok(path) = std::env::var("MTG_FULL_POOL_OUTPUT") {
        let report = json!({"checkpoints":points,"repeat_checkpoints":repeated,"stale_handles_rejected":stale_handles_rejected,"rejections":rejections});
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    }
}

#[test]
fn full_pool_reset_same_name_copies_cross_hand_boundary() {
    let doc = fixture();
    let mut g = Game::new().unwrap();
    let normal = run(&mut g, &doc["cases"][0]);
    let swapped = run(&mut g, &doc["cases"][1]);
    assert_ne!(
        normal["hand"][0], swapped["hand"][0],
        "same-name copies must remain distinct at creation"
    );
}

#[test]
fn full_pool_reset_retains_legacy_card_projection() {
    // CR 103: changing physical-copy identity must not change which card names
    // an otherwise identical top-first permutation draws in the existing API.
    let doc = fixture();
    for case in doc["cases"].as_array().unwrap() {
        let mut occurrence_game = Game::new().unwrap();
        let point = run(&mut occurrence_game, case);
        let mut legacy = Game::new().unwrap();
        let mut config = Config {
            starting_seat: case["starter"].as_u64().unwrap() as u8,
            ..Config::default()
        };
        for i in 0..2 {
            config.seats[i] = DeckConfig::new(case["decks"][i]["deck"].as_str().unwrap());
            config.seats[i].order = Some(
                case["chance"][i]["after"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_str().unwrap().split('/').nth(1).unwrap().to_string())
                    .collect(),
            );
        }
        let decision = legacy.reset(&config, 0, 0).unwrap();
        assert_eq!(decision.actor, occurrence_game.decision().unwrap().actor);
        assert_eq!(legacy.life(), occurrence_game.life());
        for (i, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            for (zone, field) in [(Zone::Hand(seat), "hand"), (Zone::Library(seat), "library")] {
                let mut old: Vec<_> = legacy
                    .objects
                    .in_zone(zone)
                    .map(|h| legacy.objects.get(h).unwrap().card.identity().key)
                    .collect();
                let mut new: Vec<_> = point[field][i]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|id| id.as_str().unwrap().split('/').nth(1).unwrap())
                    .collect();
                if field == "hand" {
                    old.sort();
                    new.sort();
                }
                assert_eq!(old, new);
            }
        }
    }
}

include!("mulligan_reference_tests.rs");
