// L0-009: execute the same normal-reset prefix using the existing admission client.
fn forge_abilities_point(
    g: &Game,
    client: &AdmissionClient,
    base: &BTreeMap<String, u64>,
) -> Value {
    let mut point = client.point(g, "source");
    point["incarnations"]
        .as_object_mut()
        .unwrap()
        .retain(|id, _| base.contains_key(id));
    for hand in point["hand"].as_array_mut().unwrap() {
        hand.as_array_mut()
            .unwrap()
            .sort_by_key(|v| v.as_str().unwrap().to_owned());
    }
    for field in ["battlefield", "exile"] {
        point[field]
            .as_array_mut()
            .unwrap()
            .sort_by_key(|v| v.as_str().unwrap().to_owned());
    }
    for (id, incarnation) in point["incarnations"].as_object_mut().unwrap() {
        *incarnation = json!(incarnation.as_u64().unwrap() - base[id]);
    }
    for value in point["stack"].as_array_mut().unwrap() {
        if value["ability"] == "trigger" {
            let id = value["key"]["source"].as_str().unwrap();
            value["key"]["incarnation"] =
                json!(value["key"]["incarnation"].as_u64().unwrap() - base[id]);
            value.as_object_mut().unwrap().retain(|key, _| {
                matches!(
                    key.as_str(),
                    "ability" | "key" | "controller" | "target" | "source_card"
                )
            });
            continue;
        }
        let id = value["source"].as_str().unwrap();
        value["incarnation"] = json!(value["incarnation"].as_u64().unwrap() - base[id]);
        value.as_object_mut().unwrap().remove("action");
        for key in ["object", "raw_birth"] {
            value.as_object_mut().unwrap().remove(key);
        }
        if value["target"].is_object() {
            let id = value["target"]["source"].as_str().unwrap();
            value["target"]["incarnation"] =
                json!(value["target"]["incarnation"].as_u64().unwrap() - base[id]);
        }
        if let Some(targets) = value.get_mut("targets").and_then(Value::as_array_mut) {
            for target in targets {
                let id = target["object"]["source"].as_str().unwrap();
                target["object"]["incarnation"] =
                    json!(target["object"]["incarnation"].as_u64().unwrap() - base[id]);
            }
        }
    }
    for permanent in point["permanents"].as_object_mut().unwrap().values_mut() {
        if permanent["power"].is_null() {
            permanent["sick"] = json!(false);
        }
    }
    let mut effects: BTreeMap<String, [u32; 2]> = BTreeMap::new();
    for m in &g.turns.modifications {
        let entry = effects
            .entry(client.trigger.client.id(g, m.handle))
            .or_default();
        entry[0] += m.boost + m.power_boost;
        entry[1] += m.boost;
    }
    effects.retain(|_, v| *v != [0, 0]);
    point["effects"] = json!(effects);
    for h in g.objects.in_zone(Zone::Battlefield) {
        let d = card_definitions::definition(g.objects.get(h).unwrap().card);
        let v = &mut point["permanents"][client.trigger.client.id(g, h)];
        for (key, value) in [
            ("flying", d.flying()),
            ("reach", d.reach()),
            ("vigilance", d.vigilance()),
            ("deathtouch", d.deathtouch()),
            ("haste", g.has_haste(h)),
            ("trample", g.has_trample(h)),
        ] {
            v[key] = json!(value);
        }
    }
    point.as_object_mut().unwrap().retain(|key, _| {
        matches!(
            key.as_str(),
            "turn"
                | "step"
                | "active"
                | "actor"
                | "life"
                | "mana"
                | "library"
                | "graveyard"
                | "hand"
                | "exile"
                | "battlefield"
                | "permanents"
                | "incarnations"
                | "stack"
                | "effects"
        )
    });
    point
}

fn forge_abilities_execute(fixture: &Value) -> Result<Value, String> {
    require(
        fixture["schema_version"] == 2 && fixture["complete_game"] == false,
        "/casting prefix schema",
    )?;
    let doc = &fixture["opening"];
    let reset = london_reset(doc)?;
    let mut g = Game::new().unwrap();
    let (opening, output) =
        london_continue(&mut g, &reset.cases[0], &doc["cases"][0], |g, tape| {
            let mut client = AdmissionClient {
                extended_observation: true,
                trigger: TriggerClient {
                    client: PlayedClient::new(g, tape, &doc["cases"][0])?,
                    creation: 0,
                    creations: vec![],
                    departures: vec![],
                    cast_start: None,
                    event: 0,
                    witnesses: vec![],
                    pending_keys: vec![],
                    stack_keys: vec![],
                    old_sources: vec![],
                },
                activation_action: None,
                allocations: vec![],
                cleanups: vec![],
                marked: vec![],
            };
            let initial = client.point(g, "initial");
            let mut base: BTreeMap<String, u64> = initial["incarnations"]
                .as_object()
                .unwrap()
                .iter()
                .map(|(id, v)| (id.clone(), v.as_u64().unwrap()))
                .collect();
            let mut checkpoints = vec![];
            let mut consumed = vec![];
            let mut last_source = 0;
            for op in fixture["operations"]
                .as_array()
                .ok_or("first divergence: /operations")?
            {
                let keys: BTreeSet<&str> = op
                    .as_object()
                    .ok_or("first divergence: /operation")?
                    .keys()
                    .map(String::as_str)
                    .collect();
                let expected_keys: BTreeSet<&str> = if op.get("checkpoint").is_some() {
                    ["source_cursor", "checkpoint"].into_iter().collect()
                } else {
                    ["source_cursor", "action"].into_iter().collect()
                };
                require(keys == expected_keys, "/operation fields")?;
                let source = op["source_cursor"]
                    .as_u64()
                    .ok_or("first divergence: /source cursor")?;
                require(source >= last_source, "/source chronology")?;
                last_source = source;
                if let Some(expected) = op.get("checkpoint") {
                    require(
                        !g.turns.pending_triggers.iter().any(Option::is_some),
                        "/unsupported pending triggers",
                    )?;
                    let actual = forge_abilities_point(g, &client, &base);
                    if let Some(difference) = priority_difference(
                        expected,
                        &actual,
                        &format!("/source/{}/checkpoint", op["source_cursor"]),
                    ) {
                        return Err(format!("first divergence: {difference}"));
                    }
                    checkpoints.push(actual);
                } else {
                    let mut action = op["action"].clone();
                    if let Some(id) = action["source"].as_str() {
                        action["incarnation"] = json!(
                            action["incarnation"]
                                .as_u64()
                                .ok_or("first divergence: /incarnation")?
                                + base.get(id).ok_or("first divergence: /unknown source")?
                        );
                    }
                    if let Some(order) = action["order"].as_array_mut() {
                        for key in order {
                            let id = key["source"]
                                .as_str()
                                .ok_or("first divergence: /trigger source")?;
                            key["incarnation"] = json!(
                                key["incarnation"]
                                    .as_u64()
                                    .ok_or("first divergence: /trigger incarnation")?
                                    + base
                                        .get(id)
                                        .ok_or("first divergence: /unknown trigger source")?
                            );
                        }
                    }
                    client.apply(g, &action).map_err(|e| {
                        format!("first divergence: /source/{} {e}", op["source_cursor"])
                    })?;
                    for born in &client.trigger.creations {
                        base.entry(born["id"].as_str().unwrap().to_owned())
                            .or_insert(born["incarnation"].as_u64().unwrap());
                    }
                    consumed.push(op["action"].clone());
                }
            }
            require(
                json!(last_source) == fixture["endpoint"],
                "/unconsumed endpoint",
            )?;
            require(g.outcome().is_none(), "/unexpected terminal")?;
            require(
                json!(consumed.len()) == fixture["actions"],
                "/unconsumed casting actions",
            )?;
            Ok(json!({"checkpoints":checkpoints,"consumed":consumed,"complete_game":false}))
        })?;
    Ok(json!({"opening":opening,"gameplay":output}))
}

#[test]
fn captured_forge_abilities_strict_replay() {
    let text = std::env::var("MTG_FORGE_ABILITIES_INPUT")
        .map(|p| std::fs::read_to_string(p).unwrap())
        .unwrap_or_else(|_| {
            include_str!("../../../fixtures/reference/forge-abilities.json").to_owned()
        });
    let fixtures: Vec<Value> = serde_json::from_str(&text).unwrap();
    assert!(!fixtures.is_empty());
    let mut results = vec![];
    for fixture in fixtures {
        let run = forge_abilities_execute(&fixture).unwrap();
        assert_eq!(run, forge_abilities_execute(&fixture).unwrap());
        let operations = fixture["operations"].as_array().unwrap();
        let mut faults = vec!["payment", "checkpoint"];
        if operations
            .iter()
            .any(|op| op["action"]["kind"] == "order_triggers")
        {
            faults.extend(["trigger-source", "trigger-event", "trigger-controller"]);
        }
        if operations.iter().any(|op| {
            op["action"]["order"]
                .as_array()
                .is_some_and(|order| order.len() > 1)
        }) {
            faults.push("trigger-order");
        }
        if operations
            .iter()
            .any(|op| op["action"]["kind"] == "target_player")
        {
            faults.push("trigger-target");
        }
        if operations
            .iter()
            .any(|op| op["action"]["kind"] == "activation_pay")
        {
            faults.extend(["activation-payment", "activation-source"]);
        }
        if operations
            .iter()
            .any(|op| op["action"]["kind"] == "activation_target")
        {
            faults.push("activation-target");
        }
        if operations.iter().any(|op| {
            op["checkpoint"]["permanents"]
                .as_object()
                .is_some_and(|cards| {
                    cards
                        .values()
                        .any(|c| c["card"] == "druid-of-the-cowl" && c["trample"] == true)
                })
        }) {
            faults.push("temporary-trample");
        }
        for fault in faults {
            let mut bad = fixture.clone();
            let ops = bad["operations"].as_array_mut().unwrap();
            match fault {
                "trigger-source" | "trigger-event" | "trigger-controller" => {
                    let op = ops
                        .iter_mut()
                        .find(|op| op["action"]["kind"] == "order_triggers")
                        .unwrap();
                    match fault {
                        "trigger-source" => op["action"]["order"][0]["incarnation"] = json!(999999),
                        "trigger-event" => op["action"]["order"][0]["event"] = json!(999999),
                        _ => {
                            op["action"]["actor"] =
                                json!(1 - op["action"]["actor"].as_u64().unwrap())
                        }
                    }
                }
                "trigger-order" => {
                    let op = ops
                        .iter_mut()
                        .find(|op| {
                            op["action"]["order"]
                                .as_array()
                                .is_some_and(|order| order.len() > 1)
                        })
                        .unwrap();
                    op["action"]["order"].as_array_mut().unwrap().reverse();
                }
                "trigger-target" => {
                    let op = ops
                        .iter_mut()
                        .find(|op| op["action"]["kind"] == "target_player")
                        .unwrap();
                    op["action"]["player"] = json!(1 - op["action"]["player"].as_u64().unwrap());
                }
                "activation-payment" | "activation-source" | "activation-target" => {
                    let (kind, field) = match fault {
                        "activation-payment" => ("activation_pay", "color"),
                        "activation-source" => ("activate", "incarnation"),
                        _ => ("activation_target", "incarnation"),
                    };
                    let op = ops
                        .iter_mut()
                        .find(|op| op["action"]["kind"] == kind)
                        .unwrap();
                    op["action"][field] = json!(if field == "color" { 0 } else { 999999 });
                }
                "temporary-trample" => {
                    let card = ops
                        .iter_mut()
                        .filter_map(|op| {
                            op.get_mut("checkpoint")
                                .and_then(|point| point.get_mut("permanents"))
                                .and_then(Value::as_object_mut)
                        })
                        .flat_map(|cards| cards.values_mut())
                        .find(|c| c["card"] == "druid-of-the-cowl" && c["trample"] == true)
                        .unwrap();
                    card["trample"] = json!(false);
                }
                "payment" => {
                    let op = ops
                        .iter_mut()
                        .find(|op| op["action"]["kind"] == "pay")
                        .unwrap();
                    op["action"]["color"] = json!(0);
                }
                _ => {
                    let op = ops
                        .iter_mut()
                        .find(|op| op.get("checkpoint").is_some())
                        .unwrap();
                    op["checkpoint"]["life"][0] = json!(99);
                }
            }
            assert!(forge_abilities_execute(&bad).is_err(), "accepted {fault}");
        }
        results.push(run);
    }
    if let Ok(path) = std::env::var("MTG_FORGE_ABILITIES_OUTPUT") {
        std::fs::write(path, serde_json::to_vec(&results).unwrap()).unwrap();
    }
}
