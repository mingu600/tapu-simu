//! Field effect move composers for common patterns
//!
//! This module provides composer functions for common field effect patterns,
//! building on the core field system to create reusable move implementations.

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::BattleState;
use crate::core::instructions::{BattleInstruction, FieldInstruction, Terrain, Weather};
use super::super::core::field_system::{
    weather_move_with_extension, terrain_move_with_extension, screen_move, hazard_move,
    hazard_removal_move, ScreenType, HazardType, HazardRemovalType,
};

/// Weather-setting move with proper duration and extension handling
pub fn weather_setting_move(
    state: &BattleState,
    weather: Weather,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    weather_move_with_extension(state, weather, user_position)
}

/// Terrain-setting move with proper duration and extension handling
pub fn terrain_setting_move(
    state: &BattleState,
    terrain: Terrain,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    terrain_move_with_extension(state, terrain, user_position)
}

/// Screen move (Reflect, Light Screen, Aurora Veil)
pub fn screen_setting_move(
    state: &BattleState,
    user_position: BattlePosition,
    screen_type: ScreenType,
) -> Vec<BattleInstruction> {
    let duration = Some(5); // Standard screen duration
    screen_move(state, user_position, screen_type, duration)
}

/// Hazard-setting move
pub fn hazard_setting_move(
    state: &BattleState,
    user_position: BattlePosition,
    hazard_type: HazardType,
) -> Vec<BattleInstruction> {
    hazard_move(state, user_position, hazard_type)
}

/// Hazard removal move
pub fn hazard_clearing_move(
    state: &BattleState,
    user_position: BattlePosition,
    removal_type: HazardRemovalType,
) -> Vec<BattleInstruction> {
    hazard_removal_move(state, user_position, removal_type)
}

// Specific weather moves
pub fn sunny_day_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    weather_setting_move(state, Weather::Sun, user_position)
}

pub fn rain_dance_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    weather_setting_move(state, Weather::Rain, user_position)
}

pub fn sandstorm_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    weather_setting_move(state, Weather::Sand, user_position)
}

pub fn hail_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    weather_setting_move(state, Weather::Hail, user_position)
}

pub fn snow_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    weather_setting_move(state, Weather::Snow, user_position)
}

// Specific terrain moves
pub fn electric_terrain_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    terrain_setting_move(state, Terrain::Electric, user_position)
}

pub fn grassy_terrain_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    terrain_setting_move(state, Terrain::Grassy, user_position)
}

pub fn misty_terrain_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    terrain_setting_move(state, Terrain::Misty, user_position)
}

pub fn psychic_terrain_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    terrain_setting_move(state, Terrain::Psychic, user_position)
}

// Specific screen moves
pub fn reflect_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    screen_setting_move(state, user_position, ScreenType::Reflect)
}

pub fn light_screen_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    screen_setting_move(state, user_position, ScreenType::LightScreen)
}

pub fn aurora_veil_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    // Aurora Veil requires hail or snow
    let can_use = matches!(
        state.field.weather.condition,
        Weather::Hail | Weather::Snow
    );

    if can_use {
        screen_setting_move(state, user_position, ScreenType::AuroraVeil)
    } else {
        vec![BattleInstruction::Field(FieldInstruction::Message {
            message: "But it failed!".to_string(),
            affected_positions: vec![],
        })]
    }
}

// Specific hazard moves
pub fn spikes_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    hazard_setting_move(state, user_position, HazardType::Spikes)
}

pub fn toxic_spikes_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    hazard_setting_move(state, user_position, HazardType::ToxicSpikes)
}

pub fn stealth_rock_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    hazard_setting_move(state, user_position, HazardType::StealthRock)
}

pub fn sticky_web_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    hazard_setting_move(state, user_position, HazardType::StickyWeb)
}

// Specific hazard removal moves
pub fn rapid_spin_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    hazard_clearing_move(state, user_position, HazardRemovalType::RapidSpin)
}

pub fn defog_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    hazard_clearing_move(state, user_position, HazardRemovalType::Defog)
}

pub fn tidy_up_move(
    state: &BattleState,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    hazard_clearing_move(state, user_position, HazardRemovalType::TidyUp)
}

/// Gravity move (affects field globally)
pub fn gravity_move(
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    vec![BattleInstruction::Field(FieldInstruction::Message {
        message: format!("Gravity was intensified by {:?}!", user_position),
        affected_positions: vec![],
    })]
}

/// Trick Room move (affects field globally)
pub fn trick_room_move(
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    vec![BattleInstruction::Field(FieldInstruction::Message {
        message: format!("Trick Room was activated by {:?}!", user_position),
        affected_positions: vec![],
    })]
}

/// Magic Room move (affects field globally)
pub fn magic_room_move(
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    vec![BattleInstruction::Field(FieldInstruction::Message {
        message: format!("Magic Room was activated by {:?}!", user_position),
        affected_positions: vec![],
    })]
}

/// Wonder Room move (affects field globally)
pub fn wonder_room_move(
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    vec![BattleInstruction::Field(FieldInstruction::Message {
        message: format!("Wonder Room was activated by {:?}!", user_position),
        affected_positions: vec![],
    })]
}