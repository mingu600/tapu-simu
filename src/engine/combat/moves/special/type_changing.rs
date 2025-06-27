//! # Type-Changing Move Effects

//! 
//! This module contains moves that change their type based on various conditions
//! like the user's type, held item, or other battle conditions.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstructions, BattleInstruction, PokemonInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;
use crate::types::PokemonType;
use crate::engine::combat::moves::apply_generic_effects;

// =============================================================================
// TYPE-CHANGING MOVES
// =============================================================================

/// Apply Judgment - type matches user's primary type (Arceus)
pub fn apply_judgment(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Judgment's type matches the user's primary type (or Tera type in Gen 9+)
        let judgment_type = if !user_pokemon.types.is_empty() {
            user_pokemon.types[0]
        } else {
            PokemonType::Normal // Fallback to Normal type
        };
        
        // Change the move's type to match the user's type
        let mut modified_move_data = move_data.clone();
        modified_move_data.move_type = judgment_type;
        
        // Apply damage with the modified type
        instructions.extend(apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation, branch_on_damage));
    }
    
    instructions
}

/// Apply Multi-Attack - type matches user's primary type (Silvally)
pub fn apply_multi_attack(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Multi-Attack's type matches the user's primary type
        let attack_type = if !user_pokemon.types.is_empty() {
            user_pokemon.types[0]
        } else {
            PokemonType::Normal // Fallback to Normal type
        };
        
        // Change the move's type to match the user's type
        let mut modified_move_data = move_data.clone();
        modified_move_data.move_type = attack_type;
        
        // Apply damage with the modified type
        instructions.extend(apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation, branch_on_damage));
    }
    
    instructions
}

/// Apply Revelation Dance - type matches user's primary type (Oricorio)
pub fn apply_revelation_dance(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Revelation Dance's type matches the user's primary type
        let dance_type = if !user_pokemon.types.is_empty() {
            user_pokemon.types[0]
        } else {
            PokemonType::Normal // Fallback to Normal type
        };
        
        // Change the move's type to match the user's type
        let mut modified_move_data = move_data.clone();
        modified_move_data.move_type = dance_type;
        
        // Apply damage with the modified type
        instructions.extend(apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation, branch_on_damage));
    }
    
    instructions
}

/// Apply Ivy Cudgel - type depends on Ogerpon's mask/form
pub fn apply_ivy_cudgel(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Ivy Cudgel's type depends on Ogerpon's form/mask
        let cudgel_type = determine_ivy_cudgel_type(&user_pokemon.species, user_pokemon.item.as_ref());
        
        // Change the move's type based on the form/mask
        let mut modified_move_data = move_data.clone();
        modified_move_data.move_type = cudgel_type;
        
        // Apply damage with the modified type
        instructions.extend(apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation, branch_on_damage));
    }
    
    instructions
}

/// Apply Tera Blast - type matches user's Tera type (Gen 9)
pub fn apply_tera_blast(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // In Gen 9+, Tera Blast's type matches the user's Tera type
        let blast_type = if generation.generation.number() >= 9 {
            // Check if Pokemon is Terastallized and has a Tera type
            if user_pokemon.is_terastallized {
                if let Some(tera_type) = user_pokemon.tera_type {
                    tera_type
                } else {
                    // Terastallized but no Tera type set, use primary type
                    if !user_pokemon.types.is_empty() {
                        user_pokemon.types[0]
                    } else {
                        PokemonType::Normal
                    }
                }
            } else {
                // Not Terastallized, use primary type
                if !user_pokemon.types.is_empty() {
                    user_pokemon.types[0]
                } else {
                    PokemonType::Normal
                }
            }
        } else {
            PokemonType::Normal // Always Normal in pre-Gen 9
        };
        
        // Change the move's type to match the Tera type
        let mut modified_move_data = move_data.clone();
        modified_move_data.move_type = blast_type;
        
        // Apply damage with the modified type
        instructions.extend(apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation, branch_on_damage));
    }
    
    instructions
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Determine Ivy Cudgel's type based on Ogerpon's form and mask
fn determine_ivy_cudgel_type(species: &crate::types::PokemonName, item: Option<&crate::types::Items>) -> PokemonType {
    // Check for Ogerpon forms and their corresponding types
    match species {
        crate::types::PokemonName::OGERPON => {
            match item {
                Some(item_id) => {
                    match item_id {
                        crate::types::Items::WELLSPRINGMASK => PokemonType::Water,
                        crate::types::Items::HEARTHFLAMEMASK => PokemonType::Fire,
                        crate::types::Items::CORNERSTONEMASK => PokemonType::Rock,
                        _ => PokemonType::Grass, // Base form or no mask
                    }
                }
                None => PokemonType::Grass, // No item
            }
        }
        _ => PokemonType::Grass, // Default to Grass type
    }
}

