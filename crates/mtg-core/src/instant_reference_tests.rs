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
    births: Vec<u64>,
    history: Vec<(Handle, usize, u64)>,
    last_resolution: Value,
}
impl Adapter {
    fn new(game: Game, objects: Vec<Value>, handles: Vec<Handle>) -> Self {
        let births = handles
            .iter()
            .map(|h| game.objects.semantic_identity(*h).unwrap().0)
            .collect();
        let mut adapter = Self {
            game,
            objects,
            births,
            history: vec![],
            damage: vec![],
            last_resolution: Value::Null,
        };
        adapter.remember();
        adapter
    }
    fn remember(&mut self) {
        for zone in Zone::ALL {
            for h in self.game.objects.in_zone(zone) {
                let (birth, incarnation) = self.game.objects.semantic_identity(h).unwrap();
                if let Some(index) = self.births.iter().position(|b| *b == birth)
                    && !self.history.iter().any(|(old, _, _)| *old == h)
                {
                    self.history.push((h, index, incarnation));
                }
            }
        }
    }
    fn handle(&self, id: &str) -> Handle {
        let index = self
            .objects
            .iter()
            .position(|o| o["id"] == id)
            .expect("unknown object");
        let found: Vec<_> = Zone::ALL
            .into_iter()
            .flat_map(|z| self.game.objects.in_zone(z))
            .filter(|h| self.game.objects.semantic_identity(*h).unwrap().0 == self.births[index])
            .collect();
        assert_eq!(found.len(), 1, "missing semantic object");
        found[0]
    }
    fn identity(&self, handle: Handle) -> (usize, u64) {
        let (_, index, incarnation) = self
            .history
            .iter()
            .find(|(h, _, _)| *h == handle)
            .expect("missing historical identity");
        (*index, *incarnation)
    }
    fn id(&self, handle: Handle) -> &str {
        text(&self.objects[self.identity(handle).0]["id"])
    }
    fn target(&self, handle: Handle) -> Value {
        json!({"id":self.id(handle),"incarnation":self.identity(handle).1})
    }
    fn checkpoint(&self, name: &str) -> Value {
        let g = &self.game;
        let (turn, active, step) = g.turn_position().unwrap();
        assert_eq!((turn, active), (1, Seat::P0));
        let step = match step {
            Step::Upkeep => "upkeep",
            Step::PrecombatMain => "precombat_main",
            Step::BeginningCombat => "begin_combat",
            Step::DeclareAttackers => "declare_attackers",
            Step::DeclareBlockers => "declare_blockers",
            Step::CombatDamage => "combat_damage",
            _ => panic!("unsupported checkpoint step"),
        };
        let d = g
            .turn_decision()
            .expect("checkpoint must be settled priority");
        assert_eq!(
            d.kind,
            TurnKind::Priority,
            "checkpoint must be settled priority"
        );
        let expected_handles: Vec<_> = self
            .objects
            .iter()
            .map(|o| self.handle(text(&o["id"])))
            .collect();
        let actual_handles: Vec<_> = Zone::ALL
            .into_iter()
            .flat_map(|zone| g.objects.in_zone(zone))
            .collect();
        assert_eq!(
            actual_handles.len(),
            expected_handles.len(),
            "missing/extra object"
        );
        assert!(
            actual_handles.iter().all(|h| expected_handles.contains(h)),
            "unexpected object or zone"
        );
        for zone in [
            Zone::Library(Seat::P0),
            Zone::Library(Seat::P1),
            Zone::Exile,
        ] {
            assert_eq!(
                g.objects.in_zone(zone).count(),
                0,
                "unsupported nonempty zone"
            );
        }
        let objects: Vec<_> = self.objects.iter().map(|spec| {
            let h = self.handle(text(&spec["id"])); let o = g.objects.get(h).unwrap();
            assert_eq!(o.card.identity().key, text(&spec["card"]), "card identity changed");
            assert_eq!(o.owner, seat(&spec["owner"]), "owner changed");
            assert_eq!(o.controller, o.owner, "unsupported controller change");
            let zone = match o.zone { Zone::Battlefield => "battlefield", Zone::Stack => "stack", Zone::Hand(_) => "hand", Zone::Graveyard(_) => "graveyard", _ => panic!("unsupported zone") };
            let c = (o.zone == Zone::Battlefield).then(|| g.creature_state(h)).flatten();
            json!({"id": spec["id"], "card": spec["card"], "owner": spec["owner"], "zone": zone, "tapped": o.tapped,
                "power": c.map(|s| s.power), "toughness": c.map(|s| s.toughness), "damage": c.map(|s| s.damage), "incarnation":g.objects.semantic_identity(h).unwrap().1})
        }).collect();
        let stack: Vec<_> = g.turns.stack.iter().map(|h| {
            let effect = g.turns.effects.iter().find(|(spell, _)| spell == h).unwrap().1;
            let ts = match effect { Effect::Growth(t) => vec![self.target(t)], Effect::Bite(a,b) => vec![self.target(a),self.target(b)] };
            json!({"id": self.id(*h), "controller": seat_index(g.objects.get(*h).unwrap().controller), "targets": ts})
        }).collect();
        let combat: Vec<_> = g
            .combat()
            .iter()
            .map(|a| {
                json!({
                    "attacker":self.target(a.creature), "blocked":a.blocked,
                    "blockers":a.blockers.iter().map(|b| self.target(*b)).collect::<Vec<_>>()
                })
            })
            .collect();
        json!({"name":name,"state":{"combat":combat,"turn":turn,"step":step,"active":0,"priority":seat_index(d.actor),"life":g.life(),"mana":g.mana(),"stack":stack,"objects":objects,"damage_events":self.damage,"last_resolution":self.last_resolution}})
    }
    fn pass(&mut self, actor: Seat) {
        let d = self.game.turn_decision().expect("priority");
        let top = self.game.turns.stack.last().copied();
        let top_id = top.map(|h| self.id(h).to_owned());
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
            self.remember();
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
        // A single blocked attacker with no blockers has no damage allocation
        // choice. Translate the core's explicit completion boundary only when
        // its legal allocation list is empty; never choose among alternatives.
        if let Some(d) = self.game.turn_decision()
            && d.kind == TurnKind::Combat(combat::CombatKind::Damage)
        {
            let c = self.game.combat_decision(d.actor, 80).unwrap();
            assert!(c.damage.is_empty(), "unscripted combat damage allocation");
            self.game.finish_combat(d.actor, d.id).unwrap();
            self.remember();
        }
        if let Some(h) = top
            && !self.game.turns.stack.contains(&h)
        {
            let r = self.game.last_resolution().expect("missing resolution");
            self.last_resolution =
                json!({"id":top_id.unwrap(),"legal_targets":r.legal_targets,"resolved":r.resolved});
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
    let mut handles = vec![];
    for o in &objects {
        keys(o, &["id", "card", "owner", "zone"]);
        assert!(ids.insert(text(&o["id"])));

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
        handles.push(
            g.objects
                .allocate(CardId::from_key(text(&o["card"])).unwrap(), owner, zone)
                .unwrap(),
        );
    }
    g.start_turns().unwrap();
    let mut a = Adapter::new(g, objects, handles);
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
                "attackers" => {
                    keys(action, &["kind", "actor", "attackers"]);
                    assert!(cast_actor.is_none());
                    let cards: Vec<_> = action["attackers"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|id| a.handle(text(id)))
                        .collect();
                    let d = a.game.turn_decision().unwrap();
                    let d = a.game.select_attackers(actor, d.id, &cards).unwrap();
                    a.game.finish_combat(actor, d.id).unwrap();
                }
                "blockers" => {
                    keys(action, &["kind", "actor", "blocks"]);
                    assert!(cast_actor.is_none());
                    let blocks: Vec<_> = action["blocks"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|b| {
                            keys(b, &["blocker", "attacker"]);
                            (
                                a.handle(text(&b["blocker"])),
                                a.handle(text(&b["attacker"])),
                            )
                        })
                        .collect();
                    let d = a.game.turn_decision().unwrap();
                    let d = a.game.select_blockers(actor, d.id, &blocks).unwrap();
                    a.game.finish_combat(actor, d.id).unwrap();
                }
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
        a.remember();
        consumed.push(action.clone());
    }
    assert!(cast_actor.is_none());
    assert!(a.game.turns.stack.is_empty(), "unfinished stack");
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

#[test]
fn instant_checkpoint_rejects_unlisted_objects_in_every_zone() {
    // Inventory is part of explicit synthetic setup, regardless of whether the
    // added card would change this short spell sequence's result.
    for zone in Zone::ALL {
        let mut g = Game::new().unwrap();
        g.life = [20; 2];
        g.rng = Some(EpisodeRng::new(VERSION, 0, 0, Stream::Environment).unwrap());
        g.kept = [true; 2];
        g.objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        g.start_turns().unwrap();
        let handles = g.objects.in_zone(Zone::Battlefield).collect();
        let mut a = Adapter::new(
            g,
            vec![json!({"id":"source","card":"bear-cub","owner":0,"zone":"battlefield"})],
            handles,
        );
        a.checkpoint("initial"); // Minimal complete inventory is valid.
        a.game
            .objects
            .allocate(CardId::from_key("forest").unwrap(), Seat::P1, zone)
            .unwrap();
        assert!(
            std::panic::catch_unwind(|| a.checkpoint("initial")).is_err(),
            "unlisted object in {zone:?}"
        );
    }
}

#[test]
fn instant_departed_targets_literal_expectations() {
    // CR 400.7, 608.2b: real responding Bite spells kill selected targets;
    // duplicate-name permanents and the new graveyard objects cannot replace them.
    for case in fixture()["cases"].as_array().unwrap().iter().skip(2) {
        let result = execute(case);
        assert_eq!(result["checkpoints"], expectations()[text(&case["id"])]);
        assert_eq!(result["consumed"], case["script"]);
    }
}

#[test]
fn instant_identity_survives_departure_and_reentry_without_retargeting() {
    // Observer-only synthetic CR 400.7 regression, separate from the real-spell
    // matched scenarios. A new incarnation and a same-name card are distinct.
    let mut g = Game::new().unwrap();
    let card = CardId::from_key("bear-cub").unwrap();
    let original = g
        .objects
        .allocate(card, Seat::P0, Zone::Battlefield)
        .unwrap();
    let decoy = g
        .objects
        .allocate(card, Seat::P0, Zone::Battlefield)
        .unwrap();
    let mut a = Adapter::new(
        g,
        vec![
            json!({"id":"original","card":"bear-cub","owner":0,"zone":"battlefield"}),
            json!({"id":"decoy","card":"bear-cub","owner":0,"zone":"battlefield"}),
        ],
        vec![original, decoy],
    );
    let dead = a
        .game
        .objects
        .move_to(original, Zone::Graveyard(Seat::P0))
        .unwrap();
    a.remember();
    let returned = a.game.objects.move_to(dead, Zone::Battlefield).unwrap();
    a.remember();
    assert_eq!(a.target(original), json!({"id":"original","incarnation":0}));
    assert_eq!(a.target(dead), json!({"id":"original","incarnation":1}));
    assert_eq!(a.target(returned), json!({"id":"original","incarnation":2}));
    assert_eq!(a.target(decoy), json!({"id":"decoy","incarnation":0}));
    assert_eq!(a.handle("original"), returned);
    assert!(a.game.objects.get(original).is_err());
    a.history.retain(|(h, _, _)| *h != original);
    assert!(std::panic::catch_unwind(|| a.target(original)).is_err());
}

#[test]
fn instant_departure_choices_are_not_silently_dropped() {
    for base in fixture()["cases"].as_array().unwrap().iter().skip(2) {
        let valid = |case: &Value| {
            std::panic::catch_unwind(|| {
                let result = execute(case);
                assert_eq!(result["checkpoints"], expectations()[text(&base["id"])]);
            })
            .is_ok()
        };
        assert!(valid(base));
        // Every scripted callback, including passes after departure, is required.
        for i in 0..base["script"].as_array().unwrap().len() {
            let mut omitted = base.clone();
            omitted["script"].as_array_mut().unwrap().remove(i);
            assert!(!valid(&omitted), "silently omitted {i}");
            let mut extra = base.clone();
            let choice = extra["script"][i].clone();
            extra["script"].as_array_mut().unwrap().insert(i, choice);
            assert!(!valid(&extra), "silently ignored {i}");
        }
    }
}

#[test]
fn instant_lineage_binding_still_checks_actual_card_and_owner() {
    // Binding by creation identity must retain the old observer's validation of
    // card/owner rather than echoing setup metadata over a corrupted live object.
    for mutation in ["card", "owner", "controller"] {
        let mut g = Game::new().unwrap();
        g.life = [20; 2];
        g.rng = Some(EpisodeRng::new(VERSION, 0, 0, Stream::Environment).unwrap());
        g.kept = [true; 2];
        let h = g
            .objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                Seat::P0,
                Zone::Battlefield,
            )
            .unwrap();
        g.start_turns().unwrap();
        let mut a = Adapter::new(
            g,
            vec![json!({"id":"source","card":"bear-cub","owner":0,"zone":"battlefield"})],
            vec![h],
        );
        a.checkpoint("initial");
        let o = a.game.objects.get_mut(h).unwrap();
        match mutation {
            "card" => o.card = CardId::from_key("forest").unwrap(),
            "owner" => o.owner = Seat::P1,
            _ => o.controller = Seat::P1,
        }
        assert!(
            std::panic::catch_unwind(|| a.checkpoint("initial")).is_err(),
            "unobserved {mutation}"
        );
    }
}

#[test]
fn instant_blocker_script_reaches_damage_with_remembered_blocked_status() {
    // CR 509.1h/510.1c: a blocked vanilla attacker with no remaining blockers
    // remains blocked and assigns no combat damage to the defending player.
    let f = fixture();
    let case = f["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "bite-killed-blocker")
        .unwrap();
    let result = execute(case);
    assert_eq!(result["checkpoints"], expectations()["bite-killed-blocker"]);
    assert_eq!(result["consumed"], case["script"]);
}

#[test]
fn instant_blocker_illegal_declarations_and_response_order_fail() {
    let f = fixture();
    let base = f["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "bite-killed-blocker")
        .unwrap();
    let valid = |c: &Value| {
        std::panic::catch_unwind(|| {
            let r = execute(c);
            assert_eq!(r["checkpoints"], expectations()["bite-killed-blocker"]);
        })
        .is_ok()
    };
    assert!(valid(base));
    for mutation in [
        "foreign-attacker",
        "land-attacker",
        "duplicate-attacker",
        "foreign-blocker",
        "land-blocker",
        "nonattacking-target",
        "duplicate-blocker",
        "wrong-declarer",
        "response-order",
    ] {
        let mut c = base.clone();
        let script = c["script"].as_array_mut().unwrap();
        let attack = script
            .iter()
            .position(|a| a["kind"] == "attackers")
            .unwrap();
        let block = script.iter().position(|a| a["kind"] == "blockers").unwrap();
        match mutation {
            "foreign-attacker" => script[attack]["attackers"] = json!(["blocker"]),
            "land-attacker" => script[attack]["attackers"] = json!(["forest0"]),
            "duplicate-attacker" => script[attack]["attackers"] = json!(["attacker", "attacker"]),
            "foreign-blocker" => script[block]["blocks"][0]["blocker"] = json!("attacker"),
            "land-blocker" => script[block]["blocks"][0]["blocker"] = json!("forest0"),
            "nonattacking-target" => script[block]["blocks"][0]["attacker"] = json!("forest0"),
            "duplicate-blocker" => {
                let b = script[block]["blocks"][0].clone();
                script[block]["blocks"].as_array_mut().unwrap().push(b);
            }
            "wrong-declarer" => script[block]["actor"] = json!(0),
            _ => {
                let cast = script
                    .iter()
                    .position(|a| a["name"] == "bite-cast")
                    .unwrap();
                script.swap(cast + 1, cast + 3);
            }
        }
        assert!(!valid(&c), "survived {mutation}");
    }
}
