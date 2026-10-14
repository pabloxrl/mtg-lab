//! Explicit synthetic positions for the three exact GH-76 catalog cases.
//! CR 510.2/704.5a and RFC B036: vanilla Cub deals 2, terminal payoff is +/-1.
use super::*;
use crate::trajectory::*;
use turns::{Step, TurnAction, TurnSelection};
fn pass(g: &mut Game) {
    let d = g.turn_decision().unwrap();
    g.apply_turn(
        d.actor,
        &TurnAction {
            decision: d.id,
            selection: TurnSelection::Pass(d.candidate(0)),
        },
    )
    .unwrap();
}
fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 0).unwrap();
    while let Some(d) = g.decision() {
        g.apply(
            d.actor,
            &OpeningAction {
                decision: d.id,
                selection: Selection::Choose(d.candidate(0)),
            },
        )
        .unwrap();
    }
    g.start_turns().unwrap();
    g
}
fn recorder(g: &Game) -> Recorder {
    let h = Header {
        id: EpisodeKey {
            run: "dcc4aecd-0508-438e-9ba1-2622050130ac".into(),
            ordinal: 0,
        },
        versions: Versions {
            schema: 1,
            engine: "mtg-core-test".into(),
            rules: "2026-09-25".into(),
            cards: "foundations_micro_v1".into(),
            action: "combat-v1".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["script-v1".into(), "script-v1".into()],
        starting_seat: 0,
        limits: Limits::default(),
        restricted_replay: None,
    };
    Recorder::new(&h, &Frame::capture(g).unwrap()).unwrap()
}
fn damage_position(life: i64) -> Game {
    let mut g = ready();
    g.turns.position = Some((3, Seat::P0, Step::BeginningCombat));
    g.life = [20, life];
    let cub = g
        .objects
        .allocate(
            CardId::from_key("bear-cub").unwrap(),
            Seat::P0,
            Zone::Battlefield,
        )
        .unwrap();
    pass(&mut g);
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    let d = g.select_attackers(d.actor, d.id, &[cub]).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
    pass(&mut g);
    pass(&mut g);
    let d = g.turn_decision().unwrap();
    let d = g.select_blockers(d.actor, d.id, &[]).unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
    pass(&mut g);
    pass(&mut g);
    g
}
fn finish_choice() -> Choice {
    Choice {
        kind: "damage_completion".into(),
        logical_action: 0,
        micro_choice: 0,
        candidates: vec![Candidate {
            semantic: "finish_combat".into(),
            features: vec![1],
        }],
        legal_mask: vec![true],
        selected: 0,
        policy: PolicyInfo::default(),
    }
}
#[test]
fn trajectory_rules_terminal_rewards_once_positive() {
    let mut g = damage_position(2);
    let mut r = recorder(&g);
    let before = Frame::capture(&g).unwrap();
    let d = g.turn_decision().unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
    r.append(&before, &finish_choice(), &Frame::capture(&g).unwrap())
        .unwrap();
    assert_eq!(g.life(), [20, 0]);
    assert_eq!(
        r.episode().decisions().len(),
        1,
        "lethal damage transition retained"
    );
    assert_eq!(r.episode().decisions()[0].reward, [1, -1]);
    assert!(r.episode().decisions()[0].terminated);
    assert!(!r.episode().decisions()[0].truncated);
    for _ in 0..3 {
        assert_eq!(r.episode().seat(Seat::P0).unwrap().total_return, 1);
        assert_eq!(r.episode().seat(Seat::P1).unwrap().unassigned_reward, -1);
    }
    assert_eq!(
        r.finish(&Frame::capture(&g).unwrap(), End::Completed),
        Err(Error::AlreadyEnded)
    );
}
#[test]
fn trajectory_rules_terminal_rewards_once_negative() {
    // Setup five life: after two damage P1 has exactly three, as catalog specifies.
    let mut g = damage_position(5);
    let mut r = recorder(&g);
    let before = Frame::capture(&g).unwrap();
    let d = g.turn_decision().unwrap();
    g.finish_combat(d.actor, d.id).unwrap();
    r.append(&before, &finish_choice(), &Frame::capture(&g).unwrap())
        .unwrap();
    assert_eq!(g.life(), [20, 3]);
    assert_eq!(
        r.episode().decisions().len(),
        1,
        "nonterminal transition retained"
    );
    assert_eq!(r.episode().decisions()[0].reward, [0, 0]);
    assert!(!r.episode().decisions()[0].terminated);
    assert!(r.episode().footer().is_none());
}
#[test]
fn trajectory_rules_terminal_rewards_once_interaction() {
    let mut g = ready();
    let mut r = recorder(&g);
    g.life = [0, 0];
    g.settle_terminal(None);
    let f = Frame::capture(&g).unwrap();
    r.finish(&f, End::Completed).unwrap();
    assert!(
        r.episode().footer().is_some(),
        "simultaneous-loss boundary must be retained"
    );
    assert_eq!(r.episode().footer().unwrap().returns, [0, 0]);
    for s in [Seat::P0, Seat::P1] {
        let seq = r.episode().seat(s).unwrap();
        assert!(seq.transitions.is_empty());
        assert_eq!(seq.unassigned_reward, 0);
        assert_eq!(seq.final_observation.terminal.unwrap().winner, None);
    }
    assert_eq!(r.finish(&f, End::Completed), Err(Error::AlreadyEnded));
}
