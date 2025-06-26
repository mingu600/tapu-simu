//! Damage modifier system
//!
//! This module contains all the focused modifier modules that handle
//! different aspects of damage calculation modifiers.

pub mod weather;
pub mod terrain;
pub mod field;
pub mod format;
pub mod items;
pub mod abilities;

// Re-export all modifier functions for easy access
pub use weather::{is_weather_negated, get_weather_stat_multiplier, get_weather_damage_modifier};
pub use terrain::{is_grounded, get_terrain_damage_modifier};
pub use field::get_screen_damage_modifier;
pub use format::get_spread_move_modifier;
pub use items::get_gen2_item_modifier;
pub use abilities::has_adaptability_ability;