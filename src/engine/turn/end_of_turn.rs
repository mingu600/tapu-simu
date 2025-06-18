//! # End-of-Turn Processing
//! 
//! This module handles all end-of-turn effects that occur after both Pokemon have made their moves.
//! This includes status damage, weather effects, terrain effects, volatile status decrements,
//! side condition decrements, and other ongoing effects.
//!
//! Following poke-engine's pattern, end-of-turn processing is critical for battle state consistency
//! and includes proper ordering of effects to match official game mechanics.

use crate::core::battle_format::BattlePosition;
use crate::core::instruction::{
    Instruction, StateInstructions, PositionDamageInstruction, PositionHealInstruction,
    PokemonStatus, Weather, Terrain, ChangeVolatileStatusDurationInstruction,
    RemoveVolatileStatusInstruction, ChangeStatusDurationInstruction, RemoveStatusInstruction,
    DecrementSideConditionDurationInstruction, RemoveSideConditionInstruction,
    ApplyStatusInstruction, ApplyVolatileStatusInstruction, FormeChangeInstruction,
    ChangeItemInstruction, DecrementPPInstruction
};
use crate::core::state::State;
use crate::generation::GenerationMechanics;
use std::collections::HashMap;
use rand;

/// Process all end-of-turn effects and generate appropriate instructions
/// This function handles the complete end-of-turn sequence following official game order
pub fn process_end_of_turn_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Process effects in the correct order according to poke-engine mechanics
    
    // 1. Field effect decrements (weather, terrain, trick room)
    instructions.extend(process_field_effect_decrements(state, generation));
    
    // 2. Side condition decrements
    instructions.extend(process_side_condition_decrements(state, generation));
    
    // 3. Weather damage/healing (sandstorm, hail)
    instructions.extend(process_weather_effects(state, generation));
    
    // 4. Future Sight countdown/activation
    instructions.extend(process_future_sight_attacks(state, generation));
    
    // 5. Wish countdown/activation
    instructions.extend(process_wish_healing(state, generation));
    
    // 6. Status damage (burn, poison, toxic)
    instructions.extend(process_status_damage(state, generation));
    
    // 7. Item end-of-turn effects (Leftovers, Black Sludge, etc.)
    instructions.extend(process_item_effects(state, generation));
    
    // 8. Ability end-of-turn effects (Speed Boost, etc.)
    instructions.extend(process_ability_effects(state, generation));
    
    // 9. Leech Seed damage/healing
    instructions.extend(process_leech_seed_effects(state, generation));
    
    // 10. Volatile status decrements and cleanup
    instructions.extend(process_volatile_status_decrements(state, generation));
    
    // 11. Terrain effects (Grassy Terrain healing)
    instructions.extend(process_terrain_effects(state, generation));
    
    // 12. Perish Song countdown
    instructions.extend(process_perish_song_effects(state, generation));
    
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
    if state.weather != Weather::None && state.weather_turns_remaining.map_or(false, |turns| turns > 0) {
        if state.weather_turns_remaining == Some(1) {
            // Weather ends this turn
            instruction_list.push(Instruction::ChangeWeather(
                crate::core::instruction::ChangeWeatherInstruction {
                    weather: Weather::None,
                    duration: None,
                    previous_weather: Some(state.weather),
                    previous_duration: Some(state.weather_turns_remaining),
                }
            ));
        } else {
            instruction_list.push(Instruction::DecrementWeatherTurns);
        }
    }
    
    // Decrement terrain turns if active
    if state.terrain != Terrain::None && state.terrain_turns_remaining.map_or(false, |turns| turns > 0) {
        if state.terrain_turns_remaining == Some(1) {
            // Terrain ends this turn
            instruction_list.push(Instruction::ChangeTerrain(
                crate::core::instruction::ChangeTerrainInstruction {
                    terrain: Terrain::None,
                    duration: None,
                    previous_terrain: Some(state.terrain),
                    previous_duration: Some(state.terrain_turns_remaining),
                }
            ));
        } else {
            instruction_list.push(Instruction::DecrementTerrainTurns);
        }
    }
    
    // Decrement trick room turns if active
    if state.trick_room_active && state.trick_room_turns_remaining.map_or(false, |turns| turns > 0) {
        if state.trick_room_turns_remaining == Some(1) {
            // Trick Room ends this turn
            instruction_list.push(Instruction::ToggleTrickRoom(
                crate::core::instruction::ToggleTrickRoomInstruction {
                    active: false,
                    duration: None,
                }
            ));
        } else {
            instruction_list.push(Instruction::DecrementTrickRoomTurns);
        }
    }
    
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
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        // Check all active Pokemon on this side
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                let position = BattlePosition::new(side_ref, slot);
                
                match pokemon.status {
                    PokemonStatus::Burn => {
                        let damage = calculate_burn_damage(pokemon, generation);
                        if damage > 0 {
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
            })
                            ]));
                        }
                    }
                    PokemonStatus::Poison => {
                        let damage = calculate_poison_damage(pokemon, generation);
                        if damage > 0 {
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
            })
                            ]));
                        } else if damage < 0 {
                            // Poison Heal - convert damage to healing
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::PositionHeal(PositionHealInstruction {
                target_position: position,
                heal_amount: -damage,
                previous_hp: Some(0),
            })
                            ]));
                        }
                    }
                    PokemonStatus::Toxic => {
                        let damage = calculate_toxic_damage(pokemon, generation);
                        let mut instruction_list = Vec::new();
                        
                        if damage > 0 {
                            instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
            }));
                        } else if damage < 0 {
                            // Poison Heal - convert damage to healing
                            instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
                target_position: position,
                heal_amount: -damage,
                previous_hp: Some(0),
            }));
                        }
                        
                        // Increment toxic counter for next turn (even with Poison Heal)
                        instruction_list.push(Instruction::ChangeVolatileStatusDuration(
                            ChangeVolatileStatusDurationInstruction {
                                target_position: position,
                                volatile_status: crate::core::instruction::VolatileStatus::ToxicCount,
                                duration_change: 1,
                            }
                        ));
                        
                        if !instruction_list.is_empty() {
                            instructions.push(StateInstructions::new(100.0, instruction_list));
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
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                // Check for Magic Guard or Overcoat abilities (prevents weather damage)
                let ability_immune = matches!(pokemon.ability.to_lowercase().replace(" ", "").as_str(), "magicguard" | "overcoat");
                
                // Pokemon immune to sandstorm: Rock, Ground, Steel types
                let type_immune = pokemon.types.iter().any(|t| {
                    matches!(t.to_lowercase().as_str(), "rock" | "ground" | "steel")
                });
                
                if !ability_immune && !type_immune {
                    let position = BattlePosition::new(side_ref, slot);
                    let damage = pokemon.max_hp / 16; // 1/16 max HP
                    
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
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
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                // Check for Magic Guard, Overcoat, or Ice Body abilities
                let ability_immune = matches!(pokemon.ability.to_lowercase().replace(" ", "").as_str(), "magicguard" | "overcoat" | "icebody");
                
                // Pokemon immune to hail: Ice types
                let type_immune = pokemon.types.iter().any(|t| {
                    t.to_lowercase() == "ice"
                });
                
                if !ability_immune && !type_immune {
                    let position = BattlePosition::new(side_ref, slot);
                    let damage = pokemon.max_hp / 16; // 1/16 max HP
                    
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
            })
                    ]));
                } else if pokemon.ability.to_lowercase().replace(" ", "") == "icebody" {
                    // Ice Body heals 1/16 HP in hail
                    let position = BattlePosition::new(side_ref, slot);
                    let heal_amount = pokemon.max_hp / 16;
                    
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionHeal(PositionHealInstruction {
                            target_position: position,
                            heal_amount,
                            previous_hp: Some(0),
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
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                // Only grounded Pokemon are healed by Grassy Terrain
                if pokemon.hp < pokemon.max_hp && pokemon.hp > 0 && is_pokemon_grounded(pokemon) {
                    let position = BattlePosition::new(side_ref, slot);
                    let heal_amount = pokemon.max_hp / 16; // 1/16 max HP
                    
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionHeal(PositionHealInstruction {
                            target_position: position,
                            heal_amount,
                            previous_hp: Some(0),
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
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                let position = BattlePosition::new(side_ref, slot);
                let mut instruction_list = Vec::new();
                
                // Check each volatile status with duration
                for (volatile_status, &duration) in &pokemon.volatile_status_durations {
                    match volatile_status {
                        crate::core::instruction::VolatileStatus::SaltCure => {
                            // Salt Cure ongoing damage (1/8 max HP, 1/4 for Water/Steel types)
                            let damage = calculate_salt_cure_damage(pokemon);
                            if damage > 0 {
                                instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
            }));
                            }
                        }
                        crate::core::instruction::VolatileStatus::Yawn => {
                            if duration > 1 {
                                // Decrement Yawn countdown
                                instruction_list.push(Instruction::ChangeVolatileStatusDuration(
                                    ChangeVolatileStatusDurationInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                // Apply sleep when Yawn countdown reaches 0
                                if pokemon.status == PokemonStatus::None {
                                    instruction_list.push(Instruction::ApplyStatus(
                                        ApplyStatusInstruction {
                target_position: position,
                status: PokemonStatus::Sleep,
                previous_status: Some(PokemonStatus::None),
                previous_status_duration: Some(None),
            }
                                    ));
                                }
                                // Remove Yawn volatile status
                                instruction_list.push(Instruction::RemoveVolatileStatus(
                                    RemoveVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                    }
                                ));
                            }
                        }
                        crate::core::instruction::VolatileStatus::LockedMove => {
                            // Handle locked move mechanics (Outrage, Petal Dance, etc.)
                            if duration > 1 {
                                instruction_list.push(Instruction::ChangeVolatileStatusDuration(
                                    ChangeVolatileStatusDurationInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                        duration_change: -1,
                                    }
                                ));
                            } else {
                                // Apply confusion when locked move ends
                                instruction_list.push(Instruction::ApplyVolatileStatus(
                                    ApplyVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: crate::core::instruction::VolatileStatus::Confusion,
                                        duration: Some(2 + (rand::random::<u8>() % 3)), // 2-4 turns
                                    }
                                ));
                                // Remove locked move status
                                instruction_list.push(Instruction::RemoveVolatileStatus(
                                    RemoveVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: *volatile_status,
                                    }
                                ));
                            }
                        }
                        crate::core::instruction::VolatileStatus::Confusion => {
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
                        crate::core::instruction::VolatileStatus::Encore => {
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
                        crate::core::instruction::VolatileStatus::Disable => {
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
                        crate::core::instruction::VolatileStatus::Taunt => {
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
                        crate::core::instruction::PokemonStatus::Sleep => {
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
                previous_status: Some(PokemonStatus::None),
                previous_status_duration: Some(None),
            }
                                ));
                            }
                        }
                        crate::core::instruction::PokemonStatus::Freeze => {
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
                previous_status: Some(PokemonStatus::None),
                previous_status_duration: Some(None),
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
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        let mut instruction_list = Vec::new();
        
        // Check each side condition that has duration
        for (condition, &count) in &side.side_conditions {
            match condition {
                crate::core::instruction::SideCondition::Reflect => {
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
                crate::core::instruction::SideCondition::LightScreen => {
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
                crate::core::instruction::SideCondition::AuroraVeil => {
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
                crate::core::instruction::SideCondition::TailWind => {
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
                crate::core::instruction::SideCondition::Spikes |
                crate::core::instruction::SideCondition::ToxicSpikes |
                crate::core::instruction::SideCondition::StealthRock |
                crate::core::instruction::SideCondition::StickyWeb => {
                    // No duration-based removal for entry hazards
                }
                // Guards last only one turn and are handled elsewhere
                crate::core::instruction::SideCondition::WideGuard |
                crate::core::instruction::SideCondition::QuickGuard => {
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
    
    // Process Wish healing
    instructions.extend(process_wish_healing(state, generation));
    
    // Process Future Sight attacks
    instructions.extend(process_future_sight_attacks(state, generation));
    
    // Process Leech Seed damage/healing
    instructions.extend(process_leech_seed_effects(state, generation));
    
    // Process Perish Song countdown (handled via volatile status decrements)
    instructions.extend(process_perish_song_effects(state, generation));
    
    instructions
}

// =============================================================================
// DAMAGE CALCULATION FUNCTIONS
// =============================================================================

/// Calculate burn damage (generation-aware)
fn calculate_burn_damage(pokemon: &crate::core::state::Pokemon, generation: &GenerationMechanics) -> i16 {
    // Check for Magic Guard ability (prevents all indirect damage)
    if pokemon.ability.to_lowercase().replace(" ", "") == "magicguard" {
        return 0;
    }
    
    let base_damage = if generation.generation.number() >= 7 {
        // Gen 7+: 1/16 max HP
        pokemon.max_hp / 16
    } else {
        // Gen 3-6: 1/8 max HP  
        pokemon.max_hp / 8
    };
    
    // Check for Heatproof ability (reduces burn damage by half)
    if pokemon.ability.to_lowercase().replace(" ", "") == "heatproof" {
        base_damage / 2
    } else {
        base_damage
    }
}

/// Calculate poison damage
fn calculate_poison_damage(pokemon: &crate::core::state::Pokemon, generation: &GenerationMechanics) -> i16 {
    // Check for Magic Guard ability (prevents all indirect damage)
    if pokemon.ability.to_lowercase().replace(" ", "") == "magicguard" {
        return 0;
    }
    
    // Check for Poison Heal ability (heals instead of damages)
    if pokemon.ability.to_lowercase().replace(" ", "") == "poisonheal" {
        return -(pokemon.max_hp / 8); // Negative damage = healing
    }
    
    // Regular poison: 1/8 max HP in all generations
    pokemon.max_hp / 8
}

/// Calculate toxic damage (increases each turn)
fn calculate_toxic_damage(pokemon: &crate::core::state::Pokemon, generation: &GenerationMechanics) -> i16 {
    // Check for Magic Guard ability (prevents all indirect damage)
    if pokemon.ability.to_lowercase().replace(" ", "") == "magicguard" {
        return 0;
    }
    
    // Check for Poison Heal ability (heals instead of damages)
    if pokemon.ability.to_lowercase().replace(" ", "") == "poisonheal" {
        // With Poison Heal, toxic still heals 1/8 max HP regardless of toxic counter
        return -(pokemon.max_hp / 8); // Negative damage = healing
    }
    
    // Get toxic counter from volatile status duration
    // If not found, assume it's the first turn (counter = 1)
    let toxic_count = pokemon.volatile_status_durations
        .get(&crate::core::instruction::VolatileStatus::ToxicCount)
        .copied()
        .unwrap_or(1) as f32;
    
    // Toxic damage formula: (1/16) * toxic_count of max HP
    // First turn: 1/16, Second turn: 2/16, Third turn: 3/16, etc.
    let damage_fraction = (1.0 / 16.0) * toxic_count;
    (pokemon.max_hp as f32 * damage_fraction) as i16
}

// =============================================================================
// SPECIAL ONGOING EFFECTS FUNCTIONS
// =============================================================================

/// Process Wish healing
fn process_wish_healing(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        let mut instruction_list = Vec::new();
        
        // Check each slot for wish healing
        for (slot, (heal_amount, turns_remaining)) in &side.wish_healing {
            if *turns_remaining == 1 {
                // Wish activates this turn
                let position = BattlePosition::new(side_ref, *slot);
                
                // Only heal if there's a Pokemon at this position and it's not at full HP
                if let Some(pokemon) = state.get_pokemon_at_position(position) {
                    if pokemon.hp < pokemon.max_hp && pokemon.hp > 0 {
                        instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
                target_position: position,
                heal_amount: (*heal_amount).min(pokemon.max_hp - pokemon.hp),
                previous_hp: Some(0),
            }));
                    }
                }
                
                // Remove the wish after activation
                instruction_list.push(Instruction::DecrementWish(
                    crate::core::instruction::DecrementWishInstruction {
                        target_position: position,
                    }
                ));
            } else if *turns_remaining > 1 {
                // Decrement wish counter
                let position = BattlePosition::new(side_ref, *slot);
                instruction_list.push(Instruction::DecrementWish(
                    crate::core::instruction::DecrementWishInstruction {
                        target_position: position,
                    }
                ));
            }
        }
        
        if !instruction_list.is_empty() {
            instructions.push(StateInstructions::new(100.0, instruction_list));
        }
    }
    
    instructions
}

/// Process Future Sight attacks
fn process_future_sight_attacks(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        let mut instruction_list = Vec::new();
        
        // Check each slot for future sight attacks
        for (slot, (attacker_position, damage_amount, turns_remaining, move_name)) in &side.future_sight_attacks {
            if *turns_remaining == 1 {
                // Future Sight activates this turn
                let position = BattlePosition::new(side_ref, *slot);
                
                // Only damage if there's a Pokemon at this position
                if let Some(pokemon) = state.get_pokemon_at_position(position) {
                    if pokemon.hp > 0 {
                        instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: *damage_amount,
                previous_hp: Some(0),
            }));
                    }
                }
                
                // Remove the future sight after activation
                instruction_list.push(Instruction::DecrementFutureSight(
                    crate::core::instruction::DecrementFutureSightInstruction {
                        target_position: position,
                    }
                ));
            } else if *turns_remaining > 1 {
                // Decrement future sight counter
                let position = BattlePosition::new(side_ref, *slot);
                instruction_list.push(Instruction::DecrementFutureSight(
                    crate::core::instruction::DecrementFutureSightInstruction {
                        target_position: position,
                    }
                ));
            }
        }
        
        if !instruction_list.is_empty() {
            instructions.push(StateInstructions::new(100.0, instruction_list));
        }
    }
    
    instructions
}

/// Process Leech Seed effects
fn process_leech_seed_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                // Check if Pokemon has Leech Seed
                if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::LeechSeed) {
                    let position = BattlePosition::new(side_ref, slot);
                    
                    // Check for Magic Guard (prevents Leech Seed damage)
                    if pokemon.ability.to_lowercase().replace(" ", "") != "magicguard" && pokemon.hp > 0 {
                        let damage = pokemon.max_hp / 8; // 1/8 max HP
                        let opponent_side_ref = match side_ref {
                            crate::core::battle_format::SideReference::SideOne => crate::core::battle_format::SideReference::SideTwo,
                            crate::core::battle_format::SideReference::SideTwo => crate::core::battle_format::SideReference::SideOne,
                        };
                        
                        let mut instruction_list = Vec::new();
                        
                        // Damage the Leech Seeded Pokemon
                        instruction_list.push(Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
            }));
                        
                        // Heal the opponent (first active Pokemon)
                        let opponent_position = BattlePosition::new(opponent_side_ref, 0);
                        if let Some(opponent_pokemon) = state.get_pokemon_at_position(opponent_position) {
                            if opponent_pokemon.hp > 0 && opponent_pokemon.hp < opponent_pokemon.max_hp {
                                let heal_amount = damage.min(opponent_pokemon.max_hp - opponent_pokemon.hp);
                                instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
                                    target_position: opponent_position,
                                    heal_amount,
                                    previous_hp: Some(0),
                                }));
                            }
                        }
                        
                        instructions.push(StateInstructions::new(100.0, instruction_list));
                    }
                }
            }
        }
    }
    
    instructions
}

/// Process Perish Song effects
fn process_perish_song_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                let position = BattlePosition::new(side_ref, slot);
                
                // Check for Perish1 (Pokemon faints this turn)
                if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::Perish1) {
                    // Faint the Pokemon
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionDamage(PositionDamageInstruction {
                            target_position: position,
                            damage_amount: pokemon.hp, // Deal enough damage to faint
                            previous_hp: Some(pokemon.hp),
                        })
                    ]));
                }
            }
        }
    }
    
    instructions
}/// Process item end-of-turn effects
fn process_item_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                let position = BattlePosition::new(side_ref, slot);
                
                if let Some(item) = &pokemon.item {
                    match item.to_lowercase().replace(" ", "").as_str() {
                        "leftovers" => {
                            // Heal 1/16 max HP if not at full HP
                            if pokemon.hp > 0 && pokemon.hp < pokemon.max_hp {
                                let heal_amount = pokemon.max_hp / 16;
                                instructions.push(StateInstructions::new(100.0, vec![
                                    Instruction::PositionHeal(PositionHealInstruction {
                    target_position: position,
                    heal_amount,
                    previous_hp: Some(0),
                })
                                ]));
                            }
                        }
                        "blacksludge" => {
                            if pokemon.hp > 0 {
                                let is_poison_type = pokemon.types.iter().any(|t| t.to_lowercase() == "poison");
                                let amount = pokemon.max_hp / 16;
                                
                                if is_poison_type {
                                    // Heal if Poison type
                                    if pokemon.hp < pokemon.max_hp {
                                        instructions.push(StateInstructions::new(100.0, vec![
                                            Instruction::PositionHeal(PositionHealInstruction {
                target_position: position,
                heal_amount: amount,
                previous_hp: Some(0),
            })
                                        ]));
                                    }
                                } else {
                                    // Damage if not Poison type (unless Magic Guard)
                                    if pokemon.ability.to_lowercase().replace(" ", "") != "magicguard" {
                                        instructions.push(StateInstructions::new(100.0, vec![
                                            Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: amount,
                previous_hp: Some(0),
            })
                                        ]));
                                    }
                                }
                            }
                        }
                        "flameorb" => {
                            // Inflict burn if not already statused and not already burned
                            if pokemon.status == crate::core::instruction::PokemonStatus::None && pokemon.hp > 0 {
                                instructions.push(StateInstructions::new(100.0, vec![
                                    Instruction::ApplyStatus(crate::core::instruction::ApplyStatusInstruction {
                target_position: position,
                status: crate::core::instruction::PokemonStatus::Burn,
                previous_status: Some(PokemonStatus::None),
                previous_status_duration: Some(None),
            })
                                ]));
                            }
                        }
                        "toxicorb" => {
                            // Inflict toxic if not already statused and not already poisoned
                            if pokemon.status == crate::core::instruction::PokemonStatus::None && pokemon.hp > 0 {
                                instructions.push(StateInstructions::new(100.0, vec![
                                    Instruction::ApplyStatus(crate::core::instruction::ApplyStatusInstruction {
                target_position: position,
                status: crate::core::instruction::PokemonStatus::Toxic,
                previous_status: Some(PokemonStatus::None),
                previous_status_duration: Some(None),
            }),
                                    // Initialize toxic counter
                                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                                        target_position: position,
                                        volatile_status: crate::core::instruction::VolatileStatus::ToxicCount,
                                        duration: Some(1),
                                    })
                                ]));
                            }
                        }
                        // Process berry effects
                        item_name if is_end_of_turn_berry(item_name) => {
                            instructions.extend(process_berry_end_of_turn_effect(state, generation, position, pokemon, item_name));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    instructions
}

/// Process ability end-of-turn effects
fn process_ability_effects(
    state: &State,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        let side = state.get_side(side_ref);
        
        for slot in 0..state.format.active_pokemon_count() {
            if let Some(pokemon) = side.get_active_pokemon_at_slot(slot) {
                let position = BattlePosition::new(side_ref, slot);
                
                match pokemon.ability.to_lowercase().replace(" ", "").as_str() {
                    "speedboost" => {
                        // Increase Speed by 1 stage if possible
                        if pokemon.hp > 0 {
                            let current_speed_boost = pokemon.stat_boosts.get(&crate::core::instruction::Stat::Speed).copied().unwrap_or(0);
                            if current_speed_boost < 6 {
                                let mut boosts = HashMap::new();
                                boosts.insert(crate::core::instruction::Stat::Speed, 1);
                                instructions.push(StateInstructions::new(100.0, vec![
                                    Instruction::BoostStats(crate::core::instruction::BoostStatsInstruction {
                target_position: position,
                stat_boosts: boosts,
                previous_boosts: Some(HashMap::new()),
            })
                                ]));
                            }
                        }
                    }
                    "baddreams" => {
                        // Damage sleeping opponents by 1/8 max HP
                        if pokemon.hp > 0 {
                            instructions.extend(process_bad_dreams_effect(state, generation, position));
                        }
                    }
                    "hydration" => {
                        // Remove status conditions in rain
                        if pokemon.hp > 0 && pokemon.status != PokemonStatus::None {
                            if matches!(state.weather, Weather::Rain | Weather::HeavyRain) {
                                instructions.push(StateInstructions::new(100.0, vec![
                                    Instruction::RemoveStatus(RemoveStatusInstruction {
                target_position: position,
                previous_status: Some(PokemonStatus::None),
                previous_status_duration: Some(None),
            })
                                ]));
                            }
                        }
                    }
                    "dryskin" => {
                        // In rain: heal 1/8 HP, in sun: take 1/8 HP damage
                        if pokemon.hp > 0 {
                            match state.weather {
                                crate::core::instruction::Weather::Rain => {
                                    if pokemon.hp < pokemon.max_hp {
                                        let heal_amount = pokemon.max_hp / 8;
                                        instructions.push(StateInstructions::new(100.0, vec![
                                            Instruction::PositionHeal(PositionHealInstruction {
                    target_position: position,
                    heal_amount,
                    previous_hp: Some(0),
                })
                                        ]));
                                    }
                                }
                                crate::core::instruction::Weather::SUN => {
                                    let damage = pokemon.max_hp / 8;
                                    instructions.push(StateInstructions::new(100.0, vec![
                                        Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
            })
                                    ]));
                                }
                                _ => {}
                            }
                        }
                    }
                    "raindish" => {
                        // Heal 1/16 HP in rain
                        if pokemon.hp > 0 && pokemon.hp < pokemon.max_hp && state.weather == crate::core::instruction::Weather::Rain {
                            let heal_amount = pokemon.max_hp / 16;
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::PositionHeal(PositionHealInstruction {
                    target_position: position,
                    heal_amount,
                    previous_hp: Some(0),
                })
                            ]));
                        }
                    }
                    "solarpower" => {
                        // Take 1/8 HP damage in sun
                        if pokemon.hp > 0 && state.weather == crate::core::instruction::Weather::SUN {
                            let damage = pokemon.max_hp / 8;
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::PositionDamage(PositionDamageInstruction {
                target_position: position,
                damage_amount: damage,
                previous_hp: Some(0),
            })
                            ]));
                        }
                    }
                    // Forme change abilities
                    "hungerswitch" => {
                        // Morpeko alternates between Full Belly and Hangry forms
                        #[cfg(feature = "terastallization")]
                        let is_terastallized = pokemon.is_terastallized;
                        #[cfg(not(feature = "terastallization"))]
                        let is_terastallized = false;
                        
                        if pokemon.hp > 0 && !is_terastallized {
                            instructions.extend(process_hunger_switch_forme_change(state, generation, position, pokemon));
                        }
                    }
                    "shieldsdown" => {
                        // Minior changes form based on HP (50% threshold)
                        if pokemon.hp > 0 {
                            instructions.extend(process_shields_down_forme_change(state, generation, position, pokemon));
                        }
                    }
                    "schooling" => {
                        // Wishiwashi changes form based on HP (25% threshold)
                        if pokemon.hp > 0 {
                            instructions.extend(process_schooling_forme_change(state, generation, position, pokemon));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    instructions
}
// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Check if a Pokemon is grounded (affected by terrain and ground-based moves)
fn is_pokemon_grounded(pokemon: &crate::core::state::Pokemon) -> bool {
    // Flying types are not grounded
    let has_flying_type = pokemon.types.iter().any(|t| t.to_lowercase() == "flying");
    if has_flying_type {
        return false;
    }
    
    // Pokemon with Levitate ability are not grounded
    if pokemon.ability.to_lowercase().replace(" ", "") == "levitate" {
        return false;
    }
    
    // Pokemon holding Air Balloon are not grounded
    if let Some(item) = &pokemon.item {
        if item.to_lowercase().replace(" ", "") == "airballoon" {
            return false;
        }
    }
    
    // Check for Magnet Rise volatile status (makes Pokemon ungrounded for 5 turns)
    if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::MagnetRise) {
        return false;
    }
    
    // Check for Telekinesis volatile status (makes Pokemon ungrounded for 3 turns)
    if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::Telekinesis) {
        return false;
    }
    
    // Pokemon using Sky Drop (as the target) are not grounded
    if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::SkyDrop) {
        return false;
    }
    
    // All other Pokemon are grounded
    true
}

// =============================================================================
// NEW ABILITY EFFECT FUNCTIONS
// =============================================================================

/// Process Bad Dreams ability effect - damages sleeping opponents
fn process_bad_dreams_effect(
    state: &State,
    generation: &GenerationMechanics,
    user_position: BattlePosition,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Get the opposing side
    let opponent_side_ref = match user_position.side {
        crate::core::battle_format::SideReference::SideOne => crate::core::battle_format::SideReference::SideTwo,
        crate::core::battle_format::SideReference::SideTwo => crate::core::battle_format::SideReference::SideOne,
    };
    
    let opponent_side = state.get_side(opponent_side_ref);
    
    // Check all active Pokemon on the opposing side
    for slot in 0..state.format.active_pokemon_count() {
        if let Some(opponent_pokemon) = opponent_side.get_active_pokemon_at_slot(slot) {
            // Only affect sleeping Pokemon
            if opponent_pokemon.status == PokemonStatus::Sleep && opponent_pokemon.hp > 0 {
                let opponent_position = BattlePosition::new(opponent_side_ref, slot);
                let damage = (opponent_pokemon.max_hp / 8).min(opponent_pokemon.hp); // Cap at current HP
                
                if damage > 0 {
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::PositionDamage(PositionDamageInstruction {
                target_position: opponent_position,
                damage_amount: damage,
                previous_hp: Some(0),
            })
                    ]));
                }
            }
        }
    }
    
    instructions
}

/// Process Hunger Switch forme change for Morpeko
fn process_hunger_switch_forme_change(
    state: &State,
    generation: &GenerationMechanics,
    position: BattlePosition,
    pokemon: &crate::core::state::Pokemon,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Determine target forme based on current forme
    let target_forme = if pokemon.species.to_lowercase().contains("hangry") {
        "morpeko" // Switch to Full Belly form
    } else {
        "morpekohangry" // Switch to Hangry form
    };
    
    // Only change if it's actually different
    if pokemon.species.to_lowercase() != target_forme.to_lowercase() {
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::FormeChange(FormeChangeInstruction {
                target_position: position,
                new_forme: target_forme.to_string(),
                previous_forme: Some(pokemon.species.clone()),
            })
        ]));
    }
    
    instructions
}

/// Process Shields Down forme change for Minior
fn process_shields_down_forme_change(
    state: &State,
    generation: &GenerationMechanics,
    position: BattlePosition,
    pokemon: &crate::core::state::Pokemon,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    let hp_threshold = pokemon.max_hp / 2; // 50% HP threshold
    let is_currently_meteor = pokemon.species.to_lowercase().contains("meteor");
    
    if pokemon.hp > hp_threshold && !is_currently_meteor {
        // Above 50% HP: Change to Meteor form
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::FormeChange(FormeChangeInstruction {
                target_position: position,
                new_forme: "miniormeteor".to_string(),
                previous_forme: Some(pokemon.species.clone()),
            })
        ]));
    } else if pokemon.hp <= hp_threshold && is_currently_meteor {
        // At/below 50% HP: Change to Core form
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::FormeChange(FormeChangeInstruction {
                target_position: position,
                new_forme: "minior".to_string(),
                previous_forme: Some(pokemon.species.clone()),
            })
        ]));
    }
    
    instructions
}

/// Process Schooling forme change for Wishiwashi
fn process_schooling_forme_change(
    state: &State,
    generation: &GenerationMechanics,
    position: BattlePosition,
    pokemon: &crate::core::state::Pokemon,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    let hp_threshold = pokemon.max_hp / 4; // 25% HP threshold
    let is_currently_school = pokemon.species.to_lowercase().contains("school");
    
    if pokemon.hp > hp_threshold && !is_currently_school {
        // Above 25% HP: Change to School form
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::FormeChange(FormeChangeInstruction {
                target_position: position,
                new_forme: "wishiwashischool".to_string(),
                previous_forme: Some(pokemon.species.clone()),
            })
        ]));
    } else if pokemon.hp <= hp_threshold && is_currently_school {
        // At/below 25% HP: Change to Solo form
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::FormeChange(FormeChangeInstruction {
                target_position: position,
                new_forme: "wishiwashi".to_string(),
                previous_forme: Some(pokemon.species.clone()),
            })
        ]));
    }
    
    instructions
}

/// Calculate Salt Cure damage
fn calculate_salt_cure_damage(pokemon: &crate::core::state::Pokemon) -> i16 {
    // Check for Magic Guard ability (prevents all indirect damage)
    if pokemon.ability.to_lowercase().replace(" ", "") == "magicguard" {
        return 0;
    }
    
    // Check if Pokemon is Water or Steel type (takes 1/4 max HP damage)
    let is_water_or_steel = pokemon.types.iter().any(|t| {
        let t_lower = t.to_lowercase();
        t_lower == "water" || t_lower == "steel"
    });
    
    if is_water_or_steel {
        pokemon.max_hp / 4 // 1/4 max HP for Water/Steel types
    } else {
        pokemon.max_hp / 8 // 1/8 max HP for other types
    }
}

// =============================================================================
// BERRY EFFECT FUNCTIONS
// =============================================================================

/// Check if an item is a berry that activates at end of turn
fn is_end_of_turn_berry(item_name: &str) -> bool {
    matches!(item_name.to_lowercase().replace(" ", "").as_str(),
        "aguavberry" | "apicotberry" | "figyberry" | "ganlonberry" | "iapapaberry" |
        "lansatberry" | "leppaberry" | "liechiberry" | "magoberry" | "petayaberry" |
        "salacberry" | "starfberry" | "wikiberry" | "custapberry" | "enigmaberry" |
        "jabocaberry" | "keeberry" | "marangaberry" | "micleberry" | "rowapberry"
    )
}

/// Process berry end-of-turn effects
fn process_berry_end_of_turn_effect(
    state: &State,
    generation: &GenerationMechanics,
    position: BattlePosition,
    pokemon: &crate::core::state::Pokemon,
    berry_name: &str,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    match berry_name.to_lowercase().replace(" ", "").as_str() {
        // Pinch healing berries (activate at 25% HP for most, 50% for Gluttony)
        "figyberry" | "wikiberry" | "magoberry" | "aguavberry" | "iapapaberry" => {
            let threshold = if pokemon.ability.to_lowercase().replace(" ", "") == "gluttony" {
                pokemon.max_hp / 2 // 50% with Gluttony
            } else {
                pokemon.max_hp / 4 // 25% normally
            };
            
            if pokemon.hp <= threshold && pokemon.hp > 0 {
                let heal_amount = pokemon.max_hp / 3; // Heal 1/3 max HP
                let mut instruction_list = Vec::new();
                
                instruction_list.push(Instruction::PositionHeal(PositionHealInstruction {
                target_position: position,
                heal_amount: heal_amount.min(pokemon.max_hp - pokemon.hp),
                previous_hp: Some(0),
            }));
                
                // Remove the berry after use
                instruction_list.push(Instruction::ChangeItem(ChangeItemInstruction {
                    target_position: position,
                    new_item: None,
                    previous_item: pokemon.item.clone(),
                }));
                
                // Check for confusion from wrong nature berry (simplified implementation)
                let confusing_berries = ["figyberry", "wikiberry", "magoberry", "aguavberry", "iapapaberry"];
                if confusing_berries.contains(&berry_name.to_lowercase().replace(" ", "").as_str()) {
                    // For simplicity, assume it always causes confusion (should check nature compatibility)
                    instruction_list.push(Instruction::ApplyVolatileStatus(ApplyVolatileStatusInstruction {
                        target_position: position,
                        volatile_status: crate::core::instruction::VolatileStatus::Confusion,
                        duration: Some(2 + (rand::random::<u8>() % 3)), // 2-4 turns
                    }));
                }
                
                instructions.push(StateInstructions::new(100.0, instruction_list));
            }
        }
        
        // Stat-boosting berries (activate at 25% HP for most, 50% for Gluttony)
        "liechiberry" | "ganlonberry" | "salacberry" | "petayaberry" | "apicotberry" => {
            let threshold = if pokemon.ability.to_lowercase().replace(" ", "") == "gluttony" {
                pokemon.max_hp / 2 // 50% with Gluttony
            } else {
                pokemon.max_hp / 4 // 25% normally
            };
            
            if pokemon.hp <= threshold && pokemon.hp > 0 {
                let mut instruction_list = Vec::new();
                let mut stat_boosts = HashMap::new();
                
                match berry_name.to_lowercase().replace(" ", "").as_str() {
                    "liechiberry" => { stat_boosts.insert(crate::core::instruction::Stat::Attack, 1); }
                    "ganlonberry" => { stat_boosts.insert(crate::core::instruction::Stat::Defense, 1); }
                    "salacberry" => { stat_boosts.insert(crate::core::instruction::Stat::Speed, 1); }
                    "petayaberry" => { stat_boosts.insert(crate::core::instruction::Stat::SpecialAttack, 1); }
                    "apicotberry" => { stat_boosts.insert(crate::core::instruction::Stat::SpecialDefense, 1); }
                    _ => {}
                }
                
                instruction_list.push(Instruction::BoostStats(crate::core::instruction::BoostStatsInstruction {
                    target_position: position,
                    stat_boosts,
                    previous_boosts: Some(HashMap::new()),
                }));
                
                // Remove the berry after use
                instruction_list.push(Instruction::ChangeItem(ChangeItemInstruction {
                    target_position: position,
                    new_item: None,
                    previous_item: pokemon.item.clone(),
                }));
                
                instructions.push(StateInstructions::new(100.0, instruction_list));
            }
        }
        
        // Leppa Berry (restores PP when a move reaches 0 PP)
        "leppaberry" => {
            // Check if any move has 0 PP
            let has_zero_pp_move = pokemon.moves.values().any(|m| m.pp == 0);
            
            if has_zero_pp_move && pokemon.hp > 0 {
                let mut instruction_list = Vec::new();
                
                // Find first move with 0 PP and restore 10 PP
                if let Some((move_index, _)) = pokemon.moves.iter().find(|(_, m)| m.pp == 0) {
                    instruction_list.push(Instruction::DecrementPP(DecrementPPInstruction {
                        target_position: position,
                        move_index: move_index.to_index() as u8,
                        amount: 10, // Amount to restore (note: actual restoration logic may be different)
                    }));
                }
                
                // Remove the berry after use
                instruction_list.push(Instruction::ChangeItem(ChangeItemInstruction {
                    target_position: position,
                    new_item: None,
                    previous_item: pokemon.item.clone(),
                }));
                
                instructions.push(StateInstructions::new(100.0, instruction_list));
            }
        }
        
        _ => {
            // Other berries can be implemented here as needed
        }
    }
    
    instructions
}