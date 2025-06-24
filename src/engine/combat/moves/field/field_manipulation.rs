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

/// Apply Tidy Up - removes hazards and substitutes from both sides, then boosts user's Attack and Speed
pub fn apply_tidy_up(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Remove all hazards from both sides
    for side_ref in [SideReference::SideOne, SideReference::SideTwo] {
        // Remove Spikes
        if state.get_side_by_ref(side_ref).side_conditions.contains_key(&SideCondition::Spikes) {
            instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                side: side_ref,
                condition: SideCondition::Spikes,
                previous_duration: 0,
            }));
        }
        
        // Remove Stealth Rock
        if state.get_side_by_ref(side_ref).side_conditions.contains_key(&SideCondition::StealthRock) {
            instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                side: side_ref,
                condition: SideCondition::StealthRock,
                previous_duration: 0,
            }));
        }
        
        // Remove Toxic Spikes
        if state.get_side_by_ref(side_ref).side_conditions.contains_key(&SideCondition::ToxicSpikes) {
            instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                side: side_ref,
                condition: SideCondition::ToxicSpikes,
                previous_duration: 0,
            }));
        }
        
        // Remove Sticky Web
        if state.get_side_by_ref(side_ref).side_conditions.contains_key(&SideCondition::StickyWeb) {
            instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                side: side_ref,
                condition: SideCondition::StickyWeb,
                previous_duration: 0,
            }));
        }
    }
    
    // Remove substitutes from all Pokemon
    for side_ref in [SideReference::SideOne, SideReference::SideTwo] {
        let side = state.get_side_by_ref(side_ref);
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                if pokemon.volatile_statuses.contains(&VolatileStatus::Substitute) {
                    let position = BattlePosition::new(side_ref, slot);
                    instructions.push(BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                        target: position,
                        status: VolatileStatus::Substitute,
                        previous_duration: None,
                    }));
                }
            }
        }
    }
    
    // Boost user's Attack and Speed by 1 stage each
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    stat_boosts.insert(Stat::Speed, 1);
    
    instructions.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
        target: user_position,
        stat_changes: stat_boosts,
        previous_boosts: HashMap::new(),
    }));
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Court Change - swaps all hazards and side conditions between the two sides
pub fn apply_court_change(
    state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    let side_one_conditions = &state.sides[0].side_conditions;
    let side_two_conditions = &state.sides[1].side_conditions;
    
    // Clone the conditions before modifying
    let side_one_conditions_clone = side_one_conditions.clone();
    let side_two_conditions_clone = side_two_conditions.clone();
    
    // Remove all conditions from both sides first
    for (condition, _) in &side_one_conditions_clone {
        instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
            side: SideReference::SideOne,
            condition: *condition,
            previous_duration: 0,
        }));
    }
    
    for (condition, _) in &side_two_conditions_clone {
        instructions.push(BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
            side: SideReference::SideTwo,
            condition: *condition,
            previous_duration: 0,
        }));
    }
    
    // Apply side one's conditions to side two
    for (condition, &duration) in &side_one_conditions_clone {
        instructions.push(BattleInstruction::Field(FieldInstruction::ApplySideCondition {
            side: SideReference::SideTwo,
            condition: *condition,
            duration: duration,
            previous_duration: None,
        }));
    }
    
    // Apply side two's conditions to side one  
    for (condition, &duration) in &side_two_conditions_clone {
        instructions.push(BattleInstruction::Field(FieldInstruction::ApplySideCondition {
            side: SideReference::SideOne,
            condition: *condition,
            duration: duration,
            previous_duration: None,
        }));
    }
    
    vec![BattleInstructions::new(100.0, instructions)]
}

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