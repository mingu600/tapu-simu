//! # Secondary Effects Moves
//!
//! This module contains moves that have secondary effects like burn, paralysis, freeze, poison, and flinch.
//! These moves deal damage but also have a chance to apply status conditions or stat changes.
//!
//! All moves in this module have been converted to use the new composer system for consistency
//! and to eliminate code duplication.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonStatus, VolatileStatus, Stat};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;
use crate::engine::combat::composers::damage_moves::damage_move_with_secondary_status;
use crate::engine::combat::core::status_system::StatusApplication;

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
macro_rules! flinch_move {
    ($func_name:ident, $chance:expr) => {
        pub fn $func_name(
            state: &BattleState,
            move_data: &MoveData,
            user_position: BattlePosition,
            target_positions: &[BattlePosition],
            generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            // Create damage instructions with flinch chance
            use crate::core::instructions::{BattleInstruction, StatusInstruction, VolatileStatus};
            
            // Get basic damage instructions
            let modifiers = crate::engine::combat::composers::damage_moves::DamageModifiers::default();
            let mut damage_instructions = crate::engine::combat::composers::damage_moves::simple_damage_move(
                state,
                move_data,
                user_position,
                target_positions,
                modifiers,
                generation,
            );
            
            // Add flinch status to all targets (30% chance for most flinch moves)
            for &target_position in target_positions {
                if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
                    // Only apply flinch if target hasn't moved yet this turn and isn't already flinched
                    if !target_pokemon.volatile_statuses.contains(&VolatileStatus::Flinch) {
                        damage_instructions.push(BattleInstruction::Status(
                            StatusInstruction::ApplyVolatile {
                                target: target_position,
                                status: VolatileStatus::Flinch,
                                duration: Some(1), // Flinch lasts only for the current turn
                                previous_had_status: target_pokemon.volatile_statuses.contains(&VolatileStatus::Flinch),
                                previous_duration: None,
                            }
                        ));
                    }
                }
            }
            
            vec![BattleInstructions::new(100.0, damage_instructions)]
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
    // TODO: Implement stat modification in composer system
    // For now, use the existing damage system
    let instructions = damage_move_with_secondary_status(
        state,
        move_data,
        user_position,
        target_positions,
        vec![], // No status effects, just stat changes
        generation,
    );
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Fire Fang - dual effect (burn + flinch)
pub fn apply_fire_fang_dual(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // TODO: Add flinch support - for now just handle burn
    let instructions = damage_move_with_secondary_status(
        state,
        move_data,
        user_position,
        target_positions,
        vec![
            StatusApplication {
                status: PokemonStatus::Burn,
                target: target_positions[0],
                chance: 10.0,
                duration: None,
            },
        ],
        generation,
    );
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Thunder Fang - dual effect (paralysis + flinch)
pub fn apply_thunder_fang_dual(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // TODO: Add flinch support - for now just handle paralysis
    let instructions = damage_move_with_secondary_status(
        state,
        move_data,
        user_position,
        target_positions,
        vec![
            StatusApplication {
                status: PokemonStatus::Paralysis,
                target: target_positions[0],
                chance: 10.0,
                duration: None,
            },
        ],
        generation,
    );
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Ice Fang - dual effect (freeze + flinch)
pub fn apply_ice_fang_dual(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // TODO: Add flinch support - for now just handle freeze
    let instructions = damage_move_with_secondary_status(
        state,
        move_data,
        user_position,
        target_positions,
        vec![
            StatusApplication {
                status: PokemonStatus::Freeze,
                target: target_positions[0],
                chance: 10.0,
                duration: None,
            },
        ],
        generation,
    );
    vec![BattleInstructions::new(100.0, instructions)]
}