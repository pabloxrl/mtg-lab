// Test-only version-6 trigger client, using the delivered scalar semantic actions.
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerChoice {
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
    order: Option<Vec<Value>>,
    player: Option<u8>,
}
struct TriggerClient {
    client: PlayedClient,
    creation: usize,
    creations: Vec<Value>,
    departures: Vec<Value>,
    cast_start: Option<Value>,
    event: u64,
    witnesses: Vec<Value>,
    pending_keys: Vec<(usize, Value)>,
    stack_keys: Vec<(Handle, Value)>,
    old_sources: Vec<(Handle, Value)>,
}
impl TriggerClient {
    fn object(&self, g: &Game, e: &TriggerChoice) -> Result<actions::ObjectRef, String> {
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
            if g.turns.triggered.iter().any(|a| a.object==h) { continue; }
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
                if let Some((_, key)) = self.stack_keys.iter().find(|(o,_)| o == h) {
                    let a = g.turns.triggered.iter().find(|a| a.object == *h).unwrap();
                    return json!({"ability":"trigger","key":key,"controller":seat_index(a.declaration.controller),
                        "target":match a.declaration.kind { triggers::TriggerKind::Pyromancer{target} => target.map(seat_index), _ => None },
                        "raw_birth":g.objects.semantic_identity(*h).unwrap().0,"raw_source_handle":a.declaration.source,"source_card":a.declaration.card.identity().key});
                }
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
        json!({"boundary":boundary,"turn":turn,"step":priority_step(step),"active":seat_index(active),"actor":seat_index(actor),
            "life":g.life(),"mana":g.mana(),"land_plays":u8::from(g.turns.land_used),
            "hand":[ordered(Zone::Hand(Seat::P0)),ordered(Zone::Hand(Seat::P1))],
            "library":[ordered(Zone::Library(Seat::P0)),ordered(Zone::Library(Seat::P1))],
            "graveyard":[ordered(Zone::Graveyard(Seat::P0)),ordered(Zone::Graveyard(Seat::P1))],
            "exile":ordered(Zone::Exile),"battlefield":ordered(Zone::Battlefield),
            "stack":stack,"incarnations":incarnations,"permanents":permanents,
            "payment":payment,"targeting":targeting,"effects":effects,
            "creations":self.creations,"departures":self.departures,
            "pending_triggers":self.pending_keys.iter().filter_map(|(row,key)| g.turns.pending_triggers.get(*row).and_then(|p|p.as_ref()).map(|p|json!({"key":key,"controller":seat_index(p.controller)}))).collect::<Vec<_>>(),
            "trigger_boundary":g.turn_decision().map(|d| match d.kind { turns::TurnKind::TriggerOrder => "order", turns::TurnKind::TriggerTarget => "target", _ => "settled"})})
    }
    fn sync_triggers(&mut self, g: &Game) -> Result<(), String> {
        for h in g.objects.in_zone(Zone::Battlefield) {
            if !self.client.births.contains_key(&g.objects.semantic_identity(h).unwrap().0) {continue;}
            if !self.old_sources.iter().any(|(old,_)| *old==h) {
                self.old_sources.push((h,self.reference(g,h)));
            }
        }
        if self.pending_keys.is_empty() && g.turns.pending_triggers.iter().any(Option::is_some) {
            for (row,p) in g.turns.pending_triggers.iter().enumerate() {
                if let Some(p)=p {
                    let mut key=self.old_sources.iter().find(|(h,_)| *h==p.source).ok_or("first divergence: /unwitnessed trigger source")?.1.clone();
                    key["ability"]=json!(match p.kind { triggers::TriggerKind::Archer=>"archer",triggers::TriggerKind::Cyclops=>"cyclops",triggers::TriggerKind::Pyromancer{..}=>"pyromancer", _=>return Err("first divergence: /unsupported trigger".into()) });
                    key["event"]=json!(self.event);
                    self.witnesses.push(json!({"key":key,"after_choice":self.client.cursor,"raw_source_handle":p.source,
                        "raw_source_identity":g.objects.semantic_identity(p.source).map_err(|_|"first divergence: /missing source incarnation provenance")?,"raw_kind":p.kind}));
                    self.pending_keys.push((row,key));
                }
            }
            self.event+=1;
        }
        for a in &g.turns.triggered {
            if !self.stack_keys.iter().any(|(h,_)| *h==a.object) {
                let source=self.old_sources.iter().find(|(h,_)| *h==a.declaration.source).ok_or("first divergence: /unwitnessed stack source")?;
                let key=self.pending_keys.iter().find(|(_,k)|k["source"]==source.1["source"] && k["incarnation"]==source.1["incarnation"]).ok_or("first divergence: /unwitnessed trigger placement")?.1.clone();
                let birth=g.objects.semantic_identity(a.object).unwrap().0;
                self.client.births.insert(birth,format!("trigger/{birth}"));
                self.stack_keys.push((a.object,key));
            }
        }
        if !g.turns.pending_triggers.iter().any(Option::is_some) {self.pending_keys.clear();}
        Ok(())
    }
    fn apply(&mut self, g: &mut Game, e: &TriggerChoice) -> Result<(), String> {
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
        require((e.kind=="order_triggers")==e.order.is_some(),"/trigger order field")?;
        require((e.kind=="target_player")==e.player.is_some(),"/player target field")?;
        self.client
            .points
            .push(self.point(g, &format!("before/{}", e.sequence)));
        let before: Vec<_> = g
            .objects
            .in_zone(Zone::Battlefield)
            .map(|h| (h, self.reference(g, h)))
            .collect();
        if e.kind == "order_triggers" {
            let requested=e.order.as_ref().ok_or("first divergence: /missing trigger order")?;
            let mut choices=Vec::new();
            for key in requested {
                let row=self.pending_keys.iter().find(|(_,k)|k==key).ok_or("first divergence: /trigger source/event")?.0;
                choices.push(actions::Choice::OrderTrigger{trigger:row});
            }
            self.client.submit_many(g,actor,"trigger_order",choices)?;
        } else if e.kind == "target_player" {
            self.client.submit(g,actor,"trigger_target",actions::Choice::TargetPlayer{seat:e.player.ok_or("first divergence: /missing player target")?})?;
        } else if e.kind == "empty_attackers" {
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
                    "cast_discard",
                    actions::Choice::Discard {
                        card: self.object(g, e)?,
                    },
                ),
                "target" => {
                    let role = e.role.as_deref().unwrap();
                    require(
                        matches!(role, "growth_target" | "bite_source" | "bite_destination" | "pyromancer_target"),
                        "/target role",
                    )?;
                    (
                        if role=="pyromancer_target" {"trigger_target"} else {role},
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
        self.sync_triggers(g)?;
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
        let play: Vec<TriggerChoice> = serde_json::from_value(case["play"].clone())
            .map_err(|e| format!("first divergence: /play fields {e}"))?;
        let stop=case["stop"].as_str().and_then(|s|s.strip_prefix("triggers_settled/")).ok_or("first divergence: /stop")?;
        for e in &play {
            self.apply(g, e).map_err(|err| {
                format!(
                    "first divergence: /play/{} {}",
                    e.sequence,
                    err.trim_start_matches("first divergence: ")
                )
            })?;
            let done = self.event>0 && [Seat::P0,Seat::P1].into_iter().any(|s| g.objects.in_zone(Zone::Graveyard(s)).any(|h|self.client.id(g,h)==stop)) && g.turns.stack.is_empty()
                && g.turns.payment.is_none() && g.turns.targeting.is_none()
                && !g.turns.pending_triggers.iter().any(Option::is_some);
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
fn triggers_consume(
    g: &mut Game,
    doc: Value,
    rejected: &mut Value,
) -> Result<BTreeMap<String, Value>, String> {
    require(
        doc["schema_version"] == 6 && doc["family"] == "triggers",
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
                let mut client = TriggerClient {
                    client: PlayedClient::new(g, tape, case)?,
                    creation: 0,
                    creations: vec![],
                    departures: vec![],
                    cast_start: None,
                    event:0,witnesses:vec![],pending_keys:vec![],stack_keys:vec![],old_sources:vec![],
                };
                let error = client.run(g, case).err();
                Ok((client, error))
            },
        )?;
        let mut report = client.client.report();
        report["opening"] = opened;
        report["trigger_provenance"]=json!(client.witnesses);
        report["post_run_trigger_counts"] = json!({
            "pending":trial.turns.pending_triggers.iter().filter(|p|p.is_some()).count(),
            "stack":trial.turns.triggered.len()});
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
fn full_pool_triggers_literal_checkpoints() {
    let doc: Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-triggers.json")).unwrap();
    let mut g=Game::new().unwrap();
    let runs=triggers_consume(&mut g,doc.clone(),&mut Value::Null).expect("legal played trigger prefixes");
    let repeat=triggers_consume(&mut g,doc,&mut Value::Null).unwrap();
    for (name,run) in &runs {
        let mut a=run.clone();let mut b=repeat[name].clone();
        // Raw storage scopes are per Game, not cross-run semantic identities.
        a.as_object_mut().unwrap().remove("trigger_provenance");b.as_object_mut().unwrap().remove("trigger_provenance");
        for report in [&mut a, &mut b] {
            for point in report["points"].as_array_mut().unwrap() {
                for ability in point["stack"].as_array_mut().unwrap() {
                    if let Some(handle)=ability.get_mut("raw_source_handle") {
                        handle.as_object_mut().unwrap().remove("store");
                    }
                }
            }
        }
        assert_eq!(a,b,"actual deterministic scalar records and observations, excluding per-Game storage scope");
    }
    for (name,life) in [("archer-cyclops",19),("duplicate-archers",16),("pyromancer-bite",18)] {
        let last=runs[name]["points"].as_array().unwrap().last().unwrap();
        assert_eq!(last["life"],json!([20,life]),"pinned card damage, CR603/113.7a");
        assert_eq!(last["stack"],json!([]));assert_eq!(last["pending_triggers"],json!([]));
    }
    let negatives:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-triggers-native-negatives.json")).unwrap();
    let mut rejections=BTreeMap::new();let mut negative_runs=BTreeMap::new();
    for (name,control) in negatives.as_object().unwrap() {
        let before=format!("{g:?}");let mut rejected=Value::Null;
        let error=triggers_consume(&mut g,control["input"].clone(),&mut rejected).expect_err("invalid trigger tape accepted");
        assert_eq!(before,format!("{g:?}"),"rejected caller state/RNG");
        let expected=if name=="truncated_tape"||name=="extra_tape" {format!("first divergence: {}",control["category"].as_str().unwrap())}
            else {format!("first divergence: /play/{} {}",control["sequence"],control["category"].as_str().unwrap())};
        assert!(error.starts_with(&expected),"{name}: intended {expected}, got {error}");
        assert!(!rejected.is_null());rejections.insert(name.clone(),error);negative_runs.insert(name.clone(),rejected);
    }
    if std::env::var("MTG_FULL_POOL_FAMILY").as_deref()==Ok("triggers") {
        let path=std::env::var("MTG_FULL_POOL_OUTPUT").unwrap();
        std::fs::write(path,serde_json::to_vec_pretty(&json!({"checkpoints":runs.iter().map(|(id,v)|(id.clone(),v["points"].clone())).collect::<BTreeMap<_,_>>(),
            "runs":runs,"repeat_runs":repeat,"rejections":rejections,"negative_runs":negative_runs,"rejection_state_rng":"unchanged"})).unwrap()).unwrap();
    }
}
