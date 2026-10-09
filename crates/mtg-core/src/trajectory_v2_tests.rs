//! Independent RFC 0002 §8 ledger; CR 103.5, 601, 508–510, 514.
//! Synthetic setups are labeled; actions always use the real policy API.
use super::*;
use crate::trajectory::{
    self, End, EpisodeKey, Error, Header, Limit, Limits, PolicyInfo, Versions, v2::*,
};
use policy::{Choice as Command, Submission, VisibleRef, VisibleZone};
const CAP: usize = 256;
fn header() -> Header {
    Header {
        id: EpisodeKey {
            run: "a34c952c-723c-44ef-95f9-dcdb066db576".into(),
            ordinal: 0,
        },
        versions: Versions {
            schema: 2,
            engine: "test".into(),
            rules: "cr-20260925".into(),
            cards: "pool-v1".into(),
            action: "policy-v1".into(),
            observation: 1,
        },
        deck_hashes: ["a".repeat(64), "b".repeat(64)],
        config_hash: "c".repeat(64),
        policies: ["script-v1".into(), "script-v1".into()],
        starting_seat: 0,
        limits: Limits::default(),
        restricted_replay: Some("restricted-opaque".into()),
    }
}
fn game() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 0).unwrap();
    g
}
fn frame(g: &Game) -> Frame {
    Frame::capture(g, CAP).expect("structured capture must retain authorized inputs")
}
fn submission(g: &Game, s: Seat, choices: Vec<Command>) -> Submission {
    let d = g.policy_observe(s, CAP).unwrap().decision.unwrap();
    Submission {
        revision: d.revision,
        schema_version: 1,
        generation: d.generation,
        choices,
    }
}
fn send(g: &mut Game, s: Seat, commands: Vec<Command>) {
    let sub = submission(g, s, commands);
    g.apply_policy(s, &sub, CAP).unwrap();
}
fn ready() -> Game {
    let mut g = game();
    send(&mut g, Seat::P0, vec![Command::Keep]);
    send(&mut g, Seat::P1, vec![Command::Keep]);
    g.start_turns().unwrap();
    g
}
fn bf(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Battlefield,
        row,
    }
}
fn hand(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Hand,
        row,
    }
}
fn record(
    g: &mut Game,
    r: &mut Recorder,
    s: Seat,
    commands: Vec<Command>,
    logical: u64,
    micro: u64,
    status: ActionStatus,
) {
    let before = frame(g);
    let expected_observation = g.policy_observe(s, CAP).unwrap();
    let sub = submission(g, s, commands);
    g.apply_policy(s, &sub, CAP).unwrap();
    let choice = Choice {
        submission: sub,
        logical_action: logical,
        micro_choice: micro,
        status,
        policy: PolicyInfo::default(),
    };
    r.append(&before, &choice, &frame(g)).unwrap();
    let stored = r.episode().decisions().last().unwrap();
    assert_eq!(stored.observation, expected_observation);
    assert_eq!(stored.choice, choice);
}
#[test]
fn trajectory_v2_ordered_bottom_owned_and_atomic_rejections() {
    let mut g = game();
    // London mulligan twice, then keep: exactly two ordered bottom selections.
    send(&mut g, Seat::P0, vec![Command::Mulligan]);
    send(&mut g, Seat::P1, vec![Command::Keep]);
    send(&mut g, Seat::P0, vec![Command::Bottom { card: hand(0) }]);
    send(&mut g, Seat::P0, vec![Command::Mulligan]);
    let before = frame(&g);
    let mut r = Recorder::new(&header(), &before).unwrap();
    let expected = g.policy_observe(Seat::P0, CAP).unwrap();
    assert_eq!(expected.decision.as_ref().unwrap().count, 2);
    let commands = vec![
        Command::Bottom { card: hand(5) },
        Command::Bottom { card: hand(1) },
    ];
    let mut sub = submission(&g, Seat::P0, commands.clone());
    g.apply_policy(Seat::P0, &sub, CAP).unwrap();
    let after = frame(&g);
    let mut c = Choice {
        submission: sub.clone(),
        logical_action: 0,
        micro_choice: 0,
        status: ActionStatus::Committed,
        policy: PolicyInfo::default(),
    };
    for invalid in 0..5 {
        let mut bad = c.clone();
        match invalid {
            0 => bad.submission.schema_version = 99,
            1 => bad.submission.generation += 1,
            2 => bad.submission.choices.reverse(),
            3 => bad.submission.choices = vec![commands[0].clone(); 2],
            _ => bad.logical_action = 3,
        }
        if invalid == 2 {
            bad.submission.choices.pop();
        }
        let saved = r.clone();
        assert!(r.append(&before, &bad, &after).is_err());
        assert_eq!(r, saved);
    }
    r.append(&before, &c, &after).unwrap();
    let saved = r.clone();
    assert!(r.append(&before, &c, &after).is_err());
    assert_eq!(r, saved);
    r.finish(&after, End::Truncated(Limit::Decisions)).unwrap();
    sub.choices.clear();
    c.submission.choices.clear();
    g.reset(&Config::default(), 43, 1).unwrap();
    assert_eq!(r.episode().decisions()[0].observation, expected);
    assert_eq!(
        r.episode().decisions()[0].choice.submission.choices,
        commands
    );
    assert_eq!(
        r.episode().seat(Seat::P0).unwrap().transitions[0].decisions_elapsed,
        1
    );
    assert!(r.episode().seat(Seat::P1).unwrap().transitions.is_empty());
    for version in [0, 1, 3, u32::MAX] {
        let mut h = header();
        h.versions.schema = version;
        assert!(Recorder::new(&h, &before).is_err());
    }
    let mut fresh = Recorder::new(&header(), &before).unwrap();
    assert_eq!(
        fresh.finish(&frame(&g), End::Completed),
        Err(Error::Discontinuity)
    );
}
#[test]
fn trajectory_v2_same_seat_interleavings_rewards_and_zero_decision_seats() {
    let mut g = ready();
    let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
    // Four priority passes alternate P0/P1; source-inclusive, next-exclusive intervals.
    for i in 0..4 {
        let s = g.turn_decision().unwrap().actor;
        record(
            &mut g,
            &mut r,
            s,
            vec![Command::Pass],
            i,
            0,
            ActionStatus::Committed,
        );
    }
    g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
    r.finish(&frame(&g), End::Completed).unwrap();
    for (s, reward, links, durations) in [
        (Seat::P0, 1, vec![Some(2), None], vec![2, 2]),
        (Seat::P1, -1, vec![Some(3), None], vec![2, 1]),
    ] {
        let seq = r.episode().seat(s).unwrap();
        assert_eq!(seq.total_return, reward);
        assert_eq!(
            seq.transitions
                .iter()
                .map(|t| t.next_decision)
                .collect::<Vec<_>>(),
            links
        );
        assert_eq!(
            seq.transitions
                .iter()
                .map(|t| t.decisions_elapsed)
                .collect::<Vec<_>>(),
            durations
        );
        assert_eq!(
            seq.transitions.iter().map(|t| t.reward).collect::<Vec<_>>(),
            vec![0, reward]
        );
        assert_eq!(
            seq.transitions.last().unwrap().next_observation,
            seq.final_observation
        );
        assert_eq!(seq.unassigned_reward, 0);
    }
    assert_eq!(
        r.finish(&frame(&g), End::Completed),
        Err(Error::AlreadyEnded)
    );
    let mut g = ready();
    let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
    g.concede(Seat::P0, g.episode_id().unwrap()).unwrap();
    r.finish(&frame(&g), End::Completed).unwrap();
    assert_eq!(r.episode().seat(Seat::P0).unwrap().unassigned_reward, -1);
    assert_eq!(r.episode().seat(Seat::P1).unwrap().unassigned_reward, 1);
}
fn add(g: &mut Game, key: &str, s: Seat, zone: Zone) {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), s, zone)
        .unwrap();
}
fn ordered(hidden: bool) -> Config {
    let m: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let deck = m["decks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["id"] == "green")
        .unwrap();
    let mut order: Vec<String> = [
        "bear-cub",
        "giant-growth",
        "bite-down",
        "forest",
        "forest",
        "forest",
        "forest",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    for c in deck["cards"].as_array().unwrap() {
        let key = c["card_id"].as_str().unwrap();
        for _ in order.iter().filter(|s| s.as_str() == key).count()
            ..c["copies"].as_u64().unwrap() as usize
        {
            order.push(key.into());
        }
    }
    let mut tail = order.clone();
    if hidden {
        tail.swap(4, 20); // Opponent-private hand changes; the two earlier discards stay fixed.
        tail[10..].reverse();
    }
    Config {
        seats: vec![
            DeckConfig {
                deck: "green".into(),
                order: Some(order),
            },
            DeckConfig {
                deck: "green".into(),
                order: Some(tail),
            },
        ],
        ..Config::default()
    }
}
fn card(g: &Game, s: Seat, key: &str) -> VisibleRef {
    hand(
        g.policy_observe(s, CAP)
            .unwrap()
            .view
            .hand
            .iter()
            .position(|c| c.card == key)
            .unwrap(),
    )
}
fn advance(g: &mut Game, turn: u64, step: turns::Step) {
    for _ in 0..150 {
        if g.turn_position()
            .is_some_and(|(t, _, p)| t == turn && p == step)
        {
            return;
        }
        let s = g.turn_decision().unwrap().actor;
        let d = g.policy_observe(s, CAP).unwrap().decision.unwrap();
        let cs = if d.kind == "cleanup_discard" {
            let o = g.policy_observe(s, CAP).unwrap();
            let cards: Vec<_> = o
                .view
                .hand
                .iter()
                .enumerate()
                .filter(|(_, c)| c.card == "forest")
                .take(d.count)
                .map(|(row, _)| Command::Discard { card: hand(row) })
                .collect();
            assert_eq!(cards.len(), d.count);
            cards
        } else if d.factored.is_some() {
            vec![Command::FinishCombat]
        } else {
            vec![Command::Pass]
        };
        send(g, s, cs);
    }
    panic!("bounded played prefix did not reach requested step");
}
fn played(hidden: bool) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&ordered(hidden), 42, 0).unwrap();
    send(&mut g, Seat::P0, vec![Command::Keep]);
    send(&mut g, Seat::P1, vec![Command::Keep]);
    g.start_turns().unwrap();
    for turn in [1, 3] {
        advance(&mut g, turn, turns::Step::PrecombatMain);
        let land = card(&g, Seat::P0, "forest");
        send(&mut g, Seat::P0, vec![Command::PlayLand { card: land }]);
    }
    let cub = card(&g, Seat::P0, "bear-cub");
    send(&mut g, Seat::P0, vec![Command::Cast { card: cub }]);
    for row in [0, 1] {
        send(&mut g, Seat::P0, vec![Command::TapMana { card: bf(row) }]);
        send(&mut g, Seat::P0, vec![Command::Pay { color: 4 }]);
    }
    send(&mut g, Seat::P0, vec![Command::FinishPayment]);
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    advance(&mut g, 5, turns::Step::PrecombatMain);
    g
}
#[test]
fn trajectory_v2_played_pending_privacy_cancellation_and_full_inputs() {
    let mut twins = vec![];
    let mut hidden_hands = vec![];
    for hidden in [false, true] {
        let mut g = played(hidden);
        let before = frame(&g);
        let mut r = Recorder::new(&header(), &before).unwrap();
        let public = g.policy_observe(Seat::P1, CAP).unwrap();
        hidden_hands.push(public.view.hand.clone());
        let growth = card(&g, Seat::P0, "giant-growth");
        let commands = [
            Command::Cast { card: growth },
            Command::Target { card: bf(2) },
            Command::FinishTargets,
            Command::TapMana { card: bf(0) },
            Command::Pay { color: 4 },
            Command::CancelPayment,
        ];
        let mut inputs = vec![];
        for (i, c) in commands.into_iter().enumerate() {
            let o = g.policy_observe(Seat::P0, CAP).unwrap();
            inputs.push(o.clone());
            if i > 0 {
                assert!(o.pending.is_some());
                assert_eq!(g.policy_observe(Seat::P1, CAP).unwrap(), public);
                assert_eq!(trajectory::Frame::capture(&g), Err(Error::Unavailable));
                let p = o.pending.as_ref().unwrap();
                assert_eq!(p.card, growth);
                assert_eq!(p.targets, if i >= 2 { vec![Some(bf(2))] } else { vec![] });
                if i >= 3 {
                    assert_eq!(p.remaining.unwrap().colored[4], if i == 5 { 0 } else { 1 });
                    assert_eq!(p.sources, if i >= 4 { vec![Some(bf(0))] } else { vec![] });
                }
                let bad = submission(&g, Seat::P0, vec![Command::Target { card: bf(999) }]);
                let old = frame(&g);
                assert!(g.apply_policy(Seat::P0, &bad, CAP).is_err());
                assert_eq!(frame(&g), old);
                let saved = r.clone();
                let bad_choice = Choice {
                    submission: bad,
                    logical_action: 0,
                    micro_choice: i as u64,
                    status: ActionStatus::Continuing,
                    policy: PolicyInfo::default(),
                };
                assert!(r.append(&old, &bad_choice, &old).is_err());
                assert_eq!(r, saved);
            }
            record(
                &mut g,
                &mut r,
                Seat::P0,
                vec![c],
                0,
                i as u64,
                if i == 5 {
                    ActionStatus::Cancelled
                } else {
                    ActionStatus::Continuing
                },
            );
        }
        record(
            &mut g,
            &mut r,
            Seat::P0,
            vec![Command::Pass],
            1,
            0,
            ActionStatus::Committed,
        );
        record(
            &mut g,
            &mut r,
            Seat::P1,
            vec![Command::Pass],
            2,
            0,
            ActionStatus::Committed,
        );
        r.finish(&frame(&g), End::Truncated(Limit::Decisions))
            .unwrap();
        for (d, o) in r.episode().decisions().iter().zip(inputs) {
            assert_eq!(d.observation, o);
        }
        let seq = r.episode().seat(Seat::P0).unwrap();
        assert_eq!(
            seq.transitions
                .iter()
                .map(|t| t.decisions_elapsed)
                .collect::<Vec<_>>(),
            vec![1, 1, 1, 1, 1, 1, 2]
        );
        assert_eq!(
            seq.transitions
                .iter()
                .map(|t| t.logical_actions_elapsed)
                .collect::<Vec<_>>(),
            vec![1, 1, 1, 1, 1, 1, 2]
        );
        assert_eq!(
            seq.transitions
                .iter()
                .map(|t| t.cancelled_actions_elapsed)
                .collect::<Vec<_>>(),
            vec![0, 0, 0, 0, 0, 1, 0]
        );
        assert_eq!(r.episode().footer().unwrap().logical_actions, 3);
        assert_eq!(r.episode().footer().unwrap().cancelled_actions, 1);
        twins.push(seq);
    }
    assert_ne!(hidden_hands[0], hidden_hands[1]);
    assert_eq!(twins[0], twins[1]);
}
fn combat_position() -> Game {
    // Explicit small synthetic position, independently enumerated domains below.
    let mut g = ready();
    g.turns.position = Some((3, Seat::P0, turns::Step::BeginningCombat));
    for s in [Seat::P0, Seat::P0, Seat::P1, Seat::P1] {
        add(&mut g, "bear-cub", s, Zone::Battlefield);
    }
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    g
}
#[test]
fn trajectory_v2_combat_domains_subsets_maps_allocations_and_backtracking() {
    for bits in 0..4 {
        let mut g = combat_position();
        let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
        let cards = (0..2)
            .filter(|i| bits & (1 << i) != 0)
            .map(bf)
            .collect::<Vec<_>>();
        record(
            &mut g,
            &mut r,
            Seat::P0,
            vec![Command::SelectAttackers {
                cards: cards.clone(),
            }],
            0,
            0,
            ActionStatus::Continuing,
        );
        record(
            &mut g,
            &mut r,
            Seat::P0,
            vec![Command::SelectAttackers { cards: vec![bf(1)] }],
            0,
            1,
            ActionStatus::Continuing,
        );
        record(
            &mut g,
            &mut r,
            Seat::P0,
            vec![Command::FinishCombat],
            0,
            2,
            ActionStatus::Committed,
        );
        let ds = r.episode().decisions();
        let domain = ds[0].observation.decision.as_ref().unwrap();
        assert_eq!(domain.candidates, vec![Command::FinishCombat]);
        assert_eq!(domain.legal_mask, vec![true]);
        assert_eq!(
            domain.factored.as_ref().unwrap().attackers,
            vec![bf(0), bf(1)]
        );
        assert_eq!(
            ds[1]
                .observation
                .decision
                .as_ref()
                .unwrap()
                .factored
                .as_ref()
                .unwrap()
                .selected,
            cards
        );
        assert_eq!(
            ds[2]
                .observation
                .decision
                .as_ref()
                .unwrap()
                .factored
                .as_ref()
                .unwrap()
                .selected,
            vec![bf(1)]
        );
    }
    for x in 0..3 {
        for y in 0..3 {
            let mut g = combat_position();
            send(
                &mut g,
                Seat::P0,
                vec![Command::SelectAttackers {
                    cards: vec![bf(0), bf(1)],
                }],
            );
            send(&mut g, Seat::P0, vec![Command::FinishCombat]);
            send(&mut g, Seat::P0, vec![Command::Pass]);
            send(&mut g, Seat::P1, vec![Command::Pass]);
            let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
            let blocks = [(2, x), (3, y)]
                .into_iter()
                .filter(|(_, a)| *a != 0)
                .map(|(b, a)| (bf(b), bf(a - 1)))
                .collect::<Vec<_>>();
            record(
                &mut g,
                &mut r,
                Seat::P1,
                vec![Command::SelectBlockers {
                    blocks: blocks.clone(),
                }],
                0,
                0,
                ActionStatus::Continuing,
            );
            record(
                &mut g,
                &mut r,
                Seat::P1,
                vec![Command::FinishCombat],
                0,
                1,
                ActionStatus::Committed,
            );
            let f = r.episode().decisions()[1]
                .observation
                .decision
                .as_ref()
                .unwrap()
                .factored
                .as_ref()
                .unwrap();
            assert_eq!(f.attackers, vec![bf(0), bf(1)]);
            assert_eq!(f.blockers, vec![bf(2), bf(3)]);
            assert_eq!(f.blocks, blocks);
        }
    }
    // CR 510 permits all divisions; the API also permits omitted zero recipients.
    for n in 0..=3 {
        let mut g = combat_position();
        send(
            &mut g,
            Seat::P0,
            vec![Command::SelectAttackers { cards: vec![bf(0)] }],
        );
        send(&mut g, Seat::P0, vec![Command::FinishCombat]);
        send(&mut g, Seat::P0, vec![Command::Pass]);
        send(&mut g, Seat::P1, vec![Command::Pass]);
        send(
            &mut g,
            Seat::P1,
            vec![Command::SelectBlockers {
                blocks: vec![(bf(2), bf(0)), (bf(3), bf(0))],
            }],
        );
        send(&mut g, Seat::P1, vec![Command::FinishCombat]);
        send(&mut g, Seat::P0, vec![Command::Pass]);
        send(&mut g, Seat::P1, vec![Command::Pass]);
        let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
        let amounts = if n == 3 {
            vec![(bf(2), 2)]
        } else {
            vec![(bf(2), n), (bf(3), 2 - n)]
        };
        record(
            &mut g,
            &mut r,
            Seat::P0,
            vec![Command::AssignDamage {
                attacker: bf(0),
                amounts: amounts.clone(),
            }],
            0,
            0,
            ActionStatus::Continuing,
        );
        record(
            &mut g,
            &mut r,
            Seat::P0,
            vec![Command::FinishCombat],
            0,
            1,
            ActionStatus::Committed,
        );
        let a = &r.episode().decisions()[1]
            .observation
            .decision
            .as_ref()
            .unwrap()
            .factored
            .as_ref()
            .unwrap()
            .damage[0];
        assert_eq!(a.power, 2);
        assert_eq!(a.blockers, vec![bf(2), bf(3)]);
        assert_eq!(a.amounts, Some(amounts));
    }
}
#[test]
fn trajectory_v2_cleanup_multiple_discards_draw_failure_and_nonacting_reward() {
    let mut g = ready();
    // Explicit nine-card synthetic setup at cleanup; CR 514 requires two discards.
    add(&mut g, "forest", Seat::P0, Zone::Hand(Seat::P0));
    add(&mut g, "forest", Seat::P0, Zone::Hand(Seat::P0));
    g.turns.position = Some((1, Seat::P0, turns::Step::End));
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
    assert_eq!(
        g.policy_observe(Seat::P0, CAP)
            .unwrap()
            .decision
            .unwrap()
            .count,
        2
    );
    let cs = vec![
        Command::Discard { card: hand(8) },
        Command::Discard { card: hand(0) },
    ];
    record(
        &mut g,
        &mut r,
        Seat::P0,
        cs.clone(),
        0,
        0,
        ActionStatus::Committed,
    );
    g.concede(Seat::P0, g.episode_id().unwrap()).unwrap();
    r.finish(&frame(&g), End::Completed).unwrap();
    assert_eq!(r.episode().decisions()[0].choice.submission.choices, cs);
    assert_eq!(r.episode().seat(Seat::P1).unwrap().unassigned_reward, 1);
    assert_eq!(
        r.episode().seat(Seat::P0).unwrap().transitions[0].reward,
        -1
    );
    let mut g = ready();
    let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
    g.life = [0, 0];
    g.settle_terminal(None);
    r.finish(&frame(&g), End::Completed).unwrap();
    for s in [Seat::P0, Seat::P1] {
        assert_eq!(r.episode().seat(s).unwrap().total_return, 0);
    }
    let g = ready();
    let f = frame(&g);
    let mut r = Recorder::new(&header(), &f).unwrap();
    assert_eq!(r.finish(&f, End::Completed), Err(Error::InvalidEnd));
    r.finish(&f, End::Failed("explicit producer failure".into()))
        .unwrap();
    assert_eq!(r.episode().seat(Seat::P0), Err(Error::Quarantined));
}
#[test]
fn trajectory_v2_played_commit_retains_payment_targets_and_owned_policy_data() {
    let mut g = played(false);
    let f = frame(&g);
    let mut h = header();
    let mut r = Recorder::new(&h, &f).unwrap();
    h.policies[0].clear();
    let growth = card(&g, Seat::P0, "giant-growth");
    for (i, c) in [
        Command::Cast { card: growth },
        Command::Target { card: bf(2) },
        Command::FinishTargets,
        Command::TapMana { card: bf(0) },
        Command::Pay { color: 4 },
        Command::FinishPayment,
    ]
    .into_iter()
    .enumerate()
    {
        record(
            &mut g,
            &mut r,
            Seat::P0,
            vec![c],
            0,
            i as u64,
            if i == 5 {
                ActionStatus::Committed
            } else {
                ActionStatus::Continuing
            },
        );
    }
    let o = g.policy_observe(Seat::P0, CAP).unwrap();
    assert!(o.pending.is_none());
    assert_eq!(o.stack[0].targets, vec![Some(bf(2))]);
    let before = frame(&g);
    let sub = submission(&g, Seat::P0, vec![Command::Pass]);
    g.apply_policy(Seat::P0, &sub, CAP).unwrap();
    let mut c = Choice {
        submission: sub,
        logical_action: 1,
        micro_choice: 0,
        status: ActionStatus::Committed,
        policy: PolicyInfo {
            checkpoint: Some("literal-checkpoint".into()),
            log_probability: Some(-0.5),
            value: Some(0.25),
            ..PolicyInfo::default()
        },
    };
    let after = frame(&g);
    r.append(&before, &c, &after).unwrap();
    c.policy.checkpoint.as_mut().unwrap().clear();
    record(
        &mut g,
        &mut r,
        Seat::P1,
        vec![Command::Pass],
        2,
        0,
        ActionStatus::Committed,
    );
    let final_o = g.policy_observe(Seat::P0, CAP).unwrap();
    assert_eq!(
        final_o.view.public_zones[2].cards[2].creature,
        Some([5, 5, 0])
    );
    r.finish(&frame(&g), End::Truncated(Limit::Turns)).unwrap();
    assert_eq!(
        r.episode().decisions()[6]
            .choice
            .policy
            .checkpoint
            .as_deref(),
        Some("literal-checkpoint")
    );
    assert_eq!(
        r.episode().require_probabilities(),
        Err(Error::MissingProbability)
    );
    let saved = r.episode().clone();
    let mut seq = r.episode().seat(Seat::P0).unwrap();
    seq.transitions[0].observation.view.hand.clear();
    seq.transitions[0].choice.submission.choices.clear();
    g.reset(&ordered(false), 99, 2).unwrap();
    assert_eq!(r.episode(), &saved);
    let json = serde_json::to_string(&r.episode().seat(Seat::P0).unwrap()).unwrap();
    assert!(!json.contains("restricted-opaque"));
    assert!(!json.contains("restricted_replay"));
    assert!(!json.contains("seed"));
    assert_eq!(r.episode().header().policies[0], "script-v1");
}
#[test]
fn trajectory_v2_terminal_action_rewards_once_and_truncation_pending() {
    let mut g = combat_position();
    g.life = [20, 2];
    send(
        &mut g,
        Seat::P0,
        vec![Command::SelectAttackers { cards: vec![bf(0)] }],
    );
    send(&mut g, Seat::P0, vec![Command::FinishCombat]);
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::FinishCombat]);
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    let before = frame(&g);
    let mut r = Recorder::new(&header(), &before).unwrap();
    record(
        &mut g,
        &mut r,
        Seat::P0,
        vec![Command::FinishCombat],
        0,
        0,
        ActionStatus::Committed,
    );
    assert_eq!(r.episode().decisions()[0].reward, [1, -1]);
    assert_eq!(r.episode().footer().unwrap().boundary_reward, [0, 0]);
    assert_eq!(r.episode().seat(Seat::P0).unwrap().transitions[0].reward, 1);
    assert_eq!(r.episode().seat(Seat::P1).unwrap().unassigned_reward, -1);
    let saved = r.clone();
    assert_eq!(
        r.finish(&frame(&g), End::Completed),
        Err(Error::AlreadyEnded)
    );
    assert_eq!(r, saved);
    for limit in [Limit::Decisions, Limit::Turns, Limit::WallTime] {
        let mut g = played(false);
        let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
        let growth = card(&g, Seat::P0, "giant-growth");
        record(
            &mut g,
            &mut r,
            Seat::P0,
            vec![Command::Cast { card: growth }],
            0,
            0,
            ActionStatus::Continuing,
        );
        r.finish(&frame(&g), End::Truncated(limit)).unwrap();
        let seq = r.episode().seat(Seat::P0).unwrap();
        assert!(seq.final_observation.pending.is_some());
        assert!(seq.transitions[0].truncated);
        assert!(!seq.transitions[0].terminated);
        assert_eq!(seq.total_return, 0);
        assert!(
            r.episode()
                .seat(Seat::P1)
                .unwrap()
                .final_observation
                .pending
                .is_none()
        );
    }
}
#[test]
fn trajectory_v2_rejection_matrix_is_transactional() {
    let mut g = combat_position();
    let before = frame(&g);
    let mut r = Recorder::new(&header(), &before).unwrap();
    let sub = submission(
        &g,
        Seat::P0,
        vec![Command::SelectAttackers { cards: vec![bf(0)] }],
    );
    g.apply_policy(Seat::P0, &sub, CAP).unwrap();
    let after = frame(&g);
    let c = Choice {
        submission: sub,
        logical_action: 0,
        micro_choice: 0,
        status: ActionStatus::Continuing,
        policy: PolicyInfo::default(),
    };
    for i in 0..10 {
        let mut bad = c.clone();
        match i {
            0 => bad.submission.revision += 1,
            1 => {
                bad.submission.choices = vec![Command::SelectAttackers {
                    cards: vec![bf(0), bf(0)],
                }]
            }
            2 => {
                bad.submission.choices = vec![Command::SelectAttackers {
                    cards: vec![bf(99)],
                }]
            }
            3 => bad.status = ActionStatus::Cancelled,
            4 => bad.micro_choice = 1,
            5 => bad.policy.value = Some(f64::INFINITY),
            6 => bad.policy.log_probability = Some(0.5),
            7 => bad.submission.choices = vec![Command::Spell],
            8 => bad.submission.choices.clear(),
            _ => bad.submission.choices = vec![Command::SelectBlockers { blocks: vec![] }],
        }
        let saved = r.clone();
        assert!(r.append(&before, &bad, &after).is_err(), "invalid case {i}");
        assert_eq!(r, saved);
    }
    assert!(r.append(&before, &c, &before).is_err());
    let foreign = frame(&combat_position());
    assert_eq!(r.append(&before, &c, &foreign), Err(Error::Discontinuity));
    r.append(&before, &c, &after).unwrap();
    let saved = r.clone();
    assert_eq!(
        r.finish(&before, End::Truncated(Limit::Decisions)),
        Err(Error::InvalidEnd)
    );
    assert_eq!(r, saved);
    assert_eq!(
        r.finish(&after, End::Failed(" ".into())),
        Err(Error::InvalidEnd)
    );
    assert_eq!(r, saved);
    assert_eq!(r.episode().seat(Seat::P0), Err(Error::Quarantined));
    // Finishing the provisional declaration must retain its logical ID and next micro index.
    let sub = submission(&g, Seat::P0, vec![Command::FinishCombat]);
    g.apply_policy(Seat::P0, &sub, CAP).unwrap();
    let end = frame(&g);
    let mut finish = Choice {
        submission: sub,
        logical_action: 1,
        micro_choice: 0,
        status: ActionStatus::Committed,
        policy: PolicyInfo::default(),
    };
    assert!(r.append(&after, &finish, &end).is_err());
    assert_eq!(r, saved);
    finish.logical_action = 0;
    finish.micro_choice = 1;
    r.append(&after, &finish, &end).unwrap();
    assert_eq!(Frame::capture(&g, 0), Err(Error::Unavailable));
    let mut h = header();
    h.versions.observation = 99;
    assert_eq!(Recorder::new(&h, &end), Err(Error::InvalidHeader));
}
fn invalid_domains(g: &mut Game, good: Command, bad: Vec<Command>) {
    let before = frame(g);
    let s = g.turn_decision().unwrap().actor;
    let mut r = Recorder::new(&header(), &before).unwrap();
    let sub = submission(g, s, vec![good]);
    g.apply_policy(s, &sub, CAP).unwrap();
    let after = frame(g);
    let choice = Choice {
        submission: sub,
        logical_action: 0,
        micro_choice: 0,
        status: ActionStatus::Continuing,
        policy: PolicyInfo::default(),
    };
    for command in bad {
        let mut invalid = choice.clone();
        invalid.submission.choices = vec![command];
        let saved = r.clone();
        assert_eq!(
            r.append(&before, &invalid, &after),
            Err(Error::InvalidChoice)
        );
        assert_eq!(r, saved);
    }
    r.append(&before, &choice, &after).unwrap();
    assert_eq!(r.episode().decisions()[0].choice, choice);
}
#[test]
fn trajectory_v2_invalid_maps_allocations_and_target_cancellation() {
    let mut g = combat_position();
    send(
        &mut g,
        Seat::P0,
        vec![Command::SelectAttackers { cards: vec![bf(0)] }],
    );
    send(&mut g, Seat::P0, vec![Command::FinishCombat]);
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    invalid_domains(
        &mut g,
        Command::SelectBlockers {
            blocks: vec![(bf(2), bf(0)), (bf(3), bf(0))],
        },
        vec![
            Command::SelectBlockers {
                blocks: vec![(bf(2), bf(0)), (bf(2), bf(0))],
            },
            Command::SelectBlockers {
                blocks: vec![(bf(0), bf(0))],
            },
            Command::SelectBlockers {
                blocks: vec![(bf(2), bf(1))],
            },
            Command::SelectBlockers {
                blocks: vec![(hand(2), bf(0))],
            },
        ],
    );
    send(&mut g, Seat::P1, vec![Command::FinishCombat]);
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    invalid_domains(
        &mut g,
        Command::AssignDamage {
            attacker: bf(0),
            amounts: vec![(bf(2), 2)],
        },
        vec![
            Command::AssignDamage {
                attacker: bf(1),
                amounts: vec![(bf(2), 2)],
            },
            Command::AssignDamage {
                attacker: bf(0),
                amounts: vec![(bf(2), 1)],
            },
            Command::AssignDamage {
                attacker: bf(0),
                amounts: vec![(bf(2), 1), (bf(2), 1)],
            },
            Command::AssignDamage {
                attacker: bf(0),
                amounts: vec![(bf(1), 2)],
            },
            Command::AssignDamage {
                attacker: bf(0),
                amounts: vec![(bf(2), u32::MAX), (bf(3), 3)],
            },
            Command::FinishCombat, // Masked until the required allocation is supplied.
        ],
    );
    let mut g = played(false);
    let mut r = Recorder::new(&header(), &frame(&g)).unwrap();
    let growth = card(&g, Seat::P0, "giant-growth");
    record(
        &mut g,
        &mut r,
        Seat::P0,
        vec![Command::Cast { card: growth }],
        0,
        0,
        ActionStatus::Continuing,
    );
    record(
        &mut g,
        &mut r,
        Seat::P0,
        vec![Command::CancelTargets],
        0,
        1,
        ActionStatus::Cancelled,
    );
    record(
        &mut g,
        &mut r,
        Seat::P0,
        vec![Command::Cast { card: growth }],
        1,
        0,
        ActionStatus::Continuing,
    );
    record(
        &mut g,
        &mut r,
        Seat::P0,
        vec![Command::CancelTargets],
        1,
        1,
        ActionStatus::Cancelled,
    );
    r.finish(&frame(&g), End::Truncated(Limit::Decisions))
        .unwrap();
    assert_eq!(r.episode().footer().unwrap().cancelled_actions, 2);
    assert_eq!(r.episode().footer().unwrap().logical_actions, 2);
}

#[test]
fn m2_terminal_reward() {
    let mut g = combat_position();
    g.life = [20, 2];
    send(
        &mut g,
        Seat::P0,
        vec![Command::SelectAttackers { cards: vec![bf(0)] }],
    );
    send(&mut g, Seat::P0, vec![Command::FinishCombat]);
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::FinishCombat]);
    send(&mut g, Seat::P0, vec![Command::Pass]);
    send(&mut g, Seat::P1, vec![Command::Pass]);
    let before = frame(&g);
    let mut r = Recorder::new(&header(), &before).unwrap();
    record(
        &mut g,
        &mut r,
        Seat::P0,
        vec![Command::FinishCombat],
        0,
        0,
        ActionStatus::Committed,
    );
    // RFC B035/B036: the unblocked Cub's two damage wins for P0.
    assert_eq!(
        r.episode().decisions()[0].reward,
        [1, -1],
        "M2-MUT terminal_reward: winner plus one loser minus one"
    );
}
