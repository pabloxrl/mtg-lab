// Both public routes must refer to the same game, decisions and errors.
use mtg_core::{game, objects::Seat, opening};

#[test]
fn canonical_and_legacy_paths_share_state_and_rejection_contract() {
    let mut canonical = game::Game::new().unwrap();
    canonical.reset(&game::Config::default(), 130, 0).unwrap();
    let legacy: &mut opening::Game = &mut canonical;
    let first = legacy.decision().unwrap();
    assert_eq!(first.actor, Seat::P0);
    let action = opening::OpeningAction {
        decision: first.id,
        selection: opening::Selection::Choose(first.candidate(0)),
    };
    let before = legacy.snapshot();
    assert_eq!(
        legacy.apply(Seat::P1, &action),
        Err(game::ApplyError::WrongActor)
    );
    assert_eq!(legacy.snapshot(), before);
    legacy.apply(Seat::P0, &action).unwrap();
    assert_eq!(canonical.decision().unwrap().actor, Seat::P1);
    let second = canonical.decision().unwrap();
    canonical
        .apply(
            Seat::P1,
            &game::OpeningAction {
                decision: second.id,
                selection: game::Selection::Choose(second.candidate(0)),
            },
        )
        .unwrap();
    assert_eq!(
        canonical.resume(std::num::NonZeroUsize::MAX),
        game::Progress::OpeningComplete
    );
    let saved = canonical.snapshot();
    let mut restored = opening::Game::new().unwrap();
    restored.restore(&saved).unwrap();
    assert_eq!(restored.life(), [20, 20]);
    assert_eq!(
        restored.resume(std::num::NonZeroUsize::MAX),
        opening::Progress::OpeningComplete
    );
}
