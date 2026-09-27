//! Rules engine foundation for mtg-lab.
//!
//! Versioned RNG, generation-safe storage, opening choices, turns, land/mana transitions vanilla creature casting targeted Growth/Bite effects vanilla combat and rules terminal outcomes.
//! Other spell effects, combat keywords and CLI matches remain future work.

pub mod rng;

pub mod objects;

pub mod opening;
