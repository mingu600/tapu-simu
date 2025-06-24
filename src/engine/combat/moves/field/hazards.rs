//! # Hazard Move Effects
//! 
//! This module contains entry hazard moves that set field conditions affecting
//! Pokemon when they switch in. These are critical for competitive play.
//!
//! All moves in this module have been converted to use the new composer system.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::composers::field_moves::hazard_setting_move;
use crate::engine::combat::core::field_system::{HazardType, HazardRemovalType, hazard_removal_move};

// =============================================================================
// HAZARD SETTING MACRO
// =============================================================================

/// Macro for simple hazard-setting moves
macro_rules! hazard_move {
    ($func_name:ident, $hazard_type:expr) => {
        pub fn $func_name(
            state: &BattleState,
            user_position: BattlePosition,
            _target_positions: &[BattlePosition],
            _generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            vec![BattleInstructions::new(100.0, hazard_setting_move(state, user_position, $hazard_type))]
        }
    };
}

// =============================================================================
// HAZARD SETTING MOVES
// =============================================================================

/// Apply Spikes - sets entry hazard that damages grounded Pokemon
hazard_move!(apply_spikes, HazardType::Spikes);

/// Apply Stealth Rock - sets entry hazard based on type effectiveness
hazard_move!(apply_stealth_rock, HazardType::StealthRock);

/// Apply Toxic Spikes - sets entry hazard that poisons switching Pokemon
hazard_move!(apply_toxic_spikes, HazardType::ToxicSpikes);

/// Apply Sticky Web - sets entry hazard that lowers Speed of switching Pokemon
hazard_move!(apply_sticky_web, HazardType::StickyWeb);

// =============================================================================
// HAZARD REMOVAL MOVES
// =============================================================================

/// Apply Rapid Spin - removes hazards from user's side and deals damage
pub fn apply_rapid_spin(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // First remove hazards (primary effect)
    instructions.extend(hazard_removal_move(state, user_position, HazardRemovalType::RapidSpin));
    
    // Then deal damage to target (secondary effect)
    if !target_positions.is_empty() {
        // Rapid Spin has 50 base power
        use crate::engine::combat::composers::damage_moves::simple_damage_move;
        use crate::data::showdown_types::MoveData;
        
        let move_data = MoveData {
            name: "Rapid Spin".to_string(),
            base_power: 50,
            move_type: "Normal".to_string(),
            category: "Physical".to_string(),
            accuracy: 100,
            pp: 40,
            max_pp: 64,
            priority: 0,
            target: "Normal".to_string(),
            ..Default::default()
        };
        
        let modifiers = crate::engine::combat::composers::damage_moves::DamageModifiers::default();
        let damage_instructions = simple_damage_move(
            state,
            &move_data,
            user_position,
            target_positions,
            modifiers,
            generation,
        );
        
        // Combine damage instructions with hazard removal
        instructions.extend(damage_instructions);
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Defog - removes hazards from both sides and lowers target's Evasion
pub fn apply_defog(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // First remove hazards from both sides
    instructions.extend(hazard_removal_move(state, user_position, HazardRemovalType::Defog));
    
    // Then lower target's Evasion by 1 stage
    if !target_positions.is_empty() {
        use crate::engine::combat::composers::status_moves::stat_modification_move;
        use crate::core::instructions::Stat;
        use std::collections::HashMap;
        
        let mut stat_changes = HashMap::new();
        stat_changes.insert(Stat::Evasion, -1);
        
        instructions.extend(stat_modification_move(
            state,
            target_positions,
            &stat_changes,
            Some(user_position),
        ));
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
    use crate::core::instructions::{BattleInstruction, FieldInstruction};
    
    let mut instructions = Vec::new();
    
    // Get side conditions from both sides
    let side_one_conditions = state.sides[0].side_conditions.clone();
    let side_two_conditions = state.sides[1].side_conditions.clone();
    
    // Swap hazards between sides
    // Remove all hazards from side one and apply side two's hazards
    for (condition, value) in &side_one_conditions {
        instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
            side: crate::core::battle_format::SideReference::SideOne,
            condition: *condition,
            previous_duration: *value,
        }));
    }
    
    for (condition, value) in &side_two_conditions {
        instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
            side: crate::core::battle_format::SideReference::SideTwo,
            condition: *condition,
            previous_duration: *value,
        }));
    }
    
    // Apply swapped conditions
    for (condition, value) in &side_two_conditions {
        instructions.push(BattleInstruction::Field(FieldInstruction::ApplySideCondition {
            side: crate::core::battle_format::SideReference::SideOne,
            condition: *condition,
            duration: *value,
            previous_duration: None,
        }));
    }
    
    for (condition, value) in &side_one_conditions {
        instructions.push(BattleInstruction::Field(FieldInstruction::ApplySideCondition {
            side: crate::core::battle_format::SideReference::SideTwo,
            condition: *condition,
            duration: *value,
            previous_duration: None,
        }));
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Tidy Up - removes hazards and substitutes, raises Attack and Speed
pub fn apply_tidy_up(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // First remove hazards and substitutes
    instructions.extend(hazard_removal_move(state, user_position, HazardRemovalType::TidyUp));
    
    // Then boost user's Attack and Speed by 1 stage each
    use crate::engine::combat::composers::status_moves::self_stat_boost_move;
    use crate::core::instructions::Stat;
    use std::collections::HashMap;
    
    let mut stat_changes = HashMap::new();
    stat_changes.insert(Stat::Attack, 1);
    stat_changes.insert(Stat::Speed, 1);
    
    instructions.extend(self_stat_boost_move(
        state,
        user_position,
        &stat_changes,
    ));
    
    vec![BattleInstructions::new(100.0, instructions)]
}