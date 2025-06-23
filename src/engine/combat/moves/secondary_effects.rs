//! # Secondary Effects Moves

//! 
//! This module contains moves that have secondary effects like burn, paralysis, freeze, poison, and flinch.
//! These moves deal damage but also have a chance to apply status conditions or stat changes.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{PokemonStatus, VolatileStatus, Stat};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction, StatsInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::moves::{is_immune_to_paralysis, is_immune_to_poison, apply_generic_effects};
use std::collections::HashMap;
use crate::data::showdown_types::MoveData;

// =============================================================================
// SECONDARY EFFECTS DETERMINATION
// =============================================================================

/// Determine what secondary effect a move should have based on its properties
/// This function maps move types and names to their appropriate secondary effects
pub fn determine_secondary_effect_from_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Option<Vec<BattleInstruction>> {
    let move_name = move_data.name.to_lowercase();
    let move_type = move_data.move_type.to_lowercase();
    
    // Move-specific secondary effects
    match move_name.as_str() {
        // Fire moves that can burn
        "flamethrower" | "fireblast" | "fire blast" | "lavaplume" | "lava plume" |
        "firefang" | "fire fang" | "firepunch" | "fire punch" | "flamewheel" | "flame wheel" => {
            return Some(create_burn_instructions(state, target_positions));
        }
        
        // Electric moves that can paralyze
        "thunderbolt" | "thunder" | "discharge" | "sparklingaria" | "sparkling aria" |
        "thunderpunch" | "thunder punch" | "thunderfang" | "thunder fang" => {
            return Some(create_paralysis_instructions(state, target_positions, generation));
        }
        
        // Ice moves that can freeze
        "icebeam" | "ice beam" | "blizzard" | "icepunch" | "ice punch" |
        "icefang" | "ice fang" | "freezedry" | "freeze-dry" => {
            return Some(create_freeze_instructions(state, target_positions));
        }
        
        // Poison moves that can poison
        "sludgebomb" | "sludge bomb" | "poisonjab" | "poison jab" | 
        "sludgewave" | "sludge wave" | "poisonfang" | "poison fang" => {
            return Some(create_poison_instructions(state, target_positions, generation));
        }
        
        // Flinch-inducing moves
        "airslash" | "air slash" | "ironhead" | "iron head" | "rockslide" | "rock slide" |
        "headbutt" | "bite" | "stomp" | "astonish" | "fakebite" | "fake bite" => {
            return Some(create_flinch_instructions(target_positions));
        }
        
        // Stat-lowering moves
        "acid" => {
            return Some(create_defense_lowering_instructions(target_positions));
        }
        
        _ => {}
    }
    
    // Type-based secondary effects (generic)
    match move_type.as_str() {
        "fire" => Some(create_burn_instructions(state, target_positions)),
        "electric" => Some(create_paralysis_instructions(state, target_positions, generation)),
        "ice" => Some(create_freeze_instructions(state, target_positions)),
        "poison" => Some(create_poison_instructions(state, target_positions, generation)),
        _ => None,
    }
}

// =============================================================================
// INDIVIDUAL SECONDARY EFFECT MOVES
// =============================================================================

// Fire-type moves with burn chance
pub fn apply_flamethrower(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_burn_instructions(s, t)
    })
}

pub fn apply_fire_blast(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_burn_instructions(s, t)
    })
}

pub fn apply_lava_plume(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_burn_instructions(s, t)
    })
}

pub fn apply_fire_fang(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_burn_instructions(s, t)
    })
}

pub fn apply_fire_punch(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_burn_instructions(s, t)
    })
}

pub fn apply_flame_wheel(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_burn_instructions(s, t)
    })
}

// Electric-type moves with paralysis chance
pub fn apply_thunderbolt(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_paralysis_instructions(s, t, generation)
    })
}

pub fn apply_discharge(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_paralysis_instructions(s, t, generation)
    })
}

pub fn apply_sparkling_aria(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_paralysis_instructions(s, t, generation)
    })
}

pub fn apply_thunder_punch(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_paralysis_instructions(s, t, generation)
    })
}

pub fn apply_thunder_fang(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_paralysis_instructions(s, t, generation)
    })
}

// Ice-type moves with freeze chance
pub fn apply_ice_beam(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_freeze_instructions(s, t)
    })
}

pub fn apply_ice_punch(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_freeze_instructions(s, t)
    })
}

pub fn apply_ice_fang(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_freeze_instructions(s, t)
    })
}

// Poison-type moves with poison chance
pub fn apply_sludge_bomb(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_poison_instructions(s, t, generation)
    })
}

pub fn apply_poison_jab(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_poison_instructions(s, t, generation)
    })
}

pub fn apply_sludge_wave(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_poison_instructions(s, t, generation)
    })
}

pub fn apply_poison_fang(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |s, t| {
        create_poison_instructions(s, t, generation)
    })
}

// Flinch-inducing moves
pub fn apply_air_slash(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |_s, t| {
        create_flinch_instructions(t)
    })
}

pub fn apply_iron_head(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |_s, t| {
        create_flinch_instructions(t)
    })
}

pub fn apply_rock_slide(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |_s, t| {
        create_flinch_instructions(t)
    })
}

pub fn apply_headbutt(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |_s, t| {
        create_flinch_instructions(t)
    })
}

pub fn apply_bite(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |_s, t| {
        create_flinch_instructions(t)
    })
}

pub fn apply_stomp(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |_s, t| {
        create_flinch_instructions(t)
    })
}

pub fn apply_astonish(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    create_secondary_effect_move(state, move_data, user_position, target_positions, generation, |_s, t| {
        create_flinch_instructions(t)
    })
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Generic function to create moves with secondary effects
fn create_secondary_effect_move<F>(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    effect_fn: F,
) -> Vec<BattleInstructions>
where
    F: Fn(&BattleState, &[BattlePosition]) -> Vec<BattleInstruction>,
{
    // Apply primary damage effect
    let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, true);
    
    // Add secondary effect (typically with 10-30% chance, handled by PS data)
    let secondary_effects = effect_fn(state, target_positions);
    if !secondary_effects.is_empty() {
        instructions.push(BattleInstructions::new(30.0, secondary_effects)); // 30% is common chance
    }
    
    instructions
}

/// Create burn status instructions for targets
fn create_burn_instructions(state: &BattleState, target_positions: &[BattlePosition]) -> Vec<BattleInstruction> {
    target_positions
        .iter()
        .map(|&position| {
            let target = state.get_pokemon_at_position(position);
            BattleInstruction::Status(StatusInstruction::Apply {
                target: position,
                status: PokemonStatus::Burn,
                duration: None,
                previous_status: target.map(|p| p.status),
                previous_duration: target.and_then(|p| p.status_duration),
            })
        })
        .collect()
}

/// Create paralysis status instructions for targets
fn create_paralysis_instructions(
    state: &BattleState,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    target_positions
        .iter()
        .filter_map(|&position| {
            if let Some(target) = state.get_pokemon_at_position(position) {
                if target.status == PokemonStatus::None && !is_immune_to_paralysis(target, generation) {
                    Some(BattleInstruction::Status(StatusInstruction::Apply {
                        target: position,
                        status: PokemonStatus::Paralysis,
                        duration: None,
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Create freeze status instructions for targets
fn create_freeze_instructions(state: &BattleState, target_positions: &[BattlePosition]) -> Vec<BattleInstruction> {
    target_positions
        .iter()
        .map(|&position| {
            let target = state.get_pokemon_at_position(position);
            BattleInstruction::Status(StatusInstruction::Apply {
                target: position,
                status: PokemonStatus::Freeze,
                duration: None,
                previous_status: target.map(|p| p.status),
                previous_duration: target.and_then(|p| p.status_duration),
            })
        })
        .collect()
}

/// Create poison status instructions for targets
fn create_poison_instructions(
    state: &BattleState,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstruction> {
    target_positions
        .iter()
        .filter_map(|&position| {
            if let Some(target) = state.get_pokemon_at_position(position) {
                if target.status == PokemonStatus::None && !is_immune_to_poison(target, generation) {
                    Some(BattleInstruction::Status(StatusInstruction::Apply {
                        target: position,
                        status: PokemonStatus::Poison,
                        duration: None,
                        previous_status: Some(target.status),
                        previous_duration: target.status_duration,
                    }))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Create flinch instructions for targets
fn create_flinch_instructions(target_positions: &[BattlePosition]) -> Vec<BattleInstruction> {
    target_positions
        .iter()
        .map(|&position| {
            BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                target: position,
                status: VolatileStatus::Flinch,
                duration: Some(1), // Flinch only lasts for the current turn
                previous_had_status: false,
                previous_duration: None,
            })
        })
        .collect()
}

/// Create defense lowering instructions for targets
fn create_defense_lowering_instructions(target_positions: &[BattlePosition]) -> Vec<BattleInstruction> {
    target_positions
        .iter()
        .map(|&position| {
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Defense, -1);
            
            BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: position,
                stat_changes: stat_boosts,
                previous_boosts: HashMap::new(),
            })
        })
        .collect()
}