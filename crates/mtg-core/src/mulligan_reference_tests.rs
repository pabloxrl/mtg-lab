// Included in full_pool_reference_tests: version 1 remains unchanged.
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct LondonChoice {
    sequence: usize,
    kind: String,
    actor: usize,
    source: Value,
    round: usize,
    selection: Value,
}
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
struct LondonChance {
    sequence: usize,
    kind: String,
    actor: usize,
    source: Value,
    before: Vec<String>,
    after: Vec<String>,
}
fn london_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-mulligan.json"
    ))
    .unwrap()
}
fn london_reset(doc: &Value) -> Result<Envelope, String> {
    require(
        doc["schema_version"] == json!(2) && doc["family"] == "mulligan",
        "/version/family",
    )?;
    let mut projection = doc.clone();
    projection["schema_version"] = json!(1);
    projection["family"] = json!("reset");
    for c in projection["cases"]
        .as_array_mut()
        .ok_or("first divergence: /cases")?
    {
        require(c["stop"] == "first_upkeep", "/stop unsupported callback")?;
        c["stop"] = json!("first_declaration");
        c["choices"] = json!([]);
        let chance = c["chance"]
            .as_array_mut()
            .ok_or("first divergence: /chance")?;
        require(chance.len() >= 2, "/missing reset shuffle")?;
        chance.truncate(2);
        for e in chance {
            require(e.get("source") == Some(&Value::Null), "/chance source")?;
            e.as_object_mut()
                .ok_or("first divergence: /chance fields")?
                .remove("source");
        }
    }
    validate(projection)
}
fn london_ids(g: &Game, tape: &Tape<'_>, zone: Zone) -> Vec<String> {
    g.objects
        .in_zone(zone)
        .map(|h| tape.births[&g.objects.semantic_identity(h).unwrap().0].clone())
        .collect()
}
fn london_point(g: &Game, tape: &Tape<'_>, boundary: &str, actor: Seat) -> Value {
    let mut hand = london_ids(g, tape, Zone::Hand(actor));
    hand.sort();
    json!({"boundary":boundary,"actor":seat_index(actor),"life":g.life(),"hand":hand,
        "library":london_ids(g,tape,Zone::Library(actor)),"mulligans":g.mulligans[seat_index(actor)]})
}
fn london_execute(g: &mut Game, reset: &Case, case: &Value) -> Result<Value, String> {
    london_continue(g, reset, case, |_, _| Ok(())).map(|(opening, ())| opening)
}
fn london_continue<T>(
    g: &mut Game,
    reset: &Case,
    case: &Value,
    continuation: impl FnOnce(&mut Game, &Tape<'_>) -> Result<T, String>,
) -> Result<(Value, T), String> {
    let chance: Vec<LondonChance> = serde_json::from_value(case["chance"].clone())
        .map_err(|e| format!("first divergence: /chance {e}"))?;
    let choices: Vec<LondonChoice> = serde_json::from_value(case["choices"].clone())
        .map_err(|e| format!("first divergence: /choices {e}"))?;
    for (n, e) in chance.iter().enumerate() {
        require(
            e.sequence == n && e.source.is_null() && e.actor < 2,
            "/chance sequence/source/actor",
        )?;
        require(
            e.kind
                == if n < 2 {
                    "initial_shuffle"
                } else {
                    "mulligan_shuffle"
                },
            "/chance kind",
        )?;
        let canonical = &reset.decks[e.actor].occurrences;
        let mut actual = e.after.clone();
        actual.sort();
        let mut wanted = canonical.clone();
        wanted.sort();
        require(
            e.before == *canonical && actual == wanted,
            "/chance occurrences",
        )?;
    }
    for (n, e) in choices.iter().enumerate() {
        require(
            e.sequence == n && e.source.is_null() && e.actor < 2,
            "/choice sequence/source/actor",
        )?;
    }
    let config = Config {
        starting_seat: reset.starter,
        seats: reset
            .decks
            .iter()
            .map(|d| DeckConfig::new(&d.deck))
            .collect(),
        ..Config::default()
    };
    let mut tape = Tape {
        case: reset,
        births: BTreeMap::new(),
        handles: BTreeMap::new(),
        used: vec![],
    };
    g.reset_with_occurrence_chance(&config, 270, 0, &mut tape)
        .map_err(|e| format!("first divergence: reset {e:?}"))?;
    let rng = format!("{:?}", g.rng);
    let mut used_chance: Vec<Value> = tape
        .used
        .iter()
        .map(|e| {
            let mut v = serde_json::to_value(e).unwrap();
            v["source"] = Value::Null;
            v
        })
        .collect();
    let mut used_choices = vec![];
    let mut raw = vec![
        json!({"kind":"initial_shuffle","actor":0}),
        json!({"kind":"initial_shuffle","actor":1}),
    ];
    let mut points = vec![];
    let mut reserved = 2;
    while let Some(d) = g.decision() {
        let e = choices
            .get(used_choices.len())
            .ok_or("first divergence: /missing choice")?;
        let i = seat_index(d.actor);
        require(
            e.actor == i && e.round == g.mulligans[i],
            "/choice actor/stale round",
        )?;
        let mut order = None;
        let selection = match d.kind {
            OpeningKind::KeepOrMulligan => {
                require(e.kind == "declare", "/choice expected declaration")?;
                points.push(london_point(g, &tape, "declaration", d.actor));
                let index = match e.selection.as_str() {
                    Some("keep") => 0,
                    Some("mulligan") => 1,
                    _ => return Err("first divergence: /declaration selection".into()),
                };
                if index == 1 {
                    let event = chance
                        .get(reserved)
                        .ok_or("first divergence: /missing shuffle")?;
                    require(
                        event.actor == i && event.kind == "mulligan_shuffle",
                        "/shuffle actor/kind",
                    )?;
                    let current: BTreeMap<_, _> = g
                        .objects
                        .in_zone(Zone::Hand(d.actor))
                        .chain(g.objects.in_zone(Zone::Library(d.actor)))
                        .map(|h| {
                            (
                                tape.births[&g.objects.semantic_identity(h).unwrap().0].clone(),
                                h,
                            )
                        })
                        .collect();
                    order = Some(
                        event
                            .after
                            .iter()
                            .map(|id| {
                                current.get(id).copied().ok_or(
                                    "first divergence: /stale shuffle occurrence".to_string(),
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                    reserved += 1;
                }
                Selection::Choose(d.candidate(index))
            }
            OpeningKind::Bottom { count } => {
                require(e.kind == "bottom", "/choice expected bottom")?;
                points.push(london_point(g, &tape, "bottom", d.actor));
                let selected: Vec<String> = serde_json::from_value(e.selection.clone())
                    .map_err(|_| "first divergence: /bottom type")?;
                require(selected.len() == count, "/bottom cardinality")?;
                require(
                    selected.iter().collect::<BTreeSet<_>>().len() == count,
                    "/duplicate bottom",
                )?;
                let cards = g.bottom_cards().unwrap();
                let mut candidates = vec![];
                for id in selected {
                    let index = cards
                        .iter()
                        .position(|h| {
                            tape.births[&g.objects.semantic_identity(*h).unwrap().0] == id
                        })
                        .ok_or("first divergence: /stale bottom occurrence")?;
                    candidates.push(d.candidate(index));
                }
                Selection::Bottom(candidates)
            }
        };
        let action = OpeningAction {
            decision: d.id,
            selection,
        };
        // Submit through the production validation/work API. Drain one unit at
        // a time only to witness actual post-shuffle order before any draw.
        g.apply_quantum(
            d.actor,
            &action,
            order.as_deref(),
            NonZeroUsize::new(1).unwrap(),
        )
        .map_err(|e| format!("first divergence: /apply {e:?}"))?;
        used_choices.push(serde_json::to_value(e).unwrap());
        raw.push(json!({"kind":e.kind,"actor":i}));
        let mut pre_shuffle = None;
        while !g.work.is_empty() {
            if let Some(Work::Redraw { seat, phase: 2, .. }) = g.work.front() {
                pre_shuffle = Some(london_ids(g, &tape, Zone::Library(*seat)));
            }
            if let Some(Work::Redraw {
                seat,
                phase: 3,
                position: 0,
                ..
            }) = g.work.front()
            {
                let event = chance
                    .get(used_chance.len())
                    .ok_or("first divergence: /extra shuffle")?;
                require(event.actor == seat_index(*seat), "/runtime shuffle actor")?;
                let after = london_ids(g, &tape, Zone::Library(*seat));
                require(after == event.after, "/runtime shuffled order")?;
                let mut before = pre_shuffle
                    .take()
                    .ok_or("first divergence: /unwitnessed pre-shuffle state")?;
                let canonical = &reset.decks[event.actor].occurrences;
                before.sort_by_key(|id| canonical.iter().position(|c| c == id).unwrap());
                used_chance.push(json!({"sequence":used_chance.len(),"kind":"mulligan_shuffle","actor":seat_index(*seat),"source":null,"before":before,"after":after}));
                raw.push(json!({"kind":"mulligan_shuffle","actor":seat_index(*seat)}));
            }
            g.resume(NonZeroUsize::new(1).unwrap());
        }
        require(
            format!("{:?}", g.rng) == rng,
            "/unexpected native RNG consumption",
        )?;
    }
    require(
        used_choices.len() == choices.len() && used_chance.len() == chance.len(),
        "/incomplete ledger",
    )?;
    let turn = g
        .start_turns()
        .map_err(|e| format!("first divergence: /start turns {e:?}"))?;
    require(
        g.turn_position() == Some((1, g.starting, turns::Step::Upkeep)) && turn.actor == g.starting,
        "/first upkeep",
    )?;
    for seat in if reset.starter == 0 {
        [Seat::P0, Seat::P1]
    } else {
        [Seat::P1, Seat::P0]
    } {
        points.push(london_point(g, &tape, "first_upkeep", seat));
    }
    let result = continuation(g, &tape)?;
    Ok((
        json!({"points":points,"consumed_chance":used_chance,"consumed_choices":used_choices,"raw_callbacks":raw,"native_rng":"unchanged"}),
        result,
    ))
}
fn london_consume(g: &mut Game, doc: Value) -> Result<BTreeMap<String, Value>, String> {
    let reset = london_reset(&doc)?;
    // Transactional test-client admission: partial invalid transcripts never
    // replace the caller's game or RNG. The real engine executes every step.
    let mut trial = Game::new().unwrap();
    let mut result = BTreeMap::new();
    for (case, reset) in doc["cases"].as_array().unwrap().iter().zip(&reset.cases) {
        result.insert(reset.id.clone(), london_execute(&mut trial, reset, case)?);
    }
    *g = trial;
    Ok(result)
}
#[test]
fn full_pool_mulligan_literal_checkpoints() {
    let mut g = Game::new().unwrap();
    let runs = london_consume(&mut g, london_fixture()).unwrap();
    let points: BTreeMap<_, _> = runs
        .iter()
        .map(|(id, v)| (id.clone(), v["points"].clone()))
        .collect();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-mulligan-expectations.json"
    ))
    .unwrap();
    assert_eq!(
        serde_json::to_value(&points).unwrap(),
        expected,
        "CR 103.5 literal chronology"
    );
    assert_eq!(runs.len(), 32);
    for case in london_fixture()["cases"].as_array().unwrap() {
        let run = &runs[case["id"].as_str().unwrap()];
        assert_eq!(run["consumed_chance"], case["chance"]);
        assert_eq!(run["consumed_choices"], case["choices"]);
    }
    let repeated = london_consume(&mut g, london_fixture()).unwrap();
    assert_eq!(runs, repeated);
    let negatives: Value = match std::env::var("MTG_FULL_POOL_NEGATIVES") {
        Ok(p) if std::env::var("MTG_FULL_POOL_FAMILY").as_deref() == Ok("mulligan") => {
            serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
        }
        _ => serde_json::from_str(include_str!(
            "../../../fixtures/reference/full-pool-mulligan-negative-inputs.json"
        ))
        .unwrap(),
    };
    let mut rejected = BTreeMap::new();
    for (name, bad) in negatives.as_object().unwrap() {
        let before = format!("{g:?}");
        let error = london_consume(&mut g, bad.clone()).unwrap_err();
        assert!(error.starts_with("first divergence:"));
        assert_eq!(
            format!("{g:?}"),
            before,
            "native rejection changed state/RNG: {name}"
        );
        rejected.insert(name.clone(), error);
    }
    assert_eq!(rejected.len(), 32);
    if std::env::var("MTG_FULL_POOL_FAMILY").as_deref() == Ok("mulligan")
        && let Ok(p) = std::env::var("MTG_FULL_POOL_OUTPUT")
    {
        let report = json!({"checkpoints":points,"runs":runs,"repeat_runs":repeated,
            "rejections":rejected,"rejection_state_rng":"unchanged"});
        std::fs::write(p, serde_json::to_string_pretty(&report).unwrap()).unwrap();
    }
}
