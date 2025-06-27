//! # Move Categories Tests
//!
//! This module contains tests for various move categories including:
//! - Multi-hit moves
//! - Variable power moves  
//! - Special mechanics moves
//! - Fixed damage moves
//! - Contact mechanics
//! - Explosive moves
//! - Other special move behaviors
//!
//! These tests verify that move effects work correctly across different
//! move categories and validate proper probability branching.

mod utils;

use std::collections::HashMap;

use tapu_simu::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, Stat, StatsInstruction,
    StatusInstruction, VolatileStatus,
};

use utils::{PokemonSpec, Positions, TestBuilder};

// ============================================================================
// Multi-Hit Move Tests
// ============================================================================

/// Test basic multi-hit move functionality
/// Verifies that multi-hit moves deal damage multiple times
#[test]
fn test_basic_multi_hit_move() {
    // Most multi-hit moves hit 2-5 times with equal probability
    // For testing, we expect deterministic behavior or specific hit counts
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 27, // Single hit damage
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 27, // Second hit
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("basic multi hit move")
        .unwrap()
        .team_one(PokemonSpec::new("Dragapult").moves(vec!["Dragon Darts"]))
        .team_two(PokemonSpec::new("Metagross"))
        .turn_one_move("Dragon Darts")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Skill Link ability always gives 5 hits
/// Skill Link guarantees maximum hits for multi-hit moves
#[test]
fn test_skilllink_always_has_5_hits() {
    // Tail Slap has 85% accuracy, so we expect 15% miss and 85% hit
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 15.0, // Miss chance
            instruction_list: vec![],
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 85.0, // Hit chance
            instruction_list: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 12, // Single hit damage from Tail Slap
                    previous_hp: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 12,
                    previous_hp: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 12,
                    previous_hp: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 12,
                    previous_hp: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 12, // Fifth hit guaranteed by Skill Link
                    previous_hp: None,
                }),
            ],
            affected_positions: vec![],
        }
    ];

    TestBuilder::new("skill link always has 5 hits")
        .unwrap()
        .team_one(
            PokemonSpec::new("Cinccino")
                .ability("Skill Link")
                .moves(vec!["Tail Slap"]),
        )
        .team_two(PokemonSpec::new("Metagross"))
        .turn_one_move("Tail Slap")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Variable Power Move Tests
// ============================================================================

/// Test Bolt Beak - double power if target hasn't moved yet
/// Bolt Beak has increased power when used before the target moves
#[test]
fn test_boltbeak() {
    // When used first in the turn, Bolt Beak should have double power
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 81, // Double power Bolt Beak damage
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 34,
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("bolt beak double power when first")
        .unwrap()
        .team_one(
            PokemonSpec::new("Dracozolt")
                .moves(vec!["Bolt Beak"])
                .ability("Volt Absorb"),
        )
        .team_two(PokemonSpec::new("tangrowth").moves(vec!["Tackle"]))
        .turn_with_moves("Bolt Beak", "Tackle")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Grass Knot base power changing based on weight
/// Grass Knot's power varies based on target's weight
#[test]
fn test_grassknot_basepower_changing_based_on_weight() {
    // Test against a lightweight Pokemon (should have lower power)
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 30, // Lower power against lightweight Pokemon
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("grass knot vs lightweight")
        .unwrap()
        .team_one(PokemonSpec::new("Bulbasaur").moves(vec!["Grass Knot"]))
        .team_two(PokemonSpec::new("Pikachu")) // Lightweight Pokemon
        .turn_one_move("Grass Knot")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Grass Knot maximum damage against heavy Pokemon
#[test]
fn test_grassknot_basepower_changing_to_max_damage() {
    // Test against a very heavy Pokemon (should have maximum power)
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 91, // Maximum power against heavy Pokemon
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("grass knot vs heavyweight")
        .unwrap()
        .team_one(PokemonSpec::new("Bulbasaur").moves(vec!["Grass Knot"]))
        .team_two(PokemonSpec::new("Snorlax")) // Very heavy Pokemon
        .turn_one_move("Grass Knot")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Hard Press - more power the more HP target has
#[test]
fn test_hardpress() {
    // Hard Press deals more damage when target has high HP
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 125, // High damage when target at full HP
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("hard press at full HP")
        .unwrap()
        .team_one(PokemonSpec::new("Annihilape").moves(vec!["Hard Press"]))
        .team_two(PokemonSpec::new("Snorlax")) // Full HP
        .turn_one_move("Hard Press")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Heat Crash with highest base power
#[test]
fn test_heatcrash_highest_base_power() {
    // Heat Crash power based on weight difference - attacker much heavier
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 51, // High power due to large weight difference
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("heat crash highest power")
        .unwrap()
        .team_one(PokemonSpec::new("Snorlax").moves(vec!["Heat Crash"]))
        .team_two(PokemonSpec::new("Manaphy"))
        .turn_one_move("Heat Crash")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Heavy Slam with highest base power
#[test]
fn test_heavyslam_highest_base_power() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 51, // Maximum power Heavy Slam
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("heavy slam highest power")
        .unwrap()
        .team_one(PokemonSpec::new("Snorlax").moves(vec!["Heavy Slam"]))
        .team_two(PokemonSpec::new("Manaphy")) // Much lighter
        .turn_one_move("Heavy Slam")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Heavy Slam with lowest base power
#[test]
fn test_heavyslam_lowest_base_power() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 45, // Minimum power Heavy Slam
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("heavy slam lowest power")
        .unwrap()
        .team_one(PokemonSpec::new("Manaphy").moves(vec!["Heavy Slam"]))
        .team_two(PokemonSpec::new("Snorlax")) // Much heavier than attacker
        .turn_one_move("Heavy Slam")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Low Kick with highest damage
#[test]
fn test_lowkick_basepower_highest_damage() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 332, // Maximum power Low Kick
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("low kick highest damage")
        .unwrap()
        .team_one(PokemonSpec::new("Machop").moves(vec!["Low Kick"]))
        .team_two(PokemonSpec::new("Snorlax")) // Very heavy target
        .turn_one_move("Low Kick")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Low Kick with lowest damage
#[test]
fn test_lowkick_basepower_lowest_damage() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 40, // Minimum power Low Kick
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("low kick lowest damage")
        .unwrap()
        .team_one(PokemonSpec::new("Machop").moves(vec!["Low Kick"]))
        .team_two(PokemonSpec::new("Pikachu")) // Very light target
        .turn_one_move("Low Kick")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Special Mechanics Move Tests
// ============================================================================

/// Test Body Press - uses Defense stat for damage calculation
#[test]
fn test_bodypress() {
    // Body Press uses Defense instead of Attack for damage calculation
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 52, // Damage calculated using Defense stat
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("body press uses defense")
        .unwrap()
        .team_one(PokemonSpec::new("Corviknight").moves(vec!["Body Press"]))
        .team_two(PokemonSpec::new("Metagross"))
        .turn_one_move("Body Press")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Endeavor - reduces target's HP to match user's HP
#[test]
fn test_endeavor() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 179, // Damage to bring target HP down to user's HP (40)
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("endeavor equalizes HP")
        .unwrap()
        .team_one(PokemonSpec::new("Rattata").moves(vec!["Endeavor"]).hp(40))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Endeavor")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Endeavor versus Ghost type (should fail)
#[test]
fn test_endeavor_versus_ghost() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![], // No effect against Ghost type
        affected_positions: vec![],
    }];

    TestBuilder::new("endeavor vs ghost type")
        .unwrap()
        .team_one(PokemonSpec::new("Rattata").moves(vec!["Endeavor"]).hp(40))
        .team_two(PokemonSpec::new("Gastly"))
        .turn_one_move("Endeavor")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Endeavor when user has higher HP than opponent
#[test]
fn test_endeavor_when_higher_hp_than_opponent() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![], // No effect when user HP > target HP
        affected_positions: vec![],
    }];

    TestBuilder::new("endeavor higher HP")
        .unwrap()
        .team_one(PokemonSpec::new("Rattata").moves(vec!["Endeavor"]))
        .team_two(PokemonSpec::new("Charmander").hp(30)) // Lower HP than user
        .turn_one_move("Endeavor")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Final Gambit - deals damage equal to user's current HP, then user faints
#[test]
fn test_finalgambit() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 126, // Damage equal to user's current HP
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 126, // User faints (takes damage equal to their current HP)
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("final gambit")
        .unwrap()
        .team_one(
            PokemonSpec::new("Dodrio")
                .moves(vec!["Final Gambit"])
                .hp(126),
        )
        .team_two(PokemonSpec::new("Clefable"))
        .turn_one_move("Final Gambit")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Final Gambit versus Ghost type
#[test]
fn test_finalgambit_versus_ghost() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // User still faints even though move failed
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 126, // User faints
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0],
    }];

    TestBuilder::new("final gambit vs ghost")
        .unwrap()
        .team_one(
            PokemonSpec::new("Dodrio")
                .moves(vec!["Final Gambit"])
                .hp(126),
        )
        .team_two(PokemonSpec::new("Gastly"))
        .turn_one_move("Final Gambit")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Foul Play - uses target's Attack stat for damage calculation
#[test]
fn test_foulplay() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 84, // Damage calculated using target's Attack stat
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("foul play uses target attack")
        .unwrap()
        .team_one(PokemonSpec::new("Umbreon").moves(vec!["Foul Play"]))
        .team_two(PokemonSpec::new("Machamp")) // High Attack stat
        .turn_one_move("Foul Play")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Hydro Steam - increased power in sun
#[test]
fn test_hydrosteam() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 76, // Increased power in sun instead of reduced
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("hydro steam in sun")
        .unwrap()
        .team_one(PokemonSpec::new("Volcanion").moves(vec!["Hydro Steam"]))
        .team_two(PokemonSpec::new("Venusaur"))
        .with_weather(tapu_simu::core::instructions::Weather::Sun)
        .turn_one_move("Hydro Steam")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Mind Blown - damages user and targets
#[test]
fn test_mindblown() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 125, // High power explosion damage
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 123, // User takes 50% of max HP as damage
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("mind blown damages user")
        .unwrap()
        .team_one(PokemonSpec::new("Blacephalon").moves(vec!["Mind Blown"]))
        .team_two(PokemonSpec::new("Manaphy"))
        .turn_one_move("Mind Blown")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Mind Blown does not overkill user
#[test]
fn test_mindblown_does_not_overkill() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 125, // Normal target damage
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 30, // Damage capped at remaining HP
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("mind blown does not overkill user")
        .unwrap()
        .team_one(
            PokemonSpec::new("Blacephalon")
                .moves(vec!["Mind Blown"])
                .hp(30),
        )
        .team_two(PokemonSpec::new("Manaphy"))
        .turn_one_move("Mind Blown")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Mind Blown into Damp ability
#[test]
fn test_mindblown_into_damp() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![], // No effect due to Damp
        affected_positions: vec![],
    }];

    TestBuilder::new("mind blown into damp")
        .unwrap()
        .team_one(PokemonSpec::new("Blacephalon").moves(vec!["Mind Blown"]))
        .team_two(PokemonSpec::new("Golduck").ability("Damp"))
        .turn_one_move("Mind Blown")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Mind's Eye versus Ghost type
#[test]
fn test_mindseye_versus_ghost_type() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 46, // Normal damage despite Ghost type immunity
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("mind's eye vs ghost")
        .unwrap()
        .team_one(
            PokemonSpec::new("Heatmor")
                .ability("Mind's Eye")
                .moves(vec!["Tackle"]),
        )
        .team_two(PokemonSpec::new("Gengar"))
        .turn_one_move("Tackle")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Moongeist Beam into Ice Scales
#[test]
fn test_moongeistbeam_into_ice_scales() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 168, // Full damage, ignoring Ice Scales
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("moongeist beam ignores ice scales")
        .unwrap()
        .team_one(PokemonSpec::new("Lunala").moves(vec!["Moongeist Beam"]))
        .team_two(PokemonSpec::new("Frosmoth").ability("Ice Scales"))
        .turn_one_move("Moongeist Beam")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Fixed Damage Move Tests
// ============================================================================

/// Test Pain Split - averages HP between user and target
#[test]
fn test_painsplit() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            BattleInstruction::Pokemon(PokemonInstruction::Heal {
                target: Positions::SIDE_ONE_0,
                amount: 13, // User gains HP to reach average
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 13, // Target loses HP to reach average
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("pain split averages HP")
        .unwrap()
        .team_one(
            PokemonSpec::new("Misdreavus")
                .moves(vec!["Pain Split"])
                .hp(54),
        )
        .team_two(PokemonSpec::new("Charmander").hp(80)) // average would be ~67
        .turn_one_move("Pain Split")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Seismic Toss - deals damage equal to user's level
#[test]
fn test_seismictoss() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 100, // Damage equal to user's level
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("seismic toss level damage")
        .unwrap()
        .team_one(
            PokemonSpec::new("Machamp")
                .level(100)
                .moves(vec!["Seismic Toss"]),
        )
        .team_two(PokemonSpec::new("Clefable"))
        .turn_one_move("Seismic Toss")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Seismic Toss does not overkill
#[test]
fn test_seismictoss_does_not_overkill() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 50, // Capped at remaining HP
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("seismic toss does not overkill")
        .unwrap()
        .team_one(
            PokemonSpec::new("Machamp")
                .level(100)
                .moves(vec!["Seismic Toss"]),
        )
        .team_two(PokemonSpec::new("Charmander").hp(50))
        .turn_one_move("Seismic Toss")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Seismic Toss versus Ghost type
#[test]
fn test_seismictoss_versus_ghost_type() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![], // No effect against Ghost type
        affected_positions: vec![],
    }];

    TestBuilder::new("seismic toss vs ghost")
        .unwrap()
        .team_one(PokemonSpec::new("Machamp").moves(vec!["Seismic Toss"]))
        .team_two(PokemonSpec::new("Gastly"))
        .turn_one_move("Seismic Toss")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Super Fang - deals 50% of target's current HP
#[test]
fn test_superfang() {
    // Super Fang has 90% accuracy, so we expect 10% miss and 90% hit
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 10.0, // Miss chance
            instruction_list: vec![],
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 90.0, // Hit chance  
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 109, // 50% of target's current HP
                previous_hp: None,
            })],
            affected_positions: vec![],
        }
    ];

    TestBuilder::new("super fang halves HP")
        .unwrap()
        .team_one(PokemonSpec::new("Rattata").moves(vec!["Super Fang"]))
        .team_two(PokemonSpec::new("Charmander"))
        .turn_one_move("Super Fang")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Super Fang at 1 HP
#[test]
fn test_superfang_at_1hp() {
    // Super Fang has 90% accuracy, so we expect 10% miss and 90% hit
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 10.0, // Miss chance
            instruction_list: vec![],
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 90.0, // Hit chance
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 1,
                previous_hp: None,
            })],
            affected_positions: vec![],
        }
    ];

    TestBuilder::new("super fang at 1 HP")
        .unwrap()
        .team_one(PokemonSpec::new("Rattata").moves(vec!["Super Fang"]))
        .team_two(PokemonSpec::new("Charmander").hp(1))
        .turn_one_move("Super Fang")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Super Fang versus Ghost type
#[test]
fn test_superfang_versus_ghost_type() {
    // Super Fang has 90% accuracy, but does no damage to Ghost types
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 10.0, // Miss chance
            instruction_list: vec![],
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 90.0, // Hit chance, but no effect due to immunity
            instruction_list: vec![],
            affected_positions: vec![],
        }
    ];

    TestBuilder::new("super fang vs ghost")
        .unwrap()
        .team_one(PokemonSpec::new("Rattata").moves(vec!["Super Fang"]))
        .team_two(PokemonSpec::new("Gastly"))
        .turn_one_move("Super Fang")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Contact Mechanics Tests
// ============================================================================

/// Test contact multi-hit move versus Rocky Helmet
/// Each hit should trigger Rocky Helmet damage
#[test]
fn test_contact_multi_hit_move_versus_rockyhelmet() {
    // Tail Slap has 85% accuracy
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 15.0, // Miss chance
            instruction_list: vec![],
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 85.0, // Hit chance
        instruction_list: vec![
            // First hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 11,
                previous_hp: None,
            }),
            // Rocky Helmet retaliation from first hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 48, // Rocky Helmet damage (1/6 of max HP)
                previous_hp: None,
            }),
            // First hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 11,
                previous_hp: None,
            }),
            // Rocky Helmet retaliation from first hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 48, // Rocky Helmet damage (1/6 of max HP)
                previous_hp: None,
            }),
            // First hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 11,
                previous_hp: None,
            }),
            // Rocky Helmet retaliation from first hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 48, // Rocky Helmet damage (1/6 of max HP)
                previous_hp: None,
            }),
            // First hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 11,
                previous_hp: None,
            }),
            // Rocky Helmet retaliation from first hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 48, // Rocky Helmet damage (1/6 of max HP)
                previous_hp: None,
            }),
            // First hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 11,
                previous_hp: None,
            }),
            // Rocky Helmet retaliation from first hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 48, // Rocky Helmet damage (1/6 of max HP)
                previous_hp: None,
            }),
            ],
            affected_positions: vec![],
        }
    ];

    TestBuilder::new("contact multi-hit vs rocky helmet")
        .unwrap()
        .team_one(
            PokemonSpec::new("Cinccino")
                .moves(vec!["Tail Slap"])
                .ability("Skill Link"),
        )
        .team_two(PokemonSpec::new("Skarmory").item("Rocky Helmet"))
        .turn_one_move("Tail Slap")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test multi-hit move where first hit breaks substitute
#[test]
fn test_multi_hit_move_where_first_hit_breaks_substitute() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // First hit: Substitute takes damage and breaks
            BattleInstruction::Pokemon(PokemonInstruction::ChangeSubstituteHealth {
                target: Positions::SIDE_TWO_0,
                new_health: 0,       // Substitute health reduced to 0
                previous_health: 50, // Was 50 before the hit
            }),
            // Substitute breaks - remove the volatile status
            BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                target: Positions::SIDE_TWO_0,
                status: VolatileStatus::Substitute,
                previous_duration: None,
            }),
            // Second hit: Direct damage to Pokemon (69 damage)
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 69,
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("multi-hit breaks substitute")
        .unwrap()
        .team_one(
            PokemonSpec::new("Dragapult")
                .moves(vec!["Dragon Darts"])
                .ability("Clear Body"),
        )
        .team_two(PokemonSpec::new("Manaphy").item("Rocky Helmet"))
        .with_substitute(Positions::SIDE_TWO_0, 50)
        .turn_one_move("Dragon Darts")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test triple multi-hit move vs substitute and Rocky Helmet
#[test]
fn test_triple_multihit_move_versus_substitute_and_rockyhelmet() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // First hit: Substitute takes damage
            BattleInstruction::Pokemon(PokemonInstruction::ChangeSubstituteHealth {
                target: Positions::SIDE_TWO_0,
                new_health: 2,       // Substitute health reduced to 0
                previous_health: 30, // Was 50 before the hit
            }),
            // Second hit: Substitute takes damage and breaks
            BattleInstruction::Pokemon(PokemonInstruction::ChangeSubstituteHealth {
                target: Positions::SIDE_TWO_0,
                new_health: 0,      // Substitute health reduced to 0
                previous_health: 2, // Was 50 before the hit
            }),
            // Substitute breaks - remove the volatile status
            BattleInstruction::Status(StatusInstruction::RemoveVolatile {
                target: Positions::SIDE_TWO_0,
                status: VolatileStatus::Substitute,
                previous_duration: None,
            }),
            // Second hit: Direct damage to Pokemon
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 28,
                previous_hp: None,
            }),
            // Rocky Helmet retaliation from third hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 56, // Rocky Helmet damage (1/6 of max HP)
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("triple multi-hit vs sub and helmet")
        .unwrap()
        .team_one(PokemonSpec::new("Urshifu-Rapid-Strike").moves(vec!["Surging Strikes"]))
        .team_two(PokemonSpec::new("Manaphy").item("Rocky Helmet"))
        .with_substitute(Positions::SIDE_TWO_0, 30)
        .turn_one_move("Surging Strikes")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Explosive Move Tests
// ============================================================================

/// Test Explosion into Damp ability
#[test]
fn test_explosion_into_damp() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![], // No effect due to Damp
        affected_positions: vec![],
    }];

    TestBuilder::new("explosion into damp")
        .unwrap()
        .team_one(PokemonSpec::new("Electrode").moves(vec!["Explosion"]))
        .team_two(PokemonSpec::new("Golduck").ability("Damp"))
        .turn_one_move("Explosion")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Explosion into Ghost type
#[test]
fn test_explosion_into_ghost_type() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // User still faints even though explosion didn't hit
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 261, // User faints
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0],
    }];

    TestBuilder::new("explosion into ghost")
        .unwrap()
        .team_one(PokemonSpec::new("Electrode").moves(vec!["Explosion"]))
        .team_two(PokemonSpec::new("Gastly"))
        .turn_one_move("Explosion")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test fast Explosion makes other side unable to move
#[test]
fn test_fast_explosion_makes_other_side_unable_to_move() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // Explosion damage
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 45, // Heavy explosion damage
                previous_hp: None,
            }),
            // User faints
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 261,
                previous_hp: None,
            }),
            // Opponent's move is cancelled due to target fainting
        ],
        affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("fast explosion cancels opponent move")
        .unwrap()
        .team_one(PokemonSpec::new("Electrode").moves(vec!["Explosion"]))
        .team_two(PokemonSpec::new("Metagross").moves(vec!["Tackle"]))
        .turn_with_moves("Explosion", "Tackle")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Other Special Move Tests
// ============================================================================

/// Test Scale Shot only boosts once per use
#[test]
fn test_scaleshot_only_boosts_once() {
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 10.0,
            instruction_list: vec![],
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 90.0,
            instruction_list: vec![
                // Multiple hits
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 13,
                    previous_hp: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 13,
                    previous_hp: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 13,
                    previous_hp: None,
                }),
                // Only one speed boost and defense drop at the end
                BattleInstruction::Stats(StatsInstruction::BoostStats {
                    target: Positions::SIDE_ONE_0,
                    stat_changes: {
                        let mut changes = HashMap::new();
                        changes.insert(Stat::Speed, 1);
                        changes.insert(Stat::Defense, -1);
                        changes
                    },
                    previous_boosts: HashMap::new(),
                }),
            ],
            affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
        },
    ];

    TestBuilder::new("scale shot boosts once")
        .unwrap()
        .team_one(PokemonSpec::new("Kommo-o").moves(vec!["Scale Shot"]))
        .team_two(PokemonSpec::new("Metagross"))
        .turn_one_move("Scale Shot")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Population Bomb with Wide Lens
#[test]
fn test_population_bomb_with_widelens() {
    // Population Bomb with Wide Lens has 99% accuracy (90% base * 1.1)
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 1.0, // Miss chance  
            instruction_list: vec![],
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 99.0, // Hit chance
        instruction_list: vec![
            // All 10 hits land due to improved accuracy
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8, // Low power per hit
                previous_hp: None,
            }),
            // Repeat for all 10 hits...
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8,
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8, // Low power per hit
                previous_hp: None,
            }),
            // Repeat for all 10 hits...
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8,
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8, // Low power per hit
                previous_hp: None,
            }),
            // Repeat for all 10 hits...
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8,
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8, // Low power per hit
                previous_hp: None,
            }),
            // Repeat for all 10 hits...
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8,
                previous_hp: None,
            }),
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8, // Low power per hit
                previous_hp: None,
            }),
            // Repeat for all 10 hits...
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 8,
                previous_hp: None,
            }),
            ],
            affected_positions: vec![],
        }
    ];

    TestBuilder::new("population bomb with wide lens")
        .unwrap()
        .team_one(
            PokemonSpec::new("Maushold")
                .item("Wide Lens")
                .moves(vec!["Population Bomb"]),
        )
        .team_two(PokemonSpec::new("Metagross"))
        .turn_one_move("Population Bomb")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test basic Levitate ability
#[test]
fn test_basic_levitate() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![], // Ground move has no effect
        affected_positions: vec![],
    }];

    TestBuilder::new("levitate vs ground move")
        .unwrap()
        .team_one(PokemonSpec::new("Diglett").moves(vec!["Earthquake"]))
        .team_two(PokemonSpec::new("Gengar").ability("Levitate"))
        .turn_one_move("Earthquake")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Additional Complex Move Tests
// ============================================================================

/// Test Acrobatics with no item (double power)
#[test]
fn test_acrobatics_no_item() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 47, // Double power when no item
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("acrobatics no item")
        .unwrap()
        .team_one(PokemonSpec::new("Crobat").moves(vec!["Acrobatics"]))
        .team_two(PokemonSpec::new("Metagross"))
        .turn_one_move("Acrobatics")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Acrobatics with item (normal power)
#[test]
fn test_acrobatics_with_item() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 24, // Normal power when holding item
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("acrobatics with item")
        .unwrap()
        .team_one(
            PokemonSpec::new("Crobat")
                .item("Leftovers")
                .moves(vec!["Acrobatics"]),
        )
        .team_two(PokemonSpec::new("Metagross"))
        .turn_one_move("Acrobatics")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Weather Ball in different weather
#[test]
fn test_weatherball_in_rain() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 130, // Double power in weather, becomes Water-type
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("weather ball in rain")
        .unwrap()
        .team_one(PokemonSpec::new("Zapdos").moves(vec!["Weather Ball"]))
        .team_two(PokemonSpec::new("Snorlax"))
        .with_weather(tapu_simu::core::instructions::Weather::Rain)
        .turn_one_move("Weather Ball")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Terrain Pulse in Electric Terrain
#[test]
fn test_terrainpulse_in_electric_terrain() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 129, // Double power in terrain, becomes Electric-type
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("terrain pulse in electric terrain")
        .unwrap()
        .team_one(PokemonSpec::new("Raichu").moves(vec!["Terrain Pulse"]))
        .team_two(PokemonSpec::new("Snorlax"))
        .with_terrain(tapu_simu::core::instructions::Terrain::Electric)
        .turn_one_move("Terrain Pulse")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Facade with status condition (double power)
#[test]
fn test_facade_with_burn() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 85, // Double power when burned
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("facade with burn")
        .unwrap()
        .team_one(
            PokemonSpec::new("Swellow")
                .moves(vec!["Facade"])
                .status(tapu_simu::core::instructions::PokemonStatus::Burn)
                .ability("Guts"),
        )
        .team_two(PokemonSpec::new("Metagross"))
        .turn_one_move("Facade")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Hex with status condition (double power)
#[test]
fn test_hex_with_status() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 172, // Double power against statused Pokemon
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("hex with status")
        .unwrap()
        .team_one(PokemonSpec::new("Mismagius").moves(vec!["Hex"]))
        .team_two(
            PokemonSpec::new("Clefable")
                .status(tapu_simu::core::instructions::PokemonStatus::Paralysis),
        )
        .turn_one_move("Hex")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Wake-Up Slap on sleeping Pokemon (double power)
#[test]
fn test_wakeupslap_on_sleeping() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 153, // Double power against sleeping Pokemon
                previous_hp: None,
            }),
            // Should also cure sleep status
        ],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("wake-up slap on sleeping")
        .unwrap()
        .team_one(PokemonSpec::new("Hariyama").moves(vec!["Wake-Up Slap"]))
        .team_two(
            PokemonSpec::new("Metagross")
                .status(tapu_simu::core::instructions::PokemonStatus::Sleep),
        )
        .turn_one_move("Wake-Up Slap")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Punishment against boosted Pokemon (increased power)
#[test]
fn test_punishment_against_boosted() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 76, // Increased power based on stat boosts
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("punishment against boosted")
        .unwrap()
        .team_one(PokemonSpec::new("Absol").moves(vec!["Punishment"]))
        .team_two(PokemonSpec::new("Clefable"))
        .with_stat_changes(Positions::SIDE_TWO_0, {
            let mut changes = HashMap::new();
            changes.insert(Stat::Attack, 2);
            changes.insert(Stat::Defense, 1);
            changes
        })
        .turn_one_move("Punishment")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Stored Power with stat boosts (increased power)
#[test]
fn test_stored_power_with_boosts() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 74, // Increased power based on user's stat boosts
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("stored power with boosts")
        .unwrap()
        .team_one(PokemonSpec::new("Mewtwo").moves(vec!["Stored Power"]))
        .team_two(PokemonSpec::new("Metagross"))
        .with_stat_changes(Positions::SIDE_ONE_0, {
            let mut changes = HashMap::new();
            changes.insert(Stat::SpecialAttack, 2);
            changes.insert(Stat::Speed, 1);
            changes
        })
        .turn_one_move("Stored Power")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Gyro Ball against faster Pokemon (high power)
#[test]
fn test_gyro_ball_high_power() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 154, // High power against much faster Pokemon
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("gyro ball high power")
        .unwrap()
        .team_one(PokemonSpec::new("Forretress").moves(vec!["Gyro Ball"]))
        .team_two(PokemonSpec::new("Ninjask")) // Very fast Pokemon
        .turn_one_move("Gyro Ball")
        .expect_instructions(expected_instructions)
        .assert_success();
}
