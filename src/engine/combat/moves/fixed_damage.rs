//! # Fixed Damage Move Effects
//! 
//! This module contains moves that deal fixed amounts of damage based on specific formulas,
//! such as user level, percentage of HP, etc.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction,
};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;

// =============================================================================
// FIXED DAMAGE MOVES
// =============================================================================

/// Apply Seismic Toss - damage equals user's level
pub fn apply_seismic_toss(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let damage_amount = user.level as i16;
        let mut instructions = Vec::new();
        
        for &target_position in target_positions {
            let instruction = BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: damage_amount,
                previous_hp: None, // Will be filled by state application
            });
            instructions.push(BattleInstructions::new(100.0, vec![instruction]));
        }
        
        if instructions.is_empty() {
            instructions.push(BattleInstructions::new(100.0, vec![]));
        }
        
        instructions
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Night Shade - damage equals user's level
pub fn apply_night_shade(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Same as Seismic Toss
    apply_seismic_toss(state, user_position, target_positions, generation)
}

/// Apply Endeavor - reduces target HP to user's HP
pub fn apply_endeavor(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let mut instructions = Vec::new();
        
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                if target.hp > user.hp {
                    let damage_amount = target.hp - user.hp;
                    let instruction = BattleInstruction::Pokemon(PokemonInstruction::Damage {
                        target: target_position,
                        amount: damage_amount,
                        previous_hp: Some(target.hp),
                    });
                    instructions.push(BattleInstructions::new(100.0, vec![instruction]));
                }
            }
        }
        
        if instructions.is_empty() {
            instructions.push(BattleInstructions::new(100.0, vec![]));
        }
        
        instructions
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Final Gambit - damage equals user's HP, user faints
pub fn apply_final_gambit(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let damage_amount = user.hp;
        let mut instruction_list = Vec::new();
        
        // User faints
        instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Faint {
            target: user_position,
            previous_hp: user.hp, // Store actual HP before fainting
            previous_status: Some(user.status),
        }));
        
        // Deal damage to targets
        for &target_position in target_positions {
            instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: target_position,
                amount: damage_amount,
                previous_hp: None,
            }));
        }
        
        vec![BattleInstructions::new(100.0, instruction_list)]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Nature's Madness - halves target's HP
pub fn apply_natures_madness(
    state: &BattleState,
    _user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            let damage_amount = target.hp / 2;
            if damage_amount > 0 {
                let instruction = BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: target_position,
                    amount: damage_amount,
                    previous_hp: Some(target.hp),
                });
                instructions.push(BattleInstructions::new(100.0, vec![instruction]));
            }
        }
    }
    
    if instructions.is_empty() {
        instructions.push(BattleInstructions::new(100.0, vec![]));
    }
    
    instructions
}

/// Apply Ruination - halves target's HP
pub fn apply_ruination(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Same as Nature's Madness
    apply_natures_madness(state, user_position, target_positions, generation)
}

/// Apply Super Fang - halves target's HP
pub fn apply_super_fang(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Same as Nature's Madness
    apply_natures_madness(state, user_position, target_positions, generation)
}