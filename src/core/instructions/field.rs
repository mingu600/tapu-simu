//! # Field-Related Instructions
//! 
//! Instructions that affect the battlefield conditions: weather, terrain,
//! global effects like Trick Room and Gravity, side conditions, etc.

use crate::core::battle_format::{BattlePosition, SideReference};
use crate::core::instruction::{Weather, Terrain, SideCondition};
use serde::{Deserialize, Serialize};

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
        }
    }
}