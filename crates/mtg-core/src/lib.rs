//! Deterministic rules engine for mtg-lab.
//!
//! Game state, rule transitions and seat-authorized choices are owned by game;
//! persistence and command-line orchestration live in separate crates.

pub mod game;
pub mod objects;
pub mod rng;

/// Compatibility facade for the original opening-rooted API.
/// New callers may use game; both paths name the same types and implementation.
pub mod opening {
    pub use crate::game::*;
}

pub mod trajectory;

pub mod episode;
pub mod metrics;
