//! # Engine Data Types
//! 
//! This module defines engine-specific data types that are optimized for
//! battle mechanics while staying compatible with rustemon/PokeAPI data.

use serde::{Deserialize, Serialize};
use crate::core::battle_state::MoveCategory;

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
    pub target: crate::data::showdown_types::MoveTarget,
    pub effect_chance: Option<i16>,
    pub effect_description: String,
    pub flags: Vec<String>,
}


/// Type effectiveness multiplier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeEffectiveness {
    SuperEffective, // 2.0x
    Effective,      // 1.0x
    NotVeryEffective, // 0.5x
    NoEffect,       // 0.0x
}

/// Pokemon natures that affect stat growth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nature {
    Hardy, Lonely, Brave, Adamant, Naughty,
    Bold, Docile, Relaxed, Impish, Lax,
    Timid, Hasty, Serious, Jolly, Naive,
    Modest, Mild, Quiet, Bashful, Rash,
    Calm, Gentle, Sassy, Careful, Quirky,
}

/// Type alias for stats - using EngineBaseStats as the standard stats structure
pub type Stats = EngineBaseStats;

impl Nature {
    /// Get the attack stat modifier for this nature
    pub fn attack_modifier(&self) -> f64 {
        match self {
            Nature::Lonely | Nature::Brave | Nature::Adamant | Nature::Naughty => 1.1,
            Nature::Bold | Nature::Timid | Nature::Modest | Nature::Calm => 0.9,
            _ => 1.0,
        }
    }
    
    /// Get the defense stat modifier for this nature
    pub fn defense_modifier(&self) -> f64 {
        match self {
            Nature::Bold | Nature::Relaxed | Nature::Impish | Nature::Lax => 1.1,
            Nature::Lonely | Nature::Hasty | Nature::Mild | Nature::Gentle => 0.9,
            _ => 1.0,
        }
    }
    
    /// Get the special attack stat modifier for this nature
    pub fn special_attack_modifier(&self) -> f64 {
        match self {
            Nature::Modest | Nature::Mild | Nature::Quiet | Nature::Rash => 1.1,
            Nature::Adamant | Nature::Impish | Nature::Jolly | Nature::Careful => 0.9,
            _ => 1.0,
        }
    }
    
    /// Get the special defense stat modifier for this nature
    pub fn special_defense_modifier(&self) -> f64 {
        match self {
            Nature::Calm | Nature::Gentle | Nature::Sassy | Nature::Careful => 1.1,
            Nature::Naughty | Nature::Lax | Nature::Naive | Nature::Rash => 0.9,
            _ => 1.0,
        }
    }
    
    /// Get the speed stat modifier for this nature
    pub fn speed_modifier(&self) -> f64 {
        match self {
            Nature::Timid | Nature::Hasty | Nature::Jolly | Nature::Naive => 1.1,
            Nature::Brave | Nature::Relaxed | Nature::Quiet | Nature::Sassy => 0.9,
            _ => 1.0,
        }
    }
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
    fn test_type_effectiveness() {
        assert_eq!(TypeEffectiveness::SuperEffective.multiplier(), 2.0);
        assert_eq!(TypeEffectiveness::Effective.multiplier(), 1.0);
        assert_eq!(TypeEffectiveness::NotVeryEffective.multiplier(), 0.5);
        assert_eq!(TypeEffectiveness::NoEffect.multiplier(), 0.0);
    }
}