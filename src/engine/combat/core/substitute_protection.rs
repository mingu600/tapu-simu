//! Substitute Protection System
//!
//! This module handles substitute interactions with moves and effects,
//! including which effects are blocked by hitting a substitute.

use crate::core::instructions::{StatusInstruction, StatsInstruction};

/// Result of applying damage to a Pokemon with substitute consideration
#[derive(Debug, Clone, PartialEq)]
pub struct SubstituteDamageResult {
    /// Whether the substitute was hit (and therefore should block secondary effects)
    pub hit_substitute: bool,
    /// Whether the substitute was broken this turn
    pub substitute_broken: bool,
    /// Actual damage dealt to the Pokemon (after substitute absorption)
    pub damage_to_pokemon: i16,
}

/// Types of effects that can be blocked by substitute
#[derive(Debug, Clone, PartialEq)]
pub enum EffectType {
    StatusCondition,
    StatChange,
    VolatileStatus,
    DirectDamage,
    Healing,
    AbilityChange,
    ItemChange,
    TypeChange,
}

/// Determines if a substitute should block a specific effect
pub fn substitute_blocks_effect(effect_type: &EffectType, hit_substitute: bool) -> bool {
    if !hit_substitute {
        return false;
    }
    
    match effect_type {
        // Substitute blocks most secondary effects when hit
        EffectType::StatusCondition => true,
        EffectType::StatChange => true,
        EffectType::VolatileStatus => true,
        
        // Direct effects still apply
        EffectType::DirectDamage => false, // Damage was already handled by substitute
        EffectType::Healing => false,      // Healing can still work
        EffectType::AbilityChange => false, // Some moves like Gastro Acid bypass substitute
        EffectType::ItemChange => false,   // Item manipulation can bypass substitute
        EffectType::TypeChange => false,   // Type changes can bypass substitute
    }
}

/// Categorizes different instruction types for substitute blocking
pub fn get_effect_type_for_status_instruction(instruction: &StatusInstruction) -> EffectType {
    match instruction {
        StatusInstruction::Apply { .. } => EffectType::StatusCondition,
        StatusInstruction::Remove { .. } => EffectType::StatusCondition,
        StatusInstruction::ChangeDuration { .. } => EffectType::StatusCondition,
        StatusInstruction::ApplyVolatile { .. } => EffectType::VolatileStatus,
        StatusInstruction::RemoveVolatile { .. } => EffectType::VolatileStatus,
        StatusInstruction::ChangeVolatileDuration { .. } => EffectType::VolatileStatus,
        _ => EffectType::StatusCondition,
    }
}

// This function is redundant since get_effect_type_for_status_instruction handles both
// status and volatile status instructions. Keeping for API compatibility.
pub fn get_effect_type_for_volatile_instruction(instruction: &StatusInstruction) -> EffectType {
    get_effect_type_for_status_instruction(instruction)
}

pub fn get_effect_type_for_stats_instruction(instruction: &StatsInstruction) -> EffectType {
    EffectType::StatChange
}

/// Checks if a Pokemon should block effects due to substitute being hit
pub fn should_block_effect(
    effect_type: EffectType,
    substitute_damage_result: &SubstituteDamageResult,
) -> bool {
    substitute_blocks_effect(&effect_type, substitute_damage_result.hit_substitute)
}