//! GH-160: real normal resets; CR 103.5, 104.3a, 117, 601, 508–510, 514.
//! Literal commands/timing form the ledger. Captured policy input is checked
//! against the actual input delivered before submission, never history output.
use mtg_core::{
    episode::{Driver, Progress, Status},
    game::{
        Config, DeckConfig,
        policy::{self, Choice as C, VisibleRef, VisibleZone},
    },
    objects::Seat,
    trajectory::{self, EpisodeKey, Header, Limits, PolicyInfo, Versions, v2::ActionStatus as A},
};
use serde_json::{Value, json};
use std::num::NonZeroUsize;
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
        policies: ["ledger".into(), "ledger".into()],
        starting_seat: 0,
        limits: Limits::default(),
        restricted_replay: None,
    }
}
pub fn config() -> Config {
    let prefix = [
        "bear-cub",
        "bear-cub",
        "giant-growth",
        "forest",
        "forest",
        "forest",
        "forest",
    ];
    let inventory = [
        ("forest", 16),
        ("bear-cub", 4),
        ("giant-growth", 3),
        ("bite-down", 3),
        ("llanowar-elves", 3),
        ("druid-of-the-cowl", 2),
        ("magnigoth-sentry", 2),
        ("tajuru-pathwarden", 2),
        ("thornweald-archer", 3),
        ("wildheart-invoker", 2),
    ];
    let mut order: Vec<String> = prefix.into_iter().map(String::from).collect();
    for (key, count) in inventory {
        for _ in order.iter().filter(|x| x.as_str() == key).count()..count {
            order.push(key.into());
        }
    }
    Config {
        seats: vec![
            DeckConfig {
                deck: "green".into(),
                order: Some(order)
            };
            2
        ],
        ..Config::default()
    }
}
fn hand(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Hand,
        row,
    }
}
fn bf(row: usize) -> VisibleRef {
    VisibleRef {
        zone: VisibleZone::Battlefield,
        row,
    }
}
fn normalized(bytes: &[u8]) -> Value {
    fn scrub(v: &mut Value) {
        match v {
            Value::Object(m) => {
                for (k, v) in m {
                    if k == "scope" || k == "store" {
                        *v = json!(0)
                    } else {
                        scrub(v)
                    }
                }
            }
            Value::Array(a) => {
                for v in a {
                    scrub(v)
                }
            }
            _ => (),
        }
    }
    let e: Value = serde_json::from_slice(bytes).unwrap();
    let mut v: Value = serde_json::from_str(e["payload"].as_str().unwrap()).unwrap();
    v["objects"]["id"] = json!(0);
    scrub(&mut v);
    v
}
fn settle(d: &mut Driver) {
    for _ in 0..300 {
        if d.advance(NonZeroUsize::MIN).unwrap() != Progress::InternalYield {
            return;
        }
    }
    panic!("settlement bound")
}
struct Row {
    seat: Seat,
    input: policy::Observation,
    sub: policy::Submission,
    logical: u64,
    micro: u64,
    status: A,
    stats: PolicyInfo,
}
struct Script {
    on: Driver,
    off: Driver,
    rows: Vec<Row>,
    logical: u64,
    micro: u64,
}
impl Script {
    fn new(config: &Config) -> Self {
        Self::with_header(config, &header())
    }
    fn with_header(config: &Config, capture_header: &Header) -> Self {
        let mut on = Driver::new(CAP).unwrap();
        let mut off = Driver::new(CAP).unwrap();
        on.reset_captured(config, 160, 0, NonZeroUsize::MIN, capture_header)
            .unwrap();
        off.reset(config, 160, 0, NonZeroUsize::MIN).unwrap();
        settle(&mut on);
        settle(&mut off);
        Self {
            on,
            off,
            rows: vec![],
            logical: 0,
            micro: 0,
        }
    }
    fn send(&mut self, seat: Seat, kind: &str, choices: Vec<C>, status: A) {
        let input = self.on.observe(seat).unwrap();
        let d = input.decision.as_ref().unwrap();
        assert_eq!(d.kind, kind);
        assert_eq!(input, self.off.observe(seat).unwrap());
        let sub = policy::Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices,
        };
        let snapshot = self.on.privileged_snapshot();
        let count = self.on.privileged_history().len();
        let captured = self.on.trajectory().cloned();
        let mut bad = sub.clone();
        bad.generation += 1;
        assert!(self.on.submit(seat, &bad).is_err());
        let other = if seat == Seat::P0 { Seat::P1 } else { Seat::P0 };
        assert!(self.on.submit(other, &sub).is_err());
        assert_eq!(self.on.privileged_snapshot(), snapshot);
        assert_eq!(self.on.privileged_history().len(), count);
        assert_eq!(self.on.trajectory(), captured.as_ref());
        let stats = if self.rows.len() == 2 {
            PolicyInfo {
                log_probability: Some(-0.25),
                value: Some(0.5),
                checkpoint: Some("supplied".into()),
                ..PolicyInfo::default()
            }
        } else {
            PolicyInfo::default()
        };
        self.on
            .submit_with_policy(seat, &sub, &stats)
            .unwrap_or_else(|e| {
                panic!(
                    "row {} {seat:?} {kind} {:?} {:?}: {e:?}",
                    self.rows.len(),
                    input.view.turn,
                    sub.choices
                )
            });
        self.off.submit(seat, &sub).unwrap();
        self.rows.push(Row {
            seat,
            input,
            sub,
            logical: self.logical,
            micro: self.micro,
            status: status.clone(),
            stats,
        });
        if status == A::Continuing {
            self.micro += 1
        } else {
            self.logical += 1;
            self.micro = 0
        }
        // Exact full state/RNG, modulo independent owner capability scopes only.
        assert_eq!(
            normalized(&self.on.privileged_snapshot()),
            normalized(&self.off.privileged_snapshot())
        );
        self.check();
        if self.on.observe(Seat::P0).unwrap().view.terminal.is_none() {
            settle(&mut self.on);
            settle(&mut self.off);
        }
        assert_eq!(
            normalized(&self.on.privileged_snapshot()),
            normalized(&self.off.privileged_snapshot())
        );
    }
    fn check(&self) {
        let e = self
            .on
            .trajectory()
            .expect("capture must retain every accepted decision");
        assert_eq!(e.decisions().len(), self.rows.len());
        let mut seats = [0, 0];
        for (index, (d, r)) in e.decisions().iter().zip(&self.rows).enumerate() {
            let actor = if r.seat == Seat::P0 { 0 } else { 1 };
            assert_eq!(
                (d.index, d.actor, d.seat_index),
                (index, actor as u8, seats[actor])
            );
            seats[actor] += 1;
            let next_actor = self
                .rows
                .get(index + 1)
                .map(|next| if next.seat == Seat::P0 { 0 } else { 1 })
                .or_else(|| self.on.observe(Seat::P0).unwrap().view.acting_seat);
            assert_eq!(d.next_actor, next_actor);
            assert_eq!(d.observation, r.input);
            assert_eq!(d.choice.submission, r.sub);
            assert_eq!(
                (
                    d.choice.logical_action,
                    d.choice.micro_choice,
                    d.choice.status.clone()
                ),
                (r.logical, r.micro, r.status.clone())
            );
            assert_eq!(d.choice.policy, r.stats);
        }
        assert_eq!(self.on.privileged_history().len(), self.rows.len());
        assert!(self.off.trajectory().is_none());
    }
    fn idle(&mut self) {
        let v = self.on.observe(Seat::P0).unwrap().view;
        let seat = if v.acting_seat == Some(0) {
            Seat::P0
        } else {
            Seat::P1
        };
        let o = self.on.observe(seat).unwrap();
        let d = o.decision.unwrap();
        let choices = match d.kind {
            "priority" => vec![C::Pass],
            "attackers" | "blockers" | "combat_damage" => vec![C::FinishCombat],
            "cleanup_discard" => o
                .view
                .hand
                .iter()
                .enumerate()
                .filter(|(_, c)| c.card == "forest")
                .take(d.count)
                .map(|(row, _)| C::Discard { card: hand(row) })
                .collect(),
            _ => panic!("unscripted {}", d.kind),
        };
        self.send(seat, d.kind, choices, A::Committed);
    }
    fn until(&mut self, turn: u64, step: &str) {
        for _ in 0..200 {
            if self
                .on
                .observe(Seat::P0)
                .unwrap()
                .view
                .turn
                .is_some_and(|p| p.0 == turn && p.2 == step)
            {
                return;
            }
            self.idle()
        }
        panic!("bounded script {turn}/{step}")
    }
    fn card(&self, seat: Seat, key: &str) -> VisibleRef {
        hand(
            self.on
                .observe(seat)
                .unwrap()
                .view
                .hand
                .iter()
                .position(|c| c.card == key)
                .unwrap_or_else(|| {
                    panic!(
                        "missing {key} at {:?}",
                        self.on.observe(seat).unwrap().view.turn
                    )
                }),
        )
    }
    fn finish_concession(mut self, loser: Seat) -> (mtg_core::episode::EpisodeResult, Vec<Row>) {
        self.on
            .concede(loser, self.on.episode_id().unwrap())
            .unwrap();
        self.off
            .concede(loser, self.off.episode_id().unwrap())
            .unwrap();
        assert_eq!(
            normalized(&self.on.privileged_snapshot()),
            normalized(&self.off.privileged_snapshot())
        );
        let result = self.on.finish().unwrap();
        assert!(self.on.finish().is_err());
        assert!(self.on.advance(NonZeroUsize::MIN).is_err());
        if let Some(r) = self.rows.last() {
            assert!(self.on.submit(r.seat, &r.sub).is_err())
        }
        assert!(self.off.finish().unwrap().trajectory().is_none());
        (result, self.rows)
    }
}
fn check_seats(result: &mtg_core::episode::EpisodeResult, rows: &[Row], returns: [i8; 2]) {
    let e = result.trajectory().expect("owned canonical episode");
    let f = e.footer().unwrap();
    assert_eq!(f.returns, returns);
    assert_eq!(f.decisions, rows.len());
    for (s, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
        let seq = e.seat(seat).unwrap();
        let indices: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.seat == seat)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(seq.transitions.len(), indices.len());
        assert_eq!(seq.total_return, returns[s]);
        assert_eq!(
            seq.unassigned_reward,
            if indices.is_empty() { returns[s] } else { 0 }
        );
        for (i, t) in seq.transitions.iter().enumerate() {
            let next = indices.get(i + 1).copied();
            let end = next.unwrap_or(rows.len());
            assert_eq!(t.next_decision, next);
            assert_eq!(t.decisions_elapsed, end - indices[i]);
            let interval = &rows[indices[i]..end];
            let logical = interval
                .iter()
                .map(|r| r.logical)
                .collect::<std::collections::BTreeSet<_>>()
                .len();
            let cancelled = interval.iter().filter(|r| r.status == A::Cancelled).count();
            assert_eq!(t.logical_actions_elapsed, logical);
            assert_eq!(t.cancelled_actions_elapsed, cancelled);
            assert_eq!(
                t.next_observation,
                next.map(|n| rows[n].input.clone())
                    .unwrap_or(f.final_observations[s].clone())
            );
            assert_eq!(t.reward, if next.is_none() { returns[s] } else { 0 });
            assert_eq!(t.terminated, next.is_none());
        }
    }
}
#[test]
fn opening_capture_seat_sequences_concession_and_owned_reset() {
    let mut s = Script::new(&config());
    let d = s.on.observe(Seat::P0).unwrap().decision.unwrap();
    assert_eq!(d.candidates, vec![C::Keep, C::Mulligan]);
    assert_eq!(d.legal_mask, vec![true, true]);
    s.send(Seat::P0, "keep_or_mulligan", vec![C::Keep], A::Committed);
    s.send(Seat::P1, "keep_or_mulligan", vec![C::Keep], A::Committed);
    assert_eq!(
        s.on.observe(Seat::P0).unwrap().view.turn,
        Some((1, 0, "upkeep"))
    );
    s.send(Seat::P0, "priority", vec![C::Pass], A::Committed);
    s.send(Seat::P1, "priority", vec![C::Pass], A::Committed);
    let (result, rows) = s.finish_concession(Seat::P1);
    check_seats(&result, &rows, [1, -1]);
    assert_eq!(
        result
            .trajectory()
            .unwrap()
            .footer()
            .unwrap()
            .boundary_reward,
        [1, -1]
    );
    assert_eq!(
        result.trajectory().unwrap().require_probabilities(),
        Err(trajectory::Error::MissingProbability)
    );
    let saved = result.trajectory().unwrap().clone();
    let mut d = Driver::new(CAP).unwrap();
    d.reset_captured(&config(), 7, 0, NonZeroUsize::MAX, &header())
        .unwrap();
    d.finish().unwrap();
    assert_eq!(result.trajectory(), Some(&saved));
}
#[test]
fn zero_decision_and_nonacting_terminal_seats() {
    for loser in [Seat::P0, Seat::P1] {
        for decisions in 0..=1 {
            let mut s = Script::new(&config());
            if decisions == 1 {
                s.send(Seat::P0, "keep_or_mulligan", vec![C::Keep], A::Committed)
            }
            let (r, rows) = s.finish_concession(loser);
            check_seats(&r, &rows, if loser == Seat::P0 { [-1, 1] } else { [1, -1] });
        }
    }
}
#[test]
fn unfinished_capture_remains_quarantined() {
    let mut s = Script::new(&config());
    s.send(Seat::P0, "keep_or_mulligan", vec![C::Keep], A::Committed);
    let r = s.on.finish().unwrap();
    assert_eq!(r.status(), Status::Incomplete);
    assert!(r.trajectory().unwrap().footer().is_none());
    assert_eq!(
        r.trajectory().unwrap().seat(Seat::P0),
        Err(trajectory::Error::Quarantined)
    );
}
fn board(s: &Script, seat: Seat, key: &str) -> VisibleRef {
    let o = s.on.observe(Seat::P0).unwrap();
    let cards = &o
        .view
        .public_zones
        .iter()
        .find(|z| z.zone == "battlefield")
        .unwrap()
        .cards;
    bf(cards
        .iter()
        .position(|c| c.card == key && c.controller == if seat == Seat::P0 { 0 } else { 1 })
        .unwrap())
}
fn cast_cub(s: &mut Script, seat: Seat) {
    let card = s.card(seat, "bear-cub");
    s.send(seat, "priority", vec![C::Cast { card }], A::Continuing);
    let o = s.on.observe(seat).unwrap();
    let cards = &o
        .view
        .public_zones
        .iter()
        .find(|z| z.zone == "battlefield")
        .unwrap()
        .cards;
    let lands: Vec<_> = cards
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            c.card == "forest" && !c.tapped && c.controller == if seat == Seat::P0 { 0 } else { 1 }
        })
        .map(|(row, _)| row)
        .take(2)
        .collect();
    assert_eq!(lands.len(), 2);
    for row in lands {
        s.send(
            seat,
            "payment",
            vec![C::TapMana { card: bf(row) }],
            A::Continuing,
        );
        s.send(seat, "payment", vec![C::Pay { color: 4 }], A::Continuing);
    }
    s.send(seat, "payment", vec![C::FinishPayment], A::Committed);
    s.idle();
    s.idle();
}
fn growth(s: &mut Script) {
    let card = s.card(Seat::P0, "giant-growth");
    let target = board(s, Seat::P0, "bear-cub");
    let opponent = s.on.observe(Seat::P1).unwrap();
    // Targets cancelled once, payment cancelled once, then committed. Each retry
    // is a distinct logical action; all provisional choices remain decisions.
    s.send(Seat::P0, "priority", vec![C::Cast { card }], A::Continuing);
    s.send(
        Seat::P0,
        "growth_target",
        vec![C::CancelTargets],
        A::Cancelled,
    );
    for cancel in [true, false] {
        s.send(Seat::P0, "priority", vec![C::Cast { card }], A::Continuing);
        assert_eq!(s.on.observe(Seat::P1).unwrap(), opponent);
        let pending = s.on.observe(Seat::P0).unwrap().pending.unwrap();
        assert_eq!(pending.card, card);
        assert!(pending.targets.is_empty());
        s.send(
            Seat::P0,
            "growth_target",
            vec![C::Target { card: target }],
            A::Continuing,
        );
        s.send(
            Seat::P0,
            "targets_complete",
            vec![C::FinishTargets],
            A::Continuing,
        );
        assert_eq!(s.on.observe(Seat::P1).unwrap(), opponent);
        let pending = s.on.observe(Seat::P0).unwrap().pending.unwrap();
        assert_eq!(pending.targets, vec![Some(target)]);
        assert_eq!(pending.remaining.unwrap().colored[4], 1);
        if cancel {
            s.send(Seat::P0, "payment", vec![C::CancelPayment], A::Cancelled)
        } else {
            let land = board(s, Seat::P0, "forest");
            s.send(
                Seat::P0,
                "payment",
                vec![C::TapMana { card: land }],
                A::Continuing,
            );
            s.send(
                Seat::P0,
                "payment",
                vec![C::Pay { color: 4 }],
                A::Continuing,
            );
            s.send(Seat::P0, "payment", vec![C::FinishPayment], A::Committed);
        }
    }
    s.idle();
    s.idle();
    let o = s.on.observe(Seat::P0).unwrap();
    let cub = o
        .view
        .public_zones
        .iter()
        .find(|z| z.zone == "battlefield")
        .unwrap()
        .cards
        .iter()
        .find(|c| c.card == "bear-cub" && c.controller == 0)
        .unwrap();
    assert_eq!(cub.creature, Some([5, 5, 0]));
}
#[test]
fn complete_played_ledger_targets_payment_cancel_retry_and_factored_combat() {
    played_combat(header());
}
// Shared with the recorder's integration audit. All original assertions execute
// in both callers; only actual run provenance replaces the component header.
pub fn played_combat(capture_header: Header) -> mtg_core::episode::EpisodeResult {
    let mut s = Script::with_header(&config(), &capture_header);
    for seat in [Seat::P0, Seat::P1] {
        s.send(seat, "keep_or_mulligan", vec![C::Keep], A::Committed)
    }
    assert_eq!(s.on.observe(Seat::P0).unwrap().view.hand_counts, [7, 7]);
    assert_eq!(
        s.on.observe(Seat::P0).unwrap().view.library_counts,
        [33, 33]
    );
    for turn in 1u64..=27 {
        let seat = if turn % 2 == 1 { Seat::P0 } else { Seat::P1 };
        s.until(turn, "precombat_main");
        let card = s.card(seat, "forest");
        s.send(seat, "priority", vec![C::PlayLand { card }], A::Committed);
        if turn == 3 || turn == 4 || turn == 6 {
            cast_cub(&mut s, seat)
        }
        if turn == 7 {
            growth(&mut s)
        }
        if turn >= 7 && turn % 2 == 1 {
            let cub = board(&s, Seat::P0, "bear-cub");
            s.until(turn, "declare_attackers");
            let o = s.on.observe(Seat::P0).unwrap();
            let d = o.decision.unwrap();
            assert_eq!(d.candidates, vec![C::FinishCombat]);
            assert_eq!(d.legal_mask, vec![true]);
            assert_eq!(d.factored.unwrap().attackers, vec![cub]);
            // Independently enumerate all 2^1 subsets, then choose the attacker.
            for cards in [vec![], vec![cub]] {
                s.send(
                    Seat::P0,
                    "attackers",
                    vec![C::SelectAttackers { cards }],
                    A::Continuing,
                )
            }
            s.send(Seat::P0, "attackers", vec![C::FinishCombat], A::Committed);
            s.idle();
            s.idle();
            if turn == 7 {
                let o = s.on.observe(Seat::P1).unwrap();
                let blockers: Vec<_> = o
                    .view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .unwrap()
                    .cards
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.card == "bear-cub" && c.controller == 1)
                    .map(|(row, _)| bf(row))
                    .collect();
                assert_eq!(blockers.len(), 2);
                let blocker = blockers[0];
                let second = blockers[1];
                let domain =
                    s.on.observe(Seat::P1)
                        .unwrap()
                        .decision
                        .unwrap()
                        .factored
                        .unwrap();
                assert_eq!(domain.attackers, vec![cub]);
                assert_eq!(domain.blockers, blockers);
                for blocks in [
                    vec![],
                    vec![(blocker, cub)],
                    vec![(second, cub)],
                    vec![(blocker, cub), (second, cub)],
                ] {
                    s.send(
                        Seat::P1,
                        "blockers",
                        vec![C::SelectBlockers { blocks }],
                        A::Continuing,
                    )
                }
                s.send(Seat::P1, "blockers", vec![C::FinishCombat], A::Committed);
                s.idle();
                s.idle();
                let d = s.on.observe(Seat::P0).unwrap().decision.unwrap();
                assert_eq!(d.kind, "combat_damage");
                let allocation = &d.factored.unwrap().damage[0];
                assert_eq!(
                    (
                        allocation.attacker,
                        allocation.power,
                        allocation.blockers.clone()
                    ),
                    (cub, 5, vec![blocker, second])
                );
                // All six nonnegative sum-5 allocations are independently enumerated.
                for n in 0..=5 {
                    s.send(
                        Seat::P0,
                        "combat_damage",
                        vec![C::AssignDamage {
                            attacker: cub,
                            amounts: vec![(blocker, n), (second, 5 - n)],
                        }],
                        A::Continuing,
                    );
                }
                s.send(
                    Seat::P0,
                    "combat_damage",
                    vec![C::AssignDamage {
                        attacker: cub,
                        amounts: vec![(blocker, 2), (second, 3)],
                    }],
                    A::Continuing,
                );
                s.send(
                    Seat::P0,
                    "combat_damage",
                    vec![C::FinishCombat],
                    A::Committed,
                );
                // Each blocker has one recipient: its two damage is automatic.
                // CR 510: simultaneous 4 damage leaves the 5/5 alive; both 2/2s die.
                let o = s.on.observe(Seat::P0).unwrap();
                let cards = &o
                    .view
                    .public_zones
                    .iter()
                    .find(|z| z.zone == "battlefield")
                    .unwrap()
                    .cards;
                assert_eq!(cards.iter().filter(|c| c.card == "bear-cub").count(), 1);
                assert_eq!(
                    cards
                        .iter()
                        .find(|c| c.card == "bear-cub")
                        .unwrap()
                        .creature,
                    Some([5, 5, 4])
                );
                assert_eq!(s.on.observe(Seat::P0).unwrap().view.life, [20, 20]);
            } else {
                s.send(Seat::P1, "blockers", vec![C::FinishCombat], A::Committed);
                let expected = 20 - ((turn - 7) / 2 * 2) as i64;
                for _ in 0..12 {
                    if s.on.observe(Seat::P0).unwrap().view.life == [20, expected] {
                        break;
                    }
                    s.idle()
                }
                assert_eq!(s.on.observe(Seat::P0).unwrap().view.life, [20, expected]);
            }
        }
    }
    s.check();
    let final_views = [
        s.on.observe(Seat::P0).unwrap(),
        s.on.observe(Seat::P1).unwrap(),
    ];
    let result = s.on.finish().unwrap();
    assert_eq!(
        result.status(),
        Status::Completed(mtg_core::game::terminal::Outcome {
            winner: Some(Seat::P0),
            losses: [None, Some(mtg_core::game::terminal::LossReason::Life)],
        })
    );
    check_seats(&result, &s.rows, [1, -1]);
    let e = result.trajectory().unwrap();
    let f = e.footer().unwrap();
    assert_eq!(f.boundary_reward, [0, 0]);
    assert_eq!(f.cancelled_actions, 2);
    assert_eq!(f.logical_actions as u64, s.logical);
    assert_eq!(f.final_observations, final_views);
    assert_eq!(e.decisions().iter().map(|d| d.reward[0]).sum::<i8>(), 1);
    assert_eq!(e.decisions().iter().map(|d| d.reward[1]).sum::<i8>(), -1);
    assert!(s.on.finish().is_err());
    assert!(s.on.submit(Seat::P0, &s.rows[0].sub).is_err());
    let owned = e.clone();
    let old = s.rows[0].sub.clone();
    let mut h = header();
    h.id.ordinal = 1;
    s.on.reset_captured(&config(), 99, 1, NonZeroUsize::MAX, &h)
        .unwrap();
    assert!(s.on.submit(Seat::P0, &old).is_err());
    assert_eq!(result.trajectory(), Some(&owned));
    // Actual accepted semantic stream is an additional completeness/replay check.
    let mut replay = mtg_core::game::Game::new().unwrap();
    replay.reset(&config(), 160, 0).unwrap();
    for record in result.privileged_history() {
        mtg_core::game::actions::apply(&mut replay, record, CAP).unwrap();
        if replay.decision().is_none()
            && replay.turn_position().is_none()
            && replay.outcome().is_none()
        {
            replay.start_turns().unwrap();
        }
    }
    assert_eq!(
        normalized(&replay.snapshot()),
        normalized(result.privileged_snapshot())
    );

    check_authorized_replay(result.clone());
    result
}
#[test]
fn multi_bottom_and_hidden_hand_library_twins() {
    let mut s = Script::new(&config());
    s.send(
        Seat::P0,
        "keep_or_mulligan",
        vec![C::Mulligan],
        A::Committed,
    );
    s.send(Seat::P1, "keep_or_mulligan", vec![C::Keep], A::Committed);
    s.send(
        Seat::P0,
        "bottom",
        vec![C::Bottom { card: hand(0) }],
        A::Committed,
    );
    s.send(
        Seat::P0,
        "keep_or_mulligan",
        vec![C::Mulligan],
        A::Committed,
    );
    let d = s.on.observe(Seat::P0).unwrap().decision.unwrap();
    assert_eq!(d.count, 2);
    assert_eq!(
        d.candidates,
        (0..7)
            .map(|row| C::Bottom { card: hand(row) })
            .collect::<Vec<_>>()
    );
    assert_eq!(d.legal_mask, vec![true; 7]);
    let snapshot = s.on.privileged_snapshot();
    let count = s.on.trajectory().unwrap().decisions().len();
    let short = policy::Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation,
        choices: vec![C::Bottom { card: hand(5) }],
    };
    assert!(s.on.submit(Seat::P0, &short).is_err());
    assert_eq!(s.on.privileged_snapshot(), snapshot);
    assert_eq!(s.on.trajectory().unwrap().decisions().len(), count);
    s.send(
        Seat::P0,
        "bottom",
        vec![C::Bottom { card: hand(5) }, C::Bottom { card: hand(1) }],
        A::Committed,
    );
    assert_eq!(s.on.observe(Seat::P0).unwrap().view.hand_counts, [5, 7]);
    s.send(Seat::P0, "keep_or_mulligan", vec![C::Keep], A::Committed);
    let (r, rows) = s.finish_concession(Seat::P1);
    check_seats(&r, &rows, [1, -1]);
    let mut records = vec![];
    let mut hands = vec![];
    for hidden in [false, true] {
        let mut c = config();
        if hidden {
            let order = c.seats[1].order.as_mut().unwrap();
            order.swap(0, 23);
            order[10..].reverse();
        }
        let mut s = Script::new(&c);
        hands.push(s.on.observe(Seat::P1).unwrap().view.hand);
        s.send(Seat::P0, "keep_or_mulligan", vec![C::Keep], A::Committed);
        s.send(Seat::P1, "keep_or_mulligan", vec![C::Keep], A::Committed);
        for turn in [1, 3] {
            s.until(turn, "precombat_main");
            let card = s.card(Seat::P0, "forest");
            s.send(
                Seat::P0,
                "priority",
                vec![C::PlayLand { card }],
                A::Committed,
            )
        }
        cast_cub(&mut s, Seat::P0);
        s.until(5, "precombat_main");
        growth(&mut s);
        let (r, _) = s.finish_concession(Seat::P1);
        records.push(r.trajectory().unwrap().seat(Seat::P0).unwrap());
    }
    assert_ne!(hands[0], hands[1]);
    assert_eq!(records[0], records[1]);
}
#[test]
fn direct_submit_cannot_bypass_capture_and_invalid_stats_headers_are_atomic() {
    let mut s = Script::new(&Config::default()); // Real shuffle, not ordered fixture.
    let o = s.on.observe(Seat::P0).unwrap();
    let d = o.decision.unwrap();
    let sub = policy::Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation,
        choices: vec![C::Keep],
    };
    let snapshot = s.on.privileged_snapshot();
    let before = s.on.trajectory().cloned();
    for info in [
        PolicyInfo {
            log_probability: Some(f64::NAN),
            ..PolicyInfo::default()
        },
        PolicyInfo {
            log_probability: Some(0.1),
            ..PolicyInfo::default()
        },
        PolicyInfo {
            value: Some(f64::INFINITY),
            ..PolicyInfo::default()
        },
    ] {
        assert!(s.on.submit_with_policy(Seat::P0, &sub, &info).is_err());
        assert_eq!(s.on.privileged_snapshot(), snapshot);
        assert_eq!(s.on.trajectory(), before.as_ref());
    }
    s.on.submit(Seat::P0, &sub).unwrap();
    s.off.submit(Seat::P0, &sub).unwrap();
    assert_eq!(
        normalized(&s.on.privileged_snapshot()),
        normalized(&s.off.privileged_snapshot())
    );
    assert_eq!(s.on.trajectory().unwrap().decisions().len(), 1);
    assert_eq!(
        s.on.trajectory().unwrap().decisions()[0].choice.policy,
        PolicyInfo::default()
    );
    let saved = s.on.trajectory().unwrap().clone();
    let snapshot = s.on.privileged_snapshot();
    assert!(s.on.submit(Seat::P0, &sub).is_err());
    assert_eq!(s.on.privileged_snapshot(), snapshot);
    assert_eq!(s.on.trajectory(), Some(&saved));
    s.on.finish().unwrap();
    let mut bad = header();
    bad.versions.schema = 99;
    assert!(
        s.on.reset_captured(&config(), 1, 0, NonZeroUsize::MAX, &bad)
            .is_err()
    );
    assert_eq!(s.on.privileged_snapshot(), snapshot);
    assert_eq!(s.on.trajectory(), Some(&saved));
    bad = header();
    bad.id.ordinal = 1;
    assert!(
        s.on.reset_captured(&config(), 1, 0, NonZeroUsize::MAX, &bad)
            .is_err()
    );
    assert_eq!(s.on.trajectory(), Some(&saved));
    s.on.reset_captured(&config(), 1, 0, NonZeroUsize::MIN, &header())
        .unwrap();
    let unfinished = s.on.finish().unwrap();
    assert_eq!(unfinished.status(), Status::Incomplete);
    assert!(unfinished.capture_requested());
    assert!(unfinished.trajectory().is_none());
}
#[test]
fn next_domain_capacity_overflow_cannot_erase_accepted_opening_decisions() {
    let mut on = Driver::new(2).unwrap();
    let mut off = Driver::new(2).unwrap();
    on.reset_captured(&config(), 160, 0, NonZeroUsize::MAX, &header())
        .unwrap();
    off.reset(&config(), 160, 0, NonZeroUsize::MAX).unwrap();
    for seat in [Seat::P0, Seat::P1] {
        let d = on.observe(seat).unwrap().decision.unwrap();
        let sub = policy::Submission {
            schema_version: 1,
            revision: d.revision,
            generation: d.generation,
            choices: vec![C::Keep],
        };
        on.submit(seat, &sub).unwrap();
        off.submit(seat, &sub).unwrap();
    }
    settle(&mut on);
    settle(&mut off);
    assert_eq!(
        normalized(&on.privileged_snapshot()),
        normalized(&off.privileged_snapshot())
    );
    assert_eq!(on.trajectory().unwrap().decisions().len(), 2);
    assert!(on.observe(Seat::P0).is_err());
    assert!(off.observe(Seat::P0).is_err());
    let r = on.finish().unwrap();
    assert_eq!(r.status(), Status::Incomplete);
    assert!(r.trajectory().unwrap().footer().is_none());
    assert_eq!(r.trajectory().unwrap().decisions().len(), 2);
}

// GH-162 acceptance extends the actual normal-reset combat script above. Its
// opening 7/33, paid Cub, Growth 5/5 with 4 damage, and ten 2-damage attacks
// are literal CR-derived checkpoints, not generated replay expectations.
fn check_authorized_replay(mut result: mtg_core::episode::EpisodeResult) {
    use mtg_core::episode::replay::{Availability, Error, Registry};
    let mut registry = Registry::default();
    let id = "45b374c9-a4db-44fe-a681-603c4055708a";
    assert_eq!(
        registry.register(id, &mut result),
        Ok(Availability::Available(id.into()))
    );
    assert_eq!(
        result
            .trajectory()
            .unwrap()
            .header()
            .restricted_replay
            .as_deref(),
        Some(id)
    );
    assert_eq!(
        registry.resolve(id, &result, |_, _| false),
        Err(Error::Denied)
    );
    assert_eq!(
        registry.resolve("unknown", &result, |_, _| true),
        Err(Error::Unknown)
    );
    let key = result.trajectory().unwrap().header().id.clone();
    let bytes = registry
        .resolve(id, &result, |artifact, episode| {
            artifact == id && *episode == key
        })
        .unwrap();
    let game = mtg_core::opening::replay::played::verify(bytes).unwrap();
    assert_eq!(
        normalized(&game.snapshot()),
        normalized(result.privileged_snapshot())
    );
    let payload: Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(payload["initial"]["life"], json!([20, 20]));
    assert_eq!(
        payload["choices"].as_array().unwrap().last().unwrap()["after"]["life"],
        json!([20, 0])
    );
    assert!(payload["choices"].as_array().unwrap().iter().any(|r| {
        r["after"]["zones"].as_array().unwrap().iter().any(|z| {
            z["objects"]
                .as_array()
                .unwrap()
                .iter()
                .any(|o| o["creature"] == json!({"power":5,"toughness":5,"damage":4}))
        })
    }));
    for seat in [Seat::P0, Seat::P1] {
        let reader =
            serde_json::to_string(&result.trajectory().unwrap().seat(seat).unwrap()).unwrap();
        for secret in [
            "master",
            "rng",
            "privileged_history",
            "mtg-core-played-replay",
        ] {
            assert!(!reader.contains(secret));
        }
    }
    let mut corrupt = payload;
    corrupt["choices"][0]["after"]["life"] = json!([19, 20]);
    assert!(
        mtg_core::opening::replay::played::verify(&serde_json::to_vec(&corrupt).unwrap()).is_err()
    );
    registry.remove(id).unwrap();
    assert_eq!(
        registry.resolve(id, &result, |_, _| true),
        Err(Error::Unknown)
    );
    assert!(registry.register(id, &mut result).is_err());
    assert_eq!(
        Registry::default().resolve(id, &result, |_, _| true),
        Err(Error::Unknown)
    );
}
