//! # Switch Effects Processing
//! 
//! This module handles all effects that occur when Pokemon switch in or out of battle.
//! This includes entry hazards (Spikes, Stealth Rock, etc.), abilities that trigger on switch,
//! and other switch-related mechanics.
//!
//! Following poke-engine's pattern, switch effects are processed in a specific order
//! to match official game mechanics.

use crate::core::battle_format::{BattlePosition, SideReference};
use crate::core::instruction::{
    Instruction, StateInstructions, PositionDamageInstruction, PositionHealInstruction,
    ApplyStatusInstruction, ApplyVolatileStatusInstruction, BoostStatsInstruction,
    SideCondition, PokemonStatus, VolatileStatus, Stat, RemoveVolatileStatusInstruction
};
use crate::core::state::{State, Pokemon};
use crate::generation::GenerationMechanics;
use crate::engine::combat::damage_calc::is_grounded;
use std::collections::HashMap;

/// Process all switch-in effects for a Pokemon entering battle
/// This handles entry hazards, abilities, and other switch-in effects
pub fn process_switch_in_effects(
    state: &State,
    switching_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Process effects in the correct order according to game mechanics
    
    // 1. Entry hazards (Spikes, Stealth Rock, Toxic Spikes, Sticky Web)
    instructions.extend(process_entry_hazards(state, switching_position, generation));
    
    // 2. Switch-in abilities
    instructions.extend(process_switch_in_abilities(state, switching_position, generation));
    
    // 3. Items that activate on switch-in
    instructions.extend(process_switch_in_items(state, switching_position, generation));
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Process all switch-out effects for a Pokemon leaving battle
/// This handles abilities and items that trigger on switch-out
pub fn process_switch_out_effects(
    state: &State,
    switching_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // 1. Switch-out abilities
    instructions.extend(process_switch_out_abilities(state, switching_position, generation));
    
    // 2. Items that activate on switch-out
    instructions.extend(process_switch_out_items(state, switching_position, generation));
    
    // 3. Cleanup volatile statuses that don't persist
    instructions.extend(process_switch_out_volatile_cleanup(state, switching_position, generation));
    
    if instructions.is_empty() {
        instructions.push(StateInstructions::empty());
    }
    
    instructions
}

/// Process entry hazards when a Pokemon switches in
fn process_entry_hazards(
    state: &State,
    switching_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    let side = state.get_side(switching_position.side);
    let pokemon = match side.get_active_pokemon_at_slot(switching_position.slot) {
        Some(pokemon) => pokemon,
        None => return instructions,
    };
    
    // Process each type of entry hazard
    
    // Spikes
    if let Some(&spikes_layers) = side.side_conditions.get(&SideCondition::Spikes) {
        if spikes_layers > 0 && is_grounded(pokemon) {
            let damage = calculate_spikes_damage(pokemon, spikes_layers as i8);
            if damage > 0 {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::PositionDamage(PositionDamageInstruction {
                target_position: switching_position,
                damage_amount: damage,
                previous_hp: Some(0),
            })
                ]));
            }
        }
    }
    
    // Stealth Rock
    if let Some(&stealth_rock) = side.side_conditions.get(&SideCondition::StealthRock) {
        if stealth_rock > 0 {
            let damage = calculate_stealth_rock_damage(state, pokemon, generation);
            if damage > 0 {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::PositionDamage(PositionDamageInstruction {
                target_position: switching_position,
                damage_amount: damage,
                previous_hp: Some(0),
            })
                ]));
            }
        }
    }
    
    // Toxic Spikes
    if let Some(&toxic_spikes_layers) = side.side_conditions.get(&SideCondition::ToxicSpikes) {
        if toxic_spikes_layers > 0 && is_grounded(pokemon) {
            let effect = get_toxic_spikes_effect(pokemon, toxic_spikes_layers as i8, generation);
            if let Some(status) = effect {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyStatus(ApplyStatusInstruction {
                        target_position: switching_position,
                        status,
                        previous_status: Some(PokemonStatus::None),
                        previous_status_duration: Some(None),
                    })
                ]));
            }
        }
    }
    
    // Sticky Web
    if let Some(&sticky_web) = side.side_conditions.get(&SideCondition::StickyWeb) {
        if sticky_web > 0 && is_grounded(pokemon) {
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(Stat::Speed, -1);
            
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::BoostStats(BoostStatsInstruction {
                    target_position: switching_position,
                    stat_boosts,
                    previous_boosts: Some(HashMap::new()),
                })
            ]));
        }
    }
    
    instructions
}

/// Process switch-in abilities
fn process_switch_in_abilities(
    state: &State,
    switching_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    let side = state.get_side(switching_position.side);
    let pokemon = match side.get_active_pokemon_at_slot(switching_position.slot) {
        Some(pokemon) => pokemon,
        None => return instructions,
    };
    
    match pokemon.ability.to_lowercase().as_str() {
        // Weather-setting abilities
        "drought" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeWeather(crate::core::instruction::ChangeWeatherInstruction {
                    weather: crate::core::instruction::Weather::SUN,
                    duration: Some(5),
                    previous_weather: Some(state.weather),
                    previous_duration: Some(state.weather_turns_remaining),
                })
            ]));
        }
        "drizzle" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeWeather(crate::core::instruction::ChangeWeatherInstruction {
                    weather: crate::core::instruction::Weather::Rain,
                    duration: Some(5),
                    previous_weather: Some(state.weather),
                    previous_duration: Some(state.weather_turns_remaining),
                })
            ]));
        }
        "sand stream" | "sandstream" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeWeather(crate::core::instruction::ChangeWeatherInstruction {
                    weather: crate::core::instruction::Weather::SAND,
                    duration: Some(5),
                    previous_weather: Some(state.weather),
                    previous_duration: Some(state.weather_turns_remaining),
                })
            ]));
        }
        "snow warning" | "snowwarning" => {
            let weather = if generation.generation.number() >= 9 {
                crate::core::instruction::Weather::SNOW
            } else {
                crate::core::instruction::Weather::HAIL
            };
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeWeather(crate::core::instruction::ChangeWeatherInstruction {
                    weather,
                    duration: Some(5),
                    previous_weather: Some(state.weather),
                    previous_duration: Some(state.weather_turns_remaining),
                })
            ]));
        }
        
        // Terrain-setting abilities
        "electric surge" | "electricsurge" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeTerrain(crate::core::instruction::ChangeTerrainInstruction {
                    terrain: crate::core::instruction::Terrain::ELECTRICTERRAIN,
                    duration: Some(5),
                    previous_terrain: Some(state.terrain),
                    previous_duration: Some(state.terrain_turns_remaining),
                })
            ]));
        }
        "grassy surge" | "grassysurge" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeTerrain(crate::core::instruction::ChangeTerrainInstruction {
                    terrain: crate::core::instruction::Terrain::GRASSYTERRAIN,
                    duration: Some(5),
                    previous_terrain: Some(state.terrain),
                    previous_duration: Some(state.terrain_turns_remaining),
                })
            ]));
        }
        "misty surge" | "mistysurge" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeTerrain(crate::core::instruction::ChangeTerrainInstruction {
                    terrain: crate::core::instruction::Terrain::MISTYTERRAIN,
                    duration: Some(5),
                    previous_terrain: Some(state.terrain),
                    previous_duration: Some(state.terrain_turns_remaining),
                })
            ]));
        }
        "psychic surge" | "psychicsurge" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeTerrain(crate::core::instruction::ChangeTerrainInstruction {
                    terrain: crate::core::instruction::Terrain::PSYCHICTERRAIN,
                    duration: Some(5),
                    previous_terrain: Some(state.terrain),
                    previous_duration: Some(state.terrain_turns_remaining),
                })
            ]));
        }
        
        // Intimidate
        "intimidate" => {
            instructions.extend(apply_intimidate_effect(state, switching_position, generation));
        }
        
        // Download
        "download" => {
            instructions.extend(apply_download_effect(state, switching_position, generation));
        }
        
        // Trace
        "trace" => {
            instructions.extend(apply_trace_effect(state, switching_position, generation));
        }
        
        _ => {}
    }
    
    instructions
}

/// Apply Intimidate ability effect
fn apply_intimidate_effect(
    state: &State,
    user_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Intimidate lowers Attack of all opposing Pokemon
    let opposing_side = user_position.side.opposite();
    let opposing_side_data = state.get_side(opposing_side);
    
    for slot in 0..state.format.active_pokemon_count() {
        if let Some(opponent) = opposing_side_data.get_active_pokemon_at_slot(slot) {
            // Check for immunities (Clear Body, Hyper Cutter, etc.)
            if !is_immune_to_intimidate(opponent, generation) {
                let mut stat_boosts = HashMap::new();
                stat_boosts.insert(Stat::Attack, -1);
                
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::BoostStats(BoostStatsInstruction {
                        target_position: BattlePosition::new(opposing_side, slot),
                        stat_boosts,
                        previous_boosts: Some(HashMap::new()),
                    })
                ]));
            }
        }
    }
    
    instructions
}

/// Apply Download ability effect
fn apply_download_effect(
    state: &State,
    user_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Download compares opponent's Defense vs Special Defense and boosts accordingly
    let opposing_side = user_position.side.opposite();
    let opposing_side_data = state.get_side(opposing_side);
    
    let mut total_defense = 0;
    let mut total_special_defense = 0;
    let mut opponent_count = 0;
    
    for slot in 0..state.format.active_pokemon_count() {
        if let Some(opponent) = opposing_side_data.get_active_pokemon_at_slot(slot) {
            total_defense += opponent.get_effective_stat(Stat::Defense);
            total_special_defense += opponent.get_effective_stat(Stat::SpecialDefense);
            opponent_count += 1;
        }
    }
    
    if opponent_count > 0 {
        let avg_defense = total_defense / opponent_count;
        let avg_special_defense = total_special_defense / opponent_count;
        
        let mut stat_boosts = HashMap::new();
        if avg_defense < avg_special_defense {
            // Boost Attack if Defense is lower
            stat_boosts.insert(Stat::Attack, 1);
        } else {
            // Boost Special Attack if Special Defense is lower or equal
            stat_boosts.insert(Stat::SpecialAttack, 1);
        }
        
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::BoostStats(BoostStatsInstruction {
                target_position: user_position,
                stat_boosts,
                previous_boosts: Some(HashMap::new()),
            })
        ]));
    }
    
    instructions
}

/// Apply Trace ability effect
fn apply_trace_effect(
    state: &State,
    user_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Trace copies the ability of a random opponent
    let opposing_side = user_position.side.opposite();
    let opposing_side_data = state.get_side(opposing_side);
    
    let mut traceable_abilities = Vec::new();
    
    for slot in 0..state.format.active_pokemon_count() {
        if let Some(opponent) = opposing_side_data.get_active_pokemon_at_slot(slot) {
            if is_ability_traceable(&opponent.ability, generation) {
                traceable_abilities.push(opponent.ability.clone());
            }
        }
    }
    
    if !traceable_abilities.is_empty() {
        // For now, trace the first available ability
        // In a full implementation, this would be random
        let new_ability = &traceable_abilities[0];
        
        instructions.push(StateInstructions::new(100.0, vec![
            Instruction::ChangeAbility(crate::core::instruction::ChangeAbilityInstruction {
                target_position: user_position,
                new_ability: new_ability.clone(),
                previous_ability: None,
            })
        ]));
    }
    
    instructions
}

/// Process switch-in items
fn process_switch_in_items(
    state: &State,
    switching_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // TODO: Implement item effects on switch-in
    // Examples: Air Balloon, Choice items, etc.
    
    instructions
}

/// Process switch-out abilities
fn process_switch_out_abilities(
    state: &State,
    switching_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // TODO: Implement abilities that trigger on switch-out
    // Examples: Regenerator, Natural Cure, etc.
    
    instructions
}

/// Process switch-out items
fn process_switch_out_items(
    state: &State,
    switching_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // TODO: Implement item effects on switch-out
    
    instructions
}

/// Clean up volatile statuses that don't persist through switching
fn process_switch_out_volatile_cleanup(
    state: &State,
    switching_position: BattlePosition,
    generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(pokemon) = state.get_pokemon_at_position(switching_position) {
        let mut instruction_list = Vec::new();
        
        // Most volatile statuses are cleared when switching out
        // Some statuses persist through switching (e.g., Substitute in some contexts)
        for volatile_status in &pokemon.volatile_statuses {
            match volatile_status {
                // Statuses that persist through switching
                crate::core::instruction::VolatileStatus::Substitute => {
                    // Substitute is usually lost when switching, but there are exceptions
                    // For now, remove it on switch
                    instruction_list.push(Instruction::RemoveVolatileStatus(
                        RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: *volatile_status,
                        }
                    ));
                }
                
                // Statuses that are always cleared on switch
                crate::core::instruction::VolatileStatus::Confusion |
                crate::core::instruction::VolatileStatus::Flinch |
                crate::core::instruction::VolatileStatus::Attract |
                crate::core::instruction::VolatileStatus::Torment |
                crate::core::instruction::VolatileStatus::Disable |
                crate::core::instruction::VolatileStatus::Encore |
                crate::core::instruction::VolatileStatus::Taunt |
                crate::core::instruction::VolatileStatus::HelpingHand |
                crate::core::instruction::VolatileStatus::MagicCoat |
                crate::core::instruction::VolatileStatus::FollowMe |
                crate::core::instruction::VolatileStatus::Protect |
                crate::core::instruction::VolatileStatus::Endure |
                crate::core::instruction::VolatileStatus::FocusEnergy |
                crate::core::instruction::VolatileStatus::LaserFocus |
                crate::core::instruction::VolatileStatus::Rage |
                crate::core::instruction::VolatileStatus::Charge |
                crate::core::instruction::VolatileStatus::DefenseCurl |
                crate::core::instruction::VolatileStatus::Stockpile |
                crate::core::instruction::VolatileStatus::PowerTrick |
                crate::core::instruction::VolatileStatus::Electrify |
                crate::core::instruction::VolatileStatus::Embargo |
                crate::core::instruction::VolatileStatus::GastroAcid |
                crate::core::instruction::VolatileStatus::Foresight |
                crate::core::instruction::VolatileStatus::MiracleEye => {
                    instruction_list.push(Instruction::RemoveVolatileStatus(
                        RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: *volatile_status,
                        }
                    ));
                }
                
                // Some statuses persist through switching
                crate::core::instruction::VolatileStatus::LeechSeed |
                crate::core::instruction::VolatileStatus::Curse |
                crate::core::instruction::VolatileStatus::Nightmare |
                crate::core::instruction::VolatileStatus::AquaRing |
                crate::core::instruction::VolatileStatus::Ingrain => {
                    // These typically persist through switching in most circumstances
                    // Some may have special rules based on generation or other factors
                    // For now, keep them
                }
                
                // Position-based statuses that are removed on switch
                crate::core::instruction::VolatileStatus::MagnetRise |
                crate::core::instruction::VolatileStatus::Telekinesis |
                crate::core::instruction::VolatileStatus::Roost => {
                    instruction_list.push(Instruction::RemoveVolatileStatus(
                        RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: *volatile_status,
                        }
                    ));
                }
                
                // Two-turn moves that are interrupted by switching
                crate::core::instruction::VolatileStatus::Fly |
                crate::core::instruction::VolatileStatus::Dig |
                crate::core::instruction::VolatileStatus::Dive |
                crate::core::instruction::VolatileStatus::Bounce |
                crate::core::instruction::VolatileStatus::SkyDrop |
                crate::core::instruction::VolatileStatus::FreezeeShock |
                crate::core::instruction::VolatileStatus::IceBurn |
                crate::core::instruction::VolatileStatus::Geomancy |
                crate::core::instruction::VolatileStatus::Electroshot => {
                    instruction_list.push(Instruction::RemoveVolatileStatus(
                        RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: *volatile_status,
                        }
                    ));
                }
                
                // Default case - remove most other volatile statuses on switch
                _ => {
                    instruction_list.push(Instruction::RemoveVolatileStatus(
                        RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: *volatile_status,
                        }
                    ));
                }
            }
        }
        
        if !instruction_list.is_empty() {
            instructions.push(StateInstructions::new(100.0, instruction_list));
        }
    }
    
    instructions
}

// =============================================================================
// DAMAGE CALCULATION FUNCTIONS
// =============================================================================

/// Calculate Spikes damage based on layers
fn calculate_spikes_damage(pokemon: &Pokemon, layers: i8) -> i16 {
    match layers {
        1 => pokemon.max_hp / 8,  // 1/8 max HP
        2 => pokemon.max_hp / 6,  // 1/6 max HP
        3 => pokemon.max_hp / 4,  // 1/4 max HP
        _ => 0,
    }
}

/// Calculate Stealth Rock damage based on type effectiveness
fn calculate_stealth_rock_damage(
    state: &State,
    pokemon: &Pokemon,
    generation: &GenerationMechanics,
) -> i16 {
    use crate::engine::combat::type_effectiveness::{PokemonType, TypeChart};
    
    let type_chart = TypeChart::new(generation.generation.number());
    let rock_type = PokemonType::Rock;
    
    let pokemon_type1 = PokemonType::from_str(&pokemon.types[0]).unwrap_or(PokemonType::Normal);
    let pokemon_type2 = if pokemon.types.len() > 1 {
        PokemonType::from_str(&pokemon.types[1]).unwrap_or(pokemon_type1)
    } else {
        pokemon_type1
    };
    
    let effectiveness = type_chart.calculate_damage_multiplier(
        rock_type,
        (pokemon_type1, pokemon_type2),
        None,
        None,
    );
    
    // Base damage is 1/8 max HP, modified by type effectiveness
    let base_damage = pokemon.max_hp / 8;
    (base_damage as f32 * effectiveness) as i16
}

/// Get Toxic Spikes effect based on layers and Pokemon type
fn get_toxic_spikes_effect(
    pokemon: &Pokemon,
    layers: i8,
    generation: &GenerationMechanics,
) -> Option<PokemonStatus> {
    // Poison types absorb Toxic Spikes
    if pokemon.types.iter().any(|t| t.to_lowercase() == "poison") {
        return None; // Absorbed, no effect
    }
    
    // Steel types are immune to poison
    if generation.generation.number() >= 2 {
        if pokemon.types.iter().any(|t| t.to_lowercase() == "steel") {
            return None;
        }
    }
    
    match layers {
        1 => Some(PokemonStatus::Poison),      // Regular poison
        2 => Some(PokemonStatus::Toxic),       // Badly poisoned
        _ => None,
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Check if a Pokemon is immune to Intimidate
fn is_immune_to_intimidate(pokemon: &Pokemon, generation: &GenerationMechanics) -> bool {
    match pokemon.ability.to_lowercase().as_str() {
        "clear body" | "hyper cutter" | "white smoke" | "full metal body" | 
        "inner focus" | "oblivious" | "own tempo" | "scrappy" => true,
        _ => false,
    }
}

/// Check if an ability can be traced
fn is_ability_traceable(ability: &str, generation: &GenerationMechanics) -> bool {
    // Some abilities cannot be traced
    match ability.to_lowercase().as_str() {
        "trace" | "forecast" | "flower gift" | "illusion" | "imposter" |
        "multitype" | "zen mode" | "stance change" | "power construct" |
        "schooling" | "comatose" | "shields down" | "disguise" |
        "rks system" | "battle bond" | "power of alchemy" | "receiver" |
        "wonder guard" | "air lock" | "cloud nine" => false,
        _ => true,
    }
}