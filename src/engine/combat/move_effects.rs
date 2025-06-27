//! # Move Effects
//! 
//! This module handles special move effects and their implementation with generation awareness.
//! The implementation has been split into focused modules for better maintainability.
//!
//! ## Generation Awareness
//! 
//! All move effects are generation-aware, allowing for proper implementation of mechanics
//! that changed between generations. This includes:
//! - Type immunities (e.g., Electric types immune to paralysis in Gen 6+)
//! - Move behavior changes (e.g., powder moves vs Grass types in Gen 6+)
//! - Status effect mechanics (e.g., burn reducing physical attack)
//! - Accuracy and effect chances that varied by generation

// Re-export the new move modules
pub use crate::engine::combat::moves::*;

use crate::core::battle_state::{Pokemon, MoveCategory};
use crate::core::battle_state::BattleState;
use crate::core::instructions::{PokemonStatus, VolatileStatus, Stat, Weather, SideCondition, Terrain};
use crate::data::showdown_types::MoveData;
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction,
    FieldInstruction, StatsInstruction,
};
use crate::core::battle_format::{BattlePosition, SideReference};
use crate::generation::GenerationMechanics;
use crate::engine::combat::type_effectiveness::TypeChart;
use crate::types::PokemonType;
use crate::engine::combat::moves::MoveContext;
use crate::types::{BattleError, BattleResult};
use std::collections::HashMap;


/// Main move effect dispatcher - delegates to the new module system
pub fn apply_move_effects(
    state: &BattleState,
    move_data: &MoveData,
    user_position: BattlePosition,
    target_positions: &[BattlePosition],
    generation: &GenerationMechanics,
    context: &MoveContext,
    repository: &crate::data::GameDataRepository,
) -> Vec<BattleInstructions> {
    // Delegate to the new modular system
    crate::engine::combat::moves::apply_move_effects(
        state, move_data, user_position, target_positions, generation, context, repository, false
    ).unwrap_or_else(|_| vec![])
}

/// Helper function to check if a move is super effective against a target
pub fn is_super_effective(move_type: &str, target: &Pokemon, generation: &GenerationMechanics) -> bool {
    crate::engine::combat::moves::is_super_effective(move_type, target, generation)
}


// This file now serves as a dispatcher to the modular move system
// All actual implementations are in the categorized modules under moves/


// Add more wrapper functions as needed for backward compatibility
// These can be gradually migrated to the new module system