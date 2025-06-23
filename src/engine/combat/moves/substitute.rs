//! # Substitute Move Effects
//! 
//! This module contains the substitute move implementation.

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
// SUBSTITUTE MOVE
// =============================================================================

/// Apply Substitute - creates a substitute that absorbs damage
pub fn apply_substitute(
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
        // Check if Pokemon has enough HP (need at least 25% max HP)
        let cost = pokemon.max_hp / 4;
        if pokemon.hp > cost {
            let mut instructions = Vec::new();
            
            // Damage user for 25% of max HP
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: cost,
                previous_hp: Some(pokemon.hp),
            }));
            
            // Apply substitute volatile status
            instructions.push(BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                target: target_position,
                status: VolatileStatus::Substitute,
                duration: None, // Lasts until broken
                previous_had_status: false,
                previous_duration: None,
            }));
            
            vec![BattleInstructions::new(100.0, instructions)]
        } else {
            // Not enough HP - move fails
            vec![BattleInstructions::new(100.0, vec![])]
        }
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}