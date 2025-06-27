//! # Terrain Type System
//!
//! This module defines the Terrain enum used throughout the battle system
//! for terrain conditions that affect the battlefield.

use serde::{Deserialize, Serialize};

/// Terrain conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Terrain {
    None,
    Electric,
    ElectricTerrain, // Alias for Electric
    Grassy,
    GrassyTerrain, // Alias for Grassy
    Misty,
    MistyTerrain, // Alias for Misty
    Psychic,
    PsychicTerrain, // Alias for Psychic
}

impl From<u8> for Terrain {
    fn from(value: u8) -> Self {
        match value {
            0 => Terrain::None,
            1 => Terrain::Electric,
            2 => Terrain::ElectricTerrain,
            3 => Terrain::Grassy,
            4 => Terrain::GrassyTerrain,
            5 => Terrain::Misty,
            6 => Terrain::MistyTerrain,
            7 => Terrain::Psychic,
            8 => Terrain::PsychicTerrain,
            _ => Terrain::None, // Default fallback
        }
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Terrain::None
    }
}