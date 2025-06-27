//! Centralized field effect management system
//!
//! This module provides a unified interface for managing field effects like weather,
//! terrain, and side conditions. It consolidates all the logic previously duplicated
//! across move implementations and ensures consistent handling of field interactions.

use crate::core::battle_format::{BattlePosition, SideReference};
use crate::core::battle_state::BattleState;
use crate::core::instructions::{
    BattleInstruction, FieldInstruction, SideCondition, Terrain, Weather,
};

/// Set weather condition with proper duration and source tracking
pub fn set_weather(
    weather: Weather,
    duration: Option<u8>,
    source: Option<BattlePosition>,
) -> BattleInstruction {
    BattleInstruction::Field(FieldInstruction::Weather {
        new_weather: weather,
        previous_weather: Weather::None, // Will be filled by battle state
        turns: duration,
        previous_turns: None,
        source,
    })
}

/// Set terrain condition with proper duration and source tracking
pub fn set_terrain(
    terrain: Terrain,
    duration: Option<u8>,
    source: Option<BattlePosition>,
) -> BattleInstruction {
    BattleInstruction::Field(FieldInstruction::Terrain {
        new_terrain: terrain,
        previous_terrain: Terrain::None, // Will be filled by battle state
        turns: duration,
        previous_turns: None,
        source,
    })
}

/// Apply a side condition with proper duration management
pub fn apply_side_condition(
    side: SideReference,
    condition: SideCondition,
    duration: Option<u8>,
) -> BattleInstruction {
    BattleInstruction::Field(FieldInstruction::ApplySideCondition {
        side,
        condition,
        duration: duration.unwrap_or(5), // Default duration if not specified
        previous_duration: None,
    })
}

/// Remove a side condition
pub fn remove_side_condition(side: SideReference, condition: SideCondition) -> BattleInstruction {
    BattleInstruction::Field(FieldInstruction::RemoveSideCondition {
        side,
        condition,
        previous_duration: 0, // Will be filled by battle state
    })
}

/// Weather move implementation using the core system
pub fn weather_move(
    weather: Weather,
    duration: Option<u8>,
    source: BattlePosition,
) -> Vec<BattleInstruction> {
    vec![set_weather(weather, duration, Some(source))]
}

/// Terrain move implementation using the core system
pub fn terrain_move(
    terrain: Terrain,
    duration: Option<u8>,
    source: BattlePosition,
) -> Vec<BattleInstruction> {
    vec![set_terrain(terrain, duration, Some(source))]
}

/// Screen move implementation (Reflect, Light Screen, Aurora Veil)
pub fn screen_move(
    state: &BattleState,
    user_position: BattlePosition,
    screen_type: ScreenType,
    duration: Option<u8>,
) -> Vec<BattleInstruction> {
    let user_side = user_position.side;
    let condition = match screen_type {
        ScreenType::Reflect => SideCondition::Reflect,
        ScreenType::LightScreen => SideCondition::LightScreen,
        ScreenType::AuroraVeil => SideCondition::AuroraVeil,
        ScreenType::Safeguard => SideCondition::Safeguard,
        ScreenType::Mist => SideCondition::Mist,
        ScreenType::LuckyChant => SideCondition::LuckyChant,
    };

    vec![apply_side_condition(user_side, condition, duration)]
}

/// Hazard move implementation (Spikes, Stealth Rock, Toxic Spikes)
pub fn hazard_move(
    state: &BattleState,
    user_position: BattlePosition,
    hazard_type: HazardType,
) -> Vec<BattleInstruction> {
    let target_side = user_position.side.opposite();
    let condition = match hazard_type {
        HazardType::Spikes => SideCondition::Spikes,
        HazardType::ToxicSpikes => SideCondition::ToxicSpikes,
        HazardType::StealthRock => SideCondition::StealthRock,
        HazardType::StickyWeb => SideCondition::StickyWeb,
    };

    vec![apply_side_condition(target_side, condition, None)]
}

/// Hazard removal move implementation (Rapid Spin, Defog)
pub fn hazard_removal_move(
    state: &BattleState,
    user_position: BattlePosition,
    removal_type: HazardRemovalType,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();
    let user_side = user_position.side;

    match removal_type {
        HazardRemovalType::RapidSpin => {
            // Rapid Spin removes hazards from user's side
            let hazards = [
                SideCondition::Spikes,
                SideCondition::ToxicSpikes,
                SideCondition::StealthRock,
                SideCondition::StickyWeb,
            ];

            for hazard in &hazards {
                if state.sides[user_side as usize]
                    .side_conditions
                    .contains_key(hazard)
                {
                    instructions.push(remove_side_condition(user_side, *hazard));
                }
            }
        }
        HazardRemovalType::Defog => {
            // Defog removes hazards from both sides
            let hazards = [
                SideCondition::Spikes,
                SideCondition::ToxicSpikes,
                SideCondition::StealthRock,
                SideCondition::StickyWeb,
            ];

            for side_idx in 0..2 {
                let side_ref = if side_idx == 0 {
                    SideReference::SideOne
                } else {
                    SideReference::SideTwo
                };

                for hazard in &hazards {
                    if state.sides[side_idx].side_conditions.contains_key(hazard) {
                        instructions.push(remove_side_condition(side_ref, *hazard));
                    }
                }
            }

            // Defog also removes screens from the target's side
            let target_side = user_position.side.opposite();
            let screens = [
                SideCondition::Reflect,
                SideCondition::LightScreen,
                SideCondition::AuroraVeil,
            ];

            for screen in &screens {
                if state.sides[target_side as usize]
                    .side_conditions
                    .contains_key(screen)
                {
                    instructions.push(remove_side_condition(target_side, *screen));
                }
            }
        }
        HazardRemovalType::TidyUp => {
            // Tidy Up removes hazards from both sides and substitutes
            let hazards = [
                SideCondition::Spikes,
                SideCondition::ToxicSpikes,
                SideCondition::StealthRock,
                SideCondition::StickyWeb,
            ];

            for side_idx in 0..2 {
                let side_ref = if side_idx == 0 {
                    SideReference::SideOne
                } else {
                    SideReference::SideTwo
                };

                for hazard in &hazards {
                    if state.sides[side_idx].side_conditions.contains_key(hazard) {
                        instructions.push(remove_side_condition(side_ref, *hazard));
                    }
                }
            }

            // Remove substitutes from all active Pokemon on both sides
            for side_idx in 0..2 {
                let side_ref = if side_idx == 0 {
                    SideReference::SideOne
                } else {
                    SideReference::SideTwo
                };
                
                for slot in 0..state.format.active_pokemon_count() {
                    let position = BattlePosition::new(side_ref, slot);
                    if let Some(pokemon) = state.get_pokemon_at_position(position) {
                        if pokemon.volatile_statuses.contains(&crate::core::instructions::VolatileStatus::Substitute) {
                            instructions.push(BattleInstruction::Status(
                                crate::core::instructions::StatusInstruction::RemoveVolatile {
                                    target: position,
                                    status: crate::core::instructions::VolatileStatus::Substitute,
                                    previous_duration: None,
                                }
                            ));
                        }
                    }
                }
            }
        }
    }

    instructions
}

/// Check if weather should be extended by an item or ability
pub fn check_weather_extension(
    state: &BattleState,
    weather: Weather,
    source_position: BattlePosition,
) -> Option<u8> {
    let pokemon = state.get_pokemon_at_position(source_position)?;

    // Check for weather-extending items
    if let Some(ref item) = pokemon.item {
        match (weather, item) {
            (Weather::Sun | Weather::HarshSun, crate::types::Items::HEATROCK) => Some(8),
            (Weather::Rain | Weather::HeavyRain, crate::types::Items::DAMPROCK) => Some(8),
            (Weather::Sand | Weather::Sandstorm, crate::types::Items::SMOOTHROCK) => Some(8),
            (Weather::Hail | Weather::Snow, crate::types::Items::ICYROCK) => Some(8),
            _ => None,
        }
    } else {
        None
    }
}

/// Check if terrain should be extended by an item or ability
pub fn check_terrain_extension(
    state: &BattleState,
    terrain: Terrain,
    source_position: BattlePosition,
) -> Option<u8> {
    let pokemon = state.get_pokemon_at_position(source_position)?;

    // Check for terrain-extending items
    if let Some(ref item) = pokemon.item {
        match (terrain, item) {
            (Terrain::Electric | Terrain::ElectricTerrain, crate::types::Items::TERRAINEXTENDER) => Some(8),
            (Terrain::Grassy | Terrain::GrassyTerrain, crate::types::Items::TERRAINEXTENDER) => Some(8),
            (Terrain::Misty | Terrain::MistyTerrain, crate::types::Items::TERRAINEXTENDER) => Some(8),
            (Terrain::Psychic | Terrain::PsychicTerrain, crate::types::Items::TERRAINEXTENDER) => Some(8),
            _ => None,
        }
    } else {
        None
    }
}

/// Types of screen moves
#[derive(Debug, Clone, Copy)]
pub enum ScreenType {
    Reflect,
    LightScreen,
    AuroraVeil,
    Safeguard,
    Mist,
    LuckyChant,
}

/// Types of hazard moves
#[derive(Debug, Clone, Copy)]
pub enum HazardType {
    Spikes,
    ToxicSpikes,
    StealthRock,
    StickyWeb,
}

/// Types of hazard removal moves
#[derive(Debug, Clone, Copy)]
pub enum HazardRemovalType {
    RapidSpin,
    Defog,
    TidyUp,
}

/// Weather-setting move with item duration extension
pub fn weather_move_with_extension(
    state: &BattleState,
    weather: Weather,
    source: BattlePosition,
) -> Vec<BattleInstruction> {
    let base_duration = Some(5); // Default weather duration
    let extended_duration = check_weather_extension(state, weather, source)
        .or(base_duration);

    vec![set_weather(weather, extended_duration, Some(source))]
}

/// Terrain-setting move with item duration extension
pub fn terrain_move_with_extension(
    state: &BattleState,
    terrain: Terrain,
    source: BattlePosition,
) -> Vec<BattleInstruction> {
    let base_duration = Some(5); // Default terrain duration
    let extended_duration = check_terrain_extension(state, terrain, source)
        .or(base_duration);

    vec![set_terrain(terrain, extended_duration, Some(source))]
}

/// Check if a side condition can be applied (not blocked by other effects)
pub fn can_apply_side_condition(
    state: &BattleState,
    side: SideReference,
    condition: SideCondition,
) -> bool {
    let side_state = &state.sides[side as usize];

    match condition {
        SideCondition::Spikes => {
            // Spikes can stack up to 3 layers
            side_state
                .side_conditions
                .get(&SideCondition::Spikes)
                .map_or(true, |&layers| layers < 3)
        }
        SideCondition::ToxicSpikes => {
            // Toxic Spikes can stack up to 2 layers
            side_state
                .side_conditions
                .get(&SideCondition::ToxicSpikes)
                .map_or(true, |&layers| layers < 2)
        }
        SideCondition::StealthRock | SideCondition::StickyWeb => {
            // These don't stack, so can only be applied if not already present
            !side_state.side_conditions.contains_key(&condition)
        }
        _ => {
            // Most other conditions don't stack
            !side_state.side_conditions.contains_key(&condition)
        }
    }
}

/// Apply multiple field effects in sequence
pub fn apply_multiple_field_effects(
    state: &BattleState,
    effects: Vec<FieldEffect>,
) -> Vec<BattleInstruction> {
    let mut instructions = Vec::new();

    for effect in effects {
        match effect {
            FieldEffect::Weather { weather, duration, source } => {
                instructions.push(set_weather(weather, duration, source));
            }
            FieldEffect::Terrain { terrain, duration, source } => {
                instructions.push(set_terrain(terrain, duration, source));
            }
            FieldEffect::SideCondition { side, condition, duration } => {
                if can_apply_side_condition(state, side, condition) {
                    instructions.push(apply_side_condition(side, condition, duration));
                }
            }
        }
    }

    instructions
}

/// Represents a field effect to be applied
#[derive(Debug, Clone)]
pub enum FieldEffect {
    Weather {
        weather: Weather,
        duration: Option<u8>,
        source: Option<BattlePosition>,
    },
    Terrain {
        terrain: Terrain,
        duration: Option<u8>,
        source: Option<BattlePosition>,
    },
    SideCondition {
        side: SideReference,
        condition: SideCondition,
        duration: Option<u8>,
    },
}