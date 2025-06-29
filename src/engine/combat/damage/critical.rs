//! Critical hit probability calculation system
//!
//! This module handles all aspects of critical hit probability calculation
//! across different Pokemon generations, including generation-specific
//! formulas and stage-based systems.

use crate::constants::moves::{
    GEN1_CRIT_RATE_DIVISOR,
    // Critical hit rates
    GEN1_CRIT_SPEED_DIVISOR,
    GEN1_HIGH_CRIT_MOVES,
    GEN1_HIGH_CRIT_MULTIPLIER,
    GEN1_MAX_CRIT_RATE,
    GEN2_BASE_CRIT_RATE,
    GEN2_CRIT_STAGES,
    GEN2_HIGH_CRIT_MOVES,
    GEN2_HIGH_CRIT_RATE,
    GEN3_5_CRIT_STAGES,
    GEN6_CRIT_STAGES,
    GEN7_9_CRIT_STAGES,
    GUARANTEED_CRIT_MOVES,
    // Move lists
    HIGH_CRIT_MOVES,
};
use crate::core::battle_state::Pokemon;
use crate::data::showdown_types::MoveData;
use crate::utils::normalize_name;

/// Calculate critical hit probability with generation-specific stage system
///
/// Uses the official critical hit stage table for accurate calculation across
/// all Pokemon generations. Handles generation-specific mechanics including
/// abilities that prevent critical hits and moves that guarantee critical hits.
///
/// ## Parameters
/// - `attacker`: The Pokemon using the move
/// - `defender`: The Pokemon receiving the move
/// - `move_data`: Complete move information
/// - `generation`: The Pokemon generation for mechanics
///
/// ## Returns
/// The probability of a critical hit as a float (0.0 to 1.0)
pub fn critical_hit_probability(
    attacker: &Pokemon,
    defender: &Pokemon,
    move_data: &MoveData,
    generation: crate::generation::Generation,
) -> f32 {
    // Check for abilities that prevent critical hits (Gen 3+)
    if matches!(
        generation,
        crate::generation::Generation::Gen3
            | crate::generation::Generation::Gen4
            | crate::generation::Generation::Gen5
            | crate::generation::Generation::Gen6
            | crate::generation::Generation::Gen7
            | crate::generation::Generation::Gen8
            | crate::generation::Generation::Gen9
    ) {
        let defender_ability = &defender.ability;
        if *defender_ability == crate::types::Abilities::SHELLARMOR
            || *defender_ability == crate::types::Abilities::BATTLEARMOR
        {
            return 0.0; // No critical hit possible
        }
    }

    // Check for guaranteed critical hit moves first (applies to certain generations)
    let move_name = move_data.name.as_str();
    if GUARANTEED_CRIT_MOVES.contains(&move_name) {
        return 1.0; // Always critical hit
    }

    // Generation-specific critical hit calculation
    match generation {
        crate::generation::Generation::Gen1 => {
            return critical_hit_probability_gen1(attacker, move_data);
        }
        crate::generation::Generation::Gen2 => {
            return critical_hit_probability_gen2(attacker, move_data);
        }
        _ => {
            // Gen 3+ uses stage-based system
        }
    }

    // Calculate critical hit stage for Gen 3+
    let mut crit_stage = 0;

    // High critical hit ratio moves increase stage by 1
    if HIGH_CRIT_MOVES.contains(&move_name) {
        crit_stage += 1;
    }

    // Ability modifiers (Gen 3+)
    match attacker.ability {
        crate::types::Abilities::SUPERLUCK => {
            crit_stage += 1;
        }
        _ => {}
    }

    // Item modifiers
    if let Some(item) = &attacker.item {
        match item {
            crate::types::Items::SCOPELENS => {
                crit_stage += 1;
            }
            crate::types::Items::RAZORCLAW => {
                crit_stage += 1;
            }
            crate::types::Items::LUCKYPUNCH => {
                if attacker.species == crate::types::PokemonName::CHANSEY {
                    crit_stage += 2;
                }
            }
            crate::types::Items::LEEK | crate::types::Items::STICK => {
                if attacker.species == crate::types::PokemonName::FARFETCHD
                    || attacker.species == crate::types::PokemonName::SIRFETCHD
                {
                    crit_stage += 2;
                }
            }
            _ => {}
        }
    }

    // Convert stage to probability using generation-specific table
    calculate_crit_rate_from_stage(crit_stage, generation)
}

/// Calculate Gen 1 critical hit probability based on base Speed
///
/// Gen 1 uses a unique Speed-based formula:
/// - Normal moves: floor(base_speed / 2) / 256
/// - High crit moves: min(8 * floor(base_speed / 2), 255) / 256
///
/// ## Parameters
/// - `attacker`: The Pokemon using the move
/// - `move_data`: Move information to check for high crit ratio
///
/// ## Returns
/// The critical hit probability for Gen 1
pub fn critical_hit_probability_gen1(attacker: &Pokemon, move_data: &MoveData) -> f32 {
    // Get the base Speed stat for critical hit calculation
    // In Gen 1, we use the base species stat, not the effective stat
    let base_speed = attacker.base_stats.speed;

    // Get move name for comparison
    let move_name = move_data.name.as_str();

    // Calculate critical hit rate using the correct Gen 1 formula
    let crit_rate = if GEN1_HIGH_CRIT_MOVES.contains(&move_name) {
        // High crit moves: min(8 * floor(base_speed / 2), 255)
        let rate_numerator = std::cmp::min(
            GEN1_HIGH_CRIT_MULTIPLIER * (base_speed / GEN1_CRIT_SPEED_DIVISOR),
            255,
        );
        rate_numerator as f32 / GEN1_CRIT_RATE_DIVISOR
    } else {
        // Normal moves: floor(base_speed / 2) / 256
        let rate_numerator = base_speed / GEN1_CRIT_SPEED_DIVISOR;
        rate_numerator as f32 / GEN1_CRIT_RATE_DIVISOR
    };

    // Cap at 255/256 to match Gen 1 behavior
    let final_rate = crit_rate.min(GEN1_MAX_CRIT_RATE);
    final_rate
}

/// Calculate Gen 2 critical hit probability
///
/// Gen 2 uses fixed stages:
/// - Base rate: 17/256 (~6.64%)
/// - High crit moves: +1 stage (1/8 = 12.5%)
///
/// ## Parameters
/// - `attacker`: The Pokemon using the move
/// - `move_data`: Move information to check for high crit ratio
///
/// ## Returns
/// The critical hit probability for Gen 2
pub fn critical_hit_probability_gen2(attacker: &Pokemon, move_data: &MoveData) -> f32 {
    // Get move name for comparison
    let move_name = move_data.name.as_str();

    // Gen 2 uses fixed stages, not multipliers
    if GEN2_HIGH_CRIT_MOVES.contains(&move_name) {
        // High crit rate: +1 stage = 1/8 = 12.5%
        GEN2_HIGH_CRIT_RATE
    } else {
        // Base crit rate: 17/256 ≈ 6.64%
        GEN2_BASE_CRIT_RATE
    }
}

/// Convert critical hit stage to probability using generation-specific table
///
/// Based on official Pokemon critical hit probability table.
/// Different generations use different stage-to-probability mappings.
///
/// ## Parameters
/// - `stage`: The critical hit stage (0 = base, higher = better)
/// - `generation`: The Pokemon generation for mechanics
///
/// ## Returns
/// The probability corresponding to the stage for that generation
fn calculate_crit_rate_from_stage(stage: i32, generation: crate::generation::Generation) -> f32 {
    let stage_index = stage.max(0) as usize;

    match generation {
        crate::generation::Generation::Gen2 => {
            // Gen 2 uses different formula - handled separately
            GEN2_CRIT_STAGES
                .get(stage_index)
                .copied()
                .unwrap_or(GEN2_CRIT_STAGES.last().copied().unwrap_or(0.5))
        }
        crate::generation::Generation::Gen3
        | crate::generation::Generation::Gen4
        | crate::generation::Generation::Gen5 => {
            // Gen 3-5
            GEN3_5_CRIT_STAGES
                .get(stage_index)
                .copied()
                .unwrap_or(GEN3_5_CRIT_STAGES.last().copied().unwrap_or(0.5))
        }
        crate::generation::Generation::Gen6 => {
            // Gen 6
            GEN6_CRIT_STAGES
                .get(stage_index)
                .copied()
                .unwrap_or(GEN6_CRIT_STAGES.last().copied().unwrap_or(0.5))
        }
        crate::generation::Generation::Gen7
        | crate::generation::Generation::Gen8
        | crate::generation::Generation::Gen9 => {
            // Gen 7-9
            GEN7_9_CRIT_STAGES
                .get(stage_index)
                .copied()
                .unwrap_or(GEN7_9_CRIT_STAGES.last().copied().unwrap_or(0.5))
        }
        _ => {
            // Fallback - shouldn't reach here for Gen 1 or Gen 2
            GEN7_9_CRIT_STAGES[0]
        }
    }
}
