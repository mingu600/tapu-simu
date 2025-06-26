//! # Counter Move Effects
//! 
//! This module contains counter moves that return damage based on damage received.

use crate::core::battle_state::{BattleState, MoveCategory};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction,
};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;

// =============================================================================
// COUNTER MOVES
// =============================================================================

/// Apply Counter - returns 2x physical damage
pub fn apply_counter(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Get the side that would be targeted by counter (opposing side)
    let target_side = match user_position.side {
        SideReference::SideOne => &state.sides[1],
        SideReference::SideTwo => &state.sides[0],
    };
    
    // Check if damage was dealt and if it was physical
    if target_side.damage_dealt.damage > 0 && 
       target_side.damage_dealt.move_category == MoveCategory::Physical &&
       !target_side.damage_dealt.hit_substitute {
        
        // Counter does 2x the physical damage received
        let counter_damage = (target_side.damage_dealt.damage as f64 * 2.0) as i16;
        
        let mut instruction_list = Vec::new();
        
        // Deal damage to the first target (should be the opposing Pokemon who dealt damage)
        if let Some(&target_position) = target_positions.first() {
            // Check type immunity - Counter can't hit Ghost types
            if let Some(target_pokemon) = state.get_pokemon_at_position(target_position) {
                if target_pokemon.types.contains(&"ghost".to_string()) {
                    return vec![BattleInstructions::new(100.0, vec![])];
                }
            }
            
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: counter_damage,
                previous_hp: None, // Will be filled by state application
            }));
        }
        
        if instruction_list.is_empty() {
            vec![BattleInstructions::new(100.0, vec![])]
        } else {
            vec![BattleInstructions::new(100.0, instruction_list)]
        }
    } else {
        // No physical damage was taken, Counter fails
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Mirror Coat - returns 2x special damage
pub fn apply_mirror_coat(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Get the side that would be targeted by mirror coat (opponent side)
    let target_side = match user_position.side {
        SideReference::SideOne => &state.sides[1],
        SideReference::SideTwo => &state.sides[0],
    };
    
    // Check if damage was dealt and if it was special
    if target_side.damage_dealt.damage > 0 && 
       target_side.damage_dealt.move_category == MoveCategory::Special &&
       !target_side.damage_dealt.hit_substitute {
        
        // Mirror Coat does 2x the special damage received
        let counter_damage = (target_side.damage_dealt.damage as f64 * 2.0) as i16;
        
        let mut instruction_list = Vec::new();
        
        // Deal damage to the first target (should be the opposing Pokemon who dealt damage)
        if let Some(&target_position) = target_positions.first() {
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: counter_damage,
                previous_hp: None, // Will be filled by state application
            }));
        }
        
        if instruction_list.is_empty() {
            vec![BattleInstructions::new(100.0, vec![])]
        } else {
            vec![BattleInstructions::new(100.0, instruction_list)]
        }
    } else {
        // No special damage was taken, Mirror Coat fails
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Comeuppance - returns 1.5x damage taken
pub fn apply_comeuppance(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Get the side that would be targeted by comeuppance (opponent side)
    let target_side = match user_position.side {
        SideReference::SideOne => &state.sides[1],
        SideReference::SideTwo => &state.sides[0],
    };
    
    // Check if damage was dealt (any category)
    if target_side.damage_dealt.damage > 0 && !target_side.damage_dealt.hit_substitute {
        
        // Comeuppance does 1.5x the damage received
        let counter_damage = (target_side.damage_dealt.damage as f64 * 1.5) as i16;
        
        let mut instruction_list = Vec::new();
        
        // Deal damage to the first target (should be the opposing Pokemon who dealt damage)
        if let Some(&target_position) = target_positions.first() {
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: counter_damage,
                previous_hp: None, // Will be filled by state application
            }));
        }
        
        if instruction_list.is_empty() {
            vec![BattleInstructions::new(100.0, vec![])]
        } else {
            vec![BattleInstructions::new(100.0, instruction_list)]
        }
    } else {
        // No damage was taken, Comeuppance fails
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Metal Burst - returns 1.5x damage taken
pub fn apply_metal_burst(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Metal Burst has identical mechanics to Comeuppance
    apply_comeuppance(state, user_position, target_positions, generation)
}