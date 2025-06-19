//! # Switch Effects Processing
//! 
//! This module handles all effects that occur when Pokemon switch in or out of battle.
//! This includes entry hazards (Spikes, Stealth Rock, etc.), abilities that trigger on switch,
//! and other switch-related mechanics.
//!
//! Following poke-engine's pattern, switch effects are processed in a specific order
//! to match official game mechanics.

use crate::core::battle_format::BattlePosition;
use crate::core::instruction::{
    Instruction, StateInstructions, PositionDamageInstruction,
    ApplyStatusInstruction, BoostStatsInstruction,
    SideCondition, PokemonStatus, Stat, RemoveVolatileStatusInstruction,
    ChangeItemInstruction
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
                    weather: crate::core::instruction::Weather::Sun,
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
                    weather: crate::core::instruction::Weather::Sand,
                    duration: Some(5),
                    previous_weather: Some(state.weather),
                    previous_duration: Some(state.weather_turns_remaining),
                })
            ]));
        }
        "snow warning" | "snowwarning" => {
            let weather = if generation.generation.number() >= 9 {
                crate::core::instruction::Weather::Snow
            } else {
                crate::core::instruction::Weather::Hail
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
                    terrain: crate::core::instruction::Terrain::ElectricTerrain,
                    duration: Some(5),
                    previous_terrain: Some(state.terrain),
                    previous_duration: Some(state.terrain_turns_remaining),
                })
            ]));
        }
        "grassy surge" | "grassysurge" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeTerrain(crate::core::instruction::ChangeTerrainInstruction {
                    terrain: crate::core::instruction::Terrain::GrassyTerrain,
                    duration: Some(5),
                    previous_terrain: Some(state.terrain),
                    previous_duration: Some(state.terrain_turns_remaining),
                })
            ]));
        }
        "misty surge" | "mistysurge" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeTerrain(crate::core::instruction::ChangeTerrainInstruction {
                    terrain: crate::core::instruction::Terrain::MistyTerrain,
                    duration: Some(5),
                    previous_terrain: Some(state.terrain),
                    previous_duration: Some(state.terrain_turns_remaining),
                })
            ]));
        }
        "psychic surge" | "psychicsurge" => {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ChangeTerrain(crate::core::instruction::ChangeTerrainInstruction {
                    terrain: crate::core::instruction::Terrain::PsychicTerrain,
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
        
        // Gen 9 Legendary Abilities (Zacian/Zamazenta)
        "intrepid sword" | "intrepidsword" => {
            instructions.extend(apply_intrepid_sword_effect(state, switching_position, generation));
        }
        "dauntless shield" | "dauntlessshield" => {
            instructions.extend(apply_dauntless_shield_effect(state, switching_position, generation));
        }
        
        // Paradox Pokemon Abilities
        "protosynthesis" => {
            instructions.extend(apply_protosynthesis_effect(state, switching_position, generation));
        }
        "quark drive" | "quarkdrive" => {
            instructions.extend(apply_quark_drive_effect(state, switching_position, generation));
        }
        
        // Ogerpon Embody Aspect Abilities
        "embody aspect" | "embodyaspect" => {
            instructions.extend(apply_embody_aspect_effect(state, switching_position, generation));
        }
        
        // Utility Abilities
        "screen cleaner" | "screencleaner" => {
            instructions.extend(apply_screen_cleaner_effect(state, switching_position, generation));
        }
        "slow start" | "slowstart" => {
            instructions.extend(apply_slow_start_effect(state, switching_position, generation));
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
    
    let side = state.get_side(switching_position.side);
    let pokemon = match side.get_active_pokemon_at_slot(switching_position.slot) {
        Some(pokemon) => pokemon,
        None => return instructions,
    };
    
    if let Some(item) = &pokemon.item {
        match item.to_lowercase().as_str() {
            // Air Balloon - Provides Ground immunity
            "air balloon" | "airballoon" => {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::AirBalloon,
                        duration: None, // Lasts until popped by damage
                    })
                ]));
            }
            
            // Choice items - Lock into first move used
            "choice band" | "choiceband" => {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::ChoiceLock,
                        duration: None, // Lasts until switch out
                    })
                ]));
            }
            
            "choice scarf" | "choicescarf" => {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::ChoiceLock,
                        duration: None, // Lasts until switch out
                    })
                ]));
            }
            
            "choice specs" | "choicespecs" => {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::ChoiceLock,
                        duration: None, // Lasts until switch out
                    })
                ]));
            }
            
            // Iron Ball - Makes Pokemon grounded
            "iron ball" | "ironball" => {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::IronBall,
                        duration: None, // Lasts while holding item
                    })
                ]));
            }
            
            // Toxic Orb - Badly poisons holder at end of turn
            "toxic orb" | "toxicorb" => {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::ToxicOrb,
                        duration: Some(1), // Activates at end of first turn
                    })
                ]));
            }
            
            // Flame Orb - Burns holder at end of turn
            "flame orb" | "flameorb" => {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::FlameOrb,
                        duration: Some(1), // Activates at end of first turn
                    })
                ]));
            }
            
            // Mental Herb - Removes attraction and choice lock (one time use)
            "mental herb" | "mentalherb" => {
                // Mental Herb is reactive - it activates when needed
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::MentalHerb,
                        duration: None, // Lasts until consumed
                    })
                ]));
            }
            
            // Power Herb - Allows immediate use of charge moves (one time use)
            "power herb" | "powerherb" => {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                        target_position: switching_position,
                        volatile_status: crate::core::instruction::VolatileStatus::PowerHerb,
                        duration: None, // Lasts until consumed
                    })
                ]));
            }
            
            // Room Service - Lowers Speed when Trick Room is active
            "room service" | "roomservice" => {
                if state.trick_room_active {
                    let mut stat_boosts = HashMap::new();
                    stat_boosts.insert(Stat::Speed, -1);
                    
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::BoostStats(BoostStatsInstruction {
                            target_position: switching_position,
                            stat_boosts,
                            previous_boosts: Some(HashMap::new()),
                        }),
                        // Remove the item after use
                        Instruction::ChangeItem(ChangeItemInstruction {
                            target_position: switching_position,
                            new_item: None,
                            previous_item: Some(item.clone()),
                        })
                    ]));
                }
            }
            
            // Booster Energy - Activates Protosynthesis/Quark Drive
            "booster energy" | "boosterenergy" => {
                let user_side = state.get_side(switching_position.side);
                if let Some(pokemon) = user_side.get_active_pokemon_at_slot(switching_position.slot) {
                    match pokemon.ability.to_lowercase().as_str() {
                        "protosynthesis" => {
                            let highest_stat = calculate_highest_stat_excluding_hp(pokemon);
                            let mut stat_boosts = HashMap::new();
                            stat_boosts.insert(highest_stat, 1);
                            
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::BoostStats(BoostStatsInstruction {
                                    target_position: switching_position,
                                    stat_boosts,
                                    previous_boosts: Some(HashMap::new()),
                                }),
                                // Remove the item after use
                                Instruction::ChangeItem(ChangeItemInstruction {
                                    target_position: switching_position,
                                    new_item: None,
                                    previous_item: Some(item.clone()),
                                })
                            ]));
                        }
                        "quark drive" | "quarkdrive" => {
                            let highest_stat = calculate_highest_stat_excluding_hp(pokemon);
                            let mut stat_boosts = HashMap::new();
                            stat_boosts.insert(highest_stat, 1);
                            
                            instructions.push(StateInstructions::new(100.0, vec![
                                Instruction::BoostStats(BoostStatsInstruction {
                                    target_position: switching_position,
                                    stat_boosts,
                                    previous_boosts: Some(HashMap::new()),
                                }),
                                // Remove the item after use
                                Instruction::ChangeItem(ChangeItemInstruction {
                                    target_position: switching_position,
                                    new_item: None,
                                    previous_item: Some(item.clone()),
                                })
                            ]));
                        }
                        _ => {}
                    }
                }
            }
            
            _ => {}
        }
    }
    
    instructions
}

/// Process switch-out abilities
fn process_switch_out_abilities(
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
        "natural cure" | "naturalcure" => {
            // Remove status conditions when switching out
            if pokemon.status != crate::core::instruction::PokemonStatus::None {
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::RemoveStatus(crate::core::instruction::RemoveStatusInstruction {
                        target_position: switching_position,
                        previous_status: Some(pokemon.status),
                        previous_status_duration: Some(pokemon.status_duration),
                    })
                ]));
            }
        }
        "regenerator" => {
            // Heal 1/3 HP when switching out
            if pokemon.hp < pokemon.max_hp && pokemon.hp > 0 {
                let heal_amount = pokemon.max_hp / 3;
                instructions.push(StateInstructions::new(100.0, vec![
                    Instruction::PositionHeal(crate::core::instruction::PositionHealInstruction {
                        target_position: switching_position,
                        heal_amount,
                        previous_hp: Some(0),
                    })
                ]));
            }
        }
        "zero to hero" | "zerotohero" => {
            // Palafin forme change when switching out
            instructions.extend(apply_zero_to_hero_forme_change(state, switching_position, generation));
        }
        "gulp missile" | "gulpmissile" => {
            // Cramorant forme change back to base form
            instructions.extend(apply_gulp_missile_switch_out(state, switching_position, generation));
        }
        "hunger switch" | "hungerswitch" => {
            // Morpeko forme change when switching out (alternate forme)
            instructions.extend(apply_hunger_switch_switch_out(state, switching_position, generation));
        }
        _ => {}
    }
    
    instructions
}

/// Process switch-out items
fn process_switch_out_items(
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
    
    if let Some(item) = &pokemon.item {
        match item.to_lowercase().as_str() {
            // Red Card - Forces attacker to switch out (consumed on use)
            "red card" | "redcard" => {
                // Red Card is reactive and consumed when hit
                // No switch-out effect needed
            }
            
            // Eject Button - Forces user to switch out when hit (consumed on use)
            "eject button" | "ejectbutton" => {
                // Eject Button is reactive and consumed when hit
                // No switch-out effect needed
            }
            
            // Emergency Exit / Wimp Out related items would be handled here
            // but those are ability-based
            
            // White Herb - Removes negative stat changes (consumed on use)
            "white herb" | "whiteherb" => {
                // Check if Pokemon has any negative stat changes to remove
                let mut has_negative_boosts = false;
                let mut stat_changes = HashMap::new();
                
                for (stat, &boost) in &pokemon.stat_boosts {
                    if boost < 0 {
                        has_negative_boosts = true;
                        stat_changes.insert(*stat, 0 - boost); // Reset to 0
                    }
                }
                
                if has_negative_boosts {
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::BoostStats(BoostStatsInstruction {
                            target_position: switching_position,
                            stat_boosts: stat_changes,
                            previous_boosts: Some(pokemon.stat_boosts.clone()),
                        }),
                        // Remove the item after use
                        Instruction::ChangeItem(ChangeItemInstruction {
                            target_position: switching_position,
                            new_item: None,
                            previous_item: Some(item.clone()),
                        })
                    ]));
                }
            }
            
            // Shell Bell - Heals based on damage dealt (no switch-out effect)
            "shell bell" | "shellbell" => {
                // No switch-out effect for Shell Bell
            }
            
            // Shed Shell - Allows switching even when trapped
            "shed shell" | "shedshell" => {
                // Remove any trapping effects when switching out
                let mut remove_instructions = Vec::new();
                
                for volatile_status in &pokemon.volatile_statuses {
                    match volatile_status {
                        crate::core::instruction::VolatileStatus::Trap |
                        crate::core::instruction::VolatileStatus::PartialTrap |
                        crate::core::instruction::VolatileStatus::MeanLook |
                        crate::core::instruction::VolatileStatus::SpiderWeb |
                        crate::core::instruction::VolatileStatus::Block |
                        crate::core::instruction::VolatileStatus::SkyDrop => {
                            remove_instructions.push(Instruction::RemoveVolatileStatus(
                                RemoveVolatileStatusInstruction {
                                    target_position: switching_position,
                                    volatile_status: *volatile_status,
                                }
                            ));
                        }
                        _ => {}
                    }
                }
                
                if !remove_instructions.is_empty() {
                    instructions.push(StateInstructions::new(100.0, remove_instructions));
                }
            }
            
            // Grip Claw / Binding Band extend trapping moves (no switch-out effect)
            "grip claw" | "gripclaw" | "binding band" | "bindingband" => {
                // No switch-out effect
            }
            
            // Toxic Orb / Flame Orb - remove their volatile status markers
            "toxic orb" | "toxicorb" => {
                // Remove the ToxicOrb volatile status
                if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::ToxicOrb) {
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: crate::core::instruction::VolatileStatus::ToxicOrb,
                        })
                    ]));
                }
            }
            
            "flame orb" | "flameorb" => {
                // Remove the FlameOrb volatile status
                if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::FlameOrb) {
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: crate::core::instruction::VolatileStatus::FlameOrb,
                        })
                    ]));
                }
            }
            
            // Air Balloon - remove when switching out
            "air balloon" | "airballoon" => {
                if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::AirBalloon) {
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: crate::core::instruction::VolatileStatus::AirBalloon,
                        })
                    ]));
                }
            }
            
            // Choice items - remove choice lock when switching out
            "choice band" | "choiceband" | "choice scarf" | "choicescarf" | "choice specs" | "choicespecs" => {
                if pokemon.volatile_statuses.contains(&crate::core::instruction::VolatileStatus::ChoiceLock) {
                    instructions.push(StateInstructions::new(100.0, vec![
                        Instruction::RemoveVolatileStatus(RemoveVolatileStatusInstruction {
                            target_position: switching_position,
                            volatile_status: crate::core::instruction::VolatileStatus::ChoiceLock,
                        })
                    ]));
                }
            }
            
            _ => {}
        }
    }
    
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

/// Apply Zero to Hero forme change for Palafin
fn apply_zero_to_hero_forme_change(
    _state: &State,
    switching_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Palafin changes to Hero form when switching out
    instructions.push(StateInstructions::new(100.0, vec![
        Instruction::FormeChange(crate::core::instruction::FormeChangeInstruction {
            target_position: switching_position,
            new_forme: "palafinhero".to_string(),
            previous_forme: None,
        })
    ]));
    
    instructions
}

/// Apply Gulp Missile switch-out effect for Cramorant  
fn apply_gulp_missile_switch_out(
    state: &State,
    switching_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    let side = state.get_side(switching_position.side);
    if let Some(pokemon) = side.get_active_pokemon_at_slot(switching_position.slot) {
        // Return to base form when switching out
        if pokemon.species.to_lowercase().contains("gulping") || pokemon.species.to_lowercase().contains("gorging") {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::FormeChange(crate::core::instruction::FormeChangeInstruction {
                    target_position: switching_position,
                    new_forme: "cramorant".to_string(),
                    previous_forme: Some(pokemon.species.clone()),
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Hunger Switch switch-out effect for Morpeko
fn apply_hunger_switch_switch_out(
    state: &State,
    switching_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    let side = state.get_side(switching_position.side);
    if let Some(pokemon) = side.get_active_pokemon_at_slot(switching_position.slot) {
        // Alternate forme when switching out
        let target_forme = if pokemon.species.to_lowercase().contains("hangry") {
            "morpeko" // Switch to Full Belly form
        } else {
            "morpekohangry" // Switch to Hangry form
        };
        
        if pokemon.species.to_lowercase() != target_forme.to_lowercase() {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::FormeChange(crate::core::instruction::FormeChangeInstruction {
                    target_position: switching_position,
                    new_forme: target_forme.to_string(),
                    previous_forme: Some(pokemon.species.clone()),
                })
            ]));
        }
    }
    
    instructions
}

// =============================================================================
// GEN 9 LEGENDARY ABILITIES
// =============================================================================

/// Apply Intrepid Sword ability effect (Zacian) - Boosts Attack by 1 on switch-in
fn apply_intrepid_sword_effect(
    _state: &State,
    user_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Attack, 1);
    
    instructions.push(StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: user_position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ]));
    
    instructions
}

/// Apply Dauntless Shield ability effect (Zamazenta) - Boosts Defense by 1 on switch-in
fn apply_dauntless_shield_effect(
    _state: &State,
    user_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    let mut stat_boosts = HashMap::new();
    stat_boosts.insert(Stat::Defense, 1);
    
    instructions.push(StateInstructions::new(100.0, vec![
        Instruction::BoostStats(BoostStatsInstruction {
            target_position: user_position,
            stat_boosts,
            previous_boosts: Some(HashMap::new()),
        })
    ]));
    
    instructions
}

// =============================================================================
// PARADOX POKEMON ABILITIES
// =============================================================================

/// Apply Protosynthesis ability effect - Boosts highest stat in Sun weather (or with Booster Energy)
fn apply_protosynthesis_effect(
    state: &State,
    user_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Check if conditions are met for Protosynthesis activation
    let should_activate = matches!(state.weather, crate::core::instruction::Weather::Sun | crate::core::instruction::Weather::HarshSun);
    
    if should_activate {
        let user_side = state.get_side(user_position.side);
        if let Some(pokemon) = user_side.get_active_pokemon_at_slot(user_position.slot) {
            // Determine highest stat (excluding HP)
            let highest_stat = calculate_highest_stat_excluding_hp(pokemon);
            
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(highest_stat, 1);
            
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::BoostStats(BoostStatsInstruction {
                    target_position: user_position,
                    stat_boosts,
                    previous_boosts: Some(HashMap::new()),
                })
            ]));
            
            // Apply Protosynthesis volatile status to track that it's active
            let protosynthesis_status = match highest_stat {
                Stat::Attack => crate::core::instruction::VolatileStatus::ProtosynthesisAttack,
                Stat::Defense => crate::core::instruction::VolatileStatus::ProtosynthesisDefense,
                Stat::SpecialAttack => crate::core::instruction::VolatileStatus::ProtosynthesisSpecialAttack,
                Stat::SpecialDefense => crate::core::instruction::VolatileStatus::ProtosynthesisSpecialDefense,
                Stat::Speed => crate::core::instruction::VolatileStatus::ProtosynthesisSpeed,
                _ => crate::core::instruction::VolatileStatus::ProtosynthesisAttack, // Fallback
            };
            
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: protosynthesis_status,
                    duration: None, // Lasts as long as conditions are met
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Quark Drive ability effect - Boosts highest stat in Electric Terrain (or with Booster Energy)
fn apply_quark_drive_effect(
    state: &State,
    user_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Check if conditions are met for Quark Drive activation
    let should_activate = state.terrain == crate::core::instruction::Terrain::ElectricTerrain;
    
    if should_activate {
        let user_side = state.get_side(user_position.side);
        if let Some(pokemon) = user_side.get_active_pokemon_at_slot(user_position.slot) {
            // Determine highest stat (excluding HP)
            let highest_stat = calculate_highest_stat_excluding_hp(pokemon);
            
            let mut stat_boosts = HashMap::new();
            stat_boosts.insert(highest_stat, 1);
            
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::BoostStats(BoostStatsInstruction {
                    target_position: user_position,
                    stat_boosts,
                    previous_boosts: Some(HashMap::new()),
                })
            ]));
            
            // Apply Quark Drive volatile status to track that it's active
            let quark_drive_status = match highest_stat {
                Stat::Attack => crate::core::instruction::VolatileStatus::QuarkDriveAttack,
                Stat::Defense => crate::core::instruction::VolatileStatus::QuarkDriveDefense,
                Stat::SpecialAttack => crate::core::instruction::VolatileStatus::QuarkDriveSpecialAttack,
                Stat::SpecialDefense => crate::core::instruction::VolatileStatus::QuarkDriveSpecialDefense,
                Stat::Speed => crate::core::instruction::VolatileStatus::QuarkDriveSpeed,
                _ => crate::core::instruction::VolatileStatus::QuarkDriveAttack, // Fallback
            };
            
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
                    target_position: user_position,
                    volatile_status: quark_drive_status,
                    duration: None, // Lasts as long as conditions are met
                })
            ]));
        }
    }
    
    instructions
}

// =============================================================================
// OGERPON EMBODY ASPECT ABILITIES
// =============================================================================

/// Apply Embody Aspect ability effect (Ogerpon) - Boosts different stats based on forme
fn apply_embody_aspect_effect(
    state: &State,
    user_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    let user_side = state.get_side(user_position.side);
    if let Some(pokemon) = user_side.get_active_pokemon_at_slot(user_position.slot) {
        // Determine which stat to boost based on Ogerpon's forme
        let stat_to_boost = match pokemon.species.to_lowercase().as_str() {
            "ogerpon" | "ogerponteal" => Stat::Speed,                    // Teal Mask (base forme)
            "ogerponwellspring" => Stat::SpecialDefense,                 // Wellspring Mask
            "ogerponcornerstone" => Stat::Defense,                       // Cornerstone Mask  
            "ogerponhearthflame" => Stat::Attack,                        // Hearthflame Mask
            _ => return instructions, // Not an Ogerpon forme
        };
        
        let mut stat_boosts = HashMap::new();
        stat_boosts.insert(stat_to_boost, 1);
        
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

// =============================================================================
// UTILITY ABILITIES
// =============================================================================

/// Apply Screen Cleaner ability effect - Removes all screens from both sides
fn apply_screen_cleaner_effect(
    _state: &State,
    _user_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Remove Reflect, Light Screen, and Aurora Veil from both sides
    for side_ref in [crate::core::battle_format::SideReference::SideOne, crate::core::battle_format::SideReference::SideTwo] {
        for screen in [
            crate::core::instruction::SideCondition::Reflect,
            crate::core::instruction::SideCondition::LightScreen,
            crate::core::instruction::SideCondition::AuroraVeil,
        ] {
            instructions.push(StateInstructions::new(100.0, vec![
                Instruction::RemoveSideCondition(crate::core::instruction::RemoveSideConditionInstruction {
                    side: side_ref,
                    condition: screen,
                })
            ]));
        }
    }
    
    instructions
}

/// Apply Slow Start ability effect (Regigigas) - Applies Slow Start volatile status for 5 turns
fn apply_slow_start_effect(
    _state: &State,
    user_position: BattlePosition,
    _generation: &GenerationMechanics,
) -> Vec<StateInstructions> {
    let mut instructions = Vec::new();
    
    // Apply Slow Start volatile status (halves Attack and Speed for 5 turns)
    instructions.push(StateInstructions::new(100.0, vec![
        Instruction::ApplyVolatileStatus(crate::core::instruction::ApplyVolatileStatusInstruction {
            target_position: user_position,
            volatile_status: crate::core::instruction::VolatileStatus::SlowStart,
            duration: Some(5),
        })
    ]));
    
    instructions
}

// =============================================================================
// HELPER FUNCTIONS FOR NEW ABILITIES
// =============================================================================

/// Calculate the highest stat of a Pokemon excluding HP
fn calculate_highest_stat_excluding_hp(pokemon: &Pokemon) -> Stat {
    let stats = [
        (Stat::Attack, pokemon.get_effective_stat(Stat::Attack)),
        (Stat::Defense, pokemon.get_effective_stat(Stat::Defense)),
        (Stat::SpecialAttack, pokemon.get_effective_stat(Stat::SpecialAttack)),
        (Stat::SpecialDefense, pokemon.get_effective_stat(Stat::SpecialDefense)),
        (Stat::Speed, pokemon.get_effective_stat(Stat::Speed)),
    ];
    
    // Find the stat with the highest value
    stats.iter()
        .max_by_key(|(_, value)| *value)
        .map(|(stat, _)| *stat)
        .unwrap_or(Stat::Attack) // Default to Attack if somehow nothing is found
}