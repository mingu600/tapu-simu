//! # Damage Calculation System
//!
//! This module provides a comprehensive damage calculation system for Pokemon battles,
//! with full support for all generations and battle formats. The system is organized
//! into focused sub-modules for maintainability and clarity.
//!
//! ## Architecture
//!
//! - `core`: Main entry points and damage calculation dispatch
//! - `damage_rolls`: Pokemon's 16-roll damage variance system
//! - `critical_hits`: Critical hit probability and mechanics
//! - `modifiers`: Weather, terrain, abilities, items, and other damage modifiers
//! - `generation_mechanics`: Generation-specific damage calculations
//! - `utils`: Utility functions used throughout the system
//!
//! ## Usage
//!
//! Calculate damage between two Pokemon using the main damage calculation function.

pub mod core;
pub mod damage_rolls;
pub mod critical_hits;
pub mod modifiers;
pub mod generation_mechanics;
pub mod utils;

// Re-export main functions and types for convenience
pub use core::{
    calculate_damage_with_positions,
    calculate_critical_hit_probability, 
    should_critical_hit,
    calculate_all_damage_possibilities,
    calculate_damage_summary,
    DamageSummary,
};

pub use damage_rolls::{
    DamageRolls,
    calculate_all_damage_rolls,
    get_damage_for_roll,
    compare_health_with_damage_multiples,
};

pub use critical_hits::critical_hit_probability;

pub use modifiers::{
    get_weather_damage_modifier,
    get_terrain_damage_modifier,
    get_screen_damage_modifier,
    get_spread_move_modifier,
    get_stab_modifier,
    calculate_all_modifiers,
};

pub use utils::poke_round;

// as the name it had in the original damage_calc.rs
pub use core::calculate_damage_with_positions as calculate_damage_with_modern_context;