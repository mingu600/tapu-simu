//! # Battle Data Types
//!
//! This module defines core data types used throughout the battle system.

use serde::{Deserialize, Serialize};
use crate::types::from_string::FromNormalizedString;

/// Pokemon stats structure
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Stats {
    pub hp: i16,
    pub attack: i16,
    pub defense: i16,
    pub special_attack: i16,
    pub special_defense: i16,
    pub speed: i16,
}

/// Pokemon natures that affect stat growth
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Nature {
    Hardy,
    Lonely,
    Brave,
    Adamant,
    Naughty,
    Bold,
    Docile,
    Relaxed,
    Impish,
    Lax,
    Timid,
    Hasty,
    Serious,
    Jolly,
    Naive,
    Modest,
    Mild,
    Quiet,
    Bashful,
    Rash,
    Calm,
    Gentle,
    Sassy,
    Careful,
    Quirky,
}

impl FromNormalizedString for Nature {
    fn from_normalized_str(s: &str) -> Option<Self> {
        match s {
            "hardy" => Some(Nature::Hardy),
            "lonely" => Some(Nature::Lonely),
            "brave" => Some(Nature::Brave),
            "adamant" => Some(Nature::Adamant),
            "naughty" => Some(Nature::Naughty),
            "bold" => Some(Nature::Bold),
            "docile" => Some(Nature::Docile),
            "relaxed" => Some(Nature::Relaxed),
            "impish" => Some(Nature::Impish),
            "lax" => Some(Nature::Lax),
            "timid" => Some(Nature::Timid),
            "hasty" => Some(Nature::Hasty),
            "serious" => Some(Nature::Serious),
            "jolly" => Some(Nature::Jolly),
            "naive" => Some(Nature::Naive),
            "modest" => Some(Nature::Modest),
            "mild" => Some(Nature::Mild),
            "quiet" => Some(Nature::Quiet),
            "bashful" => Some(Nature::Bashful),
            "rash" => Some(Nature::Rash),
            "calm" => Some(Nature::Calm),
            "gentle" => Some(Nature::Gentle),
            "sassy" => Some(Nature::Sassy),
            "careful" => Some(Nature::Careful),
            "quirky" => Some(Nature::Quirky),
            _ => None,
        }
    }
    
    fn valid_strings() -> Vec<&'static str> {
        vec![
            "hardy", "lonely", "brave", "adamant", "naughty",
            "bold", "docile", "relaxed", "impish", "lax",
            "timid", "hasty", "serious", "jolly", "naive",
            "modest", "mild", "quiet", "bashful", "rash",
            "calm", "gentle", "sassy", "careful", "quirky",
        ]
    }
}

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
