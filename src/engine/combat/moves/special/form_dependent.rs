//! # Form-Dependent Move Effects

//! 
//! This module contains moves that change type or effect based on the user's form.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{SideCondition};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, FieldInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::moves::apply_generic_effects;
use std::collections::HashMap;
use crate::core::instructions::Stat;
use crate::core::instructions::StatsInstruction;
use crate::data::showdown_types::MoveData;

// =============================================================================
// FORM-DEPENDENT MOVES
// =============================================================================

/// Apply Aura Wheel - Electric/Dark type based on Morpeko form, always boosts Speed
pub fn apply_aura_wheel(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Check Morpeko form to determine type
        let normalized_species = crate::utils::normalize_name(&user_pokemon.species);
        let modified_move_type = if normalized_species.contains("hangry") {
            "Dark" // Hangry Mode Morpeko
        } else {
            "Electric" // Full Belly Mode Morpeko (default)
        };
        
        // Create modified move data with form-based type
        let modified_move_data = MoveData {
            move_type: modified_move_type.to_string(),
            ..move_data.clone()
        };
        
        // Apply move effects with boosted Speed (Aura Wheel always boosts Speed by 1 stage)
        let mut instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation, branch_on_damage);
        
        // Add Speed boost
        let mut speed_boost = HashMap::new();
        speed_boost.insert(Stat::Speed, 1);
        
        instructions.push(BattleInstructions::new(100.0, vec![
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: user_position,
                stat_changes: speed_boost,
                previous_boosts: HashMap::new(),
            }),
        ]));
        
        instructions
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
    }
}

/// Apply Raging Bull - type varies by Tauros form, breaks screens and has double power against them
pub fn apply_raging_bull(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Determine type based on Tauros form
        let normalized_species = crate::utils::normalize_name(&user_pokemon.species);
        let modified_move_type = match normalized_species.as_str() {
            s if s.contains("tauros") && s.contains("combat") => "Fighting", // Paldean Combat Form
            s if s.contains("tauros") && s.contains("blaze") => "Fire",     // Paldean Blaze Form
            s if s.contains("tauros") && s.contains("aqua") => "Water",     // Paldean Aqua Form
            _ => &move_data.move_type, // Regular Tauros keeps Normal type
        };
        
        // Check if screens are present on the target's side to boost power
        let power_multiplier = if !target_positions.is_empty() {
            let target_side = state.get_side_by_ref(target_positions[0].side);
            if target_side.side_conditions.contains_key(&SideCondition::Reflect) ||
               target_side.side_conditions.contains_key(&SideCondition::LightScreen) {
                2.0 // Double power against screens
            } else {
                1.0
            }
        } else {
            1.0
        };
        
        // Create modified move data
        let mut modified_move_data = MoveData {
            move_type: modified_move_type.to_string(),
            ..move_data.clone()
        };
        
        // Apply power multiplier if screens are present
        if power_multiplier > 1.0 {
            if modified_move_data.base_power > 0 {
                modified_move_data.base_power = ((modified_move_data.base_power as f32 * power_multiplier) as u16);
            }
        }
        
        // Apply move effects
        let mut instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation, branch_on_damage);
        
        // Remove screens after hitting (screen-breaking effect)
        if !target_positions.is_empty() {
            let target_side_ref = target_positions[0].side;
            let target_side = state.get_side_by_ref(target_side_ref);
            
            // Remove Reflect
            if target_side.side_conditions.contains_key(&SideCondition::Reflect) {
                instructions.push(BattleInstructions::new(100.0, vec![
                    BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                        side: target_side_ref,
                        condition: SideCondition::Reflect,
                        previous_duration: 0,
                    }),
                ]));
            }
            
            // Remove Light Screen
            if target_side.side_conditions.contains_key(&SideCondition::LightScreen) {
                instructions.push(BattleInstructions::new(100.0, vec![
                    BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
                        side: target_side_ref,
                        condition: SideCondition::LightScreen,
                        previous_duration: 0,
                    }),
                ]));
            }
        }
        
        instructions
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
    }
}