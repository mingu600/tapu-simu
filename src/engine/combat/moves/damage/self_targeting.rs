//! # Self-Targeting Move Effects
//! 
//! This module contains moves that affect the user, including self-damage and self-destruct moves.
//! Consolidates shared functionality like Damp ability checking.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::data::showdown_types::MoveData;
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::engine::combat::moves::apply_generic_effects;

// =============================================================================
// SELF-DAMAGE MOVES (user takes damage but doesn't faint)
// =============================================================================

/// Apply Mind Blown - user takes damage equal to half their max HP
pub fn apply_mind_blown(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check for Damp ability which prevents explosive moves
    if has_damp_ability(state) {
        return vec![BattleInstructions::new(100.0, vec![])];
    }
    
    let user_pokemon = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    // Create combined instructions with both target damage and self-damage
    let target_damage_instructions = apply_generic_effects(state, move_data, user_position, target_positions, generation, branch_on_damage);
    
    let mut combined_instructions = Vec::new();
    
    for mut instruction_set in target_damage_instructions {
        // Calculate self-damage
        let self_damage = (user_pokemon.max_hp / 2).min(user_pokemon.hp); // Don't exceed current HP
        
        // Add self-damage to each instruction set
        instruction_set.instruction_list.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: user_position,
            amount: self_damage,
            previous_hp: Some(user_pokemon.hp),
        }));
        
        // Update affected positions to include user
        if !instruction_set.affected_positions.contains(&user_position) {
            instruction_set.affected_positions.push(user_position);
        }
        
        combined_instructions.push(instruction_set);
    }
    
    // If no target damage instructions were generated, create one with just self-damage
    if combined_instructions.is_empty() {
        let self_damage = (user_pokemon.max_hp / 2).min(user_pokemon.hp);
        combined_instructions.push(BattleInstructions::new(100.0, vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: user_position,
                amount: self_damage,
                previous_hp: Some(user_pokemon.hp),
            }),
        ]));
    }
    
    combined_instructions
}

// =============================================================================
// SELF-DESTRUCT MOVES (user faints after use)
// =============================================================================

/// Apply Explosion - high power, user faints
pub fn apply_explosion(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Check if any Pokemon on the field has Damp ability which prevents explosion moves
    if has_damp_ability(state) {
        // Damp prevents the move entirely - no effect, user doesn't faint
        return vec![BattleInstructions {
            percentage: 100.0,
            instruction_list: vec![],
            affected_positions: vec![],
        }];
    }
    
    let user = match state.get_pokemon_at_position(user_position) {
        Some(pokemon) => pokemon,
        None => return vec![BattleInstructions::new(100.0, vec![])],
    };
    
    let mut all_instructions = Vec::new();
    
    // Filter targets - explosion can't hit Ghost types
    let valid_targets: Vec<BattlePosition> = target_positions
        .iter()
        .filter(|&&target_pos| {
            if let Some(target) = state.get_pokemon_at_position(target_pos) {
                // Check type effectiveness - if it's 0x, the move can't hit
                !is_move_immune("Normal", target, generation)
            } else {
                false
            }
        })
        .copied()
        .collect();
    
    let user_current_hp = user.hp;
    
    if !valid_targets.is_empty() {
        // Apply damage to valid targets first
        let damage_instructions = apply_generic_effects(
            state, move_data, user_position, &valid_targets, generation, branch_on_damage
        );
        
        // Add user fainting to each damage instruction set
        for mut instruction_set in damage_instructions {
            // Add user damage (fainting) - damage equal to current HP
            instruction_set.instruction_list.push(
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: user_position,
                    amount: user_current_hp,
                    previous_hp: Some(user.hp),
                })
            );
            // Update affected positions to include user
            if !instruction_set.affected_positions.contains(&user_position) {
                instruction_set.affected_positions.push(user_position);
            }
            all_instructions.push(instruction_set);
        }
    } else {
        // No valid targets (all Ghost types), but user still faints
        all_instructions.push(BattleInstructions {
            percentage: 100.0,
            instruction_list: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: user_position,
                    amount: user_current_hp,
                    previous_hp: Some(user.hp),
                })
            ],
            affected_positions: vec![user_position],
        });
    }
    
    all_instructions
}

/// Apply Self-Destruct - identical to Explosion but lower power
pub fn apply_self_destruct(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    branch_on_damage: bool,
) -> Vec<BattleInstructions> {
    // Self-Destruct has identical mechanics to Explosion, just different base power
    apply_explosion(state, move_data, user_position, target_positions, generation, branch_on_damage)
}

// =============================================================================
// SHARED HELPER FUNCTIONS
// =============================================================================

/// Check if any Pokemon on the field has the Damp ability
/// This prevents explosive moves from working entirely
fn has_damp_ability(state: &BattleState) -> bool {
    // In Pokemon, Damp prevents explosion/self-destruct moves if any Pokemon on the field has it
    // Check only active Pokemon on both sides
    for side in &state.sides {
        for &active_index in &side.active_pokemon_indices {
            if let Some(index) = active_index {
                if let Some(pokemon) = side.pokemon.get(index) {
                    // Only check active Pokemon that are not fainted and have ability not suppressed
                    if pokemon.hp > 0 && !pokemon.ability_suppressed && 
                       pokemon.ability == crate::types::Abilities::DAMP {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Check if a move is immune against a target due to type effectiveness
fn is_move_immune(
    move_type: &str,
    target: &crate::core::battle_state::Pokemon,
    generation: &GenerationMechanics,
) -> bool {
    let type_chart = TypeChart::get_cached(generation.generation as u8);
    
    let attacking_type = match PokemonType::from_normalized_str(move_type) {
        Some(t) => t,
        None => return false, // Unknown type, assume not immune
    };
    
    // Get target types from the Vec<PokemonType>
    let target_type1 = if let Some(pokemon_type) = target.types.get(0) {
        *pokemon_type
    } else {
        return false; // No types defined
    };
    
    let target_type2 = if let Some(pokemon_type) = target.types.get(1) {
        *pokemon_type
    } else {
        target_type1 // Single type Pokemon
    };
    
    // Check type effectiveness
    let effectiveness = type_chart.calculate_damage_multiplier(
        attacking_type,
        (target_type1, target_type2),
        None, // No tera type for now
        None, // No special move name handling
    );
    
    effectiveness == 0.0
}