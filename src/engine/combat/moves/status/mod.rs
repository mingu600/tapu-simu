//! # Status-related Move Effects
//!
//! This module contains all move effects that modify Pokemon status conditions,
//! stats, or provide healing effects.

pub mod status_effects;
pub mod stat_modifying;
pub mod healing;
pub mod item_interaction;

pub use status_effects::*;
pub use stat_modifying::*;
pub use healing::*;
pub use item_interaction::*;