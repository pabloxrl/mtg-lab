//! Reset to the initial opening decision. All inspection here is privileged.
use crate::objects::{CardId, Handle, ObjectStore, Seat, StorageError, Zone};
use crate::rng::{EpisodeRng, Stream, VERSION};

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
    NotStarted,
    OpeningPending,
    EmptyLibrary,
    Storage(StorageError),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetError {
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
    objects: ObjectStore,
    life: [u32; 2],
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
    pub fn life(&self) -> [u32; 2] {
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
        self.apply_inner(actor, action, None)
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
        self.apply_inner(actor, action, Some(order))
    }
    fn apply_inner(
        &mut self,
        actor: Seat,
        action: &OpeningAction,
        order: Option<&[Handle]>,
    ) -> Result<Option<OpeningDecision>, ApplyError> {
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
                for candidate in candidates {
                    self.objects
                        .move_to(cards[candidate.index], Zone::Library(actor))
                        .expect("bounded reserved opening move");
                }
                self.needs_bottom[index] = false;
                if self.mulligans[index] == 7 {
                    self.kept[index] = true;
                }
            }
            _ => unreachable!("validated selection"),
        }
        self.generation = generation;
        self.advance_opening();
        Ok(self.decision)
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
                let random = explicit.is_none();
                let mut shuffled = explicit.unwrap_or_else(|| old.clone());
                if random {
                    shuffle(&mut shuffled, self.rng.as_mut().expect("reset RNG"));
                }
                self.redraw(seat, &old, &mut shuffled);
                self.mulligans[i] += 1;
                self.needs_bottom[i] = true;
            }
            self.declarations = [None; 2];
            self.advance_opening();
            return;
        }
        self.decision = None;
    }
    fn redraw(&mut self, seat: Seat, old: &[Handle], shuffled: &mut [Handle]) {
        let mut current = Vec::with_capacity(40);
        for &h in old {
            current.push(
                self.objects
                    .move_to(h, Zone::Library(seat))
                    .expect("bounded reserved opening move"),
            );
        }
        // Zone moves change handles. Map the selected permutation to the new
        // identities before installing the top-first order.
        let updated: Vec<_> = shuffled
            .iter()
            .map(|h| current[old.iter().position(|old| old == h).expect("permutation")])
            .collect();
        self.objects.reorder(Zone::Library(seat), &updated);
        for _ in 0..7 {
            let top = self
                .objects
                .in_zone(Zone::Library(seat))
                .next()
                .expect("full deck");
            self.objects
                .move_to(top, Zone::Hand(seat))
                .expect("bounded reserved opening move");
        }
    }
    /// Rules primitive for a caller that has already established a draw event.
    /// There is no card-selection parameter. Turn timing/SBAs are separate work.
    pub fn draw_top(&mut self, seat: Seat) -> Result<Handle, DrawError> {
        if self.rng.is_none() {
            return Err(DrawError::NotStarted);
        }
        if self.decision.is_some() {
            return Err(DrawError::OpeningPending);
        }
        let top = self
            .objects
            .in_zone(Zone::Library(seat))
            .next()
            .ok_or(DrawError::EmptyLibrary)?;
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
        let mut rng = EpisodeRng::new(&config.rng_version, master, episode, Stream::Environment)
            .map_err(|_| ResetError::UnsupportedRng)?;
        let mut decks = [
            validate_deck(&config.seats[0])?,
            validate_deck(&config.seats[1])?,
        ];
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(ResetError::DecisionExhausted)?;
        for (deck, input) in decks.iter_mut().zip(&config.seats) {
            if input.order.is_none() {
                shuffle(deck, &mut rng);
            }
        }
        // Reserve before clearing. Subsequent allocations/moves fit these exact
        // fixed-deck bounds; generations start fresh at zero after reset.
        self.objects
            .reserve_reset(80, [40, 40, 40, 40, 0, 0, 0, 0, 0])
            .map_err(ResetError::Storage)?;
        self.objects.reset().map_err(ResetError::Storage)?;
        for (deck, seat) in decks.into_iter().zip([Seat::P0, Seat::P1]) {
            for card in deck {
                self.objects
                    .allocate(card, seat, Zone::Library(seat))
                    .expect("reserved opening storage");
            }
            for _ in 0..7 {
                let top = self
                    .objects
                    .in_zone(Zone::Library(seat))
                    .next()
                    .expect("40-card deck");
                self.objects
                    .move_to(top, Zone::Hand(seat))
                    .expect("reserved hand and fresh generation");
            }
        }
        self.starting = actor;
        self.kept = [false; 2];
        self.declarations = [None; 2];
        self.mulligans = [0; 2];
        self.orders = [None, None];
        self.needs_bottom = [false; 2];
        self.life = [20; 2];
        self.rng = Some(rng);
        self.generation = generation;
        let decision = OpeningDecision {
            generation,
            actor,
            candidates: &[OpeningChoice::Keep, OpeningChoice::Mulligan],
            kind: OpeningKind::KeepOrMulligan,
            id: DecisionId {
                scope: self.objects.scope(),
                generation,
            },
        };
        self.decision = Some(decision);
        Ok(decision)
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
fn bounded(mut next: impl FnMut() -> u64, size: u64) -> usize {
    let limit = (1_u128 << 64) / u128::from(size) * u128::from(size);
    loop {
        let word = next();
        if u128::from(word) < limit {
            return (word % size) as usize;
        }
    }
}
fn shuffle<T>(cards: &mut [T], rng: &mut EpisodeRng) {
    for size in (2..=cards.len()).rev() {
        let index = bounded(|| rng.next_u64(), size as u64);
        cards.swap(size - 1, index);
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
