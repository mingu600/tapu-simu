//! # Special Combat Mechanics

//! 
//! This module contains moves with unique combat mechanics that don't fit into other categories.

use crate::core::battle_state::{BattleState, MoveCategory};
use crate::core::instructions::VolatileStatus;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::move_effects::apply_generic_effects;
use crate::data::showdown_types::MoveData;

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
) -> Vec<BattleInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Compare Attack vs Special Attack stats to determine category
        let attack_stat = user_pokemon.stats.attack;
        let special_attack_stat = user_pokemon.stats.special_attack;
        
        let modified_move_data = MoveData {
            category: if attack_stat > special_attack_stat {
                "Physical".to_string()
            } else {
                "Special".to_string()
            },
            ..move_data.clone()
        };
        
        apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation, true)
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation, true)
    }
}

/// Apply Sky Drop - Two-turn move that lifts target into the sky
pub fn apply_sky_drop(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Check if user is already in the Sky Drop charging state
        if user_pokemon.volatile_statuses.contains(&VolatileStatus::SkyDrop) {
            // Second turn - attack and remove both Pokemon from sky
            let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, true);
            
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