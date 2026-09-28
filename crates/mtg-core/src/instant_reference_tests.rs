//! Test-only synthetic upkeep adapter. No rule implementation or expected-state calculation.
use super::*;
use mana::Color;
use serde_json::{Value, json};
use targets::Effect;
use turns::{Step, TurnAction, TurnKind, TurnSelection};

fn fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/reference/instant-responses.json"
    ))
    .unwrap()
}
fn expectations() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/reference/instant-expectations.json"
    ))
    .unwrap()
}
fn seat(v: &Value) -> Seat {
    match v.as_u64().expect("seat") {
        0 => Seat::P0,
        1 => Seat::P1,
        _ => panic!("unknown seat"),
    }
}
fn text(v: &Value) -> &str {
    v.as_str().expect("string")
}
fn keys(v: &Value, expected: &[&str]) {
    let mut actual: Vec<_> = v
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    let mut expected = expected.to_vec();
    actual.sort();
    expected.sort();
    assert_eq!(actual, expected, "missing/extra fields");
}
struct Adapter {
    game: Game,
    objects: Vec<Value>,
    damage: Vec<Value>,
}
impl Adapter {
    fn handle(&self, id: &str) -> Handle {
        let spec = self
            .objects
            .iter()
            .find(|o| o["id"] == id)
            .expect("unknown object");
        let mut found = vec![];
        for zone in [
            Zone::Battlefield,
            Zone::Stack,
            Zone::Hand(Seat::P0),
            Zone::Hand(Seat::P1),
            Zone::Graveyard(Seat::P0),
            Zone::Graveyard(Seat::P1),
        ] {
            for h in self.game.objects.in_zone(zone) {
                let o = self.game.objects.get(h).unwrap();
                if o.card.identity().key == text(&spec["card"]) && o.owner == seat(&spec["owner"]) {
                    found.push(h);
                }
            }
        }
        assert_eq!(found.len(), 1, "ambiguous/missing semantic object");
        found[0]
    }
    fn id(&self, handle: Handle) -> &str {
        let o = self.game.objects.get(handle).unwrap();
        text(
            &self
                .objects
                .iter()
                .find(|v| text(&v["card"]) == o.card.identity().key && seat(&v["owner"]) == o.owner)
                .unwrap()["id"],
        )
    }
    fn checkpoint(&self, name: &str) -> Value {
        let g = &self.game;
        assert_eq!(g.turn_position(), Some((1, Seat::P0, Step::Upkeep)));
        let d = g
            .turn_decision()
            .expect("checkpoint must be settled priority");
        assert_eq!(d.kind, TurnKind::Priority);
        let objects: Vec<_> = self.objects.iter().map(|spec| {
            let h = self.handle(text(&spec["id"])); let o = g.objects.get(h).unwrap();
            let zone = match o.zone { Zone::Battlefield => "battlefield", Zone::Stack => "stack", Zone::Hand(_) => "hand", Zone::Graveyard(_) => "graveyard", _ => panic!("unsupported zone") };
            let c = (o.zone == Zone::Battlefield).then(|| g.creature_state(h)).flatten();
            json!({"id": spec["id"], "card": spec["card"], "owner": spec["owner"], "zone": zone, "tapped": o.tapped,
                "power": c.map(|s| s.power), "toughness": c.map(|s| s.toughness), "damage": c.map(|s| s.damage)})
        }).collect();
        let stack: Vec<_> = g.turns.stack.iter().map(|h| {
            let effect = g.turns.effects.iter().find(|(spell, _)| spell == h).unwrap().1;
            let ts = match effect { Effect::Growth(t) => vec![self.id(t)], Effect::Bite(a,b) => vec![self.id(a),self.id(b)] };
            json!({"id": self.id(*h), "controller": seat_index(g.objects.get(*h).unwrap().controller), "targets": ts})
        }).collect();
        json!({"name":name,"state":{"turn":1,"step":"upkeep","active":0,"priority":seat_index(d.actor),"life":g.life(),"mana":g.mana(),"stack":stack,"objects":objects,"damage_events":self.damage}})
    }
    fn pass(&mut self, actor: Seat) {
        let d = self.game.turn_decision().expect("priority");
        // Observe actual applied damage during automatic settlement before lethal SBA
        // removes the object. No choice/checkpoint is published at this internal boundary.
        let source = self
            .game
            .turns
            .stack
            .last()
            .and_then(|h| self.game.turns.effects.iter().find(|(s, _)| s == h))
            .and_then(|(_, e)| match e {
                Effect::Bite(a, b) => Some((self.id(*a).to_owned(), *b)),
                _ => None,
            });
        let before = source
            .as_ref()
            .and_then(|(_, b)| self.game.creature_state(*b))
            .map_or(0, |s| s.damage);
        let mut seen = false;
        let mut p = self
            .game
            .apply_turn_quantum(
                actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Pass(d.candidate(0)),
                },
                NonZeroUsize::new(1).unwrap(),
            )
            .unwrap();
        loop {
            if let Some((ref a, b)) = source
                && let Some(c) = self.game.creature_state(b)
                && c.damage > before
                && !seen
            {
                self.damage
                    .push(json!({"source":a,"target":self.id(b),"amount":c.damage-before}));
                seen = true;
            }
            if p != Progress::InternalYield {
                break;
            }
            p = self.game.resume(NonZeroUsize::new(1).unwrap());
        }
    }
}
fn execute(case: &Value) -> Value {
    keys(case, &["id", "setup", "script"]);
    let setup = &case["setup"];
    keys(
        setup,
        &[
            "boundary",
            "active",
            "priority",
            "life",
            "mana",
            "libraries",
            "objects",
        ],
    );
    assert_eq!(setup["boundary"], "turn-1-upkeep-priority");
    assert_eq!(setup["active"], 0);
    assert_eq!(setup["priority"], 0);
    assert_eq!(
        setup["mana"],
        json!([[0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0]])
    );
    assert_eq!(setup["libraries"], json!([[], []]));
    let mut g = Game::new().unwrap();
    g.life = serde_json::from_value(setup["life"].clone()).unwrap();
    g.rng = Some(EpisodeRng::new(VERSION, 0, 0, Stream::Environment).unwrap());
    g.kept = [true; 2];
    let objects = setup["objects"].as_array().unwrap().clone();
    let mut ids = std::collections::BTreeSet::new();
    let mut identities = std::collections::BTreeSet::new();
    for o in &objects {
        keys(o, &["id", "card", "owner", "zone"]);
        assert!(ids.insert(text(&o["id"])));
        assert!(identities.insert((text(&o["card"]), o["owner"].as_u64().unwrap())));
        let owner = seat(&o["owner"]);
        let zone = match text(&o["zone"]) {
            "battlefield" => Zone::Battlefield,
            "hand" => Zone::Hand(owner),
            _ => panic!("unsupported setup zone"),
        };
        assert!(
            [
                "bear-cub",
                "forest",
                "mountain",
                "giant-growth",
                "bite-down"
            ]
            .contains(&text(&o["card"]))
        );
        g.objects
            .allocate(CardId::from_key(text(&o["card"])).unwrap(), owner, zone)
            .unwrap();
    }
    g.start_turns().unwrap();
    let mut a = Adapter {
        game: g,
        objects,
        damage: vec![],
    };
    let mut checkpoints = vec![];
    let mut consumed = vec![];
    let mut cast_actor = None;
    for action in case["script"].as_array().expect("script") {
        let kind = text(&action["kind"]);
        if kind == "checkpoint" {
            keys(action, &["kind", "name"]);
            assert!(cast_actor.is_none());
            checkpoints.push(a.checkpoint(text(&action["name"])));
        } else {
            let actor = seat(&action["actor"]);
            match kind {
                "mana" => {
                    keys(action, &["kind", "actor", "source"]);
                    assert!(cast_actor.is_none());
                    let d = a.game.turn_decision().unwrap();
                    let h = a.handle(text(&action["source"]));
                    a.game.tap_mana(actor, d.id, h).unwrap();
                }
                "cast" => {
                    keys(action, &["kind", "actor", "source"]);
                    assert!(cast_actor.is_none());
                    let d = a.game.turn_decision().unwrap();
                    let h = a.handle(text(&action["source"]));
                    a.game.begin_targeted_cast(actor, d.id, h, 80).unwrap();
                    cast_actor = Some(actor);
                }
                "target" => {
                    keys(action, &["kind", "actor", "target"]);
                    assert_eq!(cast_actor, Some(actor));
                    let d = a
                        .game
                        .target_decision(actor)
                        .expect("unexpected target choice");
                    let h = a.handle(text(&action["target"]));
                    let next = a.game.choose_target(actor, d.id, h).unwrap();
                    if next.kind == targets::TargetKind::Complete {
                        a.game.finish_targets(actor, next.id).unwrap();
                    }
                }
                "pay" => {
                    keys(action, &["kind", "actor", "color"]);
                    assert_eq!(cast_actor, Some(actor));
                    let d = a
                        .game
                        .payment_decision(actor)
                        .expect("payment requested before targets");
                    let color = match text(&action["color"]) {
                        "G" => Color::Green,
                        "R" => Color::Red,
                        _ => panic!("unsupported color"),
                    };
                    a.game.choose_payment(actor, d.id, color).unwrap();
                }
                "finish_cast" => {
                    keys(action, &["kind", "actor"]);
                    assert_eq!(cast_actor, Some(actor));
                    let d = a
                        .game
                        .payment_decision(actor)
                        .expect("payment requested before targets");
                    a.game.finish_cast(actor, d.id).unwrap();
                    cast_actor = None;
                }
                "pass" => {
                    keys(action, &["kind", "actor"]);
                    assert!(cast_actor.is_none());
                    a.pass(actor);
                }
                _ => panic!("unsupported choice"),
            }
        }
        consumed.push(action.clone());
    }
    assert!(cast_actor.is_none());
    assert!(a.game.turns.stack.is_empty(), "unfinished stack");
    assert_eq!(
        checkpoints
            .iter()
            .map(|c| text(&c["name"]))
            .collect::<Vec<_>>(),
        [
            "initial",
            "bite-cast",
            "response-cast",
            "growth-resolved",
            "bite-resolved"
        ]
    );
    json!({"checkpoints":checkpoints,"consumed":consumed})
}
#[test]
fn instant_shared_script_literal_expectations() {
    for case in fixture()["cases"].as_array().unwrap() {
        let result = execute(case);
        assert_eq!(result["checkpoints"], expectations()[text(&case["id"])]);
        assert_eq!(result["consumed"], case["script"]);
    }
}
#[test]
fn instant_strict_choice_regressions() {
    let base = fixture()["cases"][0].clone();
    // Every omission, duplicated choice, wrong actor and adjacent reordering
    // must fail execution or the independently fixed semantic checkpoints.
    let valid = |case: &Value| {
        std::panic::catch_unwind(|| {
            let r = execute(case);
            assert_eq!(r["checkpoints"], expectations()["destination"]);
        })
        .is_ok()
    };
    assert!(valid(&base));
    let n = base["script"].as_array().unwrap().len();
    for i in 0..n {
        let mut c = base.clone();
        c["script"].as_array_mut().unwrap().remove(i);
        assert!(!valid(&c), "omitted {i}");
        let mut c = base.clone();
        let duplicate = c["script"][i].clone();
        c["script"].as_array_mut().unwrap().insert(i, duplicate);
        assert!(!valid(&c), "extra {i}");
        if base["script"][i].get("actor").is_some() {
            let mut c = base.clone();
            c["script"][i]["actor"] = json!(1 - base["script"][i]["actor"].as_u64().unwrap());
            assert!(!valid(&c), "actor {i}");
        }
    }
    // Reordering commutative mana activations is legal Magic, but it is a different
    // script. The shared runner additionally checks exact consumption against its input.
    for (i, j) in [(3, 4), (4, 5), (5, 6), (8, 10), (17, 18)] {
        let mut c = base.clone();
        c["script"].as_array_mut().unwrap().swap(i, j);
        assert!(!valid(&c), "reorder {i}/{j}");
    }
}
#[test]
fn instant_export_observations() {
    let document = std::env::var("MTG_INSTANT_INPUT").map_or_else(
        |_| fixture(),
        |p| serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap(),
    );
    keys(&document, &["version", "cases"]);
    assert_eq!(document["version"], 1);
    let mut out = serde_json::Map::new();
    for c in document["cases"].as_array().unwrap() {
        assert!(out.insert(text(&c["id"]).to_owned(), execute(c)).is_none());
    }
    if let Ok(path) = std::env::var("MTG_INSTANT_OUTPUT") {
        std::fs::write(path, serde_json::to_string_pretty(&out).unwrap()).unwrap();
    } else {
        for (id, r) in out {
            assert_eq!(r["checkpoints"], expectations()[id]);
        }
    }
}
