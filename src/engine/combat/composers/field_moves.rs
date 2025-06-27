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
///
/// This composer handles all standard weather-setting moves with automatic duration
/// management and item-based extensions.
///
/// # What it handles automatically:
/// - Base weather duration (5 turns)
/// - Item-based duration extension (Heat Rock, Damp Rock, Smooth Rock, Icy Rock)
/// - Proper source tracking for the weather condition
/// - Weather replacement (overwrites existing weather)
/// - Instruction generation for battle state application
///
/// # When to use:
/// - Standard weather moves (Sunny Day, Rain Dance, Sandstorm, Hail, Snow)
/// - Simple weather setting without additional effects
/// - When you want automatic item extension handling
///
/// # When NOT to use:
/// - Moves with custom weather durations (use `weather_move` directly)
/// - Moves that set weather conditionally (implement custom logic)
/// - Moves with additional effects beyond weather (use in combination with other composers)
/// - Primordial weather conditions (use dedicated primordial weather functions)
///
/// # Example usage patterns:
/// ```rust
/// // Basic weather move
/// pub fn sunny_day_move(
///     state: &BattleState,
///     user_position: BattlePosition,
/// ) -> Vec<BattleInstruction> {
///     weather_setting_move(state, Weather::Sun, user_position)
/// }
///
/// // For moves with additional effects, combine with other composers:
/// pub fn hurricane_move(
///     state: &BattleState,
///     user_position: BattlePosition,
///     target_position: BattlePosition,
/// ) -> Vec<BattleInstruction> {
///     let mut instructions = damage_move(state, user_position, target_position, 110);
///     // Hurricane sets rain if it misses due to accuracy
///     if should_set_rain_on_miss(state) {
///         instructions.extend(weather_setting_move(state, Weather::Rain, user_position));
///     }
///     instructions
/// }
/// ```
///
/// # Common moves that use this composer:
/// - Sunny Day (Weather::Sun)
/// - Rain Dance (Weather::Rain)
/// - Sandstorm (Weather::Sand)
/// - Hail (Weather::Hail)
/// - Snow (Weather::Snow)
/// - Drought ability activation (Weather::Sun)
/// - Drizzle ability activation (Weather::Rain)
/// - Sand Stream ability activation (Weather::Sand)
/// - Snow Warning ability activation (Weather::Snow)
pub fn weather_setting_move(
    state: &BattleState,
    weather: Weather,
    user_position: BattlePosition,
) -> Vec<BattleInstruction> {
    weather_move_with_extension(state, weather, user_position)
}

/// Terrain-setting move with proper duration and extension handling
///
/// This composer handles all standard terrain-setting moves with automatic duration
/// management and item-based extensions.
///
/// # What it handles automatically:
/// - Base terrain duration (5 turns)
/// - Item-based duration extension (Terrain Extender extends to 8 turns)
/// - Proper source tracking for the terrain condition
/// - Terrain replacement (overwrites existing terrain)
/// - Instruction generation for battle state application
/// - Affects all grounded Pokemon on the field
///
/// # When to use:
/// - Standard terrain moves (Electric Terrain, Grassy Terrain, Misty Terrain, Psychic Terrain)
/// - Simple terrain setting without additional effects
/// - When you want automatic item extension handling
/// - Terrain-setting abilities (Electric Surge, Grassy Surge, etc.)
///
/// # When NOT to use:
/// - Moves with custom terrain durations (use `terrain_move` directly)
/// - Moves that set terrain conditionally (implement custom logic)
/// - Moves with additional effects beyond terrain (use in combination with other composers)
/// - Secret Power terrain effects (use dedicated Secret Power logic)
///
/// # Example usage patterns:
/// ```rust
/// // Basic terrain move
/// pub fn electric_terrain_move(
///     state: &BattleState,
///     user_position: BattlePosition,
/// ) -> Vec<BattleInstruction> {
///     terrain_setting_move(state, Terrain::Electric, user_position)
/// }
///
/// // For moves with additional effects, combine with other composers:
/// pub fn nature_power_move(
///     state: &BattleState,
///     user_position: BattlePosition,
///     target_position: BattlePosition,
/// ) -> Vec<BattleInstruction> {
///     // Nature Power's effect varies by terrain
///     let move_effect = match state.field.terrain.condition {
///         Terrain::Electric => thunderbolt_move(state, user_position, target_position),
///         Terrain::Grassy => energy_ball_move(state, user_position, target_position),
///         Terrain::Misty => moonblast_move(state, user_position, target_position),
///         Terrain::Psychic => psychic_move(state, user_position, target_position),
///         _ => tri_attack_move(state, user_position, target_position),
///     };
///     move_effect
/// }
/// ```
///
/// # Common moves that use this composer:
/// - Electric Terrain (Terrain::Electric)
/// - Grassy Terrain (Terrain::Grassy)
/// - Misty Terrain (Terrain::Misty)
/// - Psychic Terrain (Terrain::Psychic)
/// - Electric Surge ability activation (Terrain::Electric)
/// - Grassy Surge ability activation (Terrain::Grassy)
/// - Misty Surge ability activation (Terrain::Misty)
/// - Psychic Surge ability activation (Terrain::Psychic)
/// - Tapus' terrain-setting abilities
///
/// # Terrain effects (handled by battle state, not this composer):
/// - Electric Terrain: Prevents sleep, boosts Electric moves by 50%
/// - Grassy Terrain: Heals 1/16 HP per turn, boosts Grass moves by 50%, weakens Earthquake/Bulldoze/Magnitude
/// - Misty Terrain: Prevents status conditions, halves Dragon move damage
/// - Psychic Terrain: Prevents priority moves, boosts Psychic moves by 50%
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