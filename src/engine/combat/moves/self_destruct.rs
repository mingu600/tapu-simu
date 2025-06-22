//! # Self-Destruct Move Effects

//! 
//! This module contains moves that cause the user to faint after use.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;

// =============================================================================
// SELF-DESTRUCT MOVES
// =============================================================================

/// Apply Explosion - high power, user faints
pub fn apply_explosion(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Apply damage first
    instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation));
    
    // Get the user's current HP before fainting
    let user_current_hp = state.get_pokemon_at_position(user_position)
        .map(|pokemon| pokemon.hp)
        .unwrap_or(0);
    
    // User faints
    instructions.push(BattleInstructions::new(100.0, vec![
        BattleInstruction::Pokemon(PokemonInstruction::Faint {
            target: user_position,
            previous_hp: user_current_hp,
            previous_status: None,
        }),
    ]));
    
    instructions
}

/// Apply Self-Destruct - identical to Explosion but lower power
pub fn apply_self_destruct(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Self-Destruct has identical mechanics to Explosion, just different base power
    apply_explosion(state, move_data, user_position, target_positions, generation)
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