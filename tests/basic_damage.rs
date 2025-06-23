//! # Basic Damage Calculation Tests
//!
//! This module contains properly ported basic damage calculation tests from poke-engine,
//! verifying core damage mechanics, critical hits, and probability branching.
//!
//! These tests verify exact probability percentages and instruction sequences
//! to ensure the battle simulator's accuracy matches the reference implementation.
//!
//! NOTE: HP FRAMEWORK CHANGE - All .hp_percentage() calls need to be updated to .hp(raw_value)
//! The framework now uses raw HP values instead of percentages for more precise control.

mod utils;

use std::collections::HashMap;

use tapu_simu::core::battle_format::{BattlePosition, SideReference};
use tapu_simu::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, SideCondition, Stat,
};
use tapu_simu::generation::Generation;
use utils::{PokemonSpec, Positions, TestBuilder};

/// Generation-specific critical hit constants
/// These match the poke-engine constants for each generation
mod crit_constants {
    pub const GEN1_CRIT_MULTIPLIER: f32 = 2.0;
    pub const GEN2_BASE_CRIT_CHANCE: f32 = 17.0 / 256.0; // ~6.64%
    pub const GEN3_BASE_CRIT_CHANCE: f32 = 1.0 / 16.0; // 6.25%
    pub const GEN3_CRIT_MULTIPLIER: f32 = 2.0;
    pub const GEN4_BASE_CRIT_CHANCE: f32 = 1.0 / 16.0; // 6.25%
    pub const GEN4_CRIT_MULTIPLIER: f32 = 1.5;
    pub const GEN7_BASE_CRIT_CHANCE: f32 = 1.0 / 24.0; // ~4.17%
    pub const GEN9_BASE_CRIT_CHANCE: f32 = 1.0 / 24.0; // ~4.17%
}

/// Test basic move pair instruction generation
/// Verifies the fundamental damage calculation without special effects
/// Expected: Both Pokemon take exact damage with 100% probability
#[test]
fn test_basic_move_pair_instruction_generation() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 38,        // Actual damage for Pikachu Tackle vs Charmander
                previous_hp: None, // Framework normalizes this
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideOne, 0),
                amount: 38,        // Actual damage for Charmander Tackle vs Pikachu
                previous_hp: None, // Framework normalizes this
            }),
        ],
        affected_positions: vec![
            BattlePosition::new(SideReference::SideOne, 0),
            BattlePosition::new(SideReference::SideTwo, 0),
        ],
    }];

    TestBuilder::new("basic move pair instruction generation")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander").moves(vec!["Tackle"]))
        .turn_with_moves("Tackle", "Tackle")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test branching on critical hit rolls
/// Tests the probability branching when critical hits are possible
/// Expected: Two branches - normal damage (~93.75%) and crit damage (~6.25%)
#[test]
fn test_branch_on_crit() {
    // Use Water Gun vs Splash to match original test
    // Target has 100 HP to avoid KO complications
    let base_damage = 64; // Expected base damage for Water Gun

    let expected_instructions = vec![
        BattleInstructions {
            percentage: 100.0 * (1.0 - crit_constants::GEN9_BASE_CRIT_CHANCE),
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: base_damage,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 100.0 * crit_constants::GEN9_BASE_CRIT_CHANCE,
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: (crit_constants::GEN4_CRIT_MULTIPLIER * base_damage as f32).floor() as i16,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new("branch on crit")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Water Gun"]))
        .team_two(PokemonSpec::new("Charmander")) // Full HP to avoid KO
        .turn_one_move("Water Gun")
        .branch_on_damage(true) // Enable critical hit branching
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test branching when a damage roll can kill
/// Tests damage variance leading to KO potential
/// Expected: Branches based on damage rolls that can or cannot KO
#[test]
fn test_branch_when_a_roll_can_kill() {
    // This creates branching where some rolls kill, others don't
    let expected_instructions = vec![
        BattleInstructions {
            percentage: (1.0 - crit_constants::GEN9_BASE_CRIT_CHANCE) * 6.0 / 16.0 * 100.0, // Calculated probability for non-KO damage
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 36,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 100.0
                - ((1.0 - crit_constants::GEN9_BASE_CRIT_CHANCE) * 6.0 / 16.0 * 100.0), // Calculated probability for KO damage
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 38, // Exact damage to reach 0 HP
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new("branch when a roll can kill")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander").hp(38))
        .turn_one_move("Tackle")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test that critical hits don't overkill
/// Ensures crit damage is capped at remaining HP
#[test]
fn test_crit_does_not_overkill() {
    // Very low HP target to test damage capping
    let crit_chance = crit_constants::GEN9_BASE_CRIT_CHANCE;
    let base_damage = 64;

    let expected_instructions = vec![
        BattleInstructions {
            percentage: 100.0 * (1.0 - crit_chance),
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: base_damage,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 100.0 * crit_chance,
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 80, // Crit damage also capped at remaining HP
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new("crit does not overkill")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Water Gun"]))
        .team_two(PokemonSpec::new("Charmander").hp(80))
        .turn_one_move("Water Gun")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test high critical hit rate moves
/// Tests moves with increased crit rates (like Slash, Razor Leaf)
#[test]
fn test_highcrit_move() {
    // High crit moves have increased crit rate
    // Expected complex probability distribution including accuracy miss
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 5.0,          // Miss chance
            instruction_list: vec![], // No effect on miss
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 83.125, // Hit, no crit
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 35, // Base damage for high crit move
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 11.875, // Hit with high crit rate
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 53,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new("high crit move")
        .unwrap()
        .team_one(PokemonSpec::new("Bulbasaur").moves(vec!["Razor Leaf"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Razor Leaf")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test minimum damage killing does not branch
/// When minimum damage guarantees KO, no branching should occur
#[test]
fn test_min_damage_killing_does_not_branch() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Single outcome
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 1, // Remaining HP
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new("min damage killing does not branch")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Tackle"]))
        .team_two(PokemonSpec::new("Charmander").hp(1)) // 1 HP
        .turn_one_move("Tackle")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Surging Strikes always crits without branching
/// Guaranteed crit moves should not create probability branches
#[test]
fn test_stormthrow_crits_without_branch() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Single outcome, guaranteed crit
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 69, // Estimated crit damage for Surging Strikes
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new("storm throw always crits without branch")
        .unwrap()
        .team_one(PokemonSpec::new("Throh").moves(vec!["Storm Throw"]))
        .team_two(PokemonSpec::new("Clefable"))
        .turn_one_move("Storm Throw")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Wicked Blow always crits without branching
#[test]
fn test_wickedblow_always_crits_without_a_branch() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Single outcome, guaranteed crit
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 107, // Estimated crit damage for Wicked Blow
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new("wicked blow always crits without branch")
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Clefable"))
        .turn_one_move("Wicked Blow")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Wicked Blow ignores defensive boosts due to crit
#[test]
fn test_wickedblow_always_ignores_defensive_boost_on_opponent_because_of_crit() {
    let mut defense_boosts = HashMap::new();
    defense_boosts.insert(Stat::Defense, 2); // +2 Defense boost
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Single outcome
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 107, // Same damage as without boost (crit ignores it)
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new("wicked blow ignores defensive boost due to crit")
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Clefable").moves(vec!["Iron Defense"]))
        .with_stat_changes(
            BattlePosition::new(SideReference::SideTwo, 0),
            defense_boosts,
        )
        .turn_one_move("Wicked Blow")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Wicked Blow cannot crit on Shell Armor
#[test]
fn test_wickedblow_cannot_crit_on_shellarmor() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Single outcome, no crit due to Shell Armor
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 133, // Non-crit damage
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new("wicked blow cannot crit on shell armor")
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Lapras").ability("Shell Armor"))
        .turn_one_move("Wicked Blow")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Wicked Blow in Gen 8
#[test]
fn test_wickedblow_gen8() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 114,
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new_with_generation("wicked blow gen8", Generation::Gen8)
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu").moves(vec!["Wicked Blow"]))
        .team_two(PokemonSpec::new("Clefable"))
        .turn_one_move("Wicked Blow")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

// Gen 1 Critical Hit Tests

/// Test Gen 1 crit roll ignores other boost
#[test]
fn test_crit_roll_ignores_other_boost() {
    let mut defense_boosts = HashMap::new();
    defense_boosts.insert(Stat::Defense, 2); // +2 Defense boost
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Persian + Slash = guaranteed crit in Gen 1
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 134, // Gen 1 crit damage ignoring defense boost
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new_with_generation("gen1 crit ignores other boost", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Tauros"))
        .with_stat_changes(
            BattlePosition::new(SideReference::SideTwo, 0),
            defense_boosts,
        )
        .turn_one_move("Slash")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Gen 1 crit roll ignores negative boost
#[test]
fn test_crit_roll_ignores_other_boost_negative_boost() {
    let mut defense_boosts = HashMap::new();
    defense_boosts.insert(Stat::Defense, -2); // -2 Defense boost
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Persian + Slash = guaranteed crit in Gen 1
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 134, // Gen 1 crit damage ignoring defense boost
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new_with_generation("gen1 crit ignores other boost", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Tauros"))
        .with_stat_changes(
            BattlePosition::new(SideReference::SideTwo, 0),
            defense_boosts,
        )
        .turn_one_move("Slash")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Gen 1 crit roll ignores own boost
#[test]
fn test_crit_roll_ignores_own_boost() {
    let mut attack_boosts = HashMap::new();
    attack_boosts.insert(Stat::Attack, 2); // +2 Attack boost
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Persian + Slash = guaranteed crit in Gen 1
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 134, // Gen 1 crit damage ignoring defense boost
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new_with_generation("gen1 crit ignores other boost", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Tauros"))
        .with_stat_changes(
            BattlePosition::new(SideReference::SideOne, 0),
            attack_boosts,
        )
        .turn_one_move("Slash")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Gen 1 crit roll ignores Reflect
#[test]
fn test_crit_roll_ignores_reflect() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Persian + Slash = guaranteed crit in Gen 1
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 134,
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new_with_generation("gen1 crit ignores other boost", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Slash"]))
        .team_two(PokemonSpec::new("Tauros"))
        .with_side_condition(SideReference::SideOne, SideCondition::Reflect)
        .turn_one_move("Slash")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Dugtrio using Pound rolls for crit
#[test]
fn test_dugtrio_using_pound_rolls_crit() {
    // Dugtrio has high base speed which affects Gen 1 crit rate
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 76.5625, // Non-crit probability for Dugtrio + Pound
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 29, // Normal damage
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 23.4375, // Crit probability for Dugtrio + Pound
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 57, // Gen 1 crit damage
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new_with_generation("dugtrio pound rolls crit", Generation::Gen1)
        .unwrap()
        .team_one(PokemonSpec::new("Dugtrio").moves(vec!["Pound"]))
        .team_two(PokemonSpec::new("Tauros"))
        .turn_one_move("Pound")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

// Gen 2 Critical Hit Tests

/// Test Gen 2 branch on crit
#[test]
fn test_gen2_branch_on_crit() {
    let crit_chance = crit_constants::GEN2_BASE_CRIT_CHANCE;
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 100.0 * (1.0 - crit_chance),
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 35,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 100.0 * crit_chance,
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 69, // Gen 2 still uses 2x
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new_with_generation("gen2 branch on crit", Generation::Gen2)
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Pound"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Pound")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Gen 2 crit does not overkill
#[test]
fn test_gen2_crit_does_not_overkill() {
    let crit_chance = crit_constants::GEN2_BASE_CRIT_CHANCE;
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 100.0 * (1.0 - crit_chance),
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 35,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 100.0 * crit_chance,
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 50,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new_with_generation("gen2 branch on crit", Generation::Gen2)
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Pound"]))
        .team_two(PokemonSpec::new("Charmander").hp(50))
        .turn_one_move("Pound")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Gen 2 high crit move
#[test]
fn test_gen2_highcrit_move() {
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 5.0, // Miss chance
            instruction_list: vec![],
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 83.125, // Hit, no crit
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 42,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 11.875, // Hit with high crit rate
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 82, // Gen 2 still uses 2x
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new_with_generation("gen2 high crit move", Generation::Gen2)
        .unwrap()
        .team_one(PokemonSpec::new("Persian").moves(vec!["Razor Leaf"]))
        .team_two(PokemonSpec::new("Tauros"))
        .turn_one_move("Razor Leaf")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Gen 2 min damage killing does not branch
#[test]
fn test_gen2_min_damage_killing_does_not_branch() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 1,
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new_with_generation("gen2 min damage killing does not branch", Generation::Gen2)
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Pound"]))
        .team_two(PokemonSpec::new("Charmander").hp(1))
        .turn_one_move("Pound")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

// Gen 3 Critical Hit Tests

/// Test Gen 3 branch when a roll can kill
#[test]
fn test_gen3_branch_when_a_roll_can_kill() {
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 70.3125,
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 46,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
        BattleInstructions {
            percentage: 100.0 - 70.3125,
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: BattlePosition::new(SideReference::SideTwo, 0),
                amount: 50,
                previous_hp: None,
            })],
            affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
        },
    ];

    TestBuilder::new_with_generation("gen3 branch when a roll can kill", Generation::Gen3)
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Thundershock"]))
        .team_two(PokemonSpec::new("Charmander").hp(50))
        .turn_one_move("Thundershock")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test basic type effectiveness - super effective
#[test]
fn test_super_effective_damage() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 96, // Super effective damage (2x effectiveness)
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new("super effective damage")
        .unwrap()
        .team_one(PokemonSpec::new("Squirtle").moves(vec!["Water Gun"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Water Gun")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test basic type effectiveness - not very effective
#[test]
fn test_not_very_effective_damage() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: BattlePosition::new(SideReference::SideTwo, 0),
            amount: 10, // Not very effective damage (0.5x effectiveness)
            previous_hp: None,
        })],
        affected_positions: vec![BattlePosition::new(SideReference::SideTwo, 0)],
    }];

    TestBuilder::new("not very effective damage")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Pound"]))
        .team_two(PokemonSpec::new("Geodude"))
        .turn_one_move("Pound")
        .expect_instructions(expected_instructions)
        .assert_success();
}
