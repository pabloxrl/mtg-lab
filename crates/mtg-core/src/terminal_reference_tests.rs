//! Test-only synthetic before-SBA adapter, versioned separately from game replay.
use super::*;
use serde_json::{Value, json};

// serde_json::Value alone keeps the last duplicate member. Reject conflicting
// raw action/seat keys before validation or any game mutation.
struct Unique(Value);
impl<'de> serde::Deserialize<'de> for Unique {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Unique;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("unique JSON fields")
            }
            fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Unique, E> {
                Ok(Unique(json!(v)))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Unique, E> {
                Ok(Unique(Value::Null))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Unique, A::Error> {
                let mut values = vec![];
                while let Some(Unique(v)) = a.next_element()? {
                    values.push(v);
                }
                Ok(Unique(Value::Array(values)))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut a: A) -> Result<Unique, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some((key, Unique(value))) = a.next_entry::<String, Unique>()? {
                    if values.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("duplicate JSON field"));
                    }
                }
                Ok(Unique(Value::Object(values)))
            }
        }
        d.deserialize_any(Visitor)
    }
}
fn parse(text: &str) -> Result<Value, String> {
    serde_json::from_str::<Unique>(text)
        .map(|v| v.0)
        .map_err(|e| format!("terminal input: {e}"))
}

fn require(ok: bool, why: &str) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(format!("terminal input: {why}"))
    }
}
fn fields(v: &Value, required: &[&str], optional: &[&str]) -> Result<(), String> {
    let o = v.as_object().ok_or("terminal input: object required")?;
    require(
        o.keys()
            .all(|k| required.contains(&k.as_str()) || optional.contains(&k.as_str())),
        "unknown fields",
    )?;
    require(
        required.iter().all(|k| o.contains_key(*k)),
        "missing fields",
    )
}
fn pair(v: &Value, boolean: bool) -> bool {
    v.as_array().is_some_and(|a| {
        a.len() == 2
            && a.iter().all(|n| {
                if boolean {
                    n.is_boolean()
                } else {
                    n.as_i64().is_some_and(|n| i32::try_from(n).is_ok())
                }
            })
    })
}
fn seat(v: &Value) -> bool {
    matches!(v.as_u64(), Some(0 | 1))
}
fn expectation(v: &Value, version: u64) -> Result<(), String> {
    let mut required = vec!["life", "lost", "library", "hand"];
    if version == 2 {
        required.extend(["outcome", "winner", "draw"]);
    }
    fields(v, &required, &[])?;
    for field in ["life", "lost", "library", "hand"] {
        require(pair(&v[field], field == "lost"), "invalid expectation pair")?;
    }
    if version == 2 {
        require(
            matches!(v["outcome"].as_str(), Some("ongoing" | "win" | "draw")),
            "invalid outcome",
        )?;
        require(
            v["winner"].is_null() || seat(&v["winner"]),
            "invalid winner",
        )?;
        require(v["draw"].is_boolean(), "invalid draw")?;
    }
    Ok(())
}
pub(super) fn validate(doc: &Value) -> Result<(), String> {
    fields(doc, &["version", "basis", "cases"], &[])?;
    let version = doc["version"].as_u64().ok_or("terminal input: version")?;
    require(matches!(version, 1 | 2), "unsupported version")?;
    require(
        doc["basis"].as_str().is_some_and(|s| !s.is_empty()),
        "basis required",
    )?;
    let cases = doc["cases"]
        .as_array()
        .ok_or("terminal input: cases required")?;
    require(!cases.is_empty(), "cases required")?;
    let mut seen = std::collections::BTreeSet::new();
    for c in cases {
        let mut required = vec!["id", "life", "action", "expected"];
        let mut optional = vec!["library_card"];
        if version == 2 {
            required.extend([
                "life_order",
                "injection_order",
                "expected_pending",
                "expected_drawn",
            ]);
            optional.extend(["draw_seat", "concede_seat"]);
        }
        fields(c, &required, &optional)?;
        let id = c["id"].as_str().ok_or("terminal input: id required")?;
        require(
            !id.is_empty()
                && id.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_'
                })
                && id.as_bytes()[0].is_ascii_alphanumeric()
                && seen.insert(id),
            "invalid/duplicate id",
        )?;
        require(pair(&c["life"], false), "invalid life")?;
        let action = c["action"]
            .as_str()
            .ok_or("terminal input: action required")?;
        require(
            matches!(action, "settle" | "draw" | "concede"),
            "invalid action",
        )?;
        if let Some(card) = c.get("library_card") {
            require(
                card == "Forest" && action == "draw",
                "conflicting library_card",
            )?;
        }
        expectation(&c["expected"], version)?;
        if version == 2 {
            require(
                c["life_order"] == json!([0, 1]) || c["life_order"] == json!([1, 0]),
                "invalid life_order",
            )?;
            require(
                c["injection_order"] == "life_then_draw"
                    || (c["injection_order"] == "draw_then_life" && action == "draw"),
                "conflicting injection_order",
            )?;
            for (kind, field) in [("draw", "draw_seat"), ("concede", "concede_seat")] {
                require(
                    c.get(field).is_some() == (action == kind),
                    "conflicting or missing action seat",
                )?;
                if c.get(field).is_some() {
                    require(seat(&c[field]), "invalid seat")?;
                }
            }
            require(
                action != "concede" || c["life"] == json!([20, 20]),
                "conflicting concession",
            )?;
            require(
                if action == "draw" {
                    seat(&c["expected_drawn"])
                } else {
                    c["expected_drawn"].is_null()
                },
                "invalid expected_drawn",
            )?;
            expectation(&c["expected_pending"], 2)?;
        }
    }
    Ok(())
}
fn snapshot(g: &Game) -> Value {
    let result = g.outcome();
    let lost = result
        .map(|o| o.losses.map(|l| l.is_some()))
        .unwrap_or([false; 2]);
    let winner = result.and_then(|o| o.winner).map(seat_index);
    let outcome = match result {
        None => "ongoing",
        Some(o) if o.winner.is_some() => "win",
        Some(_) => "draw",
    };
    json!({"life":g.life(), "lost":lost,
        "library":([Seat::P0, Seat::P1].map(|s| g.objects.in_zone(Zone::Library(s)).count())),
        "hand":([Seat::P0, Seat::P1].map(|s| g.objects.in_zone(Zone::Hand(s)).count())),
        "outcome":outcome, "winner":winner, "draw":result.is_some_and(|o| o.winner.is_none())})
}
fn inject_life(g: &mut Game, c: &Value, consumed: &mut Vec<Value>) {
    for n in c["life_order"].as_array().unwrap() {
        let seat = n.as_u64().unwrap() as usize;
        let value = c["life"][seat].as_i64().unwrap();
        g.life[seat] = value;
        consumed.push(json!({"action":"life", "seat":seat, "value":value}));
    }
}
fn draw(g: &mut Game, c: &Value, fault: &str, consumed: &mut Vec<Value>) -> Option<Seat> {
    let mut index = c["draw_seat"].as_u64().unwrap() as usize;
    if fault == "wrong_draw_seat" {
        index = 1 - index;
    }
    let seat = [Seat::P0, Seat::P1][index];
    // Synthetic effect boundary: retain the failed attempt until the single SBA.
    // Like the native spell/turn draw resolver, failure comes from the actual
    // library, not fixture expectations. draw_top cannot be used for failure here:
    // its documented standalone contract settles immediately (CR 704.3).
    let empty = g.objects.in_zone(Zone::Library(seat)).next().is_none();
    if !empty {
        g.draw_internal(seat);
    }
    consumed.push(json!({"action":"draw", "seat":index, "drawn":usize::from(!empty)}));
    empty.then_some(seat)
}
pub(super) fn observe(c: &Value, fault: &str) -> Value {
    assert!(
        matches!(fault, "none" | "wrong_draw_seat" | "premature_settlement"),
        "unknown terminal fault"
    );
    let mut g = ready();
    g.objects.reset().unwrap();
    if c.get("library_card").is_some() {
        let seat = [Seat::P0, Seat::P1][c["draw_seat"].as_u64().unwrap() as usize];
        add(&mut g, "forest", seat, Zone::Library(seat));
    }
    let mut consumed = vec![];
    let mut failed = None;
    let draw_first = c["injection_order"] == "draw_then_life";
    if draw_first {
        failed = draw(&mut g, c, fault, &mut consumed);
    } else {
        inject_life(&mut g, c, &mut consumed);
    }
    if fault == "premature_settlement" {
        g.settle_terminal(failed);
    }
    if draw_first {
        inject_life(&mut g, c, &mut consumed);
    } else if c["action"] == "draw" {
        failed = draw(&mut g, c, fault, &mut consumed);
    }
    let pending = snapshot(&g);
    if c["action"] == "concede" {
        let index = c["concede_seat"].as_u64().unwrap() as usize;
        g.concede([Seat::P0, Seat::P1][index], g.episode_id().unwrap())
            .unwrap();
        consumed.push(json!({"action":"concede", "seat":index}));
    }
    g.settle_terminal(failed);
    consumed.push(json!({"action":"settle"}));
    json!({"version":2, "pending":pending, "settled":snapshot(&g), "consumed":consumed})
}
fn expected_consumed(c: &Value) -> Value {
    let life: Vec<Value> = c["life_order"]
        .as_array()
        .unwrap()
        .iter()
        .map(
            |s| json!({"action":"life", "seat":s, "value":c["life"][s.as_u64().unwrap() as usize]}),
        )
        .collect();
    let draw = if c["action"] == "draw" {
        vec![json!({"action":"draw", "seat":c["draw_seat"], "drawn":c["expected_drawn"]})]
    } else {
        vec![]
    };
    let mut steps: Vec<Value> = if c["injection_order"] == "life_then_draw" {
        life.into_iter().chain(draw).collect()
    } else {
        draw.into_iter().chain(life).collect()
    };
    if c["action"] == "concede" {
        steps.push(json!({"action":"concede", "seat":c["concede_seat"]}));
    }
    steps.push(json!({"action":"settle"}));
    json!(steps)
}
pub(super) fn export() {
    let input = std::env::var("MTG_TERMINAL_INPUT").ok();
    let text = input
        .map(|p| std::fs::read_to_string(p).unwrap())
        .unwrap_or_else(|| include_str!("../../../fixtures/reference/terminal-v2.json").into());
    let doc = parse(&text).unwrap();
    validate(&doc).unwrap();
    let fault = std::env::var("MTG_TERMINAL_FAULT").unwrap_or_else(|_| "none".into());
    let mut results = serde_json::Map::new();
    for c in doc["cases"].as_array().unwrap() {
        let actual = if doc["version"] == 1 {
            legacy_terminal_observation(c)
        } else {
            observe(c, &fault)
        };
        results.insert(c["id"].as_str().unwrap().into(), actual);
    }
    if let Ok(path) = std::env::var("MTG_TERMINAL_OUTPUT") {
        std::fs::write(path, serde_json::to_string_pretty(&results).unwrap() + "\n").unwrap();
    } else {
        for c in doc["cases"].as_array().unwrap() {
            let actual = &results[c["id"].as_str().unwrap()];
            assert_eq!(actual["version"], 2);
            assert_eq!(
                actual["consumed"],
                expected_consumed(c),
                "{} consumed",
                c["id"]
            );
            assert_eq!(actual["settled"], c["expected"], "{} settled", c["id"]);
            assert_eq!(
                actual["pending"], c["expected_pending"],
                "{} pending",
                c["id"]
            );
        }
    }
}
#[test]
fn terminal_strict_input_corpus() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/terminal-invalid.json"
    ))
    .unwrap();
    for c in cases.as_array().unwrap() {
        let result = if let Some(text) = c["text"].as_str() {
            parse(text).and_then(|v| validate(&v))
        } else {
            validate(&c["fixture"])
        };
        assert!(result.is_err(), "accepted invalid input {}", c["id"]);
    }
    for text in [
        include_str!("../../../fixtures/reference/terminal.json"),
        include_str!("../../../fixtures/reference/terminal-v2.json"),
    ] {
        validate(&serde_json::from_str(text).unwrap()).unwrap();
    }
}
#[test]
fn terminal_actual_faults_change_outcome_or_pending_boundary() {
    let doc: Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/terminal-v2.json")).unwrap();
    for c in doc["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["id"].as_str().unwrap().starts_with("mixed-"))
    {
        let good = observe(c, "none");
        assert_eq!(good["settled"]["outcome"], "draw");
        assert_eq!(good["settled"]["lost"], json!([true, true]));
        let wrong_seat = observe(c, "wrong_draw_seat");
        assert_ne!(
            wrong_seat["settled"], good["settled"],
            "wrong seat must diverge"
        );
        let premature = observe(c, "premature_settlement");
        assert_ne!(
            premature["pending"], good["pending"],
            "intermediate SBA must diverge"
        );
        assert_ne!(
            premature["settled"], good["settled"],
            "premature winner must diverge"
        );
    }
}
