//! # Weather Move Effects
//! 
//! This module contains weather-setting moves that change battlefield conditions.
//! Weather effects influence damage, accuracy, and various move interactions.
//!
//! All moves in this module have been converted to use the new composer system.

use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, BattleInstructions, Weather};
use crate::core::battle_format::BattlePosition;
use crate::generation::GenerationMechanics;
use crate::engine::combat::composers::field_moves::weather_setting_move;

// =============================================================================
// WEATHER SETTING MACRO
// =============================================================================

/// Macro for simple weather-setting moves
macro_rules! weather_move {
    ($func_name:ident, $weather_type:expr) => {
        pub fn $func_name(
            state: &BattleState,
            user_position: BattlePosition,
            _target_positions: &[BattlePosition],
            _generation: &GenerationMechanics,
        ) -> Vec<BattleInstructions> {
            vec![BattleInstructions::new(100.0, weather_setting_move(state, $weather_type, user_position))]
        }
    };
}

// =============================================================================
// WEATHER SETTING MOVES
// =============================================================================

/// Apply Sunny Day - sets sun weather
weather_move!(apply_sunny_day, Weather::Sun);

/// Apply Rain Dance - sets rain weather
weather_move!(apply_rain_dance, Weather::Rain);

/// Apply Sandstorm - sets sandstorm weather
weather_move!(apply_sandstorm, Weather::Sandstorm);

/// Apply Hail - sets hail weather
weather_move!(apply_hail, Weather::Hail);

/// Apply Snow - sets snow weather (Gen 9)
weather_move!(apply_snow, Weather::Snow);

// =============================================================================
// SPECIAL WEATHER MOVES
// =============================================================================

/// Apply Primordial Sea - sets heavy rain (primal weather)
weather_move!(apply_primordial_sea, Weather::HeavyRain);

/// Apply Desolate Land - sets harsh sun (primal weather)
weather_move!(apply_desolate_land, Weather::HarshSun);

/// Apply Delta Stream - sets strong winds (primal weather)
weather_move!(apply_delta_stream, Weather::StrongWinds);

/// Apply Clear Skies - removes all weather
weather_move!(apply_clear_skies, Weather::None);