//! Core combat systems for centralized battle mechanics
//! 
//! This module contains the centralized battle mechanics that eliminate code
//! duplication across move implementations. These systems handle damage calculation,
//! status effects, contact effects, and field conditions in a consistent manner.

pub mod damage_system;
pub mod status_system;
pub mod contact_effects;
pub mod field_system;
pub mod move_prevention;
pub mod substitute_protection;
pub mod end_of_turn;
pub mod ability_triggers;

pub use damage_system::*;
pub use status_system::*;
pub use contact_effects::*;
pub use field_system::*;
pub use move_prevention::*;
pub use substitute_protection::*;
pub use end_of_turn::*;
pub use ability_triggers::*;