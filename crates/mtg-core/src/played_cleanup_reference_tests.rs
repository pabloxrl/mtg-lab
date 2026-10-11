// Version 7 test-only consumer. Every action uses the delivered scalar API.
fn cleanup_point(client: &SpellClient, g: &Game, boundary: &str) -> Value {
    let mut p = client.point(g, boundary);
    p["outcome"] = g.outcome().map(|o| json!({"winner":o.winner.map(seat_index),"losses":o.losses})).unwrap_or(Value::Null);
    p
}
// Observe the same turn action on a snapshot-restored engine at quantum 1.
// No test state is injected. The final snapshot must equal the scalar action.
fn cleanup_probe(client:&SpellClient,g:&Game,e:&SpellChoice)->Result<Option<(Game,Vec<Value>)>,String>{
    let (_,_,step)=g.turn_position().unwrap();
    if !matches!(step,turns::Step::End|turns::Step::Cleanup) || !matches!(e.kind.as_str(),"pass"|"discard") {return Ok(None);}
    let mut trial=Game::new().unwrap();trial.restore(&g.snapshot()).unwrap();
    let Some(d)=trial.turn_decision() else{return Ok(None)};
    let selection=if e.kind=="discard" {
        let h=client.client.handle(&trial,e.source.as_deref().unwrap())?;
        let cards=trial.discard_cards().ok_or("first divergence: /missing discard decision")?;
        let index=cards.iter().position(|c|*c==h).ok_or("first divergence: /discard source")?;
        turns::TurnSelection::Discard(vec![d.candidate(index)])
    }else{turns::TurnSelection::Pass(d.candidate(0))};
    let mut progress=trial.apply_turn_quantum(d.actor,&turns::TurnAction{decision:d.id,selection},NonZeroUsize::MIN).map_err(|e|format!("first divergence: /quantum {e:?}"))?;
    let mut points=vec![];
    while progress==Progress::InternalYield {
        if matches!(trial.work.front(),Some(Work::Turn(turns::TurnWork::CleanupEnd{..}))) {
            points.push(cleanup_point(client,&trial,"cleanup-settled"));
        }
        progress=trial.resume(NonZeroUsize::MIN);
    }
    Ok(Some((trial,points)))
}
fn cleanup_run(client: &mut SpellClient, g: &mut Game, case: &Value) -> Result<(), String> {
    require(matches!(case["stop"].as_str(),Some("terminal"|"prefix"|"concession")),"/completion kind")?;
    let play = case["play"].as_array().ok_or("first divergence: /play")?;
    for raw in play {
        let i=client.client.cursor;
        require(raw["sequence"]==json!(i), "/sequence")?;
        if raw["kind"] == "finish" {
            require(raw.as_object().is_some_and(|o|o.len()==10),"/finish fields")?;
            let _:SpellChoice=serde_json::from_value(raw.clone()).map_err(|e|format!("first divergence: /finish fields {e}"))?;
            require(["source","incarnation","color","role","mode"].iter().all(|k|raw[k].is_null()),"/finish unexpected choice")?;
            require(i+1 == play.len(), "/unused tape suffix")?;
            let (turn,_,step)=g.turn_position().unwrap();
            require(raw["turn"]==json!(turn) && raw["step"]==priority_step(step), "/final checkpoint position")?;
            require((case["stop"]=="terminal" || case["stop"]=="concession") == g.outcome().is_some(), "/premature terminal")?;
            if let Some(outcome)=g.outcome() {
                let concession=outcome.losses.contains(&Some(terminal::LossReason::Concession));
                require((case["stop"]=="concession")==concession,"/concession completion classification")?;
            }
            client.client.points.push(cleanup_point(client,g,"finish"));
            client.client.consumed.push(raw.clone());client.client.cursor+=1;
            return Ok(());
        }
        require(g.outcome().is_none(), "/choice after terminal")?;
        if raw["kind"]=="concede" {
            let (turn,_,step)=g.turn_position().unwrap();
            require(raw["turn"]==turn && raw["step"]==priority_step(step),"/concession boundary")?;
            let actor=match raw["actor"].as_u64(){Some(0)=>Seat::P0,Some(1)=>Seat::P1,_=>return Err("first divergence: /actor".into())};
            client.client.points.push(cleanup_point(client,g,&format!("before/{i}")));
            let bytes=actions::encode_concession(g,actor).map_err(|e|format!("first divergence: /concession {e:?}"))?;
            actions::apply(g,&bytes,256).map_err(|e|format!("first divergence: /concession {e:?}"))?;
            client.client.recorder.finish(&v2::Frame::capture(g,256).unwrap(),trajectory::End::Completed).unwrap();
            client.client.records.push(serde_json::from_slice(&bytes).unwrap());
            client.client.consumed.push(raw.clone());client.client.cursor+=1;continue;
        }
        if raw["kind"]=="cleanup_discard" {
            require(raw.as_object().unwrap().len()==6,"/cleanup fields")?;
            let (turn,_,step)=g.turn_position().unwrap();
            require(raw["turn"]==turn && raw["step"]==priority_step(step),"/cleanup position")?;
            let actor=match raw["actor"].as_u64(){Some(0)=>Seat::P0,Some(1)=>Seat::P1,_=>return Err("first divergence: /actor".into())};
            let selection=raw["selection"].as_array().ok_or("first divergence: /selection")?;
            let mut choices=vec![];
            for selected in selection {
                require(selected.as_object().is_some_and(|v|v.len()==2),"/selected fields")?;
                let e=SpellChoice{sequence:i,turn,step:priority_step(step).into(),actor:seat_index(actor) as u8,kind:"discard".into(),source:selected["source"].as_str().map(str::to_owned),incarnation:selected["incarnation"].as_u64(),color:None,role:None,mode:None};
                choices.push(actions::Choice::Discard{card:client.object(g,&e)?});
            }
            let before=g.snapshot();
            client.client.points.push(cleanup_point(client,g,&format!("before/{i}")));
            client.client.submit_many(g,actor,"cleanup_discard",choices)?;
            client.client.consumed.push(raw.clone());client.client.cursor+=1;
            let mut prior=Game::new().unwrap();prior.restore(&before).unwrap();
            // Current positive normal-reset cases require exactly one card.
            // Cardinality itself is validated above by the real semantic API.
            if selection.len()==1 {
                let e=SpellChoice{sequence:i,turn,step:priority_step(step).into(),actor:seat_index(actor) as u8,kind:"discard".into(),source:selection[0]["source"].as_str().map(str::to_owned),incarnation:selection[0]["incarnation"].as_u64(),color:None,role:None,mode:None};
                if let Some((trial,points))=cleanup_probe(client,&prior,&e)?{require(cleanup_point(client,&trial,"quantum")==cleanup_point(client,g,"quantum") && format!("{:?}",trial.rng)==format!("{:?}",g.rng),"/quantum/scalar cleanup equality")?;client.client.stops.extend(points);}
            }
            continue;
        }
        let e:SpellChoice=serde_json::from_value(raw.clone()).map_err(|e|format!("first divergence: /fields {e}"))?;
        // Validate/apply scalar first; probe only accepted actions to keep
        // invalid actor/incarnation/cardinality failures at the real API.
        let before=g.snapshot();
        client.apply(g,&e)?;
        let mut prior=Game::new().unwrap();prior.restore(&before).unwrap();
        if let Some((trial,points))=cleanup_probe(client,&prior,&e)? {
            require(cleanup_point(client,&trial,"quantum")==cleanup_point(client,g,"quantum") && format!("{:?}",trial.rng)==format!("{:?}",g.rng),"/quantum/scalar cleanup equality")?;
            client.client.stops.extend(points);
        }
        let last=client.client.points.last_mut().unwrap();last["outcome"]=Value::Null;
    }
    Err("first divergence: /omitted final checkpoint".into())
}
fn cleanup_consume(g: &mut Game, doc:Value, rejected:&mut Value)->Result<BTreeMap<String,Value>,String>{
    require(doc["schema_version"]==7 && doc["family"]=="cleanup", "/version/family")?;
    let mut projection=doc.clone();projection["schema_version"]=json!(3);projection["family"]=json!("priority");
    let (opening,reset)=priority_projection(&projection)?;
    let mut trial=Game::new().unwrap();let mut runs=BTreeMap::new();
    for (n,case) in doc["cases"].as_array().unwrap().iter().enumerate(){
        let (opened,(client,error))=london_continue(&mut trial,&reset.cases[n],&opening["cases"][n],|g,tape|{
            let mut client=SpellClient{client:PlayedClient::new(g,tape,case)?,creation:0,creations:vec![],departures:vec![],cast_start:None};
            let error=cleanup_run(&mut client,g,case).err();Ok((client,error))
        })?;
        let mut report=client.client.report();report["opening"]=opened;report["cleanup_checkpoints"]=json!(client.client.stops);
        report["capture_status"]=json!(if trial.outcome().is_some(){"complete terminal capture"}else{"open prefix"});
        if let Some(error)=error {report["rejected_checkpoint"]=cleanup_point(&client,&trial,"rejected");*rejected=report;return Err(error);}
        runs.insert(case["id"].as_str().unwrap().to_owned(),report);
    }
    *g=trial;Ok(runs)
}
#[test]
fn full_pool_cleanup_literal_checkpoints(){
    let doc:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-cleanup.json")).unwrap();
    let mut g=Game::new().unwrap();
    let runs=cleanup_consume(&mut g,doc.clone(),&mut Value::Null).expect("legal played cleanup/terminal tapes");
    let repeated=cleanup_consume(&mut g,doc.clone(),&mut Value::Null).unwrap();assert_eq!(runs,repeated);
    let points:BTreeMap<_,_>=runs.iter().map(|(id,r)|(id.clone(),r["points"].clone())).collect();
    for starter in [0,1] {
        let last=points[&format!("empty-library-{starter}")].as_array().unwrap().last().unwrap();
        assert_eq!(last["outcome"]["winner"],starter,"CR104/121: nonstarter attempts 34th draw first");
        assert_eq!(last["outcome"]["losses"][1-starter],"EmptyDraw");
        assert_eq!(last["library"],json!([[],[]]));assert_eq!(last["turn"],68);
    }
    let negative_text=if std::env::var("MTG_FULL_POOL_FAMILY").as_deref()==Ok("cleanup") {std::env::var("MTG_FULL_POOL_NEGATIVES").ok().map(|p|std::fs::read_to_string(p).unwrap())}else{None};
    let negatives:Value=serde_json::from_str(negative_text.as_deref().unwrap_or(include_str!("../../../fixtures/reference/full-pool-cleanup-negatives.json"))).unwrap();
    let mut rejections=BTreeMap::new();let mut negative_runs=BTreeMap::new();
    for (name,spec) in negatives.as_object().unwrap(){
        let before=format!("{g:?}");let mut rejected=Value::Null;
        let error=cleanup_consume(&mut g,spec["input"].clone(),&mut rejected).expect_err("invalid cleanup tape accepted");
        assert_eq!(format!("{g:?}"),before,"failed tape changed caller state/RNG");
        assert_eq!(error,format!("first divergence: {}",spec["native"].as_str().unwrap()),"{name}");
        assert_eq!(rejected["consumed_play"].as_array().unwrap().len(),spec["sequence"].as_u64().unwrap() as usize,"{name}: first divergence");
        rejections.insert(name.clone(),error);negative_runs.insert(name.clone(),rejected);
    }
    if let Ok(p)=std::env::var("MTG_FULL_POOL_OUTPUT") && std::env::var("MTG_FULL_POOL_FAMILY").as_deref()==Ok("cleanup") {
        std::fs::write(p,serde_json::to_vec(&json!({"runs":runs,"checkpoints":points,"repeat_runs":repeated,"rejections":rejections,"negative_runs":negative_runs,"rejection_state_rng":"unchanged"})).unwrap()).unwrap();
    }
}
