//! # Battle Simulation Constants
//!
//! This module contains all the magic numbers and constants used throughout
//! the battle simulation system.

/// Damage calculation constants
pub mod damage {
    /// Number of damage rolls in Pokemon's damage calculation system
    pub const DAMAGE_ROLL_COUNT: usize = 16;
    
    /// Minimum damage percentage (85% of base damage)
    pub const MIN_DAMAGE_PERCENT: u8 = 85;
    
    /// Maximum damage percentage (100% of base damage)
    pub const MAX_DAMAGE_PERCENT: u8 = 100;
    
    /// Damage roll increment as percentage
    pub const DAMAGE_ROLL_INCREMENT: f32 = 0.01;
    
    /// Starting damage percentage for rolls
    pub const DAMAGE_ROLL_START: f32 = 0.85;
    
    /// Average damage roll index (0-indexed, 7th roll)
    pub const AVERAGE_DAMAGE_ROLL_INDEX: usize = 7;
    
    /// Maximum damage roll index (0-indexed, 15th roll)
    pub const MAX_DAMAGE_ROLL_INDEX: usize = 15;
    
    /// Minimum damage that can be dealt
    pub const MIN_DAMAGE: i16 = 1;
}

/// Battle mechanics constants
pub mod mechanics {
    /// Maximum positive stat stage
    pub const MAX_STAT_STAGE: i8 = 6;
    
    /// Maximum negative stat stage
    pub const MIN_STAT_STAGE: i8 = -6;
    
    /// Critical hit damage multiplier (Gen 6+)
    pub const CRITICAL_HIT_RATIO: f64 = 1.5;
    
    /// Legacy critical hit damage multiplier (Gen 1-5)
    pub const LEGACY_CRITICAL_HIT_RATIO: f64 = 2.0;
}

/// Generation-specific critical hit constants
pub mod critical_hits {
    /// Gen 1 critical hit multiplier
    pub const GEN1_CRIT_MULTIPLIER: f32 = 2.0;
    
    /// Gen 2 base critical hit chance
    pub const GEN2_BASE_CRIT_CHANCE: f32 = 17.0 / 256.0; // ~6.64%
    
    /// Gen 3 base critical hit chance
    pub const GEN3_BASE_CRIT_CHANCE: f32 = 1.0 / 16.0; // 6.25%
    
    /// Gen 3 critical hit multiplier
    pub const GEN3_CRIT_MULTIPLIER: f32 = 2.0;
    
    /// Gen 4 base critical hit chance
    pub const GEN4_BASE_CRIT_CHANCE: f32 = 1.0 / 16.0; // 6.25%
    
    /// Gen 4 critical hit multiplier
    pub const GEN4_CRIT_MULTIPLIER: f32 = 1.5;
    
    /// Gen 7 base critical hit chance
    pub const GEN7_BASE_CRIT_CHANCE: f32 = 1.0 / 24.0; // ~4.17%
    
    /// Gen 9 base critical hit chance
    pub const GEN9_BASE_CRIT_CHANCE: f32 = 1.0 / 24.0; // ~4.17%
}