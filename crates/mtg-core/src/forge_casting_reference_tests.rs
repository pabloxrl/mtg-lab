// L0-008: normal-reset source prefixes; all choices use the delivered client.
fn forge_casting_point(g: &Game, client: &SpellClient, base: &BTreeMap<String, u64>) -> Value {
    let mut point = client.point(g, "source");
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
        let id = value["source"].as_str().unwrap();
        value["incarnation"] = json!(value["incarnation"].as_u64().unwrap() - base[id]);
        value.as_object_mut().unwrap().remove("action");
        for target in value["targets"].as_array_mut().unwrap() {
            let id = target["object"]["source"].as_str().unwrap();
            target["object"]["incarnation"] =
                json!(target["object"]["incarnation"].as_u64().unwrap() - base[id]);
        }
    }
    for permanent in point["permanents"].as_object_mut().unwrap().values_mut() {
        if permanent["power"].is_null() {
            permanent["sick"] = json!(false);
        }
    }
    let mut effects: BTreeMap<String, [u32; 2]> = BTreeMap::new();
    for m in &g.turns.modifications {
        let entry = effects.entry(client.client.id(g, m.handle)).or_default();
        entry[0] += m.boost + m.power_boost;
        entry[1] += m.boost;
    }
    effects.retain(|_, v| *v != [0, 0]);
    point["effects"] = json!(effects);
    for h in g.objects.in_zone(Zone::Battlefield) {
        let d = card_definitions::definition(g.objects.get(h).unwrap().card);
        let v = &mut point["permanents"][client.client.id(g, h)];
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

fn forge_casting_execute(fixture: &Value) -> Result<Value, String> {
    require(
        fixture["schema_version"] == 1 && fixture["complete_game"] == false,
        "/casting prefix schema",
    )?;
    let doc = &fixture["opening"];
    let reset = london_reset(doc)?;
    let mut g = Game::new().unwrap();
    let (opening, output) =
        london_continue(&mut g, &reset.cases[0], &doc["cases"][0], |g, tape| {
            let mut client = SpellClient {
                client: PlayedClient::new(g, tape, &doc["cases"][0])?,
                creation: 0,
                creations: vec![],
                departures: vec![],
                cast_start: None,
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
                        !g.turns.pending_triggers.iter().any(Option::is_some)
                            && g.turns.triggered.is_empty(),
                        "/unsupported pending triggers",
                    )?;
                    let actual = forge_casting_point(g, &client, &base);
                    if &actual != expected {
                        return Err(format!(
                            "first divergence: /source/{}/checkpoint expected={} actual={}",
                            op["source_cursor"], expected, actual
                        ));
                    }
                    checkpoints.push(actual);
                } else {
                    let mut action = op["action"].clone();
                    if let Some(id) = action["source"].as_str() {
                        action["incarnation"] = json!(
                            action["incarnation"]
                                .as_u64()
                                .ok_or("first divergence: /incarnation")?
                                + base[id]
                        );
                    }
                    let choice: SpellChoice = serde_json::from_value(action)
                        .map_err(|e| format!("first divergence: /casting action {e}"))?;
                    client.apply(g, &choice).map_err(|e| {
                        format!("first divergence: /source/{} {e}", op["source_cursor"])
                    })?;
                    for born in &client.creations {
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
fn captured_forge_casting_strict_replay() {
    let text = std::env::var("MTG_FORGE_CASTING_INPUT")
        .map(|p| std::fs::read_to_string(p).unwrap())
        .unwrap_or_else(|_| {
            include_str!("../../../fixtures/reference/forge-casting.json").to_owned()
        });
    let fixtures: Vec<Value> = serde_json::from_str(&text).unwrap();
    assert!(!fixtures.is_empty());
    let mut results = vec![];
    for fixture in fixtures {
        let mut run = forge_casting_execute(&fixture).unwrap();
        assert_eq!(
            run,
            forge_casting_execute(&fixture).unwrap(),
            "nondeterministic source prefix"
        );
        // CR 117 and 601: passing order, physical incarnation and payment are
        // consequential choices. Each mutation must fail in the real engine.
        let mut faults = vec![];
        for (kind, fault) in [
            ("cast", "incarnation"),
            ("pay", "payment"),
            ("pass", "pass"),
        ] {
            if fixture["operations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v["action"]["kind"] == kind)
            {
                faults.push(fault);
            }
        }
        faults.push("checkpoint");
        if fixture["operations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["action"]["kind"] == "target")
        {
            faults.push("target-incarnation");
        }
        let operations = fixture["operations"].as_array().unwrap();
        for kind in ["mode", "discard"] {
            if operations.iter().any(|v| v["action"]["kind"] == kind) {
                faults.push(kind);
            }
        }
        if operations.iter().any(|v| {
            v["action"]["kind"] == "cast"
                && v["action"]["source"]
                    .as_str()
                    .is_some_and(|s| s.contains("/giant-growth/"))
        }) {
            faults.push("response-order");
        }
        if operations.iter().any(|v| {
            v["checkpoint"]["effects"]
                .as_object()
                .is_some_and(|e| !e.is_empty())
        }) {
            faults.push("effect-amount");
        }
        if operations.iter().any(|v| {
            v["checkpoint"]["permanents"]
                .as_object()
                .is_some_and(|p| p.values().any(|c| c["card"] == "goblin-token"))
        }) {
            faults.push("token-power");
        }
        for fault in faults.iter().copied() {
            let mut bad = fixture.clone();
            let ops = bad["operations"].as_array_mut().unwrap();
            match fault {
                "mode" => {
                    let op = ops
                        .iter_mut()
                        .find(|v| v["action"]["kind"] == "mode")
                        .unwrap();
                    op["action"]["mode"] = json!(1 - op["action"]["mode"].as_u64().unwrap());
                }
                "discard" => {
                    let i = ops
                        .iter()
                        .position(|v| v["action"]["kind"] == "discard")
                        .unwrap();
                    let actor = ops[i]["action"]["actor"].as_u64().unwrap() as usize;
                    let before = ops[..i]
                        .iter()
                        .rev()
                        .find_map(|v| v.get("checkpoint"))
                        .unwrap();
                    let other = before["hand"][actor]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|v| {
                            **v != ops[i]["action"]["source"]
                                && !v.as_str().unwrap().contains("/thrill-of-possibility/")
                        })
                        .unwrap()
                        .clone();
                    let incarnation = before["incarnations"][other.as_str().unwrap()].clone();
                    ops[i]["action"]["source"] = other;
                    ops[i]["action"]["incarnation"] = incarnation;
                }
                "response-order" => {
                    // Move the response before the opponent's priority pass,
                    // preserving sequence numbers so legality catches it.
                    let i = ops
                        .iter()
                        .position(|v| {
                            v["action"]["kind"] == "cast"
                                && v["action"]["source"]
                                    .as_str()
                                    .is_some_and(|s| s.contains("/giant-growth/"))
                        })
                        .unwrap();
                    let j = ops[..i]
                        .iter()
                        .rposition(|v| v["action"]["kind"] == "pass")
                        .unwrap();
                    let left = ops[j]["action"].clone();
                    let right = ops[i]["action"].clone();
                    ops[j]["action"] = right.clone();
                    ops[j]["action"]["sequence"] = left["sequence"].clone();
                    ops[i]["action"] = left;
                    ops[i]["action"]["sequence"] = right["sequence"].clone();
                }
                "effect-amount" => {
                    let effects = ops
                        .iter_mut()
                        .find_map(|v| {
                            v.get_mut("checkpoint")
                                .and_then(|p| p.get_mut("effects"))
                                .and_then(Value::as_object_mut)
                                .filter(|e| !e.is_empty())
                        })
                        .unwrap();
                    let boost = effects.values_mut().next().unwrap();
                    boost[0] = json!(boost[0].as_u64().unwrap() + 1);
                }
                "token-power" => {
                    let token = ops
                        .iter_mut()
                        .filter_map(|v| {
                            v.get_mut("checkpoint")
                                .and_then(|p| p.get_mut("permanents"))
                                .and_then(Value::as_object_mut)
                        })
                        .flat_map(|p| p.values_mut())
                        .find(|c| c["card"] == "goblin-token")
                        .unwrap();
                    token["power"] = json!(2);
                }
                "incarnation" => {
                    let op = ops
                        .iter_mut()
                        .find(|v| v["action"]["kind"] == "cast")
                        .unwrap();
                    op["action"]["incarnation"] =
                        json!(op["action"]["incarnation"].as_u64().unwrap() + 1);
                }
                "target-incarnation" => {
                    let op = ops
                        .iter_mut()
                        .find(|v| v["action"]["kind"] == "target")
                        .unwrap();
                    op["action"]["incarnation"] =
                        json!(op["action"]["incarnation"].as_u64().unwrap() + 1);
                }
                "payment" => {
                    let op = ops
                        .iter_mut()
                        .find(|v| v["action"]["kind"] == "pay")
                        .unwrap();
                    op["action"]["color"] = json!(0);
                }
                "pass" => {
                    let op = ops
                        .iter_mut()
                        .find(|v| v["action"]["kind"] == "pass")
                        .unwrap();
                    op["action"]["actor"] = json!(1 - op["action"]["actor"].as_u64().unwrap());
                }
                "checkpoint" => {
                    let op = ops
                        .iter_mut()
                        .find(|v| v.get("checkpoint").is_some())
                        .unwrap();
                    op["checkpoint"]["life"][0] = json!(19);
                }
                _ => unreachable!(),
            }
            assert!(
                forge_casting_execute(&bad)
                    .unwrap_err()
                    .contains("first divergence"),
                "{fault}"
            );
        }
        run["negative_controls_rejected"] = json!(faults);
        results.push(run);
    }
    if let Ok(path) = std::env::var("MTG_FORGE_CASTING_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}
