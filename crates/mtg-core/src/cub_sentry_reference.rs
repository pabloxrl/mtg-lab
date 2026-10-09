//! Test-only extension of the Shivan bridge. Literal CR/Oracle expectations live
//! in shivan-expectations.json; this adapter only submits choices and observes.
use super::*;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Clone, Debug, Deserialize, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Choice {
    kind: String,
    actor: u8,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    id: String,
    mode: String,
    active: u8,
    choices: Vec<Choice>,
}
struct Tape {
    choices: Vec<Choice>,
    used: Vec<Choice>,
    id: String,
}
impl Tape {
    fn take(&mut self, kind: &str, actor: Seat) {
        let wanted = Choice {
            kind: kind.into(),
            actor: seat_index(actor) as u8,
        };
        assert_eq!(
            self.choices.get(self.used.len()),
            Some(&wanted),
            "first divergence: {}/choices/{}",
            self.id,
            self.used.len()
        );
        self.used.push(wanted);
    }
    fn pass(&mut self, g: &mut Game, actor: Seat) {
        self.take("pass", actor);
        assert_eq!(
            g.turn_decision().unwrap().actor,
            actor,
            "first divergence: actor"
        );
        pass(g);
    }
    fn pair(&mut self, g: &mut Game, actor: Seat) {
        self.pass(g, actor);
        self.pass(g, turns::opponent(actor));
    }
}
fn find(g: &Game, key: &str) -> Handle {
    [
        Zone::Battlefield,
        Zone::Stack,
        Zone::Hand(Seat::P0),
        Zone::Hand(Seat::P1),
        Zone::Graveyard(Seat::P0),
        Zone::Graveyard(Seat::P1),
    ]
    .into_iter()
    .flat_map(|z| g.objects.in_zone(z))
    .find(|h| g.objects.get(*h).unwrap().card.identity().key == key)
    .unwrap()
}
fn observe(g: &Game, keys: &[(&str, &str)]) -> Value {
    let mut objects = serde_json::Map::new();
    for (name, key) in keys {
        let h = find(g, key);
        let o = g.objects.get(h).unwrap();
        let zone = match o.zone {
            Zone::Battlefield => "battlefield",
            Zone::Stack => "stack",
            Zone::Hand(_) => "hand",
            Zone::Graveyard(_) => "graveyard",
            _ => panic!("unexpected zone"),
        };
        let c = if o.zone == Zone::Battlefield {
            g.creature_state(h)
        } else {
            None
        };
        objects.insert(
            (*name).into(),
            json!([
                zone,
                c.map(|c| c.power),
                c.map(|c| c.toughness),
                c.map(|c| c.damage),
                (o.zone == Zone::Battlefield).then_some(o.tapped),
                (o.zone == Zone::Battlefield).then(|| g.summoning_sick(h))
            ]),
        );
    }
    let (_, active, step) = g.turn_position().unwrap();
    let step = match step {
        turns::Step::PrecombatMain => "main",
        turns::Step::DeclareAttackers => "attackers",
        turns::Step::DeclareBlockers => "blockers",
        turns::Step::CombatDamage => "damage",
        _ => panic!("unexpected checkpoint step"),
    };
    json!({"active":seat_index(active),"priority":seat_index(g.turn_decision().unwrap().actor),"step":step,
        "mana":g.mana().map(|m|[m[3],m[4]]),"objects":objects,"life":g.life(),
        "stack":g.turns.stack.iter().map(|h|g.objects.get(*h).unwrap().card.identity().key).collect::<Vec<_>>()})
}
fn pay_exact(g: &mut Game, t: &mut Tape, actor: Seat, kind: &str, color: mana::Color) {
    t.take(kind, actor);
    let d = g.payment_decision(actor).unwrap();
    g.choose_payment(actor, d.id, color).unwrap();
}
fn reject(g: &mut Game, kind: &str, actor: Seat, key: &str) -> Value {
    let before = g.snapshot();
    let d = g.turn_decision().unwrap();
    let h = find(g, key);
    let accepted = match kind {
        "reject_cub" => g.begin_cast(actor, d.id, h).is_ok(),
        "reject_attack" => g.select_attackers(actor, d.id, &[h]).is_ok(),
        "reject_block" => g
            .select_blockers(actor, d.id, &[(h, find(g, "shivan-dragon"))])
            .is_ok(),
        _ => unreachable!(),
    };
    json!({"kind":kind,"accepted":accepted,"unchanged":g.snapshot()==before})
}
pub(super) fn execute(value: &Value) -> Value {
    let spec: Spec =
        serde_json::from_value(value.clone()).expect("first divergence: exact fixture schema");
    assert!(spec.active < 2, "first divergence: active");
    assert!(
        [
            "cub_cast",
            "cub_reject",
            "single_sentry",
            "single_cub_illegal"
        ]
        .contains(&spec.mode.as_str()),
        "first divergence: mode"
    );
    assert_eq!(
        spec.id,
        format!("exact_{}_{}", spec.mode, spec.active),
        "first divergence: id"
    );
    let a = if spec.active == 0 { Seat::P0 } else { Seat::P1 };
    let b = turns::opponent(a);
    let mut t = Tape {
        choices: spec.choices,
        used: vec![],
        id: spec.id,
    };
    let mut g = ready();
    // Explicit synthetic position only; no state replacement after execution starts.
    g.objects = ObjectStore::new().unwrap();
    g.turns.position = Some((1, a, turns::Step::PrecombatMain));
    g.set_turn_decision(a, turns::TurnKind::Priority);
    g.turns.passed = false;
    let cub = spec.mode.starts_with("cub_");
    let illegal = spec.mode == "single_cub_illegal";
    let owner = if spec.mode == "cub_reject" { b } else { a };
    let keys: Vec<(&str, &str)> = if cub {
        vec![("cub", "bear-cub")]
    } else if illegal {
        vec![("dragon", "shivan-dragon"), ("cub", "bear-cub")]
    } else {
        vec![
            ("dragon", "shivan-dragon"),
            ("sentry", "magnigoth-sentry"),
            ("growth", "giant-growth"),
            ("forest", "forest"),
        ]
    };
    if cub {
        g.objects
            .allocate(
                CardId::from_key("bear-cub").unwrap(),
                owner,
                Zone::Hand(owner),
            )
            .unwrap();
        g.turns.mana[seat_index(owner)][3] = 1;
        g.turns.mana[seat_index(owner)][4] = 1;
    } else {
        for (key, seat, zone) in [
            ("shivan-dragon", a, Zone::Battlefield),
            (
                if illegal {
                    "bear-cub"
                } else {
                    "magnigoth-sentry"
                },
                b,
                Zone::Battlefield,
            ),
        ] {
            g.objects
                .allocate(CardId::from_key(key).unwrap(), seat, zone)
                .unwrap();
        }
        if !illegal {
            for (key, zone) in [
                ("forest", Zone::Battlefield),
                ("giant-growth", Zone::Hand(b)),
            ] {
                g.objects
                    .allocate(CardId::from_key(key).unwrap(), b, zone)
                    .unwrap();
            }
        }
    }
    let mut points = serde_json::Map::new();
    let mut attempts = vec![];
    let mut damage = serde_json::Map::new();
    points.insert("initial".into(), observe(&g, &keys));
    if spec.mode == "cub_reject" {
        t.pass(&mut g, a);
        points.insert("before_rejection".into(), observe(&g, &keys));
        t.take("reject_cub", b);
        attempts.push(reject(&mut g, "reject_cub", b, "bear-cub"));
        points.insert("cast_rejected".into(), observe(&g, &keys));
    } else if cub {
        t.take("cast_cub", a);
        let d = g.turn_decision().unwrap();
        let h = find(&g, "bear-cub");
        g.begin_cast(a, d.id, h).unwrap();
        pay_exact(&mut g, &mut t, a, "pay_green", mana::Color::Green);
        pay_exact(&mut g, &mut t, a, "pay_red", mana::Color::Red);
        let d = g.payment_decision(a).unwrap();
        g.finish_cast(a, d.id).unwrap();
        points.insert("cast".into(), observe(&g, &keys));
        t.pair(&mut g, a);
        points.insert("resolved".into(), observe(&g, &keys));
        t.pair(&mut g, a);
        t.pair(&mut g, a);
        t.take("reject_attack", a);
        attempts.push(reject(&mut g, "reject_attack", a, "bear-cub"));
        points.insert("attack_rejected".into(), observe(&g, &keys));
    } else {
        t.pair(&mut g, a);
        t.pair(&mut g, a);
        t.take("attack", a);
        let dragon = find(&g, "shivan-dragon");
        let d = g.turn_decision().unwrap();
        let d = g.select_attackers(a, d.id, &[dragon]).unwrap();
        g.finish_combat(a, d.id).unwrap();
        t.pair(&mut g, a);
        let blocker = find(
            &g,
            if illegal {
                "bear-cub"
            } else {
                "magnigoth-sentry"
            },
        );
        if illegal {
            t.take("reject_block", b);
            attempts.push(reject(&mut g, "reject_block", b, "bear-cub"));
            points.insert("block_rejected".into(), observe(&g, &keys));
        } else {
            t.take("block", b);
            let d = g.turn_decision().unwrap();
            g.select_blockers(b, d.id, &[(blocker, dragon)]).unwrap();
        }
        let d = g.turn_decision().unwrap();
        g.finish_combat(b, d.id).unwrap();
        if !illegal {
            t.pass(&mut g, a);
            t.take("tap_forest", b);
            let d = g.turn_decision().unwrap();
            let forest = find(&g, "forest");
            g.tap_mana(b, d.id, forest).unwrap();
            t.take("cast_growth", b);
            let d = g.turn_decision().unwrap();
            let growth = find(&g, "giant-growth");
            let d = g.begin_targeted_cast(b, d.id, growth, 256).unwrap();
            let d = g.choose_target(b, d.id, blocker).unwrap();
            g.finish_targets(b, d.id).unwrap();
            pay_exact(&mut g, &mut t, b, "pay_green", mana::Color::Green);
            let d = g.payment_decision(b).unwrap();
            g.finish_cast(b, d.id).unwrap();
            points.insert("growth_cast".into(), observe(&g, &keys));
            t.pair(&mut g, b);
            points.insert("grown".into(), observe(&g, &keys));
        }
        t.pair(&mut g, a);
        t.take("damage", a);
        let d = g.turn_decision().unwrap();
        let mut progress = g.finish_combat_quantum(a, d.id, NonZeroUsize::MIN).unwrap();
        while progress == Progress::InternalYield {
            // Read actual applied marks before the queued state-based move.
            if !illegal {
                for (name, h) in [("dragon", dragon), ("sentry", blocker)] {
                    if let Some(c) = g.creature_state(h)
                        && c.damage > 0
                    {
                        damage.insert(name.into(), json!(c.damage));
                    }
                }
            }
            progress = g.resume(NonZeroUsize::MIN);
        }
        points.insert("damage".into(), observe(&g, &keys));
    }
    assert_eq!(
        t.used.len(),
        t.choices.len(),
        "first divergence: {}/choices/extra",
        t.id
    );
    json!({"points":points,"attempts":attempts,"consumed":t.used,"combat_damage":damage})
}

pub(super) fn assert_observation(expected: &Value, actual: &Value, path: &str) {
    match (expected, actual) {
        (Value::Object(e), Value::Object(a)) => {
            assert_eq!(
                e.keys().collect::<Vec<_>>(),
                a.keys().collect::<Vec<_>>(),
                "first divergence: {path}/fields"
            );
            for (key, value) in e {
                assert_observation(value, &a[key], &format!("{path}/{key}"));
            }
        }
        (Value::Array(e), Value::Array(a)) => {
            assert_eq!(e.len(), a.len(), "first divergence: {path}/length");
            for (i, (left, right)) in e.iter().zip(a).enumerate() {
                assert_observation(left, right, &format!("{path}/{i}"));
            }
        }
        _ => assert_eq!(expected, actual, "first divergence: {path}"),
    }
}

#[test]
fn exact_inputs_reject_missing_extra_wrong_actor_and_unknown_fields() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/shivan.json")).unwrap();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["id"].as_str().unwrap().starts_with("exact_"))
    {
        for mutation in [
            "missing",
            "extra",
            "actor",
            "boolean_actor",
            "unknown",
            "unknown_choice",
            "unknown_mode",
            "invalid_active",
        ] {
            let mut bad = case.clone();
            match mutation {
                "missing" => {
                    bad["choices"].as_array_mut().unwrap().pop();
                }
                "extra" => {
                    let last = bad["choices"].as_array().unwrap().last().unwrap().clone();
                    bad["choices"].as_array_mut().unwrap().push(last);
                }
                "actor" => {
                    bad["choices"][0]["actor"] =
                        json!(1 - bad["choices"][0]["actor"].as_u64().unwrap())
                }
                "boolean_actor" => bad["choices"][0]["actor"] = json!(true),
                "unknown" => bad["unknown"] = json!(1),
                "unknown_choice" => bad["choices"][0]["unknown"] = json!(1),
                "unknown_mode" => bad["mode"] = json!("unsupported"),
                "invalid_active" => bad["active"] = json!(2),
                _ => unreachable!(),
            }
            let error = std::panic::catch_unwind(|| execute(&bad))
                .expect_err("strict input mutation survived");
            let message = error
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| error.downcast_ref::<&str>().copied())
                .unwrap_or("");
            assert!(
                message.contains("first divergence"),
                "unexpected failure for {mutation}: {message}"
            );
        }
    }
}
