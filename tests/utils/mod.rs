//! # Test Utilities Module
//! 
//! This module provides the test framework infrastructure for tapu-simu,
//! including test builders, assertions, and common testing patterns.

pub mod framework;
pub mod builders;
pub mod assertions;

// Re-export key types for convenient access
pub use framework::{
    TapuTestFramework, BattleTest, TeamSpec, PokemonSpec, 
    SetupAction, ExpectedOutcome, TestResult
};

pub use builders::{
    TestBuilder, TestScenarios, Positions, StatChanges
};

pub use assertions::{
    BattleAssertions, convenience
};

// Re-export macros
pub use crate::{
    damage_test, status_test, ability_test,
    assert_battle_state, assert_damage_range, assert_stat_changes
};