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
    let mut instructions = Vec::new();
    
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
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(Stat::Evasion, -1);
        
        instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: target_position,
            stat_changes: stat_boosts,
            previous_boosts: HashMap::new(),
        }));
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}