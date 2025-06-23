//! # Tapu Simu
//! 
//! A format-aware Pokemon battle simulator designed for multi-format support.
//! This simulator supports Singles, Doubles, and VGC formats with position-based
//! targeting and comprehensive battle mechanics.
//! 
//! ## Key Features
//! 
//! - **Multi-Format Support**: Singles, Doubles, VGC formats
//! - **Position-Based Targeting**: All moves use explicit position targeting
//! - **Format-Aware Architecture**: Battle logic adapts to the active format
//! - **Modern Design**: Built from the ground up with V2 principles
//! - **No Legacy Compatibility**: Clean, focused implementation
//! 
//! ## Architecture Overview
//! 
//! - `core`: Core types and abstractions (state, instructions, battle format)
//! - `engine`: Battle mechanics implementation
//! - `data`: Pokemon data integration with Pokemon Showdown
//! - `testing`: Testing utilities and framework
//! 
//! ## Example Usage
//! 
//! ```rust
//! use tapu_simu::{BattleFormat, BattleState, MoveChoice};
//! use tapu_simu::core::move_choice::{MoveIndex, PokemonIndex};
//! use tapu_simu::core::battle_format::{BattlePosition, SideReference};
//! 
//! // Create a new singles battle using modern BattleState
//! let mut battle_state = BattleState::new(BattleFormat::gen9_ou());
//! 
//! // Create move choices
//! let move1 = MoveChoice::new_move(
//!     MoveIndex::M0, 
//!     vec![BattlePosition::new(SideReference::SideTwo, 0)]
//! );
//! let move2 = MoveChoice::new_move(
//!     MoveIndex::M0, 
//!     vec![BattlePosition::new(SideReference::SideOne, 0)]
//! );
//! ```

// Core modules
pub mod core;
pub mod engine;
pub mod data;
pub mod testing;
pub mod generation;
pub mod io;
pub mod types;
pub mod config;
pub mod simulator;
pub mod builders;

// Modern API exports (primary interfaces)
pub use core::battle_format::{BattleFormat, BattlePosition, SideReference, FormatType};
pub use core::battle_environment::{
    Player, RandomPlayer, FirstMovePlayer, DamageMaximizer,
    BattleEnvironment, BattleResult, TurnInfo, ParallelBattleResults,
    run_parallel_battles_with_states, run_battle_from_state
};
pub use core::instructions::{BattleInstruction, BattleInstructions, PokemonInstruction, FieldInstruction, StatusInstruction, StatsInstruction};
pub use core::move_choice::MoveChoice;
pub use core::battle_state::BattleState;

pub use generation::{Generation, GenerationMechanics, GenerationBattleMechanics};

// Modern API re-exports
pub use simulator::{Simulator, WinRate, BenchmarkResult};
pub use config::{Config, ConfigBuilder};
pub use builders::{BattleBuilder, FormatBuilder, TeamBuilder};

// Test framework re-export
pub use testing::framework::{TestFramework, ContactStatusResult};

// Engine module re-exports for testing
pub use engine::combat::damage_calc;
pub use engine::combat::type_effectiveness;
pub use engine::mechanics::items;
pub use engine::turn::end_of_turn;

// No compile-time feature restrictions - everything handled at runtime

