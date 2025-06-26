//! Item Constants
//!
//! Constants for item effects, multipliers, and thresholds.

/// Stat-boosting item multipliers
pub const ASSAULT_VEST_SPDEF_MULTIPLIER: f32 = 1.5;
pub const ROCKY_HELMET_RECOIL_FRACTION: f32 = 1.0 / 6.0;
pub const LEFTOVERS_HEAL_FRACTION: f32 = 1.0 / 16.0;
pub const BLACK_SLUDGE_HEAL_FRACTION: f32 = 1.0 / 16.0;
pub const BLACK_SLUDGE_DAMAGE_FRACTION: f32 = 1.0 / 8.0;

/// Type-enhancing item multipliers
pub const TYPE_ENHANCING_ITEM_MULTIPLIER: f32 = 1.2;

/// Choice item multipliers
pub const CHOICE_ITEM_ATTACK_MULTIPLIER: f32 = 1.5;

/// Berry activation thresholds
pub const BERRY_ACTIVATION_HP_THRESHOLD: f32 = 0.25; // 25% HP or less
pub const SITRUS_BERRY_HEAL_AMOUNT: u16 = 25; // Fixed 25 HP in newer generations

/// Eviolite multiplier for defensive stats
pub const EVIOLITE_DEF_MULTIPLIER: f32 = 1.5;
pub const EVIOLITE_SPDEF_MULTIPLIER: f32 = 1.5;

/// Life Orb multipliers
pub const LIFE_ORB_DAMAGE_MULTIPLIER: f32 = 1.3;
pub const LIFE_ORB_RECOIL_FRACTION: f32 = 1.0 / 10.0;

/// Expert Belt multiplier for super effective moves
pub const EXPERT_BELT_MULTIPLIER: f32 = 1.2;

/// Weakness Policy stat boost
pub const WEAKNESS_POLICY_BOOST_STAGES: i8 = 2;