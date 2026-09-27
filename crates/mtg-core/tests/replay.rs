//! Original R0002-B016/B025 and SYS-REPLAY opening contracts.
//! CR 103.5: seven initial cards, declarations in starting-seat order,
//! each mulligan draws seven then bottoms its count. Card expectations come
//! from the frozen manifest and the independent GH-65 Python RNG vectors.
use mtg_core::{
    objects::{Seat, Zone},
    opening::{replay::*, *},
};
use serde_json::{Value, json};
fn choice(actor: Seat, action: Action) -> Choice {
    Choice { actor, action }
}
fn keeps() -> Vec<Choice> {
    vec![
        choice(Seat::P0, Action::Keep {}),
        choice(Seat::P1, Action::Keep {}),
    ]
}
fn bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
fn recording() -> Value {
    serde_json::from_slice(&record(&Config::default(), 42, 9, &keeps()).unwrap()).unwrap()
}
fn cards(g: &Game, z: Zone) -> Vec<String> {
    g.objects()
        .in_zone(z)
        .map(|h| g.objects().get(h).unwrap().card.identity().key.to_owned())
        .collect()
}
#[test]
fn replay_literal_opening_and_versioned_roundtrip() {
    for starter in 0..2 {
        let c = Config {
            starting_seat: starter,
            ..Config::default()
        };
        let mut k = keeps();
        if starter == 1 {
            k.reverse();
        }
        let b = record(&c, 42, 9, &k).unwrap();
        let g = verify(&b).unwrap();
        assert_eq!(g.life(), [20, 20]);
        assert!(g.decision().is_none());
        for s in [Seat::P0, Seat::P1] {
            assert_eq!(cards(&g, Zone::Hand(s)).len(), 7);
            assert_eq!(cards(&g, Zone::Library(s)).len(), 33);
        }
        let v: Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v["version"], 1);
        assert_eq!(v["choices"][0]["choice"]["action"]["kind"], "Keep");
        assert_eq!(v["initial"]["life"], json!([20, 20]));
    }
}
#[test]
fn replay_rejects_malformed_raw_indices_and_wrong_pins() {
    for b in [b"".as_slice(), b"{}", b"{\"actions\":[0,0]}"] {
        assert!(matches!(verify(b), Err(ReplayError::Malformed)));
    }
    let original = recording();
    for field in [
        "version",
        "engine",
        "rules",
        "cards",
        "rng",
        "shuffle",
        "config_sha256",
    ] {
        let mut v = original.clone();
        v[field] = if field == "version" {
            json!(999)
        } else {
            json!("wrong")
        };
        assert!(
            matches!(verify(&bytes(&v)),Err(ReplayError::Incompatible{field:f}) if f==field),
            "{field}"
        );
    }
    let mut v = original.clone();
    v["choices"][0]["choice"]["action"] = json!({"index":0});
    assert!(matches!(verify(&bytes(&v)), Err(ReplayError::Malformed)));
    let mut v = original;
    v["choices"][0]["choice"]["action"]["index"] = json!(0);
    assert!(matches!(verify(&bytes(&v)), Err(ReplayError::Malformed)));
}
#[test]
fn replay_requires_exact_choice_consumption() {
    let mut extra = keeps();
    extra.push(choice(Seat::P0, Action::Keep {}));
    assert_eq!(
        record(&Config::default(), 42, 9, &extra),
        Err(ReplayError::UnconsumedChoice { index: 2 })
    );
    let mut v = recording();
    v["choices"].as_array_mut().unwrap().pop();
    assert!(matches!(
        verify(&bytes(&v)),
        Err(ReplayError::MissingChoice { index: 1 })
    ));
    let mut v = recording();
    let extra = v["choices"][0].clone();
    v["choices"].as_array_mut().unwrap().push(extra);
    assert!(matches!(
        verify(&bytes(&v)),
        Err(ReplayError::UnconsumedChoice { index: 2 })
    ));
    assert!(matches!(
        record(&Config::default(), 42, 9, &keeps()[..1]),
        Err(ReplayError::MissingChoice { index: 1 })
    ));
}
#[test]
fn replay_reports_first_tampered_action_or_checkpoint() {
    let mut v = recording();
    v["choices"][0]["choice"]["action"]["kind"] = json!("Mulligan");
    assert!(matches!(
        verify(&bytes(&v)),
        Err(ReplayError::Divergence { checkpoint: 1, .. })
    ));
    let mut v = recording();
    v["choices"][0]["after"]["life"][0] = json!(19);
    v["choices"][1]["after"]["life"][0] = json!(18);
    assert!(
        matches!(verify(&bytes(&v)),Err(ReplayError::Divergence{checkpoint:1,path,expected,actual}) if path=="/life/0" && expected=="19" && actual=="20")
    );
    let mut v = recording();
    v["initial"]["life"][1] = json!(0);
    assert!(matches!(
        verify(&bytes(&v)),
        Err(ReplayError::Divergence { checkpoint: 0, .. })
    ));
}

fn refs(hand: &[String], positions: &[usize]) -> Vec<CardRef> {
    positions
        .iter()
        .map(|&i| CardRef {
            card: hand[i].clone(),
            occurrence: hand[..i].iter().filter(|c| **c == hand[i]).count(),
        })
        .collect()
}
fn strings(v: &Value) -> Vec<String> {
    serde_json::from_value(v.clone()).unwrap()
}
fn vector_script(v: &Value) -> Vec<Choice> {
    let starter = v["starter"].as_u64().unwrap() as usize;
    let seats = [Seat::P0, Seat::P1];
    let order = [starter, 1 - starter];
    let mut script: Vec<_> = order
        .iter()
        .map(|&s| choice(seats[s], Action::Mulligan {}))
        .collect();
    for s in order {
        // Independent oracle first ledger has hand positions 0..5 and its
        // library's last card is the selected original position 6.
        let mut hand = strings(&v["first"]["hands"][s]);
        hand.push(
            v["first"]["libraries"][s]
                .as_array()
                .unwrap()
                .last()
                .unwrap()
                .as_str()
                .unwrap()
                .into(),
        );
        script.push(choice(
            seats[s],
            Action::Bottom {
                cards: refs(&hand, &[6]),
            },
        ));
    }
    script.push(choice(seats[starter], Action::Mulligan {}));
    script.push(choice(seats[1 - starter], Action::Keep {}));
    let mut hand = strings(&v["second"]["hands"][starter]);
    let lib = strings(&v["second"]["libraries"][starter]);
    hand.insert(1, lib[lib.len() - 1].clone());
    hand.insert(5, lib[lib.len() - 2].clone());
    script.push(choice(
        seats[starter],
        Action::Bottom {
            cards: refs(&hand, &[5, 1]),
        },
    ));
    script.push(choice(seats[starter], Action::Keep {}));
    script
}
#[test]
fn replay_mulligans_match_independent_complete_order_vectors() {
    let vectors: Value =
        serde_json::from_str(include_str!("../../../doc/evidence/mulligan/vectors.json")).unwrap();
    for v in vectors.as_array().unwrap() {
        let c = Config {
            starting_seat: v["starter"].as_u64().unwrap() as u8,
            ..Config::default()
        };
        let b = record(&c, 42, 9, &vector_script(v)).unwrap();
        let g = verify(&b).unwrap();
        let wire: Value = serde_json::from_slice(&b).unwrap();
        for (s, seat) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            assert_eq!(
                wire["choices"][3]["after"]["hands"][s],
                v["first"]["hands"][s]
            );
            assert_eq!(
                wire["choices"][3]["after"]["libraries"][s],
                v["first"]["libraries"][s]
            );
            assert_eq!(
                cards(&g, Zone::Hand(seat)),
                strings(&v["second"]["hands"][s])
            );
            assert_eq!(
                cards(&g, Zone::Library(seat)),
                strings(&v["second"]["libraries"][s])
            );
        }
        assert_eq!(
            wire["choices"][6]["after"]["mulligans"],
            if c.starting_seat == 0 {
                json!([2, 1])
            } else {
                json!([1, 2])
            }
        );
        // Stable semantic bytes despite fresh process-local object scopes.
        assert_eq!(b, record(&c, 42, 9, &vector_script(v)).unwrap());
    }
}
#[test]
fn replay_semantic_bottom_and_illegal_choices_are_strict() {
    let vectors: Value =
        serde_json::from_str(include_str!("../../../doc/evidence/mulligan/vectors.json")).unwrap();
    let script = vector_script(&vectors[0]);
    let c = Config::default();
    for (index, action, expected) in [
        (0, Action::Bottom { cards: vec![] }, ApplyError::WrongKind),
        (2, Action::Keep {}, ApplyError::WrongKind),
        (
            2,
            Action::Bottom { cards: vec![] },
            ApplyError::WrongCardinality,
        ),
        (
            2,
            Action::Bottom {
                cards: vec![CardRef {
                    card: "black-lotus".into(),
                    occurrence: 0,
                }],
            },
            ApplyError::IllegalCandidate,
        ),
        (
            2,
            Action::Bottom {
                cards: vec![CardRef {
                    card: "mountain".into(),
                    occurrence: 99,
                }],
            },
            ApplyError::IllegalCandidate,
        ),
    ] {
        let mut bad = script.clone();
        bad[index].action = action;
        assert_eq!(
            record(&c, 42, 9, &bad),
            Err(ReplayError::InvalidChoice {
                index,
                reason: expected
            })
        );
    }
    let mut bad = script.clone();
    bad[0].actor = Seat::P1;
    assert_eq!(
        record(&c, 42, 9, &bad),
        Err(ReplayError::InvalidChoice {
            index: 0,
            reason: ApplyError::WrongActor
        })
    );
    let mut bad = script.clone();
    let Action::Bottom { cards } = &mut bad[6].action else {
        panic!()
    };
    cards[1] = cards[0].clone();
    assert_eq!(
        record(&c, 42, 9, &bad),
        Err(ReplayError::InvalidChoice {
            index: 6,
            reason: ApplyError::DuplicateCandidate
        })
    );
    let mut wire: Value = serde_json::from_slice(&record(&c, 42, 9, &script).unwrap()).unwrap();
    // Different legal card, with the same raw occurrence, changes semantics.
    wire["choices"][2]["choice"]["action"]["cards"] = json!([{"card":"mountain","occurrence":0}]);
    assert!(matches!(
        verify(&bytes(&wire)),
        Err(ReplayError::Divergence { checkpoint: 3, .. })
    ));
}
#[test]
fn replay_config_seed_schema_and_checkpoint_corruption() {
    let original = recording();
    for field in ["format", "engine", "rules", "cards", "rng", "shuffle"] {
        let mut v = original.clone();
        v[field] = json!("future");
        assert!(matches!(verify(&bytes(&v)),Err(ReplayError::Incompatible{field:f}) if f==field));
    }
    for field in ["master", "episode"] {
        let mut v = original.clone();
        v[field] = json!(0);
        assert!(matches!(
            verify(&bytes(&v)),
            Err(ReplayError::Divergence { checkpoint: 0, .. })
        ));
    }
    let mut v = original.clone();
    v["config"]["starting_seat"] = json!(1);
    assert!(
        matches!(verify(&bytes(&v)),Err(ReplayError::Incompatible{field}) if field=="config_sha256")
    );
    for field in ["initial", "choices", "config", "version"] {
        let mut v = original.clone();
        v.as_object_mut().unwrap().remove(field);
        assert!(matches!(verify(&bytes(&v)), Err(ReplayError::Malformed)));
    }
    let mut v = original.clone();
    v["choices"][0].as_object_mut().unwrap().remove("after");
    assert!(matches!(verify(&bytes(&v)), Err(ReplayError::Malformed)));
    let mut v = original;
    v["choices"][0]["after"]["rng"]["state"] = json!(0);
    assert!(
        matches!(verify(&bytes(&v)),Err(ReplayError::Divergence{checkpoint:1,path,..}) if path=="/rng/state")
    );
    let c = Config {
        starting_seat: 2,
        ..Config::default()
    };
    assert_eq!(
        record(&c, 42, 9, &keeps()),
        Err(ReplayError::InvalidConfig(ResetError::InvalidStartingSeat))
    );
}
#[test]
fn replay_explicit_order_mirrors_have_literal_cards() {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../data/cards/foundations_micro_v1.json"
    ))
    .unwrap();
    for deck in manifest["decks"].as_array().unwrap() {
        let mut order = Vec::new();
        for row in deck["cards"].as_array().unwrap() {
            order.extend(std::iter::repeat_n(
                row["card_id"].as_str().unwrap().to_owned(),
                row["copies"].as_u64().unwrap() as usize,
            ));
        }
        let c = Config {
            seats: vec![
                DeckConfig {
                    deck: deck["id"].as_str().unwrap().into(),
                    order: Some(order)
                };
                2
            ],
            ..Config::default()
        };
        let b = record(&c, 42, 9, &keeps()).unwrap();
        let mut g = verify(&b).unwrap();
        let land = if deck["id"] == "red" {
            "mountain"
        } else {
            "forest"
        };
        for seat in [Seat::P0, Seat::P1] {
            assert_eq!(cards(&g, Zone::Hand(seat)), vec![land; 7]);
        }
        assert!(g.start_turns().is_ok());
    }
}

#[test]
fn replay_requires_final_decision_checkpoint_field() {
    let mut v = recording();
    v["choices"][1]["after"]
        .as_object_mut()
        .unwrap()
        .remove("decision");
    assert!(matches!(verify(&bytes(&v)), Err(ReplayError::Malformed)));
}
#[test]
fn replay_cross_process_verification() {
    const INPUT: &str = "MTG_REPLAY_TEST_INPUT";
    let vectors: Value =
        serde_json::from_str(include_str!("../../../doc/evidence/mulligan/vectors.json")).unwrap();
    if let Ok(path) = std::env::var(INPUT) {
        let g = verify(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(g.life(), [20, 20]);
        assert!(g.decision().is_none());
        for (i, s) in [Seat::P0, Seat::P1].into_iter().enumerate() {
            assert_eq!(
                cards(&g, Zone::Hand(s)),
                strings(&vectors[0]["second"]["hands"][i])
            );
            assert_eq!(
                cards(&g, Zone::Library(s)),
                strings(&vectors[0]["second"]["libraries"][i])
            );
        }
        return;
    }
    let b = record(&Config::default(), 42, 9, &vector_script(&vectors[0])).unwrap();
    let path = std::env::temp_dir().join(format!("mtg-replay-{}.json", std::process::id()));
    std::fs::write(&path, b).unwrap();
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "replay_cross_process_verification",
            "--nocapture",
        ])
        .env(INPUT, &path)
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
}
