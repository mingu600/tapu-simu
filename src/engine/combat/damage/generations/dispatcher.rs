//! Generation dispatcher for damage calculation
//!
//! This module provides the main entry point for generation-aware damage
//! calculation, dispatching to the appropriate generation-specific calculator.

use crate::engine::combat::damage_context::{DamageContext, DamageResult};
use super::super::types::DamageRolls;

/// Calculate damage using focused DamageContext
/// This is the primary damage calculation function with generation dispatch
pub fn calculate_damage(context: &DamageContext, damage_rolls: DamageRolls) -> DamageResult {
    // Use generation-specific damage calculation
    match context.format.format.generation {
        crate::generation::Generation::Gen1 => {
            super::gen1::calculate_damage_gen1(context, damage_rolls)
        }
        crate::generation::Generation::Gen2 => {
            super::gen2::calculate_damage_gen2(context, damage_rolls)
        }
        crate::generation::Generation::Gen3 => {
            super::gen3::calculate_damage_gen3(context, damage_rolls)
        }
        crate::generation::Generation::Gen4 => {
            super::gen4::calculate_damage_gen4(context, damage_rolls)
        }
        crate::generation::Generation::Gen5 | crate::generation::Generation::Gen6 => {
            super::gen56::calculate_damage_gen56(context, damage_rolls)
        }
        _ => {
            // Gen 7-9 calculation (modern getFinalDamage with pokeRound)
            super::modern::calculate_damage_modern_gen789(context, damage_rolls)
        }
    }
}