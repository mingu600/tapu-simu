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
use crate::engine::combat::type_effectiveness::{TypeChart, PokemonType};

// =============================================================================
// FIXED DAMAGE MOVES
// =============================================================================

/// Check if a move is immune against a target due to type effectiveness
fn is_move_immune(
    move_type: &str,
    target: &crate::core::battle_state::Pokemon,
    generation: &GenerationMechanics,
) -> bool {
    let type_chart = TypeChart::new(generation.generation as u8);
    
    let attacking_type = match PokemonType::from_str(move_type) {
        Some(t) => t,
        None => return false, // Unknown type, assume not immune
    };
    
    // Get target types from the Vec<String>
    let target_type1 = if let Some(type_str) = target.types.get(0) {
        match PokemonType::from_str(type_str) {
            Some(t) => t,
            None => return false,
        }
    } else {
        return false; // No types defined
    };
    
    let target_type2 = if let Some(type_str) = target.types.get(1) {
        match PokemonType::from_str(type_str) {
            Some(t) => t,
            None => target_type1, // Fallback to type1 if type2 is invalid
        }
    } else {
        target_type1 // Single type Pokemon
    };
    
    // Convert tera_type if present
    let tera_type = target.tera_type.as_ref().and_then(|t| {
        // t is already a PokemonType from move_choice, need to convert to type_effectiveness::PokemonType
        PokemonType::from_str(&format!("{:?}", t))
    });
    
    // Calculate type effectiveness
    let effectiveness = type_chart.calculate_damage_multiplier(
        attacking_type,
        (target_type1, target_type2),
        tera_type,
        None, // No special move name needed for basic type effectiveness
    );
    
    // Move is immune if effectiveness is 0.0
    effectiveness == 0.0
}

/// Apply Seismic Toss - damage equals user's level
pub fn apply_seismic_toss(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let intended_damage = user.level as i16;
        let mut instructions = Vec::new();
        
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                // Check for type immunity (Seismic Toss is a Fighting-type move)
                if is_move_immune("Fighting", target, generation) {
                    continue; // Skip this target due to immunity
                }
                
                // Cap damage at target's current HP to prevent overkill
                let damage_amount = intended_damage.min(target.hp);
                
                let instruction = BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: target_position,
                    amount: damage_amount,
                    previous_hp: None, // Will be filled by state application
                });
                instructions.push(BattleInstructions::new(100.0, vec![instruction]));
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

/// Apply Night Shade - damage equals user's level
pub fn apply_night_shade(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let damage_amount = user.level as i16;
        let mut instructions = Vec::new();
        
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                // Check for type immunity (Night Shade is a Ghost-type move)
                if is_move_immune("Ghost", target, generation) {
                    continue; // Skip this target due to immunity
                }
                
                let instruction = BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: target_position,
                    amount: damage_amount,
                    previous_hp: None, // Will be filled by state application
                });
                instructions.push(BattleInstructions::new(100.0, vec![instruction]));
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

/// Apply Endeavor - reduces target HP to user's HP
pub fn apply_endeavor(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let mut instructions = Vec::new();
        
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                // Check for type immunity (Endeavor is a Normal-type move)
                if is_move_immune("Normal", target, generation) {
                    continue; // Skip this target due to immunity
                }
                
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
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let damage_amount = user.hp;
        let mut instruction_list = Vec::new();
        
        // Deal damage to targets (only if not immune)
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                // Check for type immunity (Final Gambit is a Fighting-type move)
                if !is_move_immune("Fighting", target, generation) {
                    instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                        target: target_position,
                        amount: damage_amount,
                        previous_hp: None,
                    }));
                }
            }
        }
        
        // User takes damage equal to their HP (this will cause them to faint naturally)
        instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: user_position,
            amount: damage_amount,
            previous_hp: None,
        }));
        
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
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut instructions = Vec::new();
    
    for &target_position in target_positions {
        if let Some(target) = state.get_pokemon_at_position(target_position) {
            // Check for type immunity (Super Fang is a Normal-type move and should be blocked by Ghost types)
            if is_move_immune("Normal", target, generation) {
                continue; // Skip this target due to immunity
            }
            
            let damage_amount = if target.hp == 1 {
                1 // When target has 1 HP, deal 1 damage
            } else {
                target.hp / 2 // Half the target's current HP
            };
            
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