//! # End-of-Turn Processing
//! 
//! This module handles all end-of-turn effects that occur after both Pokemon have made their moves.
//! This includes status damage, weather effects, terrain effects, volatile status decrements,
//! side condition decrements, and other ongoing effects.
//!
//! Following poke-engine's pattern, end-of-turn processing is critical for battle state consistency
//! and includes proper ordering of effects to match official game mechanics.

use crate::battle_format::BattlePosition;
use crate::instruction::{
    Instruction, StateInstructions, PositionDamageInstruction, PositionHealInstruction,
    PokemonStatus, Weather, Terrain, ChangeVolatileStatusDurationInstruction,
    RemoveVolatileStatusInstruction, ChangeStatusDurationInstruction, RemoveStatusInstruction,
    DecrementSideConditionDurationInstruction, RemoveSideConditionInstruction
};
use crate::state::State;
use crate::generation::GenerationMechanics;

/// Process all end-of-turn effects and generate appropriate instructions
/// This function handles the complete end-of-turn sequence following official game order
pub fn process_end_of_turn_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Process effects in the correct order according to game mechanics
    
    // 1. Field effect decrements (weather, terrain, trick room)
    instructions.extend(process_field_effect_decrements(state, generation));
    
    // 2. Status damage (burn, poison, etc.)
    instructions.extend(process_status_damage(state, generation));
    
    // 3. Weather damage/healing
    instructions.extend(process_weather_effects(state, generation));
    
    // 4. Terrain effects
    instructions.extend(process_terrain_effects(state, generation));
    
    // 5. Volatile status decrements and cleanup
    instructions.extend(process_volatile_status_decrements(state, generation));
    
    // 6. Side condition decrements
    instructions.extend(process_side_condition_decrements(state, generation));
    
    // 7. Special ongoing effects (wish, future sight, etc.)
    instructions.extend(process_special_ongoing_effects(state, generation));
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Process field effect decrements (weather, terrain, trick room)
fn process_field_effect_decrements(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    let mut instruction_list = Vec::new();
    
    // Decrement weather turns if active
    if state.weather != Weather::NONE && state.weather_turns_remaining.map_or(false, |turns| turns > 0) {
        instruction_list.push(Instruction::DecrementWeatherTurns);
    }
    
    // Decrement terrain turns if active
    if state.terrain != Terrain::NONE && state.terrain_turns_remaining.map_or(false, |turns| turns > 0) {
        instruction_list.push(Instruction::DecrementTerrainTurns);
    }
    
    // Decrement trick room turns if active
    // TODO: Add trick room fields to State struct
    // if state.trick_room_active && state.trick_room_turns_remaining > 0 {
    //     instruction_list.push(Instruction::DecrementTrickRoomTurns);
    // }
    
    if !instruction_list.is_empty() {
        instructions.push(StateInstructions::new(100.0, instruction_list));
    }
    
    instructions
}

/// Process status damage from burn, poison, etc.
fn process_status_damage(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Process both sides
    for side_ref in [crate::battle_format::SideReference::SideOne, crate::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        // Check all active Pokemon on this side
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                let position = BattlePosition::new(side_ref, slot);
                
                match pokemon.status {
                    PokemonStatus::BURN => {
                        let damage = calculate_burn_damage(pokemon, generation);
                        if damage > 0 {
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::PositionDamage(PositionDamageInstruction {
                                    target_position: position,
                                    damage_amount: damage,
                                })
                            ]));
                        }
                    }
                    PokemonStatus::POISON => {
                        let damage = calculate_poison_damage(pokemon, generation);
                        if damage > 0 {
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::PositionDamage(PositionDamageInstruction {
                                    target_position: position,
                                    damage_amount: damage,
                                })
                            ]));
                        }
                    }
                    PokemonStatus::TOXIC => {
                        let damage = calculate_toxic_damage(pokemon, generation);
                        if damage > 0 {
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::PositionDamage(PositionDamageInstruction {
                                    target_position: position,
                                    damage_amount: damage,
                                })
                            ]));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    instructions
}

/// Process weather effects (damage/healing)
fn process_weather_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    match state.weather {
        Weather::SAND => {
            instructions.extend(process_sandstorm_damage(state, generation));
        }
        Weather::HAIL => {
            instructions.extend(process_hail_damage(state, generation));
        }
        Weather::SNOW => {
            // Snow doesn't deal damage in modern generations
            // But might have other effects
        }
        _ => {}
    }
    
    instructions
}

/// Process sandstorm damage
fn process_sandstorm_damage(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::battle_format::SideReference::SideOne, crate::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                // Pokemon immune to sandstorm: Rock, Ground, Steel types
                let is_immune = pokemon.types.iter().any(|t| {
                    matches!(t.to_lowercase().as_str(), "rock" | "ground" | "steel")
                });
                
                if !is_immune {
                    let position = BattlePosition::new(side_ref, slot);
                    let damage = pokemon.max_hp / 16; // 1/16 max HP
                    
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionDamage(PositionDamageInstruction {
                            target_position: position,
                            damage_amount: damage,
                        })
                    ]));
                }
            }
        }
    }
    
    instructions
}

/// Process hail damage
fn process_hail_damage(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::battle_format::SideReference::SideOne, crate::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                // Pokemon immune to hail: Ice types
                let is_immune = pokemon.types.iter().any(|t| {
                    t.to_lowercase() == "ice"
                });
                
                if !is_immune {
                    let position = BattlePosition::new(side_ref, slot);
                    let damage = pokemon.max_hp / 16; // 1/16 max HP
                    
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionDamage(PositionDamageInstruction {
                            target_position: position,
                            damage_amount: damage,
                        })
                    ]));
                }
            }
        }
    }
    
    instructions
}

/// Process terrain effects
fn process_terrain_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    match state.terrain {
        Terrain::GRASSYTERRAIN => {
            instructions.extend(process_grassy_terrain_healing(state, generation));
        }
        _ => {}
    }
    
    instructions
}

/// Process Grassy Terrain healing
fn process_grassy_terrain_healing(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::battle_format::SideReference::SideOne, crate::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                // Only grounded Pokemon are healed by Grassy Terrain
                // TODO: Add is_grounded method to Pokemon
                if pokemon.hp < pokemon.max_hp {
                    let position = BattlePosition::new(side_ref, slot);
                    let heal_amount = pokemon.max_hp / 16; // 1/16 max HP
                    
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionHeal(PositionHealInstruction {
                            target_position: position,
                            heal_amount,
                        })
                    ]));
                }
            }
        }
    }
    
    instructions
}

/// Process volatile status decrements and cleanup
fn process_volatile_status_decrements(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Process volatile status decrements for all active Pokemon
    for side_ref in [crate::battle_format::SideReference::SideOne, crate::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                let position = BattlePosition::new(side_ref, slot);
                let mut instruction_list = Vec::new();
                
                // Check each volatile status with duration
                for (volatile_status, &duration) in &pokemon.volatile_status_durations {
                    match volatile_status {
                        crate::instruction::VolatileStatus::Confusion => {
                            if duration > 1 {
                                // Decrement confusion duration
                                instruction_list.push(Instruction::ChangeVolatileStatusDuration(
                                    ChangeVolatileStatusDurationInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                // Remove confusion when duration expires
                                instruction_list.push(Instruction::RemoveVolatileStatus(
                                    RemoveVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                    }
                                ));
                            }
                        }
                        crate::instruction::VolatileStatus::Encore => {
                            if duration > 1 {
                                instruction_list.push(Instruction::ChangeVolatileStatusDuration(
                                    ChangeVolatileStatusDurationInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                instruction_list.push(Instruction::RemoveVolatileStatus(
                                    RemoveVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                    }
                                ));
                            }
                        }
                        crate::instruction::VolatileStatus::Disable => {
                            if duration > 1 {
                                instruction_list.push(Instruction::ChangeVolatileStatusDuration(
                                    ChangeVolatileStatusDurationInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                instruction_list.push(Instruction::RemoveVolatileStatus(
                                    RemoveVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                    }
                                ));
                            }
                        }
                        crate::instruction::VolatileStatus::Taunt => {
                            if duration > 1 {
                                instruction_list.push(Instruction::ChangeVolatileStatusDuration(
                                    ChangeVolatileStatusDurationInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                instruction_list.push(Instruction::RemoveVolatileStatus(
                                    RemoveVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                    }
                                ));
                            }
                        }
                        // Add other volatile statuses with durations as needed
                        _ => {
                            // For volatile statuses without specific duration logic,
                            // just decrement if duration > 1, remove if duration = 1
                            if duration > 1 {
                                instruction_list.push(Instruction::ChangeVolatileStatusDuration(
                                    ChangeVolatileStatusDurationInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                instruction_list.push(Instruction::RemoveVolatileStatus(
                                    RemoveVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                    }
                                ));
                            }
                        }
                    }
                }
                
                // Process status condition duration decrements (Sleep, Freeze)
                if let Some(status_duration) = pokemon.status_duration {
                    match pokemon.status {
                        crate::instruction::PokemonStatus::SLEEP => {
                            if status_duration > 1 {
                                instruction_list.push(Instruction::ChangeStatusDuration(
                                    ChangeStatusDurationInstruction {
                                        target_position: position,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                // Wake up when sleep duration expires
                                instruction_list.push(Instruction::RemoveStatus(
                                    RemoveStatusInstruction {
                                        target_position: position,
                                    }
                                ));
                            }
                        }
                        crate::instruction::PokemonStatus::FREEZE => {
                            // Freeze has a chance to thaw each turn in addition to duration
                            // For now, just handle duration-based thawing
                            if status_duration > 1 {
                                instruction_list.push(Instruction::ChangeStatusDuration(
                                    ChangeStatusDurationInstruction {
                                        target_position: position,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                instruction_list.push(Instruction::RemoveStatus(
                                    RemoveStatusInstruction {
                                        target_position: position,
                                    }
                                ));
                            }
                        }
                        _ => {}
                    }
                }
                
                if !instruction_list.is_empty() {
                    instructions.push(StateInstructions::new(100.0, instruction_list));
                }
            }
        }
    }
    
    instructions
}

/// Process side condition decrements
fn process_side_condition_decrements(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Process side condition decrements for both sides
    for side_ref in [crate::battle_format::SideReference::SideOne, crate::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        let mut instruction_list = Vec::new();
        
        // Check each side condition that has duration
        for (condition, &count) in &side.side_conditions {
            match condition {
                crate::instruction::SideCondition::Reflect => {
                    // Reflect lasts 5 turns (8 with Light Clay)
                    if count > 1 {
                        instruction_list.push(Instruction::DecrementSideConditionDuration(
                            DecrementSideConditionDurationInstruction {
                                side: side_ref,
                                condition: *condition,
                                amount: 1,
                            }
                        ));
                    } else {
                        instruction_list.push(Instruction::RemoveSideCondition(
                            RemoveSideConditionInstruction {
                                side: side_ref,
                                condition: *condition,
                            }
                        ));
                    }
                }
                crate::instruction::SideCondition::LightScreen => {
                    // Light Screen lasts 5 turns (8 with Light Clay)
                    if count > 1 {
                        instruction_list.push(Instruction::DecrementSideConditionDuration(
                            DecrementSideConditionDurationInstruction {
                                side: side_ref,
                                condition: *condition,
                                amount: 1,
                            }
                        ));
                    } else {
                        instruction_list.push(Instruction::RemoveSideCondition(
                            RemoveSideConditionInstruction {
                                side: side_ref,
                                condition: *condition,
                            }
                        ));
                    }
                }
                crate::instruction::SideCondition::AuroraVeil => {
                    // Aurora Veil lasts 5 turns (8 with Light Clay)
                    if count > 1 {
                        instruction_list.push(Instruction::DecrementSideConditionDuration(
                            DecrementSideConditionDurationInstruction {
                                side: side_ref,
                                condition: *condition,
                                amount: 1,
                            }
                        ));
                    } else {
                        instruction_list.push(Instruction::RemoveSideCondition(
                            RemoveSideConditionInstruction {
                                side: side_ref,
                                condition: *condition,
                            }
                        ));
                    }
                }
                crate::instruction::SideCondition::TailWind => {
                    // Tailwind lasts 4 turns
                    if count > 1 {
                        instruction_list.push(Instruction::DecrementSideConditionDuration(
                            DecrementSideConditionDurationInstruction {
                                side: side_ref,
                                condition: *condition,
                                amount: 1,
                            }
                        ));
                    } else {
                        instruction_list.push(Instruction::RemoveSideCondition(
                            RemoveSideConditionInstruction {
                                side: side_ref,
                                condition: *condition,
                            }
                        ));
                    }
                }
                // Entry hazards like Spikes, Stealth Rock, etc. don't have durations
                // They persist until removed by moves or switching
                crate::instruction::SideCondition::Spikes |
                crate::instruction::SideCondition::ToxicSpikes |
                crate::instruction::SideCondition::StealthRock |
                crate::instruction::SideCondition::StickyWeb => {
                    // No duration-based removal for entry hazards
                }
                // Guards last only one turn and are handled elsewhere
                crate::instruction::SideCondition::WideGuard |
                crate::instruction::SideCondition::QuickGuard => {
                    // These are removed at the end of the turn they're used
                    instruction_list.push(Instruction::RemoveSideCondition(
                        RemoveSideConditionInstruction {
                            side: side_ref,
                            condition: *condition,
                        }
                    ));
                }
                // Other side conditions without specific duration logic
                _ => {
                    // Handle generic duration-based side conditions
                    if count > 1 {
                        instruction_list.push(Instruction::DecrementSideConditionDuration(
                            DecrementSideConditionDurationInstruction {
                                side: side_ref,
                                condition: *condition,
                                amount: 1,
                            }
                        ));
                    } else {
                        instruction_list.push(Instruction::RemoveSideCondition(
                            RemoveSideConditionInstruction {
                                side: side_ref,
                                condition: *condition,
                            }
                        ));
                    }
                }
            }
        }
        
        if !instruction_list.is_empty() {
            instructions.push(StateInstructions::new(100.0, instruction_list));
        }
    }
    
    instructions
}

/// Process special ongoing effects (wish, future sight, etc.)
fn process_special_ongoing_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // TODO: Implement special ongoing effects
    // This would handle things like:
    // - Wish healing activation
    // - Future Sight damage activation
    // - Perish Song countdown
    // - etc.
    
    instructions
}

// =============================================================================
// DAMAGE CALCULATION FUNCTIONS
// =============================================================================

/// Calculate burn damage (generation-aware)
fn calculate_burn_damage(pokemon: &crate::state::Pokemon, generation: &GenerationMechanics) -> i16 {
    if generation.generation.number() >= 7 {
        // Gen 7+: 1/16 max HP
        pokemon.max_hp / 16
    } else {
        // Gen 1-6: 1/8 max HP
        pokemon.max_hp / 8
    }
}

/// Calculate poison damage
fn calculate_poison_damage(pokemon: &crate::state::Pokemon, generation: &GenerationMechanics) -> i16 {
    // Regular poison: 1/8 max HP in all generations
    pokemon.max_hp / 8
}

/// Calculate toxic damage (increases each turn)
fn calculate_toxic_damage(pokemon: &crate::state::Pokemon, generation: &GenerationMechanics) -> i16 {
    // Toxic damage increases each turn
    // This would need access to the toxic counter from the Pokemon state
    // For now, use base toxic damage
    pokemon.max_hp / 16
}