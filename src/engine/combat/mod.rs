//! # Combat System
//!
//! This module contains the core battle combat mechanics for Tapu Simu.
//! It provides a comprehensive implementation of Pokemon battle calculations
//! with full generation support and format awareness.
//!
//! ## Architecture Overview
//!
//! The combat system is organized into several key components:
//!
//! - **Damage Calculation** (`damage_calc`): Handles all damage calculation
//!   mechanics including type effectiveness, STAB, critical hits, and the
//!   16-roll damage variance system that matches Pokemon's actual mechanics.
//!
//! - **Damage Context** (`damage_context`): Provides a modern context system
//!   for damage calculations that encapsulates attacker, defender, move, field,
//!   and format information in a structured way.
//!
//! - **Move Effects** (`move_effects`): Legacy move effects system for 
//!   compatibility and transition support.
//!
//! - **Moves** (`moves`): Modern move effects implementation organized by
//!   category (damage, status, field, special) with comprehensive coverage
//!   of Pokemon move mechanics.
//!
//! - **Type Effectiveness** (`type_effectiveness`): Implements the type
//!   effectiveness chart with generation-specific variations and support
//!   for custom type charts.
//!
//! ## Key Features
//!
//! - **Format Awareness**: All calculations respect the active battle format
//!   (Singles, Doubles, VGC) and adjust mechanics accordingly.
//!
//! - **Generation Support**: Full compatibility with different Pokemon
//!   generations, including generation-specific damage formulas, critical
//!   hit mechanics, and type effectiveness changes.
//!
//! - **Position-Based Targeting**: All move effects and calculations use
//!   explicit position targeting for multi-format support.
//!
//! - **Comprehensive Move Coverage**: Implements hundreds of move effects
//!   with accurate mechanics matching the official games.
//!
//! ## Usage Example
//!
//! ```rust
//! use tapu_simu::engine::combat::damage_calc::{calculate_damage_with_positions, DamageRolls};
//! use tapu_simu::engine::combat::moves::apply_move_effects;
//! 
//! // Calculate damage between two Pokemon
//! let damage = calculate_damage_with_positions(
//!     &battle_state,
//!     &attacker,
//!     &defender, 
//!     &move_data,
//!     false, // not critical
//!     DamageRolls::Average,
//!     1, // single target
//!     attacker_position,
//!     defender_position,
//! );
//!
//! // Apply move effects
//! let instructions = apply_move_effects(
//!     &battle_state,
//!     &move_data,
//!     user_position,
//!     &target_positions,
//!     &generation,
//!     &move_context,
//!     &repository,
//!     true, // branch on damage
//! )?;
//! ```

pub mod damage_calc;
pub mod damage_context;
pub mod move_effects;
pub mod moves;
pub mod type_effectiveness;

// New centralized systems
pub mod core;
pub mod composers;