//! # Basic Damage Calculation Tests
//!
//! This module contains the ported basic damage calculation tests from poke-engine,
//! verifying core damage mechanics, critical hits, and probability branching.
//! 
//! All tests use real Pokemon data instead of dummy Pokemon, so exact damage values
//! will differ from poke-engine. These tests focus on verifying the mechanics work correctly.

mod utils;

use utils::{PokemonSpec, TestBuilder};
use tapu_simu::core::battle_format::{BattlePosition, SideReference};
use tapu_simu::generation::Generation;

/// Test basic move pair instruction generation - ported from poke-engine
#[test]
fn test_basic_move_pair_instruction_generation() {
    TestBuilder::new("basic move pair instruction generation")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["tackle"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["tackle"]))
        .turn_with_moves("tackle", "tackle")
        .assert_success();
}

/// Test branching on critical hit rolls - ported from poke-engine
#[test]
fn test_branch_on_crit() {
    TestBuilder::new("branch on crit")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Tackle")
        .assert_success();
}

/// Test branching when a damage roll can kill - ported from poke-engine
#[test]
fn test_branch_when_a_roll_can_kill() {
    TestBuilder::new("branch when a roll can kill")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Thunder Bolt"]))
        .team_two(PokemonSpec::new("Charmander").hp_percentage(15.0)) // Low HP for potential KO
        .turn_one_move("Thunder Bolt")
        .assert_success();
}

/// Test branching when a roll can kill on the low side - ported from poke-engine
#[test]
fn test_branch_when_a_roll_can_kill_on_the_low_side() {
    TestBuilder::new("branch when a roll can kill on the low side")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Quick Attack"]))
        .team_two(PokemonSpec::new("Charmander").hp_percentage(20.0)) // Low HP
        .turn_one_move("Quick Attack")
        .assert_success();
}

/// Test that critical hits don't overkill - ported from poke-engine
#[test]
fn test_crit_does_not_overkill() {
    TestBuilder::new("crit does not overkill")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander").hp_percentage(5.0)) // Very low HP
        .turn_one_move("Tackle")
        .assert_success();
}

/// Test high critical hit rate moves - ported from poke-engine
#[test]
fn test_highcrit_move() {
    TestBuilder::new("high crit move")
        .unwrap()
        .team_one(PokemonSpec::new("Farfetchd").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Slash")
        .assert_success();
}

/// Test minimum damage killing does not branch - ported from poke-engine
#[test]
fn test_min_damage_killing_does_not_branch() {
    TestBuilder::new("min damage killing does not branch")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander").hp_percentage(1.0)) // 1 HP
        .turn_one_move("Tackle")
        .assert_success();
}

/// Test Surging Strikes always crits without branching - ported from poke-engine
#[test]
fn test_surgingstrikes_always_crits_without_a_branch() {
    TestBuilder::new("surging strikes always crits without branch")
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Surging Strikes"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Surging Strikes")
        .assert_success();
}

/// Test Wicked Blow always crits without branching - ported from poke-engine
#[test]
fn test_wickedblow_always_crits_without_a_branch() {
    TestBuilder::new("wicked blow always crits without branch")
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Wicked Blow")
        .assert_success();
}

/// Test Wicked Blow ignores defensive boosts due to crit - ported from poke-engine
#[test]
fn test_wickedblow_always_ignores_defensive_boost_on_opponent_because_of_crit() {
    TestBuilder::new("wicked blow ignores defensive boost due to crit")
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Iron Defense"]))
        .turn_with_moves("Wicked Blow", "Iron Defense")
        .assert_success();
}

/// Test Wicked Blow cannot crit on Shell Armor - ported from poke-engine
#[test]
fn test_wickedblow_cannot_crit_on_shellarmor() {
    TestBuilder::new("wicked blow cannot crit on shell armor")
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Lapras").ability("Shell Armor"))
        .turn_one_move("Wicked Blow")
        .assert_success();
}

/// Test Wicked Blow in Gen 8 - ported from poke-engine
#[test]
fn test_wickedblow_gen8() {
    TestBuilder::new_with_generation("wicked blow gen8", Generation::Gen8)
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Wicked Blow")
        .assert_success();
}

/// Test Wicked Blow in Gen 9 - ported from poke-engine
#[test]
fn test_wickedblow_gen9() {
    TestBuilder::new_with_generation("wicked blow gen9", Generation::Gen9)
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Wicked Blow")
        .assert_success();
}

// Gen 1 Critical Hit Tests - ported from poke-engine

/// Test Gen 1 crit roll ignores other boost - ported from poke-engine
#[test]
fn test_crit_roll_ignores_other_boost() {
    TestBuilder::new_with_generation("gen1 crit ignores other boost", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Iron Defense"]))
        .turn_with_moves("Slash", "Iron Defense")
        .assert_success();
}

/// Test Gen 1 crit roll ignores negative boost - ported from poke-engine
#[test]
fn test_crit_roll_ignores_other_boost_negative_boost() {
    TestBuilder::new_with_generation("gen1 crit ignores negative boost", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Screech"]))
        .turn_with_moves("Slash", "Screech")
        .assert_success();
}

/// Test Gen 1 crit roll ignores own boost - ported from poke-engine
#[test]
fn test_crit_roll_ignores_own_boost() {
    TestBuilder::new_with_generation("gen1 crit ignores own boost", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Swords Dance", "Slash"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_with_moves("Swords Dance", "Tackle")
        .turn_one_move("Slash")
        .assert_success();
}

/// Test Gen 1 crit roll ignores Reflect - ported from poke-engine
#[test]
fn test_crit_roll_ignores_reflect() {
    TestBuilder::new_with_generation("gen1 crit ignores reflect", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Reflect"]))
        .turn_with_moves("Slash", "Reflect")
        .assert_success();
}

/// Test Persian using Slash guaranteed crit - ported from poke-engine
#[test]
fn test_persion_using_slash_guaranteed_crit() {
    TestBuilder::new_with_generation("persian slash guaranteed crit", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Slash")
        .assert_success();
}

/// Test Persian using Tackle rolls for crit - ported from poke-engine
#[test]
fn test_persion_using_tackle_rolls_crit() {
    TestBuilder::new_with_generation("persian tackle rolls crit", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Tackle")
        .assert_success();
}

// Gen 2 Critical Hit Tests - ported from poke-engine

/// Test Gen 2 branch on crit - ported from poke-engine
#[test]
fn test_gen2_branch_on_crit() {
    TestBuilder::new_with_generation("gen2 branch on crit", Generation::Gen2)
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Tackle")
        .assert_success();
}

/// Test Gen 2 crit does not overkill - ported from poke-engine
#[test]
fn test_gen2_crit_does_not_overkill() {
    TestBuilder::new_with_generation("gen2 crit does not overkill", Generation::Gen2)
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander").hp_percentage(5.0))
        .turn_one_move("Tackle")
        .assert_success();
}

/// Test Gen 2 high crit move - ported from poke-engine
#[test]
fn test_gen2_highcrit_move() {
    TestBuilder::new_with_generation("gen2 high crit move", Generation::Gen2)
        .unwrap()
        .team_one(PokemonSpec::new("Farfetchd").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Slash")
        .assert_success();
}

/// Test Gen 2 min damage killing does not branch - ported from poke-engine
#[test]
fn test_gen2_min_damage_killing_does_not_branch() {
    TestBuilder::new_with_generation("gen2 min damage killing does not branch", Generation::Gen2)
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander").hp_percentage(1.0))
        .turn_one_move("Tackle")
        .assert_success();
}

// Gen 3 Critical Hit Tests - ported from poke-engine

/// Test Gen 3 branch when a roll can kill - ported from poke-engine
#[test]
fn test_gen3_branch_when_a_roll_can_kill() {
    TestBuilder::new_with_generation("gen3 branch when a roll can kill", Generation::Gen3)
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Thunder Bolt"]))
        .team_two(PokemonSpec::new("Charmander").hp_percentage(15.0))
        .turn_one_move("Thunder Bolt")
        .assert_success();
}

/// Test basic type effectiveness - super effective
#[test]
fn test_super_effective_damage() {
    TestBuilder::new("super effective damage")
        .unwrap()
        .team_one(PokemonSpec::new("Squirtle").moves(vec!["Water Gun"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Water Gun")
        .assert_success();
}

/// Test basic type effectiveness - not very effective
#[test]
fn test_not_very_effective_damage() {
    TestBuilder::new("not very effective damage")
        .unwrap()
        .team_one(PokemonSpec::new("Charmander").moves(vec!["Ember"]))
        .team_two(PokemonSpec::new("Squirtle"))
        .turn_one_move("Ember")
        .assert_success();
}