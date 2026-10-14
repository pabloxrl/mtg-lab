// Input generation only. Engine observations below must never become goldens.
fn admission_observed_state(g: &Game) -> Value {
    let objects: Vec<_> = Zone::ALL.into_iter().flat_map(|z| g.objects.in_zone(z)).map(|h| {
        let o=g.objects.get(h).unwrap();let (birth,incarnation)=g.objects.semantic_identity(h).unwrap();
        let creature=g.creature_state(h);let definition=card_definitions::definition(o.card);
        json!({"handle":h,"birth":birth,"incarnation":incarnation,"card":o.card.identity().key,
            "owner":seat_index(o.owner),"controller":seat_index(o.controller),"zone":o.zone,
            "tapped":o.tapped,"sick":g.summoning_sick(h),
            "power":creature.map(|c|c.power),"toughness":creature.map(|c|c.toughness),
            "damage":creature.map(|c|c.damage),"haste":g.has_haste(h),"trample":g.has_trample(h),
            "flying":definition.flying(),"reach":definition.reach(),
            "vigilance":definition.vigilance(),"deathtouch":definition.deathtouch()})
    }).collect();
    json!({"objects":objects,"turns":g.turns,"life":g.life(),"mana":g.mana(),"outcome":g.outcome()})
}

struct OriginChance {
    births: BTreeMap<u64, String>,
    copies: BTreeMap<(usize, String), usize>,
    events: Vec<Value>,
    rng: EpisodeRng,
}
impl opening::ResetChance for OriginChance {
    fn created(&mut self, objects: &ObjectStore, seat: Seat, index: usize, handle: Handle) {
        let card = objects.get(handle).unwrap().card.identity().key;
        let copy = self.copies.entry((seat_index(seat), card.into())).or_default();
        let (birth, incarnation) = objects.semantic_identity(handle).unwrap();
        assert_eq!(incarnation, 0);
        assert_eq!(birth as usize, seat_index(seat) * 40 + index);
        assert!(self.births.insert(birth, format!("{}/{card}/{copy}", seat_index(seat))).is_none());
        *copy += 1;
    }
    fn shuffle(&mut self, objects: &mut ObjectStore, seat: Seat) {
        let mut handles: Vec<_> = objects.in_zone(Zone::Library(seat)).collect();
        let ids = |objects: &ObjectStore, handles: &[Handle]| handles.iter()
            .map(|h| self.births[&objects.semantic_identity(*h).unwrap().0].clone()).collect::<Vec<_>>();
        let before = ids(objects, &handles);
        let mut size = handles.len();
        while size > 1 { shuffle_word(&mut handles, &mut size, self.rng.next_u64()); }
        objects.reorder(Zone::Library(seat), &handles);
        let after = ids(objects, &objects.in_zone(Zone::Library(seat)).collect::<Vec<_>>());
        self.events.push(json!({"sequence":self.events.len(), "kind":"initial_shuffle",
            "actor":seat_index(seat),"source":null,"before":before,"after":after}));
    }
}

#[test]
fn admission_origins_witness_occurrences_before_shuffle_and_replay_actual_heuristic_choices() {
    use crate::admission_policy::{HEURISTIC_VERSION, Heuristic};
    let frozen:Value=serde_json::from_str(include_str!("../../../fixtures/reference/full-pool-admission.json")).unwrap();
    for decks in [["red", "green"], ["green", "red"], ["red", "red"], ["green", "green"]] {
        for starter in [0, 1] {
            let config = Config { starting_seat:starter,
                seats:decks.iter().map(|s| DeckConfig::new(s)).collect(), ..Config::default() };
            let mut g = Game::new().unwrap();
            let mut chance = OriginChance { births:BTreeMap::new(),copies:BTreeMap::new(),events:vec![],
                rng:EpisodeRng::new(VERSION,42,7,Stream::Environment).unwrap() };
            g.reset_with_occurrence_chance(&config,42,7,&mut chance).unwrap();
            let initial = g.snapshot();
            let policies = [0,1].map(|s| Heuristic::new(HEURISTIC_VERSION,s).unwrap());
            let mut history = vec![];
            for _ in 0..20_000 {
                if g.outcome().is_some() { break; }
                let actor = g.policy_observe(Seat::P0,1024).unwrap().view.acting_seat.unwrap();
                let seat = if actor == 0 { Seat::P0 } else { Seat::P1 };
                let observation = g.policy_observe(seat,1024).unwrap();
                let selected = policies[actor as usize].choose(&observation).unwrap();
                let bytes = actions::encode(&g,seat,&selected,1024).unwrap();
                let before = g.snapshot();
                let observed_before = admission_observed_state(&g);
                g.apply_policy(seat,&selected,1024).unwrap();
                if g.decision.is_none() && g.turn_position().is_none() { g.start_turns().unwrap(); }
                history.push(json!({"choice":serde_json::from_slice::<Value>(&bytes).unwrap(),
                    "observation":observation,"selected":selected,
                    "observed_before":observed_before,"observed_after":admission_observed_state(&g),
                    "before":serde_json::from_slice::<Value>(&before).unwrap(),
                    "after":serde_json::from_slice::<Value>(&g.snapshot()).unwrap()}));
            }
            // Retain every attempt, including a budget-limited unsuccessful one.
            if let Ok(folder) = std::env::var("MTG_ADMISSION_OCCURRENCE_ORIGINS") {
                let path = std::path::Path::new(&folder);
                std::fs::create_dir_all(path).unwrap();
                let id = format!("{}-{}-s{starter}",decks[0],decks[1]);
                let doc = json!({"schema_version":1,"purpose":"generation provenance, not oracle",
                    "policy":HEURISTIC_VERSION,"config":config,"master":42,"episode":7,
                    "chance":chance.events,"birth_occurrences":chance.births,
                    "initial":serde_json::from_slice::<Value>(&initial).unwrap(),
                    "history":history,"outcome":g.outcome()});
                serde_json::to_writer(std::io::BufWriter::new(std::fs::File::create(path.join(format!("{id}.json"))).unwrap()),&doc).unwrap();
            }
            let outcome = g.outcome().expect("origin budget exceeded; never admit truncation");
            assert!(!outcome.losses.contains(&Some(terminal::LossReason::Concession)));
            assert_eq!(chance.births.len(),80);
            assert_eq!(chance.events.len(),2);
            let row=format!("GR-040-{}{}-S{starter}-heuristic",decks[0][..1].to_uppercase(),decks[1][..1].to_uppercase());
            let expected=frozen["cases"].as_array().unwrap().iter().find(|c|c["id"]==row).unwrap();
            assert_eq!(json!(chance.events),expected["chance"],"actual pre-shuffle/shuffle provenance");
            assert_eq!(json!(history.iter().map(|r|&r["choice"]).collect::<Vec<_>>()),expected["native_records"],"actual existing heuristic provenance, not a replay-time policy");
            // Replay uses frozen semantic records, never calls a policy.
            let mut replay = Game::new().unwrap();replay.restore(&initial).unwrap();
            for row in &history {
                actions::apply(&mut replay,&serde_json::to_vec(&row["choice"]).unwrap(),1024).unwrap();
                if replay.decision.is_none() && replay.turn_position().is_none() {replay.start_turns().unwrap();}
            }
            assert_eq!(replay.outcome(),g.outcome());
            assert_eq!(replay.life(),g.life());
        }
    }
}
