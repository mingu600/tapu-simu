//! Test framework validation

mod utils;

use utils::{PokemonSpec, TestBuilder};

/// Test that invalid Pokemon names fail properly
#[test]
#[should_panic(expected = "SpeciesNotFound")]
fn test_invalid_pokemon_fails() {
    TestBuilder::new("invalid pokemon test")
        .unwrap()
        .team_one(PokemonSpec::new("InvalidPokemonName").moves(vec!["tackle"]))
        .team_two(PokemonSpec::new("Pikachu").moves(vec!["tackle"]))
        .turn_with_moves("tackle", "tackle")
        .assert_success();
}

/// Test that invalid move names fail properly
#[test] 
#[should_panic(expected = "not found")]
fn test_invalid_move_fails() {
    TestBuilder::new("invalid move test")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["InvalidMoveName"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["tackle"]))
        .turn_with_moves("InvalidMoveName", "tackle")
        .assert_success();
}

/// Test that valid Pokemon and moves work
#[test]
fn test_valid_setup_works() {
    TestBuilder::new("valid setup test")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["tackle"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["tackle"]))
        .turn_with_moves("tackle", "tackle")
        .assert_success();
}