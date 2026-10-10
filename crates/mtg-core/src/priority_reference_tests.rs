// Test-only version-3 client. All mutations go through delivered semantic actions.
use crate::trajectory::{self, v2};

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PlayedChoice {
    sequence: usize,
    turn: u64,
    step: String,
    actor: u8,
    kind: String,
    source: Option<String>,
    incarnation: Option<u64>,
    color: Option<u8>,
}
fn priority_fixture() -> Value {
    serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-priority.json"
    ))
    .unwrap()
}
fn priority_step(step: turns::Step) -> &'static str {
    use turns::Step;
    match step {
        Step::Upkeep => "upkeep",
        Step::Draw => "draw",
        Step::PrecombatMain => "precombat_main",
        Step::BeginningCombat => "begin_combat",
        Step::DeclareAttackers => "declare_attackers",
        Step::DeclareBlockers => "declare_blockers",
        Step::CombatDamage => "combat_damage",
        Step::EndCombat => "end_combat",
        Step::PostcombatMain => "postcombat_main",
        Step::End => "end_turn",
        Step::Cleanup => "cleanup",
    }
}
#[derive(Clone)]
struct PlayedClient {
    births: BTreeMap<u64, String>,
    actions: BTreeMap<String, usize>,
    cursor: usize,
    points: Vec<Value>,
    records: Vec<Value>,
    consumed: Vec<Value>,
    declared: BTreeSet<u64>,
    recorder: v2::Recorder,
    logical: u64,
    micro: u64,
    stale_rejections: usize,
    stops: Vec<Value>,
    continuation_rejections: usize,
}
impl PlayedClient {
    fn new(g: &Game, tape: &Tape<'_>, case: &Value) -> Result<Self, String> {
        let hash = |v: &Value| format!("{:x}", Sha256::digest(serde_json::to_vec(v).unwrap()));
        let header = trajectory::Header {
            id: trajectory::EpisodeKey {
                run: "27100000-0000-4000-8000-000000000001".into(),
                ordinal: 0,
            },
            versions: trajectory::Versions {
                schema: 2,
                engine: snapshot::engine().into(),
                rules: "cr-2026-09-25".into(),
                cards: "foundations_micro_v1".into(),
                action: "policy-v1".into(),
                observation: 1,
            },
            deck_hashes: [hash(&case["decks"][0]), hash(&case["decks"][1])],
            config_hash: hash(&json!({"starting_seat":case["starter"],"decks":case["decks"]})),
            policies: [
                "strict-priority-tape-v3".into(),
                "strict-priority-tape-v3".into(),
            ],
            starting_seat: case["starter"].as_u64().unwrap() as u8,
            limits: trajectory::Limits::default(),
            restricted_replay: Some("privileged-priority-input".into()),
        };
        let frame =
            v2::Frame::capture(g, 256).map_err(|e| format!("first divergence: /capture {e:?}"))?;
        Ok(Self {
            births: tape.births.clone(),
            actions: BTreeMap::new(),
            cursor: 0,
            points: vec![],
            records: vec![],
            consumed: vec![],
            declared: BTreeSet::new(),
            recorder: v2::Recorder::new(&header, &frame)
                .map_err(|e| format!("first divergence: /recorder {e:?}"))?,
            logical: 0,
            micro: 0,
            stale_rejections: 0,
            stops: vec![],
            continuation_rejections: 0,
        })
    }
    fn id(&self, g: &Game, h: Handle) -> String {
        self.births[&g.objects.semantic_identity(h).unwrap().0].clone()
    }
    fn handle(&self, g: &Game, id: &str) -> Result<Handle, String> {
        Zone::ALL
            .into_iter()
            .flat_map(|z| g.objects.in_zone(z))
            .find(|h| self.id(g, *h) == id)
            .ok_or_else(|| format!("first divergence: /source unknown {id}"))
    }
    fn point(&self, g: &Game, boundary: &str) -> Value {
        let (turn, active, step) = g.turn_position().unwrap();
        let actor = g
            .turns
            .payment
            .as_ref()
            .map(|p| p.actor())
            .or_else(|| g.turn_decision().map(|d| d.actor))
            .expect("live decision");
        let ordered = |zone| {
            g.objects
                .in_zone(zone)
                .map(|h| self.id(g, h))
                .collect::<Vec<_>>()
        };
        let mut incarnations = BTreeMap::new();
        let mut permanents = BTreeMap::new();
        for h in Zone::ALL.into_iter().flat_map(|z| g.objects.in_zone(z)) {
            let id = self.id(g, h);
            let o = g.objects.get(h).unwrap();
            assert_eq!(o.card.identity().key, id.split('/').nth(1).unwrap());
            assert_eq!(
                seat_index(o.owner).to_string(),
                id.split('/').next().unwrap()
            );
            assert_eq!(o.owner, o.controller, "unsupported controller change");
            incarnations.insert(id.clone(), g.objects.semantic_identity(h).unwrap().1);
            if o.zone == Zone::Battlefield {
                let c = g.creature_state(h);
                permanents.insert(
                    id,
                    json!({"tapped":o.tapped,"sick":g.summoning_sick(h),
                    "power":c.map(|v|v.power),"toughness":c.map(|v|v.toughness)}),
                );
            }
        }
        assert_eq!(
            incarnations.len(),
            80,
            "unexpected/missing physical occurrence"
        );
        let stack: Vec<_> = g.turns.stack.iter().map(|h| {
            let id = self.id(g,*h);
            json!({"source":id,"incarnation":g.objects.semantic_identity(*h).unwrap().1,"action":self.actions[&id]})
        }).collect();
        let payment = g.turns.payment.as_ref().map(|p| {
            let cast = g.turns.casting.as_ref().expect("only creature casting admitted");
            let id = self.id(g,cast.card());
            json!({"actor":seat_index(p.actor()),"source":id,
                "incarnation":g.objects.semantic_identity(cast.card()).unwrap().1,"action":self.actions[&id],
                "pool":p.pool(),"colored":p.remaining().colored,"generic":p.remaining().generic,
                "sources":cast.sources().iter().map(|h|self.id(g,*h)).collect::<Vec<_>>()})
        });
        json!({"boundary":boundary,"turn":turn,"step":priority_step(step),"active":seat_index(active),
            "actor":seat_index(actor),"life":g.life(),"mana":g.mana(),"land_plays":u8::from(g.turns.land_used),
            "hand":[ordered(Zone::Hand(Seat::P0)),ordered(Zone::Hand(Seat::P1))],
            "library":[ordered(Zone::Library(Seat::P0)),ordered(Zone::Library(Seat::P1))],
            "graveyard":[ordered(Zone::Graveyard(Seat::P0)),ordered(Zone::Graveyard(Seat::P1))],
            "exile":ordered(Zone::Exile),"battlefield":ordered(Zone::Battlefield),"stack":stack,
            "incarnations":incarnations,"permanents":permanents,"payment":payment})
    }
    fn submit(
        &mut self,
        g: &mut Game,
        actor: Seat,
        kind: &str,
        choice: actions::Choice,
    ) -> Result<(), String> {
        let record = actions::Record {
            version: actions::ACTION_VERSION,
            actor,
            decision: kind.into(),
            choices: vec![choice],
        };
        let bytes = serde_json::to_vec(&record).unwrap();
        let before_state = format!("{g:?}");
        let decoded = actions::decode(g, &bytes, 256);
        require(
            format!("{g:?}") == before_state,
            "/decode changed caller state/RNG",
        )?;
        let actions::Decoded::Decision { submission, .. } =
            decoded.map_err(|e| format!("first divergence: /native command {e:?}"))?
        else {
            return Err("first divergence: /unsupported concession".into());
        };
        require(
            actions::encode(g, actor, &submission, 256).unwrap() == bytes,
            "/semantic recording roundtrip",
        )?;
        let before = v2::Frame::capture(g, 256).unwrap();
        if let Err(e) = actions::apply(g, &bytes, 256) {
            require(
                format!("{g:?}") == before_state,
                "/rejected command changed state/RNG",
            )?;
            return Err(format!("first divergence: /native command {e:?}"));
        }
        let after = v2::Frame::capture(g, 256).unwrap();
        let status = after.status_after(&submission);
        self.recorder
            .append(
                &before,
                &v2::Choice {
                    submission: submission.clone(),
                    logical_action: self.logical,
                    micro_choice: self.micro,
                    status: status.clone(),
                    policy: trajectory::PolicyInfo::default(),
                },
                &after,
            )
            .map_err(|e| format!("first divergence: /recording {e:?}"))?;
        if status == v2::ActionStatus::Continuing {
            self.micro += 1;
        } else {
            self.logical += 1;
            self.micro = 0;
        }
        self.records.push(serde_json::to_value(record).unwrap());
        // The exact previously accepted candidate is stale, including when the
        // same actor retains priority. This is a real production submission.
        let stable = format!("{g:?}");
        require(
            g.apply_policy(actor, &submission, 256).is_err(),
            "/stale candidate accepted",
        )?;
        require(
            format!("{g:?}") == stable,
            "/stale candidate changed state/RNG",
        )?;
        self.stale_rejections += 1;
        Ok(())
    }
    fn apply(&mut self, g: &mut Game, e: &PlayedChoice) -> Result<(), String> {
        require(
            e.actor < 2 && e.sequence == self.cursor,
            "/play actor/sequence",
        )?;
        let actor = if e.actor == 0 { Seat::P0 } else { Seat::P1 };
        let (turn, active, step) = g.turn_position().ok_or("first divergence: /missing turn")?;
        require(turn == e.turn, "/play turn")?;
        require(priority_step(step) == e.step, "/play step")?;
        require(e.kind == "pay" || e.color.is_none(), "/unexpected color")?;
        if step == turns::Step::DeclareAttackers && !self.declared.contains(&turn) {
            require(e.kind == "empty_attackers", "/missing empty declaration")?;
        }
        self.points
            .push(self.point(g, &format!("before/{}", self.cursor)));
        if e.kind == "empty_attackers" {
            require(
                e.source.is_none() && e.incarnation.is_none(),
                "/declaration source",
            )?;
            require(
                step == turns::Step::DeclareAttackers
                    && actor == active
                    && self.declared.insert(turn),
                "/extra declaration",
            )?;
            if g.turn_decision().is_some_and(|d| {
                matches!(
                    d.kind,
                    turns::TurnKind::Combat(combat::CombatKind::Attackers)
                )
            }) {
                self.submit(
                    g,
                    actor,
                    "attackers",
                    actions::Choice::SelectAttackers { cards: vec![] },
                )?;
                self.submit(g, actor, "attackers", actions::Choice::FinishCombat {})?;
            } else {
                require(
                    !g.has_combat_creature(active) && g.combat().is_empty(),
                    "/unsupported declaration callback",
                )?;
            }
        } else {
            let mut source = None;
            if matches!(e.kind.as_str(), "play_land" | "cast" | "tap_mana") {
                let id = e
                    .source
                    .as_deref()
                    .ok_or("first divergence: /missing source")?;
                let h = self.handle(g, id)?;
                let o = g.objects.get(h).unwrap();
                if e.kind == "tap_mana" {
                    require(
                        mana::basic_color(o.card).is_some(),
                        "/unsupported mana callback",
                    )?;
                }
                if e.kind == "cast" {
                    require(
                        card_definitions::definition(o.card)
                            .creature_base()
                            .is_some(),
                        "/unsupported noncreature spell",
                    )?;
                    self.actions.insert(id.to_string(), e.sequence);
                }
                source = Some(actions::ObjectRef {
                    birth: g.objects.semantic_identity(h).unwrap().0,
                    incarnation: e
                        .incarnation
                        .ok_or("first divergence: /missing incarnation")?,
                    card: o.card.identity().key.into(),
                    owner: o.owner,
                    zone: o.zone,
                });
            } else {
                require(
                    e.source.is_none() && e.incarnation.is_none(),
                    "/unexpected source",
                )?;
            }
            let payment = g.turns.payment.is_some();
            let (kind, choice) = match e.kind.as_str() {
                "pass" => ("priority", actions::Choice::Pass {}),
                "play_land" => (
                    "priority",
                    actions::Choice::PlayLand {
                        card: source.unwrap(),
                    },
                ),
                "cast" => (
                    "priority",
                    actions::Choice::Cast {
                        card: source.unwrap(),
                    },
                ),
                "tap_mana" => (
                    if payment { "payment" } else { "priority" },
                    actions::Choice::TapMana {
                        card: source.unwrap(),
                    },
                ),
                "pay" => (
                    "payment",
                    actions::Choice::Pay {
                        color: e.color.ok_or("first divergence: /missing payment color")?,
                    },
                ),
                "finish_payment" => ("payment", actions::Choice::FinishPayment {}),
                _ => return Err("first divergence: /unsupported callback".into()),
            };
            self.submit(g, actor, kind, choice)?;
        }
        self.consumed.push(serde_json::to_value(e).unwrap());
        self.cursor += 1;
        Ok(())
    }
    fn run(&mut self, g: &mut Game, case: &Value) -> Result<(), String> {
        let play: Vec<PlayedChoice> = serde_json::from_value(case["play"].clone())
            .map_err(|e| format!("first divergence: /play fields {e}"))?;
        let stop = case["stop"].as_str().ok_or("first divergence: /stop")?;
        require(
            matches!(stop, "first_cast_committed" | "second_creature_resolved"),
            "/unsupported stop",
        )?;
        while self.cursor < play.len() {
            self.apply(g, &play[self.cursor]).map_err(|error| {
                format!("first divergence: /play/{} {}", self.cursor,
                    error.strip_prefix("first divergence: ").unwrap_or(&error))
            })?;
            if stop == "first_cast_committed"
                && !g.turns.stack.is_empty()
                && g.turns.payment.is_none()
            {
                self.stops
                    .push(json!({"next_sequence":self.cursor,"checkpoint":self.point(g,stop)}));
                return Ok(());
            }
            if g.turn_position().is_some_and(|(t, _, _)| t == 6)
                && g.turns.stack.is_empty()
                && self.actions.len() == 4
                && g.turns.payment.is_none()
                && g.objects
                    .in_zone(Zone::Battlefield)
                    .filter(|h| g.creature_state(*h).is_some())
                    .count()
                    == 4
            {
                require(self.cursor == play.len(), &format!("/play/{} /extra choice after named stop", self.cursor))?;
                self.points.push(self.point(g, stop));
                return Ok(());
            }
        }
        Err("first divergence: /missing choice before named stop".into())
    }
    fn report(&self) -> Value {
        json!({"points":self.points,"consumed_play":self.consumed,"records":self.records,
            "policy_capture":self.recorder.episode(),"capture_status":"open played prefix; no terminal/full-episode claim",
            "stale_candidates_rejected":self.stale_rejections,"stops":self.stops,
            "continuation_rejections":self.continuation_rejections})
    }
}
fn priority_projection(doc: &Value) -> Result<(Value, Envelope), String> {
    require(
        doc["schema_version"] == json!(3) && doc["family"] == "priority",
        "/version/family",
    )?;
    let mut opening = doc.clone();
    opening["schema_version"] = json!(2);
    opening["family"] = json!("mulligan");
    for case in opening["cases"]
        .as_array_mut()
        .ok_or("first divergence: /cases")?
    {
        case.as_object_mut()
            .ok_or("first divergence: /case")?
            .remove("play")
            .ok_or("first divergence: /missing play")?;
        case["stop"] = json!("first_upkeep");
    }
    let reset = london_reset(&opening)?;
    Ok((opening, reset))
}
fn priority_consume(
    g: &mut Game,
    doc: Value,
    split: bool,
) -> Result<BTreeMap<String, Value>, String> {
    let (opening, reset) = priority_projection(&doc)?;
    let mut trial = Game::new().unwrap();
    let mut runs = BTreeMap::new();
    for (n, case) in doc["cases"].as_array().unwrap().iter().enumerate() {
        let (opening_run, mut client) = london_continue(
            &mut trial,
            &reset.cases[n],
            &opening["cases"][n],
            |g, tape| {
                let mut client = PlayedClient::new(g, tape, case)?;
                let mut bounded = case.clone();
                if split {
                    bounded["stop"] = json!("first_cast_committed");
                }
                client.run(g, &bounded)?;
                Ok(client)
            },
        )?;
        if split {
            require(
                client.cursor > 0 && client.cursor < case["play"].as_array().unwrap().len(),
                "/bounded prefix did not stop",
            )?;
            for missing in [false, true] {
                let mut bad = case.clone();
                if missing {
                    bad["play"].as_array_mut().unwrap().remove(client.cursor);
                    for (n, e) in bad["play"].as_array_mut().unwrap().iter_mut().enumerate() {
                        e["sequence"] = json!(n);
                    }
                } else {
                    let actor = bad["play"][client.cursor]["actor"].as_u64().unwrap();
                    bad["play"][client.cursor]["actor"] = json!(1 - actor);
                }
                let state = format!("{trial:?}");
                require(
                    client.clone().run(&mut trial, &bad).is_err(),
                    "/invalid continuation accepted",
                )?;
                require(
                    format!("{trial:?}") == state,
                    "/invalid first continuation command changed state",
                )?;
                client.continuation_rejections += 1;
            }
            client.run(&mut trial, case)?;
        }
        let mut report = client.report();
        report["opening"] = opening_run;
        runs.insert(reset.cases[n].id.clone(), report);
    }
    *g = trial;
    Ok(runs)
}
fn priority_difference(wanted: &Value, actual: &Value, path: &str) -> Option<String> {
    match (wanted, actual) {
        (Value::Object(a), Value::Object(b)) => {
            for key in a.keys().chain(b.keys()).collect::<BTreeSet<_>>() {
                let p = format!("{path}/{key}");
                match (a.get(key), b.get(key)) {
                    (Some(x), Some(y)) => {
                        if let Some(d) = priority_difference(x, y, &p) {
                            return Some(d);
                        }
                    }
                    _ => return Some(format!("{p}: missing/extra field")),
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                if let Some(d) = priority_difference(x, y, &format!("{path}/{i}")) {
                    return Some(d);
                }
            }
            if a.len() != b.len() {
                return Some(format!("{path}: lengths {} != {}", a.len(), b.len()));
            }
        }
        _ if wanted != actual => {
            return Some(format!("{path}: expected {wanted}, actual {actual}"));
        }
        _ => {}
    }
    None
}
#[test]
fn full_pool_priority_literal_checkpoints() {
    let mut g = Game::new().unwrap();
    let fixture = priority_fixture();
    let runs = priority_consume(&mut g, fixture.clone(), false).unwrap();
    let points: BTreeMap<_, _> = runs
        .iter()
        .map(|(id, r)| (id.clone(), r["points"].clone()))
        .collect();
    let expected: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-priority-native.json"
    ))
    .unwrap();
    let actual = serde_json::to_value(&points).unwrap();
    if let Ok(p) = std::env::var("MTG_FULL_POOL_OUTPUT")
        && std::env::var("MTG_FULL_POOL_FAMILY").as_deref() == Ok("priority")
    {
        std::fs::write(
            p,
            serde_json::to_vec(&json!({"checkpoints":points,"runs":runs})).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(
        priority_difference(&expected, &actual, "$"),
        None,
        "independent CR priority oracle"
    );
    let repeated = priority_consume(&mut g, fixture.clone(), false).unwrap();
    let continued = priority_consume(&mut g, fixture.clone(), true).unwrap();
    for (id, run) in &runs {
        for field in ["points", "consumed_play", "records"] {
            assert_eq!(run[field], repeated[id][field], "repeat {id}/{field}");
            assert_eq!(
                run[field], continued[id][field],
                "strict continuation {id}/{field}"
            );
        }
        assert!(run["stale_candidates_rejected"].as_u64().unwrap() > 0);
        assert_eq!(continued[id]["continuation_rejections"], 2);
        let stop = &continued[id]["stops"][0];
        let index = stop["next_sequence"].as_u64().unwrap() as usize;
        let mut wanted = expected[id][index].clone();
        wanted["boundary"] = json!("first_cast_committed");
        assert_eq!(
            priority_difference(&wanted, &stop["checkpoint"], "$/stop"),
            None
        );
    }
    let negative_text = if std::env::var("MTG_FULL_POOL_FAMILY").as_deref() == Ok("priority") {
        std::env::var("MTG_FULL_POOL_NEGATIVES")
            .ok()
            .map(|p| std::fs::read_to_string(p).unwrap())
    } else {
        None
    };
    let negatives: Value = serde_json::from_str(negative_text.as_deref().unwrap_or(include_str!(
        "../../../fixtures/reference/full-pool-priority-negative-inputs.json"
    )))
    .unwrap();
    let mut rejections = BTreeMap::new();
    let expected_rejections: Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/full-pool-priority-negative-expectations.json"
    )).unwrap();
    assert_eq!(negatives.as_object().unwrap().len(), expected_rejections.as_object().unwrap().len());
    for (name, bad) in negatives.as_object().unwrap() {
        let before = format!("{g:?}");
        let error = priority_consume(&mut g, bad.clone(), false).unwrap_err();
        assert_eq!(format!("{g:?}"), before, "rejection state/RNG: {name}");
        assert!(error.starts_with(expected_rejections[name]["native"].as_str().unwrap()),
            "intended rejection boundary: {name}: {error}");
        rejections.insert(name, error);
    }
    assert_eq!(runs.len(), 6);
    assert_eq!(rejections.len(), 20);
    if std::env::var("MTG_FULL_POOL_FAMILY").as_deref() == Ok("priority")
        && let Ok(p) = std::env::var("MTG_FULL_POOL_OUTPUT")
    {
        std::fs::write(
            p,
            serde_json::to_vec(&json!({"checkpoints":points,"runs":runs,
            "repeat_runs":repeated,"continued_runs":continued,"rejections":rejections,
            "rejection_state_rng":"unchanged"}))
            .unwrap(),
        )
        .unwrap();
    }
}
