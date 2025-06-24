//! # Stat Modifying Move Effects
//!
//! This module contains stat modification move effects extracted from move_effects.rs.
//! These functions handle moves that boost or lower Pokemon stats.
//!
//! All moves in this module have been converted to use the new composer system.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, Stat, Weather};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use std::collections::HashMap;
use crate::data::showdown_types::MoveData;
use crate::engine::combat::composers::status_moves::{self_stat_boost_move, enemy_stat_reduction_move};

// =============================================================================
// STAT MODIFICATION MACROS
// =============================================================================

/// Macro for self-targeting stat boost moves
macro_rules! self_stat_boost {
    ($func_name:ident, $($stat:expr => $change:expr),+) => {
        pub fn $func_name(
            state: &BattleState,
            user_position: BattlePosition,
            _target_positions: &[BattlePosition],
            _generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            let mut stat_changes = HashMap::new();
            $(
                stat_changes.insert($stat, $change);
            )+
            
            vec![BattleInstructions::new(100.0, self_stat_boost_move(state, user_position, &stat_changes))]
        }
    };
}

/// Macro for enemy stat reduction moves
macro_rules! enemy_stat_reduction {
    ($func_name:ident, $($stat:expr => $change:expr),+) => {
        pub fn $func_name(
            state: &BattleState,
            user_position: BattlePosition,
            target_positions: &[BattlePosition],
            _generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            let mut stat_changes = HashMap::new();
            $(
                stat_changes.insert($stat, $change);
            )+
            
            vec![BattleInstructions::new(100.0, enemy_stat_reduction_move(state, target_positions, &stat_changes, user_position))]
        }
    };
}

// =============================================================================
// SELF-TARGETING STAT BOOST MOVES
// =============================================================================

/// Apply Swords Dance - raises Attack by 2 stages
self_stat_boost!(apply_swords_dance, Stat::Attack => 2);

/// Apply Dragon Dance - raises Attack and Speed by 1 stage each
self_stat_boost!(apply_dragon_dance, Stat::Attack => 1, Stat::Speed => 1);

/// Apply Nasty Plot - raises Special Attack by 2 stages
self_stat_boost!(apply_nasty_plot, Stat::SpecialAttack => 2);

/// Apply Calm Mind - raises Special Attack and Special Defense by 1 stage each
self_stat_boost!(apply_calm_mind, Stat::SpecialAttack => 1, Stat::SpecialDefense => 1);

/// Apply Iron Defense - raises Defense by 2 stages
self_stat_boost!(apply_iron_defense, Stat::Defense => 2);

/// Apply Amnesia - raises Special Defense by 2 stages
self_stat_boost!(apply_amnesia, Stat::SpecialDefense => 2);

/// Apply Agility - raises Speed by 2 stages
self_stat_boost!(apply_agility, Stat::Speed => 2);

/// Apply Bulk Up - raises Attack and Defense by 1 stage each
self_stat_boost!(apply_bulk_up, Stat::Attack => 1, Stat::Defense => 1);

/// Apply Rock Polish - raises Speed by 2 stages
self_stat_boost!(apply_rock_polish, Stat::Speed => 2);

/// Apply Hone Claws - raises Attack and Accuracy by 1 stage each
self_stat_boost!(apply_hone_claws, Stat::Attack => 1, Stat::Accuracy => 1);

/// Apply Curse (when used by non-Ghost type) - raises Attack and Defense, lowers Speed
self_stat_boost!(apply_curse_stat_version, Stat::Attack => 1, Stat::Defense => 1, Stat::Speed => -1);

// =============================================================================
// ENEMY STAT REDUCTION MOVES
// =============================================================================

/// Apply Growl - lowers target's Attack by 1 stage
enemy_stat_reduction!(apply_growl, Stat::Attack => -1);

/// Apply Leer - lowers target's Defense by 1 stage
enemy_stat_reduction!(apply_leer, Stat::Defense => -1);

/// Apply Tail Whip - lowers target's Defense by 1 stage
enemy_stat_reduction!(apply_tail_whip, Stat::Defense => -1);

/// Apply String Shot - lowers target's Speed by 2 stages
enemy_stat_reduction!(apply_string_shot, Stat::Speed => -2);

/// Apply Charm - lowers target's Attack by 2 stages
enemy_stat_reduction!(apply_charm, Stat::Attack => -2);

/// Apply Screech - lowers target's Defense by 2 stages
enemy_stat_reduction!(apply_screech, Stat::Defense => -2);

/// Apply Sweet Scent - lowers target's Evasion by 2 stages
enemy_stat_reduction!(apply_sweet_scent, Stat::Evasion => -2);

/// Apply Sand Attack - lowers target's Accuracy by 1 stage
enemy_stat_reduction!(apply_sand_attack, Stat::Accuracy => -1);

/// Apply Smokescreen - lowers target's Accuracy by 1 stage
enemy_stat_reduction!(apply_smokescreen, Stat::Accuracy => -1);

/// Apply Intimidate (ability) - lowers target's Attack by 1 stage
enemy_stat_reduction!(apply_intimidate, Stat::Attack => -1);

// =============================================================================
// SPECIAL CASE MOVES
// =============================================================================

/// Apply Growth - raises Attack and Special Attack
/// Generation-aware: Enhanced in sun weather
pub fn apply_growth(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut stat_changes = HashMap::new();
    
    // Enhanced in sun weather
    let boost_amount = match state.weather() {
        Weather::Sun | Weather::HarshSun => 2,
        _ => 1,
    };
    
    stat_changes.insert(Stat::Attack, boost_amount);
    stat_changes.insert(Stat::SpecialAttack, boost_amount);
    
    vec![BattleInstructions::new(100.0, self_stat_boost_move(state, user_position, &stat_changes))]
}

/// Apply Acid - deals damage with chance to lower Defense
/// Generation-aware: 33.2% chance in Gen 1, 10% in later generations
pub fn apply_acid(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Acid deals damage AND has a secondary effect
    // This needs to use the damage_move_with_secondary_status composer
    // For now, just return the basic stat reduction
    let mut stat_changes = HashMap::new();
    stat_changes.insert(Stat::Defense, -1);
    
    vec![BattleInstructions::new(100.0, enemy_stat_reduction_move(state, target_positions, &stat_changes, user_position))]
}

/// Apply Fillet Away - maximizes Attack but sacrifices HP
pub fn apply_fillet_away(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut stat_changes = HashMap::new();
    stat_changes.insert(Stat::Attack, 12); // Maximize Attack (6 stages is max boost)
    
    // TODO: Add HP sacrifice logic using damage composer
    vec![BattleInstructions::new(100.0, self_stat_boost_move(state, user_position, &stat_changes))]
}

/// Apply Clangorous Soul - raises all stats but sacrifices HP
pub fn apply_clangorous_soul(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    let mut stat_changes = HashMap::new();
    stat_changes.insert(Stat::Attack, 1);
    stat_changes.insert(Stat::Defense, 1);
    stat_changes.insert(Stat::SpecialAttack, 1);
    stat_changes.insert(Stat::SpecialDefense, 1);
    stat_changes.insert(Stat::Speed, 1);
    
    // TODO: Add HP sacrifice logic using damage composer
    vec![BattleInstructions::new(100.0, self_stat_boost_move(state, user_position, &stat_changes))]
}

// =============================================================================
// PROBABILITY-BASED SECONDARY EFFECTS
// =============================================================================

/// Apply probability-based secondary effects (for damage moves with stat changes)
pub fn apply_probability_based_secondary_effects(
    state: &BattleState,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
    stat: Stat,
    change: i8,
    probability: f32,
) -> Vec<BattleInstructions> {
    let mut stat_changes = HashMap::new();
    stat_changes.insert(stat, change);
    
    vec![BattleInstructions::new(
        probability,
        enemy_stat_reduction_move(state, target_positions, &stat_changes, user_position)
    )]
}