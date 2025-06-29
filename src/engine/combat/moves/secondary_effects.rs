//! # Secondary Effects Moves
//!
//! This module contains moves that have secondary effects like burn, paralysis, freeze, poison, and flinch.
//! These moves deal damage but also have a chance to apply status conditions or stat changes.
//!
//! All moves in this module have been converted to use the new composer system for consistency
//! and to eliminate code duplication.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonStatus, Stat, VolatileStatus};
use crate::data::showdown_types::MoveData;
use crate::engine::combat::composers::damage_moves::{damage_move_with_secondary_status, damage_move_with_secondary_volatile_status};
use crate::engine::combat::core::status_system::{StatusApplication, VolatileStatusApplication};
use crate::generation::GenerationMechanics;

// =============================================================================
// SECONDARY EFFECT MOVE MACRO
// =============================================================================

/// Macro to reduce repetitive code for simple secondary effect moves
macro_rules! secondary_effect_move {
    ($func_name:ident, $status:expr, $chance:expr) => {
        pub fn $func_name(
            state: &BattleState,
            move_data: &MoveData,
            user_position: BattlePosition,
            target_positions: &[BattlePosition],
            generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            let instructions = damage_move_with_secondary_status(
                state,
                move_data,
                user_position,
                target_positions,
                vec![StatusApplication {
                    status: $status,
                    target: target_positions[0],
                    chance: $chance,
                    duration: None,
                }],
                generation,
            );
            vec![BattleInstructions::new(100.0, instructions)]
        }
    };
}

/// Macro for moves with flinch chance - these need special handling
/// as flinch is a volatile status, not a main status
/// This macro creates branching for secondary effects only - accuracy is handled by the turn system
macro_rules! flinch_move {
    ($func_name:ident, $chance:expr) => {
        pub fn $func_name(
            state: &BattleState,
            move_data: &MoveData,
            user_position: BattlePosition,
            target_positions: &[BattlePosition],
            generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            use crate::core::instructions::{BattleInstruction, StatusInstruction, VolatileStatus};
            
            
            let mut instruction_sets = Vec::new();
            
            // Get basic damage instructions
            let modifiers = crate::engine::combat::composers::damage_moves::DamageModifiers::default();
            let damage_instructions = crate::engine::combat::composers::damage_moves::simple_damage_move(
                state,
                move_data,
                user_position,
                target_positions,
                modifiers,
                generation,
            );
            
            // Calculate affected positions from damage instructions
            let mut no_flinch_affected_positions = Vec::new();
            let mut flinch_affected_positions = Vec::new();
            
            for instruction in &damage_instructions {
                // Get affected positions from each instruction
                if let BattleInstruction::Pokemon(ref pokemon_instr) = instruction {
                    let positions = pokemon_instr.affected_positions();
                    no_flinch_affected_positions.extend(positions.clone());
                    flinch_affected_positions.extend(positions);
                }
            }
            
            // Remove duplicates
            no_flinch_affected_positions.sort();
            no_flinch_affected_positions.dedup();
            flinch_affected_positions.sort();
            flinch_affected_positions.dedup();
            
            // Hit without flinch (100 - flinch_chance)%
            let no_flinch_percentage = 100.0 - $chance;
            if no_flinch_percentage > 0.0 {
                instruction_sets.push(BattleInstructions::new_with_positions(
                    no_flinch_percentage,
                    damage_instructions.clone(),
                    no_flinch_affected_positions,
                ));
            }
            
            // Hit with flinch (flinch_chance)%
            if $chance > 0.0 {
                let mut flinch_instructions = damage_instructions.clone();
                let mut any_flinch_applied = false;
                
                // Add flinch status to all targets (with speed check)
                for &target_position in target_positions {
                    if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
                        // Only apply flinch if target hasn't moved yet this turn and isn't already flinched
                        if !target_pokemon.volatile_statuses.contains(VolatileStatus::Flinch) {
                            // Check if user is faster than target (speed-aware flinch)
                            let can_flinch = is_user_faster_than_target(state, user_position, target_position);
                            if can_flinch {
                                flinch_instructions.push(BattleInstruction::Status(
                                    StatusInstruction::ApplyVolatile {
                                        target: target_position,
                                        status: VolatileStatus::Flinch,
                                        duration: Some(1), // Flinch lasts only for the current turn
                                        previous_had_status: false,
                                        previous_duration: None,
                                    }
                                ));
                                // Target is affected by flinch status too
                                if !flinch_affected_positions.contains(&target_position) {
                                    flinch_affected_positions.push(target_position);
                                }
                                any_flinch_applied = true;
                            }
                        }
                    }
                }
                
                // Only create a separate flinch branch if at least one flinch was actually applied
                if any_flinch_applied {
                    instruction_sets.push(BattleInstructions::new_with_positions(
                        $chance,
                        flinch_instructions,
                        flinch_affected_positions,
                    ));
                } else {
                    // No flinch was applied due to speed checks, so combine the percentages
                    // Update the no-flinch branch to include the flinch chance
                    if let Some(ref mut no_flinch_branch) = instruction_sets.last_mut() {
                        no_flinch_branch.percentage += $chance;
                    }
                }
            }
            
            instruction_sets
        }
    };
}

// =============================================================================
// FIRE MOVES - BURN CHANCE
// =============================================================================

secondary_effect_move!(apply_flamethrower, PokemonStatus::Burn, 10.0);
secondary_effect_move!(apply_fire_blast, PokemonStatus::Burn, 10.0);
secondary_effect_move!(apply_lava_plume, PokemonStatus::Burn, 30.0);
secondary_effect_move!(apply_fire_fang, PokemonStatus::Burn, 10.0);
secondary_effect_move!(apply_fire_punch, PokemonStatus::Burn, 10.0);
secondary_effect_move!(apply_flame_wheel, PokemonStatus::Burn, 10.0);

// =============================================================================
// ELECTRIC MOVES - PARALYSIS CHANCE
// =============================================================================

secondary_effect_move!(apply_thunderbolt, PokemonStatus::Paralysis, 10.0);
secondary_effect_move!(apply_thunder, PokemonStatus::Paralysis, 30.0);
secondary_effect_move!(apply_discharge, PokemonStatus::Paralysis, 30.0);
secondary_effect_move!(apply_sparkling_aria, PokemonStatus::Paralysis, 10.0);
secondary_effect_move!(apply_thunder_punch, PokemonStatus::Paralysis, 10.0);
secondary_effect_move!(apply_thunder_fang, PokemonStatus::Paralysis, 10.0);

// =============================================================================
// ICE MOVES - FREEZE CHANCE
// =============================================================================

secondary_effect_move!(apply_ice_beam, PokemonStatus::Freeze, 10.0);
secondary_effect_move!(apply_blizzard, PokemonStatus::Freeze, 10.0);
secondary_effect_move!(apply_ice_punch, PokemonStatus::Freeze, 10.0);
secondary_effect_move!(apply_ice_fang, PokemonStatus::Freeze, 10.0);
secondary_effect_move!(apply_freeze_dry, PokemonStatus::Freeze, 10.0);

// =============================================================================
// POISON MOVES - POISON CHANCE
// =============================================================================

secondary_effect_move!(apply_sludge_bomb, PokemonStatus::Poison, 30.0);
secondary_effect_move!(apply_poison_jab, PokemonStatus::Poison, 30.0);
secondary_effect_move!(apply_sludge_wave, PokemonStatus::Poison, 10.0);
secondary_effect_move!(apply_poison_fang, PokemonStatus::BadlyPoisoned, 50.0);

// =============================================================================
// FLINCH MOVES
// =============================================================================

flinch_move!(apply_air_slash, 30.0);
flinch_move!(apply_iron_head, 30.0);
flinch_move!(apply_rock_slide, 30.0);
flinch_move!(apply_headbutt, 30.0);
flinch_move!(apply_bite, 30.0);
flinch_move!(apply_stomp, 30.0);
flinch_move!(apply_astonish, 30.0);
flinch_move!(apply_fake_bite, 30.0);

// =============================================================================
// SPECIAL CASES
// =============================================================================

/// Acid (Damage variant) - deals damage with chance to lower Defense
pub fn apply_acid_damage(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::engine::combat::composers::damage_moves::{simple_damage_move, DamageModifiers};
    use std::collections::HashMap;
    
    // Create stat changes map for Defense reduction
    let mut stat_changes = HashMap::new();
    stat_changes.insert(Stat::Defense, -1);
    
    let modifiers = DamageModifiers {
        stat_changes: Some(stat_changes),
        ..Default::default()
    };
    
    // Acid has a 10% chance to lower Defense
    let no_stat_change_percentage = 90.0;
    let stat_change_percentage = 10.0;
    
    let mut instruction_sets = Vec::new();
    
    // Hit without stat change (90%)
    let base_instructions = simple_damage_move(
        state,
        move_data,
        user_position,
        target_positions,
        DamageModifiers::default(),
        generation,
    );
    instruction_sets.push(BattleInstructions::new(no_stat_change_percentage, base_instructions));
    
    // Hit with stat change (10%)
    let stat_change_instructions = simple_damage_move(
        state,
        move_data,
        user_position,
        target_positions,
        modifiers,
        generation,
    );
    instruction_sets.push(BattleInstructions::new(stat_change_percentage, stat_change_instructions));
    
    instruction_sets
}

/// Fire Fang - dual effect (burn + flinch)
pub fn apply_fire_fang_dual(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut all_instructions = Vec::new();
    
    for &target_position in target_positions {
        // Check if user is faster for flinch to work
        let can_flinch = is_user_faster_than_target(state, user_position, target_position);
        
        // Create branching for dual effects: burn (10%) + flinch (10%)
        // Both effects are independent, so we need 4 branches:
        // 1. Neither effect (81%)
        // 2. Burn only (9%) 
        // 3. Flinch only (9%)
        // 4. Both effects (1%)
        
        let base_instructions = damage_move_with_secondary_status(
            state,
            move_data,
            user_position,
            &[target_position],
            vec![],
            generation,
        );
        
        // Branch 1: Neither effect (81%)
        all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_NEITHER, base_instructions.clone()));
        
        // Branch 2: Burn only (9%)
        let burn_instructions = damage_move_with_secondary_status(
            state,
            move_data,
            user_position,
            &[target_position],
            vec![StatusApplication {
                status: PokemonStatus::Burn,
                target: target_position,
                chance: 100.0, // Already factored into branch probability
                duration: None,
            }],
            generation,
        );
        all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_FIRST_ONLY, burn_instructions));
        
        // Branch 3: Flinch only (9%) - only if user is faster
        if can_flinch {
            let flinch_instructions = damage_move_with_secondary_volatile_status(
                state,
                move_data,
                user_position,
                &[target_position],
                vec![VolatileStatusApplication {
                    status: VolatileStatus::Flinch,
                    target: target_position,
                    chance: 100.0, // Already factored into branch probability
                    duration: Some(1),
                }],
                generation,
            );
            all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_SECOND_ONLY, flinch_instructions));
            
            // Branch 4: Both effects (1%)
            let mut both_instructions = damage_move_with_secondary_status(
                state,
                move_data,
                user_position,
                &[target_position],
                vec![StatusApplication {
                    status: PokemonStatus::Burn,
                    target: target_position,
                    chance: 100.0,
                    duration: None,
                }],
                generation,
            );
            
            // Add flinch to the same instructions
            both_instructions.extend(
                damage_move_with_secondary_volatile_status(
                    state,
                    move_data,
                    user_position,
                    &[target_position],
                    vec![VolatileStatusApplication {
                        status: VolatileStatus::Flinch,
                        target: target_position,
                        chance: 100.0,
                        duration: Some(1),
                    }],
                    generation,
                ).into_iter().skip(1) // Skip damage instruction since we already have it
            );
            all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_BOTH, both_instructions));
        } else {
            // If can't flinch, redistribute the 10% that would have been flinch effects
            // to the "neither effect" branch, making it 90% instead of 81%
            all_instructions[0].percentage = 90.0;
        }
    }
    
    all_instructions
}

/// Thunder Fang - dual effect (paralysis + flinch)
pub fn apply_thunder_fang_dual(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut all_instructions = Vec::new();
    
    for &target_position in target_positions {
        // Check if user is faster for flinch to work
        let can_flinch = is_user_faster_than_target(state, user_position, target_position);
        
        // Create branching for dual effects: paralysis (10%) + flinch (10%)
        let base_instructions = damage_move_with_secondary_status(
            state,
            move_data,
            user_position,
            &[target_position],
            vec![],
            generation,
        );
        
        // Branch 1: Neither effect (81%)
        all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_NEITHER, base_instructions.clone()));
        
        // Branch 2: Paralysis only (9%)
        let paralysis_instructions = damage_move_with_secondary_status(
            state,
            move_data,
            user_position,
            &[target_position],
            vec![StatusApplication {
                status: PokemonStatus::Paralysis,
                target: target_position,
                chance: 100.0,
                duration: None,
            }],
            generation,
        );
        all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_FIRST_ONLY, paralysis_instructions));
        
        // Branch 3: Flinch only (9%) - only if user is faster
        if can_flinch {
            let flinch_instructions = damage_move_with_secondary_volatile_status(
                state,
                move_data,
                user_position,
                &[target_position],
                vec![VolatileStatusApplication {
                    status: VolatileStatus::Flinch,
                    target: target_position,
                    chance: 100.0,
                    duration: Some(1),
                }],
                generation,
            );
            all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_SECOND_ONLY, flinch_instructions));
            
            // Branch 4: Both effects (1%)
            let mut both_instructions = damage_move_with_secondary_status(
                state,
                move_data,
                user_position,
                &[target_position],
                vec![StatusApplication {
                    status: PokemonStatus::Paralysis,
                    target: target_position,
                    chance: 100.0,
                    duration: None,
                }],
                generation,
            );
            
            both_instructions.extend(
                damage_move_with_secondary_volatile_status(
                    state,
                    move_data,
                    user_position,
                    &[target_position],
                    vec![VolatileStatusApplication {
                        status: VolatileStatus::Flinch,
                        target: target_position,
                        chance: 100.0,
                        duration: Some(1),
                    }],
                    generation,
                ).into_iter().skip(1)
            );
            all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_BOTH, both_instructions));
        } else {
            all_instructions[0].percentage = 90.0;
        }
    }
    
    all_instructions
}

/// Ice Fang - dual effect (freeze + flinch)
pub fn apply_ice_fang_dual(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut all_instructions = Vec::new();
    
    for &target_position in target_positions {
        // Check if user is faster for flinch to work
        let can_flinch = is_user_faster_than_target(state, user_position, target_position);
        
        // Create branching for dual effects: freeze (10%) + flinch (10%)
        let base_instructions = damage_move_with_secondary_status(
            state,
            move_data,
            user_position,
            &[target_position],
            vec![],
            generation,
        );
        
        // Branch 1: Neither effect (81%)
        all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_NEITHER, base_instructions.clone()));
        
        // Branch 2: Freeze only (9%)
        let freeze_instructions = damage_move_with_secondary_status(
            state,
            move_data,
            user_position,
            &[target_position],
            vec![StatusApplication {
                status: PokemonStatus::Freeze,
                target: target_position,
                chance: 100.0,
                duration: None,
            }],
            generation,
        );
        all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_FIRST_ONLY, freeze_instructions));
        
        // Branch 3: Flinch only (9%) - only if user is faster
        if can_flinch {
            let flinch_instructions = damage_move_with_secondary_volatile_status(
                state,
                move_data,
                user_position,
                &[target_position],
                vec![VolatileStatusApplication {
                    status: VolatileStatus::Flinch,
                    target: target_position,
                    chance: 100.0,
                    duration: Some(1),
                }],
                generation,
            );
            all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_SECOND_ONLY, flinch_instructions));
            
            // Branch 4: Both effects (1%)
            let mut both_instructions = damage_move_with_secondary_status(
                state,
                move_data,
                user_position,
                &[target_position],
                vec![StatusApplication {
                    status: PokemonStatus::Freeze,
                    target: target_position,
                    chance: 100.0,
                    duration: None,
                }],
                generation,
            );
            
            both_instructions.extend(
                damage_move_with_secondary_volatile_status(
                    state,
                    move_data,
                    user_position,
                    &[target_position],
                    vec![VolatileStatusApplication {
                        status: VolatileStatus::Flinch,
                        target: target_position,
                        chance: 100.0,
                        duration: Some(1),
                    }],
                    generation,
                ).into_iter().skip(1)
            );
            all_instructions.push(BattleInstructions::new(crate::constants::moves::DUAL_EFFECT_BOTH, both_instructions));
        } else {
            all_instructions[0].percentage = 90.0;
        }
    }
    
    all_instructions
}

/// Check if the user is faster than the target for speed-aware flinch application
fn is_user_faster_than_target(
    state: &BattleState,
    user_position: BattlePosition,
    target_position: BattlePosition,
) -> bool {
    let user_pokemon = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => {
            return false;
        }
    };
    
    let target_pokemon = match state.get_pokemon_at_position(target_position) {
        Some(pokemon) => pokemon,
        None => {
            return false;
        }
    };
    
    let user_speed = user_pokemon.get_effective_speed(state, user_position);
    let target_speed = target_pokemon.get_effective_speed(state, target_position);
    
    
    user_speed > target_speed
}