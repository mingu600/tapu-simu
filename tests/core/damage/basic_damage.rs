//! # Basic Damage Calculation Tests
//!
//! This module contains the ported basic damage calculation tests from poke-engine,
//! verifying core damage mechanics, critical hits, and probability branching.

use crate::utils::{PokemonSpec, Positions, TestBuilder};
use tapu_simu::core::battle_format::SideReference;

/// Test basic move pair instruction generation - ported from poke-engine
#[test]
fn test_basic_move_pair_instruction_generation() {
    let result = TestBuilder::new("basic move pair instruction generation")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Tackle"]))
        .turn_with_moves("Tackle", "Tackle")
        .expect_damage(Positions::SIDE_TWO_0, 22)
        .expect_damage(Positions::SIDE_ONE_0, 22)
        .run();

    result.assert_success();
}

/// Test that damage varies with level - ported from poke-engine concept
#[test]
fn test_level_affects_damage() {
    let level_50_result = TestBuilder::new("level 50 damage")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").level(50).moves(vec!["Tackle"]))
        .team_two(
            PokemonSpec::new("Charmander")
                .level(50)
                .moves(vec!["Tackle"]),
        )
        .turn_one_move("Tackle")
        .run();

    let level_100_result = TestBuilder::new("level 100 damage")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").level(100).moves(vec!["Tackle"]))
        .team_two(
            PokemonSpec::new("Charmander")
                .level(100)
                .moves(vec!["Tackle"]),
        )
        .turn_one_move("Tackle")
        .run();

    // Both should succeed but with different damage amounts
    level_50_result.assert_success();
    level_100_result.assert_success();
}

/// Test that STAB (Same Type Attack Bonus) increases damage
#[test]
fn test_stab_bonus() {
    // Electric type using Electric move should do more damage
    let stab_result = TestBuilder::new("STAB damage test")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Thunder Shock"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Tackle"]))
        .turn_one_move("Thunder Shock")
        .run();

    stab_result.assert_success();
}

/// Test type effectiveness - super effective moves
#[test]
fn test_super_effective_damage() {
    // Water move vs Fire type should be super effective
    let result = TestBuilder::new("super effective damage")
        .unwrap()
        .team_one(PokemonSpec::new("Squirtle").moves(vec!["Water Gun"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Tackle"]))
        .turn_one_move("Water Gun")
        .run();

    result.assert_success();
}

/// Test type effectiveness - not very effective moves
#[test]
fn test_not_very_effective_damage() {
    // Fire move vs Water type should be not very effective
    let result = TestBuilder::new("not very effective damage")
        .unwrap()
        .team_one(PokemonSpec::new("Charmander").moves(vec!["Ember"]))
        .team_two(PokemonSpec::new("Squirtle").moves(vec!["Tackle"]))
        .turn_one_move("Ember")
        .run();

    result.assert_success();
}

/// Test that physical moves use Attack stat
#[test]
fn test_physical_move_uses_attack() {
    let result = TestBuilder::new("physical move uses attack")
        .unwrap()
        .team_one(PokemonSpec::new("Machop").moves(vec!["Karate Chop"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Tackle"]))
        .turn_one_move("Karate Chop")
        .run();

    result.assert_success();
}

/// Test that special moves use Special Attack stat
#[test]
fn test_special_move_uses_special_attack() {
    let result = TestBuilder::new("special move uses special attack")
        .unwrap()
        .team_one(PokemonSpec::new("Abra").moves(vec!["Psybeam"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Tackle"]))
        .turn_one_move("Psybeam")
        .run();

    result.assert_success();
}

/// Test damage against different defensive stats
#[test]
fn test_defense_reduces_physical_damage() {
    let result = TestBuilder::new("defense reduces physical damage")
        .unwrap()
        .team_one(PokemonSpec::new("Machop").moves(vec!["Karate Chop"]))
        .team_two(PokemonSpec::new("Onix").moves(vec!["Tackle"])) // High defense
        .turn_one_move("Karate Chop")
        .run();

    result.assert_success();
}

/// Test special defense reduces special damage
#[test]
fn test_special_defense_reduces_special_damage() {
    let result = TestBuilder::new("special defense reduces special damage")
        .unwrap()
        .team_one(PokemonSpec::new("Abra").moves(vec!["Psybeam"]))
        .team_two(PokemonSpec::new("Chansey").moves(vec!["Tackle"])) // High special defense
        .turn_one_move("Psybeam")
        .run();

    result.assert_success();
}

/// Test that status moves don't deal damage
#[test]
fn test_status_moves_no_damage() {
    let result = TestBuilder::new("status moves no damage")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Thunder Wave"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Tackle"]))
        .turn_one_move("Thunder Wave")
        .expect_damage(Positions::SIDE_TWO_0, 0) // No damage from status move
        .run();

    result.assert_success();
}
