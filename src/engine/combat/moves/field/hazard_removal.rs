//! # Hazard Removal Move Effects
//! 
//! This module contains moves that remove entry hazards from the field.
//! These are essential for competitive play and hazard management.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{SideCondition, Stat};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, FieldInstruction, StatsInstruction,
};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use std::collections::HashMap;

/// Helper functions to create common stat boost patterns efficiently
fn single_stat_boost(stat: Stat, boost: i8) -> HashMap<Stat, i8> {
    let mut map = HashMap::with_capacity(1);
    map.insert(stat, boost);
    map
}

fn dual_stat_boost(stat1: Stat, boost1: i8, stat2: Stat, boost2: i8) -> HashMap<Stat, i8> {
    let mut map = HashMap::with_capacity(2);
    map.insert(stat1, boost1);
    map.insert(stat2, boost2);
    map
}

// =============================================================================
// HAZARD REMOVAL MOVES
// =============================================================================

/// Apply Rapid Spin - removes hazards from user's side
pub fn apply_rapid_spin(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::with_capacity(4); // Pre-allocate for 4 hazard types
    
    // Remove hazards from user's side
    for condition in [SideCondition::Spikes, SideCondition::StealthRock, SideCondition::ToxicSpikes, SideCondition::StickyWeb] {
        instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
            side: user_position.side,
            condition,
            previous_duration: 0, // Default assumption
        }));
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Defog - removes hazards from both sides and lowers target's evasion
pub fn apply_defog(
    _state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Remove hazards from both sides
    for side in [SideReference::SideOne, SideReference::SideTwo] {
        for condition in [SideCondition::Spikes, SideCondition::StealthRock, SideCondition::ToxicSpikes, SideCondition::StickyWeb] {
            instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                side,
                condition,
                previous_duration: 0, // Default assumption
            }));
        }
    }
    
    // Lower target's evasion by 1 stage
    for &target_position in target_positions {
        instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: target_position,
            stat_changes: single_stat_boost(Stat::Evasion, -1),
            previous_boosts: HashMap::new(),
        }));
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Court Change - swaps all field effects between sides
pub fn apply_court_change(
    state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Get side conditions from both sides
    let side_one_conditions = state.sides[0].side_conditions.clone();
    let side_two_conditions = state.sides[1].side_conditions.clone();
    
    // Swap hazards between sides
    // Remove all hazards from side one and apply side two's hazards
    for (condition, value) in &side_one_conditions {
        instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
            side: SideReference::SideOne,
            condition: *condition,
            previous_duration: *value,
        }));
    }
    
    for (condition, value) in &side_two_conditions {
        instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
            side: SideReference::SideTwo,
            condition: *condition,
            previous_duration: *value,
        }));
    }
    
    // Apply swapped conditions
    for (condition, value) in &side_two_conditions {
        instructions.push(BattleInstruction::Field(FieldInstruction::ApplySideCondition {
            side: SideReference::SideOne,
            condition: *condition,
            duration: *value,
            previous_duration: None,
        }));
    }
    
    for (condition, value) in &side_one_conditions {
        instructions.push(BattleInstruction::Field(FieldInstruction::ApplySideCondition {
            side: SideReference::SideTwo,
            condition: *condition,
            duration: *value,
            previous_duration: None,
        }));
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Tidy Up - removes hazards and substitutes, raises Attack and Speed
pub fn apply_tidy_up(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Remove hazards from both sides
    for side in [SideReference::SideOne, SideReference::SideTwo] {
        for condition in [SideCondition::Spikes, SideCondition::StealthRock, SideCondition::ToxicSpikes, SideCondition::StickyWeb] {
            instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                side,
                condition,
                previous_duration: 0,
            }));
        }
    }
    
    // Boost user's Attack and Speed by 1 stage each
    instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: user_position,
        stat_changes: dual_stat_boost(Stat::Attack, 1, Stat::Speed, 1),
        previous_boosts: HashMap::new(),
    }));
    
    vec![BattleInstructions::new(100.0, instructions)]
}