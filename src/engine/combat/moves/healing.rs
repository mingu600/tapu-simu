//! # Healing Move Effects
//! 
//! This module contains healing move effects extracted from move_effects.rs.
//! These functions handle moves that restore HP or provide healing effects.

use crate::core::battle_state::{Pokemon, MoveCategory};
use crate::core::battle_state::BattleState;
use crate::core::instructions::{PokemonStatus, VolatileStatus, Stat, Weather, SideCondition, Terrain};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction,
    FieldInstruction, StatsInstruction,
};
use crate::data::Repository;
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
use std::collections::HashMap;

// =============================================================================
// HEALING MOVES
// =============================================================================

/// Apply Recover - restores 50% of max HP
pub fn apply_recover(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let heal_amount = pokemon.max_hp / 2;
        let instruction = BattleInstruction::Pokemon(PokemonInstruction::Heal {
                target: target_position,
                amount: heal_amount,
                previous_hp: Some(0), // This should be set to actual previous HP
            });
        vec![BattleInstructions::new(100.0, vec![instruction])]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Roost - restores 50% of max HP
pub fn apply_roost(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recover(state, user_position, target_positions, generation)
}

/// Apply Moonlight - restores HP based on weather
/// Generation-aware: Weather effects and amounts may vary by generation
pub fn apply_moonlight(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let heal_amount = match state.weather() {
            Weather::Sun | Weather::HarshSun | Weather::HarshSunlight => {
                (pokemon.max_hp * 2) / 3 // 66% in sun
            }
            Weather::Rain | Weather::Sand | Weather::Sandstorm |
            Weather::Hail | Weather::Snow | Weather::HeavyRain | Weather::StrongWinds => {
                pokemon.max_hp / 4 // 25% in other weather
            }
            Weather::None => pokemon.max_hp / 2, // 50% in clear weather
        };
        
        let instruction = BattleInstruction::Pokemon(PokemonInstruction::Heal {
                target: target_position,
                amount: heal_amount,
                previous_hp: Some(0), // This should be set to actual previous HP
            });
        vec![BattleInstructions::new(100.0, vec![instruction])]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Synthesis - restores HP based on weather (same as Moonlight)
pub fn apply_synthesis(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_moonlight(state, user_position, target_positions, generation)
}

/// Apply Morning Sun - restores HP based on weather (same as Moonlight)
pub fn apply_morning_sun(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_moonlight(state, user_position, target_positions, generation)
}

/// Apply Soft-Boiled - restores 50% of max HP
pub fn apply_soft_boiled(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recover(state, user_position, target_positions, generation)
}

/// Apply Milk Drink - restores 50% of max HP
pub fn apply_milk_drink(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recover(state, user_position, target_positions, generation)
}

/// Apply Slack Off - restores 50% of max HP
pub fn apply_slack_off(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recover(state, user_position, target_positions, generation)
}

/// Apply Aqua Ring - provides gradual HP recovery
pub fn apply_aqua_ring(
    _state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
        target: target_position,
        status: VolatileStatus::AquaRing,
        duration: None, // Lasts until Pokemon switches out
        previous_had_status: false,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Shore Up - healing move enhanced in sand weather
pub fn apply_shore_up(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    if let Some(pokemon) = state.get_pokemon_at_position(target_position) {
        let heal_amount = match state.weather() {
            Weather::Sand | Weather::Sandstorm => {
                (pokemon.max_hp * 2) / 3 // 66% in sand
            }
            Weather::Sun | Weather::HarshSun | Weather::HarshSunlight |
            Weather::Rain | Weather::Hail | Weather::Snow | 
            Weather::HeavyRain | Weather::StrongWinds | Weather::None => {
                pokemon.max_hp / 2 // 50% normally
            }
        };
        
        let instruction = BattleInstruction::Pokemon(PokemonInstruction::Heal {
            target: target_position,
            amount: heal_amount,
            previous_hp: Some(0),
        });
        vec![BattleInstructions::new(100.0, vec![instruction])]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}