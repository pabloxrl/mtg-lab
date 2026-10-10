// Version-5 test-only played activation client. All choices use scalar semantic actions.
struct ActivationClient {
    client: PlayedClient,
    activation_action: Option<usize>,
    cast_start: Option<Value>,
}
impl ActivationClient {
    fn object(&self, g: &Game, e: &PlayedChoice) -> Result<actions::ObjectRef, String> {
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
            .activation
            .as_ref()
            .map(|a| a.actor)
            .or_else(|| g.turns.payment.as_ref().map(|p| p.actor()))
            .or_else(|| g.turn_decision().map(|d| d.actor))
            .unwrap();
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
                    "power":c.map(|c|c.power),"toughness":c.map(|c|c.toughness),"damage":c.map(|c|c.damage),"haste":g.has_haste(h),"trample":g.has_trample(h)}));
            }
        }
        let stack: Vec<_> = g
            .turns
            .stack
            .iter()
            .map(|h| {
                let id = self.client.id(g, *h);
                if let Some(a) = g.turns.abilities.iter().find(|a| a.object == *h) {
                    json!({"source":self.client.id(g,a.source),"incarnation":g.objects.semantic_identity(a.source).unwrap().1,
                        "action":self.client.actions[&id],"object":id,"raw_birth":g.objects.semantic_identity(*h).unwrap().0,
                        "ability":if a.power {"power"} else if a.invoker {"invoker"} else {"haste"},
                        "target":a.target.map(|h|self.reference(g,h))})
                } else {
                    json!({"source":id,"incarnation":g.objects.semantic_identity(*h).unwrap().1,
                        "action":self.client.actions[&id],"object":id,"ability":"spell","target":null})
                }
            })
            .collect();
        let payment=g.turns.payment.as_ref().map(|p|{
            let c=g.turns.casting.as_ref().unwrap();
            json!({"source":self.client.id(g,c.card()),"actor":seat_index(p.actor()),
                "pool":p.pool(),"colored":p.remaining().colored,"generic":p.remaining().generic,
                "sources":c.sources().iter().map(|h|self.client.id(g,*h)).collect::<Vec<_>>(),
                "targets":self.targets(g,c.effect()),"mode":c.mode(),"discard":c.discard().map(|h|self.reference(g,h))})
        });
        let activation=g.turns.activation.as_ref().map(|a|json!({
            "source":self.reference(g,a.source),"actor":seat_index(a.actor),"target":a.target.map(|h|self.reference(g,h)),
            "paid":a.paid,"reserved":a.reserved,"pool":g.activation_pool(a).unwrap(),
            "sources":a.sources.iter().map(|h|self.reference(g,*h)).collect::<Vec<_>>(),
            "ability":if a.power {"power"} else if a.invoker {"invoker"} else {"haste"}}));
        let effects: Vec<_> = g
            .turns
            .modifications
            .iter()
            .map(|m| {
                json!({"object":self.reference(g,m.handle),
            "boost":m.boost,"power_boost":m.power_boost,"damage":m.damage})
            })
            .collect();
        json!({"boundary":boundary,"turn":turn,"step":priority_step(step),"active":seat_index(active),"actor":seat_index(actor),
            "life":g.life(),"mana":g.mana(),"land_plays":u8::from(g.turns.land_used),
            "hand":[ordered(Zone::Hand(Seat::P0)),ordered(Zone::Hand(Seat::P1))],
            "library":[ordered(Zone::Library(Seat::P0)),ordered(Zone::Library(Seat::P1))],
            "graveyard":[ordered(Zone::Graveyard(Seat::P0)),ordered(Zone::Graveyard(Seat::P1))],
            "exile":ordered(Zone::Exile),"battlefield":ordered(Zone::Battlefield),
            "stack":stack,"incarnations":incarnations,"permanents":permanents,
            "payment":payment,"activation":activation,"effects":effects,"haste":g.turns.haste.iter().map(|h|self.reference(g,*h)).collect::<Vec<_>>(),"trample":g.turns.trample.iter().map(|h|self.reference(g,*h)).collect::<Vec<_>>()})
    }
    fn apply(&mut self, g: &mut Game, e: &PlayedChoice) -> Result<(), String> {
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
            matches!(e.kind.as_str(), "pay" | "activation_pay") == e.color.is_some(),
            "/payment color field",
        )?;
        require(
            matches!(
                e.kind.as_str(),
                "cast" | "play_land" | "tap_mana" | "activate" | "activation_target"
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
            let payment = g.turns.payment.is_some();
            let activation_kind = if g
                .turns
                .activation
                .as_ref()
                .is_some_and(|p| p.power || (p.invoker && p.target.is_some()))
            {
                "activation_payment"
            } else {
                "activation_target"
            };
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
                    if g.turns.activation.is_some() {
                        activation_kind
                    } else if payment {
                        "payment"
                    } else {
                        "priority"
                    },
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
                "activate" => {
                    self.cast_start = self.client.points.last().cloned();
                    self.activation_action = Some(e.sequence);
                    (
                        "priority",
                        actions::Choice::Activate {
                            card: self.object(g, e)?,
                        },
                    )
                }
                "activation_target" => (
                    "activation_target",
                    actions::Choice::Target {
                        card: self.object(g, e)?,
                    },
                ),
                "activation_pay" => (
                    "activation_payment",
                    actions::Choice::Pay {
                        color: e.color.unwrap(),
                    },
                ),
                "finish_activation" => (activation_kind, actions::Choice::FinishActivation {}),
                "cancel_activation" => (activation_kind, actions::Choice::CancelActivation {}),
                "cancel_payment" => ("payment", actions::Choice::CancelPayment {}),
                _ => return Err("first divergence: /unsupported spell callback".into()),
            };
            self.client.submit(g, actor, kind, choice)?;
            if matches!(e.kind.as_str(), "cancel_payment" | "cancel_activation") {
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
                    g.turns.payment.is_none()
                        && g.turns.casting.is_none()
                        && g.turns.activation.is_none(),
                    "/cancel left pending cast",
                )?;
            }
        }
        if e.kind == "finish_activation" {
            let h = *g
                .turns
                .stack
                .last()
                .ok_or("first divergence: /missing committed ability")?;
            let birth = g.objects.semantic_identity(h).unwrap().0;
            let action = self
                .activation_action
                .take()
                .ok_or("first divergence: /missing activation action")?;
            let id = format!("ability/{action}");
            require(
                !self.client.births.contains_key(&birth),
                "/duplicate ability identity",
            )?;
            self.client.births.insert(birth, id.clone());
            self.client.actions.insert(id, action);
        }
        self.client.consumed.push(serde_json::to_value(e).unwrap());
        self.client.cursor += 1;
        Ok(())
    }
    fn run(&mut self, g: &mut Game, case: &Value) -> Result<(), String> {
        let play: Vec<PlayedChoice> = serde_json::from_value(case["play"].clone())
            .map_err(|e| format!("first divergence: /play fields {e}"))?;
        let stop = case["stop"].as_str().ok_or("first divergence: /stop")?;
        require(stop.starts_with("activations/"), "/stop")?;
        for e in &play {
            self.apply(g, e).map_err(|err| {
                format!(
                    "first divergence: /play/{} {}",
                    e.sequence,
                    err.trim_start_matches("first divergence: ")
                )
            })?;
            let field = |source: &str| {
                self.client
                    .handle(g, source)
                    .ok()
                    .and_then(|h| g.creature_state(h))
            };
            let done = g.turns.stack.is_empty()
                && g.turns.payment.is_none()
                && g.turns.activation.is_none()
                && match stop {
                    "activations/shivan-single" | "activations/shivan-floating" => {
                        field("0/shivan-dragon/0").is_some_and(|c| c.power == 6)
                    }
                    "activations/shivan-repeated" => {
                        field("0/shivan-dragon/0").is_some_and(|c| c.power == 7)
                    }
                    "activations/invoker-empty" | "activations/invoker-floating" => {
                        field("1/bear-cub/0").is_some_and(|c| c.power == 7)
                    }
                    "activations/haste" => {
                        g.turns.mana[1][4] == 2
                            && ["1/llanowar-elves/1", "1/druid-of-the-cowl/1"]
                                .iter()
                                .all(|id| {
                                    self.client.handle(g, id).is_ok_and(|h| {
                                        g.has_haste(h) && g.objects.get(h).unwrap().tapped
                                    })
                                })
                    }
                    _ => false,
                };
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
fn activations_consume(
    g: &mut Game,
    doc: Value,
    rejected: &mut Value,
) -> Result<BTreeMap<String, Value>, String> {
    require(
        doc["schema_version"] == 5 && doc["family"] == "activations",
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
                let mut client = ActivationClient {
                    client: PlayedClient::new(g, tape, case)?,
                    activation_action: None,
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
fn full_pool_activations_literal_checkpoints() {
    let doc: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-activations.json"
    ))
    .unwrap();
    let mut g = Game::new().unwrap();
    let runs = activations_consume(&mut g, doc.clone(), &mut Value::Null)
        .expect("legal played activations");
    let repeat = activations_consume(&mut g, doc, &mut Value::Null).unwrap();
    assert_eq!(runs, repeat);
    let points: BTreeMap<_, _> = runs
        .iter()
        .map(|(id, r)| (id.clone(), r["points"].clone()))
        .collect();
    let last = |name: &str| points[name].as_array().unwrap().last().unwrap();
    assert_eq!(
        last("shivan-single")["permanents"]["0/shivan-dragon/0"]["power"],
        6
    );
    assert_eq!(
        last("shivan-repeated")["permanents"]["0/shivan-dragon/0"]["power"],
        7
    );
    for name in ["invoker-empty", "invoker-floating"] {
        assert_eq!(last(name)["permanents"]["1/bear-cub/0"]["power"], 7);
        assert_eq!(last(name)["permanents"]["1/bear-cub/0"]["trample"], true);
    }
    let negatives: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-activations-native-negatives.json"
    ))
    .unwrap();
    let mut rejections = BTreeMap::new();
    let mut negative_runs = BTreeMap::new();
    for (name, spec) in negatives.as_object().unwrap() {
        let before = format!("{g:?}");
        let mut rejected = Value::Null;
        let error = activations_consume(&mut g, spec["input"].clone(), &mut rejected)
            .expect_err("invalid activation tape accepted");
        let prefix = format!(
            "first divergence: /play/{} {}",
            spec["sequence"],
            spec["category"].as_str().unwrap()
        );
        assert!(
            error.starts_with(&prefix),
            "{name}: expected {prefix}, actual {error}"
        );
        assert_eq!(
            format!("{g:?}"),
            before,
            "rejected envelope changed caller state/RNG"
        );
        assert!(!rejected.is_null());
        rejections.insert(name.clone(), error);
        negative_runs.insert(name.clone(), rejected);
    }
    let cancels: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-activations-cancellations.json"
    ))
    .unwrap();
    let mut cancelled_runs = BTreeMap::new();
    for (name, spec) in cancels.as_object().unwrap() {
        let run = activations_consume(&mut g, spec["input"].clone(), &mut Value::Null)
            .expect("cancel at every continuation stage then execute original activation");
        cancelled_runs.insert(name.clone(), run);
    }
    if std::env::var("MTG_FULL_POOL_FAMILY").as_deref() == Ok("activations") {
        let path = std::env::var("MTG_FULL_POOL_OUTPUT").unwrap();
        std::fs::write(path,serde_json::to_vec(&json!({"checkpoints":points,"runs":runs,"repeat_runs":repeat,"cancelled_runs":cancelled_runs,"rejections":rejections,"negative_runs":negative_runs,"rejection_state_rng":"unchanged"})).unwrap()).unwrap();
    }
}
