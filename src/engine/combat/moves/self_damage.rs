//! # Self-Damage Move Effects

//! 
//! This module contains moves that damage the user without fainting them.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;

// =============================================================================
// SELF-DAMAGE MOVES
// =============================================================================

/// Apply Mind Blown - user takes damage equal to half their max HP
pub fn apply_mind_blown(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Apply damage to targets first
    instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation));
    
    // User takes damage equal to half their max HP
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let self_damage = user_pokemon.max_hp / 2;
        
        instructions.push(BattleInstructions::new(100.0, vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: user_position,
                amount: self_damage,
                previous_hp: Some(user_pokemon.hp),
            }),
        ]));
    }
    
    instructions
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Apply generic move effects
fn apply_generic_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Use the proper generic effects implementation from simple module
    crate::engine::combat::moves::simple::apply_generic_effects(
        state, move_data, user_position, target_positions, generation
    )
}