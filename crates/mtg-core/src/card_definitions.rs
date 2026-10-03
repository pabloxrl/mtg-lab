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
    HasteCreature {
        cost: ManaCost,
        power: u32,
        toughness: u32,
    },
    InvokerCreature {
        cost: ManaCost,
        power: u32,
        toughness: u32,
    },
    PowerCreature {
        cost: ManaCost,
        power: u32,
        toughness: u32,
    },
    Token,
    DiscardDraw {
        cost: ManaCost,
    },
    TokenSorcery {
        cost: ManaCost,
    },
    Instant {
        cost: ManaCost,
        effect: InstantEffect,
    },
    KeywordCreature {
        cost: ManaCost,
        flying: bool,
        reach: bool,
        vigilance: bool,
        trample: bool,
        deathtouch: bool,
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
            | Self::KeywordCreature { cost, .. }
            | Self::HasteCreature { cost, .. }
            | Self::PowerCreature { cost, .. }
            | Self::InvokerCreature { cost, .. }
            | Self::Instant { cost, .. }
            | Self::TokenSorcery { cost }
            | Self::DiscardDraw { cost } => Some(cost),
            _ => None,
        }
    }
    pub(super) fn discard_draw(self) -> bool {
        matches!(self, Self::DiscardDraw { .. })
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
            | Self::InvokerCreature {
                power, toughness, ..
            }
            | Self::PowerCreature {
                power, toughness, ..
            }
            | Self::HasteCreature {
                power, toughness, ..
            }
            | Self::KeywordCreature {
                power, toughness, ..
            } => Some((power, toughness)),
            _ => None,
        }
    }
    pub(super) fn instant_effect(self) -> Option<InstantEffect> {
        match self {
            Self::Instant { effect, .. } => Some(effect),
            _ => None,
        }
    }
    pub(super) fn flying(self) -> bool {
        matches!(
            self,
            Self::KeywordCreature { flying: true, .. } | Self::PowerCreature { .. }
        )
    }
    pub(super) fn reach(self) -> bool {
        matches!(self, Self::KeywordCreature { reach: true, .. })
    }
    pub(super) fn vigilance(self) -> bool {
        matches!(
            self,
            Self::KeywordCreature {
                vigilance: true,
                ..
            }
        )
    }
    pub(super) fn trample(self) -> bool {
        matches!(self, Self::KeywordCreature { trample: true, .. })
    }
    pub(super) fn deathtouch(self) -> bool {
        matches!(
            self,
            Self::KeywordCreature {
                deathtouch: true,
                ..
            }
        )
    }
    pub(super) fn haste_activation(self) -> bool {
        matches!(self, Self::HasteCreature { .. })
    }
    pub(super) fn invoker_activation(self) -> bool {
        matches!(self, Self::InvokerCreature { .. })
    }
    pub(super) fn power_activation(self) -> bool {
        matches!(self, Self::PowerCreature { .. })
    }
    pub(super) fn vanilla(self) -> bool {
        matches!(
            self,
            Self::VanillaCreature { .. }
                | Self::ManaCreature { .. }
                | Self::KeywordCreature { .. }
                | Self::HasteCreature { .. }
                | Self::PowerCreature { .. }
                | Self::InvokerCreature { .. }
                | Self::Token
        )
    }
}

pub(super) fn definition(card: CardId) -> Definition {
    // Only these explicit entries enable rules. Never infer support from a
    // frozen identity, a mana cost, or fixture-only creature characteristics.
    match card.identity().key {
        "thrill-of-possibility" => Definition::DiscardDraw {
            cost: ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
            },
        },
        "wildheart-invoker" => Definition::InvokerCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 2,
            },
            power: 4,
            toughness: 3,
        },
        "shivan-dragon" => Definition::PowerCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 4,
            },
            power: 5,
            toughness: 5,
        },
        "axgard-cavalry" => Definition::HasteCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
            },
            power: 2,
            toughness: 2,
        },
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
        "tajuru-pathwarden" => Definition::KeywordCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 4,
            },
            flying: false,
            reach: false,
            vigilance: true,
            trample: true,
            deathtouch: false,
            power: 5,
            toughness: 4,
        },
        "thornweald-archer" => Definition::KeywordCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 1,
            },
            flying: false,
            reach: true,
            vigilance: false,
            trample: false,
            deathtouch: true,
            power: 2,
            toughness: 1,
        },
        "magnigoth-sentry" => Definition::KeywordCreature {
            cost: ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 3,
            },
            flying: false,
            reach: true,
            vigilance: false,
            trample: false,
            deathtouch: false,
            power: 4,
            toughness: 4,
        },
        _ => Definition::Unsupported,
    }
}
