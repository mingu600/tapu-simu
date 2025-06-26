//! Core types for damage calculation system
//!
//! This module contains the fundamental types and enums used throughout
//! the damage calculation system.


/// DamageRolls enum for consistent damage calculation
/// Matches Pokemon's actual 16-roll system
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DamageRolls {
    Average, // Uses the average of the 8th and 9th damage values (true median)
    Min,     // Uses the minimum damage roll (85%)
    Max,     // Uses the maximum damage roll (100%)
    All,     // Returns all 16 possible damage values
}

impl DamageRolls {
    /// Convert DamageRolls enum to damage multiplier (legacy)
    pub fn as_multiplier(self) -> f32 {
        match self {
            DamageRolls::Average => 0.925, // Keep for backwards compatibility
            DamageRolls::Min => 0.85,
            DamageRolls::Max => 1.0,
            DamageRolls::All => 0.925, // Default to average
        }
    }
}

