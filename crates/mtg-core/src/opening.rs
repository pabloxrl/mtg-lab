//! Reset to the initial opening decision. All inspection here is privileged.
use crate::objects::{CardId, Handle, ObjectStore, Seat, StorageError, Zone};
use crate::rng::{EpisodeRng, Stream, VERSION};
use std::{collections::VecDeque, num::NonZeroUsize};

#[path = "casting.rs"]
pub mod casting;
#[path = "combat.rs"]
pub mod combat;
#[path = "mana.rs"]
pub mod mana;
#[path = "targets.rs"]
pub mod targets;
#[path = "terminal.rs"]
pub mod terminal;
#[path = "turns.rs"]
pub mod turns;

pub const FORMAT: &str = "foundations_micro_v1";
pub const SHUFFLE_VERSION: &str = "fisher-yates-rejection-v1";

#[derive(Clone, Debug)]
pub struct DeckConfig {
    pub deck: String,
    /// Optional exact post-shuffle order, top first; must match the frozen deck.
    pub order: Option<Vec<String>>,
}
impl DeckConfig {
    pub fn new(deck: &str) -> Self {
        Self {
            deck: deck.into(),
            order: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Config {
    pub format: String,
    pub seats: Vec<DeckConfig>,
    pub starting_seat: u8,
    pub game_number: u32,
    pub sideboards: bool,
    pub rng_version: String,
    pub shuffle_version: String,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            format: FORMAT.into(),
            seats: vec![DeckConfig::new("red"), DeckConfig::new("green")],
            starting_seat: 0,
            game_number: 1,
            sideboards: false,
            rng_version: VERSION.into(),
            shuffle_version: SHUFFLE_VERSION.into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpeningChoice {
    Keep,
    Mulligan,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpeningDecision {
    pub generation: u64,
    pub actor: Seat,
    pub candidates: &'static [OpeningChoice],
    pub kind: OpeningKind,
    pub id: DecisionId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecisionId {
    scope: u64,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateId {
    pub decision: DecisionId,
    pub index: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpeningKind {
    KeepOrMulligan,
    Bottom { count: usize },
}
impl OpeningDecision {
    pub fn candidate(self, index: usize) -> CandidateId {
        CandidateId {
            decision: self.id,
            index,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selection {
    Choose(CandidateId),
    Bottom(Vec<CandidateId>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpeningAction {
    pub decision: DecisionId,
    pub selection: Selection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyError {
    NoDecision,
    WorkPending,
    WrongActor,
    StaleDecision,
    StaleCandidate,
    WrongKind,
    IllegalCandidate,
    WrongCardinality,
    DuplicateCandidate,
    InvalidOrder,
    UnexpectedOrder,
    DecisionExhausted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawError {
    AlreadyEnded,
    NotStarted,
    OpeningPending,
    EmptyLibrary,
    Storage(StorageError),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetError {
    WorkPending,
    UnsupportedFormat,
    UnsupportedSeats,
    UnsupportedMatch,
    InvalidStartingSeat,
    UnsupportedDeck,
    InvalidDeckOrder,
    UnsupportedRng,
    UnsupportedShuffle,
    Storage(StorageError),
    DecisionExhausted,
}
#[derive(Debug)]
pub struct Game {
    outcome: Option<terminal::Outcome>,
    episode: Option<terminal::EpisodeId>,
    turns: turns::TurnState,
    work: VecDeque<Work>,
    objects: ObjectStore,
    life: [i64; 2],
    decision: Option<OpeningDecision>,
    rng: Option<EpisodeRng>,
    generation: u64,
    starting: Seat,
    kept: [bool; 2],
    declarations: [Option<OpeningChoice>; 2],
    mulligans: [usize; 2],
    orders: [Option<Vec<Handle>>; 2],
    needs_bottom: [bool; 2],
}
impl Game {
    pub fn new() -> Result<Self, StorageError> {
        Ok(Self {
            outcome: None,
            episode: None,
            turns: turns::TurnState::default(),
            work: VecDeque::new(),
            objects: ObjectStore::new()?,
            life: [0; 2],
            decision: None,
            rng: None,
            generation: 0,
            starting: Seat::P0,
            kept: [false; 2],
            declarations: [None; 2],
            mulligans: [0; 2],
            orders: [None, None],
            needs_bottom: [false; 2],
        })
    }
    pub fn objects(&self) -> &ObjectStore {
        &self.objects
    }
    pub fn life(&self) -> [i64; 2] {
        self.life
    }
    pub fn decision(&self) -> Option<OpeningDecision> {
        self.decision
    }
    /// Privileged candidate identities in current bottom-candidate index order.
    pub fn bottom_cards(&self) -> Option<[Handle; 7]> {
        let d = self.decision?;
        if !matches!(d.kind, OpeningKind::Bottom { .. }) {
            return None;
        }
        Some(
            self.objects
                .in_zone(Zone::Hand(d.actor))
                .collect::<Vec<_>>()
                .try_into()
                .expect("seven redrawn cards"),
        )
    }
    /// Apply a generation-scoped semantic opening selection.
    pub fn apply(
        &mut self,
        actor: Seat,
        action: &OpeningAction,
    ) -> Result<Option<OpeningDecision>, ApplyError> {
        self.apply_quantum(actor, action, None, NonZeroUsize::MAX)?;
        self.finish_work();
        Ok(self.decision)
    }
    /// Privileged test/reference chance injection. `order` is the exact full
    /// post-shuffle permutation of this seat's current hand plus library.
    /// It is NOT a policy choice. Explicit order consumes no RNG words.
    pub fn apply_with_order(
        &mut self,
        actor: Seat,
        action: &OpeningAction,
        order: &[Handle],
    ) -> Result<Option<OpeningDecision>, ApplyError> {
        self.apply_quantum(actor, action, Some(order), NonZeroUsize::MAX)?;
        self.finish_work();
        Ok(self.decision)
    }
    /// Validate a semantic selection, then perform at most `quantum` work units.
    pub fn apply_quantum(
        &mut self,
        actor: Seat,
        action: &OpeningAction,
        order: Option<&[Handle]>,
        quantum: NonZeroUsize,
    ) -> Result<Progress, ApplyError> {
        if !self.work.is_empty() {
            return Err(ApplyError::WorkPending);
        }
        let d = self.decision.ok_or(ApplyError::NoDecision)?;
        if actor != d.actor {
            return Err(ApplyError::WrongActor);
        }
        if action.decision != d.id {
            return Err(ApplyError::StaleDecision);
        }
        let index = seat_index(actor);
        match (&action.selection, d.kind) {
            (Selection::Choose(candidate), OpeningKind::KeepOrMulligan) => {
                validate_candidate(*candidate, d.id, d.candidates.len())?;
                if let Some(order) = order {
                    if candidate.index != 1 {
                        return Err(ApplyError::UnexpectedOrder);
                    }
                    let all: Vec<_> = self
                        .objects
                        .in_zone(Zone::Hand(actor))
                        .chain(self.objects.in_zone(Zone::Library(actor)))
                        .collect();
                    if order.len() != 40
                        || order
                            .iter()
                            .enumerate()
                            .any(|(i, h)| !all.contains(h) || order[..i].contains(h))
                    {
                        return Err(ApplyError::InvalidOrder);
                    }
                }
            }
            (Selection::Bottom(candidates), OpeningKind::Bottom { count, .. }) => {
                if order.is_some() {
                    return Err(ApplyError::UnexpectedOrder);
                }
                if candidates.len() != count {
                    return Err(ApplyError::WrongCardinality);
                }
                for (i, candidate) in candidates.iter().enumerate() {
                    validate_candidate(*candidate, d.id, 7)?;
                    if candidates[..i].contains(candidate) {
                        return Err(ApplyError::DuplicateCandidate);
                    }
                }
            }
            _ => return Err(ApplyError::WrongKind),
        }
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(ApplyError::DecisionExhausted)?;
        // All caller input and finite decision counters are checked above.
        // Reset reserved 40 entries per hand/library. At most seven mulligans
        // move any object <= 22 times, so storage cannot exhaust generations.
        match (&action.selection, d.kind) {
            (Selection::Choose(candidate), OpeningKind::KeepOrMulligan) => {
                let choice = d.candidates[candidate.index];
                self.declarations[index] = Some(choice);
                if choice == OpeningChoice::Keep {
                    self.kept[index] = true;
                }
                self.orders[index] = order.map(<[Handle]>::to_vec);
            }
            (Selection::Bottom(candidates), OpeningKind::Bottom { .. }) => {
                let cards = self.bottom_cards().expect("validated bottom boundary");
                self.work.push_back(Work::Bottom {
                    seat: actor,
                    cards: candidates.iter().map(|c| cards[c.index]).collect(),
                    position: 0,
                });
            }
            _ => unreachable!("validated selection"),
        }
        self.generation = generation;
        self.decision = None;
        self.work.push_back(Work::Advance);
        Ok(self.resume(quantum))
    }
    fn seat_order(&self) -> [Seat; 2] {
        match self.starting {
            Seat::P0 => [Seat::P0, Seat::P1],
            Seat::P1 => [Seat::P1, Seat::P0],
        }
    }
    fn set_decision(&mut self, actor: Seat, kind: OpeningKind) {
        self.decision = Some(OpeningDecision {
            generation: self.generation,
            actor,
            kind,
            id: DecisionId {
                scope: self.objects.scope(),
                generation: self.generation,
            },
            candidates: match kind {
                OpeningKind::KeepOrMulligan => &[OpeningChoice::Keep, OpeningChoice::Mulligan],
                OpeningKind::Bottom { .. } => &[],
            },
        });
    }
    fn advance_opening(&mut self) {
        let order = self.seat_order();
        // Finish the current round's bottoms before asking any next declaration.
        for seat in order {
            if self.needs_bottom[seat_index(seat)] {
                self.set_decision(
                    seat,
                    OpeningKind::Bottom {
                        count: self.mulligans[seat_index(seat)],
                    },
                );
                return;
            }
        }
        for seat in order {
            let i = seat_index(seat);
            if !self.kept[i] && self.declarations[i].is_none() {
                self.set_decision(seat, OpeningKind::KeepOrMulligan);
                return;
            }
        }
        if self.declarations.contains(&Some(OpeningChoice::Mulligan)) {
            // Everyone declared: all mulliganing seats redraw before bottoming.
            for seat in order {
                let i = seat_index(seat);
                if self.declarations[i] != Some(OpeningChoice::Mulligan) {
                    continue;
                }
                let old: Vec<_> = self
                    .objects
                    .in_zone(Zone::Hand(seat))
                    .chain(self.objects.in_zone(Zone::Library(seat)))
                    .collect();
                let explicit = self.orders[i].take();
                let size = if explicit.is_none() { 40 } else { 1 };
                let shuffled = explicit.unwrap_or_else(|| old.clone());
                self.work.push_back(Work::Redraw {
                    seat,
                    old,
                    shuffled,
                    current: Vec::with_capacity(40),
                    size,
                    position: 0,
                    phase: 0,
                });
            }
            self.declarations = [None; 2];
            self.work.push_back(Work::Advance);
            return;
        }
        self.decision = None;
    }
    /// Rules primitive for a caller that has already established a draw event.
    /// There is no card-selection parameter. A failed draw settles loss SBAs;
    /// callers must not use this primitive in the middle of a multi-draw effect.
    pub fn draw_top(&mut self, seat: Seat) -> Result<Handle, DrawError> {
        if self.outcome.is_some() {
            return Err(DrawError::AlreadyEnded);
        }
        if self.rng.is_none() {
            return Err(DrawError::NotStarted);
        }
        if self.decision.is_some() || !self.work.is_empty() {
            return Err(DrawError::OpeningPending);
        }
        let top = self.objects.in_zone(Zone::Library(seat)).next();
        let Some(top) = top else {
            self.settle_terminal(Some(seat));
            return Err(DrawError::EmptyLibrary);
        };
        self.objects
            .move_to(top, Zone::Hand(seat))
            .map_err(DrawError::Storage)
    }
    /// Version of the retained environment stream, absent before first reset.
    pub fn rng_version(&self) -> Option<&'static str> {
        self.rng.as_ref().map(EpisodeRng::version)
    }
    /// Validate all configuration before touching the previous game or RNG.
    /// Returns the starting player's opening boundary; callers explicitly apply
    /// each declaration and bottom selection.
    pub fn reset(
        &mut self,
        config: &Config,
        master: u64,
        episode: u64,
    ) -> Result<OpeningDecision, ResetError> {
        self.reset_quantum(config, master, episode, NonZeroUsize::MAX)?;
        self.finish_work();
        Ok(self.decision.expect("reset opening boundary"))
    }
    /// Start reset and perform bounded work. Validation and storage reservation
    /// precede mutation; another reset is rejected while work is pending.
    pub fn reset_quantum(
        &mut self,
        config: &Config,
        master: u64,
        episode: u64,
        quantum: NonZeroUsize,
    ) -> Result<Progress, ResetError> {
        if !self.work.is_empty() {
            return Err(ResetError::WorkPending);
        }
        if config.format != FORMAT {
            return Err(ResetError::UnsupportedFormat);
        }
        if config.seats.len() != 2 {
            return Err(ResetError::UnsupportedSeats);
        }
        if config.game_number != 1 || config.sideboards {
            return Err(ResetError::UnsupportedMatch);
        }
        let actor = match config.starting_seat {
            0 => Seat::P0,
            1 => Seat::P1,
            _ => return Err(ResetError::InvalidStartingSeat),
        };
        if config.shuffle_version != SHUFFLE_VERSION {
            return Err(ResetError::UnsupportedShuffle);
        }
        let rng = EpisodeRng::new(&config.rng_version, master, episode, Stream::Environment)
            .map_err(|_| ResetError::UnsupportedRng)?;
        let decks = [
            validate_deck(&config.seats[0])?,
            validate_deck(&config.seats[1])?,
        ];
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(ResetError::DecisionExhausted)?;
        // Reserve before clearing. Subsequent allocations/moves fit these exact
        // fixed-deck bounds; generations start fresh at zero after reset.
        self.objects
            .reserve_reset(80, [40, 40, 40, 40, 0, 0, 0, 0, 0])
            .map_err(ResetError::Storage)?;
        self.objects.reset().map_err(ResetError::Storage)?;
        self.outcome = None;
        self.episode = Some(terminal::EpisodeId(DecisionId {
            scope: self.objects.scope(),
            generation,
        }));
        self.turns = turns::TurnState::default();
        self.starting = actor;
        self.kept = [false; 2];
        self.declarations = [None; 2];
        self.mulligans = [0; 2];
        self.orders = [None, None];
        self.needs_bottom = [false; 2];
        self.life = [20; 2];
        self.rng = Some(rng);
        self.generation = generation;
        self.decision = None;
        self.work.push_back(Work::Reset {
            decks: Box::new(decks),
            random: [
                config.seats[0].order.is_none(),
                config.seats[1].order.is_none(),
            ],
            seat: 0,
            size: 40,
            position: 0,
            phase: 0,
        });
        Ok(self.resume(quantum))
    }
}

fn validate_deck(input: &DeckConfig) -> Result<[CardId; 40], ResetError> {
    let entries = match input.deck.as_str() {
        "red" => RED,
        "green" => GREEN,
        _ => return Err(ResetError::UnsupportedDeck),
    };
    let mut cards = [CardId::from_key(entries[0].0).expect("frozen identity"); 40];
    let mut position = 0;
    for &(key, count) in entries {
        for _ in 0..count {
            cards[position] = CardId::from_key(key).expect("frozen identity");
            position += 1;
        }
    }
    if let Some(order) = &input.order {
        if order.len() != 40 {
            return Err(ResetError::InvalidDeckOrder);
        }
        let mut remaining = entries.map(|(_, count)| count);
        for (output, key) in cards.iter_mut().zip(order) {
            let index = entries
                .iter()
                .position(|(name, _)| name == key)
                .ok_or(ResetError::InvalidDeckOrder)?;
            remaining[index] = remaining[index]
                .checked_sub(1)
                .ok_or(ResetError::InvalidDeckOrder)?;
            *output = CardId::from_key(key).expect("validated frozen key");
        }
    }
    Ok(cards)
}

// Rejection avoids modulo bias. u128 represents the exclusive 2^64 bound.
#[cfg(test)]
fn bounded(mut next: impl FnMut() -> u64, size: u64) -> usize {
    let limit = (1_u128 << 64) / u128::from(size) * u128::from(size);
    loop {
        let word = next();
        if u128::from(word) < limit {
            return (word % size) as usize;
        }
    }
}

// Frozen deck projection in manifest order; integration test checks exact counts.
const RED: &[(&str, u8); 10] = &[
    ("mountain", 16),
    ("swab-goblin", 4),
    ("axgard-cavalry", 2),
    ("crackling-cyclops", 3),
    ("firebrand-archer", 3),
    ("dragon-fodder", 4),
    ("goblin-surprise", 2),
    ("viashino-pyromancer", 2),
    ("thrill-of-possibility", 2),
    ("shivan-dragon", 2),
];
const GREEN: &[(&str, u8); 10] = &[
    ("forest", 16),
    ("bear-cub", 4),
    ("llanowar-elves", 3),
    ("druid-of-the-cowl", 2),
    ("magnigoth-sentry", 2),
    ("tajuru-pathwarden", 2),
    ("thornweald-archer", 3),
    ("giant-growth", 3),
    ("bite-down", 3),
    ("wildheart-invoker", 2),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_bounded_sampling_rejects_incomplete_tail() {
        // 2^64 = 40*q + 16, so the highest 16 words must be rejected.
        let mut words = [u64::MAX, u64::MAX - 15, u64::MAX - 16].into_iter();
        assert_eq!(bounded(|| words.next().unwrap(), 40), 39);
        assert_eq!(words.next(), None);
        assert_eq!(bounded(|| u64::MAX, 2), 1); // exact divisor rejects nothing
        assert_eq!(bounded(|| 0, 40), 0);
    }

    #[test]
    fn opening_counter_exhaustion_preserves_game_and_rng() {
        let mut game = Game::new().unwrap();
        game.reset(&Config::default(), 42, 9).unwrap();
        game.generation = u64::MAX;
        let before = format!("{game:?}");
        assert_eq!(
            game.reset(&Config::default(), 1, 2),
            Err(ResetError::DecisionExhausted)
        );
        assert_eq!(format!("{game:?}"), before);
    }

    #[test]
    fn opening_explicit_orders_do_not_consume_rng() {
        let mut config = Config::default();
        for seat in &mut config.seats {
            seat.order = Some(
                validate_deck(seat)
                    .unwrap()
                    .iter()
                    .map(|card| card.identity().key.into())
                    .collect(),
            );
        }
        let mut game = Game::new().unwrap();
        game.reset(&config, 42, 9).unwrap();
        let fresh = EpisodeRng::new(VERSION, 42, 9, Stream::Environment).unwrap();
        assert_eq!(game.rng.as_ref().unwrap(), &fresh);
        for _ in 0..100 {
            game.rng.as_mut().unwrap().next_u64();
        }
        game.reset(&config, 42, 9).unwrap();
        assert_eq!(game.rng.as_ref().unwrap(), &fresh);
    }
}

fn seat_index(seat: Seat) -> usize {
    match seat {
        Seat::P0 => 0,
        Seat::P1 => 1,
    }
}
fn validate_candidate(
    candidate: CandidateId,
    decision: DecisionId,
    count: usize,
) -> Result<(), ApplyError> {
    if candidate.decision != decision {
        return Err(ApplyError::StaleCandidate);
    }
    if candidate.index >= count {
        return Err(ApplyError::IllegalCandidate);
    }
    Ok(())
}

#[cfg(test)]
mod mulligan_invariants {
    use super::*;
    #[test]
    fn mulligan_explicit_chance_and_rejections_do_not_consume_rng() {
        let mut g = Game::new().unwrap();
        g.reset(&Config::default(), 42, 9).unwrap();
        let rng_before = format!("{:?}", g.rng);
        for seat in [Seat::P0, Seat::P1] {
            let d = g.decision().unwrap();
            let a = OpeningAction {
                decision: d.id,
                selection: Selection::Choose(d.candidate(1)),
            };
            let order: Vec<_> = g
                .objects
                .in_zone(Zone::Hand(seat))
                .chain(g.objects.in_zone(Zone::Library(seat)))
                .collect();
            let before = format!("{g:?}");
            assert_eq!(
                g.apply_with_order(seat, &a, &order[..39]),
                Err(ApplyError::InvalidOrder)
            );
            assert_eq!(format!("{g:?}"), before);
            g.apply_with_order(seat, &a, &order).unwrap();
        }
        assert_eq!(format!("{:?}", g.rng), rng_before);
        for _ in 0..2 {
            let d = g.decision().unwrap();
            let a = OpeningAction {
                decision: d.id,
                selection: Selection::Bottom(vec![d.candidate(0)]),
            };
            g.apply(d.actor, &a).unwrap();
        }
        assert_eq!(format!("{:?}", g.rng), rng_before);
    }
    #[test]
    fn mulligan_decision_exhaustion_is_atomic_in_both_kinds() {
        for bottom in [false, true] {
            let mut g = Game::new().unwrap();
            g.reset(&Config::default(), 42, 9).unwrap();
            if bottom {
                for index in [1, 0] {
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
            }
            let d = g.decision().unwrap();
            let selection = if bottom {
                Selection::Bottom(vec![d.candidate(0)])
            } else {
                Selection::Choose(d.candidate(1))
            };
            g.generation = u64::MAX;
            let before = format!("{g:?}");
            assert_eq!(
                g.apply(
                    d.actor,
                    &OpeningAction {
                        decision: d.id,
                        selection
                    }
                ),
                Err(ApplyError::DecisionExhausted)
            );
            assert_eq!(format!("{g:?}"), before);
        }
    }
}

/// Internal work is never a player choice, terminal result, or reward event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    Terminal(terminal::Outcome),
    InternalYield,
    Decision(OpeningDecision),
    OpeningComplete,
    NotStarted,
}
#[derive(Debug)]
enum Work {
    Reset {
        decks: Box<[[CardId; 40]; 2]>,
        random: [bool; 2],
        seat: usize,
        size: usize,
        position: usize,
        phase: u8,
    },
    Redraw {
        seat: Seat,
        old: Vec<Handle>,
        shuffled: Vec<Handle>,
        current: Vec<Handle>,
        size: usize,
        position: usize,
        phase: u8,
    },
    Bottom {
        seat: Seat,
        cards: Vec<Handle>,
        position: usize,
    },
    Advance,
}
// Exactly one RNG word per unit, including rejection. A rejected word retains
// the same shuffle position so even the rejection loop can yield.
fn shuffle_step<T>(cards: &mut [T], size: &mut usize, rng: &mut EpisodeRng) {
    shuffle_word(cards, size, rng.next_u64());
}
fn shuffle_word<T>(cards: &mut [T], size: &mut usize, word: u64) {
    let limit = (1_u128 << 64) / *size as u128 * *size as u128;
    if u128::from(word) < limit {
        cards.swap(*size - 1, (word % *size as u64) as usize);
        *size -= 1;
    }
}
impl Game {
    fn finish_work(&mut self) {
        while self.resume(NonZeroUsize::MAX) == Progress::InternalYield {}
    }
    /// Resume owned work exactly once, stopping at the first player boundary.
    /// At a settled boundary repeated resumes are read-only. Inspection during
    /// a yield is privileged and may see partially completed internal work.
    pub fn resume(&mut self, quantum: NonZeroUsize) -> Progress {
        if let Some(result) = self.outcome {
            return Progress::Terminal(result);
        }
        for _ in 0..quantum.get() {
            let Some(mut work) = self.work.pop_front() else {
                break;
            };
            let done = match &mut work {
                Work::Advance => {
                    self.advance_opening();
                    true
                }
                Work::Bottom {
                    seat,
                    cards,
                    position,
                } => {
                    self.objects
                        .move_to(cards[*position], Zone::Library(*seat))
                        .expect("reserved bottom move");
                    *position += 1;
                    if *position == cards.len() {
                        let i = seat_index(*seat);
                        self.needs_bottom[i] = false;
                        if self.mulligans[i] == 7 {
                            self.kept[i] = true;
                        }
                        true
                    } else {
                        false
                    }
                }
                Work::Reset {
                    decks,
                    random,
                    seat,
                    size,
                    position,
                    phase,
                } => {
                    let actor = [Seat::P0, Seat::P1][*seat];
                    match *phase {
                        0 => {
                            if random[*seat] && *size > 1 {
                                shuffle_step(&mut decks[*seat], size, self.rng.as_mut().unwrap());
                            } else if *seat == 0 {
                                *seat = 1;
                                *size = 40;
                            } else {
                                *seat = 0;
                                *phase = 1;
                            }
                            false
                        }
                        1 => {
                            self.objects
                                .allocate(decks[*seat][*position], actor, Zone::Library(actor))
                                .expect("reserved reset allocation");
                            *position += 1;
                            if *position == 40 {
                                *position = 0;
                                *phase = 2;
                            }
                            false
                        }
                        _ => {
                            self.draw_internal(actor);
                            *position += 1;
                            if *position == 7 {
                                if *seat == 0 {
                                    *seat = 1;
                                    *position = 0;
                                    *phase = 1;
                                    false
                                } else {
                                    self.set_decision(self.starting, OpeningKind::KeepOrMulligan);
                                    true
                                }
                            } else {
                                false
                            }
                        }
                    }
                }
                Work::Redraw {
                    seat,
                    old,
                    shuffled,
                    current,
                    size,
                    position,
                    phase,
                } => {
                    match *phase {
                        0 => {
                            if *size > 1 {
                                shuffle_step(shuffled, size, self.rng.as_mut().unwrap());
                            } else {
                                *phase = 1;
                            }
                            false
                        }
                        1 => {
                            current.push(
                                self.objects
                                    .move_to(old[*position], Zone::Library(*seat))
                                    .expect("reserved redraw move"),
                            );
                            *position += 1;
                            if *position == 40 {
                                *phase = 2;
                                *position = 0;
                            }
                            false
                        }
                        2 => {
                            // Fixed 40-card permutation; no unbounded rules work.
                            let updated: Vec<_> = shuffled
                                .iter()
                                .map(|h| current[old.iter().position(|o| o == h).unwrap()])
                                .collect();
                            self.objects.reorder(Zone::Library(*seat), &updated);
                            *phase = 3;
                            false
                        }
                        _ => {
                            self.draw_internal(*seat);
                            *position += 1;
                            if *position == 7 {
                                self.mulligans[seat_index(*seat)] += 1;
                                self.needs_bottom[seat_index(*seat)] = true;
                                true
                            } else {
                                false
                            }
                        }
                    }
                }
            };
            if !done {
                self.work.push_front(work);
            }
        }
        if !self.work.is_empty() {
            Progress::InternalYield
        } else if let Some(d) = self.decision {
            Progress::Decision(d)
        } else if self.rng.is_some() {
            Progress::OpeningComplete
        } else {
            Progress::NotStarted
        }
    }
    fn draw_internal(&mut self, seat: Seat) {
        let top = self
            .objects
            .in_zone(Zone::Library(seat))
            .next()
            .expect("full opening library");
        self.objects
            .move_to(top, Zone::Hand(seat))
            .expect("reserved opening draw");
    }
}

#[cfg(test)]
mod quantum {
    use super::*;
    fn q(n: usize) -> NonZeroUsize {
        NonZeroUsize::new(n).unwrap()
    }
    fn settle(g: &mut Game, mut p: Progress, n: usize) -> Progress {
        let generation = g.generation;
        let mut calls = 0;
        while p == Progress::InternalYield {
            assert_eq!(g.decision(), None);
            assert_eq!(g.generation, generation); // yields never add semantic actions
            assert_eq!(g.life(), [20, 20]); // opening has no rewards/outcomes
            let before = format!("{g:?}");
            assert_eq!(
                g.reset_quantum(&Config::default(), 0, 0, q(n)),
                Err(ResetError::WorkPending)
            );
            assert_eq!(g.draw_top(Seat::P0), Err(DrawError::OpeningPending));
            assert_eq!(format!("{g:?}"), before);
            p = g.resume(q(n));
            calls += 1;
            assert!(calls < 1000);
        }
        let before = format!("{g:?}");
        assert_eq!(g.resume(q(n)), p);
        assert_eq!(format!("{g:?}"), before);
        p
    }
    fn equivalent(a: &Game, b: &Game) {
        assert_eq!(a.rng, b.rng);
        assert_eq!(a.life, b.life);
        assert_eq!(a.generation, b.generation);
        assert_eq!(a.kept, b.kept);
        assert_eq!(a.declarations, b.declarations);
        assert_eq!(a.mulligans, b.mulligans);
        assert_eq!(a.needs_bottom, b.needs_bottom);
        assert!(a.work.is_empty() && b.work.is_empty());
        assert_eq!(
            a.decision
                .map(|d| (d.actor, d.kind, d.generation, d.candidates)),
            b.decision
                .map(|d| (d.actor, d.kind, d.generation, d.candidates))
        );
        for seat in [Seat::P0, Seat::P1] {
            for zone in [Zone::Hand(seat), Zone::Library(seat)] {
                // Scope IDs intentionally differ across independent games.
                let objects = |g: &Game| {
                    g.objects
                        .in_zone(zone)
                        .map(|h| *g.objects.get(h).unwrap())
                        .collect::<Vec<_>>()
                };
                assert_eq!(objects(a), objects(b));
            }
        }
    }
    #[test]
    fn quantum_all_opening_boundaries_match_scalar_rng_and_decisions() {
        for n in [
            1, 2, 7, 38, 39, 40, 41, 78, 79, 80, 94, 173, 174, 175, 10000,
        ] {
            for starter in 0..2 {
                for explicit in [false, true] {
                    let mut c = Config {
                        starting_seat: starter,
                        ..Config::default()
                    };
                    if explicit {
                        for input in &mut c.seats {
                            input.order = Some(
                                validate_deck(input)
                                    .unwrap()
                                    .iter()
                                    .map(|c| c.identity().key.into())
                                    .collect(),
                            );
                        }
                    }
                    let mut a = Game::new().unwrap();
                    let mut b = Game::new().unwrap();
                    a.reset(&c, 42, 9).unwrap();
                    let p = b.reset_quantum(&c, 42, 9, q(n)).unwrap();
                    settle(&mut b, p, n);
                    equivalent(&a, &b);
                    // CR 103.5: both seats take all seven mulligans, bottom the
                    // cumulative count each round, then are forced to keep zero.
                    for _ in 0..28 {
                        let da = a.decision.unwrap();
                        let db = b.decision.unwrap();
                        let select = |d: OpeningDecision| match d.kind {
                            OpeningKind::KeepOrMulligan => Selection::Choose(d.candidate(1)),
                            OpeningKind::Bottom { count } => Selection::Bottom(
                                (0..count).rev().map(|i| d.candidate(i)).collect(),
                            ),
                        };
                        let order = |g: &Game, d: OpeningDecision| {
                            if explicit && d.kind == OpeningKind::KeepOrMulligan {
                                Some(
                                    g.objects
                                        .in_zone(Zone::Hand(d.actor))
                                        .chain(g.objects.in_zone(Zone::Library(d.actor)))
                                        .collect::<Vec<_>>()
                                        .into_iter()
                                        .rev()
                                        .collect::<Vec<_>>(),
                                )
                            } else {
                                None
                            }
                        };
                        let oa = order(&a, da);
                        let ob = order(&b, db);
                        let aa = OpeningAction {
                            decision: da.id,
                            selection: select(da),
                        };
                        let ab = OpeningAction {
                            decision: db.id,
                            selection: select(db),
                        };
                        if let Some(o) = oa {
                            a.apply_with_order(da.actor, &aa, &o).unwrap();
                        } else {
                            a.apply(da.actor, &aa).unwrap();
                        }
                        let p = b.apply_quantum(db.actor, &ab, ob.as_deref(), q(n)).unwrap();
                        settle(&mut b, p, n);
                        equivalent(&a, &b);
                    }
                    assert_eq!(b.resume(q(n)), Progress::OpeningComplete);
                    assert_eq!(b.generation, 29); // reset + 14 declarations + 14 bottoms
                    for seat in [Seat::P0, Seat::P1] {
                        assert_eq!(b.objects.in_zone(Zone::Hand(seat)).count(), 0);
                        assert_eq!(b.objects.in_zone(Zone::Library(seat)).count(), 40);
                    }
                }
            }
        }
    }
    #[test]
    fn quantum_rejected_shuffle_trial_retains_cursor_and_cards() {
        // 2^64 = 40*q + 16: highest sixteen words reject, next word maps to 39.
        let mut cards: Vec<_> = (0..40).collect();
        let original = cards.clone();
        let mut size = 40;
        for word in [u64::MAX, u64::MAX - 15] {
            shuffle_word(&mut cards, &mut size, word);
            assert_eq!(size, 40);
            assert_eq!(cards, original);
        }
        shuffle_word(&mut cards, &mut size, u64::MAX - 16);
        assert_eq!(size, 39);
        assert_eq!(cards, original);
        shuffle_word(&mut cards, &mut size, 0);
        assert_eq!(size, 38);
        assert_eq!(cards[0], 38);
        assert_eq!(cards[38], 0);
    }
    #[test]
    fn quantum_one_unit_consumes_at_most_one_word_or_object_operation() {
        let mut g = Game::new().unwrap();
        let mut expected = EpisodeRng::new(VERSION, 42, 9, Stream::Environment).unwrap();
        assert_eq!(
            g.reset_quantum(&Config::default(), 42, 9, q(1)).unwrap(),
            Progress::InternalYield
        );
        expected.next_u64();
        assert_eq!(g.rng.as_ref(), Some(&expected));
        // For this pinned seed the first 39 words are accepted (also covered by
        // independent complete permutation vectors in tests/opening.rs).
        for _ in 1..39 {
            g.resume(q(1));
            expected.next_u64();
            assert_eq!(g.rng.as_ref(), Some(&expected));
            assert_eq!(g.objects.slot_count(), 0);
        }
        let mut previous = 0;
        while g.resume(q(1)) == Progress::InternalYield {
            let count = g.objects.slot_count();
            assert!(count <= previous + 1);
            previous = count;
        }
        assert_eq!(g.objects.slot_count(), 80);
        for _ in 39..78 {
            expected.next_u64();
        }
        assert_eq!(g.rng, Some(expected));
    }
}

#[cfg(test)]
#[path = "targets_tests.rs"]
mod targets_tests;
