//! # Generation X Engine Module
//! 
//! This module contains the battle engine implementation for generations 4-9.
//! It provides all the core battle mechanics and instruction generation.

pub mod instruction_generator;
pub mod damage_calc;
pub mod move_effects;
pub mod format_targeting;
pub mod ps_targeting;
pub mod format_instruction_generator;
pub mod doubles_mechanics;

// Re-exports
pub use instruction_generator::GenerationXInstructionGenerator;
pub use format_targeting::{FormatMoveTargetResolver, AutoTargetingEngine};
pub use format_instruction_generator::FormatInstructionGenerator;
pub use doubles_mechanics::DoublesSpecificMechanics;