//! # Recoil Move Effects
//! 
//! This module handles recoil move effects where the user takes damage
//! based on a percentage of damage dealt to the target.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;

/// Apply recoil move effects - now handled automatically by instruction generator
/// This function is kept for compatibility but recoil is now handled via PS data
pub fn apply_recoil_move(
    _state: &BattleState,
    _user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
    _recoil_percentage: i16,
) -> Vec<BattleInstructions> {
    // Recoil is now handled automatically in the instruction generator
    // based on PS move data, so we just return empty instructions
    vec![BattleInstructions::new(100.0, vec![])]
}

/// Apply Double Edge - deals recoil damage (33% of damage dealt)
pub fn apply_double_edge(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 33)
}

/// Apply Take Down - deals recoil damage (25% of damage dealt)
pub fn apply_take_down(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 25)
}

/// Apply Submission - deals recoil damage (25% of damage dealt)
pub fn apply_submission(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 25)
}

/// Apply Volt Tackle - deals recoil damage (33% of damage dealt)
pub fn apply_volt_tackle(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 33)
}

/// Apply Flare Blitz - deals recoil damage (33% of damage dealt)
pub fn apply_flare_blitz(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 33)
}

/// Apply Brave Bird - deals recoil damage (33% of damage dealt)
pub fn apply_brave_bird(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 33)
}

/// Apply Wild Charge - deals recoil damage (25% of damage dealt)
pub fn apply_wild_charge(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 25)
}

/// Apply Head Smash - deals recoil damage (50% of damage dealt)
pub fn apply_head_smash(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    apply_recoil_move(state, user_position, target_positions, generation, 50)
}

/// Types of damage-based effects
#[derive(Debug, Clone, PartialEq)]
pub enum DamageBasedEffectType {
    Recoil,  // User takes damage
    Drain,   // User heals
}

/// A damage-based effect that will be calculated after damage is determined
#[derive(Debug, Clone, PartialEq)]
pub struct DamageBasedEffect {
    pub effect_type: DamageBasedEffectType,
    pub user_position: BattlePosition,
    pub percentage: i16,
}

/// Create a damage-based effect instruction for moves like recoil and drain
/// This creates an instruction template that will be filled in with actual values
/// during damage calculation
pub fn create_damage_based_effect(
    effect_type: DamageBasedEffectType,
    user_position: BattlePosition,
    percentage: i16,
) -> DamageBasedEffect {
    DamageBasedEffect {
        effect_type,
        user_position,
        percentage,
    }
}

/// Apply secondary effects that depend on damage dealt
/// This function would be called by the damage calculation system
/// after determining the actual damage amount
pub fn apply_damage_based_secondary_effects(
    state: &BattleState,
    damage_dealt: i16,
    effects: &[DamageBasedEffect],
    instructions: &mut Vec<BattleInstruction>,
) {
    for effect in effects {
        match effect.effect_type {
            DamageBasedEffectType::Recoil => {
                let recoil_amount = (damage_dealt * effect.percentage) / 100;
                if recoil_amount > 0 {
                    let previous_hp = state.get_pokemon_at_position(effect.user_position).map(|p| p.hp).unwrap_or(0);
                    instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Damage {
                        target: effect.user_position,
                        amount: recoil_amount,
                        previous_hp: Some(previous_hp),
                    }));
                }
            }
            DamageBasedEffectType::Drain => {
                let heal_amount = (damage_dealt * effect.percentage) / 100;
                if heal_amount > 0 {
                    let previous_hp = state.get_pokemon_at_position(effect.user_position).map(|p| p.hp).unwrap_or(0);
                    instructions.push(BattleInstruction::Pokemon(PokemonInstruction::Heal {
                        target: effect.user_position,
                        amount: heal_amount,
                        previous_hp: Some(previous_hp),
                    }));
                }
            }
        }
    }
}