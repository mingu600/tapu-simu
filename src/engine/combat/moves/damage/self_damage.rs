//! # Self-Damage Move Effects

//! 
//! This module contains moves that damage the user without fainting them.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;

// =============================================================================
// SELF-DAMAGE MOVES
// =============================================================================

/// Apply Mind Blown - user takes damage equal to half their max HP
pub fn apply_mind_blown(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check for Damp ability which prevents explosive moves
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if crate::utils::normalize_name(&target.ability) == "damp" {
                // Damp prevents the move from working
                return vec![BattleInstructions::new(100.0, vec![])];
            }
        }
    }
    
    // Also check if user's side has any Pokemon with Damp
    let user_side_ref = user_position.side;
    let user_side = match user_side_ref {
        crate::core::battle_format::SideReference::SideOne => &state.sides[0],
        crate::core::battle_format::SideReference::SideTwo => &state.sides[1],
    };
    
    for pokemon in &user_side.pokemon {
        if crate::utils::normalize_name(&pokemon.ability) == "damp" {
            // Damp on user's side also prevents the move
            return vec![BattleInstructions::new(100.0, vec![])];
        }
    }
    
    // Check if opponent's side has any Pokemon with Damp
    let opponent_side = match user_side_ref {
        crate::core::battle_format::SideReference::SideOne => &state.sides[1],
        crate::core::battle_format::SideReference::SideTwo => &state.sides[0],
    };
    
    for pokemon in &opponent_side.pokemon {
        if crate::utils::normalize_name(&pokemon.ability) == "damp" {
            // Damp on opponent's side also prevents the move
            return vec![BattleInstructions::new(100.0, vec![])];
        }
    }
    
    let user_pokemon = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    // Create combined instructions with both target damage and self-damage
    let target_damage_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage);
    
    let mut combined_instructions = Vec::new();
    
    for mut instruction_set in target_damage_instructions {
        // Calculate self-damage
        let self_damage = (user_pokemon.max_hp / 2).min(user_pokemon.hp); // Don't exceed current HP
        
        // Add self-damage to each instruction set
        instruction_set.instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: user_position,
            amount: self_damage,
            previous_hp: Some(user_pokemon.hp),
        }));
        
        // Update affected positions to include user
        if !instruction_set.affected_positions.contains(&user_position) {
            instruction_set.affected_positions.push(user_position);
        }
        
        combined_instructions.push(instruction_set);
    }
    
    // If no target damage instructions were generated, create one with just self-damage
    if combined_instructions.is_empty() {
        let self_damage = (user_pokemon.max_hp / 2).min(user_pokemon.hp);
        combined_instructions.push(BattleInstructions::new(100.0, vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: user_position,
                amount: self_damage,
                previous_hp: Some(user_pokemon.hp),
            }),
        ]));
    }
    
    combined_instructions
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Apply generic move effects
fn apply_generic_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Use the proper generic effects implementation from main module
    crate::engine::combat::moves::apply_generic_effects(
        state, move_data, user_position, target_positions, generation, branch_on_damage
    )
}