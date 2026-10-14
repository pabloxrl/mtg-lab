// Test-only version-4 spell client, using the delivered scalar semantic actions.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SpellChoice {
    sequence: usize,
    turn: u64,
    step: String,
    actor: u8,
    kind: String,
    source: Option<String>,
    incarnation: Option<u64>,
    color: Option<u8>,
    role: Option<String>,
    mode: Option<u8>,
}
struct SpellClient {
    client: PlayedClient,
    creation: usize,
    creations: Vec<Value>,
    departures: Vec<Value>,
    cast_start: Option<Value>,
}
impl SpellClient {
    fn object(&self, g: &Game, e: &SpellChoice) -> Result<actions::ObjectRef, String> {
        let h = self.client.handle(
            g,
            e.source
                .as_deref()
                .ok_or("first divergence: /missing source")?,
        )?;
        let o = g.objects.get(h).unwrap();
        Ok(actions::ObjectRef {
            birth: g.objects.semantic_identity(h).unwrap().0,
            incarnation: e
                .incarnation
                .ok_or("first divergence: /missing incarnation")?,
            card: o.card.identity().key.into(),
            owner: o.owner,
            zone: o.zone,
        })
    }
    fn reference(&self, g: &Game, h: Handle) -> Value {
        match g.objects.semantic_identity(h) {
            Ok((birth, incarnation)) => {
                json!({"source":self.client.births[&birth],"incarnation":incarnation})
            }
            Err(_) => Value::Null,
        }
    }
    fn targets(&self, g: &Game, effect: Option<targets::Effect>) -> Value {
        match effect {
            Some(targets::Effect::Growth(a)) => {
                json!([{"role":"growth_target","object":self.reference(g,a)}])
            }
            Some(targets::Effect::Bite(a, b)) => json!([
                {"role":"bite_source","object":self.reference(g,a)},
                {"role":"bite_destination","object":self.reference(g,b)}]),
            None => json!([]),
        }
    }
    fn point(&self, g: &Game, boundary: &str) -> Value {
        let (turn, active, step) = g.turn_position().unwrap();
        let actor = g
            .turns
            .targeting
            .as_ref()
            .map(|t| t.decision().actor)
            .or_else(|| g.turns.payment.as_ref().map(|p| p.actor()))
            .or_else(|| g.turn_decision().map(|d| d.actor));
        let ordered = |z| {
            g.objects
                .in_zone(z)
                .map(|h| self.client.id(g, h))
                .collect::<Vec<_>>()
        };
        let mut incarnations = BTreeMap::new();
        let mut permanents = BTreeMap::new();
        for h in Zone::ALL.into_iter().flat_map(|z| g.objects.in_zone(z)) {
            let id = self.client.id(g, h);
            let o = g.objects.get(h).unwrap();
            incarnations.insert(id.clone(), g.objects.semantic_identity(h).unwrap().1);
            if o.zone == Zone::Battlefield {
                let c = g.creature_state(h);
                permanents.insert(id,json!({"card":o.card.identity().key,"owner":seat_index(o.owner),
                    "controller":seat_index(o.controller),"tapped":o.tapped,"sick":g.summoning_sick(h),
                    "power":c.map(|c|c.power),"toughness":c.map(|c|c.toughness),"damage":c.map(|c|c.damage)}));
            }
        }
        let stack: Vec<_> = g
            .turns
            .stack
            .iter()
            .map(|h| {
                let id = self.client.id(g, *h);
                let effect = g
                    .turns
                    .effects
                    .iter()
                    .find(|(s, _)| s == h)
                    .map(|(_, e)| *e);
                json!({"source":id,"incarnation":g.objects.semantic_identity(*h).unwrap().1,
                "action":self.client.actions[&id],"targets":self.targets(g,effect),
                "mode":g.turns.modes.iter().find(|(s,_)|s==h).map(|(_,m)|*m)})
            })
            .collect();
        let payment=g.turns.payment.as_ref().map(|p|{
            let c=g.turns.casting.as_ref().unwrap();
            json!({"source":self.client.id(g,c.card()),"actor":seat_index(p.actor()),
                "pool":p.pool(),"colored":p.remaining().colored,"generic":p.remaining().generic,
                "sources":c.sources().iter().map(|h|self.client.id(g,*h)).collect::<Vec<_>>(),
                "targets":self.targets(g,c.effect()),"mode":c.mode(),"discard":c.discard().map(|h|self.reference(g,h))})
        });
        let targeting=g.turns.targeting.as_ref().map(|t|json!({"source":self.client.id(g,t.card()),
            "role":format!("{:?}",t.decision().kind),"selected":t.selected().iter().map(|h|self.reference(g,*h)).collect::<Vec<_>>()}));
        let effects: Vec<_> = g
            .turns
            .modifications
            .iter()
            .map(|m| {
                json!({"object":self.reference(g,m.handle),
            "boost":m.boost,"power_boost":m.power_boost,"damage":m.damage})
            })
            .collect();
        json!({"boundary":boundary,"turn":turn,"step":priority_step(step),"active":seat_index(active),"actor":actor.map(seat_index),
            "life":g.life(),"mana":g.mana(),"land_plays":u8::from(g.turns.land_used),
            "hand":[ordered(Zone::Hand(Seat::P0)),ordered(Zone::Hand(Seat::P1))],
            "library":[ordered(Zone::Library(Seat::P0)),ordered(Zone::Library(Seat::P1))],
            "graveyard":[ordered(Zone::Graveyard(Seat::P0)),ordered(Zone::Graveyard(Seat::P1))],
            "exile":ordered(Zone::Exile),"battlefield":ordered(Zone::Battlefield),
            "stack":stack,"incarnations":incarnations,"permanents":permanents,
            "payment":payment,"targeting":targeting,"effects":effects,
            "creations":self.creations,"departures":self.departures})
    }
    fn apply(&mut self, g: &mut Game, e: &SpellChoice) -> Result<(), String> {
        require(
            e.actor < 2 && e.sequence == self.client.cursor,
            "/play actor/sequence",
        )?;
        let actor = if e.actor == 0 { Seat::P0 } else { Seat::P1 };
        let (turn, active, step) = g.turn_position().ok_or("first divergence: /missing turn")?;
        require(
            turn == e.turn && priority_step(step) == e.step,
            "/play position",
        )?;
        require(
            (e.kind == "pay") == e.color.is_some(),
            "/payment color field",
        )?;
        require((e.kind == "mode") == e.mode.is_some(), "/mode field")?;
        require(
            (e.kind == "target") == e.role.is_some(),
            "/target role field",
        )?;
        require(
            matches!(
                e.kind.as_str(),
                "cast" | "play_land" | "tap_mana" | "target" | "discard"
            ) == e.source.is_some(),
            "/source field",
        )?;
        require(
            e.source.is_some() == e.incarnation.is_some(),
            "/incarnation field",
        )?;
        self.client
            .points
            .push(self.point(g, &format!("before/{}", e.sequence)));
        let before: Vec<_> = g
            .objects
            .in_zone(Zone::Battlefield)
            .map(|h| (h, self.reference(g, h)))
            .collect();
        if e.kind == "empty_attackers" {
            require(
                step == turns::Step::DeclareAttackers
                    && actor == active
                    && self.client.declared.insert(turn),
                "/empty declaration",
            )?;
            if g.turn_decision().is_some_and(|d| {
                matches!(
                    d.kind,
                    turns::TurnKind::Combat(combat::CombatKind::Attackers)
                )
            }) {
                self.client.submit(
                    g,
                    actor,
                    "attackers",
                    actions::Choice::SelectAttackers { cards: vec![] },
                )?;
                self.client
                    .submit(g, actor, "attackers", actions::Choice::FinishCombat {})?;
            } else {
                require(
                    !g.has_combat_creature(active),
                    "/unwitnessed empty declaration",
                )?;
            }
        } else {
            if step == turns::Step::DeclareAttackers {
                require(
                    self.client.declared.contains(&turn),
                    "/missing empty declaration",
                )?;
            }
            if e.kind == "tap_mana" {
                let h = self.client.handle(g, e.source.as_deref().unwrap())?;
                require(
                    mana::basic_color(g.objects.get(h).unwrap().card).is_some(),
                    "/unsupported mana callback",
                )?;
            }
            let payment = g.turns.payment.is_some();
            let (kind, choice) = match e.kind.as_str() {
                "pass" => ("priority", actions::Choice::Pass {}),
                "play_land" => (
                    "priority",
                    actions::Choice::PlayLand {
                        card: self.object(g, e)?,
                    },
                ),
                "cast" => {
                    self.cast_start = self.client.points.last().cloned();
                    let reference = self.object(g, e)?;
                    self.client
                        .actions
                        .insert(e.source.clone().unwrap(), e.sequence);
                    ("priority", actions::Choice::Cast { card: reference })
                }
                "tap_mana" => (
                    if payment { "payment" } else { "priority" },
                    actions::Choice::TapMana {
                        card: self.object(g, e)?,
                    },
                ),
                "pay" => (
                    "payment",
                    actions::Choice::Pay {
                        color: e.color.unwrap(),
                    },
                ),
                "finish_payment" => ("payment", actions::Choice::FinishPayment {}),
                "mode" => (
                    "cast_mode",
                    actions::Choice::Mode {
                        mode: e.mode.unwrap(),
                    },
                ),
                "discard" => (
                    if step == turns::Step::Cleanup && g.turns.casting.is_none() { "cleanup_discard" } else { "cast_discard" },
                    actions::Choice::Discard {
                        card: self.object(g, e)?,
                    },
                ),
                "target" => {
                    let role = e.role.as_deref().unwrap();
                    require(
                        matches!(role, "growth_target" | "bite_source" | "bite_destination"),
                        "/target role",
                    )?;
                    (
                        role,
                        actions::Choice::Target {
                            card: self.object(g, e)?,
                        },
                    )
                }
                "finish_targets" => ("targets_complete", actions::Choice::FinishTargets {}),
                "cancel_payment" => ("payment", actions::Choice::CancelPayment {}),
                _ => return Err("first divergence: /unsupported spell callback".into()),
            };
            self.client.submit(g, actor, kind, choice)?;
            if e.kind == "cancel_payment" {
                let before = self
                    .cast_start
                    .as_ref()
                    .ok_or("first divergence: /cancellation without cast")?;
                let after = self.point(g, "cancelled");
                for field in [
                    "mana",
                    "hand",
                    "graveyard",
                    "battlefield",
                    "stack",
                    "incarnations",
                    "permanents",
                    "effects",
                ] {
                    require(
                        before[field] == after[field],
                        "/cancel partially committed cast",
                    )?;
                }
                require(
                    g.turns.payment.is_none() && g.turns.casting.is_none(),
                    "/cancel left pending cast",
                )?;
            }
        }
        let mut born: Vec<_> = g
            .objects
            .in_zone(Zone::Battlefield)
            .filter(|h| {
                !self
                    .client
                    .births
                    .contains_key(&g.objects.semantic_identity(*h).unwrap().0)
            })
            .collect();
        born.sort_by_key(|h| g.objects.semantic_identity(*h).unwrap().0);
        if !born.is_empty() {
            require(e.kind == "pass", "/creation without resolution")?;
            for (ordinal, h) in born.iter().enumerate() {
                let o = g.objects.get(*h).unwrap();
                let (birth, incarnation) = g.objects.semantic_identity(*h).unwrap();
                require(
                    o.card.identity().key == "goblin-token",
                    "/unsupported created object",
                )?;
                let id = format!(
                    "token/{}/{}/{}",
                    seat_index(o.controller),
                    self.creation,
                    ordinal
                );
                require(
                    !self.client.births.values().any(|old| old == &id),
                    "/duplicate token id",
                )?;
                self.client.births.insert(birth, id.clone());
                self.creations.push(
                    json!({"id":id,"birth":birth,"incarnation":incarnation,"action":e.sequence,"card":o.card.identity().key,"definition_hash":o.card.identity().content_sha256}),
                );
            }
            self.creation += 1;
        }
        for (h, reference) in before {
            if g.objects.get(h).is_err() {
                self.departures
                    .push(json!({"object":reference,"action":e.sequence,"old_handle_valid":false}));
            }
        }
        self.client.consumed.push(serde_json::to_value(e).unwrap());
        self.client.cursor += 1;
        Ok(())
    }
    fn run(&mut self, g: &mut Game, case: &Value) -> Result<(), String> {
        let play: Vec<SpellChoice> = serde_json::from_value(case["play"].clone())
            .map_err(|e| format!("first divergence: /play fields {e}"))?;
        let resolved = case["stop"]
            .as_str()
            .and_then(|s| s.strip_prefix("resolved/"))
            .ok_or("first divergence: /stop")?;
        for e in &play {
            self.apply(g, e).map_err(|err| {
                format!(
                    "first divergence: /play/{} {}",
                    e.sequence,
                    err.trim_start_matches("first divergence: ")
                )
            })?;
            let done = g.turns.stack.is_empty()
                && g.turns.payment.is_none()
                && g.turns.targeting.is_none()
                && [Seat::P0, Seat::P1].into_iter().any(|s| {
                    g.objects
                        .in_zone(Zone::Graveyard(s))
                        .any(|h| self.client.id(g, h) == resolved)
                });
            if done {
                require(
                    self.client.cursor == play.len(),
                    "/extra choice after named stop",
                )?;
                self.client
                    .points
                    .push(self.point(g, case["stop"].as_str().unwrap()));
                return Ok(());
            }
        }
        Err("first divergence: /missing choice before named stop".into())
    }
}
fn spells_consume(
    g: &mut Game,
    doc: Value,
    rejected: &mut Value,
) -> Result<BTreeMap<String, Value>, String> {
    require(
        doc["schema_version"] == 4 && doc["family"] == "spells",
        "/version/family",
    )?;
    let mut projection = doc.clone();
    projection["schema_version"] = json!(3);
    projection["family"] = json!("priority");
    let (opening, reset) = priority_projection(&projection)?;
    let mut trial = Game::new().unwrap();
    let mut runs = BTreeMap::new();
    for (n, case) in doc["cases"].as_array().unwrap().iter().enumerate() {
        let (opened, (client, error)) = london_continue(
            &mut trial,
            &reset.cases[n],
            &opening["cases"][n],
            |g, tape| {
                let mut client = SpellClient {
                    client: PlayedClient::new(g, tape, case)?,
                    creation: 0,
                    creations: vec![],
                    departures: vec![],
                    cast_start: None,
                };
                let error = client.run(g, case).err();
                Ok((client, error))
            },
        )?;
        let mut report = client.client.report();
        report["opening"] = opened;
        if let Some(error) = error {
            report["rejected_checkpoint"] = client.point(&trial, "rejected");
            *rejected = report;
            return Err(error);
        }
        runs.insert(case["id"].as_str().unwrap().to_string(), report);
    }
    *g = trial;
    Ok(runs)
}
#[test]
fn full_pool_spells_literal_checkpoints() {
    let doc: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-spells.json"
    ))
    .unwrap();
    let mut g = Game::new().unwrap();
    let runs = spells_consume(&mut g, doc.clone(), &mut Value::Null)
        .expect("legal normal-reset spell prefixes");
    let points: BTreeMap<_, _> = runs
        .iter()
        .map(|(id, r)| (id.clone(), r["points"].clone()))
        .collect();
    let repeated = spells_consume(&mut g, doc.clone(), &mut Value::Null).unwrap();
    assert_eq!(
        runs, repeated,
        "repeat actual scalar records and checkpoints"
    );
    let negative_text = if std::env::var("MTG_FULL_POOL_FAMILY").as_deref() == Ok("spells") {
        std::env::var("MTG_FULL_POOL_NEGATIVES")
            .ok()
            .map(|p| std::fs::read_to_string(p).unwrap())
    } else {
        None
    };
    let negatives: Value = serde_json::from_str(negative_text.as_deref().unwrap_or(include_str!(
        "../../../fixtures/reference/full-pool-spells-native-negatives.json"
    )))
    .unwrap();
    let mut rejections = BTreeMap::new();
    let mut negative_runs = BTreeMap::new();
    for (name, control) in negatives.as_object().unwrap() {
        let before = format!("{g:?}");
        let mut rejected = Value::Null;
        let error = spells_consume(&mut g, control["input"].clone(), &mut rejected)
            .expect_err("invalid tape accepted");
        assert!(
            !rejected.is_null(),
            "missing actual rejected-prefix observations"
        );
        negative_runs.insert(name.clone(), rejected);
        assert_eq!(
            format!("{g:?}"),
            before,
            "rejected envelope changed caller state/RNG"
        );
        let expected = match name.as_str() {
            "truncated_tape" => "first divergence: /missing choice before named stop".to_string(),
            "extra_tape" => "first divergence: /extra choice after named stop".to_string(),
            _ => format!(
                "first divergence: /play/{} {}",
                control["sequence"],
                control["category"].as_str().unwrap()
            ),
        };
        assert!(
            error.starts_with(&expected),
            "{name}: intended boundary {expected}, actual {error}"
        );
        assert!(
            !error.contains("missing choice before named stop") || name == "truncated_tape",
            "late failure cannot stand in for illegal action"
        );
        rejections.insert(name.clone(), error);
    }
    if let Ok(p) = std::env::var("MTG_FULL_POOL_OUTPUT")
        && std::env::var("MTG_FULL_POOL_FAMILY").as_deref() == Ok("spells")
    {
        std::fs::write(p,serde_json::to_vec(&json!({"checkpoints":points,"runs":runs,"repeat_runs":repeated,"rejections":rejections,"negative_runs":negative_runs,"rejection_state_rng":"unchanged"})).unwrap()).unwrap();
    }
    let last = |name: &str| points[name].as_array().unwrap().last().unwrap();
    assert_eq!(
        last("departed-target")["permanents"]["1/bear-cub/0"]["power"],
        2
    );
    assert_eq!(last("fodder")["permanents"]["token/0/0/0"]["power"], 1);
    assert_eq!(last("fodder")["permanents"]["token/0/0/1"]["toughness"], 1);
    assert_eq!(
        last("surprise-boost")["permanents"]["token/0/0/0"]["power"],
        3
    );
    assert_eq!(
        last("surprise-boost")["permanents"]["token/0/0/1"]["toughness"],
        1
    );
    assert_eq!(
        last("surprise-tokens")["creations"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        last("growth-bite")["permanents"]["1/bear-cub/0"]["power"],
        5
    );
    assert_eq!(
        last("growth-bite")["permanents"]["1/bear-cub/0"]["toughness"],
        5
    );
    assert!(
        last("growth-bite")["permanents"]
            .get("token/0/0/1")
            .is_none()
    );
    assert_eq!(last("growth-bite")["permanents"]["token/0/0/0"]["power"], 1);
    assert_eq!(
        last("thrill")["graveyard"][0],
        json!(["0/mountain/2", "0/thrill-of-possibility/0"])
    );
    assert_eq!(
        last("thrill")["hand"][0]
            .as_array()
            .unwrap()
            .iter()
            .rev()
            .take(2)
            .cloned()
            .collect::<Vec<_>>(),
        vec![json!("0/mountain/7"), json!("0/mountain/6")]
    );
}

#[test]
fn played_cleanup_discard_client_regression() {
    // CR 514.1: after the second turn's draw/pass prefix, discard the drawn
    // occurrence and start the third upkeep with exactly seven cards.
    let mut doc: Value = serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-cleanup.json")).unwrap();
    doc["schema_version"] = json!(4);
    doc["family"] = json!("spells");
    let mut case = doc["cases"][0].clone();
    let play = case["play"].as_array_mut().unwrap();
    let end = play.iter().position(|e| e["kind"] == "cleanup_discard").unwrap();
    let selected=play[end]["selection"][0].clone();
    play[end]=json!({"sequence":end,"turn":2,"step":"cleanup","actor":1,"kind":"discard","source":selected["source"],"incarnation":selected["incarnation"],"color":null,"role":null,"mode":null});
    let discarded = play[end]["source"].as_str().unwrap().to_owned();
    play.truncate(end + 1);
    case["stop"] = json!(format!("resolved/{discarded}"));
    doc["cases"] = json!([case]);
    let mut g = Game::new().unwrap();
    let runs = spells_consume(&mut g, doc, &mut Value::Null)
        .expect("CR514.1 legal played cleanup discard must be consumed");
    let final_point = runs.values().next().unwrap()["points"].as_array().unwrap().last().unwrap();
    assert_eq!(final_point["turn"], 3);
    assert_eq!(final_point["hand"][1].as_array().unwrap().len(), 7);
}
