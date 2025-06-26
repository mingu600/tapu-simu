//! # Hazard Move Effects
//! 
//! This module contains entry hazard moves that set field conditions affecting
//! Pokemon when they switch in. These are critical for competitive play.
//!
//! All moves in this module have been converted to use the new composer system.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::composers::field_moves::hazard_setting_move;
use crate::engine::combat::core::field_system::{HazardType, HazardRemovalType, hazard_removal_move};

// =============================================================================
// HAZARD SETTING MACRO
// =============================================================================

/// Macro for simple hazard-setting moves
macro_rules! hazard_move {
    ($func_name:ident, $hazard_type:expr) => {
        pub fn $func_name(
            state: &BattleState,
            user_position: BattlePosition,
            _target_positions: &[BattlePosition],
            _generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            vec![BattleInstructions::new(100.0, hazard_setting_move(state, user_position, $hazard_type))]
        }
    };
}

// =============================================================================
// HAZARD SETTING MOVES
// =============================================================================

/// Apply Spikes - sets entry hazard that damages grounded Pokemon
hazard_move!(apply_spikes, HazardType::Spikes);

/// Apply Stealth Rock - sets entry hazard based on type effectiveness
hazard_move!(apply_stealth_rock, HazardType::StealthRock);

/// Apply Toxic Spikes - sets entry hazard that poisons switching Pokemon
hazard_move!(apply_toxic_spikes, HazardType::ToxicSpikes);

/// Apply Sticky Web - sets entry hazard that lowers Speed of switching Pokemon
hazard_move!(apply_sticky_web, HazardType::StickyWeb);

