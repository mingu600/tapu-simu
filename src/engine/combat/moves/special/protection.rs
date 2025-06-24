//! # Protection Move Effects
//! 
//! This module contains implementations for protection and defensive moves.

use crate::core::battle_state::BattleState;
use crate::core::instructions::VolatileStatus;
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatusInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;

/// Apply Protect - protects user from most moves this turn
pub fn apply_protect(
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
        status: VolatileStatus::Protect,
        duration: Some(1), // Lasts for the rest of the turn
        previous_had_status: false,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Detect - same as Protect
pub fn apply_detect(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_protect(state, user_position, target_positions, generation)
}

/// Apply Endure - survives any attack with at least 1 HP
pub fn apply_endure(
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
        status: VolatileStatus::Endure,
        duration: Some(1),
        previous_had_status: false,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}