//! # Screen Move Effects
//! 
//! This module contains implementations for defensive screen moves.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{SideCondition, Weather};
use crate::core::instructions::{BattleInstruction, BattleInstructions, FieldInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;

/// Apply Light Screen - reduces Special damage taken
pub fn apply_light_screen(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side: user_position.side,
        condition: SideCondition::LightScreen,
        duration: 5, // 5 turns in most generations
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Reflect - reduces Physical damage taken
pub fn apply_reflect_move(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side: user_position.side,
        condition: SideCondition::Reflect,
        duration: 5,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Aurora Veil - combines Light Screen and Reflect effects (only in hail/snow)
pub fn apply_aurora_veil(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Aurora Veil only works in hail or snow weather
    match state.weather() {
        Weather::Hail | Weather::Snow => {
            let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
                side: user_position.side,
                condition: SideCondition::AuroraVeil,
                duration: 5,
                previous_duration: None,
            });
            
            vec![BattleInstructions::new(100.0, vec![instruction])]
        }
        _ => {
            // Move fails without hail/snow
            vec![BattleInstructions::new(100.0, vec![])]
        }
    }
}