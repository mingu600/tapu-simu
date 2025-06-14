//! # Engine Data Types
//! 
//! This module defines engine-specific data types that are optimized for
//! battle mechanics while staying compatible with rustemon/PokeAPI data.

use serde::{Deserialize, Serialize};
use crate::state::MoveCategory;

/// Engine-optimized Pokemon data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnginePokemonData {
    pub id: i32,
    pub name: String,
    pub base_stats: EngineBaseStats,
    pub types: Vec<String>,
    pub abilities: Vec<String>,
    pub moves: Vec<String>,
    pub height: i32,
    pub weight: i32,
}

/// Engine-optimized base stats
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EngineBaseStats {
    pub hp: i16,
    pub attack: i16,
    pub defense: i16,
    pub special_attack: i16,
    pub special_defense: i16,
    pub speed: i16,
}

/// Engine-optimized move data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineMoveData {
    pub id: i32,
    pub name: String,
    pub base_power: Option<i16>,
    pub accuracy: Option<i16>,
    pub pp: i16,
    pub move_type: String,
    pub category: MoveCategory,
    pub priority: i8,
    pub target: crate::data::ps_types::PSMoveTarget,
    pub effect_chance: Option<i16>,
    pub effect_description: String,
    pub flags: Vec<String>,
}

/// Move targeting information (from V1 for compatibility)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveTarget {
    SpecificMove,           // specific-move (1)
    SelectedPokemonMeFirst, // selected-pokemon-me-first (2)
    Ally,                   // ally (3)
    UsersField,             // users-field (4)
    UserOrAlly,             // user-or-ally (5)
    OpponentsField,         // opponents-field (6)
    User,                   // user (7)
    RandomOpponent,         // random-opponent (8)
    AllOtherPokemon,        // all-other-pokemon (9)
    SelectedPokemon,        // selected-pokemon (10)
    AllOpponents,           // all-opponents (11)
    EntireField,            // entire-field (12)
    UserAndAllies,          // user-and-allies (13)
    AllPokemon,             // all-pokemon (14)
    AllAllies,              // all-allies (15)
    FaintingPokemon,        // fainting-pokemon (16)
}

impl MoveTarget {
    /// Returns true if this target requires user selection
    pub fn requires_user_selection(&self) -> bool {
        matches!(self, MoveTarget::SpecificMove)
    }

    /// Returns true if this target can hit multiple Pokemon
    pub fn is_spread_move(&self) -> bool {
        matches!(
            self,
            MoveTarget::AllOpponents
                | MoveTarget::AllOtherPokemon
                | MoveTarget::AllPokemon
                | MoveTarget::UserAndAllies
        )
    }

    /// Returns true if this target affects allies
    pub fn affects_allies(&self) -> bool {
        matches!(
            self,
            MoveTarget::AllOtherPokemon
                | MoveTarget::AllPokemon
                | MoveTarget::Ally
                | MoveTarget::UserAndAllies
        )
    }
}

/// Type effectiveness multiplier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeEffectiveness {
    SuperEffective, // 2.0x
    Effective,      // 1.0x
    NotVeryEffective, // 0.5x
    NoEffect,       // 0.0x
}

impl TypeEffectiveness {
    /// Get the damage multiplier for this effectiveness
    pub fn multiplier(&self) -> f32 {
        match self {
            TypeEffectiveness::SuperEffective => 2.0,
            TypeEffectiveness::Effective => 1.0,
            TypeEffectiveness::NotVeryEffective => 0.5,
            TypeEffectiveness::NoEffect => 0.0,
        }
    }
}

/// Engine-optimized ability data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineAbilityData {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub effects: Vec<String>,
}

/// Engine-optimized item data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineItemData {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub effects: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_target_properties() {
        assert!(MoveTarget::SpecificMove.requires_user_selection());
        assert!(!MoveTarget::User.requires_user_selection());

        assert!(MoveTarget::AllOpponents.is_spread_move());
        assert!(!MoveTarget::SpecificMove.is_spread_move());

        assert!(MoveTarget::AllOtherPokemon.affects_allies());
        assert!(!MoveTarget::AllOpponents.affects_allies());
    }

    #[test]
    fn test_type_effectiveness() {
        assert_eq!(TypeEffectiveness::SuperEffective.multiplier(), 2.0);
        assert_eq!(TypeEffectiveness::Effective.multiplier(), 1.0);
        assert_eq!(TypeEffectiveness::NotVeryEffective.multiplier(), 0.5);
        assert_eq!(TypeEffectiveness::NoEffect.multiplier(), 0.0);
    }
}