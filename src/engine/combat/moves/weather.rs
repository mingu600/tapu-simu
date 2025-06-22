//! # Weather Move Effects
//! 
//! This module contains implementations for weather-setting moves.

use crate::core::battle_state::BattleState;
use crate::core::instructions::Weather;
use crate::core::instructions::{BattleInstruction, BattleInstructions, FieldInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;

/// Apply Sunny Day - sets sun weather
pub fn apply_sunny_day(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instruction = BattleInstruction::Field(FieldInstruction::Weather {
        new_weather: Weather::Sun,
        previous_weather: Weather::None,
        turns: Some(5), // 5 turns in most generations
        previous_turns: None,
        source: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Rain Dance - sets rain weather
pub fn apply_rain_dance(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instruction = BattleInstruction::Field(FieldInstruction::Weather {
        new_weather: Weather::Rain,
        previous_weather: Weather::None,
        turns: Some(5),
        previous_turns: None,
        source: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Sandstorm - sets sand weather
pub fn apply_sandstorm(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instruction = BattleInstruction::Field(FieldInstruction::Weather {
        new_weather: Weather::Sand,
        previous_weather: Weather::None,
        turns: Some(5),
        previous_turns: None,
        source: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Hail - sets hail weather
pub fn apply_hail(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instruction = BattleInstruction::Field(FieldInstruction::Weather {
        new_weather: Weather::Hail,
        previous_weather: Weather::None,
        turns: Some(5),
        previous_turns: None,
        source: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Snowscape - sets snow weather (Gen 9+ replacement for Hail)
pub fn apply_snowscape(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let instruction = BattleInstruction::Field(FieldInstruction::Weather {
        new_weather: Weather::Snow,
        previous_weather: Weather::None,
        turns: Some(5),
        previous_turns: None,
        source: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}