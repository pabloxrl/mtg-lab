//! Explicitly synthetic cleanup entry. CR 514.1–3/704: discard, expire,
//! check SBAs/triggers, exceptional priority, then repeat cleanup in that order.
use super::*;
use turns::{Step, TurnAction, TurnKind, TurnSelection};

fn ready(active: Seat, pending: bool) -> Game {
    let mut g = Game::new().unwrap();
    g.reset(
        &Config {
            starting_seat: seat_index(active) as u8,
            ..Config::default()
        },
        207,
        0,
    )
    .unwrap();
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
    g.turns.position = Some((1, active, Step::End));
    g.set_turn_decision(turns::opponent(active), TurnKind::Priority);
    g.turns.passed = true;
    for seat in [Seat::P0, Seat::P1] {
        g.draw_top(seat).unwrap();
    }
    let card = CardId::from_key("viashino-pyromancer").unwrap();
    let source = g.objects.allocate(card, active, Zone::Battlefield).unwrap();
    g.turns.modifications.push(targets::Modification {
        handle: source,
        boost: 3,
        power_boost: 0,
        damage: 3,
    });
    if pending {
        // No public injection hook: this is the declared CR 514.3a synthetic case.
        g.turns
            .pending_triggers
            .push(Some(triggers::PendingTrigger {
                source,
                card,
                controller: active,
                kind: triggers::TriggerKind::Pyromancer { target: None },
            }));
    }
    g
}
fn drain(g: &mut Game, mut p: Progress, q: NonZeroUsize) {
    let mut units = 0;
    while p == Progress::InternalYield {
        units += 1;
        assert!(units < 100);
        assert!(g.turn_decision().is_none());
        let s = g.snapshot();
        g.restore(&s).unwrap();
        p = g.resume(q);
    }
    let s = g.snapshot();
    g.restore(&s).unwrap();
}
fn act(g: &mut Game, discard: bool, q: NonZeroUsize) {
    let d = g.turn_decision().unwrap();
    let p = g
        .apply_turn_quantum(
            d.actor,
            &TurnAction {
                decision: d.id,
                selection: if discard {
                    TurnSelection::Discard(vec![d.candidate(0)])
                } else {
                    TurnSelection::Pass(d.candidate(0))
                },
            },
            q,
        )
        .unwrap();
    drain(g, p, q);
}
fn place(g: &mut Game, active: Seat, q: NonZeroUsize) {
    let d = g.turn_decision().unwrap();
    assert_eq!((d.actor, d.kind), (active, TurnKind::TriggerOrder));
    let p = g.order_triggers_quantum(active, d.id, &[0], q).unwrap();
    drain(g, p, q);
    let d = g.turn_decision().unwrap();
    let p = g
        .target_trigger_quantum(active, d.id, turns::opponent(active), q)
        .unwrap();
    drain(g, p, q);
    assert_eq!(g.turn_position(), Some((1, active, Step::Cleanup)));
    assert_eq!(g.turn_decision().unwrap().actor, active);
}
#[test]
fn cleanup_pending_trigger_discards_expires_resolves_and_repeats() {
    for active in [Seat::P0, Seat::P1] {
        for q in [
            NonZeroUsize::MIN,
            NonZeroUsize::new(3).unwrap(),
            NonZeroUsize::MAX,
        ] {
            let mut g = ready(active, true);
            act(&mut g, false, q);
            assert_eq!(
                g.turn_decision().unwrap().kind,
                TurnKind::Discard { count: 1 }
            );
            assert_eq!(g.turns.modifications.len(), 1, "514.1 precedes 514.2");
            let d = g.turn_decision().unwrap();
            let before = g.snapshot();
            for (seat, cs) in [
                (active, vec![]),
                (active, vec![d.candidate(0), d.candidate(1)]),
                (turns::opponent(active), vec![d.candidate(0)]),
            ] {
                assert!(
                    g.apply_turn_quantum(
                        seat,
                        &TurnAction {
                            decision: d.id,
                            selection: TurnSelection::Discard(cs)
                        },
                        q
                    )
                    .is_err()
                );
                assert_eq!(g.snapshot(), before);
            }
            assert!(
                g.policy_observe(turns::opponent(active), 256)
                    .unwrap()
                    .decision
                    .is_none()
            );
            act(&mut g, true, q);
            assert_eq!(g.objects.in_zone(Zone::Hand(active)).count(), 7);
            assert_eq!(
                g.objects
                    .in_zone(Zone::Hand(turns::opponent(active)))
                    .count(),
                8
            );
            assert!(g.turns.modifications.is_empty());
            let h = g.objects.in_zone(Zone::Battlefield).next().unwrap();
            let c = g.creature_state(h).unwrap();
            assert_eq!((c.power, c.toughness, c.damage), (2, 1, 0));
            assert_eq!(
                g.turn_position(),
                Some((1, active, Step::Cleanup)),
                "CR 514.3a: pending trigger must not escape into next upkeep"
            );
            place(&mut g, active, q);
            act(&mut g, false, q);
            act(&mut g, false, q);
            assert_eq!(g.life[seat_index(turns::opponent(active))], 18);
            assert_eq!(g.turn_position().unwrap().2, Step::Cleanup);
            act(&mut g, false, q);
            act(&mut g, false, q);
            assert_eq!(
                g.turn_position(),
                Some((2, turns::opponent(active), Step::Upkeep))
            );
            assert_eq!(g.objects.in_zone(Zone::Graveyard(active)).count(), 1);
        }
    }
}
#[test]
fn cleanup_ordinary_has_no_priority_and_terminal_stops_before_untap() {
    for terminal in [false, true] {
        let mut g = ready(Seat::P0, false);
        let h = g
            .objects
            .allocate(
                CardId::from_key("mountain").unwrap(),
                Seat::P1,
                Zone::Battlefield,
            )
            .unwrap();
        g.objects.get_mut(h).unwrap().tapped = true;
        act(&mut g, false, NonZeroUsize::MAX);
        if terminal {
            g.life[0] = 0;
        } // Explicit synthetic SBA at cleanup completion.
        act(&mut g, true, NonZeroUsize::MAX);
        if terminal {
            assert!(g.outcome().is_some());
            assert_eq!(
                g.turn_position().unwrap().2,
                Step::Cleanup,
                "CR 704/104: terminal must precede subsequent turn work"
            );
            assert!(
                g.objects
                    .in_zone(Zone::Battlefield)
                    .any(|h| g.objects.get(h).unwrap().tapped)
            );
        } else {
            assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::Upkeep)));
            assert_eq!(g.turn_decision().unwrap().kind, TurnKind::Priority);
        }
        assert_eq!(g.objects.in_zone(Zone::Hand(Seat::P1)).count(), 8);
    }
}

#[test]
fn cleanup_reference_literal_checkpoints() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../fixtures/reference/cleanup.json")).unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../fixtures/reference/cleanup-expectations.json"
    ))
    .unwrap();
    let mut results = serde_json::Map::new();
    for spec in fixture["cases"].as_array().unwrap() {
        let mut g = ready(Seat::P0, spec["trigger"] == true);
        let hs: Vec<_> = g.objects.in_zone(Zone::Hand(Seat::P0)).collect();
        for h in hs {
            g.objects.remove(h).unwrap();
        }
        for i in 0..spec["hand"].as_u64().unwrap() {
            let key = if i == 1 && spec["thrill"] == true {
                "thrill-of-possibility"
            } else {
                "mountain"
            };
            g.objects
                .allocate(
                    CardId::from_key(key).unwrap(),
                    Seat::P0,
                    Zone::Hand(Seat::P0),
                )
                .unwrap();
        }
        for _ in 0..2 {
            g.objects
                .allocate(
                    CardId::from_key("mountain").unwrap(),
                    Seat::P0,
                    Zone::Battlefield,
                )
                .unwrap();
        }
        let point = |g: &Game| {
            let h = g
                .objects
                .in_zone(Zone::Battlefield)
                .find(|h| g.creature_state(*h).is_some())
                .unwrap();
            let c = g.creature_state(h).unwrap();
            serde_json::json!([
                g.objects.in_zone(Zone::Hand(Seat::P0)).count(),
                g.objects.in_zone(Zone::Hand(Seat::P1)).count(),
                g.objects.in_zone(Zone::Graveyard(Seat::P0)).count(),
                g.life[1],
                g.turns.stack.len(),
                c.power,
                c.toughness,
                c.damage
            ])
        };
        let q = NonZeroUsize::MIN;
        let mut points = vec![point(&g)];
        act(&mut g, false, q);
        let d = g.turn_decision().unwrap();
        let TurnKind::Discard { count } = d.kind else {
            panic!("discard required")
        };
        let before = g.snapshot();
        for n in [count - 1, count + 1] {
            assert!(
                g.apply_turn_quantum(
                    d.actor,
                    &TurnAction {
                        decision: d.id,
                        selection: TurnSelection::Discard((0..n).map(|i| d.candidate(i)).collect())
                    },
                    q
                )
                .is_err()
            );
            assert_eq!(g.snapshot(), before);
        }
        let p = g
            .apply_turn_quantum(
                d.actor,
                &TurnAction {
                    decision: d.id,
                    selection: TurnSelection::Discard((0..count).map(|i| d.candidate(i)).collect()),
                },
                q,
            )
            .unwrap();
        drain(&mut g, p, q);
        if spec["trigger"] == true {
            place(&mut g, Seat::P0, q);
            points.push(point(&g));
            act(&mut g, false, q);
            act(&mut g, false, q);
            points.push(point(&g));
            if spec["thrill"] == true {
                let lands: Vec<_> = g
                    .objects
                    .in_zone(Zone::Battlefield)
                    .filter(|h| g.objects.get(*h).unwrap().card.identity().key == "mountain")
                    .collect();
                for h in lands {
                    let d = g.turn_decision().unwrap();
                    g.tap_mana(Seat::P0, d.id, h).unwrap();
                }
                let h = g
                    .objects
                    .in_zone(Zone::Hand(Seat::P0))
                    .find(|h| {
                        g.objects.get(*h).unwrap().card.identity().key == "thrill-of-possibility"
                    })
                    .unwrap();
                let discard = g
                    .objects
                    .in_zone(Zone::Hand(Seat::P0))
                    .find(|c| *c != h)
                    .unwrap();
                let d = g.turn_decision().unwrap();
                let p = g.begin_cast(Seat::P0, d.id, h).unwrap();
                let mut p = g.choose_cast_discard(Seat::P0, p.id, &[discard]).unwrap();
                for _ in 0..2 {
                    p = g.choose_payment(Seat::P0, p.id, mana::Color::Red).unwrap();
                }
                g.finish_cast(Seat::P0, p.id).unwrap();
                points.push(point(&g));
                act(&mut g, false, q);
                act(&mut g, false, q);
                points.push(point(&g));
            }
            act(&mut g, false, q);
            act(&mut g, false, q);
        }
        assert_eq!(g.turn_position(), Some((2, Seat::P1, Step::Upkeep)));
        points.push(point(&g));
        let id = spec["id"].as_str().unwrap();
        assert_eq!(serde_json::json!(points), expected[id], "{id}");
        results.insert(id.into(), serde_json::json!(points));
    }
    if let Ok(path) = std::env::var("MTG_CLEANUP_OUTPUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&results).unwrap()).unwrap();
    }
}

#[test]
fn cleanup_driver_private_choices_typed_capture_and_snapshot_replay() {
    use crate::{
        episode::Driver,
        trajectory::{EpisodeKey, Header, Limits, Versions},
    };
    use policy::{Choice as C, Submission, VisibleRef, VisibleZone};
    let mut expected_history = None;
    let mut expected_trajectory = None;
    for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
        let mut g = ready(Seat::P0, true);
        // Replace an existing hand card with Thrill; retain eight-card entry.
        let h = g.objects.in_zone(Zone::Hand(Seat::P0)).last().unwrap();
        g.objects.remove(h).unwrap();
        g.objects
            .allocate(
                CardId::from_key("thrill-of-possibility").unwrap(),
                Seat::P0,
                Zone::Hand(Seat::P0),
            )
            .unwrap();
        for _ in 0..2 {
            g.objects
                .allocate(
                    CardId::from_key("mountain").unwrap(),
                    Seat::P0,
                    Zone::Battlefield,
                )
                .unwrap();
        }
        let initial = g.snapshot();
        let header = Header {
            id: EpisodeKey {
                run: "20700000-1234-4234-8234-123456789abc".into(),
                ordinal: 0,
            },
            versions: Versions {
                schema: 2,
                engine: "synthetic-component".into(),
                rules: "cr-20260925".into(),
                cards: "pool-v1".into(),
                action: "policy-v1".into(),
                observation: 1,
            },
            deck_hashes: ["a".repeat(64), "b".repeat(64)],
            config_hash: "c".repeat(64),
            policies: ["script".into(), "script".into()],
            starting_seat: 0,
            limits: Limits::default(),
            restricted_replay: None,
        };
        let mut driver = Driver::synthetic_capture_test(g, header);
        let mut cast = false;
        let mut complete = false;
        for _ in 0..100 {
            let actor = if driver.observe(Seat::P0).unwrap().view.acting_seat == Some(0) {
                Seat::P0
            } else {
                Seat::P1
            };
            let o = driver.observe(actor).unwrap();
            if o.view.turn.as_ref().unwrap().0 == 2 {
                assert!(cast);
                assert_eq!(o.view.hand_counts, [7, 8]);
                assert_eq!(o.view.life, [20, 18]);
                complete = true;
                break;
            }
            let d = o.decision.as_ref().unwrap();
            let choices = match d.kind {
                "cleanup_discard" | "cast_discard" => d.candidates[..d.count].to_vec(),
                "trigger_order" => d.candidates.clone(),
                "trigger_target" => vec![C::TargetPlayer { seat: 1 }],
                "priority"
                    if actor == Seat::P0
                        && o.view.turn.as_ref().unwrap().2 == "cleanup"
                        && o.stack.is_empty()
                        && !cast =>
                {
                    cast = true;
                    vec![C::Cast {
                        card: VisibleRef {
                            zone: VisibleZone::Hand,
                            row: o
                                .view
                                .hand
                                .iter()
                                .position(|c| c.card == "thrill-of-possibility")
                                .unwrap(),
                        },
                    }]
                }
                "payment" => {
                    let legal: Vec<_> = d
                        .candidates
                        .iter()
                        .zip(&d.legal_mask)
                        .filter_map(|(c, m)| m.then_some(c))
                        .collect();
                    vec![if legal.contains(&&C::FinishPayment) {
                        C::FinishPayment
                    } else if legal.contains(&&C::Pay { color: 3 }) {
                        C::Pay { color: 3 }
                    } else {
                        (*legal
                            .iter()
                            .find(|c| matches!(c, C::TapMana { .. }))
                            .unwrap())
                        .clone()
                    }]
                }
                "priority" => vec![C::Pass],
                _ => panic!("unexpected {}", d.kind),
            };
            let sub = Submission {
                schema_version: 1,
                revision: d.revision,
                generation: d.generation,
                choices,
            };
            let snapshot = driver.privileged_snapshot();
            let history = driver.privileged_history().to_vec();
            let capture = serde_json::to_value(driver.trajectory()).unwrap();
            assert!(driver.submit(turns::opponent(actor), &sub).is_err());
            if d.kind == "cleanup_discard" {
                assert!(
                    driver
                        .observe(turns::opponent(actor))
                        .unwrap()
                        .decision
                        .is_none()
                );
                let mut bad = sub.clone();
                bad.choices.clear();
                assert!(driver.submit(actor, &bad).is_err());
            }
            assert_eq!(driver.privileged_snapshot(), snapshot);
            assert_eq!(driver.privileged_history(), history);
            assert_eq!(serde_json::to_value(driver.trajectory()).unwrap(), capture);
            driver.submit(actor, &sub).unwrap();
            while driver.advance(q).unwrap() == crate::episode::Progress::InternalYield {}
        }
        assert!(complete);
        let rows = driver.trajectory().unwrap().decisions();
        assert_eq!(
            rows.iter()
                .filter(|r| r.observation.decision.as_ref().unwrap().kind == "cleanup_discard")
                .count(),
            1
        );
        assert_eq!(
            rows.iter()
                .filter(|r| r.observation.decision.as_ref().unwrap().kind == "cast_discard")
                .count(),
            1
        );
        for row in rows {
            assert!(!row.choice.submission.choices.is_empty());
        }
        let capture = serde_json::to_value(driver.trajectory()).unwrap();
        if let Some(expected) = &expected_trajectory {
            assert_eq!(&capture, expected);
        } else {
            expected_trajectory = Some(capture);
        }
        let history = driver.privileged_history().to_vec();
        if let Some(expected) = &expected_history {
            assert_eq!(&history, expected);
        } else {
            expected_history = Some(history.clone());
        }
        // Synthetic snapshot + semantic history, explicitly not a normal-reset replay.
        let mut replay = Game::new().unwrap();
        replay.restore(&initial).unwrap();
        for record in history {
            actions::apply(&mut replay, &record, 256).unwrap();
        }
        assert_eq!(replay.turn_position(), Some((2, Seat::P1, Step::Upkeep)));
        assert_eq!(replay.life(), [20, 18]);
        assert_eq!(
            replay
                .policy_observe(Seat::P0, 256)
                .unwrap()
                .view
                .hand_counts,
            [7, 8]
        );
    }
}

#[test]
fn cleanup_trigger_lethal_stops_all_turn_work() {
    for q in [NonZeroUsize::MIN, NonZeroUsize::MAX] {
        let mut g = ready(Seat::P0, true);
        g.life[1] = 2;
        act(&mut g, false, q);
        act(&mut g, true, q);
        place(&mut g, Seat::P0, q);
        act(&mut g, false, q);
        act(&mut g, false, q);
        assert_eq!(g.outcome().unwrap().winner, Some(Seat::P0));
        assert_eq!(g.turn_position(), Some((1, Seat::P0, Step::Cleanup)));
        assert!(g.work.is_empty());
        assert!(g.turn_decision().is_none());
        let s = g.snapshot();
        assert!(matches!(g.resume(q), Progress::Terminal(_)));
        assert_eq!(g.snapshot(), s);
    }
}
