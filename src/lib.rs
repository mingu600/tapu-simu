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
//! Create a new singles battle using modern BattleState and position-based move targeting.

// Core modules
pub mod builders;
pub mod config;
pub mod constants;
pub mod core;
pub mod data;
pub mod engine;
pub mod generation;
pub mod io;
pub mod simulator;
pub mod testing;
pub mod types;
pub mod utils;

// Modern API exports (primary interfaces)
pub use core::battle_environment::{
    run_battle_from_state, run_parallel_battles_with_states, BattleEnvironment, BattleResult,
    DamageMaximizer, FirstMovePlayer, ParallelBattleResults, Player, RandomPlayer, TurnInfo,
};
pub use core::battle_format::{BattleFormat, BattlePosition, FormatType, SideReference};
pub use core::battle_state::BattleState;
pub use core::instructions::{
    BattleInstruction, BattleInstructions, FieldInstruction, PokemonInstruction, StatsInstruction,
    StatusInstruction,
};
pub use core::move_choice::MoveChoice;

pub use generation::{Generation, GenerationBattleMechanics, GenerationMechanics};

// Modern API re-exports
pub use builders::{BattleBuilder, FormatBuilder, TeamBuilder};
pub use config::{Config, ConfigBuilder};
pub use simulator::{BenchmarkResult, Simulator, WinRate};

// Test framework re-export
pub use testing::framework::{ContactStatusResult, TestFramework};

// Engine module re-exports for testing
pub use engine::combat::damage;
pub use engine::combat::type_effectiveness;
pub use engine::mechanics::items;
pub use engine::turn::end_of_turn;

// Utility re-exports
pub use utils::normalize_name;

// No compile-time feature restrictions - everything handled at runtime
