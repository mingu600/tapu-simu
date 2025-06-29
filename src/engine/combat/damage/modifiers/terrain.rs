//! Terrain-based damage modifiers and grounding mechanics
//!
//! This module handles all terrain-related modifications to damage calculations
//! and determines Pokemon grounding status for terrain and ground-type move interactions.

use crate::core::battle_state::Pokemon;
use crate::core::instructions::{Terrain, VolatileStatus};
use crate::generation::GenerationMechanics;
use crate::types::PokemonType;

/// Check if a Pokemon is grounded (affected by terrain)
pub fn is_grounded(pokemon: &Pokemon) -> bool {
    // Check for Flying type
    if pokemon.types.iter().any(|t| *t == PokemonType::Flying) {
        return false;
    }

    // Check for Levitate ability
    if pokemon.ability == crate::types::Abilities::LEVITATE {
        return false;
    }

    // Check for items that affect grounding
    if let Some(ref item) = pokemon.item {
        match *item {
            crate::types::Items::AIRBALLOON => return false, // Air Balloon makes Pokemon ungrounded
            _ => {}
        }
    }

    // Check for volatile statuses that affect grounding
    if pokemon
        .volatile_statuses
        .contains(VolatileStatus::MagnetRise)
    {
        return false; // Magnet Rise makes Pokemon ungrounded
    }
    if pokemon
        .volatile_statuses
        .contains(VolatileStatus::Telekinesis)
    {
        return false; // Telekinesis makes Pokemon ungrounded
    }

    true
}

/// Calculate terrain damage modifier
pub fn get_terrain_damage_modifier(
    terrain: &Terrain,
    move_type: &str,
    attacker: &Pokemon,
    defender: &Pokemon,
    generation_mechanics: &GenerationMechanics,
) -> f32 {
    match terrain {
        Terrain::Electric | Terrain::ElectricTerrain => {
            if let Some(crate::types::PokemonType::Electric) = crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
                if is_grounded(attacker) {
                    // Electric Terrain: 1.3x in Gen 8+, 1.5x in Gen 7
                    if generation_mechanics.generation.number() >= 8 {
                        1.3
                    } else {
                        1.5
                    }
                } else {
                    1.0
                }
            } else {
                1.0
            }
        }
        Terrain::Grassy | Terrain::GrassyTerrain => {
            if let Some(crate::types::PokemonType::Grass) = crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
                if is_grounded(attacker) {
                    // Grassy Terrain: 1.3x in Gen 8+, 1.5x in Gen 7
                    if generation_mechanics.generation.number() >= 8 {
                        1.3
                    } else {
                        1.5
                    }
                } else {
                    1.0
                }
            } else if let Some(crate::types::PokemonType::Ground) = crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
                if is_grounded(defender) {
                    // Grassy Terrain reduces Earthquake and other ground moves by 0.5x
                    0.5
                } else {
                    1.0
                }
            } else {
                1.0
            }
        }
        Terrain::Psychic | Terrain::PsychicTerrain => {
            if let Some(crate::types::PokemonType::Psychic) = crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
                if is_grounded(attacker) {
                    // Psychic Terrain: 1.3x in Gen 8+, 1.5x in Gen 7
                    if generation_mechanics.generation.number() >= 8 {
                        1.3
                    } else {
                        1.5
                    }
                } else {
                    1.0
                }
            } else {
                1.0
            }
        }
        Terrain::Misty | Terrain::MistyTerrain => {
            // Misty Terrain reduces Dragon moves by 0.5x when target is grounded
            if let Some(crate::types::PokemonType::Dragon) = crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
                if is_grounded(defender) {
                    0.5
                } else {
                    1.0
                }
            } else {
                1.0
            }
        }
        Terrain::None => 1.0,
    }
}