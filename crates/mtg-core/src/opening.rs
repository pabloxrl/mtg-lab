//! Reset to the initial opening decision. All inspection here is privileged.
use crate::objects::{CardId, ObjectStore, Seat, StorageError, Zone};
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
    pub candidates: [OpeningChoice; 2],
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
}
impl Game {
    pub fn new() -> Result<Self, StorageError> {
        Ok(Self {
            objects: ObjectStore::new()?,
            life: [0; 2],
            decision: None,
            rng: None,
            generation: 0,
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
    /// Version of the retained environment stream, absent before first reset.
    pub fn rng_version(&self) -> Option<&'static str> {
        self.rng.as_ref().map(EpisodeRng::version)
    }
    /// Validate all configuration before touching the previous game or RNG.
    /// Returns the starting player's real opening boundary; choices are applied
    /// by a subsequent capability, not automatically selected here.
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
            .reserve_reset(80, [40, 40, 7, 7, 0, 0, 0, 0, 0])
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
        self.life = [20; 2];
        self.rng = Some(rng);
        self.generation = generation;
        let decision = OpeningDecision {
            generation,
            actor,
            candidates: [OpeningChoice::Keep, OpeningChoice::Mulligan],
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
fn shuffle(cards: &mut [CardId; 40], rng: &mut EpisodeRng) {
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
