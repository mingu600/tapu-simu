//! Field-related types and implementations for battle state

use crate::core::battle_format::BattlePosition;
use crate::core::instructions::{Terrain, Weather};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-import DamageInfo from pokemon module for TurnState
use super::pokemon::DamageInfo;

/// Field conditions that affect the entire battlefield
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConditions {
    /// Current weather state
    pub weather: WeatherState,
    /// Current terrain state
    pub terrain: TerrainState,
    /// Global battlefield effects
    pub global_effects: GlobalEffects,
}

/// Weather state with source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherState {
    /// Current weather condition
    pub condition: Weather,
    /// Turns remaining (None for permanent weather)
    pub turns_remaining: Option<u8>,
    /// The position that set this weather (for ability interactions)
    pub source: Option<BattlePosition>,
}

/// Terrain state with source tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainState {
    /// Current terrain condition
    pub condition: Terrain,
    /// Turns remaining (None for permanent terrain)
    pub turns_remaining: Option<u8>,
    /// The position that set this terrain (for ability interactions)
    pub source: Option<BattlePosition>,
}

/// Global effects that affect the entire battlefield
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalEffects {
    /// Trick Room state
    pub trick_room: Option<TrickRoomState>,
    /// Gravity state
    pub gravity: Option<GravityState>,
}

/// Trick Room effect state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrickRoomState {
    /// Turns remaining
    pub turns_remaining: u8,
    /// The position that set Trick Room
    pub source: Option<BattlePosition>,
}

/// Gravity effect state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GravityState {
    /// Turns remaining
    pub turns_remaining: u8,
    /// The position that set Gravity
    pub source: Option<BattlePosition>,
}

/// Turn-related state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnState {
    /// Current turn number
    pub number: u32,
    /// Current phase of the turn
    pub phase: TurnPhase,
    /// Positions that have moved this turn (for turn order tracking)
    pub moved_this_turn: Vec<BattlePosition>,
    /// Positions that have taken damage this turn (for Avalanche-like mechanics)
    pub damaged_this_turn: HashMap<BattlePosition, DamageInfo>,
}

/// Phase of the current turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TurnPhase {
    /// Waiting for move selection
    Selection,
    /// Executing moves
    Execution,
    /// End of turn effects
    EndOfTurn,
}

impl Default for FieldConditions {
    fn default() -> Self {
        Self {
            weather: WeatherState::default(),
            terrain: TerrainState::default(),
            global_effects: GlobalEffects::default(),
        }
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            condition: Weather::None,
            turns_remaining: None,
            source: None,
        }
    }
}

impl WeatherState {
    /// Set weather with specified duration and source
    pub fn set(&mut self, condition: Weather, turns: Option<u8>, source: Option<BattlePosition>) {
        self.condition = condition;
        self.turns_remaining = turns;
        self.source = source;
    }

    /// Clear weather
    pub fn clear(&mut self) {
        self.condition = Weather::None;
        self.turns_remaining = None;
        self.source = None;
    }

    /// Decrement weather duration by one turn
    pub fn decrement_turn(&mut self) {
        if let Some(turns) = &mut self.turns_remaining {
            if *turns > 0 {
                *turns -= 1;
                if *turns == 0 {
                    self.clear();
                }
            }
        }
    }
}

impl Default for TerrainState {
    fn default() -> Self {
        Self {
            condition: Terrain::None,
            turns_remaining: None,
            source: None,
        }
    }
}

impl TerrainState {
    /// Set terrain with specified duration and source
    pub fn set(&mut self, condition: Terrain, turns: Option<u8>, source: Option<BattlePosition>) {
        self.condition = condition;
        self.turns_remaining = turns;
        self.source = source;
    }

    /// Clear terrain
    pub fn clear(&mut self) {
        self.condition = Terrain::None;
        self.turns_remaining = None;
        self.source = None;
    }

    /// Decrement terrain duration by one turn
    pub fn decrement_turn(&mut self) {
        if let Some(turns) = &mut self.turns_remaining {
            if *turns > 0 {
                *turns -= 1;
                if *turns == 0 {
                    self.clear();
                }
            }
        }
    }
}

impl Default for GlobalEffects {
    fn default() -> Self {
        Self {
            trick_room: None,
            gravity: None,
        }
    }
}

impl GlobalEffects {
    /// Set Trick Room with specified duration and source
    pub fn set_trick_room(&mut self, turns: u8, source: Option<BattlePosition>) {
        self.trick_room = Some(TrickRoomState {
            turns_remaining: turns,
            source,
        });
    }

    /// Clear Trick Room
    pub fn clear_trick_room(&mut self) {
        self.trick_room = None;
    }

    /// Set Gravity with specified duration and source
    pub fn set_gravity(&mut self, turns: u8, source: Option<BattlePosition>) {
        self.gravity = Some(GravityState {
            turns_remaining: turns,
            source,
        });
    }

    /// Clear Gravity
    pub fn clear_gravity(&mut self) {
        self.gravity = None;
    }

    /// Decrement all global effect durations by one turn
    pub fn decrement_turn(&mut self) {
        if let Some(trick_room) = &mut self.trick_room {
            if trick_room.turns_remaining > 0 {
                trick_room.turns_remaining -= 1;
                if trick_room.turns_remaining == 0 {
                    self.trick_room = None;
                }
            }
        }

        if let Some(gravity) = &mut self.gravity {
            if gravity.turns_remaining > 0 {
                gravity.turns_remaining -= 1;
                if gravity.turns_remaining == 0 {
                    self.gravity = None;
                }
            }
        }
    }
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            number: 1,
            phase: TurnPhase::Selection,
            moved_this_turn: Vec::new(),
            damaged_this_turn: HashMap::new(),
        }
    }
}

impl TurnState {
    /// Advance to the next turn
    pub fn next_turn(&mut self) {
        self.number += 1;
        self.phase = TurnPhase::Selection;
        self.moved_this_turn.clear();
        self.damaged_this_turn.clear();
    }

    /// Set the current turn phase
    pub fn set_phase(&mut self, phase: TurnPhase) {
        self.phase = phase;
    }

    /// Mark a position as having moved this turn
    pub fn mark_moved(&mut self, position: BattlePosition) {
        if !self.moved_this_turn.contains(&position) {
            self.moved_this_turn.push(position);
        }
    }

    /// Mark a position as having taken damage this turn
    pub fn mark_damaged(&mut self, position: BattlePosition, damage_info: DamageInfo) {
        self.damaged_this_turn.insert(position, damage_info);
    }

    /// Check if a position has moved this turn
    pub fn has_moved(&self, position: BattlePosition) -> bool {
        self.moved_this_turn.contains(&position)
    }

    /// Check if a position took damage this turn from a physical or special move
    pub fn took_damage_from_attack(&self, position: BattlePosition) -> bool {
        if let Some(damage_info) = self.damaged_this_turn.get(&position) {
            damage_info.is_direct_damage
                && (damage_info.move_category == crate::core::instructions::MoveCategory::Physical
                    || damage_info.move_category == crate::core::instructions::MoveCategory::Special)
        } else {
            false
        }
    }

    /// Check if user moved after taking damage (for Avalanche mechanics)
    pub fn moved_after_damage(
        &self,
        user_position: BattlePosition,
        attacker_position: BattlePosition,
    ) -> bool {
        // If user took damage from the attacker, check if user moved after attacker
        if let Some(damage_info) = self.damaged_this_turn.get(&user_position) {
            if damage_info.attacker_position == attacker_position {
                // Check if user moved after the attacker by looking at move order
                let user_move_index = self
                    .moved_this_turn
                    .iter()
                    .position(|&pos| pos == user_position);
                let attacker_move_index = self
                    .moved_this_turn
                    .iter()
                    .position(|&pos| pos == attacker_position);

                match (user_move_index, attacker_move_index) {
                    (Some(user_idx), Some(attacker_idx)) => user_idx > attacker_idx,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}