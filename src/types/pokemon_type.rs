//! # Unified Pokemon Type System
//!
//! This module provides the single source of truth for all Pokemon type operations
//! across the entire battle system, replacing fragmented string-based and
//! duplicate enum approaches.

use crate::types::from_string::FromNormalizedString;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Unified Pokemon type enum with comprehensive conversion support
/// 
/// This replaces all previous PokemonType enums and string-based type handling
/// throughout the codebase. All type operations should use this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PokemonType {
    Normal = 0,
    Fire = 1,
    Water = 2,
    Electric = 3,
    Grass = 4,
    Ice = 5,
    Fighting = 6,
    Poison = 7,
    Ground = 8,
    Flying = 9,
    Psychic = 10,
    Bug = 11,
    Rock = 12,
    Ghost = 13,
    Dragon = 14,
    Dark = 15,
    Steel = 16,
    Fairy = 17,
    /// Special type for moves without a real type (like Struggle)
    Typeless = 18,
}

impl PokemonType {
    /// Convert from normalized string (case-insensitive)
    /// 
    /// Accepts common variations and PS-style names.
    /// Returns None for invalid type names.
    pub fn from_normalized_str(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "normal" => Some(Self::Normal),
            "fire" => Some(Self::Fire),
            "water" => Some(Self::Water),
            "electric" | "electricity" => Some(Self::Electric),
            "grass" => Some(Self::Grass),
            "ice" => Some(Self::Ice),
            "fighting" | "fight" => Some(Self::Fighting),
            "poison" => Some(Self::Poison),
            "ground" => Some(Self::Ground),
            "flying" | "fly" | "bird" => Some(Self::Flying),
            "psychic" => Some(Self::Psychic),
            "bug" => Some(Self::Bug),
            "rock" => Some(Self::Rock),
            "ghost" => Some(Self::Ghost),
            "dragon" => Some(Self::Dragon),
            "dark" => Some(Self::Dark),
            "steel" => Some(Self::Steel),
            "fairy" => Some(Self::Fairy),
            "typeless" | "???" | "unknown" => Some(Self::Typeless),
            _ => None,
        }
    }

    /// Convert to normalized lowercase string
    /// 
    /// This matches Pokemon Showdown conventions and is used
    /// for data storage and network communication.
    /// 
    /// ## Usage
    /// Use this when you need the canonical string representation for:
    /// - Serializing to JSON/data files
    /// - Network communication with Pokemon Showdown
    /// - Database storage and lookup
    /// - File naming and identification
    /// 
    /// ## Example
    /// ```rust
    /// assert_eq!(PokemonType::Fire.to_normalized_str(), "fire");
    /// assert_eq!(PokemonType::Electric.to_normalized_str(), "electric");
    /// ```
    pub fn to_normalized_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Fire => "fire",
            Self::Water => "water",
            Self::Electric => "electric",
            Self::Grass => "grass",
            Self::Ice => "ice",
            Self::Fighting => "fighting",
            Self::Poison => "poison",
            Self::Ground => "ground",
            Self::Flying => "flying",
            Self::Psychic => "psychic",
            Self::Bug => "bug",
            Self::Rock => "rock",
            Self::Ghost => "ghost",
            Self::Dragon => "dragon",
            Self::Dark => "dark",
            Self::Steel => "steel",
            Self::Fairy => "fairy",
            Self::Typeless => "typeless",
        }
    }

    /// Convert to display name (Title Case)
    /// 
    /// Used for user interfaces and human-readable output.
    /// 
    /// ## Usage
    /// Use this when displaying types to users in:
    /// - Battle log messages
    /// - Pokemon stat displays
    /// - Move descriptions
    /// - Error messages and tooltips
    /// - Any user-facing text
    /// 
    /// ## Example
    /// ```rust
    /// assert_eq!(PokemonType::Fire.display_name(), "Fire");
    /// assert_eq!(PokemonType::Fighting.display_name(), "Fighting");
    /// 
    /// // For battle log
    /// println!("{} is super effective against {}!", 
    ///     move_type.display_name(), 
    ///     target_type.display_name());
    /// ```
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Fire => "Fire",
            Self::Water => "Water",
            Self::Electric => "Electric",
            Self::Grass => "Grass",
            Self::Ice => "Ice",
            Self::Fighting => "Fighting",
            Self::Poison => "Poison",
            Self::Ground => "Ground",
            Self::Flying => "Flying",
            Self::Psychic => "Psychic",
            Self::Bug => "Bug",
            Self::Rock => "Rock",
            Self::Ghost => "Ghost",
            Self::Dragon => "Dragon",
            Self::Dark => "Dark",
            Self::Steel => "Steel",
            Self::Fairy => "Fairy",
            Self::Typeless => "Typeless",
        }
    }

    /// Get all standard types (excludes Typeless)
    /// 
    /// Used for iteration over real Pokemon types. Typeless is excluded
    /// as it's only used for special moves like Struggle.
    pub fn all_standard_types() -> [Self; 18] {
        [
            Self::Normal, Self::Fire, Self::Water, Self::Electric,
            Self::Grass, Self::Ice, Self::Fighting, Self::Poison,
            Self::Ground, Self::Flying, Self::Psychic, Self::Bug,
            Self::Rock, Self::Ghost, Self::Dragon, Self::Dark,
            Self::Steel, Self::Fairy,
        ]
    }

    /// Get all types including Typeless
    /// 
    /// Used for internal systems that need to handle all possible type values.
    pub fn all_types() -> [Self; 19] {
        [
            Self::Normal, Self::Fire, Self::Water, Self::Electric,
            Self::Grass, Self::Ice, Self::Fighting, Self::Poison,
            Self::Ground, Self::Flying, Self::Psychic, Self::Bug,
            Self::Rock, Self::Ghost, Self::Dragon, Self::Dark,
            Self::Steel, Self::Fairy, Self::Typeless,
        ]
    }

    /// Get the numeric index for type effectiveness calculations
    /// 
    /// This matches the type effectiveness matrix indices.
    pub fn as_index(&self) -> usize {
        *self as usize
    }
}


/// Implementation of unified string parsing trait
impl FromNormalizedString for PokemonType {
    fn from_normalized_str(s: &str) -> Option<Self> {
        // Delegate to the existing inherent method
        PokemonType::from_normalized_str(s)
    }
    
    fn valid_strings() -> Vec<&'static str> {
        vec![
            "normal", "fire", "water", "electric", "grass", "ice",
            "fighting", "poison", "ground", "flying", "psychic", "bug",
            "rock", "ghost", "dragon", "dark", "steel", "fairy", "typeless"
        ]
    }
}

/// FromStr implementation for string parsing
impl FromStr for PokemonType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_normalized_str(s)
            .ok_or_else(|| format!("Invalid Pokemon type: {}", s))
    }
}

/// Display implementation using display names
impl fmt::Display for PokemonType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_normalized_str() {
        assert_eq!(PokemonType::from_normalized_str("fire"), Some(PokemonType::Fire));
        assert_eq!(PokemonType::from_normalized_str("FIRE"), Some(PokemonType::Fire));
        assert_eq!(PokemonType::from_normalized_str("Fire"), Some(PokemonType::Fire));
        assert_eq!(PokemonType::from_normalized_str("electric"), Some(PokemonType::Electric));
        assert_eq!(PokemonType::from_normalized_str("electricity"), Some(PokemonType::Electric));
        assert_eq!(PokemonType::from_normalized_str("???"), Some(PokemonType::Typeless));
        assert_eq!(PokemonType::from_normalized_str("invalid"), None);
    }

    #[test]
    fn test_to_normalized_str() {
        assert_eq!(PokemonType::Fire.to_normalized_str(), "fire");
        assert_eq!(PokemonType::Electric.to_normalized_str(), "electric");
        assert_eq!(PokemonType::Typeless.to_normalized_str(), "typeless");
    }

    #[test]
    fn test_display_name() {
        assert_eq!(PokemonType::Fire.display_name(), "Fire");
        assert_eq!(PokemonType::Electric.display_name(), "Electric");
        assert_eq!(PokemonType::Typeless.display_name(), "Typeless");
    }

    #[test]
    fn test_all_standard_types() {
        let types = PokemonType::all_standard_types();
        assert_eq!(types.len(), 18);
        assert!(!types.contains(&PokemonType::Typeless));
        assert!(types.contains(&PokemonType::Fire));
    }

    #[test]
    fn test_all_types() {
        let types = PokemonType::all_types();
        assert_eq!(types.len(), 19);
        assert!(types.contains(&PokemonType::Typeless));
        assert!(types.contains(&PokemonType::Fire));
    }


    #[test]
    fn test_from_str() {
        assert_eq!("fire".parse::<PokemonType>().unwrap(), PokemonType::Fire);
        assert!("invalid".parse::<PokemonType>().is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", PokemonType::Fire), "Fire");
        assert_eq!(format!("{}", PokemonType::Electric), "Electric");
    }

    #[test]
    fn test_as_index() {
        assert_eq!(PokemonType::Normal.as_index(), 0);
        assert_eq!(PokemonType::Fire.as_index(), 1);
        assert_eq!(PokemonType::Typeless.as_index(), 18);
    }
}