//! Generation-specific damage calculation modules
//!
//! This module contains all generation-specific damage calculation
//! implementations, each handling the unique mechanics of their
//! respective Pokemon generations.

pub mod gen1;
pub mod gen2;
pub mod gen3;
pub mod gen4;
pub mod gen56;
pub mod modern;

// Re-export generation-specific calculation functions
pub use gen1::{critical_hit_probability_gen1, calculate_damage_gen1};
pub use gen2::{critical_hit_probability_gen2, calculate_damage_gen2};
pub use gen3::calculate_damage_gen3;
pub use gen4::calculate_damage_gen4;
pub use gen56::calculate_damage_gen56;
pub use modern::calculate_damage_modern_gen789;