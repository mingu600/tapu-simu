//! # Special Combat Mechanics

//! 
//! This module contains moves with unique combat mechanics that don't fit into other categories.

use crate::core::battle_state::BattleState;
use crate::core::instructions::VolatileStatus;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::moves::apply_generic_effects;
use crate::engine::combat::damage_calc::{calculate_damage_with_positions, DamageRolls, critical_hit_probability};
use crate::data::showdown_types::MoveData;

// Constants to avoid string allocations
const PHYSICAL_CATEGORY: &str = "Physical";
const SPECIAL_CATEGORY: &str = "Special";

// =============================================================================
// SPECIAL COMBAT MECHANICS
// =============================================================================

/// Apply Photon Geyser - uses higher of Attack or Special Attack to determine category
pub fn apply_photon_geyser(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    _branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::damage_moves::dynamic_category_move;
    
    // Use the centralized dynamic category system
    let instructions = dynamic_category_move(
        state,
        move_data,
        user_position,
        target_positions,
        Box::new(|state, user_position| {
            if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
                // Compare Attack vs Special Attack stats to determine category
                let attack_stat = user_pokemon.stats.attack;
                let special_attack_stat = user_pokemon.stats.special_attack;
                
                if attack_stat > special_attack_stat {
                    PHYSICAL_CATEGORY.to_string()
                } else {
                    SPECIAL_CATEGORY.to_string()
                }
            } else {
                SPECIAL_CATEGORY.to_string() // Default fallback
            }
        }),
        generation,
    );
    
    // Handle accuracy
    let accuracy = move_data.accuracy as f32;
    if accuracy < 100.0 {
        vec![
            BattleInstructions::new(100.0 - accuracy, vec![]), // Miss
            BattleInstructions::new(accuracy, instructions),    // Hit
        ]
    } else {
        vec![BattleInstructions::new(100.0, instructions)]
    }
}

/// Apply Sky Drop - Two-turn move that lifts target into the sky
pub fn apply_sky_drop(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Check if user is already in the Sky Drop charging state
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::SkyDrop) {
            // Second turn - attack and remove both Pokemon from sky
            let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage);
            
            // Remove Sky Drop status from user
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                    target: user_position,
                    status: VolatileStatus::SkyDrop,
                    previous_duration: None,
                }),
            ]));
            
            // Remove Sky Drop status from target (if any)
            for &target_position in target_positions {
                if let Some(target) = state.get_pokemon_at_position(target_position) {
                    if target.volatile_statuses.contains(&VolatileStatus::SkyDrop) {
                        instructions.push(BattleInstructions::new(100.0, vec![
                            BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                                target: target_position,
                                status: VolatileStatus::SkyDrop,
                                previous_duration: None,
                            }),
                        ]));
                    }
                }
            }
            
            instructions
        } else {
            // First turn - lift target into sky and apply Sky Drop status to both Pokemon
            let mut instructions = Vec::new();
            
            // Apply Sky Drop status to user
            instructions.push(BattleInstructions::new(100.0, vec![
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: user_position,
                    status: VolatileStatus::SkyDrop,
                    duration: None, // Lasts until second turn
                    previous_had_status: false,
                    previous_duration: None,
                }),
            ]));
            
            // Apply Sky Drop status to target (lifted into sky)
            for &target_position in target_positions {
                if let Some(_target) = state.get_pokemon_at_position(target_position) {
                    instructions.push(BattleInstructions::new(100.0, vec![
                        BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                            target: target_position,
                            status: VolatileStatus::SkyDrop,
                            duration: None, // Lasts until second turn
                            previous_had_status: false,
                            previous_duration: None,
                        }),
                    ]));
                }
            }
            
            instructions
        }
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Body Press - uses Defense stat for damage calculation instead of Attack
pub fn apply_body_press(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    _branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::damage_moves::stat_substitution_move;
    use crate::core::instructions::Stat;
    
    // Use the centralized stat substitution system
    let instructions = stat_substitution_move(
        state,
        move_data,
        user_position,
        target_positions,
        Stat::Defense,  // Use Defense as Attack
        None,           // No defense stat substitution
        false,          // Don't use target stats
        generation,
    );
    
    // Handle accuracy
    let accuracy = move_data.accuracy as f32;
    if accuracy < 100.0 {
        vec![
            BattleInstructions::new(100.0 - accuracy, vec![]), // Miss
            BattleInstructions::new(accuracy, instructions),    // Hit
        ]
    } else {
        vec![BattleInstructions::new(100.0, instructions)]
    }
}



/// Apply Foul Play - uses target's Attack stat for damage calculation instead of user's Attack
pub fn apply_foul_play(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    _branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::damage_moves::stat_substitution_move;
    use crate::core::instructions::Stat;
    
    // Use the centralized stat substitution system
    let instructions = stat_substitution_move(
        state,
        move_data,
        user_position,
        target_positions,
        Stat::Attack,   // Use Attack stat (but from target)
        None,           // No defense stat substitution
        true,           // Use target's stats
        generation,
    );
    
    // Handle accuracy
    let accuracy = move_data.accuracy as f32;
    if accuracy < 100.0 {
        vec![
            BattleInstructions::new(100.0 - accuracy, vec![]), // Miss
            BattleInstructions::new(accuracy, instructions),    // Hit
        ]
    } else {
        vec![BattleInstructions::new(100.0, instructions)]
    }
}


