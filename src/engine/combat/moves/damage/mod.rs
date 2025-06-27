//! # Damage-related Move Effects
//!
//! This module contains all move effects that deal direct damage or have
//! damage-related mechanics like recoil, drain, or fixed damage.

pub mod fixed_damage;
pub mod multi_hit;
pub mod self_targeting;
pub mod variable_power;

pub use fixed_damage::*;
pub use multi_hit::*;
pub use self_targeting::*;
pub use variable_power::*;
