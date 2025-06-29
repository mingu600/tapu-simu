//! # Utility and Field Effect Move Effects
//! 
//! This module contains utility moves and field manipulation effects.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{PokemonStatus, VolatileStatus, Stat};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction, StatsInstruction,
};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::types::StatBoostArray;
use std::collections::HashMap;

// =============================================================================
// UTILITY AND FIELD EFFECT MOVES
// =============================================================================

/// Apply Aromatherapy - heals status conditions of all team members
pub fn apply_aromatherapy(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    let user_side = user_position.side;
    
    // Clear status from all Pokemon on user's team
    let side = state.get_side_by_ref(user_side);
    for (slot, pokemon) in side.pokemon.iter().enumerate() {
        if pokemon.status != PokemonStatus::None {
            let position = BattlePosition::new(user_side, slot);
            instructions.push(BattleInstruction::Status(StatusInstruction::Apply {
                target: position,
                status: PokemonStatus::None,
                duration: None,
                previous_status: Some(pokemon.status),
                previous_duration: pokemon.status_duration,
            }));
        }
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Heal Bell - same as Aromatherapy
pub fn apply_heal_bell(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_aromatherapy(state, user_position, target_positions, generation)
}

/// Apply Attract - causes infatuation
pub fn apply_attract(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Check if target is already attracted or has immunity (like Oblivious)
            if !target.volatile_statuses.contains(VolatileStatus::Attract) {
                let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: target_position,
                    status: VolatileStatus::Attract,
                    duration: None, // Lasts until Pokemon switches out
                    previous_had_status: false,
                    previous_duration: None,
                });
                instructions.push(BattleInstructions::new(100.0, vec![instruction]));
            } else {
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Confuse Ray - causes confusion
pub fn apply_confuse_ray(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
            target: target_position,
            status: VolatileStatus::Confusion,
            duration: Some(4), // Lasts 2-5 turns in most generations
            previous_had_status: false,
            previous_duration: None,
        });
        instructions.push(BattleInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Haze - resets all stat changes for all Pokemon
pub fn apply_haze(
    state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Reset stat boosts for all active Pokemon
    for side_ref in [SideReference::SideOne, SideReference::SideTwo] {
        for slot in 0..state.format.active_pokemon_count() {
            let position = BattlePosition::new(side_ref, slot);
            if let Some(pokemon) = state.get_pokemon_at_position(position) {
                if !pokemon.stat_boosts.is_empty() {
                    let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
                        target: position,
                        stat_changes: std::collections::HashMap::new(), // Reset all to 0
                        previous_boosts: std::collections::HashMap::new(),
                    });
                    instructions.push(instruction);
                }
            }
        }
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Clear Smog - removes all stat changes from target
pub fn apply_clear_smog(
    _state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        let instruction = BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: target_position,
            stat_changes: std::collections::HashMap::new(), // Reset all to 0
            previous_boosts: std::collections::HashMap::new(),
        });
        instructions.push(BattleInstructions::new(100.0, vec![instruction]));
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}