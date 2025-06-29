//! # Move Choice System
//! 
//! This module defines the move choice system for the V2 engine.
//! All move choices are format-aware and use explicit position targeting.

use crate::core::battle_format::BattlePosition;
use serde::{Deserialize, Serialize};

/// Represents a player's choice for a single turn
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MoveChoice {
    /// Use a move with explicit target positions
    Move {
        move_index: MoveIndex,
        target_positions: Vec<BattlePosition>,
    },
    /// Use a move with Terastallization (Gen 9+ only)
    MoveTera {
        move_index: MoveIndex,
        target_positions: Vec<BattlePosition>,
        tera_type: PokemonType,
    },
    /// Switch to a different Pokemon
    Switch(PokemonIndex),
    /// Do nothing (used for speed calculations or when no valid moves)
    None,
}

impl MoveChoice {
    /// Create a new move choice with target positions
    pub fn new_move(move_index: MoveIndex, target_positions: Vec<BattlePosition>) -> Self {
        Self::Move {
            move_index,
            target_positions,
        }
    }

    /// Create a new Terastallization move choice (Gen 9+ only)
    pub fn new_tera_move(
        move_index: MoveIndex,
        target_positions: Vec<BattlePosition>,
        tera_type: PokemonType,
    ) -> Self {
        Self::MoveTera {
            move_index,
            target_positions,
            tera_type,
        }
    }

    /// Create a switch choice
    pub fn new_switch(pokemon_index: PokemonIndex) -> Self {
        Self::Switch(pokemon_index)
    }

    /// Returns the target positions for this move choice, if any
    pub fn target_positions(&self) -> Option<&Vec<BattlePosition>> {
        match self {
            Self::Move { target_positions, .. } => Some(target_positions),
            Self::MoveTera { target_positions, .. } => Some(target_positions),
            Self::Switch(_) | Self::None => None,
        }
    }

    /// Returns the move index for this choice, if it's a move
    pub fn move_index(&self) -> Option<MoveIndex> {
        match self {
            Self::Move { move_index, .. } => Some(*move_index),
            Self::MoveTera { move_index, .. } => Some(*move_index),
            Self::Switch(_) | Self::None => None,
        }
    }

    /// Returns true if this is a move choice (not switch or none)
    pub fn is_move(&self) -> bool {
        match self {
            Self::Move { .. } => true,
            Self::MoveTera { .. } => true,
            _ => false,
        }
    }

    /// Returns true if this is a switch choice
    pub fn is_switch(&self) -> bool {
        matches!(self, Self::Switch(_))
    }

    /// Returns true if this choice uses Terastallization (Gen 9+ only)
    pub fn is_tera(&self) -> bool {
        matches!(self, Self::MoveTera { .. })
    }

    /// Returns the Tera type if this is a Tera move (Gen 9+ only)
    pub fn tera_type(&self) -> Option<PokemonType> {
        match self {
            Self::MoveTera { tera_type, .. } => Some(*tera_type),
            _ => None,
        }
    }

    /// Get the move target type from the move data (requires state access)
    pub fn get_move_target(&self, state: &crate::core::battle_state::BattleState, user_position: crate::core::battle_format::BattlePosition) -> Option<crate::data::showdown_types::MoveTarget> {
        let move_index = self.move_index()?;
        
        // Get the user's side using the provided position
        let user_side = match user_position.side {
            crate::core::battle_format::SideReference::SideOne => &state.sides[0],
            crate::core::battle_format::SideReference::SideTwo => &state.sides[1],
        };
        
        if let Some(pokemon) = user_side.get_active_pokemon_at_slot(user_position.slot) {
            if let Some(move_data) = pokemon.get_move(move_index) {
                return Some(move_data.target);
            }
        }
        
        None
    }

    /// Check if this move targets multiple positions (spread move)
    pub fn is_spread_move(&self, state: &crate::core::battle_state::BattleState, user_position: crate::core::battle_format::BattlePosition) -> bool {
        if let Some(move_target) = self.get_move_target(state, user_position) {
            move_target.is_spread_move()
        } else {
            false
        }
    }

    /// Check if this move can affect allies
    pub fn affects_allies(&self, state: &crate::core::battle_state::BattleState, user_position: crate::core::battle_format::BattlePosition) -> bool {
        if let Some(move_target) = self.get_move_target(state, user_position) {
            move_target.affects_allies()
        } else {
            false
        }
    }

    /// Update the target positions for a move choice (mutable version)
    pub fn set_target_positions(&mut self, new_targets: Vec<BattlePosition>) {
        match self {
            Self::Move { target_positions, .. } => *target_positions = new_targets,
            Self::MoveTera { target_positions, .. } => *target_positions = new_targets,
            _ => {} // No effect on switch or none choices
        }
    }

    /// Convert the move choice to a human-readable string for logging
    pub fn to_string(&self, side: &crate::core::battle_state::BattleSide, user_slot: usize) -> String {
        match self {
            Self::Move { move_index, target_positions } => {
                let move_name = if let Some(pokemon) = side.get_active_pokemon_at_slot(user_slot) {
                    if let Some(move_data) = pokemon.get_move(*move_index) {
                        move_data.name.as_str().to_string()
                    } else {
                        format!("Move{:?}", move_index)
                    }
                } else {
                    format!("Move{:?}", move_index)
                };
                
                if target_positions.is_empty() {
                    move_name
                } else {
                    let targets: Vec<String> = target_positions.iter()
                        .map(|pos| format!("{}:{}", pos.side.to_string(), pos.slot))
                        .collect();
                    format!("{} -> [{}]", move_name, targets.join(", "))
                }
            }
            Self::MoveTera { move_index, target_positions, tera_type } => {
                let move_name = if let Some(pokemon) = side.get_active_pokemon_at_slot(user_slot) {
                    if let Some(move_data) = pokemon.get_move(*move_index) {
                        move_data.name.as_str().to_string()
                    } else {
                        format!("Move{:?}", move_index)
                    }
                } else {
                    format!("Move{:?}", move_index)
                };
                
                if target_positions.is_empty() {
                    format!("{} (Tera {:?})", move_name, tera_type)
                } else {
                    let targets: Vec<String> = target_positions.iter()
                        .map(|pos| format!("{}:{}", pos.side.to_string(), pos.slot))
                        .collect();
                    format!("{} (Tera {:?}) -> [{}]", move_name, tera_type, targets.join(", "))
                }
            }
            Self::Switch(pokemon_index) => {
                let pokemon_name = if let Some(pokemon) = side.pokemon.get(pokemon_index.to_index()) {
                    pokemon.species.as_str().to_string()
                } else {
                    format!("Pokemon{:?}", pokemon_index)
                };
                format!("Switch to {}", pokemon_name)
            }
            Self::None => "None".to_string(),
        }
    }
}

impl Default for MoveChoice {
    fn default() -> Self {
        Self::None
    }
}

/// Represents a Pokemon's move slot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveIndex {
    M0,
    M1,
    M2,
    M3,
}

impl MoveIndex {
    /// Convert to array index
    pub fn to_index(self) -> usize {
        match self {
            MoveIndex::M0 => 0,
            MoveIndex::M1 => 1,
            MoveIndex::M2 => 2,
            MoveIndex::M3 => 3,
        }
    }

    /// Create from array index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(MoveIndex::M0),
            1 => Some(MoveIndex::M1),
            2 => Some(MoveIndex::M2),
            3 => Some(MoveIndex::M3),
            _ => None,
        }
    }

    /// Get all move indices
    pub fn all() -> [MoveIndex; 4] {
        [MoveIndex::M0, MoveIndex::M1, MoveIndex::M2, MoveIndex::M3]
    }

    /// Create from u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        Self::from_index(value as usize)
    }
}

/// Represents a Pokemon's team position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PokemonIndex {
    P0,
    P1,
    P2,
    P3,
    P4,
    P5,
}

impl PokemonIndex {
    /// Convert to array index
    pub fn to_index(self) -> usize {
        match self {
            PokemonIndex::P0 => 0,
            PokemonIndex::P1 => 1,
            PokemonIndex::P2 => 2,
            PokemonIndex::P3 => 3,
            PokemonIndex::P4 => 4,
            PokemonIndex::P5 => 5,
        }
    }

    /// Create from array index
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(PokemonIndex::P0),
            1 => Some(PokemonIndex::P1),
            2 => Some(PokemonIndex::P2),
            3 => Some(PokemonIndex::P3),
            4 => Some(PokemonIndex::P4),
            5 => Some(PokemonIndex::P5),
            _ => None,
        }
    }

    /// Get all Pokemon indices
    pub fn all() -> [PokemonIndex; 6] {
        [
            PokemonIndex::P0,
            PokemonIndex::P1,
            PokemonIndex::P2,
            PokemonIndex::P3,
            PokemonIndex::P4,
            PokemonIndex::P5,
        ]
    }
}

/// Pokemon types for Terastallization (Gen 9+ only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PokemonType {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
    Unknown,
}

impl PokemonType {
    /// Get all Pokemon types
    pub fn all() -> Vec<PokemonType> {
        vec![
            PokemonType::Normal,
            PokemonType::Fire,
            PokemonType::Water,
            PokemonType::Electric,
            PokemonType::Grass,
            PokemonType::Ice,
            PokemonType::Fighting,
            PokemonType::Poison,
            PokemonType::Ground,
            PokemonType::Flying,
            PokemonType::Psychic,
            PokemonType::Bug,
            PokemonType::Rock,
            PokemonType::Ghost,
            PokemonType::Dragon,
            PokemonType::Dark,
            PokemonType::Steel,
            PokemonType::Fairy,
        ]
    }
}

