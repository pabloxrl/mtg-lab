//! R0002-B016/B025: original restore contracts, CR 103.5 opening ledger.
use mtg_core::opening::snapshot::RestoreError;
use mtg_core::{
    objects::{Seat, Zone},
    opening::*,
};
use std::num::NonZeroUsize;
fn q() -> NonZeroUsize {
    NonZeroUsize::new(1).unwrap()
}
fn choose(g: &mut Game, index: usize) {
    let d = g.decision().unwrap();
    g.apply(
        d.actor,
        &OpeningAction {
            decision: d.id,
            selection: Selection::Choose(d.candidate(index)),
        },
    )
    .unwrap();
}
fn ordered() -> Config {
    let m: serde_json::Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    let mut c = Config::default();
    for (i, d) in m["decks"].as_array().unwrap().iter().enumerate() {
        let mut order = Vec::new();
        for e in d["cards"].as_array().unwrap() {
            order.extend(std::iter::repeat_n(
                e["card_id"].as_str().unwrap().to_owned(),
                e["copies"].as_u64().unwrap() as usize,
            ));
        }
        order.reverse();
        c.seats[i] = DeckConfig {
            deck: d["id"].as_str().unwrap().into(),
            order: Some(order),
        };
    }
    c
}
fn cards(g: &Game, z: Zone) -> Vec<String> {
    g.objects()
        .in_zone(z)
        .map(|h| g.objects().get(h).unwrap().card.identity().key.to_owned())
        .collect()
}
#[test]
fn snapshot_literal_corruption_is_atomic() {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 42, 9).unwrap();
    for bytes in [b"".as_slice(), b"not a snapshot", b"{}", b"{\"version\":1}"] {
        let before = format!("{g:?}");
        assert_eq!(g.restore(bytes), Err(RestoreError::Corrupt));
        assert_eq!(format!("{g:?}"), before);
    }
}
#[test]
fn snapshot_pending_mulligan_restores_expected_choices_and_cards() {
    let c = ordered();
    let mut g = Game::new().unwrap();
    g.reset(&c, 42, 9).unwrap();
    choose(&mut g, 1); // P0 declaration retained while P1 chooses.
    let bytes = g.snapshot();
    let mut restored = Game::new().unwrap();
    restored.restore(&bytes).unwrap();
    let d = restored.decision().expect("restored pending P1 decision");
    assert_eq!(d.actor, Seat::P1);
    assert_eq!(d.kind, OpeningKind::KeepOrMulligan);
    assert_eq!(d.candidates, [OpeningChoice::Keep, OpeningChoice::Mulligan]);
    assert_eq!(
        cards(&restored, Zone::Hand(Seat::P0)),
        c.seats[0].order.as_ref().unwrap()[..7]
    );
    choose(&mut restored, 0);
    let d = restored.decision().unwrap();
    assert_eq!(d.actor, Seat::P0);
    assert_eq!(d.kind, OpeningKind::Bottom { count: 1 });
    // Future RNG comparison supplements the independent CR/card ledger above.
    choose(&mut g, 0);
    assert_eq!(
        cards(&restored, Zone::Hand(Seat::P0)),
        cards(&g, Zone::Hand(Seat::P0))
    );
}
#[test]
fn snapshot_reset_yield_resumes_to_independent_ordered_ledger() {
    let c = ordered();
    let mut source = Game::new().unwrap();
    assert_eq!(
        source.reset_quantum(&c, 42, 9, q()).unwrap(),
        Progress::InternalYield
    );
    let mut restored = Game::new().unwrap();
    restored.restore(&source.snapshot()).unwrap();
    let mut progress = restored.resume(NonZeroUsize::MAX);
    if progress == Progress::InternalYield {
        progress = restored.resume(NonZeroUsize::MAX);
    }
    let Progress::Decision(d) = progress else {
        panic!("expected opening decision: {progress:?}")
    };
    assert_eq!(d.actor, Seat::P0);
    assert_eq!(restored.life(), [20, 20]);
    for (i, s) in [Seat::P0, Seat::P1].into_iter().enumerate() {
        let order = c.seats[i].order.as_ref().unwrap();
        assert_eq!(cards(&restored, Zone::Hand(s)), order[..7]);
        assert_eq!(cards(&restored, Zone::Library(s)), order[7..]);
    }
}

fn reload(g: &mut Game) {
    let saved = g.snapshot();
    let before = normalized(&saved);
    g.restore(&saved).unwrap();
    assert_eq!(
        normalized(&g.snapshot()),
        before,
        "all serialized fields except fresh capability scope"
    );
}
fn normalized(bytes: &[u8]) -> serde_json::Value {
    let envelope: serde_json::Value = serde_json::from_slice(bytes).unwrap();
    let mut payload: serde_json::Value =
        serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
    fn scopes(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                for (k, v) in m {
                    if k == "store" || k == "scope" {
                        *v = 0.into();
                    } else {
                        scopes(v);
                    }
                }
            }
            serde_json::Value::Array(a) => {
                for v in a {
                    scopes(v);
                }
            }
            _ => {}
        }
    }
    scopes(&mut payload);
    payload["objects"]["id"] = 0.into();
    payload
}
fn drain_with_snapshots(g: &mut Game, mut p: Progress) {
    let mut count = 0;
    while p == Progress::InternalYield {
        count += 1;
        assert!(count < 500);
        reload(g);
        p = g.resume(q());
    }
    reload(g);
}
fn choose_yield(g: &mut Game, index: usize) {
    let d = g.decision().unwrap();
    let p = g
        .apply_quantum(
            d.actor,
            &OpeningAction {
                decision: d.id,
                selection: Selection::Choose(d.candidate(index)),
            },
            None,
            q(),
        )
        .unwrap();
    drain_with_snapshots(g, p);
}
fn bottom_yield(g: &mut Game, indices: &[usize]) {
    let d = g.decision().unwrap();
    assert_eq!(
        d.kind,
        OpeningKind::Bottom {
            count: indices.len()
        }
    );
    let p = g
        .apply_quantum(
            d.actor,
            &OpeningAction {
                decision: d.id,
                selection: Selection::Bottom(indices.iter().map(|&i| d.candidate(i)).collect()),
            },
            None,
            q(),
        )
        .unwrap();
    drain_with_snapshots(g, p);
}
#[test]
fn snapshot_every_random_reset_redraw_bottom_yield_matches_independent_vectors() {
    // Reuse the independently authored Python arbitrary-precision PRNG/CR ledger
    // committed by GH-65; no expected cards generated by the Rust game.
    let vectors: serde_json::Value =
        serde_json::from_str(include_str!("../../../doc/evidence/mulligan/vectors.json")).unwrap();
    for vector in vectors.as_array().unwrap() {
        let c = Config {
            starting_seat: vector["starter"].as_u64().unwrap() as u8,
            ..Config::default()
        };
        let mut g = Game::new().unwrap();
        let p = g.reset_quantum(&c, 42, 9, q()).unwrap();
        drain_with_snapshots(&mut g, p);
        choose_yield(&mut g, 1);
        choose_yield(&mut g, 1);
        bottom_yield(&mut g, &[6]);
        bottom_yield(&mut g, &[6]);
        for checkpoint in ["first", "second"] {
            if checkpoint == "second" {
                choose_yield(&mut g, 1);
                choose_yield(&mut g, 0);
                bottom_yield(&mut g, &[5, 1]);
            }
            for (i, s) in [Seat::P0, Seat::P1].into_iter().enumerate() {
                for (field, z) in [("hands", Zone::Hand(s)), ("libraries", Zone::Library(s))] {
                    let expected: Vec<_> = vector[checkpoint][field][i]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .collect();
                    assert_eq!(cards(&g, z), expected, "{checkpoint}/{field}/{i}");
                }
            }
        }
    }
}
#[test]
fn snapshot_versions_corruption_and_truncation_preserve_destination() {
    let mut g = Game::new().unwrap();
    g.reset(&ordered(), 1, 2).unwrap();
    let good = g.snapshot();
    let before = format!("{g:?}");
    let envelope: serde_json::Value = serde_json::from_slice(&good).unwrap();
    for version in [0, 2, 4294967295u32] {
        let mut bad = envelope.clone();
        bad["version"] = version.into();
        assert_eq!(
            g.restore(&serde_json::to_vec(&bad).unwrap()),
            Err(RestoreError::UnsupportedVersion)
        );
        assert_eq!(format!("{g:?}"), before);
    }
    let mut bad = envelope.clone();
    bad["engine"] = "future-core".into();
    assert_eq!(
        g.restore(&serde_json::to_vec(&bad).unwrap()),
        Err(RestoreError::IncompatibleEngine)
    );
    for n in [0, 1, good.len() / 2, good.len() - 1] {
        assert_eq!(g.restore(&good[..n]), Err(RestoreError::Corrupt));
        assert_eq!(format!("{g:?}"), before);
    }
    for (field, replacement) in [
        ("format", "other"),
        ("sha256", "0000"),
        ("payload", "{}"),
        ("payload", "null"),
    ] {
        let mut bad = envelope.clone();
        bad[field] = replacement.into();
        assert_eq!(
            g.restore(&serde_json::to_vec(&bad).unwrap()),
            Err(RestoreError::Corrupt)
        );
        assert_eq!(format!("{g:?}"), before);
    }
    let mut trailing = good.clone();
    trailing.extend_from_slice(b"null");
    assert_eq!(g.restore(&trailing), Err(RestoreError::Corrupt));
    assert_eq!(format!("{g:?}"), before);
}
#[test]
fn snapshot_fresh_capabilities_preserve_generations_and_reject_old_actions() {
    let mut source = Game::new().unwrap();
    source.reset(&ordered(), 1, 2).unwrap();
    source.reset(&ordered(), 3, 4).unwrap(); // nonzero reset epoch/decision generation
    let old = source
        .objects()
        .in_zone(Zone::Hand(Seat::P0))
        .next()
        .unwrap();
    let d = source.decision().unwrap();
    let bytes = source.snapshot();
    let mut destination = Game::new().unwrap();
    destination.reset(&ordered(), 8, 9).unwrap();
    let foreign = destination
        .objects()
        .in_zone(Zone::Hand(Seat::P0))
        .next()
        .unwrap();
    destination.restore(&bytes).unwrap();
    assert_eq!(normalized(&destination.snapshot()), normalized(&bytes));
    for h in [old, foreign] {
        assert!(destination.objects().get(h).is_err());
    }
    let before = format!("{destination:?}");
    assert_eq!(
        destination.apply(
            d.actor,
            &OpeningAction {
                decision: d.id,
                selection: Selection::Choose(d.candidate(0))
            }
        ),
        Err(ApplyError::StaleDecision)
    );
    assert_eq!(format!("{destination:?}"), before);
    assert_eq!(destination.decision().unwrap().generation, d.generation);
    choose(&mut destination, 0);
    choose(&mut destination, 0);
    assert_eq!(source.decision(), Some(d)); // restore and advance never mutate source
    assert_eq!(
        source.objects().get(old).unwrap().zone,
        Zone::Hand(Seat::P0)
    );
    destination.reset(&ordered(), 3, 4).unwrap();
    assert!(destination.decision().unwrap().generation > d.generation);
}
#[test]
fn snapshot_unstarted_turns_knowledge_terminal_and_reset() {
    use mtg_core::opening::{
        terminal::LossReason,
        turns::{TurnAction, TurnSelection},
    };
    let mut g = Game::new().unwrap();
    reload(&mut g);
    assert_eq!(g.resume(q()), Progress::NotStarted);
    g.reset(&ordered(), 42, 9).unwrap();
    choose(&mut g, 0);
    choose(&mut g, 0);
    let h = g.objects().in_zone(Zone::Hand(Seat::P0)).next().unwrap();
    g.privileged_reveal_to(Seat::P1, h).unwrap();
    let observation = g.observe(Seat::P1).unwrap();
    reload(&mut g);
    // Observation IDs are scoped; compare the independently relevant revelation.
    assert_eq!(
        g.observe(Seat::P1).unwrap().remembered,
        observation.remembered
    );
    let d = g.start_turns().unwrap();
    g.apply_turn(
        d.actor,
        &TurnAction {
            decision: d.id,
            selection: TurnSelection::Pass(d.candidate(0)),
        },
    )
    .unwrap();
    reload(&mut g);
    assert_eq!(g.turn_decision().unwrap().actor, Seat::P1);
    let outcome = g.concede(Seat::P1, g.episode_id().unwrap()).unwrap();
    reload(&mut g);
    assert_eq!(outcome.winner, Some(Seat::P0));
    assert_eq!(outcome.losses, [None, Some(LossReason::Concession)]);
    assert_eq!(g.outcome(), Some(outcome));
    assert_eq!(g.resume(q()), Progress::Terminal(outcome));
    g.reset(&ordered(), 0, 0).unwrap();
    reload(&mut g);
    assert_eq!(g.life(), [20, 20]);
    assert_eq!(g.outcome(), None);
}

#[test]
fn snapshot_pending_explicit_chance_order_survives_all_seven_rounds() {
    let c = ordered();
    let mut expected = c.seats[0].order.clone().unwrap();
    let mut g = Game::new().unwrap();
    g.reset(&c, 42, 9).unwrap();
    for round in 1..=7 {
        let d = g.decision().unwrap();
        assert_eq!(d.actor, Seat::P0);
        let mut order = g
            .objects()
            .in_zone(Zone::Hand(Seat::P0))
            .chain(g.objects().in_zone(Zone::Library(Seat::P0)))
            .collect::<Vec<_>>();
        order.reverse();
        expected.reverse();
        let p = g
            .apply_quantum(
                d.actor,
                &OpeningAction {
                    decision: d.id,
                    selection: Selection::Choose(d.candidate(1)),
                },
                Some(&order),
                q(),
            )
            .unwrap();
        drain_with_snapshots(&mut g, p);
        if round == 1 {
            choose_yield(&mut g, 0);
        }
        assert_eq!(cards(&g, Zone::Hand(Seat::P0)), expected[..7]);
        let indices: Vec<_> = (0..round).rev().collect();
        bottom_yield(&mut g, &indices);
        let hand = expected[round..7].to_vec();
        let mut library = expected[7..].to_vec();
        library.extend(indices.iter().map(|&i| expected[i].clone()));
        assert_eq!(cards(&g, Zone::Hand(Seat::P0)), hand);
        assert_eq!(cards(&g, Zone::Library(Seat::P0)), library);
        expected = hand.into_iter().chain(library).collect();
    }
    assert_eq!(g.decision(), None);
    assert_eq!(g.resume(q()), Progress::OpeningComplete);
}
#[test]
fn snapshot_cross_process_restore() {
    const INPUT: &str = "MTG_SNAPSHOT_TEST_INPUT";
    if let Ok(path) = std::env::var(INPUT) {
        let mut g = Game::new().unwrap();
        // Fresh process starts its store counter at zero; restored scopes must
        // still be isolated from subsequently created games.
        g.restore(&std::fs::read(path).unwrap()).unwrap();
        let c = ordered();
        assert_eq!(g.decision().unwrap().actor, Seat::P1);
        assert_eq!(
            cards(&g, Zone::Hand(Seat::P0)),
            c.seats[0].order.as_ref().unwrap()[..7]
        );
        let h = g.objects().in_zone(Zone::Hand(Seat::P0)).next().unwrap();
        let mut other = Game::new().unwrap();
        other.reset(&c, 42, 9).unwrap();
        assert!(other.objects().get(h).is_err());
        choose(&mut g, 0);
        assert_eq!(g.decision().unwrap().kind, OpeningKind::Bottom { count: 1 });
        return;
    }
    let mut g = Game::new().unwrap();
    g.reset(&ordered(), 42, 9).unwrap();
    choose(&mut g, 1);
    let path = std::env::temp_dir().join(format!("mtg-snapshot-{}.json", std::process::id()));
    std::fs::write(&path, g.snapshot()).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "snapshot_cross_process_restore", "--nocapture"])
        .env(INPUT, &path)
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn snapshot_structural_corruption_with_recomputed_digest_is_atomic() {
    use sha2::{Digest, Sha256};
    let mut g = Game::new().unwrap();
    g.reset(&ordered(), 42, 9).unwrap();
    let envelope: serde_json::Value = serde_json::from_slice(&g.snapshot()).unwrap();
    let payload: serde_json::Value =
        serde_json::from_str(envelope["payload"].as_str().unwrap()).unwrap();
    let before = format!("{g:?}");
    for path in ["card", "zone", "scope", "candidate", "mulligans", "extra"] {
        let mut p = payload.clone();
        match path {
            "card" => p["objects"]["slots"][0]["object"]["card"] = 255.into(),
            "zone" => p["objects"]["zones"][0][0] = 9999.into(),
            "scope" => p["decision"]["id"]["scope"] = u64::MAX.into(),
            "candidate" => p["decision"]["candidates"] = serde_json::json!(["Mulligan"]),
            "mulligans" => p["mulligans"][0] = 8.into(),
            "extra" => p["unknown"] = true.into(),
            _ => unreachable!(),
        }
        let text = serde_json::to_string(&p).unwrap();
        let mut bad = envelope.clone();
        bad["sha256"] = format!("{:x}", Sha256::digest(text.as_bytes())).into();
        bad["payload"] = text.into();
        assert_eq!(
            g.restore(&serde_json::to_vec(&bad).unwrap()),
            Err(RestoreError::Corrupt),
            "{path}"
        );
        assert_eq!(format!("{g:?}"), before);
    }
}
