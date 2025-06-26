//! # Field Manipulation Move Effects
//! 
//! This module contains moves that manipulate the battle field, including
//! hazard removal, condition swapping, and weather setting with additional effects.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{SideCondition, VolatileStatus, Stat, Weather};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, FieldInstruction, StatusInstruction, StatsInstruction,
};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use std::collections::HashMap;

// =============================================================================
// FIELD MANIPULATION MOVES
// =============================================================================


/// Apply Chilly Reception - sets Snow weather for 5 turns and forces user to switch
pub fn apply_chilly_reception(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Set Snow weather (5 turns)
    instructions.push(BattleInstruction::Field(FieldInstruction::Weather {
        new_weather: Weather::Snow,
        previous_weather: state.weather(),
        turns: Some(5),
        previous_turns: state.field.weather.turns_remaining,
        source: None,
    }));
    
    // Force the user to switch out - apply MustSwitch volatile status
    instructions.push(BattleInstruction::Status(StatusInstruction::ApplyVolatile {
        target: user_position,
        status: VolatileStatus::MustSwitch,
        duration: Some(1),
        previous_had_status: false,
        previous_duration: None,
    }));
    
    vec![BattleInstructions::new(100.0, instructions)]
}