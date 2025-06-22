//! # Hazard Move Effects
//! 
//! This module contains entry hazard moves that set field conditions affecting
//! Pokemon when they switch in. These are critical for competitive play.

use crate::core::battle_state::BattleState;
use crate::core::instructions::SideCondition;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, FieldInstruction,
};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;

// =============================================================================
// HAZARD SETTING MOVES
// =============================================================================

/// Apply Spikes - sets entry hazard that damages grounded Pokemon
pub fn apply_spikes(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Target the opposing side
    let target_side = match user_position.side {
        SideReference::SideOne => SideReference::SideTwo,
        SideReference::SideTwo => SideReference::SideOne,
    };
    
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side: target_side,
        condition: SideCondition::Spikes,
        duration: 0, // Permanent until removed
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Stealth Rock - sets entry hazard based on type effectiveness  
pub fn apply_stealth_rock(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_side = match user_position.side {
        SideReference::SideOne => SideReference::SideTwo,
        SideReference::SideTwo => SideReference::SideOne,
    };
    
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side: target_side,
        condition: SideCondition::StealthRock,
        duration: 0,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Toxic Spikes - sets entry hazard that poisons
pub fn apply_toxic_spikes(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_side = match user_position.side {
        SideReference::SideOne => SideReference::SideTwo,
        SideReference::SideTwo => SideReference::SideOne,
    };
    
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side: target_side,
        condition: SideCondition::ToxicSpikes,
        duration: 0,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}

/// Apply Sticky Web - sets entry hazard that lowers Speed
pub fn apply_sticky_web(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_side = match user_position.side {
        SideReference::SideOne => SideReference::SideTwo,
        SideReference::SideTwo => SideReference::SideOne,
    };
    
    let instruction = BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side: target_side,
        condition: SideCondition::StickyWeb,
        duration: 0,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![instruction])]
}