//! Core combat systems for centralized battle mechanics
//! 
//! This module contains the centralized battle mechanics that eliminate code
//! duplication across move implementations. These systems handle damage calculation,
//! status effects, contact effects, and field conditions in a consistent manner.

pub mod damage_system;
pub mod status_system;
pub mod contact_effects;
pub mod field_system;

pub use damage_system::*;
pub use status_system::*;
pub use contact_effects::*;
pub use field_system::*;