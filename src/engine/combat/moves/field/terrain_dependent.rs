//! # Terrain-Dependent Move Effects

//! 
//! This module contains moves that have effects dependent on the active terrain.

use crate::core::battle_state::BattleState;
use crate::core::instructions::Terrain;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, FieldInstruction, PokemonInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;
use crate::engine::combat::moves::apply_generic_effects;

// =============================================================================
// TERRAIN-DEPENDENT MOVES
// =============================================================================

/// Apply Grassy Glide - priority move in Grassy Terrain
pub fn apply_grassy_glide(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check if we're in Grassy Terrain
    let has_priority = matches!(state.terrain(), Terrain::GrassyTerrain);
    
    if has_priority {
        // The move already has +1 priority in Grassy Terrain, which should be handled
        // in the move priority calculation, not here. This function handles any
        // additional effects beyond the priority boost.
        
        // Grassy Glide is just a physical Grass-type move with conditional priority
        // No special effects beyond damage, so we use generic effects
        apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
    } else {
        // Without Grassy Terrain, it's just a normal priority move
        apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
    }
}

/// Apply Expanding Force - boosted power and area effect in Psychic Terrain
pub fn apply_expanding_force(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check if Psychic Terrain is active
    if state.terrain() == Terrain::PsychicTerrain {
        // 1.5x power in Psychic Terrain
        let power_multiplier = 1.5;
        apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier, branch_on_damage)
    } else {
        // Normal power
        apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
    }
}

/// Apply Rising Voltage - boosted power in Electric Terrain
pub fn apply_rising_voltage(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check if Electric Terrain is active
    if state.terrain() == Terrain::ElectricTerrain {
        // 1.5x power in Electric Terrain
        let power_multiplier = 1.5;
        apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier, branch_on_damage)
    } else {
        // Normal power
        apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
    }
}

/// Apply Misty Explosion - boosted power in Misty Terrain, user faints
pub fn apply_misty_explosion(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Check if Misty Terrain is active for power boost
    let power_multiplier = if state.terrain() == Terrain::MistyTerrain {
        1.5
    } else {
        1.0
    };
    
    // Apply power modifier if terrain is active
    if power_multiplier > 1.0 {
        instructions.extend(apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier, branch_on_damage));
    } else {
        instructions.extend(apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage));
    }
    
    // Get the user's current HP before fainting
    let user_current_hp = state.get_pokemon_at_position(user_position)
        .map(|pokemon| pokemon.hp)
        .unwrap_or(0);
    
    // User faints (self-destruct effect)
    instructions.push(BattleInstructions::new(100.0, vec![
        BattleInstruction::Pokemon(PokemonInstruction::Faint {
            target: user_position,
            previous_hp: user_current_hp,
            previous_status: None,
        }),
    ]));
    
    instructions
}

/// Apply Psy Blade - boosted power in Electric Terrain
pub fn apply_psy_blade(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check if Electric Terrain is active
    if state.terrain() == Terrain::ElectricTerrain {
        // 1.5x power in Electric Terrain
        let power_multiplier = 1.5;
        apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier, branch_on_damage)
    } else {
        // Normal power
        apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage)
    }
}

/// Apply Steel Roller - fails without terrain, removes terrain after hitting
pub fn apply_steel_roller(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check if any terrain is active
    if state.terrain() == Terrain::None {
        // Move fails when no terrain is active
        vec![BattleInstructions::new(100.0, vec![])]
    } else {
        // Normal move behavior when terrain is active
        let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage);
        
        // Remove terrain after hitting
        instructions.push(BattleInstructions::new(100.0, vec![
            BattleInstruction::Field(FieldInstruction::Terrain {
                new_terrain: Terrain::None,
                previous_terrain: state.terrain(),
                turns: None,
                previous_turns: state.field.terrain.turns_remaining,
                source: None,
            }),
        ]));
        
        instructions
    }
}

/// Apply Ice Spinner - removes terrain after hitting
pub fn apply_ice_spinner(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    let mut instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage);
    
    // Remove terrain after hitting (if any terrain is active)
    if state.terrain() != Terrain::None {
        instructions.push(BattleInstructions::new(100.0, vec![
            BattleInstruction::Field(FieldInstruction::Terrain {
                new_terrain: Terrain::None,
                previous_terrain: state.terrain(),
                turns: None,
                previous_turns: state.field.terrain.turns_remaining,
                source: None,
            }),
        ]));
    }
    
    instructions
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Apply power modifier to a move
fn apply_power_modifier_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    power_multiplier: f32,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Create modified move data with adjusted power
    let mut modified_move = move_data.clone();
    if modified_move.base_power > 0 {
        modified_move.base_power = ((modified_move.base_power as f32 * power_multiplier) as u16);
    }
    
    // Apply generic effects with the modified move data
    apply_generic_effects(state, &modified_move, user_position, target_positions, generation, branch_on_damage)
}
