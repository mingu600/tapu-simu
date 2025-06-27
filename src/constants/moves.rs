//! Move-related constants
//! 
//! This module contains all hardcoded constants used in move implementations,
//! extracted from the move files to improve maintainability and prevent magic numbers.

use crate::engine::combat::type_effectiveness::PokemonType;

// =============================================================================
// DAMAGE CALCULATION CONSTANTS
// =============================================================================

/// Standard damage variance range (85% to 100% of calculated damage)
pub const DAMAGE_VARIANCE_MIN: f32 = 0.85;
pub const DAMAGE_VARIANCE_MAX: f32 = 1.0;

/// Number of damage rolls for variance calculation
pub const DAMAGE_ROLL_COUNT: usize = 16;

/// Minimum damage percentage (85%)
pub const MIN_DAMAGE_PERCENT: u8 = 85;

/// Minimum damage (1 HP)
pub const MIN_DAMAGE: i16 = 1;

/// Maximum damage roll index (100% roll)
pub const MAX_DAMAGE_ROLL_INDEX: usize = 15;

/// Average damage roll index (approximately 92.5%)
pub const AVERAGE_DAMAGE_ROLL_INDEX: usize = 7;

/// Damage roll increment (1% per roll)
pub const DAMAGE_ROLL_INCREMENT: f32 = 0.01;

/// Damage roll start percentage (85%)
pub const DAMAGE_ROLL_START: f32 = 0.85;

// =============================================================================
// CRITICAL HIT CONSTANTS
// =============================================================================

/// Critical hit multiplier for Gen 4+ (1.5x)
pub const CRITICAL_HIT_MULTIPLIER: f32 = 1.5;

/// Critical hit multiplier for Gen 1-3 (2.0x)
pub const CRITICAL_HIT_MULTIPLIER_LEGACY: f32 = 2.0;

// Generation-specific base critical hit rates
/// Gen 1 base critical hit calculation uses Speed/2/256
pub const GEN1_CRIT_SPEED_DIVISOR: i16 = 2;
pub const GEN1_CRIT_RATE_DIVISOR: f32 = 256.0;
pub const GEN1_HIGH_CRIT_MULTIPLIER: i16 = 8;
pub const GEN1_MAX_CRIT_RATE: f32 = 255.0 / 256.0;

/// Gen 2 base critical hit rate (17/256 ≈ 6.64%)
pub const GEN2_BASE_CRIT_RATE: f32 = 17.0 / 256.0;
/// Gen 2 high critical hit rate (+1 stage = 1/8 = 12.5%)
pub const GEN2_HIGH_CRIT_RATE: f32 = 1.0 / 8.0;

/// Gen 3-5 base critical hit rate (1/16 = 6.25%)
pub const GEN3_5_BASE_CRIT_RATE: f32 = 1.0 / 16.0;

/// Gen 6 base critical hit rate (1/16 = 6.25%)
pub const GEN6_BASE_CRIT_RATE: f32 = 1.0 / 16.0;

/// Gen 7-9 base critical hit rate (1/24 ≈ 4.17%)
pub const GEN7_9_BASE_CRIT_RATE: f32 = 1.0 / 24.0;

// Critical hit stage probabilities for different generations
/// Gen 2 critical hit stage rates
pub const GEN2_CRIT_STAGES: &[f32] = &[
    17.0 / 256.0,  // Stage 0: ~6.64%
    1.0 / 8.0,     // Stage 1: 12.5%
    1.0 / 4.0,     // Stage 2: 25%
    85.0 / 256.0,  // Stage 3: ~33.2%
    1.0 / 2.0,     // Stage 4+: 50% (cap)
];

/// Gen 3-5 critical hit stage rates
pub const GEN3_5_CRIT_STAGES: &[f32] = &[
    1.0 / 16.0,    // Stage 0: 6.25%
    1.0 / 8.0,     // Stage 1: 12.5%
    1.0 / 4.0,     // Stage 2: 25%
    1.0 / 3.0,     // Stage 3: ~33.33%
    1.0 / 2.0,     // Stage 4+: 50% (cap)
];

/// Gen 6 critical hit stage rates
pub const GEN6_CRIT_STAGES: &[f32] = &[
    1.0 / 16.0,    // Stage 0: 6.25%
    1.0 / 8.0,     // Stage 1: 12.5%
    1.0 / 2.0,     // Stage 2: 50%
    1.0,           // Stage 3+: 100% (always crit)
];

/// Gen 7-9 critical hit stage rates
pub const GEN7_9_CRIT_STAGES: &[f32] = &[
    1.0 / 24.0,    // Stage 0: ~4.17%
    1.0 / 8.0,     // Stage 1: 12.5%
    1.0 / 2.0,     // Stage 2: 50%
    1.0,           // Stage 3+: 100% (always crit)
];

// High critical hit ratio moves (+1 crit stage)
pub const HIGH_CRIT_MOVES: &[&str] = &[
    "slash",
    "razorleaf",
    "crabhammer", 
    "karatechop",
    "aerialace",
    "airslash",
    "attackorder",
    "crosschop",
    "leafblade",
    "nightslash",
    "psychocut",
    "shadowclaw",
    "spacialrend",
    "stoneedge",
];

// Guaranteed critical hit moves (always crit)
pub const GUARANTEED_CRIT_MOVES: &[&str] = &[
    "frostbreath",
    "stormthrow", 
    "wickedblow",
    "surgingstrikes",
    "flowertrick",
];

// Gen 1 high critical hit moves (different list than modern)
pub const GEN1_HIGH_CRIT_MOVES: &[&str] = &[
    "slash",
    "razorleaf", 
    "crabhammer",
    "karatechop",
];

// Gen 2 high critical hit moves
pub const GEN2_HIGH_CRIT_MOVES: &[&str] = &[
    "slash",
    "razorleaf", 
    "crabhammer",
    "karatechop",
    "aerialace", // Added in Gen 3 but should work in Gen 2 fallback
];

// =============================================================================
// MOVE POWER CONSTANTS
// =============================================================================

/// Base power for Weather Ball in different weather conditions
pub const WEATHER_BALL_BOOSTED_POWER: u16 = 2;

/// Power multiplier for Facade when user has status condition
pub const FACADE_STATUS_MULTIPLIER: u16 = 2;

/// Power multiplier for Hex against statused targets
pub const HEX_STATUS_MULTIPLIER: u16 = 2;

// =============================================================================
// HP THRESHOLD CONSTANTS FOR VARIABLE POWER MOVES
// =============================================================================

/// HP thresholds for Reversal and Flail power calculation
pub const REVERSAL_HP_THRESHOLDS: &[(f32, u16)] = &[
    (0.0208, 200),   // <= 1/48 HP = 200 power
    (0.0417, 150),   // <= 1/24 HP = 150 power  
    (0.1042, 100),   // <= 1/9.6 HP = 100 power
    (0.2083, 80),    // <= 1/4.8 HP = 80 power
    (0.3542, 40),    // <= 17/48 HP = 40 power
    (1.0, 20),       // > 17/48 HP = 20 power
];

/// Weight thresholds for Grass Knot and Low Kick power calculation
pub const WEIGHT_POWER_THRESHOLDS: &[(f32, u16)] = &[
    (200.0, 120),    // >= 200.0 kg = 120 power
    (100.0, 100),    // >= 100.0 kg = 100 power
    (50.0, 80),      // >= 50.0 kg = 80 power
    (25.0, 60),      // >= 25.0 kg = 60 power
    (10.0, 40),      // >= 10.0 kg = 40 power
    (0.0, 20),       // < 10.0 kg = 20 power
];

/// Weight ratio thresholds for Heat Crash and Heavy Slam power calculation
pub const WEIGHT_RATIO_POWER_THRESHOLDS: &[(f32, u16)] = &[
    (5.0, 120),      // >= 5x weight ratio = 120 power
    (4.0, 100),      // >= 4x weight ratio = 100 power  
    (3.0, 80),       // >= 3x weight ratio = 80 power
    (2.0, 60),       // >= 2x weight ratio = 60 power
    (0.0, 40),       // < 2x weight ratio = 40 power
];

/// Speed ratio thresholds for Electro Ball power calculation
pub const SPEED_RATIO_POWER_THRESHOLDS: &[(f32, u16)] = &[
    (4.0, 150),      // >= 4x speed ratio = 150 power
    (3.0, 120),      // >= 3x speed ratio = 120 power
    (2.0, 80),       // >= 2x speed ratio = 80 power
    (1.0, 60),       // >= 1x speed ratio = 60 power
    (0.0, 40),       // < 1x speed ratio = 40 power
];

// =============================================================================
// STATUS CONDITION PROBABILITY CONSTANTS
// =============================================================================

/// Standard burn chance for moves like Flamethrower
pub const BURN_CHANCE_STANDARD: u8 = 10;

/// Standard paralysis chance for moves like Thunderbolt
pub const PARALYSIS_CHANCE_STANDARD: u8 = 10;

/// Standard freeze chance for moves like Ice Beam
pub const FREEZE_CHANCE_STANDARD: u8 = 10;

/// Standard poison chance for moves like Sludge Bomb
pub const POISON_CHANCE_STANDARD: u8 = 30;

/// Standard flinch chance for moves like Air Slash
pub const FLINCH_CHANCE_STANDARD: u8 = 30;

/// Dual status effect probabilities for moves like Fire Fang
pub const DUAL_EFFECT_NEITHER: f32 = 81.0;    // 81% chance of neither effect
pub const DUAL_EFFECT_FIRST_ONLY: f32 = 9.0;  // 9% chance of first effect only
pub const DUAL_EFFECT_SECOND_ONLY: f32 = 9.0; // 9% chance of second effect only
pub const DUAL_EFFECT_BOTH: f32 = 1.0;        // 1% chance of both effects

// =============================================================================
// TYPE-SPECIFIC CONSTANTS
// =============================================================================

/// Types that are immune to Electric-type moves
pub const ELECTRIC_IMMUNE_TYPES: &[PokemonType] = &[PokemonType::Ground];

/// Types that resist Poison-type moves
pub const POISON_RESISTANT_TYPES: &[PokemonType] = &[PokemonType::Poison, PokemonType::Steel];

/// Types that can be affected by Freeze-Dry's special effectiveness
pub const FREEZE_DRY_TARGETS: &[PokemonType] = &[PokemonType::Water];

// =============================================================================
// TERRAIN PULSE TYPE MAPPINGS
// =============================================================================

/// Type changes for Terrain Pulse based on active terrain
pub const TERRAIN_PULSE_TYPES: &[(crate::core::instructions::Terrain, PokemonType)] = &[
    (crate::core::instructions::Terrain::Electric, PokemonType::Electric),
    (crate::core::instructions::Terrain::Grassy, PokemonType::Grass),
    (crate::core::instructions::Terrain::Misty, PokemonType::Fairy),
    (crate::core::instructions::Terrain::Psychic, PokemonType::Psychic),
];

// =============================================================================
// WEATHER BALL TYPE MAPPINGS  
// =============================================================================

/// Type changes for Weather Ball based on active weather
pub const WEATHER_BALL_TYPES: &[(crate::core::instructions::Weather, PokemonType)] = &[
    (crate::core::instructions::Weather::Sun, PokemonType::Fire),
    (crate::core::instructions::Weather::HarshSun, PokemonType::Fire),
    (crate::core::instructions::Weather::HarshSunlight, PokemonType::Fire),
    (crate::core::instructions::Weather::Rain, PokemonType::Water),
    (crate::core::instructions::Weather::HeavyRain, PokemonType::Water),
    (crate::core::instructions::Weather::Sand, PokemonType::Rock),
    (crate::core::instructions::Weather::Sandstorm, PokemonType::Rock),
    (crate::core::instructions::Weather::Hail, PokemonType::Ice),
    (crate::core::instructions::Weather::Snow, PokemonType::Ice),
    (crate::core::instructions::Weather::StrongWinds, PokemonType::Flying),
];