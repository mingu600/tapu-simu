//! # Field-Related Instructions
//! 
//! Instructions that affect the battlefield conditions: weather, terrain,
//! global effects like Trick Room and Gravity, side conditions, etc.

use crate::core::battle_format::{BattlePosition, SideReference};
use serde::{Deserialize, Serialize};

/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    None,
    Hail,
    Rain,
    Sandstorm,
    Sand, // Alias for Sandstorm
    Snow,
    Sun,
    HarshSunlight,
    HarshSun, // Alias for HarshSunlight
    HeavyRain,
    StrongWinds,
}

impl From<u8> for Weather {
    fn from(value: u8) -> Self {
        match value {
            0 => Weather::None,
            1 => Weather::Hail,
            2 => Weather::Rain,
            3 => Weather::Sandstorm,
            4 => Weather::Sand,
            5 => Weather::Snow,
            6 => Weather::Sun,
            7 => Weather::HarshSunlight,
            8 => Weather::HarshSun,
            9 => Weather::HeavyRain,
            10 => Weather::StrongWinds,
            _ => Weather::None, // Default fallback
        }
    }
}

impl Default for Weather {
    fn default() -> Self {
        Weather::None
    }
}

/// Terrain conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    None,
    Electric,
    ElectricTerrain, // Alias for Electric
    Grassy,
    GrassyTerrain, // Alias for Grassy
    Misty,
    MistyTerrain, // Alias for Misty
    Psychic,
    PsychicTerrain, // Alias for Psychic
}

impl From<u8> for Terrain {
    fn from(value: u8) -> Self {
        match value {
            0 => Terrain::None,
            1 => Terrain::Electric,
            2 => Terrain::ElectricTerrain,
            3 => Terrain::Grassy,
            4 => Terrain::GrassyTerrain,
            5 => Terrain::Misty,
            6 => Terrain::MistyTerrain,
            7 => Terrain::Psychic,
            8 => Terrain::PsychicTerrain,
            _ => Terrain::None, // Default fallback
        }
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain::None
    }
}

/// Side conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideCondition {
    Reflect,
    LightScreen,
    AuroraVeil,
    Mist,
    Safeguard,
    Tailwind,
    Spikes,
    ToxicSpikes,
    StealthRock,
    StickyWeb,
    Wish,
    FutureSight,
    DoomDesire,
    HealingWish,
    LunarDance,
    CraftyShield,
    MatBlock,
    QuickGuard,
    WideGuard,
    LuckyChant,
}

impl From<u8> for SideCondition {
    fn from(value: u8) -> Self {
        match value {
            0 => SideCondition::Reflect,
            1 => SideCondition::LightScreen,
            2 => SideCondition::AuroraVeil,
            3 => SideCondition::Mist,
            4 => SideCondition::Safeguard,
            5 => SideCondition::Tailwind,
            6 => SideCondition::Spikes,
            7 => SideCondition::ToxicSpikes,
            8 => SideCondition::StealthRock,
            9 => SideCondition::StickyWeb,
            10 => SideCondition::Wish,
            11 => SideCondition::FutureSight,
            12 => SideCondition::DoomDesire,
            13 => SideCondition::HealingWish,
            14 => SideCondition::LunarDance,
            15 => SideCondition::CraftyShield,
            16 => SideCondition::MatBlock,
            17 => SideCondition::QuickGuard,
            18 => SideCondition::WideGuard,
            _ => SideCondition::Reflect, // Default fallback
        }
    }
}

/// Field-related instruction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldInstruction {
    /// Change weather conditions
    Weather {
        new_weather: Weather,
        previous_weather: Weather,
        turns: Option<u8>,
        previous_turns: Option<u8>,
        source: Option<BattlePosition>,
    },
    /// Change terrain conditions
    Terrain {
        new_terrain: Terrain,
        previous_terrain: Terrain,
        turns: Option<u8>,
        previous_turns: Option<u8>,
        source: Option<BattlePosition>,
    },
    /// Toggle Trick Room
    TrickRoom {
        active: bool,
        turns: Option<u8>,
        source: Option<BattlePosition>,
        previous_active: bool,
        previous_turns: Option<u8>,
    },
    /// Toggle Gravity
    Gravity {
        active: bool,
        turns: Option<u8>,
        source: Option<BattlePosition>,
        previous_active: bool,
        previous_turns: Option<u8>,
    },
    /// Apply side condition
    ApplySideCondition {
        side: SideReference,
        condition: SideCondition,
        duration: u8,
        previous_duration: Option<u8>,
    },
    /// Remove side condition
    RemoveSideCondition {
        side: SideReference,
        condition: SideCondition,
        previous_duration: u8,
    },
    /// Decrement side condition duration
    DecrementSideConditionDuration {
        side: SideReference,
        condition: SideCondition,
        previous_duration: u8,
    },
    /// Decrement weather turns remaining
    DecrementWeatherTurns {
        previous_turns: Option<u8>,
    },
    /// Decrement terrain turns remaining
    DecrementTerrainTurns {
        previous_turns: Option<u8>,
    },
    /// Decrement trick room turns remaining
    DecrementTrickRoomTurns {
        previous_turns: Option<u8>,
    },
    /// Decrement gravity turns remaining
    DecrementGravityTurns {
        previous_turns: Option<u8>,
    },
    /// Toggle force switch for a side
    ToggleForceSwitch {
        side: SideReference,
        active: bool,
        previous_state: bool,
    },
    /// Toggle baton passing for a side
    ToggleBatonPassing {
        side: SideReference,
        active: bool,
        previous_state: bool,
    },
    /// Display a message (for debugging/logging)
    Message {
        message: String,
        affected_positions: Vec<BattlePosition>,
    },
}

impl FieldInstruction {
    /// Returns all positions affected by this instruction
    pub fn affected_positions(&self) -> Vec<BattlePosition> {
        match self {
            // Weather and terrain affect all positions
            FieldInstruction::Weather { .. } => {
                vec![
                    BattlePosition { side: SideReference::SideOne, slot: 0 },
                    BattlePosition { side: SideReference::SideOne, slot: 1 },
                    BattlePosition { side: SideReference::SideTwo, slot: 0 },
                    BattlePosition { side: SideReference::SideTwo, slot: 1 },
                ]
            },
            FieldInstruction::Terrain { .. } => {
                vec![
                    BattlePosition { side: SideReference::SideOne, slot: 0 },
                    BattlePosition { side: SideReference::SideOne, slot: 1 },
                    BattlePosition { side: SideReference::SideTwo, slot: 0 },
                    BattlePosition { side: SideReference::SideTwo, slot: 1 },
                ]
            },
            // Global effects affect all positions
            FieldInstruction::TrickRoom { .. } => {
                vec![
                    BattlePosition { side: SideReference::SideOne, slot: 0 },
                    BattlePosition { side: SideReference::SideOne, slot: 1 },
                    BattlePosition { side: SideReference::SideTwo, slot: 0 },
                    BattlePosition { side: SideReference::SideTwo, slot: 1 },
                ]
            },
            FieldInstruction::Gravity { .. } => {
                vec![
                    BattlePosition { side: SideReference::SideOne, slot: 0 },
                    BattlePosition { side: SideReference::SideOne, slot: 1 },
                    BattlePosition { side: SideReference::SideTwo, slot: 0 },
                    BattlePosition { side: SideReference::SideTwo, slot: 1 },
                ]
            },
            // Side conditions affect all positions on that side
            FieldInstruction::ApplySideCondition { side, .. } => {
                match side {
                    SideReference::SideOne => vec![
                        BattlePosition { side: SideReference::SideOne, slot: 0 },
                        BattlePosition { side: SideReference::SideOne, slot: 1 },
                    ],
                    SideReference::SideTwo => vec![
                        BattlePosition { side: SideReference::SideTwo, slot: 0 },
                        BattlePosition { side: SideReference::SideTwo, slot: 1 },
                    ],
                }
            },
            FieldInstruction::RemoveSideCondition { side, .. } => {
                match side {
                    SideReference::SideOne => vec![
                        BattlePosition { side: SideReference::SideOne, slot: 0 },
                        BattlePosition { side: SideReference::SideOne, slot: 1 },
                    ],
                    SideReference::SideTwo => vec![
                        BattlePosition { side: SideReference::SideTwo, slot: 0 },
                        BattlePosition { side: SideReference::SideTwo, slot: 1 },
                    ],
                }
            },
            FieldInstruction::DecrementSideConditionDuration { side, .. } => {
                match side {
                    SideReference::SideOne => vec![
                        BattlePosition { side: SideReference::SideOne, slot: 0 },
                        BattlePosition { side: SideReference::SideOne, slot: 1 },
                    ],
                    SideReference::SideTwo => vec![
                        BattlePosition { side: SideReference::SideTwo, slot: 0 },
                        BattlePosition { side: SideReference::SideTwo, slot: 1 },
                    ],
                }
            },
            // Turn decrements affect all positions
            FieldInstruction::DecrementWeatherTurns { .. } |
            FieldInstruction::DecrementTerrainTurns { .. } |
            FieldInstruction::DecrementTrickRoomTurns { .. } |
            FieldInstruction::DecrementGravityTurns { .. } => {
                vec![
                    BattlePosition { side: SideReference::SideOne, slot: 0 },
                    BattlePosition { side: SideReference::SideOne, slot: 1 },
                    BattlePosition { side: SideReference::SideTwo, slot: 0 },
                    BattlePosition { side: SideReference::SideTwo, slot: 1 },
                ]
            },
            // Force switch affects all positions on that side
            FieldInstruction::ToggleForceSwitch { side, .. } => {
                match side {
                    SideReference::SideOne => vec![
                        BattlePosition { side: SideReference::SideOne, slot: 0 },
                        BattlePosition { side: SideReference::SideOne, slot: 1 },
                    ],
                    SideReference::SideTwo => vec![
                        BattlePosition { side: SideReference::SideTwo, slot: 0 },
                        BattlePosition { side: SideReference::SideTwo, slot: 1 },
                    ],
                }
            },
            // Baton passing affects all positions on that side
            FieldInstruction::ToggleBatonPassing { side, .. } => {
                match side {
                    SideReference::SideOne => vec![
                        BattlePosition { side: SideReference::SideOne, slot: 0 },
                        BattlePosition { side: SideReference::SideOne, slot: 1 },
                    ],
                    SideReference::SideTwo => vec![
                        BattlePosition { side: SideReference::SideTwo, slot: 0 },
                        BattlePosition { side: SideReference::SideTwo, slot: 1 },
                    ],
                }
            },
            FieldInstruction::Message { affected_positions, .. } => affected_positions.clone(),
        }
    }

    /// Whether this instruction can be undone
    pub fn is_undoable(&self) -> bool {
        match self {
            // All field instructions store previous state for undo
            FieldInstruction::Weather { .. } => true,
            FieldInstruction::Terrain { .. } => true,
            FieldInstruction::TrickRoom { .. } => true,
            FieldInstruction::Gravity { .. } => true,
            FieldInstruction::ApplySideCondition { previous_duration, .. } => previous_duration.is_some(),
            FieldInstruction::RemoveSideCondition { .. } => true,
            FieldInstruction::DecrementSideConditionDuration { .. } => true,
            FieldInstruction::DecrementWeatherTurns { previous_turns, .. } => previous_turns.is_some(),
            FieldInstruction::DecrementTerrainTurns { previous_turns, .. } => previous_turns.is_some(),
            FieldInstruction::DecrementTrickRoomTurns { previous_turns, .. } => previous_turns.is_some(),
            FieldInstruction::DecrementGravityTurns { previous_turns, .. } => previous_turns.is_some(),
            FieldInstruction::ToggleForceSwitch { .. } => true,
            FieldInstruction::ToggleBatonPassing { .. } => true,
            FieldInstruction::Message { .. } => false, // Messages are not undoable
        }
    }
}