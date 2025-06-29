//! # Field-Related Instructions
//! 
//! Instructions that affect the battlefield conditions: weather, terrain,
//! global effects like Trick Room and Gravity, side conditions, etc.

use crate::core::battle_format::{BattlePosition, SideReference};
use crate::types::{Weather, Terrain};
use serde::{Deserialize, Serialize};


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
    pub fn affected_positions(&self, format: &crate::core::battle_format::BattleFormat) -> Vec<BattlePosition> {
        match self {
            // Weather and terrain affect all positions
            FieldInstruction::Weather { .. } => {
                BattlePosition::all_positions(format)
            },
            FieldInstruction::Terrain { .. } => {
                BattlePosition::all_positions(format)
            },
            // Global effects affect all positions
            FieldInstruction::TrickRoom { .. } => {
                BattlePosition::all_positions(format)
            },
            FieldInstruction::Gravity { .. } => {
                BattlePosition::all_positions(format)
            },
            // Side conditions affect all positions on that side
            FieldInstruction::ApplySideCondition { side, .. } => {
                (0..format.active_pokemon_count())
                    .map(|slot| BattlePosition::new(*side, slot))
                    .collect()
            },
            FieldInstruction::RemoveSideCondition { side, .. } => {
                (0..format.active_pokemon_count())
                    .map(|slot| BattlePosition::new(*side, slot))
                    .collect()
            },
            FieldInstruction::DecrementSideConditionDuration { side, .. } => {
                (0..format.active_pokemon_count())
                    .map(|slot| BattlePosition::new(*side, slot))
                    .collect()
            },
            // Turn decrements affect all positions
            FieldInstruction::DecrementWeatherTurns { .. } |
            FieldInstruction::DecrementTerrainTurns { .. } |
            FieldInstruction::DecrementTrickRoomTurns { .. } |
            FieldInstruction::DecrementGravityTurns { .. } => {
                BattlePosition::all_positions(format)
            },
            // Force switch affects all positions on that side
            FieldInstruction::ToggleForceSwitch { side, .. } => {
                (0..format.active_pokemon_count())
                    .map(|slot| BattlePosition::new(*side, slot))
                    .collect()
            },
            // Baton passing affects all positions on that side
            FieldInstruction::ToggleBatonPassing { side, .. } => {
                (0..format.active_pokemon_count())
                    .map(|slot| BattlePosition::new(*side, slot))
                    .collect()
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