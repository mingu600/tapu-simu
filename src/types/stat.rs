//! # Stat Type System
//!
//! This module defines the Stat enum used throughout the battle system
//! for Pokemon stats that can be boosted, lowered, or modified.

use crate::types::from_string::FromNormalizedString;
use serde::{Deserialize, Serialize};

/// Pokemon stats that can be boosted/lowered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stat {
    Hp,
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    Accuracy,
    Evasion,
}

impl From<u8> for Stat {
    fn from(value: u8) -> Self {
        match value {
            0 => Stat::Hp,
            1 => Stat::Attack,
            2 => Stat::Defense,
            3 => Stat::SpecialAttack,
            4 => Stat::SpecialDefense,
            5 => Stat::Speed,
            6 => Stat::Accuracy,
            7 => Stat::Evasion,
            _ => Stat::Hp, // Default fallback
        }
    }
}

/// Implementation of unified string parsing trait
impl FromNormalizedString for Stat {
    fn from_normalized_str(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "hp" | "health" | "hitpoints" => Some(Self::Hp),
            "attack" | "atk" => Some(Self::Attack),
            "defense" | "def" => Some(Self::Defense),
            "specialattack" | "spatk" | "spa" => Some(Self::SpecialAttack),
            "specialdefense" | "spdef" | "spd" => Some(Self::SpecialDefense),
            "speed" | "spe" => Some(Self::Speed),
            "accuracy" | "acc" => Some(Self::Accuracy),
            "evasion" | "eva" => Some(Self::Evasion),
            _ => None,
        }
    }
    
    fn valid_strings() -> Vec<&'static str> {
        vec![
            "hp", "attack", "defense", "specialattack", "specialdefense", 
            "speed", "accuracy", "evasion"
        ]
    }
}