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

/// Compact array storage for stat boosts (-6 to +6)
/// More memory efficient than HashMap for stat boosts
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StatBoostArray([i8; 8]);

impl StatBoostArray {
    /// Get the boost value for a specific stat (HashMap-compatible)
    /// Returns Some(value) for HashMap compatibility, but always returns Some since we have defaults
    pub fn get(&self, stat: Stat) -> Option<i8> {
        Some(self.0[stat as usize])
    }
    
    /// Get the boost value for a specific stat directly without Option wrapper
    pub fn get_direct(&self, stat: Stat) -> i8 {
        self.0[stat as usize]
    }
    
    /// Insert/set the boost value for a specific stat (HashMap-compatible)
    pub fn insert(&mut self, stat: Stat, value: i8) {
        self.0[stat as usize] = value.clamp(-6, 6);
    }
    
    /// Remove a stat boost, setting it to 0 (HashMap-compatible)
    pub fn remove(&mut self, stat: Stat) {
        self.0[stat as usize] = 0;
    }
    
    /// Clear all stat boosts (HashMap-compatible)
    pub fn clear(&mut self) {
        self.0 = [0; 8];
    }
    
    /// Get mutable reference to boost value (HashMap-compatible)
    pub fn get_mut(&mut self, stat: Stat) -> &mut i8 {
        &mut self.0[stat as usize]
    }
    
    /// Set the boost value for a specific stat, clamping to valid range
    pub fn set(&mut self, stat: Stat, value: i8) {
        self.insert(stat, value);
    }
    
    /// Modify the boost value for a specific stat by a delta amount
    pub fn modify(&mut self, stat: Stat, delta: i8) {
        let current = self.get_direct(stat);
        self.insert(stat, current + delta);
    }
    
    /// Reset all stat boosts to 0 (alias for clear)
    pub fn reset(&mut self) {
        self.clear();
    }
    
    /// Check if any stat has a boost/drop
    pub fn has_any_boosts(&self) -> bool {
        self.0.iter().any(|&boost| boost != 0)
    }
    
    /// Check if no stat has boosts (HashMap-compatible)
    pub fn is_empty(&self) -> bool {
        !self.has_any_boosts()
    }
    
    /// Get an iterator over all boost values (HashMap-compatible)
    pub fn values(&self) -> impl Iterator<Item = i8> + '_ {
        self.0.iter().copied()
    }
    
    /// Get an iterator over (stat, boost) pairs (HashMap-compatible)
    pub fn iter(&self) -> impl Iterator<Item = (Stat, i8)> + '_ {
        self.0.iter().enumerate().map(|(i, &boost)| (Stat::from(i as u8), boost))
    }
    
    /// Get all non-zero boosts as a HashMap for compatibility
    pub fn to_hashmap(&self) -> std::collections::HashMap<Stat, i8> {
        let mut map = std::collections::HashMap::new();
        for (i, &boost) in self.0.iter().enumerate() {
            if boost != 0 {
                map.insert(Stat::from(i as u8), boost);
            }
        }
        map
    }
    
    /// Create from a HashMap for compatibility
    pub fn from_hashmap(map: &std::collections::HashMap<Stat, i8>) -> Self {
        let mut array = Self::default();
        for (&stat, &boost) in map {
            array.insert(stat, boost);
        }
        array
    }
    
    /// Constructor that accepts anything that can convert to Self
    pub fn new<T: Into<Self>>(value: T) -> Self {
        value.into()
    }
    
    /// Iterator over non-zero stat boosts (for applying changes)
    pub fn iter_changes(&self) -> impl Iterator<Item = (Stat, i8)> + '_ {
        (0..8).filter_map(move |i| {
            let boost = self.0[i];
            if boost != 0 {
                Some((Stat::from(i as u8), boost))
            } else {
                None
            }
        })
    }
}

// Try implementing conversion for various integer types that might be used in tests
impl From<std::collections::HashMap<Stat, i8>> for StatBoostArray {
    fn from(map: std::collections::HashMap<Stat, i8>) -> Self {
        Self::from_hashmap(&map)
    }
}

impl From<std::collections::HashMap<Stat, i16>> for StatBoostArray {
    fn from(map: std::collections::HashMap<Stat, i16>) -> Self {
        let mut array = Self::default();
        for (stat, boost) in map {
            array.insert(stat, boost as i8);
        }
        array
    }
}

impl From<std::collections::HashMap<Stat, i32>> for StatBoostArray {
    fn from(map: std::collections::HashMap<Stat, i32>) -> Self {
        let mut array = Self::default();
        for (stat, boost) in map {
            array.insert(stat, boost as i8);
        }
        array
    }
}

impl From<std::collections::HashMap<Stat, i64>> for StatBoostArray {
    fn from(map: std::collections::HashMap<Stat, i64>) -> Self {
        let mut array = Self::default();
        for (stat, boost) in map {
            array.insert(stat, boost as i8);
        }
        array
    }
}

impl From<std::collections::HashMap<Stat, isize>> for StatBoostArray {
    fn from(map: std::collections::HashMap<Stat, isize>) -> Self {
        let mut array = Self::default();
        for (stat, boost) in map {
            array.insert(stat, boost as i8);
        }
        array
    }
}

/// Implement From trait for automatic conversion from HashMap reference
impl From<&std::collections::HashMap<Stat, i8>> for StatBoostArray {
    fn from(map: &std::collections::HashMap<Stat, i8>) -> Self {
        Self::from_hashmap(map)
    }
}

/// Implement Deref to HashMap for backwards compatibility in tests
impl std::ops::Deref for StatBoostArray {
    type Target = [i8; 8];
    
    fn deref(&self) -> &Self::Target {
        &self.0
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