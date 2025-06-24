//! Effect composition layer for combining basic effects into complex moves
//! 
//! This module provides composer functions that combine the core battle systems
//! into common move patterns, reducing code duplication and providing consistent
//! behavior across similar moves.

pub mod damage_moves;
pub mod status_moves;
pub mod field_moves;

pub use damage_moves::*;
pub use status_moves::*;
pub use field_moves::*;