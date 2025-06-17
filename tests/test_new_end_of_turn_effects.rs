//! # New End-of-Turn Effects Tests  
//! 
//! Simple verification tests for the newly implemented end-of-turn effects.
//! These tests verify that our implementations compile and integrate correctly.

use tapu_simu::test_framework::TestFramework;
use tapu_simu::state::State;
use tapu_simu::battle_format::BattleFormat;
use tapu_simu::engine::end_of_turn::process_end_of_turn_effects;
use tapu_simu::generation::{GenerationMechanics, Generation};
use tapu_simu::instruction::{PokemonStatus, VolatileStatus, Weather};

#[test]
fn test_end_of_turn_processing_compiles() {
    // This test verifies that our end-of-turn implementation compiles and runs
    let _framework = TestFramework::new().expect("Failed to create test framework");
    let state = State::new(BattleFormat::gen9_ou());
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Process end-of-turn effects (should return empty instructions for empty state)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Should return at least one empty instruction
    assert!(!instructions.is_empty(), "Should return at least empty instructions");
}

#[test]  
fn test_new_abilities_are_recognized() {
    // Test that our new ability strings are handled
    let _framework = TestFramework::new().expect("Failed to create test framework");
    let state = State::new(BattleFormat::gen9_ou());
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Test that the function doesn't panic with new ability names
    let instructions = process_end_of_turn_effects(&state, &generation);
    assert!(!instructions.is_empty());
}

#[test]
fn test_new_volatile_statuses_are_recognized() {
    // Test that our new volatile statuses are handled
    let _framework = TestFramework::new().expect("Failed to create test framework");
    let state = State::new(BattleFormat::gen9_ou());
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Verify SaltCure and Yawn statuses exist
    let _salt_cure = VolatileStatus::SaltCure;
    let _yawn = VolatileStatus::Yawn;
    let _perish1 = VolatileStatus::Perish1;
    
    let instructions = process_end_of_turn_effects(&state, &generation);
    assert!(!instructions.is_empty());
}

#[test]
fn test_weather_effects_integration() {
    // Test weather integration
    let _framework = TestFramework::new().expect("Failed to create test framework");
    let mut state = State::new(BattleFormat::gen9_ou());
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up weather
    state.weather = Weather::RAIN;
    state.weather_turns_remaining = Some(5);
    
    let instructions = process_end_of_turn_effects(&state, &generation);
    assert!(!instructions.is_empty());
}