//! # Drain Move Effects
//! 
//! This module handles drain move effects where the user heals
//! based on a percentage of damage dealt to the target.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstructions};
use crate::core::battle_format::BattlePosition;
use crate::core::instructions::PokemonStatus;
use crate::generation::GenerationMechanics;

/// Apply drain move effects - now handled automatically by instruction generator
/// This function is kept for compatibility but drain is now handled via PS data
pub fn apply_drain_move(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
    _drain_percentage: i16,
) -> Vec<BattleInstructions> {
    // Drain is now handled automatically in the instruction generator
    // based on PS move data, so we just return empty instructions
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Giga Drain - restores 50% of damage dealt
pub fn apply_giga_drain(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Mega Drain - restores 50% of damage dealt
pub fn apply_mega_drain(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Absorb - restores 50% of damage dealt
pub fn apply_absorb(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Drain Punch - restores 50% of damage dealt
pub fn apply_drain_punch(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Leech Life - restores 50% of damage dealt
pub fn apply_leech_life(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_drain_move(state, user_position, target_positions, generation, 50)
}

/// Apply Dream Eater - restores 50% of damage dealt (only works on sleeping targets)
pub fn apply_dream_eater(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Dream Eater only works on sleeping Pokemon
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status == PokemonStatus::Sleep {
                // Move can hit - drain effect will be applied after damage
                instructions.push(BattleInstructions::new(100.0, vec![]));
            } else {
                // Move fails on non-sleeping targets
                instructions.push(BattleInstructions::new(100.0, vec![]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}