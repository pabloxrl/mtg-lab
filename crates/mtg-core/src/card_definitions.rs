//! Typed rules projection of the frozen card manifest, not a card-text interpreter.
//! Identity order/hashes remain in objects; an identity alone never grants play.
use super::mana::{Color, ManaCost};
use crate::objects::CardId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InstantEffect {
    Growth,
    Bite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Definition {
    BasicLand(Color),
    VanillaCreature {
        cost: ManaCost,
        power: u32,
        toughness: u32,
    },
    ManaCreature {
        cost: ManaCost,
        power: u32,
        toughness: u32,
        color: Color,
    },
    Token,
    TokenSorcery {
        cost: ManaCost,
    },
    Instant {
        cost: ManaCost,
        effect: InstantEffect,
    },
    /// Existing synthetic-position characteristics only. No casting, mana or
    /// combat behavior: Magnigoth's reach is not implemented in the M1 slice.
    UnsupportedCreature {
        power: u32,
        toughness: u32,
    },
    Unsupported,
}

impl Definition {
    pub(super) fn cost(self) -> Option<ManaCost> {
        match self {
            Self::VanillaCreature { cost, .. }
            | Self::ManaCreature { cost, .. }
            | Self::Instant { cost, .. }
            | Self::TokenSorcery { cost } => Some(cost),
            _ => None,
        }
    }
    pub(super) fn basic_color(self) -> Option<Color> {
        match self {
            Self::BasicLand(color) => Some(color),
            _ => None,
        }
    }
    pub(super) fn mana_color(self) -> Option<Color> {
        match self {
            Self::BasicLand(color) | Self::ManaCreature { color, .. } => Some(color),
            _ => None,
        }
    }
    pub(super) fn creature_base(self) -> Option<(u32, u32)> {
        match self {
            Self::Token => Some((1, 1)),
            Self::VanillaCreature {
                power, toughness, ..
            }
            | Self::ManaCreature {
                power, toughness, ..
            }
            | Self::UnsupportedCreature { power, toughness } => Some((power, toughness)),
            _ => None,
        }
    }
    pub(super) fn instant_effect(self) -> Option<InstantEffect> {
        match self {
            Self::Instant { effect, .. } => Some(effect),
            _ => None,
        }
    }
    pub(super) fn vanilla(self) -> bool {
        matches!(
            self,
            Self::VanillaCreature { .. } | Self::ManaCreature { .. } | Self::Token
        )
    }
}

pub(super) fn definition(card: CardId) -> Definition {
    // Only these explicit entries enable rules. Never infer support from a
    // frozen identity, a mana cost, or fixture-only creature characteristics.
    match card.identity().key {
        "goblin-token" => Definition::Token,
        "dragon-fodder" => Definition::TokenSorcery {
            cost: ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
            },
        },
        "llanowar-elves" => Definition::ManaCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 0,
            },
            power: 1,
            toughness: 1,
            color: Color::Green,
        },
        "druid-of-the-cowl" => Definition::ManaCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
            },
            power: 1,
            toughness: 3,
            color: Color::Green,
        },
        "forest" => Definition::BasicLand(Color::Green),
        "mountain" => Definition::BasicLand(Color::Red),
        "bear-cub" => Definition::VanillaCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
            },
            power: 2,
            toughness: 2,
        },
        "swab-goblin" => Definition::VanillaCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
            },
            power: 2,
            toughness: 2,
        },
        "giant-growth" => Definition::Instant {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 0,
            },
            effect: InstantEffect::Growth,
        },
        "bite-down" => Definition::Instant {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
            },
            effect: InstantEffect::Bite,
        },
        "magnigoth-sentry" => Definition::UnsupportedCreature {
            power: 4,
            toughness: 4,
        },
        _ => Definition::Unsupported,
    }
}
