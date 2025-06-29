//! # Priority Move Effects

//! 
//! This module contains priority move implementations. Most priority moves have
//! no special effects beyond their priority value, which is handled by the PS data.

use crate::core::battle_state::BattleState;
use crate::core::instructions::VolatileStatus;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;

// =============================================================================
// PRIORITY MOVES
// =============================================================================

/// Apply Accelerock - Rock-type priority move
pub fn apply_accelerock(
    _state: &BattleState,
    _move_data: &MoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Priority is handled by PS data, no special effects
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Aqua Jet - Water-type priority move
pub fn apply_aqua_jet(
    _state: &BattleState,
    _move_data: &MoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Bullet Punch - Steel-type priority move
pub fn apply_bullet_punch(
    _state: &BattleState,
    _move_data: &MoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Extreme Speed - +2 priority Normal move
pub fn apply_extreme_speed(
    _state: &BattleState,
    _move_data: &MoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Fake Out - flinches, only works on first turn
pub fn apply_fake_out(
    _state: &BattleState,
    _move_data: &MoveData,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Apply flinch to targets (damage is handled separately)
    for &target_position in target_positions {
        let instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
            target: target_position,
            status: VolatileStatus::Flinch,
            duration: Some(1),
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

/// Apply Feint - breaks through protection
pub fn apply_feint(
    state: &BattleState,
    _move_data: &MoveData,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Feint removes protection from targets
    for &target_position in target_positions {
        if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
            let mut instruction_list = Vec::new();
            
            // Remove Protect status
            if target_pokemon.volatile_statuses.contains(VolatileStatus::Protect) {
                instruction_list.push(BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: target_position,
                    status: VolatileStatus::Protect,
                    previous_duration: None,
                }));
            }
            
            // Remove other protection statuses
            if target_pokemon.volatile_statuses.contains(VolatileStatus::Endure) {
                instruction_list.push(BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: target_position,
                    status: VolatileStatus::Endure,
                    previous_duration: None,
                }));
            }
            
            if !instruction_list.is_empty() {
                instructions.push(BattleInstructions::new(100.0, instruction_list));
            }
        }
    }
    
    instructions
}

/// Apply First Impression - Bug-type priority, only works on first turn
pub fn apply_first_impression(
    _state: &BattleState,
    _move_data: &MoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // First Impression only works on the first turn the Pokemon is on the field
    // For now, we'll implement basic logic - in a full implementation, 
    // we'd need to track turn count since Pokemon entered battle
    
    // This is a priority move with no special effects beyond damage
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Mach Punch - Fighting-type priority move
pub fn apply_mach_punch(
    _state: &BattleState,
    _move_data: &MoveData,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    vec![BattleInstructions::new(100.0, vec![])]
}