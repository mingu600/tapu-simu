//! # Type Removal Move Effects

//! 
//! This module contains moves that remove one of the user's types after use.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;
use crate::types::PokemonType;
use crate::engine::combat::moves::apply_generic_effects;

// =============================================================================
// TYPE REMOVAL MOVES
// =============================================================================

/// Apply Burn Up - removes user's Fire type after use
pub fn apply_burn_up(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Apply damage first
    instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation, false));
    
    // Remove Fire type from user
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let mut new_types = user_pokemon.types.clone();
        new_types.retain(|t| *t != PokemonType::Fire);
        
        // If user becomes typeless, make them Normal type
        if new_types.is_empty() {
            new_types.push(PokemonType::Normal);
        }
        
        instructions.push(BattleInstructions::new(100.0, vec![
            BattleInstruction::Pokemon(PokemonInstruction::ChangeType {
                target: user_position,
                new_types: new_types.iter().map(|t| t.to_normalized_str().to_string()).collect(),
                previous_types: user_pokemon.types.iter().map(|t| t.to_normalized_str().to_string()).collect(),
            })
        ]));
    }
    
    instructions
}

/// Apply Double Shock - removes user's Electric type after use
pub fn apply_double_shock(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Apply damage first
    instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation, false));
    
    // Remove Electric type from user
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let mut new_types = user_pokemon.types.clone();
        new_types.retain(|t| *t != PokemonType::Electric);
        
        // If user becomes typeless, make them Normal type
        if new_types.is_empty() {
            new_types.push(PokemonType::Normal);
        }
        
        instructions.push(BattleInstructions::new(100.0, vec![
            BattleInstruction::Pokemon(PokemonInstruction::ChangeType {
                target: user_position,
                new_types: new_types.iter().map(|t| t.to_normalized_str().to_string()).collect(),
                previous_types: user_pokemon.types.iter().map(|t| t.to_normalized_str().to_string()).collect(),
            })
        ]));
    }
    
    instructions
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

