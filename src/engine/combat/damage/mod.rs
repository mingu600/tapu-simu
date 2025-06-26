//! Modular damage calculation system
//!
//! This module provides a clean, modular approach to Pokemon damage calculations
//! while maintaining full API compatibility with the existing system.

pub mod types;
pub mod calculator;
pub mod modifiers;
pub mod generations;
pub mod rolls;
pub mod critical;
pub mod utils;


// Re-export main types and functions for compatibility
pub use types::DamageRolls;
pub use calculator::calculate_damage_with_positions;
pub use rolls::{calculate_all_damage_rolls, get_damage_for_roll, compare_health_with_damage_multiples};
pub use critical::critical_hit_probability;
pub use utils::{poke_round, calculate_final_damage_roll, calculate_final_damage_gen12, calculate_final_damage_gen56};

// Re-export modifiers for compatibility
pub use modifiers::{
    is_weather_negated,
    get_weather_stat_multiplier,
    get_weather_damage_modifier,
    get_screen_damage_modifier,
    get_terrain_damage_modifier,
    is_grounded,
    get_spread_move_modifier,
    get_gen2_item_modifier,
    has_adaptability_ability,
};

// Re-export generation-specific functions for compatibility
pub use generations::{
    critical_hit_probability_gen1,
    critical_hit_probability_gen2,
    calculate_damage_gen1,
    calculate_damage_gen2,
    calculate_damage_gen3,
    calculate_damage_gen4,
    calculate_damage_gen56,
    calculate_damage_modern_gen789,
    calculate_damage,
};