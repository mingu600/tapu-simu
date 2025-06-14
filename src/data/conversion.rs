//! # Data Conversion Utilities
//! 
//! This module provides conversion utilities between rustemon types and
//! engine-optimized types.

use rustemon::model::pokemon::Pokemon as RustemonPokemon;
use rustemon::model::moves::Move as RustemonMove;
use crate::data::types::{EnginePokemonData, EngineBaseStats, EngineMoveData, MoveTarget};
use crate::state::MoveCategory;

/// Convert rustemon Pokemon data to engine Pokemon data
pub fn rustemon_pokemon_to_engine(pokemon: RustemonPokemon) -> EnginePokemonData {
    EnginePokemonData {
        id: pokemon.id as i32,
        name: pokemon.name.clone(),
        base_stats: EngineBaseStats {
            hp: pokemon.stats.iter()
                .find(|s| s.stat.name == "hp")
                .map(|s| s.base_stat as i16)
                .unwrap_or(50),
            attack: pokemon.stats.iter()
                .find(|s| s.stat.name == "attack")
                .map(|s| s.base_stat as i16)
                .unwrap_or(50),
            defense: pokemon.stats.iter()
                .find(|s| s.stat.name == "defense")
                .map(|s| s.base_stat as i16)
                .unwrap_or(50),
            special_attack: pokemon.stats.iter()
                .find(|s| s.stat.name == "special-attack")
                .map(|s| s.base_stat as i16)
                .unwrap_or(50),
            special_defense: pokemon.stats.iter()
                .find(|s| s.stat.name == "special-defense")
                .map(|s| s.base_stat as i16)
                .unwrap_or(50),
            speed: pokemon.stats.iter()
                .find(|s| s.stat.name == "speed")
                .map(|s| s.base_stat as i16)
                .unwrap_or(50),
        },
        types: pokemon.types.iter()
            .map(|t| t.type_.name.clone())
            .collect(),
        abilities: pokemon.abilities.iter()
            .map(|a| a.ability.name.clone())
            .collect(),
        moves: pokemon.moves.iter()
            .map(|m| m.move_.name.clone())
            .collect(),
        height: pokemon.height as i32,
        weight: pokemon.weight as i32,
    }
}

/// Convert rustemon move data to engine move data
pub fn rustemon_move_to_engine(move_data: RustemonMove) -> EngineMoveData {
    EngineMoveData {
        id: move_data.id as i32,
        name: move_data.name.clone(),
        base_power: move_data.power.map(|p| p as i16),
        accuracy: move_data.accuracy.map(|a| a as i16),
        pp: move_data.pp.map(|p| p as i16).unwrap_or(10),
        move_type: move_data.type_.name.clone(),
        category: match move_data.damage_class.name.as_str() {
            "physical" => MoveCategory::Physical,
            "special" => MoveCategory::Special,
            "status" => MoveCategory::Status,
            _ => MoveCategory::Status,
        },
        priority: move_data.priority as i8,
        target: convert_move_target(&move_data.target.name),
        effect_chance: move_data.effect_chance.map(|c| c as i16),
        effect_description: move_data.effect_entries.first()
            .map(|e| e.short_effect.clone())
            .unwrap_or_default(),
        flags: move_data.meta.as_ref()
            .map(|_meta| {
                // Convert meta flags to string representation
                Vec::new() // TODO: Implement proper flag conversion
            })
            .unwrap_or_default(),
    }
}

/// Convert rustemon move target to engine move target
fn convert_move_target(target_name: &str) -> MoveTarget {
    match target_name {
        "specific-move" => MoveTarget::SpecificMove,
        "selected-pokemon-me-first" => MoveTarget::SpecificMove,
        "ally" => MoveTarget::Ally,
        "users-field" => MoveTarget::UsersField,
        "user-or-ally" => MoveTarget::UserOrAlly,
        "opponents-field" => MoveTarget::OpponentsField,
        "user" => MoveTarget::User,
        "random-opponent" => MoveTarget::RandomOpponent,
        "all-other-pokemon" => MoveTarget::AllOtherPokemon,
        "selected-pokemon" => MoveTarget::SpecificMove,
        "all-opponents" => MoveTarget::AllOpponents,
        "entire-field" => MoveTarget::EntireField,
        "user-and-allies" => MoveTarget::UserAndAllies,
        "all-pokemon" => MoveTarget::AllPokemon,
        _ => MoveTarget::SpecificMove, // Default fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_target_conversion() {
        assert_eq!(convert_move_target("specific-move"), MoveTarget::SpecificMove);
        assert_eq!(convert_move_target("all-other-pokemon"), MoveTarget::AllOtherPokemon);
        assert_eq!(convert_move_target("user"), MoveTarget::User);
        assert_eq!(convert_move_target("unknown-target"), MoveTarget::SelectedPokemon); // Default fallback
    }
}