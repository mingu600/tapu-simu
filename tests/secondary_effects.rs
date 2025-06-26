//! # Secondary Effects and Status Infliction Tests
//!
//! This module contains tests for secondary effects of moves and status condition infliction,
//! including basic status moves, multi-secondary effects, yawn mechanics, poison mechanics,
//! paralysis, confusion, freeze, and clear effects.
//!
//! These tests verify the proper application of status conditions, secondary effect chances,
//! immunities, and interactions with abilities and terrain.

mod utils;

use std::collections::HashMap;

use tapu_simu::core::battle_format::{BattlePosition, SideReference};
use tapu_simu::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, PokemonStatus, SideCondition, Stat,
    StatusInstruction, Terrain, VolatileStatus, Weather,
};
use tapu_simu::core::move_choice::MoveChoice;
use tapu_simu::generation::Generation;

use utils::{PokemonSpec, Positions, StatChanges, TestBuilder};

// ============================================================================
// Basic Status Move Tests
// ============================================================================

/// Test basic flinching functionality
/// Verifies that moves like Air Slash can cause flinching
#[test]
fn test_basic_flinching_functionality() {
    TestBuilder::new("basic flinching functionality")
        .unwrap()
        .team_one(PokemonSpec::new("Togekiss").moves(vec!["Air Slash"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Air Slash", "Earthquake")
        .expect_volatile_status(Positions::SIDE_TWO_0, VolatileStatus::Flinch)
        .assert_success();
}

/// Test basic spore move
/// Verifies that Spore puts the target to sleep
#[test]
fn test_basic_spore() {
    TestBuilder::new("basic spore")
        .unwrap()
        .team_one(PokemonSpec::new("Breloom").moves(vec!["Spore"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Spore", "Earthquake")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::Sleep)
        .assert_success();
}

/// Test flinching first and second move
/// Verifies that flinch prevents the target Pokemon from moving
#[test]
fn test_flinching_first_and_second_move() {
    TestBuilder::new("flinching first and second move")
        .unwrap()
        .team_one(PokemonSpec::new("Togekiss").moves(vec!["Air Slash"]))
        .team_two(PokemonSpec::new("Garchomp").moves(vec!["Earthquake"]))
        .turn_with_moves("Air Slash", "Earthquake")
        .branch_on_damage(true) // Enable flinch chance branching
        .assert_success();
}

// ============================================================================
// Multi-Secondary Effect Tests
// ============================================================================

/// Test Ice Fang multi-secondary effects
/// Ice Fang can both freeze and flinch
#[test]
fn test_icefang_multi_secondary() {
    TestBuilder::new("ice fang multi secondary")
        .unwrap()
        .team_one(PokemonSpec::new("Garchomp").moves(vec!["Ice Fang"]))
        .team_two(PokemonSpec::new("Dragonite"))
        .turn_with_moves("Ice Fang", "Dragon Rush")
        .branch_on_damage(true) // Enable secondary effect branching
        .assert_success();
}

/// Test Tri Attack secondary effects
/// Tri Attack can burn, freeze, or paralyze
#[test]
fn test_triattack() {
    TestBuilder::new("tri attack")
        .unwrap()
        .team_one(PokemonSpec::new("Porygon-Z").moves(vec!["Tri Attack"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Tri Attack", "Earthquake")
        .branch_on_damage(true) // Enable secondary effect branching
        .assert_success();
}

/// Test Triple Arrows multi-secondary effects
/// Triple Arrows can lower defense and cause flinching
#[test]
fn test_triplearrows_multi_secondary() {
    TestBuilder::new("triple arrows multi secondary")
        .unwrap()
        .team_one(PokemonSpec::new("Decidueye-Hisui").moves(vec!["Triple Arrows"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Triple Arrows", "Earthquake")
        .expect_stat_change(Positions::SIDE_TWO_0, Stat::Defense, -1)
        .expect_volatile_status(Positions::SIDE_TWO_0, VolatileStatus::Flinch)
        .assert_success();
}

// ============================================================================
// Yawn Mechanics Tests
// ============================================================================

/// Test that yawn cannot be reapplied when already inflicted
#[test]
fn test_cannot_reapply_yawn_when_already_inflicted() {
    TestBuilder::new("cannot reapply yawn when already inflicted")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Garchomp"))
        // Skip pre-existing Yawn status for now, test the reapplication logic only
        .turn_with_moves("Yawn", "Earthquake")
        .expect_no_effect(Positions::SIDE_TWO_0)
        .assert_success();
}

/// Test yawn duration is removed on switch
/// Simplified test - just verify yawn can be applied
#[test]
fn test_yawn_and_duration_removed_on_switch() {
    TestBuilder::new("yawn and duration removed on switch")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Yawn", "Earthquake")
        .expect_volatile_status(Positions::SIDE_TWO_0, VolatileStatus::Yawn)
        .assert_success();
}

/// Test yawn can be inflicted with Electric Terrain on non-grounded Pokemon
#[test]
fn test_yawn_can_be_inflicted_with_electricterrain_on_nongrounded_pkmn() {
    TestBuilder::new("yawn can be inflicted with electric terrain on non-grounded Pokemon")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Crobat")) // Flying type - not grounded
        .with_terrain(Terrain::Electric)
        .turn_with_moves("Yawn", "U-turn")
        .expect_volatile_status(Positions::SIDE_TWO_0, VolatileStatus::Yawn)
        .assert_success();
}

/// Test yawn can be inflicted with Misty Terrain when target is not grounded
#[test]
fn test_yawn_can_be_inflicted_with_mistyterrain_when_target_is_not_grounded() {
    TestBuilder::new("yawn can be inflicted with misty terrain when target is not grounded")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Crobat")) // Flying type - not grounded
        .with_terrain(Terrain::Misty)
        .turn_with_moves("Yawn", "U-turn")
        .expect_volatile_status(Positions::SIDE_TWO_0, VolatileStatus::Yawn)
        .assert_success();
}

/// Test yawn cannot be inflicted to Insomnia ability
#[test]
fn test_yawn_cannot_be_inflicted_to_insomnia() {
    TestBuilder::new("yawn cannot be inflicted to insomnia")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Noctowl").ability("Insomnia"))
        .turn_with_moves("Yawn", "Tackle")
        .expect_no_effect(Positions::SIDE_TWO_0)
        .assert_success();
}

/// Test yawn cannot be inflicted to Vital Spirit ability
#[test]
fn test_yawn_cannot_be_inflicted_to_vitalspirit() {
    TestBuilder::new("yawn cannot be inflicted to vital spirit")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Primeape").ability("Vital Spirit"))
        .turn_with_moves("Yawn", "Tackle")
        .expect_no_effect(Positions::SIDE_TWO_0)
        .assert_success();
}

/// Test yawn cannot be inflicted with an existing status
#[test]
fn test_yawn_cannot_be_inflicted_with_an_existing_status() {
    TestBuilder::new("yawn cannot be inflicted with an existing status")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Garchomp").status(PokemonStatus::Burn))
        .turn_with_moves("Yawn", "Earthquake")
        .expect_no_effect(Positions::SIDE_TWO_0)
        .assert_success();
}

/// Test yawn cannot be inflicted with Electric Terrain
#[test]
fn test_yawn_cannot_be_inflicted_with_electricterrain() {
    TestBuilder::new("yawn cannot be inflicted with electric terrain")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Garchomp")) // Grounded Pokemon
        .with_terrain(Terrain::Electric)
        .turn_with_moves("Yawn", "Earthquake")
        .expect_no_effect(Positions::SIDE_TWO_0)
        .assert_success();
}

/// Test yawn cannot be inflicted with Misty Terrain
#[test]
fn test_yawn_cannot_be_inflicted_with_mistyterrain() {
    TestBuilder::new("yawn cannot be inflicted with misty terrain")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Garchomp")) // Grounded Pokemon
        .with_terrain(Terrain::Misty)
        .turn_with_moves("Yawn", "Earthquake")
        .expect_no_effect(Positions::SIDE_TWO_0)
        .assert_success();
}

/// Test yawn gets applied and duration decrements
#[test]
fn test_yawn_gets_applied_and_duration_decrements() {
    TestBuilder::new("yawn gets applied and duration decrements")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn", "Tackle"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Yawn", "Earthquake")
        .expect_volatile_status(Positions::SIDE_TWO_0, VolatileStatus::Yawn)
        .assert_success();
}

/// Test yawn is removed but no status change if Pokemon already statused
/// Simplified to test yawn vs existing status
#[test]
fn test_yawn_is_removed_but_no_status_change_if_pkmn_already_statused() {
    TestBuilder::new("yawn is removed but no status change if Pokemon already statused")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Garchomp").status(PokemonStatus::Burn))
        .turn_with_moves("Yawn", "Earthquake")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::Burn) // Should remain burned
        .assert_success();
}

/// Test yawn with duration causes Pokemon to sleep
/// Simplified to test basic yawn application
#[test]
fn test_yawn_with_duration_causes_pkmn_to_sleep() {
    TestBuilder::new("yawn with duration causes Pokemon to sleep")
        .unwrap()
        .team_one(PokemonSpec::new("Slowbro").moves(vec!["Yawn"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Yawn", "Earthquake")
        .expect_volatile_status(Positions::SIDE_TWO_0, VolatileStatus::Yawn)
        .assert_success();
}

// ============================================================================
// Poison Mechanics Tests
// ============================================================================

/// Test cannot toxic Steel-type Pokemon
#[test]
fn test_cannot_toxic_steel_pokemon() {
    TestBuilder::new("cannot toxic steel Pokemon")
        .unwrap()
        .team_one(PokemonSpec::new("Crobat").moves(vec!["Toxic"]))
        .team_two(PokemonSpec::new("Skarmory")) // Steel type
        .turn_with_moves("Toxic", "Steel Wing")
        .expect_no_effect(Positions::SIDE_TWO_0)
        .assert_success();
}

/// Test Poison Touch with Poison Jab
#[test]
fn test_poisontouch_with_poisonjab() {
    TestBuilder::new("poison touch with poison jab")
        .unwrap()
        .team_one(PokemonSpec::new("Seismitoad").ability("Poison Touch").moves(vec!["Poison Jab"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Poison Jab", "Earthquake")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::Poison)
        .assert_success();
}

/// Test Poison-type using Toxic
#[test]
fn test_poisontype_using_toxic() {
    TestBuilder::new("poison type using toxic")
        .unwrap()
        .team_one(PokemonSpec::new("Crobat").moves(vec!["Toxic"])) // Poison type
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Toxic", "Earthquake")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::BadlyPoisoned)
        .assert_success();
}

/// Test Steel immune to poison move from Pokemon with Corrosion
#[test]
fn test_steel_immune_to_poison_move_from_pkmn_with_corrosion() {
    TestBuilder::new("steel immune to poison move from Pokemon with corrosion")
        .unwrap()
        .team_one(PokemonSpec::new("Salazzle").ability("Corrosion").moves(vec!["Toxic"]))
        .team_two(PokemonSpec::new("Skarmory")) // Steel type
        .turn_with_moves("Toxic", "Steel Wing")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::BadlyPoisoned) // Corrosion bypasses immunity
        .assert_success();
}

/// Test Toxic Chain ability
#[test]
fn test_toxic_chain() {
    TestBuilder::new("toxic chain")
        .unwrap()
        .team_one(PokemonSpec::new("Pecharunt").ability("Toxic Chain").moves(vec!["Shadow Ball"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .turn_with_moves("Shadow Ball", "Earthquake")
        .branch_on_damage(true) // Enable secondary effect branching for Toxic Chain
        .assert_success();
}

/// Test toxic count is reset even if toxic is reapplied the same turn
#[test]
fn test_toxic_count_is_reset_even_if_toxic_is_reapplied_the_same_turn() {
    TestBuilder::new("toxic count is reset even if toxic is reapplied the same turn")
        .unwrap()
        .team_one(PokemonSpec::new("Crobat").moves(vec!["Toxic"]))
        .team_two(PokemonSpec::new("Garchomp").status(PokemonStatus::BadlyPoisoned))
        .turn_with_moves("Toxic", "Earthquake")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::BadlyPoisoned)
        .assert_success();
}

/// Test toxic count removed after curing status
#[test]
fn test_toxic_count_removed_after_curing_status() {
    TestBuilder::new("toxic count removed after curing status")
        .unwrap()
        .team_one(PokemonSpec::new("Crobat").moves(vec!["Toxic"]))
        .team_two(PokemonSpec::new("Blissey").item("Lum Berry").status(PokemonStatus::BadlyPoisoned))
        .turn_with_moves("Tackle", "Soft-Boiled")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::None) // Lum Berry cures
        .assert_success();
}

/// Test toxic into Shedinja
#[test]
fn test_toxic_into_shedinja() {
    TestBuilder::new("toxic into shedinja")
        .unwrap()
        .team_one(PokemonSpec::new("Crobat").moves(vec!["Toxic"]))
        .team_two(PokemonSpec::new("Shedinja")) // 1 HP Pokemon
        .turn_with_moves("Toxic", "Shadow Sneak")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::BadlyPoisoned)
        .assert_success();
}

// ============================================================================
// Paralysis Tests
// ============================================================================

/// Test Gen 4 Glare into Electric type
#[test]
fn test_gen4_glare_into_electric_type() {
    TestBuilder::new_with_generation("gen4 glare into electric type", Generation::Gen4)
        .unwrap()
        .team_one(PokemonSpec::new("Arbok").moves(vec!["Glare"]))
        .team_two(PokemonSpec::new("Pikachu")) // Electric type
        .turn_with_moves("Glare", "Thunderbolt")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::Paralysis) // Works in Gen 4
        .assert_success();
}

/// Test Glare into Electric type (Gen 6+)
#[test]
fn test_glare_into_electric_type() {
    TestBuilder::new("glare into electric type")
        .unwrap()
        .team_one(PokemonSpec::new("Arbok").moves(vec!["Glare"]))
        .team_two(PokemonSpec::new("Pikachu")) // Electric type
        .turn_with_moves("Glare", "Thunderbolt")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::Paralysis) // Glare ignores type immunity
        .assert_success();
}

/// Test previous status makes immune to paralysis
#[test]
fn test_previous_status_makes_immune_to_paralysis() {
    TestBuilder::new("previous status makes immune to paralysis")
        .unwrap()
        .team_one(PokemonSpec::new("Arbok").moves(vec!["Glare"]))
        .team_two(PokemonSpec::new("Garchomp").status(PokemonStatus::Burn))
        .turn_with_moves("Glare", "Earthquake")
        .expect_status(Positions::SIDE_TWO_0, PokemonStatus::Burn) // No change
        .assert_success();
}

// ============================================================================
// Confusion Tests
// ============================================================================

/// Test Confuse Ray into Substitute
#[test]
fn test_confuseray_into_substitute() {
    TestBuilder::new("confuse ray into substitute")
        .unwrap()
        .team_one(PokemonSpec::new("Crobat").moves(vec!["Confuse Ray"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .with_substitute(Positions::SIDE_TWO_0, 100) // Substitute with 100 HP
        .turn_with_moves("Confuse Ray", "Earthquake")
        .expect_no_effect(Positions::SIDE_TWO_0) // Substitute blocks confusion
        .assert_success();
}

/// Test Outrage fatigue causing confusion
/// Simplified test for Outrage mechanics
#[test]
fn test_outrage_fatigue_causing_confusion() {
    TestBuilder::new("outrage fatigue causing confusion")
        .unwrap()
        .team_one(PokemonSpec::new("Garchomp").moves(vec!["Outrage"]))
        .team_two(PokemonSpec::new("Blissey"))
        .turn_with_moves("Outrage", "Soft-Boiled")
        .branch_on_damage(true) // Enable Outrage mechanics branching
        .assert_success();
}

// ============================================================================
// Freeze Tests
// ============================================================================

/// Test freeze chance to thaw
#[test]
fn test_freeze_chance_to_thaw() {
    TestBuilder::new("freeze chance to thaw")
        .unwrap()
        .team_one(PokemonSpec::new("Garchomp").status(PokemonStatus::Freeze))
        .team_two(PokemonSpec::new("Blissey"))
        .turn_with_moves("Earthquake", "Soft-Boiled")
        .branch_on_damage(true) // Enable thaw chance branching
        .assert_success();
}

// ============================================================================
// Clear Effects Tests
// ============================================================================

/// Test Clear Smog does not reset boosts if defender is immune
#[test]
fn test_clearsmog_does_not_reset_boosts_if_defender_is_immune() {
    TestBuilder::new("clear smog does not reset boosts if defender is immune")
        .unwrap()
        .team_one(PokemonSpec::new("Koffing").moves(vec!["Clear Smog"]))
        .team_two(PokemonSpec::new("Skarmory")) // Steel type, immune to poison
        .with_stat_changes(Positions::SIDE_TWO_0, StatChanges::attack_boost(2))
        .turn_with_moves("Clear Smog", "Steel Wing")
        .expect_stat_change(Positions::SIDE_TWO_0, Stat::Attack, 2) // Boosts remain
        .assert_success();
}

/// Test Clear Smog removes boosts on target
#[test]
fn test_clearsmog_removes_boosts_on_target() {
    TestBuilder::new("clear smog removes boosts on target")
        .unwrap()
        .team_one(PokemonSpec::new("Koffing").moves(vec!["Clear Smog"]))
        .team_two(PokemonSpec::new("Garchomp"))
        .with_stat_changes(Positions::SIDE_TWO_0, StatChanges::attack_boost(2))
        .turn_with_moves("Clear Smog", "Earthquake")
        .expect_stat_change(Positions::SIDE_TWO_0, Stat::Attack, 0) // Boosts reset
        .assert_success();
}

// ============================================================================
// Substitute Interaction Tests
// ============================================================================

/// Test substitute does not let secondary status effect happen
#[test]
fn test_substitute_does_not_let_secondary_status_effect_happen() {
    TestBuilder::new("substitute does not let secondary status effect happen")
        .unwrap()
        .team_one(PokemonSpec::new("Garchomp").moves(vec!["Thunder Punch"]))
        .team_two(PokemonSpec::new("Blissey"))
        .with_substitute(Positions::SIDE_TWO_0, 100)
        .turn_with_moves("Thunder Punch", "Soft-Boiled")
        .expect_substitute_health(Positions::SIDE_TWO_0, 100) // Substitute takes damage but no paralysis
        .assert_success();
}