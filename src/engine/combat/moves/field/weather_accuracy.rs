//! # Weather-Dependent Accuracy Move Effects

//! 
//! This module contains moves that have perfect accuracy in certain weather conditions.

use crate::core::battle_state::BattleState;
use crate::core::instructions::Weather;
use crate::core::instructions::BattleInstructions;
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;

// =============================================================================
// WEATHER-DEPENDENT ACCURACY MOVES
// =============================================================================

/// Apply Blizzard - weather accuracy is handled automatically by the turn system
pub fn apply_blizzard(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Weather-based accuracy (perfect accuracy in hail/snow) is automatically
    // handled by the turn generation system's calculate_move_accuracy function
    apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
}

/// Apply Hurricane - weather accuracy is handled automatically by the turn system
pub fn apply_hurricane(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Weather-based accuracy (perfect accuracy in rain, reduced in sun) is automatically
    // handled by the turn generation system's calculate_move_accuracy function
    apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
}

/// Apply Thunder - weather accuracy is handled automatically by the turn system
pub fn apply_thunder(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Weather-based accuracy (perfect accuracy in rain, reduced in sun) is automatically
    // handled by the turn generation system's calculate_move_accuracy function
    apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
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
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Use the proper generic effects implementation from main module
    crate::engine::combat::moves::apply_generic_effects(
        state, move_data, user_position, target_positions, generation, branch_on_damage
    )
}