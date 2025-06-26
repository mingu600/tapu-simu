//! Constants Module
//! 
//! This module contains all game constants organized by category.
//! This replaces hardcoded magic numbers and strings throughout the codebase.

pub mod items;
pub mod moves;

// Re-export commonly used constants
pub use items::*;
pub use moves::*;