//! Weather-based damage and stat modifiers
//!
//! This module handles all weather-related modifications to damage calculations
//! and stat values, including weather negation effects.

use crate::core::battle_state::{BattleState, Pokemon};
use crate::core::instructions::{Weather, Stat};
use crate::generation::GenerationMechanics;

/// Check if weather effects are negated by abilities like Cloud Nine or Air Lock
pub fn is_weather_negated(state: &BattleState) -> bool {
    use crate::engine::mechanics::abilities::{apply_ability_effect, AbilityContext};
    use crate::core::battle_format::{BattlePosition, SideReference};

    // Check all active Pokemon for weather negation abilities
    for (side_index, side) in state.sides.iter().enumerate() {
        for (slot_index, pokemon) in side.pokemon.iter().enumerate() {
            let ability_id = pokemon.ability;
            let position = BattlePosition::new(
                if side_index == 0 { SideReference::SideOne } else { SideReference::SideTwo },
                slot_index
            );
            
            let context = AbilityContext {
                user_position: position,
                target_position: None,
                move_type: None,
                move_id: None,
                base_power: None,
                is_critical: false,
                is_contact: false,
                state: state,
            };
            
            if apply_ability_effect(&ability_id, context).negates_weather {
                return true;
            }
        }
    }
    false
}

/// Calculate weather-based stat multipliers (Sandstorm SpDef for Rock, Snow Def for Ice)
pub fn get_weather_stat_multiplier(
    state: &BattleState,
    weather: &Weather,
    pokemon: &Pokemon,
    stat: Stat,
) -> f32 {
    // Check if weather is negated by Cloud Nine or Air Lock
    if is_weather_negated(state) {
        return 1.0;
    }

    match weather {
        Weather::Sand => {
            // Sandstorm boosts Special Defense of Rock types by 1.5x
            if stat == Stat::SpecialDefense
                && pokemon.types.iter().any(|t| *t == crate::types::PokemonType::Rock)
            {
                1.5
            } else {
                1.0
            }
        }
        Weather::Snow => {
            // Snow boosts Defense of Ice types by 1.5x
            if stat == Stat::Defense
                && pokemon.types.iter().any(|t| *t == crate::types::PokemonType::Ice)
            {
                1.5
            } else {
                1.0
            }
        }
        _ => 1.0,
    }
}

/// Calculate weather-based damage modifiers for moves
pub fn get_weather_damage_modifier(
    state: &BattleState,
    weather: &Weather,
    move_type: &str,
    _generation_mechanics: &GenerationMechanics,
) -> f32 {
    // Check if weather is negated by Cloud Nine or Air Lock
    if is_weather_negated(state) {
        return 1.0;
    }

    match weather {
        Weather::Sun => match crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
            Some(crate::types::PokemonType::Fire) => 1.5,
            Some(crate::types::PokemonType::Water) => 0.5,
            _ => 1.0,
        },
        Weather::Rain => match crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
            Some(crate::types::PokemonType::Water) => 1.5,
            Some(crate::types::PokemonType::Fire) => 0.5,
            _ => 1.0,
        },
        Weather::HarshSunlight | Weather::HarshSun => {
            match crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
                Some(crate::types::PokemonType::Fire) => 1.5,
                Some(crate::types::PokemonType::Water) => 0.0, // Water moves fail in harsh sun
                _ => 1.0,
            }
        }
        Weather::HeavyRain => {
            match crate::types::PokemonType::from_normalized_str(&move_type.to_lowercase()) {
                Some(crate::types::PokemonType::Water) => 1.5,
                Some(crate::types::PokemonType::Fire) => 0.0, // Fire moves fail in heavy rain
                _ => 1.0,
            }
        }
        Weather::Sand
        | Weather::Sandstorm
        | Weather::Hail
        | Weather::Snow
        | Weather::StrongWinds
        | Weather::None => 1.0,
    }
}