//! # Core Battle Concepts
//!
//! This module contains the fundamental abstractions and data structures that form
//! the foundation of Tapu Simu's battle system. These core concepts are designed
//! with multi-format support and position-based targeting as first-class features.
//!
//! ## Core Components
//!
//! - **Battle Format** (`battle_format`): Defines the rules and constraints for
//!   different battle formats (Singles, Doubles, VGC). Each format specifies the
//!   number of active Pokemon per side, targeting rules, and format-specific mechanics.
//!
//! - **Battle Environment** (`battle_environment`): Provides the high-level battle
//!   orchestration system that manages turn order, player interactions, and battle
//!   progression. Includes player AI implementations and parallel battle execution.
//!
//! - **Battle State** (`battle_state`): The immutable battle state representation
//!   that captures the complete state of a Pokemon battle at any given moment.
//!   Includes Pokemon, field conditions, side conditions, and turn information.
//!
//! - **Instructions** (`instructions`): The instruction system that represents
//!   atomic battle actions. Instructions are generated during move resolution
//!   and applied to transform battle states.
//!
//! - **Move Choice** (`move_choice`): Represents player move selections with
//!   explicit target specification. Supports all move types including switches,
//!   mega evolution, and Z-moves with format-aware targeting.
//!
//! - **Targeting** (`targeting`): Position-based targeting system that handles
//!   target selection and validation across different battle formats.
//!
//! ## Design Principles
//!
//! - **Immutability**: Battle states are immutable; changes create new states
//!   through instruction application.
//!
//! - **Position-Based**: All targeting uses explicit battle positions rather
//!   than implicit opponent references.
//!
//! - **Format-Aware**: Every component understands and respects the active
//!   battle format's rules and constraints.
//!
//! - **Instruction-Driven**: All battle changes flow through the instruction
//!   system for consistency and debuggability.
//!
//! ## Usage Example
//!
//! Create a new battle state and move choice with position-based targeting.

pub mod battle_format;
pub mod battle_environment;
pub mod battle_state;
pub mod instructions;
pub mod move_choice;
pub mod targeting;