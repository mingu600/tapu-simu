//! # Healing Move Effects
//! 
//! This module contains healing move effects extracted from move_effects.rs.
//! These functions handle moves that restore HP or provide healing effects.
//!
//! All moves in this module have been converted to use the new composer system.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction, PokemonStatus, Weather};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::composers::status_moves::healing_move;
use crate::engine::combat::core::status_system::{StatusApplication, apply_multiple_status_effects};

// =============================================================================
// HEALING MOVE MACRO
// =============================================================================

/// Macro for simple healing moves
macro_rules! simple_healing_move {
    ($func_name:ident, $heal_fraction:expr) => {
        pub fn $func_name(
            state: &BattleState,
            user_position: BattlePosition,
            target_positions: &[BattlePosition],
            _generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            let target_position = if target_positions.is_empty() {
                user_position
            } else {
                target_positions[0]
            };
            
            vec![BattleInstructions::new(100.0, healing_move(state, target_position, $heal_fraction, Some(user_position)))]
        }
    };
}

// =============================================================================
// BASIC HEALING MOVES
// =============================================================================

/// Apply Recover - restores 50% of max HP
simple_healing_move!(apply_recover, 0.5);

/// Apply Roost - restores 50% of max HP
simple_healing_move!(apply_roost, 0.5);

/// Apply Slack Off - restores 50% of max HP
simple_healing_move!(apply_slack_off, 0.5);

/// Apply Soft Boiled - restores 50% of max HP
simple_healing_move!(apply_soft_boiled, 0.5);

/// Apply Milk Drink - restores 50% of max HP
simple_healing_move!(apply_milk_drink, 0.5);

/// Apply Moonlight - restores variable HP based on weather
pub fn apply_moonlight(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    // Weather affects healing amount
    let heal_fraction = match state.weather() {
        Weather::Sun | Weather::HarshSun => 0.667, // 2/3 HP
        Weather::Rain | Weather::HeavyRain | Weather::Sandstorm | Weather::Hail => 0.25, // 1/4 HP
        _ => 0.5, // 1/2 HP in clear weather
    };
    
    vec![BattleInstructions::new(100.0, healing_move(state, target_position, heal_fraction, Some(user_position)))]
}

/// Apply Synthesis - restores variable HP based on weather
pub fn apply_synthesis(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    // Weather affects healing amount
    let heal_fraction = match state.weather() {
        Weather::Sun | Weather::HarshSun => 0.667, // 2/3 HP
        Weather::Rain | Weather::HeavyRain | Weather::Sandstorm | Weather::Hail => 0.25, // 1/4 HP
        _ => 0.5, // 1/2 HP in clear weather
    };
    
    vec![BattleInstructions::new(100.0, healing_move(state, target_position, heal_fraction, Some(user_position)))]
}

/// Apply Morning Sun - restores variable HP based on weather
pub fn apply_morning_sun(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    // Weather affects healing amount
    let heal_fraction = match state.weather() {
        Weather::Sun | Weather::HarshSun => 0.667, // 2/3 HP
        Weather::Rain | Weather::HeavyRain | Weather::Sandstorm | Weather::Hail => 0.25, // 1/4 HP
        _ => 0.5, // 1/2 HP in clear weather
    };
    
    vec![BattleInstructions::new(100.0, healing_move(state, target_position, heal_fraction, Some(user_position)))]
}

/// Apply Rest - restores full HP and induces sleep
pub fn apply_rest(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    let mut instructions = healing_move(state, target_position, 1.0, Some(user_position)); // Full heal
    
    // Add sleep status
    let sleep_instructions = apply_multiple_status_effects(
        state,
        vec![StatusApplication {
            status: PokemonStatus::Sleep,
            target: target_position,
            chance: 100.0,
            duration: Some(2), // 2 turns of sleep
        }]
    );
    
    instructions.extend(sleep_instructions);
    
    vec![BattleInstructions::new(100.0, instructions)]
}

/// Apply Pain Split - averages HP between user and target
pub fn apply_pain_split(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if target_positions.is_empty() {
        return vec![BattleInstructions::new(100.0, vec![])];
    }
    
    let target_position = target_positions[0];
    
    if let (Some(user), Some(target)) = (
        state.get_pokemon_at_position(user_position),
        state.get_pokemon_at_position(target_position)
    ) {
        let average_hp = (user.hp + target.hp) / 2;
        let user_heal = average_hp - user.hp;
        let target_heal = average_hp - target.hp;
        
        let mut instructions = Vec::new();
        
        if user_heal > 0 {
            instructions.extend(healing_move(state, user_position, user_heal as f32 / user.max_hp as f32, Some(user_position)));
        } else if user_heal < 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: user_position,
                amount: (-user_heal) as i16,
                previous_hp: None,
            }));
        }
        
        if target_heal > 0 {
            instructions.extend(healing_move(state, target_position, target_heal as f32 / target.max_hp as f32, Some(user_position)));
        } else if target_heal < 0 {
            instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: (-target_heal) as i16,
                previous_hp: None,
            }));
        }
        
        vec![BattleInstructions::new(100.0, instructions)]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Wish - heals the user's side at the end of the next turn
pub fn apply_wish(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Get the user's max HP for wish calculation
    let heal_amount = if let Some(user_pokemon) = state.get_pokemon_at_position(user_position) {
        user_pokemon.max_hp / 2 // Wish heals 50% of the user's max HP
    } else {
        0
    };
    
    // Set up delayed healing for 2 turns from now
    let wish_instruction = BattleInstruction::Pokemon(PokemonInstruction::SetWish {
        target: user_position,
        heal_amount,
        turns_remaining: 2,
        previous_wish: None, // Will be filled by battle state
    });
    
    vec![BattleInstructions::new(100.0, vec![wish_instruction])]
}

/// Apply Heal Bell - cures status conditions of all team members
pub fn apply_heal_bell(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::core::battle_format::SideReference;
    
    let user_side = user_position.side;
    let mut cure_instructions = Vec::new();
    
    // Cure all Pokemon on the user's side (active and bench)
    let side_index = user_side.to_index();
    for (pokemon_index, pokemon) in state.sides[side_index].pokemon.iter().enumerate() {
        if pokemon.status != PokemonStatus::None {
            // Only cure if Pokemon has a status condition
            let position = crate::core::battle_format::BattlePosition {
                side: user_side,
                slot: pokemon_index,
            };
            
            let status_applications = vec![StatusApplication {
                status: PokemonStatus::None, // Clear status
                target: position,
                chance: 100.0,
                duration: None,
            }];
            
            cure_instructions.extend(apply_multiple_status_effects(state, status_applications));
        }
    }
    
    vec![BattleInstructions::new(100.0, cure_instructions)]
}

/// Apply Aromatherapy - cures status conditions of all team members
pub fn apply_aromatherapy(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Same as Heal Bell
    apply_heal_bell(state, user_position, target_positions, generation)
}

/// Apply Aqua Ring - heals 1/16 of max HP each turn
pub fn apply_aqua_ring(
    _state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    use crate::core::instructions::VolatileStatus;
    
    // Apply Aqua Ring volatile status which provides healing each turn
    let aqua_ring_instruction = BattleInstruction::Status(crate::core::instructions::StatusInstruction::ApplyVolatile {
        target: user_position,
        status: VolatileStatus::AquaRing,
        duration: None, // Aqua Ring lasts until switched out
        previous_had_status: false,
        previous_duration: None,
    });
    
    vec![BattleInstructions::new(100.0, vec![aqua_ring_instruction])]
}

/// Apply Shore Up - restores HP, more in sandstorm
pub fn apply_shore_up(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let target_position = if target_positions.is_empty() {
        user_position
    } else {
        target_positions[0]
    };
    
    // Heals more in sandstorm
    let heal_fraction = match state.weather() {
        Weather::Sandstorm => 0.667, // 2/3 HP in sandstorm
        _ => 0.5, // 1/2 HP normally
    };
    
    vec![BattleInstructions::new(100.0, healing_move(state, target_position, heal_fraction, Some(user_position)))]
}