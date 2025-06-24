//! # Screen Move Effects
//! 
//! This module contains defensive screen moves that reduce damage taken
//! by the user's team. These moves are crucial for defensive strategies.
//!
//! All moves in this module have been converted to use the new composer system.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, Weather};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::composers::field_moves::screen_setting_move;
use crate::engine::combat::core::field_system::ScreenType;

// =============================================================================
// SCREEN SETTING MACRO
// =============================================================================

/// Macro for simple screen-setting moves
macro_rules! screen_move {
    ($func_name:ident, $screen_type:expr) => {
        pub fn $func_name(
            state: &BattleState,
            user_position: BattlePosition,
            _target_positions: &[BattlePosition],
            _generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            vec![BattleInstructions::new(100.0, screen_setting_move(state, user_position, $screen_type))]
        }
    };
}

// =============================================================================
// SCREEN SETTING MOVES
// =============================================================================

/// Apply Reflect - reduces physical damage
screen_move!(apply_reflect, ScreenType::Reflect);

/// Apply Light Screen - reduces special damage
screen_move!(apply_light_screen, ScreenType::LightScreen);

/// Apply Aurora Veil - reduces both physical and special damage (requires hail/snow)
pub fn apply_aurora_veil(
    state: &BattleState,
    user_position: BattlePosition,
    _target_positions: &[BattlePosition],
    _generation: &GenerationMechanics,
) -> Vec<BattleInstructions> {
    // Aurora Veil can only be used in hail or snow
    let can_use = matches!(state.weather(), Weather::Hail | Weather::Snow);
    
    if can_use {
        vec![BattleInstructions::new(100.0, screen_setting_move(state, user_position, ScreenType::AuroraVeil))]
    } else {
        // Move fails if not in hail/snow
        vec![BattleInstructions::new(100.0, vec![])]
    }
}

/// Apply Safeguard - prevents status conditions
screen_move!(apply_safeguard, ScreenType::Safeguard);

/// Apply Mist - prevents stat reduction
screen_move!(apply_mist, ScreenType::Mist);

/// Apply Lucky Chant - prevents critical hits against the team
screen_move!(apply_lucky_chant, ScreenType::LuckyChant);