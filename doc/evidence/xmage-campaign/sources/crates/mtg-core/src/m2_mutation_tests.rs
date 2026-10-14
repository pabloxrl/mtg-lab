//! Minimized, explicitly synthetic positions; no new card support or reachability
//! claim. Literal rules/contract oracles are independent of the mutation runner.
use super::*;
use policy::{Choice, PolicyError, Submission};
use turns::{Step, TurnAction, TurnSelection};

fn ready() -> Game {
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 213, 0).unwrap();
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
fn add(g: &mut Game, key: &str, zone: Zone) -> Handle {
    g.objects
        .allocate(CardId::from_key(key).unwrap(), Seat::P0, zone)
        .unwrap()
}
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
fn growth(g: &mut Game, target: Handle) {
    let h = add(g, "giant-growth", Zone::Hand(Seat::P0));
    g.turns.mana[0][4] = 1;
    let d = g.turn_decision().unwrap();
    let t = g.begin_targeted_cast(Seat::P0, d.id, h, 256).unwrap();
    let t = g.choose_target(Seat::P0, t.id, target).unwrap();
    let p = g.finish_targets(Seat::P0, t.id).unwrap();
    let p = g
        .choose_payment(Seat::P0, p.id, mana::Color::Green)
        .unwrap();
    g.finish_cast(Seat::P0, p.id).unwrap();
}

#[test]
fn priority() {
    // CR 117.3d/117.4: one pass cannot end upkeep; the opponent may respond.
    let mut g = ready();
    pass(&mut g);
    assert_eq!(
        (g.turn_decision().unwrap().actor, g.turn_position()),
        (Seat::P1, Some((1, Seat::P0, Step::Upkeep))),
        "M2-MUT priority: opponent receives priority in the same step"
    );
}
#[test]
fn mana_stack() {
    // CR 605.3b: a mana ability resolves immediately without using the stack.
    let mut g = ready();
    let elf = add(&mut g, "llanowar-elves", Zone::Battlefield);
    let d = g.turn_decision().unwrap();
    g.tap_mana(Seat::P0, d.id, elf).unwrap();
    assert_eq!(
        (g.objects.in_zone(Zone::Stack).count(), g.mana()[0][4]),
        (0, 1),
        "M2-MUT mana_stack: immediate green mana and empty stack"
    );
}
#[test]
fn sick_tap() {
    // CR 302.6: this newly controlled Elf cannot pay a tap-symbol cost.
    let mut g = ready();
    let elf = add(&mut g, "llanowar-elves", Zone::Battlefield);
    g.turns.sick.push(elf);
    let d = g.turn_decision().unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.tap_mana(Seat::P0, d.id, elf),
        Err(mana::ManaError::IllegalSource),
        "M2-MUT sick_tap: sick Elf tap is rejected"
    );
    assert_eq!(g.snapshot(), before);
}
#[test]
fn cast_trigger() {
    // CR 601.2i/603.2 and pinned Archer: Growth triggers on cast, before resolution.
    let mut g = ready();
    let archer = add(&mut g, "firebrand-archer", Zone::Battlefield);
    growth(&mut g, archer);
    assert_eq!(
        g.trigger_candidates(Seat::P0).len(),
        1,
        "M2-MUT cast_trigger: committed Growth creates one Archer trigger"
    );
}
#[test]
fn boost_expiry() {
    // CR 514.2/611.2a: Growth's +3/+3 ends in cleanup.
    let mut g = ready();
    let cub = add(&mut g, "bear-cub", Zone::Battlefield);
    growth(&mut g, cub);
    pass(&mut g);
    pass(&mut g);
    assert_eq!(g.creature_state(cub).unwrap().power, 5);
    g.turns.position = Some((1, Seat::P0, Step::End));
    pass(&mut g);
    pass(&mut g);
    assert_eq!(
        g.creature_state(cub).map(|c| (c.power, c.toughness)),
        Some((2, 2)),
        "M2-MUT boost_expiry: Cub returns to printed 2/2"
    );
}
#[test]
fn cleanup_damage() {
    // CR 514.2/704.5g: remove +3/+3 AND 2 marked damage before any SBA.
    let mut g = ready();
    let cub = add(&mut g, "bear-cub", Zone::Battlefield);
    g.turns.modifications.push(targets::Modification {
        handle: cub,
        boost: 3,
        power_boost: 0,
        damage: 2,
    });
    g.turns.position = Some((1, Seat::P0, Step::End));
    pass(&mut g);
    pass(&mut g);
    assert_eq!(
        g.creature_state(cub)
            .map(|c| (c.power, c.toughness, c.damage)),
        Some((2, 2, 0)),
        "M2-MUT cleanup_damage: damaged grown Cub survives cleanup as clean 2/2"
    );
}
#[test]
fn stale_action() {
    // SYS-CORE-003: the old generation cannot authorize an otherwise legal pass.
    let mut g = ready();
    let d = g.policy_observe(Seat::P0, 256).unwrap().decision.unwrap();
    let stale = Submission {
        schema_version: 1,
        revision: d.revision,
        generation: d.generation,
        choices: vec![Choice::Pass],
    };
    pass(&mut g);
    pass(&mut g);
    let before = g.snapshot();
    assert_eq!(
        g.apply_policy(Seat::P0, &stale, 256),
        Err(PolicyError::StaleDecision),
        "M2-MUT stale_action: old generation is rejected"
    );
    assert_eq!(g.snapshot(), before);
}
#[test]
fn private_hand() {
    // SYS-PRIV-001: swapping two unseen opponent card identities changes no input.
    let mut g = ready();
    let hand = g.objects.in_zone(Zone::Hand(Seat::P1)).next().unwrap();
    let card = g.objects.get(hand).unwrap().card;
    let library = g
        .objects
        .in_zone(Zone::Library(Seat::P1))
        .find(|h| g.objects.get(*h).unwrap().card != card)
        .unwrap();
    let before = g.policy_observe(Seat::P0, 256).unwrap();
    g.objects.get_mut(hand).unwrap().card = g.objects.get(library).unwrap().card;
    g.objects.get_mut(library).unwrap().card = card;
    assert_eq!(
        g.policy_observe(Seat::P0, 256).unwrap(),
        before,
        "M2-MUT private_hand: hidden swap preserves policy observation and choices"
    );
}
#[test]
fn clipped_choices() {
    // RFC B016/B033: overflow is explicit, never a shortened candidate list.
    let mut g = Game::new().unwrap();
    g.reset(&Config::default(), 213, 0).unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.policy_observe(Seat::P0, 1),
        Err(PolicyError::CapacityExceeded),
        "M2-MUT clipped_choices: keep and mulligan cannot fit in one row"
    );
    assert_eq!(g.snapshot(), before);
}
