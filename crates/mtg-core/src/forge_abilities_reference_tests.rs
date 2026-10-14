// L0-009: execute the same normal-reset prefix using the existing admission client.
fn forge_abilities_point(
    g: &Game,
    client: &AdmissionClient,
    base: &BTreeMap<String, u64>,
) -> Value {
    forge_abilities_normalize(client.point(g, "source"), base, false)
}
fn forge_abilities_normalize(
    mut point: Value,
    base: &BTreeMap<String, u64>,
    full_game: bool,
) -> Value {
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
    for m in point["effects"].as_array().unwrap() {
        let entry = effects
            .entry(m["object"]["source"].as_str().unwrap().to_owned())
            .or_default();
        entry[0] += (m["boost"].as_u64().unwrap() + m["power_boost"].as_u64().unwrap()) as u32;
        entry[1] += m["boost"].as_u64().unwrap() as u32;
    }
    effects.retain(|_, v| *v != [0, 0]);
    point["effects"] = json!(effects);
    if full_game {
        for group in point["combat"].as_array_mut().unwrap() {
            for key in ["attacker", "blockers"] {
                let refs: Vec<&mut Value> = if key == "attacker" {
                    vec![&mut group[key]]
                } else {
                    group[key].as_array_mut().unwrap().iter_mut().collect()
                };
                for reference in refs {
                    let id = reference["source"].as_str().unwrap();
                    reference["incarnation"] =
                        json!(reference["incarnation"].as_u64().unwrap() - base[id]);
                }
            }
        }
    }
    point.as_object_mut().unwrap().retain(|key, _| {
        (full_game && matches!(key.as_str(), "combat" | "outcome"))
            || matches!(
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
    let full_game = fixture["schema_version"] == 3 && fixture["complete_game"] == true;
    require(
        full_game || (fixture["schema_version"] == 2 && fixture["complete_game"] == false),
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
            if full_game {
                require(
                    json!(base) == fixture["initial_incarnations"],
                    "/opening incarnation provenance",
                )?;
            }
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
                    if full_game
                        && g.turn_decision().is_some_and(|d| {
                            d.kind == turns::TurnKind::Combat(combat::CombatKind::Damage)
                        })
                    {
                        let actor = g.turn_decision().unwrap().actor;
                        let prior = g
                            .objects
                            .in_zone(Zone::Battlefield)
                            .map(|h| (h, client.trigger.reference(g, h)))
                            .collect();
                        client.finish_damage(g, actor)?;
                        client.observe_transition(g, prior, consumed.len())?;
                    }
                    let actual = if full_game {
                        forge_abilities_normalize(client.point(g, "source"), &base, true)
                    } else {
                        forge_abilities_point(g, &client, &base)
                    };
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
                    if let Some(selection) =
                        action.get_mut("selection").and_then(Value::as_array_mut)
                    {
                        fn offset(
                            value: &mut Value,
                            base: &BTreeMap<String, u64>,
                        ) -> Result<(), String> {
                            if let Some(id) = value["source"].as_str() {
                                value["incarnation"] = json!(
                                    value["incarnation"].as_u64().ok_or("missing incarnation")?
                                        + base.get(id).ok_or("unknown source")?
                                );
                            } else if let Some(object) = value.as_object_mut() {
                                for v in object.values_mut() {
                                    if v.is_object() || v.is_array() {
                                        offset(v, base)?;
                                    }
                                }
                            } else if let Some(array) = value.as_array_mut() {
                                for v in array {
                                    offset(v, base)?;
                                }
                            }
                            Ok(())
                        }
                        for value in selection {
                            offset(value, &base)?;
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
            require(g.outcome().is_some() == full_game, "/unexpected terminal")?;
            require(
                json!(consumed.len()) == fixture["actions"],
                "/unconsumed casting actions",
            )?;
            let mut result =
                json!({"checkpoints":checkpoints,"consumed":consumed,"complete_game":full_game});
            if full_game {
                for (kind, actual) in [
                    ("marked", &client.marked),
                    ("cleanup_checkpoints", &client.cleanups),
                ] {
                    let expected = fixture["auxiliary"][kind]
                        .as_array()
                        .ok_or("missing auxiliary source checkpoints")?;
                    require(
                        expected.len() == actual.len(),
                        &format!("/auxiliary/{kind}/count"),
                    )?;
                    for (expected, actual) in expected.iter().zip(actual) {
                        let actual = forge_abilities_normalize(actual.clone(), &base, true);
                        if let Some(delta) = priority_difference(
                            &expected["checkpoint"],
                            &actual,
                            &format!("/source/{}/auxiliary/{kind}", expected["source_cursor"]),
                        ) {
                            return Err(format!("first divergence: {delta}"));
                        }
                    }
                }
                let mut raw = client.trigger.client.report();
                raw["capture_status"] =
                    json!("complete rules-terminal source replay; not corpus admission");
                raw["marked"] = json!(client.marked);
                raw["cleanup_checkpoints"] = json!(client.cleanups);
                raw["trigger_provenance"] = json!(client.trigger.witnesses);
                result["admission"] = raw;
            }
            Ok(result)
        })?;
    let mut output = output;
    if full_game {
        output["admission"]["opening"] = opening.clone();
    }
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

#[test]
fn captured_forge_combat_strict_replay() {
    // ObjectStore IDs identify separate engine instances, not game objects.
    // Preserve their equality relationships and every slot/generation/epoch;
    // canonicalize only this process allocation metadata for repeat comparison.
    // Raw output retains the original handles for provenance validation.
    fn canonical_stores(value: &mut Value, stores: &mut BTreeMap<u64, usize>) {
        match value {
            Value::Object(object) => {
                if let Some(handle) = object.get_mut("raw_source_handle") {
                    let store = handle["store"].as_u64().unwrap();
                    let next = stores.len();
                    handle["store"] = json!(*stores.entry(store).or_insert(next));
                }
                for child in object.values_mut() {
                    canonical_stores(child, stores);
                }
            }
            Value::Array(array) => {
                for child in array {
                    canonical_stores(child, stores);
                }
            }
            _ => {}
        }
    }
    let path = std::env::var("MTG_FORGE_COMBAT_INPUT")
        .unwrap_or_else(|_| "../../fixtures/reference/forge-combat.json".into());
    let fixtures: Vec<Value> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert!(!fixtures.is_empty());
    let results: Vec<_> = fixtures
        .iter()
        .map(|fixture| {
            let result = forge_abilities_execute(fixture).unwrap();
            let mut repeated = forge_abilities_execute(fixture).unwrap();
            let mut stable = result.clone();
            canonical_stores(&mut stable, &mut BTreeMap::new());
            canonical_stores(&mut repeated, &mut BTreeMap::new());
            if let Some(delta) = priority_difference(&stable, &repeated, "/repeat") {
                panic!("replay repetition divergence: {delta}");
            }
            let mut faults = vec!["expired-effect", "terminal-reason"];
            for (kind, fault) in [
                ("damage", "allocation"),
                ("graveyard_order", "graveyard-order"),
            ] {
                if fixture["operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|op| op["action"]["kind"] == kind)
                {
                    faults.push(fault);
                }
            }
            for fault in faults {
                let mut bad = fixture.clone();
                match fault {
                    "allocation" => {
                        let op = bad["operations"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|op| op["action"]["kind"] == "damage")
                            .unwrap();
                        let amount = &mut op["action"]["selection"][0]["amounts"][0]["amount"];
                        *amount = json!(amount.as_u64().unwrap() + 1);
                    }
                    "graveyard-order" => {
                        let op = bad["operations"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|op| op["action"]["kind"] == "graveyard_order")
                            .unwrap();
                        op["action"]["selection"].as_array_mut().unwrap().reverse();
                    }
                    "expired-effect" => {
                        let row = bad["auxiliary"]["cleanup_checkpoints"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|row| {
                                row["checkpoint"]["permanents"]
                                    .as_object()
                                    .unwrap()
                                    .values()
                                    .any(|p| !p["power"].is_null())
                            })
                            .unwrap();
                        let point = &mut row["checkpoint"];
                        let source = point["permanents"]
                            .as_object()
                            .unwrap()
                            .iter()
                            .find(|(_, p)| !p["power"].is_null())
                            .unwrap()
                            .0
                            .clone();
                        point["effects"][source] = json!([3, 3]);
                    }
                    "terminal-reason" => {
                        let point = &mut bad["operations"]
                            .as_array_mut()
                            .unwrap()
                            .last_mut()
                            .unwrap()["checkpoint"];
                        let loss = point["outcome"]["losses"]
                            .as_array_mut()
                            .unwrap()
                            .iter_mut()
                            .find(|v| !v.is_null())
                            .unwrap();
                        *loss = json!(if *loss == "Life" { "EmptyDraw" } else { "Life" });
                    }
                    _ => unreachable!(),
                }
                let error = forge_abilities_execute(&bad).expect_err(fault);
                assert!(error.contains("first divergence:"), "{fault}: {error}");
            }
            result
        })
        .collect();
    if let Ok(path) = std::env::var("MTG_FORGE_COMBAT_OUTPUT") {
        std::fs::write(path, serde_json::to_vec(&results).unwrap()).unwrap();
    }
}
