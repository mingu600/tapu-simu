//! # Damage-related Move Effects
//!
//! This module contains all move effects that deal direct damage or have
//! damage-related mechanics like recoil, drain, or fixed damage.

pub mod drain;
pub mod recoil;
pub mod fixed_damage;
pub mod multi_hit;
pub mod variable_power;
pub mod self_damage;
pub mod self_destruct;

pub use drain::*;
pub use recoil::*;
pub use fixed_damage::*;
pub use multi_hit::*;
pub use variable_power::*;
pub use self_damage::*;
pub use self_destruct::*;