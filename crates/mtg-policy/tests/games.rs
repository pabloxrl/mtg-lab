//! Original reachable scripts; no direct state edits. CR 103/117/305/601/608,
//! 508–510/514/704 and frozen Bear Cub/Growth/Bite text justify checkpoints.
use mtg_core::{
    game::{
        Config, DeckConfig, Game,
        policy::{Choice, Observation, PolicyError, Submission, VisibleRef, VisibleZone},
        turns::Step,
    },
    objects::Seat,
};
use mtg_policy::{LegalRandom, RNG_VERSION, VERSION};
const CAP: usize = 1024;
fn seat(n: u8) -> Seat {
    if n == 0 { Seat::P0 } else { Seat::P1 }
}
fn config(start: u8, twin: bool) -> Config {
    let mut order = [
        ("bear-cub", 2),
        ("giant-growth", 1),
        ("bite-down", 1),
        ("forest", 16),
        ("bear-cub", 2),
        ("giant-growth", 2),
        ("bite-down", 2),
        ("llanowar-elves", 3),
        ("druid-of-the-cowl", 2),
        ("magnigoth-sentry", 2),
        ("tajuru-pathwarden", 2),
        ("thornweald-archer", 3),
        ("wildheart-invoker", 2),
    ]
    .into_iter()
    .flat_map(|(k, n)| vec![k.to_owned(); n])
    .collect::<Vec<_>>();
    if twin {
        order[25..].reverse();
    }
    let mut seats = vec![
        DeckConfig {
            deck: "green".into(),
            order: Some(order)
        };
        2
    ];
    if twin {
        seats[usize::from(1 - start)]
            .order
            .as_mut()
            .unwrap()
            .swap(2, 39);
    }
    Config {
        starting_seat: start,
        seats,
        ..Config::default()
    }
}
fn new(start: u8, twin: bool) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&config(start, twin), 19, 4).unwrap();
    g
}
fn active(g: &Game) -> (Seat, Observation) {
    for s in [Seat::P0, Seat::P1] {
        let o = g.policy_observe(s, CAP).unwrap();
        if o.decision.is_some() {
            return (s, o);
        }
    }
    panic!("no active observation")
}
fn advance(g: &mut Game) {
    if g.decision().is_none() && g.turn_position().is_none() && g.outcome().is_none() {
        g.start_turns().unwrap();
    }
}
fn apply(g: &mut Game, s: Seat, a: &Submission) {
    g.apply_policy(s, a, CAP).unwrap();
    advance(g);
}
fn force(g: &mut Game, accept: impl Fn(&Choice, &Observation) -> bool) -> Choice {
    let (s, o) = active(g);
    // Bounded search over independently initialized policies forces coverage of
    // a legal choice in this reachable position; it is not the complete-game policy.
    for seed in 0..4096 {
        let mut p = LegalRandom::new(VERSION, RNG_VERSION, seed, 0, o.view.seat).unwrap();
        let a = p.choose(&o).unwrap();
        if a.choices.len() == 1 && accept(&a.choices[0], &o) {
            let c = a.choices[0].clone();
            apply(g, s, &a);
            return c;
        }
    }
    panic!("required choice not selected: {:?}", o.decision)
}
fn exact(g: &mut Game, wanted: Choice) {
    force(g, |c, _| *c == wanted);
}
fn card(o: &Observation, r: &VisibleRef) -> &'static str {
    match r.zone {
        VisibleZone::Hand => o.view.hand[r.row].card,
        VisibleZone::Battlefield => {
            o.view
                .public_zones
                .iter()
                .find(|z| z.zone == "battlefield")
                .unwrap()
                .cards[r.row]
                .card
        }
    }
}
fn idle(g: &mut Game) {
    let (s, o) = active(g);
    let d = o.decision.as_ref().unwrap();
    match d.kind {
        "priority" => exact(g, Choice::Pass),
        "attackers" | "blockers" | "combat_damage" => exact(g, Choice::FinishCombat),
        "cleanup_discard" => {
            let a = LegalRandom::new(VERSION, RNG_VERSION, 0, 0, o.view.seat)
                .unwrap()
                .choose(&o)
                .unwrap();
            apply(g, s, &a);
        }
        _ => panic!("unexpected {:?}", d),
    }
}
fn until(g: &mut Game, t: u64, step: Step) {
    for _ in 0..500 {
        if g.turn_position().is_some_and(|p| p.0 == t && p.2 == step) {
            return;
        }
        idle(g);
    }
    panic!("script limit");
}
fn cast(g: &mut Game, name: &str) {
    force(
        g,
        |c, o| matches!(c,Choice::Cast{card:r} if card(o,r)==name),
    );
}
fn pay(g: &mut Game) {
    for _ in 0..20 {
        let (_, o) = active(g);
        let d = o.decision.as_ref().unwrap();
        if d.kind != "payment" {
            return;
        }
        let desired = d
            .candidates
            .iter()
            .zip(&d.legal_mask)
            .find(|(c, ok)| **ok && matches!(c, Choice::FinishPayment | Choice::Pay { .. }))
            .or_else(|| {
                d.candidates
                    .iter()
                    .zip(&d.legal_mask)
                    .find(|(c, ok)| **ok && matches!(c, Choice::TapMana { .. }))
            })
            .unwrap()
            .0
            .clone();
        exact(g, desired);
    }
    panic!("payment bound");
}
fn pair(g: &mut Game) {
    exact(g, Choice::Pass);
    exact(g, Choice::Pass);
}
fn setup(start: u8, twin: bool) -> Game {
    let mut g = new(start, twin);
    exact(&mut g, Choice::Keep);
    exact(&mut g, Choice::Keep);
    for t in 1..=6 {
        until(&mut g, t, Step::PrecombatMain);
        force(&mut g, |c, _| matches!(c, Choice::PlayLand { .. }));
        if t >= 3 {
            cast(&mut g, "bear-cub");
            pay(&mut g);
            pair(&mut g);
        }
    }
    until(&mut g, 7, Step::PrecombatMain);
    g
}
#[test]
fn reachable_both_seat_land_creature_spells_combat_and_damage() {
    for start in [0, 1] {
        let mut g = setup(start, false);
        let (_, o) = active(&g);
        let battlefield = o
            .view
            .public_zones
            .iter()
            .find(|z| z.zone == "battlefield")
            .unwrap();
        assert_eq!(
            battlefield
                .cards
                .iter()
                .filter(|c| c.card == "bear-cub")
                .count(),
            4
        );
        assert_eq!(g.life(), [20, 20]);
        until(&mut g, 7, Step::DeclareAttackers);
        let (_, o) = active(&g);
        let f = o.decision.unwrap().factored.unwrap();
        let attacker = f.attackers[0];
        exact(
            &mut g,
            Choice::SelectAttackers {
                cards: vec![attacker],
            },
        );
        exact(&mut g, Choice::FinishCombat);
        pair(&mut g);
        let (_, o) = active(&g);
        let f = o.decision.unwrap().factored.unwrap();
        assert_eq!(f.blockers.len(), 2);
        exact(
            &mut g,
            Choice::SelectBlockers {
                blocks: f.blockers.iter().map(|b| (*b, attacker)).collect(),
            },
        );
        exact(&mut g, Choice::FinishCombat);
        pair(&mut g);
        let (_, o) = active(&g);
        let damage = o.decision.unwrap().factored.unwrap().damage[0].clone();
        assert_eq!(damage.power, 2);
        assert_eq!(damage.blockers.len(), 2);
        exact(
            &mut g,
            Choice::AssignDamage {
                attacker,
                amounts: damage.blockers.iter().map(|b| (*b, 1)).collect(),
            },
        );
        exact(&mut g, Choice::FinishCombat);
        // Modern CR510 permits 1+1: attacker dies to four damage, both blockers live.
        let o = g.policy_observe(seat(start), CAP).unwrap();
        assert_eq!(
            o.view
                .public_zones
                .iter()
                .find(|z| z.zone == "battlefield")
                .unwrap()
                .cards
                .iter()
                .filter(|c| c.card == "bear-cub")
                .count(),
            3
        );
        assert_eq!(g.life(), [20, 20]);
        // Remaining active Cub receives Growth, then deals five through Bite.
        cast(&mut g, "giant-growth");
        force(
            &mut g,
            |c, o| matches!(c,Choice::Target{card:r} if o.view.public_zones.iter().find(|z|z.zone=="battlefield").unwrap().cards[r.row].controller==start),
        );
        exact(&mut g, Choice::FinishTargets);
        pay(&mut g);
        pair(&mut g);
        let o = g.policy_observe(seat(start), CAP).unwrap();
        assert!(
            o.view
                .public_zones
                .iter()
                .find(|z| z.zone == "battlefield")
                .unwrap()
                .cards
                .iter()
                .any(|c| c.controller == start && c.creature == Some([5, 5, 0]))
        );
        cast(&mut g, "bite-down");
        force(
            &mut g,
            |c, o| matches!(c,Choice::Target{card:r} if o.view.public_zones.iter().find(|z|z.zone=="battlefield").unwrap().cards[r.row].controller==start),
        );
        force(
            &mut g,
            |c, o| matches!(c,Choice::Target{card:r} if o.view.public_zones.iter().find(|z|z.zone=="battlefield").unwrap().cards[r.row].controller!=start),
        );
        exact(&mut g, Choice::FinishTargets);
        pay(&mut g);
        pair(&mut g);
        let o = g.policy_observe(seat(start), CAP).unwrap();
        assert_eq!(
            o.view
                .public_zones
                .iter()
                .find(|z| z.zone == "battlefield")
                .unwrap()
                .cards
                .iter()
                .filter(|c| c.card == "bear-cub")
                .count(),
            2
        );
    }
}
#[test]
fn hidden_twins_evaluation_nonmutation_and_rejected_candidates() {
    for start in [0, 1] {
        let mut a = setup(start, false);
        let b = setup(start, true);
        let (s, oa) = active(&a);
        let (_, ob) = active(&b);
        assert_eq!(oa, ob);
        assert_ne!(
            a.policy_observe(seat(1 - start), CAP).unwrap().view.hand,
            b.policy_observe(seat(1 - start), CAP).unwrap().view.hand
        );
        assert_ne!(semantic_state(&a), semantic_state(&b));
        let before = serde_json::to_value(&a).unwrap();
        let mut p = LegalRandom::new(VERSION, RNG_VERSION, 42, 7, start).unwrap();
        let mut q = LegalRandom::new(VERSION, RNG_VERSION, 42, 7, start).unwrap();
        for _ in 0..20 {
            assert_eq!(p.choose(&oa), q.choose(&ob));
        }
        let chosen = p.choose(&oa).unwrap();
        assert_eq!(
            serde_json::to_value(&a).unwrap(),
            before,
            "evaluation changes no state, including environment RNG"
        );
        assert_eq!(
            a.apply_policy(seat(1 - start), &chosen, CAP),
            Err(PolicyError::WrongActor)
        );
        let mut wrong = chosen.clone();
        wrong.choices = vec![Choice::PlayLand {
            card: VisibleRef {
                zone: VisibleZone::Hand,
                row: usize::MAX,
            },
        }];
        assert_eq!(
            a.apply_policy(s, &wrong, CAP),
            Err(PolicyError::InvalidSelection)
        );
        assert_eq!(serde_json::to_value(&a).unwrap(), before);
        apply(&mut a, s, &chosen);
        let after = serde_json::to_value(&a).unwrap();
        assert_eq!(
            a.apply_policy(s, &chosen, CAP),
            Err(PolicyError::StaleDecision)
        );
        assert_eq!(serde_json::to_value(&a).unwrap(), after);
    }
}
// ObjectStore scopes intentionally differ between independent games (capability
// safety, doc/objects.md). Normalize only these process-local namespace IDs;
// compare every rules field, object incarnation, pending field and RNG word.
fn semantic_state(g: &Game) -> serde_json::Value {
    fn normalize(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, v) in m {
                    if k == "scope" || k == "store" {
                        *v = 0.into();
                    } else {
                        normalize(v);
                    }
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    normalize(v);
                }
            }
            _ => (),
        }
    }
    let mut v = serde_json::to_value(g).unwrap();
    normalize(&mut v);
    v["objects"]["id"] = 0.into();
    v
}
#[derive(Debug, PartialEq, Eq)]
enum End {
    Terminal,
    Truncated,
}
fn run(
    start: u8,
    seed: u64,
    episode: u64,
    limit: usize,
) -> (End, Vec<Submission>, serde_json::Value, [usize; 2]) {
    let mut g = new(start, false);
    let mut ps = [0, 1].map(|s| LegalRandom::new(VERSION, RNG_VERSION, seed, episode, s).unwrap());
    let mut trace = vec![];
    let mut counts = [0; 2];
    for _ in 0..limit {
        if g.outcome().is_some() {
            return (End::Terminal, trace, semantic_state(&g), counts);
        }
        let (s, o) = active(&g);
        let i = o.view.seat as usize;
        let before = serde_json::to_value(&g).unwrap();
        let a = ps[i].choose(&o).unwrap();
        assert_eq!(serde_json::to_value(&g).unwrap(), before);
        apply(&mut g, s, &a);
        counts[i] += 1;
        trace.push(a);
    }
    (
        if g.outcome().is_some() {
            End::Terminal
        } else {
            End::Truncated
        },
        trace,
        semantic_state(&g),
        counts,
    )
}
#[test]
fn seeded_complete_games_both_seats_and_explicit_truncation() {
    for start in [0, 1] {
        for (seed, episode) in [(0, 0), (42, 7), (1, 1)] {
            let a = run(start, seed, episode, 20000);
            let b = run(start, seed, episode, 20000);
            assert_eq!(a.0, b.0);
            assert_eq!(a.1, b.1);
            assert_eq!(a.2, b.2);
            assert_eq!(a.3, b.3);
            assert_eq!(a.0, End::Terminal, "seed {seed}, start {start}");
            assert!(a.3.iter().all(|n| *n > 0));
            // Independently check that the recorded rules loss has its required
            // condition: CR704.5a life<=0 or CR704.5b empty library on draw.
            for (loser, reason) in a.2["outcome"]["losses"]
                .as_array()
                .unwrap()
                .iter()
                .enumerate()
            {
                match reason.as_str() {
                    Some("Life") => assert!(a.2["life"][loser].as_i64().unwrap() <= 0),
                    Some("EmptyDraw") => {
                        assert!(
                            a.2["objects"]["zones"][loser]
                                .as_array()
                                .unwrap()
                                .is_empty()
                        );
                        assert_eq!(a.2["turns"]["position"][2], "Draw");
                        assert_eq!(
                            a.2["turns"]["position"][1],
                            if loser == 0 { "P0" } else { "P1" }
                        );
                    }
                    None => (),
                    _ => panic!("policy never concedes"),
                }
            }
            println!(
                "start={start} policy_seed={seed} episode={episode} terminal decisions={:?} outcome={}",
                a.3, a.2["outcome"]
            );
            assert!(
                a.1.iter()
                    .flat_map(|s| &s.choices)
                    .any(|c| matches!(c, Choice::PlayLand { .. }))
            );
            assert!(
                a.1.iter()
                    .flat_map(|s| &s.choices)
                    .any(|c| matches!(c, Choice::Cast { .. }))
            );
        }
    }
    for limit in [0, 1, 20] {
        let r = run(0, 0, 0, limit);
        assert_eq!(r.0, End::Truncated);
        assert_eq!(r.1.len(), limit);
        assert!(r.2["outcome"].is_null());
    }
}

#[test]
fn reachable_red_mountain_swab_goblin_supported_content() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let deck = manifest["decks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == "red")
        .unwrap();
    let mut order = deck["cards"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| {
            vec![c["card_id"].as_str().unwrap().to_owned(); c["copies"].as_u64().unwrap() as usize]
        })
        .collect::<Vec<_>>();
    order.sort_by_key(|key| match key.as_str() {
        "swab-goblin" => 0,
        "mountain" => 1,
        _ => 2,
    });
    for start in [0, 1] {
        let mut g = Game::new().unwrap();
        g.reset(
            &Config {
                starting_seat: start,
                seats: vec![
                    DeckConfig {
                        deck: "red".into(),
                        order: Some(order.clone())
                    };
                    2
                ],
                ..Config::default()
            },
            0,
            0,
        )
        .unwrap();
        exact(&mut g, Choice::Keep);
        exact(&mut g, Choice::Keep);
        until(&mut g, 1, Step::PrecombatMain);
        force(&mut g, |c, _| matches!(c, Choice::PlayLand { .. }));
        until(&mut g, 3, Step::PrecombatMain);
        force(&mut g, |c, _| matches!(c, Choice::PlayLand { .. }));
        cast(&mut g, "swab-goblin");
        pay(&mut g);
        pair(&mut g);
        let o = g.policy_observe(seat(start), CAP).unwrap();
        let bf = o
            .view
            .public_zones
            .iter()
            .find(|z| z.zone == "battlefield")
            .unwrap();
        assert_eq!(
            bf.cards
                .iter()
                .filter(|c| c.card == "mountain" && c.tapped)
                .count(),
            2
        );
        assert_eq!(
            bf.cards
                .iter()
                .filter(|c| c.card == "swab-goblin"
                    && c.creature == Some([2, 2, 0])
                    && c.summoning_sick)
                .count(),
            1
        );
    }
}

#[test]
fn hidden_twins_during_target_payment_and_factored_combat() {
    for start in [0, 1] {
        let mut games = [setup(start, false), setup(start, true)];
        let check = |games: &[Game; 2]| {
            let (_, a) = active(&games[0]);
            let (_, b) = active(&games[1]);
            assert_eq!(a, b);
            let before = games.each_ref().map(|g| serde_json::to_value(g).unwrap());
            let mut p = LegalRandom::new(VERSION, RNG_VERSION, 7, 9, start).unwrap();
            let mut q = LegalRandom::new(VERSION, RNG_VERSION, 7, 9, start).unwrap();
            for _ in 0..8 {
                assert_eq!(p.choose(&a), q.choose(&b));
            }
            assert_eq!(
                games.each_ref().map(|g| serde_json::to_value(g).unwrap()),
                before
            );
        };
        for g in &mut games {
            cast(g, "giant-growth");
        }
        check(&games);
        for g in &mut games {
            force(
                g,
                |c, o| matches!(c,Choice::Target{card:r} if o.view.public_zones.iter().find(|z|z.zone=="battlefield").unwrap().cards[r.row].controller==start),
            );
        }
        check(&games);
        for g in &mut games {
            exact(g, Choice::FinishTargets);
        }
        check(&games);
        for g in &mut games {
            force(g, |c, _| matches!(c, Choice::TapMana { .. }));
        }
        check(&games);
        for g in &mut games {
            pay(g);
            pair(g);
            until(g, 7, Step::DeclareAttackers);
        }
        check(&games);
    }
}
