//! # Weather Type System
//!
//! This module defines the Weather enum used throughout the battle system
//! for weather conditions that affect the battlefield.

use serde::{Deserialize, Serialize};

/// Weather conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Weather {
    None,
    Hail,
    Rain,
    Sandstorm,
    Sand, // Alias for Sandstorm
    Snow,
    Sun,
    HarshSunlight,
    HarshSun, // Alias for HarshSunlight
    HeavyRain,
    StrongWinds,
}

impl From<u8> for Weather {
    fn from(value: u8) -> Self {
        match value {
            0 => Weather::None,
            1 => Weather::Hail,
            2 => Weather::Rain,
            3 => Weather::Sandstorm,
            4 => Weather::Sand,
            5 => Weather::Snow,
            6 => Weather::Sun,
            7 => Weather::HarshSunlight,
            8 => Weather::HarshSun,
            9 => Weather::HeavyRain,
            10 => Weather::StrongWinds,
            _ => Weather::None, // Default fallback
        }
    }
}

impl Default for Weather {
    fn default() -> Self {
        Weather::None
    }
}