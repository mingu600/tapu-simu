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
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::engine::combat::composers::damage_moves::{simple_damage_move, DamageModifiers};
use crate::data::showdown_types::MoveData;

// =============================================================================
// FIXED DAMAGE MOVES
// =============================================================================

/// Fixed damage calculation function type
type FixedDamageCalculator = fn(&crate::core::battle_state::Pokemon, &crate::core::battle_state::Pokemon) -> i16;

/// Apply fixed damage move using infrastructure
fn apply_fixed_damage_move(
    state: &BattleState,
    move_type: PokemonType,
    damage_calculator: FixedDamageCalculator,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let type_chart = TypeChart::get_cached(generation.generation as u8);
        let mut instructions = Vec::new();
        
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                // Check for type immunity using infrastructure
                let target_type1 = target.types.get(0).copied().unwrap_or(PokemonType::Normal);
                let target_type2 = target.types.get(1).copied().unwrap_or(target_type1);
                let tera_type = target.tera_type;
                
                let effectiveness = type_chart.calculate_damage_multiplier(
                    move_type,
                    (target_type1, target_type2),
                    tera_type,
                    None,
                );
                
                // Skip if immune
                if effectiveness == 0.0 {
                    continue;
                }
                
                // Calculate fixed damage using the provided calculator
                let damage_amount = damage_calculator(user, target);
                
                // Cap damage to prevent overkill
                let final_damage = damage_amount.min(target.hp);
                
                if final_damage > 0 {
                    let instruction = BattleInstruction::Pokemon(PokemonInstruction::Damage {
                        target: target_position,
                        amount: final_damage,
                        previous_hp: None,
                    });
                    instructions.push(BattleInstructions::new_with_positions(
                        100.0, 
                        vec![instruction], 
                        vec![target_position]
                    ));
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

/// Apply Seismic Toss - damage equals user's level
pub fn apply_seismic_toss(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_fixed_damage_move(
        state,
        PokemonType::Fighting,
        |user, _target| user.level as i16,
        user_position,
        target_positions,
        generation,
    )
}

/// Apply Night Shade - damage equals user's level
pub fn apply_night_shade(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_fixed_damage_move(
        state,
        PokemonType::Ghost,
        |user, _target| user.level as i16,
        user_position,
        target_positions,
        generation,
    )
}

/// Apply Endeavor - reduces target HP to user's HP
pub fn apply_endeavor(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_fixed_damage_move(
        state,
        PokemonType::Normal,
        |user, target| {
            if target.hp > user.hp {
                target.hp - user.hp
            } else {
                0
            }
        },
        user_position,
        target_positions,
        generation,
    )
}

/// Apply Final Gambit - damage equals user's HP, user faints
pub fn apply_final_gambit(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    if let Some(user) = state.get_pokemon_at_position(user_position) {
        let type_chart = TypeChart::get_cached(generation.generation as u8);
        let user_hp = user.hp;
        let mut instruction_list = Vec::new();
        let mut affected_positions = vec![user_position]; // User always takes damage
        
        // Deal damage to targets (only if not immune)
        for &target_position in target_positions {
            if let Some(target) = state.get_pokemon_at_position(target_position) {
                // Check for type immunity using infrastructure
                let target_type1 = target.types.get(0).copied().unwrap_or(PokemonType::Normal);
                let target_type2 = target.types.get(1).copied().unwrap_or(target_type1);
                let tera_type = target.tera_type;
                
                let effectiveness = type_chart.calculate_damage_multiplier(
                    PokemonType::Fighting,
                    (target_type1, target_type2),
                    tera_type,
                    None,
                );
                
                if effectiveness != 0.0 {
                    let final_damage = user_hp.min(target.hp);
                    instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                        target: target_position,
                        amount: final_damage,
                        previous_hp: None,
                    }));
                    affected_positions.push(target_position);
                }
            }
        }
        
        // User takes damage equal to their HP (this will cause them to faint naturally)
        instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: user_position,
            amount: user_hp,
            previous_hp: None,
        }));
        
        vec![BattleInstructions::new_with_positions(100.0, instruction_list, affected_positions)]
    } else {
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Nature's Madness - halves target's HP
pub fn apply_natures_madness(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_fixed_damage_move(
        state,
        PokemonType::Fairy, // Nature's Madness is a Fairy-type move
        |_user, target| {
            if target.hp == 1 {
                1 // When target has 1 HP, deal 1 damage
            } else {
                target.hp / 2 // Half the target's current HP
            }
        },
        user_position,
        target_positions,
        generation,
    )
}

/// Apply Ruination - halves target's HP
pub fn apply_ruination(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_fixed_damage_move(
        state,
        PokemonType::Dark, // Ruination is a Dark-type move
        |_user, target| {
            if target.hp == 1 {
                1 // When target has 1 HP, deal 1 damage
            } else {
                target.hp / 2 // Half the target's current HP
            }
        },
        user_position,
        target_positions,
        generation,
    )
}

/// Apply Super Fang - halves target's HP
pub fn apply_super_fang(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_fixed_damage_move(
        state,
        PokemonType::Normal, // Super Fang is a Normal-type move
        |_user, target| {
            if target.hp == 1 {
                1 // When target has 1 HP, deal 1 damage
            } else {
                target.hp / 2 // Half the target's current HP
            }
        },
        user_position,
        target_positions,
        generation,
    )
}