// Version 9 combat extension: normal reset, actual scalar actions, no state injection.
struct PlayedCombatClient {
    activation: ActivationClient,
    marked: Vec<Value>,
    allocations: Vec<Value>,
    legal_checks: Vec<Value>,
}
impl PlayedCombatClient {
    fn finish_damage(&mut self,g:&mut Game,actor:Seat)->Result<(),String> {
        let mut trial=Game::new().unwrap();trial.restore(&g.snapshot()).unwrap();
        let id=trial.turn_decision().unwrap().id;
        let mut progress=trial.finish_combat_quantum(actor,id,NonZeroUsize::MIN).map_err(|e|format!("first divergence: /quantum {e:?}"))?;
        let mut captured=false;
        while progress==Progress::InternalYield {
            if !captured && matches!(trial.work.front(),Some(Work::CombatDeaths(_)|Work::SpellMove{..}|Work::FinishCombat)) {
                let mut permanents=BTreeMap::new();
                for h in trial.objects.in_zone(Zone::Battlefield) {
                    if let Some(c)=trial.creature_state(h) {
                        permanents.insert(self.activation.client.id(&trial,h),json!({"power":c.power,"toughness":c.toughness,"damage":c.damage,"incarnation":trial.objects.semantic_identity(h).unwrap().1}));
                    }
                }
                self.marked.push(json!({"life":trial.life(),"permanents":permanents}));captured=true;
            }
            progress=trial.resume(NonZeroUsize::MIN);
        }
        require(captured,"/missing simultaneous damage observation")?;
        self.activation.client.submit(g,actor,"combat_damage",actions::Choice::FinishCombat {})?;
        require(self.point(g,"quantum")==self.point(&trial,"quantum") && format!("{:?}",g.rng)==format!("{:?}",trial.rng),"/quantum scalar mismatch")?;
        Ok(())
    }
    fn point(&self, g: &Game, boundary: &str) -> Value {
        let mut p = self.activation.point(g, boundary);
        for h in g.objects.in_zone(Zone::Battlefield) {
            let o=g.objects.get(h).unwrap();let d=card_definitions::definition(o.card);
            let v=&mut p["permanents"][self.activation.client.id(g,h)];
            v["flying"]=json!(d.flying());v["reach"]=json!(d.reach());v["vigilance"]=json!(d.vigilance());v["deathtouch"]=json!(d.deathtouch());
        }
        p["combat"] = json!(g.combat().iter().map(|a| json!({
            "attacker": self.activation.reference(g,a.creature), "blocked":a.blocked,
            "blockers":a.blockers.iter().map(|h|self.activation.reference(g,*h)).collect::<Vec<_>>()
        })).collect::<Vec<_>>());
        p["allocations"] = json!(self.allocations);
        p
    }
    fn object(&self, g: &Game, v: &Value) -> Result<actions::ObjectRef,String> {
        require(v.as_object().is_some_and(|o|o.len()==2),"/object fields")?;
        let e=PlayedChoice {sequence:0,turn:0,step:String::new(),actor:0,kind:String::new(),source:v["source"].as_str().map(str::to_owned),incarnation:v["incarnation"].as_u64(),color:None};
        self.activation.object(g,&e)
    }
    fn apply(&mut self,g:&mut Game,v:&Value)->Result<bool,String> {
        let mut projection=v.clone();let selection=projection.as_object_mut().unwrap().remove("selection");
        let event=projection.as_object_mut().unwrap().remove("event");
        let e:PlayedChoice=serde_json::from_value(projection).map_err(|e|format!("first divergence: /play fields {e}"))?;
        require(e.sequence==self.activation.client.cursor && e.actor<2,"/play actor/sequence")?;
        let actor=if e.actor==0 {Seat::P0}else{Seat::P1};
        let (turn,_,step)=g.turn_position().unwrap();
        require(turn==e.turn && priority_step(step)==e.step,"/play position")?;
        require(e.kind!="legacy_blocker_order","/legacy blocker-order callback")?;
        require(event.is_some()==(e.kind=="graveyard_order"),"/graveyard event field")?;
        let special=matches!(e.kind.as_str(),"attackers"|"blockers"|"damage"|"finish"|"graveyard_order");
        require(selection.is_some()==(special && e.kind!="finish"),"/selection field")?;
        if special {require(g.turn_decision().is_some_and(|d|d.actor==actor),"/wrong combat actor")?;}
        if !special {
            self.activation.apply(g,&e)?;
            let index=self.activation.client.points.len()-1;
            // Activation client observed the pre-action state; augment it from
            // the combat state, which ordinary priority/payment does not select.
            let p=&mut self.activation.client.points[index];
            p["combat"]=json!([]);p["allocations"]=json!([]);
            return Ok(false);
        }
        require(e.source.is_none()&&e.incarnation.is_none()&&e.color.is_none(),"/combat fields")?;
        self.activation.client.points.push(self.point(g,&format!("before/{}",e.sequence)));
        if e.kind=="graveyard_order" {
            require(event==Some(json!(format!("combat-sba/{turn}"))),"/graveyard event")?;
            require(g.turn_decision().is_some_and(|d|d.kind==turns::TurnKind::GraveyardOrder),"/unexpected graveyard choice")?;
            let cards=selection.as_ref().unwrap().as_array().ok_or("first divergence: /selection array")?;
            let choices=cards.iter().map(|v|Ok(actions::Choice::OrderGraveyard {card:self.object(g,v)?})).collect::<Result<Vec<_>,String>>()?;
            self.activation.client.submit_many(g,actor,"graveyard_order",choices)?;
        } else if e.kind=="finish" {
            require(step==turns::Step::CombatDamage && matches!(g.turn_decision().map(|d|d.kind),Some(turns::TurnKind::Priority)),"/finish before settled damage")?;
        } else {
            let selected=selection.as_ref().unwrap().as_array().ok_or("first divergence: /selection array")?;
            let (kind,choice)=match e.kind.as_str() {
                "attackers"=>{
                    require(self.activation.client.declared.insert(turn),"/extra declaration")?;
                    ("attackers",actions::Choice::SelectAttackers {cards:selected.iter().map(|v|self.object(g,v)).collect::<Result<_,_>>()?})
                },
                "blockers"=>("blockers",actions::Choice::SelectBlockers {blocks:selected.iter().map(|v|Ok((self.object(g,&v["blocker"])?,self.object(g,&v["attacker"])?))).collect::<Result<_,String>>()?}),
                "damage"=>{
                    require(selected.len()==1,"/damage cardinality")?;
                    let v=&selected[0];
                    let attack=self.object(g,&v["attacker"])?;
                    if let Some(a)=g.combat().iter().find(|a|self.activation.client.id(g,a.creature)==v["attacker"]["source"]) {
                        let ids:Vec<_>=a.blockers.iter().map(|h|self.activation.client.id(g,*h)).collect();
                        let chosen:Vec<_>=v["amounts"].as_array().ok_or("first divergence: /amounts")?.iter().map(|v|v["target"]["source"].as_str().unwrap_or("").to_string()).collect();
                        require(ids==chosen,"/reordered damage recipient")?;
                    }

                    ("combat_damage",actions::Choice::AssignDamage {attacker:attack,amounts:v["amounts"].as_array().ok_or("first divergence: /amounts")?.iter().map(|v|{
                        let n=v["amount"].as_u64().and_then(|n|u32::try_from(n).ok()).ok_or("first divergence: /negative or oversized damage")?;
                        Ok((self.object(g,&v["target"])?,n))
                    }).collect::<Result<_,String>>()?})
                },
                _=>unreachable!(),
            };
            let d=g.combat_decision(actor,256).map_err(|e|format!("first divergence: /combat decision {e:?}"))?;
            if kind=="attackers" {self.legal_checks.push(json!({"kind":kind,"sequence":e.sequence,"legal":d.attackers.iter().map(|h|self.activation.reference(g,*h)).collect::<Vec<_>>()}));}
            if kind=="blockers" {self.legal_checks.push(json!({"kind":kind,"sequence":e.sequence,"legal":d.blockers.iter().flat_map(|b|d.attackers.iter().filter(|a|!d.forbidden_blocks.contains(&(*b,**a))).map(|a|json!({"blocker":self.activation.reference(g,*b),"attacker":self.activation.reference(g,*a)}))).collect::<Vec<_>>()}));}
            self.activation.client.submit(g,actor,kind,choice)?;
            if kind=="combat_damage" {
                self.allocations=g.turns.combat.assignments.iter().map(|(h,amounts)|json!({"attacker":self.activation.reference(g,*h),"amounts":amounts.iter().map(|(b,n)|json!({"target":self.activation.reference(g,*b),"amount":n})).collect::<Vec<_>>()})).collect();
                self.finish_damage(g,actor)?;
            } else {self.activation.client.submit(g,actor,kind,actions::Choice::FinishCombat {})?;}
        }
        self.activation.client.consumed.push(v.clone());self.activation.client.cursor+=1;
        Ok(e.kind=="finish")
    }
    fn run(&mut self,g:&mut Game,c:&Value)->Result<(),String> {
        let play=c["play"].as_array().ok_or("first divergence: /play")?;
        for v in play {
            // Capture before ordinary actions as well, without reconstructing
            // combat state from the tape or assuming empty combat.
            require(v["sequence"]==json!(self.activation.client.cursor),"/play actor/sequence")?;
            if matches!(v["kind"].as_str(),Some("finish"|"graveyard_order")) && g.turn_decision().is_some_and(|d|d.kind==turns::TurnKind::Combat(combat::CombatKind::Damage)) {
                let d=g.combat_decision(g.turn_decision().unwrap().actor,256).unwrap();
                require(d.damage.is_empty(),"/missing damage allocation")?;
                self.finish_damage(g,d.actor)?;
            }
            let before=self.point(g,&format!("before/{}",self.activation.client.cursor));
            let done=self.apply(g,v).map_err(|e|format!("first divergence: /play/{} {}",self.activation.client.cursor,e.trim_start_matches("first divergence: ")))?;
            *self.activation.client.points.last_mut().unwrap()=before;
            if done {
                require(self.activation.client.cursor==play.len(),"/extra choice after named stop")?;
                return Ok(());
            }
        }
        Err("first divergence: /missing choice before named stop".into())
    }
}
fn combat_consume(g:&mut Game,doc:Value,rejected:&mut Value)->Result<BTreeMap<String,Value>,String> {
    require(doc["schema_version"]==9 && doc["family"]=="combat","/version/family")?;
    let mut projection=doc.clone();projection["schema_version"]=json!(3);projection["family"]=json!("priority");
    let (opening,reset)=priority_projection(&projection)?;
    let mut trial=Game::new().unwrap();let mut runs=BTreeMap::new();
    for (n,c) in doc["cases"].as_array().unwrap().iter().enumerate() {
        let (opened,(client,error))=london_continue(&mut trial,&reset.cases[n],&opening["cases"][n],|g,tape|{
            let mut client=PlayedCombatClient {activation:ActivationClient {client:PlayedClient::new(g,tape,c)?,activation_action:None,cast_start:None},marked:vec![],allocations:vec![],legal_checks:vec![]};
            let error=client.run(g,c).err();Ok((client,error))
        })?;
        let mut report=client.activation.client.report();report["opening"]=opened;report["marked"]=json!(client.marked);report["legal_checks"]=json!(client.legal_checks);
        if let Some(error)=error { *rejected=report;return Err(error); }
        runs.insert(c["id"].as_str().unwrap().to_string(),report);
    }
    *g=trial;Ok(runs)
}
#[test]
fn full_pool_combat_literal_checkpoints() {
    let doc:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-combat.json")).unwrap();
    let expected:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-combat-expectations.json")).unwrap();
    let mut g=Game::new().unwrap();
    let runs=combat_consume(&mut g,doc.clone(),&mut Value::Null).expect("legal played combat");
    let repeat=combat_consume(&mut g,doc,&mut Value::Null).unwrap();assert_eq!(runs,repeat);
    let points:BTreeMap<_,_>=runs.iter().map(|(id,r)|(id.clone(),&r["points"])).collect();
    for (name,rows) in &points {
        let last=rows.as_array().unwrap().last().unwrap();assert_eq!(last["step"],"combat_damage");
        assert_eq!(last["life"],expected[name]["life"],"{name}");
        let dead:Vec<_>=last["graveyard"].as_array().unwrap().iter().flat_map(|a|a.as_array().unwrap()).cloned().collect();
        let mut want=expected[name]["dead"].as_array().unwrap().clone();want.sort_by_key(|v|v.to_string());let mut dead=dead;dead.sort_by_key(|v|v.to_string());assert_eq!(dead,want,"{name}");
    }
    let negatives:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-combat-negatives.json")).unwrap();
    let mut rejections=BTreeMap::new();let mut negative_runs=BTreeMap::new();
    for (name,spec) in negatives.as_object().unwrap() {
        let before=g.snapshot();let mut rejected=Value::Null;
        let error=combat_consume(&mut g,spec["input"].clone(),&mut rejected).expect_err("invalid combat tape accepted");
        let category=spec["native"].as_str().unwrap();
        assert!(error.ends_with(category),"{name}: expected {category}, got {error}");
        assert_eq!(before,g.snapshot());assert_eq!(rejected["consumed_play"].as_array().unwrap().len(),spec["sequence"].as_u64().unwrap() as usize,"{name}");
        rejections.insert(name.clone(),error);negative_runs.insert(name.clone(),rejected);
    }
    if std::env::var("MTG_FULL_POOL_FAMILY").as_deref()==Ok("combat") {
        // Preserve the complete envelope without cloning every report into a
        // second Value and then allocating a whole-output byte buffer. Normal
        // discovery shares the container with earlier clients' retained data.
        #[derive(serde::Serialize)]
        struct Export<'a> {
            checkpoints: &'a BTreeMap<String, &'a Value>,
            negative_runs: &'a BTreeMap<String, Value>,
            rejections: &'a BTreeMap<String, String>,
            repeat_runs: &'a BTreeMap<String, Value>,
            runs: &'a BTreeMap<String, Value>,
        }
        let mut output=std::io::BufWriter::with_capacity(128*1024,
            std::fs::File::create(std::env::var("MTG_FULL_POOL_OUTPUT").unwrap()).unwrap());
        serde_json::to_writer(&mut output,&Export {checkpoints:&points,negative_runs:&negative_runs,
            rejections:&rejections,repeat_runs:&repeat,runs:&runs}).unwrap();
        std::io::Write::flush(&mut output).unwrap();
    }
}
