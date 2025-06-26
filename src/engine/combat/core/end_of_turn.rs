//! End-of-Turn Processing Pipeline
//!
//! This module implements comprehensive end-of-turn processing following poke-engine order:
//! 1. Remove single-turn volatile statuses (Flinch, etc.)
//! 2. Weather effects (damage + ability triggers)
//! 3. Terrain effects
//! 4. Field effect timers (Trick Room, Light Screen, etc.)
//! 5. Status condition damage
//! 6. Ability end-of-turn triggers
//! 7. Item end-of-turn effects

use crate::core::battle_format::{BattlePosition, SideReference};
use crate::core::battle_state::BattleState;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, StatusInstruction, 
    PokemonStatus, VolatileStatus, Weather, Terrain
};
use std::collections::HashMap;

/// Generate comprehensive end-of-turn instructions following poke-engine order
pub fn generate_end_of_turn_instructions(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut all_instructions = Vec::new();
    
    // 1. Remove single-turn volatile statuses
    all_instructions.extend(remove_expiring_volatile_statuses(battle_state));
    
    // 2. Weather effects
    all_instructions.extend(apply_weather_effects(battle_state));
    
    // 3. Terrain effects  
    all_instructions.extend(apply_terrain_effects(battle_state));
    
    // 4. Field effect timers
    all_instructions.extend(decrement_field_timers(battle_state));
    
    // 5. Status condition damage
    all_instructions.extend(apply_status_damage(battle_state));
    
    // 6. Ability end-of-turn triggers
    all_instructions.extend(trigger_end_of_turn_abilities_wrapper(battle_state));
    
    // 7. Item end-of-turn effects
    all_instructions.extend(apply_item_effects(battle_state));
    
    // If no effects, return empty instruction set
    if all_instructions.is_empty() {
        vec![BattleInstructions::new(100.0, vec![])]
    } else {
        all_instructions
    }
}

/// Remove single-turn volatile statuses (Flinch, single-turn protection, etc.)
fn remove_expiring_volatile_statuses(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    // Single-turn statuses that automatically expire
    let single_turn_statuses = [
        VolatileStatus::Flinch,
        VolatileStatus::Protect,
        VolatileStatus::Endure,
        VolatileStatus::MagicCoat,
        VolatileStatus::FollowMe,
        VolatileStatus::HelpingHand,
    ];
    
    for position in battle_state.get_all_active_positions() {
        if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
            for &status in &single_turn_statuses {
                if pokemon.volatile_statuses.contains(&status) {
                    instructions.push(BattleInstructions::new(
                        100.0,
                        vec![BattleInstruction::Status(
                            StatusInstruction::RemoveVolatile {
                                target: position,
                                status,
                                previous_duration: pokemon.volatile_status_durations.get(&status).copied(),
                            }
                        )]
                    ));
                }
            }
        }
    }
    
    instructions
}

/// Apply weather effects in proper order
fn apply_weather_effects(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    match battle_state.weather() {
        Weather::Sandstorm => {
            instructions.extend(apply_sandstorm_damage(battle_state));
        }
        Weather::Hail => {
            instructions.extend(apply_hail_damage(battle_state));
        }
        Weather::Sun | Weather::Rain => {
            // These don't do direct damage but may trigger abilities
            instructions.extend(trigger_weather_abilities(battle_state));
        }
        _ => {}
    }
    
    instructions
}

/// Apply sandstorm damage to non-immune Pokemon
fn apply_sandstorm_damage(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    for position in battle_state.get_all_active_positions() {
        if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
            // Skip if immune to sandstorm
            if is_sandstorm_immune(pokemon) {
                continue;
            }
            
            let damage = (pokemon.max_hp / 16).max(1);
            instructions.push(BattleInstructions::new(
                100.0,
                vec![BattleInstruction::Pokemon(
                    PokemonInstruction::Damage {
                        target: position,
                        amount: damage,
                        previous_hp: Some(pokemon.hp),
                    }
                )]
            ));
        }
    }
    
    instructions
}

/// Apply hail damage to non-immune Pokemon
fn apply_hail_damage(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    for position in battle_state.get_all_active_positions() {
        if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
            // Skip if immune to hail
            if is_hail_immune(pokemon) {
                continue;
            }
            
            let damage = (pokemon.max_hp / 16).max(1);
            instructions.push(BattleInstructions::new(
                100.0,
                vec![BattleInstruction::Pokemon(
                    PokemonInstruction::Damage {
                        target: position,
                        amount: damage,
                        previous_hp: Some(pokemon.hp),
                    }
                )]
            ));
        }
    }
    
    instructions
}

/// Check if Pokemon is immune to sandstorm damage
fn is_sandstorm_immune(pokemon: &crate::core::battle_state::Pokemon) -> bool {
    // Immune types
    if pokemon.types.contains(&"Ground".to_string()) ||
       pokemon.types.contains(&"Rock".to_string()) ||
       pokemon.types.contains(&"Steel".to_string()) {
        return true;
    }
    
    // Immune abilities
    match pokemon.ability.as_str() {
        "sandveil" | "sandrush" | "sandforce" | "overcoat" | "magicguard" => true,
        _ => false,
    }
}

/// Check if Pokemon is immune to hail damage
fn is_hail_immune(pokemon: &crate::core::battle_state::Pokemon) -> bool {
    // Immune types
    if pokemon.types.contains(&"Ice".to_string()) {
        return true;
    }
    
    // Immune abilities
    match pokemon.ability.as_str() {
        "icebody" | "snowcloak" | "overcoat" | "magicguard" => true,
        _ => false,
    }
}

/// Trigger weather-related abilities (placeholder for now)
fn trigger_weather_abilities(
    _battle_state: &BattleState
) -> Vec<BattleInstructions> {
    // TODO: Implement weather ability triggers like:
    // - Dry Skin in rain/sun
    // - Rain Dish in rain
    // - Solar Power in sun
    // - Ice Body in hail
    Vec::new()
}

/// Apply terrain effects
fn apply_terrain_effects(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    match battle_state.terrain() {
        Terrain::Grassy => {
            instructions.extend(apply_grassy_terrain_healing(battle_state));
        }
        _ => {}
    }
    
    instructions
}

/// Apply Grassy Terrain healing
fn apply_grassy_terrain_healing(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    for position in battle_state.get_all_active_positions() {
        if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
            // Only heal grounded Pokemon
            if is_grounded(pokemon) {
                let healing = (pokemon.max_hp / 16).max(1);
                if pokemon.hp < pokemon.max_hp {
                    instructions.push(BattleInstructions::new(
                        100.0,
                        vec![BattleInstruction::Pokemon(
                            PokemonInstruction::Heal {
                                target: position,
                                amount: healing,
                                previous_hp: Some(pokemon.hp),
                            }
                        )]
                    ));
                }
            }
        }
    }
    
    instructions
}

/// Check if Pokemon is grounded (affected by terrain effects)
fn is_grounded(pokemon: &crate::core::battle_state::Pokemon) -> bool {
    // Not grounded if Flying type or has Air Balloon or Levitate
    if pokemon.types.contains(&"Flying".to_string()) {
        return false;
    }
    
    if let Some(ref item) = pokemon.item {
        if item == "airballoon" {
            return false;
        }
    }
    
    if pokemon.ability == "levitate" {
        return false;
    }
    
    // Check for Magnet Rise, Telekinesis, etc.
    if pokemon.volatile_statuses.contains(&VolatileStatus::MagnetRise) ||
       pokemon.volatile_statuses.contains(&VolatileStatus::Telekinesis) {
        return false;
    }
    
    true
}

/// Decrement field effect timers
fn decrement_field_timers(
    _battle_state: &BattleState
) -> Vec<BattleInstructions> {
    // TODO: Implement field timer decrementation
    // - Trick Room
    // - Light Screen / Reflect
    // - Aurora Veil
    // - Tailwind
    // - etc.
    Vec::new()
}

/// Apply status condition damage (burn, poison, toxic) with ability interactions
fn apply_status_damage(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for position in battle_state.get_all_active_positions() {
            if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
                // Check for abilities that modify status damage
                let blocks_status_damage = match pokemon.ability.as_str() {
                    "magicguard" => true, // Magic Guard prevents all indirect damage
                    "poisonheal" => matches!(pokemon.status, PokemonStatus::Poison | PokemonStatus::BadlyPoisoned),
                    _ => false,
                };
                
                if blocks_status_damage {
                    continue;
                }
                
                let damage = match pokemon.status {
                    PokemonStatus::Burn => {
                        Some((pokemon.max_hp / 16).max(1))
                    }
                    PokemonStatus::Poison => {
                        Some((pokemon.max_hp / 8).max(1))
                    }
                    PokemonStatus::BadlyPoisoned => {
                        // Toxic damage increases each turn
                        // TODO: Track toxic counter properly
                        let toxic_counter = pokemon.status_duration.unwrap_or(1);
                        Some((pokemon.max_hp * toxic_counter as i16 / 16).max(1))
                    }
                    _ => None,
                };
                
                if let Some(damage_amount) = damage {
                    instructions.push(BattleInstructions::new(
                        100.0,
                        vec![BattleInstruction::Pokemon(
                            PokemonInstruction::Damage {
                                target: position,
                                amount: damage_amount,
                                previous_hp: Some(pokemon.hp),
                            }
                        )]
                    ));
                }
            }
        }
    
    instructions
}

/// Trigger end-of-turn abilities using the centralized ability system
fn trigger_end_of_turn_abilities_wrapper(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    // Use the centralized ability trigger system
    let ability_instructions = super::ability_triggers::trigger_end_of_turn_abilities(battle_state);
    
    // Convert individual instructions to BattleInstructions
    let mut result = Vec::new();
    for instruction in ability_instructions {
        result.push(BattleInstructions::new(100.0, vec![instruction]));
    }
    
    result
}

/// Apply item end-of-turn effects
fn apply_item_effects(
    battle_state: &BattleState
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for position in battle_state.get_all_active_positions() {
        if let Some(pokemon) = battle_state.get_pokemon_at_position(position) {
            if let Some(ref item) = pokemon.item {
                match item.as_str() {
                    "leftovers" => {
                        if pokemon.hp < pokemon.max_hp {
                            let healing = (pokemon.max_hp / 16).max(1);
                            instructions.push(BattleInstructions::new(
                                100.0,
                                vec![BattleInstruction::Pokemon(
                                    PokemonInstruction::Heal {
                                        target: position,
                                        amount: healing,
                                        previous_hp: Some(pokemon.hp),
                                    }
                                )]
                            ));
                        }
                    }
                    "blacksludge" => {
                        if pokemon.types.contains(&"Poison".to_string()) {
                            // Heal if Poison type
                            if pokemon.hp < pokemon.max_hp {
                                let healing = (pokemon.max_hp / 16).max(1);
                                instructions.push(BattleInstructions::new(
                                    100.0,
                                    vec![BattleInstruction::Pokemon(
                                        PokemonInstruction::Heal {
                                            target: position,
                                            amount: healing,
                                            previous_hp: Some(pokemon.hp),
                                        }
                                    )]
                                ));
                            }
                        } else {
                            // Damage if not Poison type
                            let damage = (pokemon.max_hp / 8).max(1);
                            instructions.push(BattleInstructions::new(
                                100.0,
                                vec![BattleInstruction::Pokemon(
                                    PokemonInstruction::Damage {
                                        target: position,
                                        amount: damage,
                                        previous_hp: Some(pokemon.hp),
                                    }
                                )]
                            ));
                        }
                    }
                    "stickybarb" => {
                        let damage = (pokemon.max_hp / 8).max(1);
                        instructions.push(BattleInstructions::new(
                            100.0,
                            vec![BattleInstruction::Pokemon(
                                PokemonInstruction::Damage {
                                    target: position,
                                    amount: damage,
                                    previous_hp: Some(pokemon.hp),
                                }
                            )]
                        ));
                    }
                    _ => {}
                }
            }
        }
    }
    
    instructions
}