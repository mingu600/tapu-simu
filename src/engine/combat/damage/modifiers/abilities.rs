//! Ability-based damage modifiers
//!
//! This module handles damage modifications from Pokemon abilities,
//! including STAB modifiers like Adaptability.

use crate::core::battle_state::Pokemon;

/// Check if a Pokemon has the Adaptability ability
pub fn has_adaptability_ability(pokemon: &Pokemon) -> bool {
    pokemon.ability == "adaptability"
}