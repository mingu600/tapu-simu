//! # Accuracy and Miss Mechanics Tests
//!
//! This module contains tests for accuracy mechanics and move effects that
//! interact with missing, including:
//! - Move accuracy modifications
//! - Miss-triggered effects
//! - Weather accuracy interactions
//! - Ability accuracy interactions
//! - PP interactions with missing
//! - Status interactions with missing
//!
//! These tests verify that accuracy calculations and miss mechanics work
//! correctly in various scenarios and validate proper probability branching.

mod utils;

use std::collections::HashMap;

use tapu_simu::core::battle_format::{BattlePosition, SideReference};
use tapu_simu::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, PokemonStatus, StatusInstruction,
    VolatileStatus, Weather,
};

use utils::{PokemonSpec, Positions, TestBuilder};

// ============================================================================
// Compound Eyes and Accuracy Tests
// ============================================================================

/// Test that Compound Eyes doesn't cause instructions with more than 100% accuracy
/// Compound Eyes increases accuracy by 30%, but this shouldn't create impossible probabilities
#[test]
fn test_compound_eyes_does_not_cause_instructions_with_more_than_100_percent() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Should cap at 100%
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 22,
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("compound eyes doesn't exceed 100%")
        .unwrap()
        .team_one(
            PokemonSpec::new("Butterfree")
                .ability("Compound Eyes")
                .moves(vec!["Tackle"]), // 100% accuracy move
        )
        .team_two(PokemonSpec::new("Charizard"))
        .turn_one_move("Tackle")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Flinching on Moves That Can Miss
// ============================================================================

/// Test flinching move that can miss
/// When a flinching move misses, the flinch effect shouldn't occur
#[test]
fn test_flinching_on_move_that_can_miss() {
    // Air Slash has 95% accuracy and 30% flinch chance
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 5.0, // Miss chance
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 87,
                previous_hp: None,
            })],
            affected_positions: vec![Positions::SIDE_ONE_0],
        },
        BattleInstructions {
            percentage: 66.5, // Hit, no flinch (95% * 70%)
            instruction_list: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 42,
                    previous_hp: None,
                }),
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_ONE_0,
                    amount: 87,
                    previous_hp: None,
                }),
            ],
            affected_positions: vec![Positions::SIDE_TWO_0, Positions::SIDE_ONE_0],
        },
        BattleInstructions {
            percentage: 28.5, // Hit with flinch (95% * 30%)
            instruction_list: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 42,
                    previous_hp: None,
                }),
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: Positions::SIDE_TWO_0,
                    status: VolatileStatus::Flinch,
                    duration: Some(1),
                    previous_had_status: false,
                    previous_duration: None,
                }),
            ],
            affected_positions: vec![Positions::SIDE_TWO_0],
        },
    ];

    TestBuilder::new("flinching on move that can miss")
        .unwrap()
        .team_one(PokemonSpec::new("Swanna").moves(vec!["Air Slash"]))
        .team_two(PokemonSpec::new("Metagross").moves(vec!["Iron Head"]))
        .turn_with_moves("Air Slash", "Iron Head")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Weather Accuracy Interactions
// ============================================================================

/// Test Blizzard in hail (perfect accuracy)
/// Blizzard normally has 70% accuracy but becomes 100% accurate in hail
#[test]
fn test_blizzard_in_hail() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0, // Perfect accuracy in hail
        instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: Positions::SIDE_TWO_0,
            amount: 61,
            previous_hp: None,
        })],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("blizzard in hail")
        .unwrap()
        .team_one(PokemonSpec::new("Articuno").moves(vec!["Blizzard"]))
        .team_two(PokemonSpec::new("Manaphy"))
        .with_weather(Weather::Hail)
        .turn_one_move("Blizzard")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Item Theft and Miss Interactions
// ============================================================================

/// Test Magician doesn't steal if move misses
/// Magician ability should only activate on hit, not on miss
#[test]
fn test_magician_does_not_steal_if_move_misses() {
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
                    amount: 72, // Fire Blast damage to Manaphy
                    previous_hp: None,
                }),
                // Manaphy loses its item
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: Positions::SIDE_TWO_0,
                    new_item: None,
                    previous_item: Some("Leftovers".to_string()),
                }),
                // Delphox gains the stolen item via Magician
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: Positions::SIDE_ONE_0,
                    new_item: Some("Leftovers".to_string()),
                    previous_item: None,
                }),
            ],
            affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
        },
    ];

    TestBuilder::new("magician doesn't steal on miss")
        .unwrap()
        .team_one(
            PokemonSpec::new("Delphox")
                .ability("Magician")
                .moves(vec!["Fire Blast"]),
        )
        .team_two(PokemonSpec::new("Manaphy").item("Leftovers"))
        .turn_one_move("Fire Blast")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Item Activation on Miss
// ============================================================================

/// Test Throat Spray with move that can miss
/// Throat Spray should only activate when the sound move hits
#[test]
fn test_throatspray_with_move_that_can_miss() {
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 10.0,         // Miss chance for Supersonic
            instruction_list: vec![], // No Throat Spray activation on miss
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 90.0, // Hit chance
            instruction_list: vec![
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: Positions::SIDE_TWO_0,
                    status: VolatileStatus::Confusion,
                    duration: Some(4),
                    previous_had_status: false,
                    previous_duration: None,
                }),
                // Throat Spray activates on successful sound move
                BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
                    target: Positions::SIDE_ONE_0,
                    new_item: None,
                    previous_item: Some("Throat Spray".to_string()),
                }),
                BattleInstruction::Stats(
                    tapu_simu::core::instructions::StatsInstruction::BoostStats {
                        target: Positions::SIDE_ONE_0,
                        stat_changes: {
                            let mut changes = HashMap::new();
                            changes.insert(tapu_simu::core::instructions::Stat::SpecialAttack, 1);
                            changes
                        },
                        previous_boosts: HashMap::new(),
                    },
                ),
            ],
            affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
        },
    ];

    TestBuilder::new("throat spray with move that can miss")
        .unwrap()
        .team_one(
            PokemonSpec::new("Exploud")
                .item("Throat Spray")
                .moves(vec!["Supersonic"]), // Sound move with 55% accuracy
        )
        .team_two(PokemonSpec::new("Clefable"))
        .turn_one_move("Supersonic")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Ability Activation on Miss
// ============================================================================

/// Test Sand Spit doesn't activate on miss
/// Sand Spit should only activate when the Pokemon is hit by a move
#[test]
fn test_sandspit_does_not_activate_on_miss() {
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 20.0,         // Miss chance for Dynamic Punch
            instruction_list: vec![], // No Sand Spit activation on miss
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 50.0, // Hit, no confusion (80% * 62.5%)
            instruction_list: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 74,
                    previous_hp: None,
                }),
                // Sand Spit activates when hit
                BattleInstruction::Field(
                    tapu_simu::core::instructions::FieldInstruction::Weather {
                        new_weather: Weather::Sandstorm,
                        previous_weather: Weather::None,
                        turns: Some(5),
                        previous_turns: None,
                        source: Some(Positions::SIDE_TWO_0),
                    },
                ),
            ],
            affected_positions: vec![Positions::SIDE_TWO_0],
        },
        BattleInstructions {
            percentage: 30.0, // Hit with confusion (80% * 37.5%)
            instruction_list: vec![
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_TWO_0,
                    amount: 74,
                    previous_hp: None,
                }),
                BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                    target: Positions::SIDE_TWO_0,
                    status: VolatileStatus::Confusion,
                    duration: Some(4),
                    previous_had_status: false,
                    previous_duration: None,
                }),
                // Sand Spit activates when hit
                BattleInstruction::Field(
                    tapu_simu::core::instructions::FieldInstruction::Weather {
                        new_weather: Weather::Sandstorm,
                        previous_weather: Weather::None,
                        turns: Some(5),
                        previous_turns: None,
                        source: Some(Positions::SIDE_TWO_0),
                    },
                ),
            ],
            affected_positions: vec![Positions::SIDE_TWO_0],
        },
    ];

    TestBuilder::new("sand spit doesn't activate on miss")
        .unwrap()
        .team_one(PokemonSpec::new("Machamp").moves(vec!["Dynamic Punch"]))
        .team_two(PokemonSpec::new("Silicobra").ability("Sand Spit"))
        .turn_one_move("Dynamic Punch")
        .branch_on_damage(true)
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Unique Move Miss Interactions
// ============================================================================

/// Test Poltergeist missing
/// Poltergeist only works if target has an item, and can miss normally
#[test]
fn test_poltergeist_missing() {
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 10.0,         // Miss chance
            instruction_list: vec![], // No effect on miss
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 90.0, // Hit chance
            instruction_list: vec![BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 84, // Poltergeist damage when target has item
                previous_hp: None,
            })],
            affected_positions: vec![Positions::SIDE_TWO_0],
        },
    ];

    TestBuilder::new("poltergeist missing")
        .unwrap()
        .team_one(PokemonSpec::new("Drifblim").moves(vec!["Poltergeist"]))
        .team_two(PokemonSpec::new("Clefable").item("Leftovers"))
        .turn_one_move("Poltergeist")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Clanging Soul missing
/// Clanging Soul can miss and shouldn't activate self-boost on miss
#[test]
fn test_clangoroussoul_missing() {
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 15.0,         // Miss chance for Clangorous Soul
            instruction_list: vec![], // No self-boost on miss
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 85.0, // Hit chance
            instruction_list: vec![
                // Self-damage first
                BattleInstruction::Pokemon(PokemonInstruction::Damage {
                    target: Positions::SIDE_ONE_0,
                    amount: 83, // 1/3 max HP damage to self
                    previous_hp: None,
                }),
                // Then stat boosts
                BattleInstruction::Stats(
                    tapu_simu::core::instructions::StatsInstruction::BoostStats {
                        target: Positions::SIDE_ONE_0,
                        stat_changes: {
                            let mut changes = HashMap::new();
                            changes.insert(tapu_simu::core::instructions::Stat::Attack, 1);
                            changes.insert(tapu_simu::core::instructions::Stat::Defense, 1);
                            changes.insert(tapu_simu::core::instructions::Stat::SpecialAttack, 1);
                            changes.insert(tapu_simu::core::instructions::Stat::SpecialDefense, 1);
                            changes.insert(tapu_simu::core::instructions::Stat::Speed, 1);
                            changes
                        },
                        previous_boosts: HashMap::new(),
                    },
                ),
            ],
            affected_positions: vec![Positions::SIDE_ONE_0],
        },
    ];

    TestBuilder::new("clangorous soul missing")
        .unwrap()
        .team_one(PokemonSpec::new("Kommo-o").moves(vec!["Clangorous Soul"]))
        .team_two(PokemonSpec::new("Clefable"))
        .turn_one_move("Clangorous Soul")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// PP and Sleep Interactions
// ============================================================================

/// Test using move while asleep does not decrement PP
/// Moves should not lose PP when used while asleep
#[test]
fn test_using_move_while_asleep_does_not_decrement_pp() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![], // No move execution while asleep
        affected_positions: vec![],
    }];

    TestBuilder::new("move while asleep doesn't use PP")
        .unwrap()
        .team_one(
            PokemonSpec::new("Snorlax")
                .moves(vec!["Body Slam"])
                .status(PokemonStatus::Sleep),
        )
        .team_two(PokemonSpec::new("Clefable"))
        .turn_one_move("Body Slam")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test PP not decremented when flinched
/// Moves should not lose PP when the user is flinched
#[test]
fn test_pp_not_decremented_when_flinched() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // Opponent's move that causes flinch goes first
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 38,
                previous_hp: None,
            }),
            BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                target: Positions::SIDE_ONE_0,
                status: VolatileStatus::Flinch,
                duration: Some(1),
                previous_had_status: false,
                previous_duration: None,
            }),
            // User's move is prevented by flinch - no PP loss
        ],
        affected_positions: vec![Positions::SIDE_ONE_0],
    }];

    TestBuilder::new("PP not decremented when flinched")
        .unwrap()
        .team_one(PokemonSpec::new("Snorlax").moves(vec!["Body Slam"]))
        .team_two(PokemonSpec::new("Togekiss").moves(vec!["Air Slash"]))
        .turn_with_moves("Body Slam", "Air Slash") // Togekiss goes first and flinches
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Wonder Skin Interactions
// ============================================================================

/// Test Wonder Skin against Poison Powder
/// Wonder Skin makes status moves have 50% accuracy when they would hit
#[test]
fn test_wonderskin_against_poisonpowder() {
    let expected_instructions = vec![
        BattleInstructions {
            percentage: 50.0,         // Wonder Skin reduces accuracy to 50%
            instruction_list: vec![], // Miss
            affected_positions: vec![],
        },
        BattleInstructions {
            percentage: 50.0, // Hit through Wonder Skin
            instruction_list: vec![BattleInstruction::Status(StatusInstruction::Apply {
                target: Positions::SIDE_TWO_0,
                status: PokemonStatus::Poison,
                duration: None,
                previous_status: Some(PokemonStatus::None),
                previous_duration: None,
            })],
            affected_positions: vec![Positions::SIDE_TWO_0],
        },
    ];

    TestBuilder::new("wonder skin against poison powder")
        .unwrap()
        .team_one(PokemonSpec::new("Vileplume").moves(vec!["Poison Powder"]))
        .team_two(PokemonSpec::new("Skitty").ability("Wonder Skin"))
        .turn_one_move("Poison Powder")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Crash Move Interactions
// ============================================================================

/// Test crash move into Protect
/// High Jump Kick should cause crash damage when hitting Protect
#[test]
fn test_crash_move_into_protect() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // Protect activates first
            BattleInstruction::Status(StatusInstruction::ApplyVolatile {
                target: Positions::SIDE_TWO_0,
                status: VolatileStatus::Protect,
                duration: Some(1),
                previous_had_status: false,
                previous_duration: None,
            }),
            // High Jump Kick crashes into Protect
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 124, // Half of max HP crash damage
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0, Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("crash move into protect")
        .unwrap()
        .team_one(PokemonSpec::new("Hitmonlee").moves(vec!["High Jump Kick"]))
        .team_two(PokemonSpec::new("Clefable").moves(vec!["Protect"]))
        .turn_with_moves("High Jump Kick", "Protect")
        .expect_instructions(expected_instructions)
        .assert_success();
}

// ============================================================================
// Priority Move Miss Interactions
// ============================================================================

/// Test Sucker Punch fails versus faster attacking move
/// Sucker Punch should fail if the target uses a faster move first
#[test]
fn test_suckerpunch_fails_versus_faster_attacking_move() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // Quick Attack goes first due to priority
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 28,
                previous_hp: None,
            }),
            // Sucker Punch fails because target already moved
        ],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("sucker punch fails vs faster move")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Quick Attack"]))
        .team_two(PokemonSpec::new("Absol").moves(vec!["Sucker Punch"]))
        .turn_with_moves("Quick Attack", "Sucker Punch")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Thunder Clap fails versus faster attacking move
/// Thunder Clap should fail if target has already moved this turn
#[test]
fn test_thunderclap_fails_versus_faster_attacking_move() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // Quick Attack goes first
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_TWO_0,
                amount: 28,
                previous_hp: None,
            }),
            // Thunder Clap fails because target already attacked
        ],
        affected_positions: vec![Positions::SIDE_TWO_0],
    }];

    TestBuilder::new("thunder clap fails vs faster move")
        .unwrap()
        .team_one(PokemonSpec::new("Pikachu").moves(vec!["Quick Attack"]))
        .team_two(PokemonSpec::new("Raikou").moves(vec!["Thunder Clap"]))
        .turn_with_moves("Quick Attack", "Thunder Clap")
        .expect_instructions(expected_instructions)
        .assert_success();
}

/// Test Metal Burst fails when moving first
/// Metal Burst should fail if the user moves before taking damage
#[test]
fn test_metalburst_fails_moving_first() {
    let expected_instructions = vec![BattleInstructions {
        percentage: 100.0,
        instruction_list: vec![
            // Metal Burst fails when moving first (no prior damage)
            // Tackle hits normally after
            BattleInstruction::Pokemon(PokemonInstruction::Damage {
                target: Positions::SIDE_ONE_0,
                amount: 38,
                previous_hp: None,
            }),
        ],
        affected_positions: vec![Positions::SIDE_ONE_0],
    }];

    TestBuilder::new("metal burst fails moving first")
        .unwrap()
        .team_one(
            PokemonSpec::new("Metagross")
                .moves(vec!["Metal Burst"])
                .ev_spread(0, 0, 0, 0, 0, 252), // Max speed to go first
        )
        .team_two(PokemonSpec::new("Snorlax").moves(vec!["Tackle"]))
        .turn_with_moves("Metal Burst", "Tackle")
        .expect_instructions(expected_instructions)
        .assert_success();
}
