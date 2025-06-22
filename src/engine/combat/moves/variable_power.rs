//! # Variable Power Move Effects

//! 
//! This module contains variable power move implementations that calculate power
//! based on various battle conditions such as HP, status, speed, weather, etc.

use crate::core::battle_state::{Pokemon, MoveCategory, BattleState};
use crate::core::instructions::{PokemonStatus, VolatileStatus, Stat, Weather, SideCondition, Terrain};
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction, FieldInstruction, StatsInstruction};
use crate::data::ps::repository::Repository;
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};
use crate::engine::combat::moves::MoveContext;
use crate::engine::combat::moves::simple;
use std::collections::HashMap;
use crate::data::showdown_types::MoveData;

// =============================================================================
// VARIABLE POWER MOVE FUNCTIONS
// =============================================================================

/// Apply Facade - doubles power when user has burn/paralysis/poison
pub fn apply_facade(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        // Facade doubles power if user has a status condition (Burn, Paralysis, Poison)
        let has_status = matches!(user.status, 
            PokemonStatus::Burn | PokemonStatus::Paralysis | 
            PokemonStatus::Poison | PokemonStatus::Toxic
        );
        
        if has_status {
            // Return a power modifier instruction that doubles the base power
            // This will be handled by the damage calculation system
            return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, 2.0);
        }
    }
    
    // No status condition, normal power
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Hex - doubles power against statused targets
pub fn apply_hex(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut has_statused_target = false;
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.status != PokemonStatus::None {
                has_statused_target = true;
                break;
            }
        }
    }
    
    if has_statused_target {
        // Return a power modifier instruction that doubles the base power
        return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, 2.0);
    }
    
    // No statused targets, normal power
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Gyro Ball - power based on speed comparison
pub fn apply_gyro_ball(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if target_positions.is_empty() {
        return vec![BattleInstructions::new(100.0, vec![])];
    }
    
    let target_position = target_positions[0];
    
    if let (Some(user), Some(target)) = (
        state.get_pokemon_at_position(user_position),
        state.get_pokemon_at_position(target_position)
    ) {
        // Gyro Ball power = min(150, max(1, 25 * target_speed / user_speed))
        let user_speed = user.stats.speed as f32;
        let target_speed = target.stats.speed as f32;
        
        if user_speed > 0.0 && move_data.base_power > 0 {
            let base_power = move_data.base_power as f32;
            let power_multiplier = ((25.0 * target_speed / user_speed) / base_power).min(150.0 / base_power).max(1.0 / base_power);
            return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier);
        }
    }
    
    // Fallback to normal power
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Reversal - power based on user's remaining HP
pub fn apply_reversal(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        // Reversal power based on HP percentage
        let hp_percentage = (user.hp as f32 / user.max_hp as f32) * 100.0;
        
        let base_power = if hp_percentage > 68.75 {
            20
        } else if hp_percentage > 35.42 {
            40
        } else if hp_percentage > 20.83 {
            80
        } else if hp_percentage > 10.42 {
            100
        } else if hp_percentage > 4.17 {
            150
        } else {
            200
        };
        
        let power_multiplier = if move_data.base_power > 0 {
            base_power as f32 / move_data.base_power as f32
        } else {
            1.0 // Fallback if move has no base power
        };
        return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier);
    }
    
    // Fallback to normal power
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Acrobatics - doubles power when user has no held item
pub fn apply_acrobatics(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        // Acrobatics doubles power if user has no item or an unusable item
        let has_no_item = user.item.is_none() || user.item.as_ref().map_or(true, |item| item.is_empty());
        
        if has_no_item {
            return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, 2.0);
        }
    }
    
    // Has item, normal power
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Weather Ball - power and type change with weather
pub fn apply_weather_ball(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Weather Ball doubles power and changes type based on weather
    let power_multiplier = match state.weather() {
        Weather::Sun | Weather::HarshSun | Weather::HarshSunlight |
        Weather::Rain | Weather::HeavyRain |
        Weather::Sand | Weather::Sandstorm |
        Weather::Hail | Weather::Snow | Weather::StrongWinds => 2.0,
        Weather::None => 1.0,
    };
    
    if power_multiplier > 1.0 {
        return apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier);
    }
    
    // No weather, normal power
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Avalanche - doubles power if user was damaged this turn
pub fn apply_avalanche(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Check if user was hit by a Physical or Special move this turn and moved second
    let power_multiplier = if user_moved_after_taking_damage(state, user_position) {
        2.0 // Double power if user took damage from attack and moved second
    } else {
        1.0 // Base power
    };
    
    // Apply generic damage with modified power
    let modified_move_data = MoveData {
        base_power: ((move_data.base_power as f32 * power_multiplier) as u16),
        ..move_data.clone()
    };
    
    let generic_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
    instructions.extend(generic_instructions);
    
    instructions
}

/// Apply Bolt Beak - doubles power if user goes first
pub fn apply_boltbeak(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    // Check if user moves first this turn using context
    let power_multiplier = if context.going_first {
        2.0 // Double power when moving first
    } else {
        1.0 // Base power
    };
    
    let modified_move_data = MoveData {
        base_power: ((move_data.base_power as f32 * power_multiplier) as u16),
        ..move_data.clone()
    };
    
    let generic_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
    instructions.extend(generic_instructions);
    
    instructions
}

/// Apply Fishious Rend - doubles power if user goes first
pub fn apply_fishious_rend(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
) -> Vec<BattleInstructions> {
    // Fishious Rend has identical mechanics to Bolt Beak
    apply_boltbeak(state, move_data, user_position, target_positions, generation, context)
}

/// Apply Electro Ball - power based on speed comparison
pub fn apply_electroball(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        for &target_position in target_positions {
            if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
                // Calculate speed stats with boosts
                let user_speed = calculate_boosted_speed(user_pokemon);
                let target_speed = calculate_boosted_speed(target_pokemon);
                
                // Calculate speed ratio and determine power
                let speed_ratio = if target_speed > 0 {
                    user_speed as f32 / target_speed as f32
                } else {
                    4.0 // Max power if target has 0 speed
                };
                
                let base_power = if speed_ratio >= 4.0 {
                    150i16
                } else if speed_ratio >= 3.0 {
                    120i16
                } else if speed_ratio >= 2.0 {
                    80i16
                } else if speed_ratio >= 1.0 {
                    60i16
                } else {
                    40i16
                };
                
                let modified_move_data = MoveData {
                    base_power: base_power as u16,
                    ..move_data.clone()
                };
                
                let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
                instructions.extend(target_instructions);
            }
        }
    }
    
    instructions
}

/// Apply Eruption - power based on user's remaining HP
pub fn apply_eruption(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        // Power = 150 * (current HP / max HP)
        let hp_percentage = user_pokemon.hp as f32 / user_pokemon.max_hp as f32;
        let base_power = (150.0 * hp_percentage).max(1.0) as i16; // Minimum 1 power
        
        let modified_move_data = MoveData {
            base_power: base_power as u16,
            ..move_data.clone()
        };
        
        let generic_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
        instructions.extend(generic_instructions);
    }
    
    instructions
}

/// Apply Water Spout - power based on user's remaining HP
pub fn apply_waterspout(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Water Spout has identical mechanics to Eruption
    apply_eruption(state, move_data, user_position, target_positions, generation)
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Apply a power modifier to a move
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
    apply_generic_effects(state, &modified_move, user_position, target_positions, generation)
}

/// Calculate speed with stat boosts applied
fn calculate_boosted_speed(pokemon: &Pokemon) -> i32 {
    let base_speed = pokemon.stats.speed;
    let boost_multiplier = match pokemon.stat_boosts.get(&Stat::Speed).unwrap_or(&0) {
        -6 => 0.25,
        -5 => 0.28,
        -4 => 0.33,
        -3 => 0.4,
        -2 => 0.5,
        -1 => 0.66,
        0 => 1.0,
        1 => 1.5,
        2 => 2.0,
        3 => 2.5,
        4 => 3.0,
        5 => 3.5,
        6 => 4.0,
        _ => 1.0,
    };
    
    (base_speed as f32 * boost_multiplier) as i32
}

/// Check if user moved after taking damage this turn
fn user_moved_after_taking_damage(state: &BattleState, user_position: BattlePosition) -> bool {
    // Use the modern battle state's turn tracking system
    state.user_moved_after_taking_damage(user_position)
}

/// Apply Dragon Energy - power based on user's remaining HP
pub fn apply_dragon_energy(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Dragon Energy has identical mechanics to Eruption
    apply_eruption(state, move_data, user_position, target_positions, generation)
}

/// Apply Grass Knot - power based on target's weight
pub fn apply_grass_knot(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
            // Power based on target's weight (in kg)
            let target_weight = target_pokemon.weight_kg;
            let base_power = if target_weight >= 200.0 {
                120i16
            } else if target_weight >= 100.0 {
                100i16
            } else if target_weight >= 50.0 {
                80i16
            } else if target_weight >= 25.0 {
                60i16
            } else if target_weight >= 10.0 {
                40i16
            } else {
                20i16
            };
            
            let modified_move_data = MoveData {
                base_power: base_power as u16,
                ..move_data.clone()
            };
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    instructions
}

/// Apply Low Kick - power based on target's weight
pub fn apply_low_kick(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Low Kick has identical mechanics to Grass Knot
    apply_grass_knot(state, move_data, user_position, target_positions, generation)
}

/// Apply Heat Crash - power based on weight ratio between user and target
pub fn apply_heat_crash(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        let user_weight = user_pokemon.weight_kg;
        
        for &target_position in target_positions {
            if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
                let target_weight = target_pokemon.weight_kg;
                let weight_ratio = if target_weight > 0.0 {
                    user_weight / target_weight
                } else {
                    5.0 // Max power if target has 0 weight
                };
                
                let base_power = if weight_ratio >= 5.0 {
                    120i16
                } else if weight_ratio >= 4.0 {
                    100i16
                } else if weight_ratio >= 3.0 {
                    80i16
                } else if weight_ratio >= 2.0 {
                    60i16
                } else {
                    40i16
                };
                
                let modified_move_data = MoveData {
                    base_power: base_power as u16,
                    ..move_data.clone()
                };
                
                let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
                instructions.extend(target_instructions);
            }
        }
    }
    
    instructions
}

/// Apply Heavy Slam - power based on weight ratio between user and target
pub fn apply_heavy_slam(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Heavy Slam has identical mechanics to Heat Crash
    apply_heat_crash(state, move_data, user_position, target_positions, generation)
}

/// Apply Barb Barrage - doubles power against poisoned targets
pub fn apply_barb_barrage(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Modify power based on poison status
            let mut modified_move_data = move_data.clone();
            if target.status == PokemonStatus::Poison || target.status == PokemonStatus::Toxic {
                if modified_move_data.base_power > 0 {
                    modified_move_data.base_power = (modified_move_data.base_power * 2); // Double power
                }
            }
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Collision Course - 1.3x power when super effective
pub fn apply_collision_course(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    let move_type = &move_data.move_type;
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let mut modified_move_data = move_data.clone();
            
            // Check if move is super effective against target
            if is_super_effective(move_type, target, generation) {
                let current_power = modified_move_data.base_power.max(100);
                modified_move_data.base_power = ((current_power as f32 * 1.3) as u16);
            }
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Electro Drift - 1.3x power when super effective
pub fn apply_electro_drift(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Electro Drift has identical mechanics to Collision Course
    apply_collision_course(state, move_data, user_position, target_positions, generation)
}

/// Apply Freeze-Dry - Ice move that's super effective against Water types
pub fn apply_freeze_dry(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let mut modified_move_data = move_data.clone();
            
            // Check if target is Water type - if so, boost damage significantly
            let has_water_type = target.types.get(0).map_or(false, |t| t.to_lowercase() == "water") || 
                                 target.types.get(1).map_or(false, |t| t.to_lowercase() == "water");
            
            if has_water_type {
                // Freeze-Dry treats Water types as if they were weak to Ice
                // This effectively makes it super effective (2x) against Water
                let current_power = modified_move_data.base_power.max(70);
                modified_move_data.base_power = ((current_power as f32 * 2.0) as u16);
            }
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Hard Press - power decreases as target's HP increases (1-100 power)
pub fn apply_hard_press(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let hp_percentage = (target.hp as f32 / target.max_hp as f32) * 100.0;
            let power = ((hp_percentage / 100.0) * 100.0).max(1.0) as i16;
            
            let mut modified_move_data = move_data.clone();
            modified_move_data.base_power = (power as u16);
            
            let target_instructions = apply_generic_effects(state, &modified_move_data, user_position, &[target_position], generation);
            instructions.extend(target_instructions);
        }
    }
    
    instructions
}

/// Apply Hydro Steam - boosted power in sun weather
pub fn apply_hydro_steam(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let power_multiplier = match state.weather() {
        Weather::Sun | Weather::HarshSun | Weather::HarshSunlight => 1.5, // 1.5x power in sun
        Weather::Rain | Weather::HeavyRain | Weather::Sand | Weather::Sandstorm |
        Weather::Hail | Weather::Snow | Weather::StrongWinds | Weather::None => 1.0,
    };
    
    apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier)
}

/// Apply Last Respects - power increases based on fainted team members
pub fn apply_last_respects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let user_side = match user_position.side {
        SideReference::SideOne => &state.sides[0],
        SideReference::SideTwo => &state.sides[1],
    };
    
    // Count fainted Pokemon
    let fainted_count = user_side.pokemon.iter()
        .filter(|p| p.hp == 0)
        .count() as u8;
    
    let power = 50 + (fainted_count * 50); // Base 50 + 50 per fainted
    let mut modified_move_data = move_data.clone();
    modified_move_data.base_power = (power.min(250) as u16); // Cap at reasonable power
    
    apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
}

/// Apply Poltergeist - fails if target has no item
pub fn apply_poltergeist(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            if target.item.is_none() {
                // Move fails if target has no item
                instructions.push(BattleInstructions::new(100.0, vec![]));
            } else {
                let target_instructions = apply_generic_effects(state, move_data, user_position, &[target_position], generation);
                instructions.extend(target_instructions);
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Pursuit - doubles power against switching targets
pub fn apply_pursuit(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &super::MoveContext,
) -> Vec<BattleInstructions> {
    // Check if targets are switching using proper opponent context
    let is_targeting_switcher = target_positions.iter().any(|&target_pos| {
        context.is_opponent_switching(target_pos)
    });
    
    if is_targeting_switcher {
        // Double power against switching targets
        let power_multiplier = 2.0;
        apply_power_modifier_move(state, move_data, user_position, target_positions, generation, power_multiplier)
    } else {
        // Normal power
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Stored Power - power increases with stat boosts
pub fn apply_stored_power(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let total_boosts: i32 = user.stat_boosts.values()
            .filter(|&&boost| boost > 0)
            .map(|&boost| boost as i32)
            .sum();
        
        let power = 20 + (total_boosts * 20); // Base 20 + 20 per positive boost
        let mut modified_move_data = move_data.clone();
        modified_move_data.base_power = (power.min(250) as u16); // Cap at reasonable power
        
        apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
    } else {
        apply_generic_effects(state, move_data, user_position, target_positions, generation)
    }
}

/// Apply Power Trip - identical to Stored Power
pub fn apply_power_trip(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_stored_power(state, move_data, user_position, target_positions, generation)
}

/// Apply Strength Sap - heals based on target's Attack stat and lowers it
pub fn apply_strength_sap(
    state: &BattleState,
    _move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instruction_list = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Lower target's Attack by 1 stage
            let mut stat_changes = HashMap::new();
            stat_changes.insert(Stat::Attack, -1);
            instruction_list.push(BattleInstruction::Stats(StatsInstruction::BoostStats {
                target: target_position,
                stat_changes: stat_changes,
                previous_boosts: HashMap::new(),
            }));
            
            // Heal user based on target's current Attack stat
            if let Some(user) = state.get_pokemon_at_position(user_position) {
                let heal_amount = target.stats.attack as i16;
                instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Heal {
                    target: user_position,
                    amount: heal_amount,
                    previous_hp: Some(user.hp),
                }));
            }
        }
    }
    
    if instruction_list.is_empty() {
        vec![BattleInstructions::new(100.0, vec![])]
    } else {
        vec![BattleInstructions::new(100.0, instruction_list)]
    }
}

/// Apply Sucker Punch - priority move that fails against status moves
pub fn apply_sucker_punch(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &super::MoveContext,
) -> Vec<BattleInstructions> {
    // Sucker Punch fails if the target isn't using a damaging move
    for &target_position in target_positions {
        if context.is_opponent_using_status_move(target_position) || context.is_opponent_switching(target_position) {
            // Move fails if target is using a status move or switching
            return vec![BattleInstructions::new(100.0, vec![])];
        }
    }
    
    // If all targets are using damaging moves, proceed normally
    apply_generic_effects(state, move_data, user_position, target_positions, generation)
}

/// Apply Thunder Clap - priority move that fails against status moves
pub fn apply_thunder_clap(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &super::MoveContext,
) -> Vec<BattleInstructions> {
    // Thunder Clap fails if the target isn't using a damaging move (same as Sucker Punch)
    for &target_position in target_positions {
        if context.is_opponent_using_status_move(target_position) || context.is_opponent_switching(target_position) {
            // Move fails if target is using a status move or switching
            return vec![BattleInstructions::new(100.0, vec![])];
        }
    }
    
    // If all targets are using damaging moves, proceed normally
    apply_generic_effects(state, move_data, user_position, target_positions, generation)
}

/// Apply Terrain Pulse - power and type change based on terrain
pub fn apply_terrain_pulse(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut modified_move_data = move_data.clone();
    
    // Base power is 50, doubles to 100 on terrain
    let base_power = 50;
    
    // Type and power change based on terrain
    match state.terrain() {
        Terrain::Electric | Terrain::ElectricTerrain => {
            modified_move_data.move_type = "Electric".to_string();
            modified_move_data.base_power = (base_power * 2);
        }
        Terrain::Grassy | Terrain::GrassyTerrain => {
            modified_move_data.move_type = "Grass".to_string();
            modified_move_data.base_power = (base_power * 2);
        }
        Terrain::Misty | Terrain::MistyTerrain => {
            modified_move_data.move_type = "Fairy".to_string();
            modified_move_data.base_power = (base_power * 2);
        }
        Terrain::Psychic | Terrain::PsychicTerrain => {
            modified_move_data.move_type = "Psychic".to_string();
            modified_move_data.base_power = (base_power * 2);
        }
        Terrain::None => {
            // Remains Normal type with base power
            modified_move_data.move_type = "Normal".to_string();
            modified_move_data.base_power = (base_power);
        }
    }
    
    apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation)
}

/// Apply Upper Hand - priority counter to priority moves
pub fn apply_upper_hand(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &super::MoveContext,
) -> Vec<BattleInstructions> {
    use crate::core::instructions::{BattleInstruction, StatusInstruction};
    use crate::core::instructions::VolatileStatus;
    
    // Check if any target is using a priority move this turn using proper context
    let opponent_using_priority = target_positions.iter().any(|&target_pos| {
        if let Some(priority) = context.get_opponent_priority(target_pos) {
            priority > 0 && !context.is_opponent_switching(target_pos)
        } else {
            false
        }
    });
    
    let mut modified_move_data = move_data.clone();
    let mut instructions = Vec::new();
    
    if opponent_using_priority {
        // Upper Hand gets increased power and causes flinch when countering priority moves
        modified_move_data.base_power = (65); // Increased from base 40 power
        
        // Apply flinch effect to targets after damage
        for &target_position in target_positions {
            let flinch_instruction = BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                target: target_position,
                status: VolatileStatus::Flinch,
                duration: Some(1),
                previous_had_status: false,
                previous_duration: None,
            });
            instructions.push(BattleInstructions::new(100.0, vec![flinch_instruction]));
        }
    } else {
        // Upper Hand fails if opponent is not using a priority move
        return vec![BattleInstructions::new(100.0, vec![])];
    }
    
    // Apply the modified move effects first
    let mut move_instructions = apply_generic_effects(state, &modified_move_data, user_position, target_positions, generation);
    
    // Append flinch instructions
    move_instructions.extend(instructions);
    
    move_instructions
}


/// Convert from move_choice::PokemonType to type_effectiveness::PokemonType
fn convert_tera_type(tera: crate::core::move_choice::PokemonType) -> PokemonType {
    match tera {
        crate::core::move_choice::PokemonType::Normal => PokemonType::Normal,
        crate::core::move_choice::PokemonType::Fire => PokemonType::Fire,
        crate::core::move_choice::PokemonType::Water => PokemonType::Water,
        crate::core::move_choice::PokemonType::Electric => PokemonType::Electric,
        crate::core::move_choice::PokemonType::Grass => PokemonType::Grass,
        crate::core::move_choice::PokemonType::Ice => PokemonType::Ice,
        crate::core::move_choice::PokemonType::Fighting => PokemonType::Fighting,
        crate::core::move_choice::PokemonType::Poison => PokemonType::Poison,
        crate::core::move_choice::PokemonType::Ground => PokemonType::Ground,
        crate::core::move_choice::PokemonType::Flying => PokemonType::Flying,
        crate::core::move_choice::PokemonType::Psychic => PokemonType::Psychic,
        crate::core::move_choice::PokemonType::Bug => PokemonType::Bug,
        crate::core::move_choice::PokemonType::Rock => PokemonType::Rock,
        crate::core::move_choice::PokemonType::Ghost => PokemonType::Ghost,
        crate::core::move_choice::PokemonType::Dragon => PokemonType::Dragon,
        crate::core::move_choice::PokemonType::Dark => PokemonType::Dark,
        crate::core::move_choice::PokemonType::Steel => PokemonType::Steel,
        crate::core::move_choice::PokemonType::Fairy => PokemonType::Fairy,
        crate::core::move_choice::PokemonType::Unknown => PokemonType::Normal,
    }
}

/// Calculate type effectiveness for a move against a target
fn calculate_type_effectiveness(move_type: &str, target: &Pokemon, generation: &GenerationMechanics) -> f32 {
    let type_chart = TypeChart::new(generation.generation.number());
    let move_pokemon_type = match PokemonType::from_str(move_type) {
        Some(t) => t,
        None => return 1.0, // Neutral if invalid type
    };
    
    // Get target's types
    let target_type1 = PokemonType::from_str(&target.types[0]).unwrap_or(PokemonType::Normal);
    let target_type2 = if target.types.len() > 1 {
        PokemonType::from_str(&target.types[1]).unwrap_or(target_type1)
    } else {
        target_type1
    };
    
    // Handle Tera type if applicable
    let tera_type = if target.is_terastallized {
        target.tera_type.map(convert_tera_type)
    } else {
        None
    };
    
    type_chart.calculate_damage_multiplier(
        move_pokemon_type,
        (target_type1, target_type2),
        tera_type,
        None, // No special move override
    )
}

/// Check if move is super effective (>1.0 multiplier)
pub fn is_super_effective(move_type: &str, target: &Pokemon, generation: &GenerationMechanics) -> bool {
    calculate_type_effectiveness(move_type, target, generation) > 1.0
}

/// Check if a move is not very effective against a target (<1.0 multiplier)
pub fn is_not_very_effective(move_type: &str, target: &Pokemon, generation: &GenerationMechanics) -> bool {
    let effectiveness = calculate_type_effectiveness(move_type, target, generation);
    effectiveness > 0.0 && effectiveness < 1.0
}

/// Check if a target is immune to a move type (0.0 multiplier)
pub fn is_immune_to_type(move_type: &str, target: &Pokemon, generation: &GenerationMechanics) -> bool {
    calculate_type_effectiveness(move_type, target, generation) == 0.0
}

/// Check if a Pokemon is immune to paralysis
pub fn is_immune_to_paralysis(target: &Pokemon, generation: &GenerationMechanics) -> bool {
    // Electric types are immune to paralysis in Gen 6+
    if generation.generation.number() >= 6 {
        target.types.iter().any(|t| t.to_lowercase() == "electric")
    } else {
        false
    }
}

/// Check if a Pokemon is immune to poison
pub fn is_immune_to_poison(target: &Pokemon, generation: &GenerationMechanics) -> bool {
    // Poison and Steel types are immune to poison
    target.types.iter().any(|t| {
        let t_lower = t.to_lowercase();
        t_lower == "poison" || t_lower == "steel"
    })
}

/// Apply generic move effects (damage calculation with standard mechanics)
fn apply_generic_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Delegate to the simple module's generic effects implementation
    crate::engine::combat::moves::simple::apply_generic_effects(
        state, move_data, user_position, target_positions, generation
    )
}


// Placeholder function removed - moves now use proper MoveContext with opponent switching information

/// Apply Me First - copies opponent's move with 1.5x power if user goes first
pub fn apply_me_first(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &super::MoveContext,
    repository: &crate::data::ps::repository::Repository,
) -> Vec<BattleInstructions> {
    // Me First only works if the user goes first
    if !context.going_first {
        return vec![BattleInstructions::new(100.0, vec![])]; // Move fails
    }
    
    // Find the first target that is using a damaging move
    for &target_pos in target_positions {
        if let Some(opponent_info) = context.opponent_moves.get(&target_pos) {
            // Me First fails against status moves and switching
            if opponent_info.is_switching || opponent_info.move_category == crate::core::battle_state::MoveCategory::Status {
                continue;
            }
            
            // Try to copy the opponent's move from repository
            if let Some(repo_move_data) = repository.find_move_by_name(&opponent_info.move_name) {
                // Convert to showdown_types::MoveData first
                let mut enhanced_move = crate::data::showdown_types::MoveData {
                    name: repo_move_data.name.clone(),
                    base_power: repo_move_data.base_power as u16,
                    accuracy: repo_move_data.accuracy as u16,
                    pp: repo_move_data.pp,
                    max_pp: repo_move_data.max_pp,
                    move_type: repo_move_data.move_type.to_string(),
                    category: repo_move_data.category.clone(),
                    priority: repo_move_data.priority,
                    target: repo_move_data.target.clone(),
                    flags: repo_move_data.flags.iter().map(|f| (f.clone(), 1)).collect(),
                    drain: repo_move_data.drain,
                    recoil: repo_move_data.recoil,
                    ..crate::data::showdown_types::MoveData::default()
                };
                
                // Create enhanced version with 1.5x power
                if enhanced_move.base_power > 0 {
                    enhanced_move.base_power = ((enhanced_move.base_power as f32 * 1.5) as u16).min(999);
                }
                
                // Use the copied move with enhanced power
                return simple::apply_generic_effects(state, &enhanced_move, user_position, target_positions, generation);
            }
        }
    }
    
    // Me First failed - no valid move to copy
    vec![BattleInstructions::new(100.0, vec![])]
}

// Placeholder function removed - moves now use proper MoveContext with opponent information