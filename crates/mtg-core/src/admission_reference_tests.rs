// Version 10 strict test-only full games using the delivered scalar client.
struct AdmissionClient {
    extended_observation: bool,
    trigger: TriggerClient,
    activation_action: Option<usize>,
    allocations: Vec<Value>,
    cleanups: Vec<Value>,
    marked: Vec<Value>,
}
impl AdmissionClient {
    fn finish_damage(&mut self,g:&mut Game,actor:Seat)->Result<(),String> {
        // Delivered version-9 observer: snapshot-restored quantum execution
        // witnesses simultaneous marks before SBA, then must equal scalar play.
        let mut trial=Game::new().unwrap();trial.restore(&g.snapshot()).unwrap();
        let id=trial.turn_decision().unwrap().id;
        let mut progress=trial.finish_combat_quantum(actor,id,NonZeroUsize::MIN).map_err(|e|format!("first divergence: /damage quantum {e:?}"))?;
        let mut captured=false;
        while progress==Progress::InternalYield {
            if !captured && matches!(trial.work.front(),Some(Work::CombatDeaths(_)|Work::SpellMove{..}|Work::FinishCombat)) {
                self.marked.push(self.point(&trial,"simultaneous-before-sba"));captured=true;
            }
            progress=trial.resume(NonZeroUsize::MIN);
        }
        require(captured,"/missing simultaneous damage observation")?;
        self.submit(g,actor,"combat_damage",vec![actions::Choice::FinishCombat{}])?;
        require(self.point(g,"damage-quantum")==self.point(&trial,"damage-quantum")&&format!("{:?}",g.rng)==format!("{:?}",trial.rng),"/damage quantum scalar mismatch")?;
        Ok(())
    }
    fn point(&self,g:&Game,boundary:&str)->Value {
        let mut p=self.trigger.point(g,boundary);
        p["outcome"]=g.outcome().map(|o|json!({"winner":o.winner.map(seat_index),"losses":o.losses})).unwrap_or(Value::Null);
        for h in g.objects.in_zone(Zone::Battlefield) {
            let d=card_definitions::definition(g.objects.get(h).unwrap().card);
            let v=&mut p["permanents"][self.trigger.client.id(g,h)];
            for (key,value) in [("flying",d.flying()),("reach",d.reach()),("vigilance",d.vigilance()),("deathtouch",d.deathtouch()),("haste",g.has_haste(h)),("trample",g.has_trample(h))] {v[key]=json!(value);}
        }
        p["combat"]=json!(g.combat().iter().map(|a|json!({"attacker":self.trigger.reference(g,a.creature),"blocked":a.blocked,"blockers":a.blockers.iter().map(|h|self.trigger.reference(g,*h)).collect::<Vec<_>>()})).collect::<Vec<_>>());
        p["allocations"]=json!(self.allocations);
        p["activation"]=g.turns.activation.as_ref().map(|a|json!({"source":self.trigger.reference(g,a.source),"actor":seat_index(a.actor),"target":a.target.map(|h|self.trigger.reference(g,h)),"paid":a.paid,"reserved":a.reserved,"pool":g.activation_pool(a).unwrap(),"sources":a.sources.iter().map(|h|self.trigger.reference(g,*h)).collect::<Vec<_>>(),"ability":if a.power {"power"} else if a.invoker {"invoker"} else {"haste"}})).unwrap_or(Value::Null);
        if self.extended_observation {
            // CR400.7/611.2c: an old handle retained until cleanup is not an
            // active grant on a departed creature (nor on its new incarnation).
            let live = |h: &&Handle| g.objects.get(**h).is_ok_and(|o| o.zone == Zone::Battlefield);
            p["grants"]=json!(g.turns.haste.iter().filter(live).map(|h|json!({"object":self.trigger.reference(g,*h),"keyword":"haste"})).chain(g.turns.trample.iter().filter(live).map(|h|json!({"object":self.trigger.reference(g,*h),"keyword":"trample"}))).collect::<Vec<_>>());
        }
        p
    }
    fn object(&self,g:&Game,value:&Value)->Result<actions::ObjectRef,String> {
        let h=self.trigger.client.handle(g,value["source"].as_str().ok_or("first divergence: /source")?)?;
        let o=g.objects.get(h).unwrap();let (birth,_)=g.objects.semantic_identity(h).unwrap();
        Ok(actions::ObjectRef{birth,incarnation:value["incarnation"].as_u64().ok_or("first divergence: /incarnation")?,card:o.card.identity().key.into(),owner:o.owner,zone:o.zone})
    }
    fn submit(&mut self,g:&mut Game,actor:Seat,kind:&str,choices:Vec<actions::Choice>)->Result<(),String> {
        self.trigger.client.submit_many(g,actor,kind,choices)
    }
    fn observe_transition(&mut self,g:&Game,before:Vec<(Handle,Value)>,sequence:usize)->Result<(),String> {
        self.trigger.sync_triggers(g)?;
        let mut born:Vec<_>=g.objects.in_zone(Zone::Battlefield).filter(|h|!self.trigger.client.births.contains_key(&g.objects.semantic_identity(*h).unwrap().0)).collect();
        born.sort_by_key(|h|g.objects.semantic_identity(*h).unwrap().0);
        for (ordinal,h) in born.iter().enumerate() {
            let o=g.objects.get(*h).unwrap();let (birth,incarnation)=g.objects.semantic_identity(*h).unwrap();
            require(o.card.identity().key=="goblin-token","/unsupported created object")?;
            let id=format!("token/{}/{}/{ordinal}",seat_index(o.controller),self.trigger.creation);
            require(!self.trigger.client.births.values().any(|v|v==&id),"/duplicate token id")?;
            self.trigger.client.births.insert(birth,id.clone());
            self.trigger.creations.push(json!({"id":id,"birth":birth,"incarnation":incarnation,"action":sequence,"card":o.card.identity().key,"definition_hash":o.card.identity().content_sha256}));
        }
        if !born.is_empty(){self.trigger.creation+=1;}
        for (h,reference) in before {if g.objects.get(h).is_err(){self.trigger.departures.push(json!({"object":reference,"action":sequence,"old_handle_valid":false}));}}
        Ok(())
    }
    fn apply(&mut self,g:&mut Game,v:&Value)->Result<bool,String> {
        use actions::Choice as C;
        let kind=v["kind"].as_str().ok_or("first divergence: /kind")?;
        let mut fields:BTreeSet<&str>="sequence turn step actor kind source incarnation color role mode order player".split_whitespace().collect();
        if matches!(kind,"attackers"|"blockers"|"damage"|"graveyard_order"|"cleanup_discard"){fields.insert("selection");}
        if kind=="graveyard_order"{fields.insert("event");}
        let supplied:BTreeSet<&str>=v.as_object().ok_or("first divergence: /choice fields")?.keys().map(String::as_str).collect();
        require(supplied==fields,"/choice fields")?;
        let object=matches!(kind,"cast"|"play_land"|"tap_mana"|"target"|"discard"|"activate"|"activation_target");
        require(if object {v["source"].is_string()&&v["incarnation"].is_u64()} else {v["source"].is_null()&&v["incarnation"].is_null()},"/source/incarnation field")?;
        for (field,present) in [("color",matches!(kind,"pay"|"activation_pay")),("role",kind=="target"),("mode",kind=="mode"),("order",kind=="order_triggers"),("player",kind=="target_player")]{require(present!=v[field].is_null(),"/choice nullable field")?;}
        if matches!(kind,"pay"|"activation_pay"){require(v["color"].as_u64().is_some_and(|n|n<6),"/payment color")?;}
        if kind=="mode"{require(v["mode"].as_u64().is_some_and(|n|n<2),"/mode")?;}
        if kind=="target_player"{require(v["player"].as_u64().is_some_and(|n|n<2),"/target player")?;}
        let i=self.trigger.client.cursor;require(v["sequence"]==json!(i),"/sequence")?;
        let actor=match v["actor"].as_u64(){Some(0)=>Seat::P0,Some(1)=>Seat::P1,_=>return Err("first divergence: /actor".into())};
        if !matches!(kind,"damage") && g.turn_decision().is_some_and(|d|d.kind==turns::TurnKind::Combat(combat::CombatKind::Damage)) {
            let d=g.combat_decision(g.turn_decision().unwrap().actor,256).map_err(|e|format!("first divergence: /combat {e:?}"))?;
            // The decision lists all allocatable groups, including groups whose
            // explicit choices were already submitted. FinishCombat validates
            // completeness; it must not invent a missing allocation.
            let prior=g.objects.in_zone(Zone::Battlefield).map(|h|(h,self.trigger.reference(g,h))).collect();
            self.finish_damage(g,d.actor)?;
            self.observe_transition(g,prior,i)?;
        }
        let (turn,_,step)=g.turn_position().ok_or("first divergence: /missing turn")?;
        require(v["turn"]==json!(turn)&&v["step"]==priority_step(step),"/play position")?;
        if kind=="finish" {
            let outcome=g.outcome().ok_or("first divergence: /truncated replay")?;
            require(!outcome.losses.contains(&Some(terminal::LossReason::Concession)),"/concession quota")?;
            self.trigger.client.points.push(self.point(g,"finish"));
            self.trigger.client.consumed.push(v.clone());self.trigger.client.cursor+=1;return Ok(true);
        }
        require(g.outcome().is_none(),"/choice after terminal")?;
        self.trigger.client.points.push(self.point(g,&format!("before/{i}")));
        let before=g.objects.in_zone(Zone::Battlefield).map(|h|(h,self.trigger.reference(g,h))).collect();
        let snapshot=g.snapshot();
        let selection=||v["selection"].as_array().ok_or("first divergence: /selection".to_string());
        let payment=g.turns.payment.is_some();
        let activation_kind=if g.turns.activation.as_ref().is_some_and(|a|a.power||a.invoker){"activation_payment"}else{"activation_target"};
        match kind {
            "empty_attackers"|"attackers"=>{
                require(self.trigger.client.declared.insert(turn),"/extra declaration")?;
                if g.turn_decision().is_some_and(|d|d.kind==turns::TurnKind::Combat(combat::CombatKind::Attackers)) {
                    if kind=="attackers" {self.submit(g,actor,"attackers",vec![C::SelectAttackers{cards:selection()?.iter().map(|v|self.object(g,v)).collect::<Result<_,_>>()?}])?;}
                    self.submit(g,actor,"attackers",vec![C::FinishCombat{}])?;
                } else {require(kind=="empty_attackers","/missing attackers decision")?;}
            }
            "blockers"=>{
                let selected=selection()?;
                if !selected.is_empty(){self.submit(g,actor,"blockers",vec![C::SelectBlockers{blocks:selected.iter().map(|v|Ok((self.object(g,&v["blocker"])?,self.object(g,&v["attacker"])?))).collect::<Result<_,String>>()?}])?;}
                if g.turn_decision().is_some_and(|d|d.kind==turns::TurnKind::Combat(combat::CombatKind::Blockers)){self.submit(g,actor,"blockers",vec![C::FinishCombat{}])?;}
                else {require(selected.is_empty(),"/missing blockers decision")?;}
            }
            "damage"=>{
                require(selection()?.len()==1,"/damage group")?;let a=&selection()?[0];
                self.submit(g,actor,"combat_damage",vec![C::AssignDamage{attacker:self.object(g,&a["attacker"])?,amounts:a["amounts"].as_array().ok_or("first divergence: /damage amounts")?.iter().map(|v|Ok((self.object(g,&v["target"])?,v["amount"].as_u64().and_then(|n|u32::try_from(n).ok()).ok_or("first divergence: /damage amount")?))).collect::<Result<_,String>>()?}])?;
                self.allocations.push(a.clone());
            }
            "graveyard_order"|"cleanup_discard"=>{
                let cards=selection()?.iter().map(|v|self.object(g,v)).collect::<Result<Vec<_>,_>>()?;
                self.submit(g,actor,kind,cards.into_iter().map(|card|if kind=="graveyard_order"{C::OrderGraveyard{card}}else{C::Discard{card}}).collect())?;
            }
            "order_triggers"=>{
                let order=v["order"].as_array().ok_or("first divergence: /order")?;
                let choices=order.iter().map(|key|self.trigger.pending_keys.iter().find(|(_,k)|k==key).map(|(row,_)|C::OrderTrigger{trigger:*row}).ok_or("first divergence: /trigger source/event".to_string())).collect::<Result<_,_>>()?;
                self.submit(g,actor,"trigger_order",choices)?;
            }
            _=>{
                let (decision,choice)=match kind {
                    "pass"=>("priority",C::Pass{}),
                    "play_land"=>("priority",C::PlayLand{card:self.object(g,v)?}),
                    "cast"=>{self.trigger.client.actions.insert(v["source"].as_str().unwrap().into(),i);("priority",C::Cast{card:self.object(g,v)?})},
                    "tap_mana"=>(if g.turns.activation.is_some(){activation_kind}else if payment{"payment"}else{"priority"},C::TapMana{card:self.object(g,v)?}),
                    "pay"=>("payment",C::Pay{color:v["color"].as_u64().ok_or("first divergence: /color")? as u8}),
                    "finish_payment"=>("payment",C::FinishPayment{}),
                    "target"=>(v["role"].as_str().ok_or("first divergence: /role")?,C::Target{card:self.object(g,v)?}),
                    "finish_targets"=>("targets_complete",C::FinishTargets{}),
                    "mode"=>("cast_mode",C::Mode{mode:v["mode"].as_u64().ok_or("first divergence: /mode")? as u8}),
                    "discard"=>("cast_discard",C::Discard{card:self.object(g,v)?}),
                    "target_player"=>("trigger_target",C::TargetPlayer{seat:v["player"].as_u64().ok_or("first divergence: /player")? as u8}),
                    "activate"=>{self.activation_action=Some(i);("priority",C::Activate{card:self.object(g,v)?})},
                    "activation_target"=>("activation_target",C::Target{card:self.object(g,v)?}),
                    "activation_pay"=>("activation_payment",C::Pay{color:v["color"].as_u64().ok_or("first divergence: /color")? as u8}),
                    "finish_activation"=>(activation_kind,C::FinishActivation{}),
                    _=>return Err(format!("first divergence: /unsupported choice {kind}")),
                };self.submit(g,actor,decision,vec![choice])?;
            }
        }
        if kind=="finish_activation" {
            let h=*g.turns.stack.last().ok_or("first divergence: /missing ability")?;
            let action=self.activation_action.take().ok_or("first divergence: /missing activation")?;
            let id=format!("ability/{action}");self.trigger.client.births.insert(g.objects.semantic_identity(h).unwrap().0,id.clone());self.trigger.client.actions.insert(id,action);
        }
        self.observe_transition(g,before,i)?;
        // Observe cleanup at the actual quantum boundary, never reconstruct it
        // from the following upkeep or overwrite downstream engine state.
        if matches!(step,turns::Step::End|turns::Step::Cleanup)&&matches!(kind,"pass"|"cleanup_discard") {
            let mut trial=Game::new().unwrap();trial.restore(&snapshot).unwrap();
            let d=trial.turn_decision().unwrap();
            let selected=if kind=="pass"{turns::TurnSelection::Pass(d.candidate(0))}else{
                let available=trial.discard_cards().ok_or("first divergence: /discard decision")?;
                let cards=selection()?.iter().map(|v|{
                    let h=self.trigger.client.handle(&trial,v["source"].as_str().unwrap())?;
                    let index=available.iter().position(|c|*c==h).ok_or("first divergence: /discard source")?;Ok(d.candidate(index))
                }).collect::<Result<Vec<_>,String>>()?;turns::TurnSelection::Discard(cards)
            };
            let mut progress=trial.apply_turn_quantum(actor,&turns::TurnAction{decision:d.id,selection:selected},NonZeroUsize::MIN).map_err(|e|format!("first divergence: /cleanup quantum {e:?}"))?;
            while progress==Progress::InternalYield {
                if matches!(trial.work.front(),Some(Work::Turn(turns::TurnWork::CleanupEnd{..}))){self.cleanups.push(self.point(&trial,"cleanup-settled"));}
                progress=trial.resume(NonZeroUsize::MIN);
            }
            require(self.point(g,"quantum")==self.point(&trial,"quantum"),"/cleanup scalar mismatch")?;
        }
        self.trigger.client.consumed.push(v.clone());self.trigger.client.cursor+=1;Ok(false)
    }
}

fn admission_consume(case:&Value,pins:&Value,rejected:&mut Value)->Result<Value,String> {
    admission_consume_observed(case,pins,rejected,false)
}

fn admission_consume_observed(case:&Value,pins:&Value,rejected:&mut Value,extended_observation:bool)->Result<Value,String> {
    require(case["stop"]=="terminal","/nonterminal quota")?;
    let mut projection=json!({"schema_version":3,"family":"priority","pins":pins,"cases":[case]});
    projection["cases"][0].as_object_mut().unwrap().remove("native_records");projection["cases"][0].as_object_mut().unwrap().remove("native_indices");
    let (opening,reset)=priority_projection(&projection)?;
    let mut g=Game::new().unwrap();
    let (opened,client)=london_continue(&mut g,&reset.cases[0],&opening["cases"][0],|g,tape|{
        let mut client=AdmissionClient{extended_observation,trigger:TriggerClient{client:PlayedClient::new(g,tape,case)?,creation:0,creations:vec![],departures:vec![],cast_start:None,event:0,witnesses:vec![],pending_keys:vec![],stack_keys:vec![],old_sources:vec![]},activation_action:None,allocations:vec![],cleanups:vec![],marked:vec![]};
        let result=(||{
            let mut finished=false;
            for v in case["play"].as_array().ok_or("first divergence: /play")? {
                require(!finished,"/unused tape suffix")?;
                finished=client.apply(g,v).map_err(|e|format!("first divergence: /play/{} {e}",client.trigger.client.cursor))?;
            }
            require(finished,"/omitted final checkpoint")
        })();
        if let Err(error)=result{*rejected=client.trigger.client.report();return Err(error);}
        Ok(client)
    })?;
    let mut report=client.trigger.client.report();report["capture_status"]=json!("complete rules-terminal admission candidate; admission requires the independent verifier");
    report["marked"]=json!(client.marked);report["opening"]=opened;report["cleanup_checkpoints"]=json!(client.cleanups);report["trigger_provenance"]=json!(client.trigger.witnesses);
    Ok(report)
}

#[test]
fn full_pool_admission_literal_checkpoints() {
    let doc:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-admission.json")).unwrap();
    let oracle:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-admission-expectations.json")).unwrap();
    let mut index=BTreeMap::new();
    for case in doc["cases"].as_array().unwrap() {
        let id=case["id"].as_str().unwrap();let mut paths=vec![];
        for repeat in 1..=2 {
            let report=admission_consume(case,&doc["pins"],&mut Value::Null).unwrap();
            let final_point=report["points"].as_array().unwrap().last().unwrap();
            if id.ends_with("directed") {
                for key in ["life","outcome","turn","step","library"] {assert_eq!(final_point[key],oracle[id][key],"{id} {key}");}
                assert_eq!(report["cleanup_checkpoints"].as_array().unwrap().len(),67);
            } else {
                assert_eq!(report["records"],json!(&case["native_records"].as_array().unwrap()[2..]),"exact frozen heuristic choices");
                for seat in 0..2 {if final_point["outcome"]["losses"][seat]=="Life" {assert!(final_point["life"][seat].as_i64().unwrap()<=0,"CR704.5a");}}
            }
            if let Ok(folder)=std::env::var("MTG_ADMISSION_OUTPUT") {
                let path=std::path::Path::new(&folder);std::fs::create_dir_all(path).unwrap();let name=format!("{id}.native-{repeat}.json");
                serde_json::to_writer(std::io::BufWriter::new(std::fs::File::create(path.join(&name)).unwrap()),&report).unwrap();paths.push(name);
            }
        }
        index.insert(id.to_string(),paths);
    }
    let specs:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-admission-native-negatives.json")).unwrap();
    let mut negatives=BTreeMap::new();
    for (name,spec) in specs.as_object().unwrap() {
        let mut rejected=json!({"consumed_play":[],"points":[]});
        let pins=spec.get("pins").unwrap_or(&doc["pins"]);
        let error=admission_consume(&spec["case"],pins,&mut rejected).expect_err("invalid admission tape accepted");
        assert!(error.contains("first divergence:"),"{name}: {error}");
        if let Some(sequence)=spec["sequence"].as_u64(){assert_eq!(rejected["consumed_play"].as_array().unwrap().len(),sequence as usize,"{name}: {error}");}
        negatives.insert(name.clone(),json!({"error":error,"run":rejected}));
    }
    if let Ok(folder)=std::env::var("MTG_ADMISSION_OUTPUT") {
        std::fs::write(std::path::Path::new(&folder).join("native-index.json"),serde_json::to_vec(&index).unwrap()).unwrap();
        serde_json::to_writer(std::io::BufWriter::new(std::fs::File::create(std::path::Path::new(&folder).join("native-negatives.json")).unwrap()),&negatives).unwrap();
    }
}

#[test]
fn captured_xmage_ai_strict_replay() {
    let path=std::env::var("MTG_AI_REPLAY_INPUT").unwrap_or_else(|_|concat!(env!("CARGO_MANIFEST_DIR"),"/../../fixtures/reference/xmage-ai-normalized.json").into());
    let doc:Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    for case in doc["cases"].as_array().unwrap() {
        for repeat in 1..=2 {
            let mut rejected=Value::Null;
            let result=admission_consume_observed(case,&doc["pins"],&mut rejected,true);
            if let Ok(folder)=std::env::var("MTG_AI_REPLAY_OUTPUT") {
                let output=match &result {Ok(run)=>json!({"run":run}),Err(error)=>json!({"error":error,"run":rejected})};
                std::fs::write(std::path::Path::new(&folder).join(format!("native-{repeat}.json")),serde_json::to_vec(&output).unwrap()).unwrap();
            }
            let report=result.unwrap();
            assert_eq!(report["consumed_play"],case["play"]);
            assert_eq!(report["opening"]["consumed_chance"],case["chance"]);
            assert_eq!(report["opening"]["consumed_choices"],case["choices"]);
        }
    }
    let negative_path=std::env::var("MTG_AI_REPLAY_NEGATIVES").unwrap_or_else(|_|concat!(env!("CARGO_MANIFEST_DIR"),"/../../fixtures/reference/xmage-ai-negative.json").into());
    let specs:Value=serde_json::from_str(&std::fs::read_to_string(negative_path).unwrap()).unwrap();
    let mut negatives=json!({});
    for (name,spec) in specs.as_object().unwrap() {
        let mut rejected=Value::Null;
        let error=admission_consume_observed(&spec["case"],&doc["pins"],&mut rejected,true).unwrap_err();
        if let Some(i)=spec["sequence"].as_u64() {
            assert!(error.contains(&format!("/play/{i} ")),"{name}: {error}");
            assert_eq!(rejected["consumed_play"].as_array().unwrap().len(),i as usize,"{name}");
        }
        negatives[name]=json!({"error":error,"run":rejected});
    }
    if let Ok(folder)=std::env::var("MTG_AI_REPLAY_OUTPUT") {
        std::fs::write(std::path::Path::new(&folder).join("native-negatives.json"),serde_json::to_vec(&negatives).unwrap()).unwrap();
    }
}
