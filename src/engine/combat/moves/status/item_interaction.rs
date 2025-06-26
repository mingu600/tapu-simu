//! # Item Interaction Move Effects

//! 
//! This module contains moves that interact with held items in various ways.

use crate::core::battle_state::{BattleState, Pokemon};
use crate::core::instructions::{PokemonStatus, VolatileStatus};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, StatusInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;

// =============================================================================
// ITEM INTERACTION MOVES
// =============================================================================

/// Apply Trick - swaps items between user and target
pub fn apply_trick(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if target_positions.is_empty() {
        return vec![BattleInstructions::new(100.0, vec![])];
    }
    
    let target_position = target_positions[0]; // Trick only targets one Pokemon
    
    // Get user and target Pokemon
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    let target = match state.get_pokemon_at_position(target_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    // Check if move should fail
    if should_item_swap_fail(user, target) {
        return vec![BattleInstructions::new(100.0, vec![])];
    }
    
    // Create item swap instructions
    let mut instructions = Vec::new();
    
    // Change user's item to target's item
    instructions.push(BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
        target: user_position,
        new_item: target.item.clone(),
        previous_item: user.item.clone(),
    }));
    
    // Change target's item to user's item
    instructions.push(BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
        target: target_position,
        new_item: user.item.clone(),
        previous_item: target.item.clone(),
    }));
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Switcheroo - identical to Trick but Dark-type
pub fn apply_switcheroo(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Switcheroo has identical mechanics to Trick
    apply_trick(state, user_position, target_positions, generation)
}

/// Apply Knock Off - removes target's item and boosts damage if item present
pub fn apply_knock_off(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let mut instruction_list = Vec::new();
            
            // Bonus damage if target has an item (Gen 6+)
            let power_multiplier = if generation.generation.number() >= 6 && target.item.is_some() {
                1.5
            } else {
                1.0
            };
            
            // Apply damage with potential bonus
            let damage_instructions = apply_power_modifier_move(state, move_data, user_position, &[target_position], generation, power_multiplier);
            for damage_instruction in damage_instructions {
                instruction_list.extend(damage_instruction.instruction_list);
            }
            
            // Remove target's item if it has one
            if target.item.is_some() {
                instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: target_position,
                    new_item: None,
                    previous_item: target.item.clone(),
                }));
            }
            
            instructions.push(BattleInstructions::new(100.0, instruction_list));
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Thief - steals target's item if user has none
pub fn apply_thief(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                let mut instruction_list = Vec::new();
                
                // Apply damage first
                let damage_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, false);
                for damage_instruction in damage_instructions {
                    instruction_list.extend(damage_instruction.instruction_list);
                }
                
                // Steal item if user has none and target has one
                if user.item.is_none() && target.item.is_some() {
                    let stolen_item = target.item.clone();
                    
                    // Give item to user
                    instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: user_position,
                        new_item: stolen_item,
                        previous_item: user.item.clone(),
                    }));
                    
                    // Remove item from target
                    instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                        target: target_position,
                        new_item: None,
                        previous_item: target.item.clone(),
                    }));
                }
                
                instructions.push(BattleInstructions::new(100.0, instruction_list));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Fling - power and effect based on held item
pub fn apply_fling(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    repository: &crate::data::GameDataRepository,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        if let Some(item) = &user.item {
            // Check if item can be flung
            let can_be_flung = repository.items.can_item_be_flung(item);
            
            if !can_be_flung {
                // Move fails if item can't be flung
                return vec![BattleInstructions::new(100.0, vec![])];
            }
            
            let mut instruction_list = Vec::new();
            
            // Get item-specific power
            let fling_power = get_fling_power(item, repository);
            
            // Create modified move data with item-specific power
            let mut modified_move = move_data.clone();
            modified_move.base_power = (fling_power as u16);
            
            // Apply damage with item-specific power
            let damage_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, false);
            for damage_instruction in damage_instructions {
                instruction_list.extend(damage_instruction.instruction_list);
            }
            
            // Apply item-specific status effects
            for target_position in target_positions {
                if let Some(target) = state.get_pokemon_at_position(*target_position) {
                    // Apply main status effect if item has one
                    if let Some(status) = get_fling_status(item) {
                        let status_effect = match status {
                            "brn" => PokemonStatus::Burn,
                            "par" => PokemonStatus::Paralysis,
                            "psn" => PokemonStatus::Poison,
                            "tox" => PokemonStatus::BadlyPoisoned,
                            "slp" => PokemonStatus::Sleep,
                            "frz" => PokemonStatus::Freeze,
                            _ => continue, // Unknown status
                        };
                        
                        // Don't apply if target already has a status condition
                        if target.status == PokemonStatus::None {
                            instruction_list.push(BattleInstruction::Status(StatusInstruction::Apply {
                                target: *target_position,
                                status: status_effect,
                                duration: None,
                                previous_status: Some(target.status),
                                previous_duration: target.status_duration,
                            }));
                        }
                    }
                    
                    // Apply volatile status effect if item has one
                    if let Some(volatile_status) = get_fling_volatile_status(item) {
                        let status_effect = match volatile_status {
                            "flinch" => VolatileStatus::Flinch,
                            _ => continue, // Unknown volatile status
                        };
                        
                        instruction_list.push(BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                            target: *target_position,
                            status: status_effect,
                            duration: Some(1),
                            previous_had_status: false,
                            previous_duration: None,
                        }));
                    }
                }
            }
            
            // User loses their item after flinging it
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                target: user_position,
                new_item: None,
                previous_item: user.item.clone(),
            }));
            
            vec![BattleInstructions::new(100.0, instruction_list)]
        } else {
            // User has no item, move fails
            vec![BattleInstructions::new(100.0, vec![])]
        }
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Check if item swap should fail
fn should_item_swap_fail(user: &Pokemon, target: &Pokemon) -> bool {
    // Fail if both Pokemon have the same item (including both having no item)
    if user.item == target.item {
        return true;
    }
    
    // Fail if target has Sticky Hold ability
    if crate::utils::normalize_name(&target.ability) == "stickyhold" {
        return true;
    }
    
    // Fail if target has a permanent item
    if target.item.as_ref().map_or(false, |item| is_permanent_item(item, &target.species)) {
        return true;
    }
    
    // Fail if target is behind a Substitute
    if target.volatile_statuses.contains(&VolatileStatus::Substitute) {
        return true;
    }
    
    false
}

/// Check if an item is permanent and cannot be removed
fn is_permanent_item(item: &str, pokemon_species: &str) -> bool {
    let normalized_item = crate::utils::normalize_name(item);
    let normalized_species = crate::utils::normalize_name(pokemon_species);
    
    match normalized_item.as_str() {
        // Arceus plates
        "dracoplate" | "dreadplate" | "earthplate" | "fistplate" | 
        "flameplate" | "icicleplate" | "insectplate" | "ironplate" |
        "meadowplate" | "mindplate" | "pixieplate" | "skyplate" |
        "splashplate" | "spookyplate" | "stoneplate" | "toxicplate" |
        "zapplate" => normalized_species.starts_with("arceus"),
        
        // Origin forme items
        "lustrousglobe" => normalized_species.contains("palkia"),
        "griseouscore" => normalized_species.contains("giratina"),
        "adamantcrystal" => normalized_species.contains("dialga"),
        
        // Rusted weapons
        "rustedsword" => normalized_species.contains("zacian"),
        "rustedshield" => normalized_species.contains("zamazenta"),
        
        _ => false,
    }
}

/// Get fling power for an item
fn get_fling_power(item: &str, repository: &crate::data::GameDataRepository) -> u8 {
    repository.items.get_item_fling_power(item)
        .unwrap_or_else(|| {
            // If item not found in repository, use default power for unknown items
            // TODO: Replace with proper logging when logging system is available
            10 // Default fling power for unknown items
        })
}

/// Get fling status effect for an item
fn get_fling_status(item: &str) -> Option<&'static str> {
    let normalized_item = crate::utils::normalize_name(item);
    match normalized_item.as_str() {
        "flameorb" => Some("brn"),
        "toxicorb" => Some("tox"),
        _ => None,
    }
}

/// Get fling volatile status effect for an item
fn get_fling_volatile_status(item: &str) -> Option<&'static str> {
    let normalized_item = crate::utils::normalize_name(item);
    match normalized_item.as_str() {
        "kingsrock" | "razorfang" => Some("flinch"),
        _ => None,
    }
}

/// Apply power modifier to a move
fn apply_power_modifier_move(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    power_multiplier: f32,
) -> Vec<BattleInstructions> {
    // Create modified move data with adjusted power
    let mut modified_move = move_data.clone();
    if modified_move.base_power > 0 {
        modified_move.base_power = ((modified_move.base_power as f32 * power_multiplier) as u16);
    }
    
    // Apply generic effects with the modified move data
    apply_generic_effects(state, move_data, user_position, target_positions, generation, false)
}

/// Apply generic move effects
fn apply_generic_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Use the proper generic effects implementation from simple module
    crate::engine::combat::moves::simple::apply_generic_effects(
        state, move_data, user_position, target_positions, generation, branch_on_damage
    )
}