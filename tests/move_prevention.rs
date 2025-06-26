//! Integration tests for move prevention system
//! 
//! Tests the poke-engine style move prevention mechanics including status conditions,
//! flinch, and other prevention effects with proper probability branching.

mod utils;

use tapu_simu::core::battle_state::Pokemon;
use tapu_simu::core::battle_format::{BattlePosition, SideReference};
use tapu_simu::core::instructions::{PokemonStatus, VolatileStatus, BattleInstructions, BattleInstruction, PokemonInstruction, StatusInstruction};
use tapu_simu::core::move_choice::{MoveChoice, MoveIndex};
use tapu_simu::engine::combat::core::move_prevention::{cannot_use_move, generate_prevention_instructions, MovePreventionReason};
use tapu_simu::data::showdown_types::MoveData;
use utils::{TestBuilder, PokemonSpec};

#[test]
fn test_flinch_prevents_normal_priority_moves() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .pokemon(SideReference::SideTwo, 0, PokemonSpec::new("Charmander").with_hp(100, 100))
        .build();
    
    // Get the Pokemon and manually add flinch status
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    // Create a modified Pokemon with flinch
    let mut modified_pokemon = pokemon.clone();
    modified_pokemon.volatile_statuses.insert(VolatileStatus::Flinch);
    
    let move_data = MoveData {
        name: "Tackle".to_string(),
        priority: 0, // All moves should be prevented by flinch
        ..Default::default()
    };
    
    let move_choice = MoveChoice::Move {
        move_index: MoveIndex::M1,
        target_positions: vec![],
    };
    
    let result = cannot_use_move(&modified_pokemon, &move_choice, Some(&move_data), state, position);
    assert_eq!(result, Some(MovePreventionReason::Flinch));
}

#[test]
fn test_flinch_prevents_high_priority_moves_too() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .pokemon(SideReference::SideTwo, 0, PokemonSpec::new("Charmander").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    // Create a modified Pokemon with flinch
    let mut modified_pokemon = pokemon.clone();
    modified_pokemon.volatile_statuses.insert(VolatileStatus::Flinch);
    
    let move_data = MoveData {
        name: "Quick Attack".to_string(),
        priority: 1, // High priority should STILL be prevented by flinch
        ..Default::default()
    };
    
    let move_choice = MoveChoice::Move {
        move_index: MoveIndex::M1,
        target_positions: vec![],
    };
    
    let result = cannot_use_move(&modified_pokemon, &move_choice, Some(&move_data), state, position);
    assert_eq!(result, Some(MovePreventionReason::Flinch)); // Flinch prevents ALL moves
}

#[test]
fn test_sleep_prevention_with_wake_up_chance() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .pokemon(SideReference::SideTwo, 0, PokemonSpec::new("Charmander").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    // Create a modified Pokemon with sleep
    let mut modified_pokemon = pokemon.clone();
    modified_pokemon.status = PokemonStatus::Sleep;
    modified_pokemon.status_duration = Some(2); // Second turn of sleep
    
    let move_choice = MoveChoice::Move {
        move_index: MoveIndex::M1,
        target_positions: vec![],
    };
    
    let result = cannot_use_move(&modified_pokemon, &move_choice, None, state, position);
    assert_eq!(result, Some(MovePreventionReason::Sleep { wake_up_chance: 50.0 }));
}

#[test]
fn test_paralysis_prevention() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .pokemon(SideReference::SideTwo, 0, PokemonSpec::new("Charmander").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    // Create a modified Pokemon with paralysis
    let mut modified_pokemon = pokemon.clone();
    modified_pokemon.status = PokemonStatus::Paralysis;
    
    let move_choice = MoveChoice::Move {
        move_index: MoveIndex::M1,
        target_positions: vec![],
    };
    
    let result = cannot_use_move(&modified_pokemon, &move_choice, None, state, position);
    assert_eq!(result, Some(MovePreventionReason::Paralysis));
}

#[test]
fn test_confusion_prevention() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .pokemon(SideReference::SideTwo, 0, PokemonSpec::new("Charmander").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    // Create a modified Pokemon with confusion
    let mut modified_pokemon = pokemon.clone();
    modified_pokemon.volatile_statuses.insert(VolatileStatus::Confusion);
    
    let move_choice = MoveChoice::Move {
        move_index: MoveIndex::M1,
        target_positions: vec![],
    };
    
    let result = cannot_use_move(&modified_pokemon, &move_choice, None, state, position);
    
    // Should return confusion prevention with calculated self-damage
    match result {
        Some(MovePreventionReason::Confusion { self_damage }) => {
            assert!(self_damage > 0);
            assert!(self_damage < modified_pokemon.hp); // Confusion can't kill
        }
        _ => panic!("Expected confusion prevention"),
    }
}

#[test]
fn test_freeze_prevention_with_fire_move() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .pokemon(SideReference::SideTwo, 0, PokemonSpec::new("Charmander").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    // Create a modified Pokemon with freeze
    let mut modified_pokemon = pokemon.clone();
    modified_pokemon.status = PokemonStatus::Freeze;
    
    let move_data = MoveData {
        name: "Flamethrower".to_string(),
        move_type: "Fire".to_string(), // Fire moves should always thaw
        ..Default::default()
    };
    
    let move_choice = MoveChoice::Move {
        move_index: MoveIndex::M1,
        target_positions: vec![],
    };
    
    let result = cannot_use_move(&modified_pokemon, &move_choice, Some(&move_data), state, position);
    assert_eq!(result, Some(MovePreventionReason::Freeze { thaw_chance: 100.0 }));
}

#[test]
fn test_taunt_prevents_status_moves() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .pokemon(SideReference::SideTwo, 0, PokemonSpec::new("Charmander").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    // Create a modified Pokemon with taunt
    let mut modified_pokemon = pokemon.clone();
    modified_pokemon.volatile_statuses.insert(VolatileStatus::Taunt);
    
    let move_data = MoveData {
        name: "Thunder Wave".to_string(),
        category: "Status".to_string(), // Status move should be prevented
        ..Default::default()
    };
    
    let move_choice = MoveChoice::Move {
        move_index: MoveIndex::M1,
        target_positions: vec![],
    };
    
    let result = cannot_use_move(&modified_pokemon, &move_choice, Some(&move_data), state, position);
    assert_eq!(result, Some(MovePreventionReason::Taunt));
}

#[test]
fn test_no_prevention_for_switches() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .pokemon(SideReference::SideTwo, 0, PokemonSpec::new("Charmander").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    // Create a modified Pokemon with multiple status conditions
    let mut modified_pokemon = pokemon.clone();
    modified_pokemon.status = PokemonStatus::Sleep;
    modified_pokemon.volatile_statuses.insert(VolatileStatus::Flinch);
    modified_pokemon.volatile_statuses.insert(VolatileStatus::Confusion);
    
    let switch_choice = MoveChoice::Switch(tapu_simu::core::move_choice::PokemonIndex::P1);
    
    let result = cannot_use_move(&modified_pokemon, &switch_choice, None, state, position);
    assert_eq!(result, None); // Switches should never be prevented
}

#[test]
fn test_prevention_instructions_flinch() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    let prevention = MovePreventionReason::Flinch;
    let instructions = generate_prevention_instructions(prevention, position, pokemon);
    
    // Flinch should be deterministic (100% prevention)
    assert_eq!(instructions.len(), 1);
    assert_eq!(instructions[0].percentage, 100.0);
    assert!(instructions[0].instruction_list.is_empty()); // No special instructions for flinch
}

#[test]
fn test_prevention_instructions_paralysis() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    let prevention = MovePreventionReason::Paralysis;
    let instructions = generate_prevention_instructions(prevention, position, pokemon);
    
    // Should have two instruction sets: move succeeds (75%) and move prevented (25%)
    assert_eq!(instructions.len(), 2);
    
    // Check success branch
    let success_branch = instructions.iter().find(|i| i.percentage == 75.0);
    assert!(success_branch.is_some());
    
    // Check prevention branch
    let prevent_branch = instructions.iter().find(|i| i.percentage == 25.0);
    assert!(prevent_branch.is_some());
}

#[test]
fn test_prevention_instructions_confusion() {
    let mut test = TestBuilder::new()
        .pokemon(SideReference::SideOne, 0, PokemonSpec::new("Pikachu").with_hp(100, 100))
        .build();
    
    let state = test.get_battle_state();
    let position = BattlePosition::new(SideReference::SideOne, 0);
    let pokemon = state.get_pokemon_at_position(position).unwrap();
    
    let prevention = MovePreventionReason::Confusion { self_damage: 25 };
    let instructions = generate_prevention_instructions(prevention, position, pokemon);
    
    // Should have two instruction sets: move succeeds (67%) and move prevented with self-damage (33%)
    assert_eq!(instructions.len(), 2);
    
    // Check success branch
    let success_branch = instructions.iter().find(|i| i.percentage == 67.0);
    assert!(success_branch.is_some());
    
    // Check prevention branch (should include self-damage)
    let prevent_branch = instructions.iter().find(|i| i.percentage == 33.0);
    assert!(prevent_branch.is_some());
    
    // Verify self-damage instruction is present
    if let Some(branch) = prevent_branch {
        let has_damage = branch.instruction_list.iter().any(|inst| {
            matches!(inst, BattleInstruction::Pokemon(PokemonInstruction::Damage { amount: 25, .. }))
        });
        assert!(has_damage, "Confusion prevention should include self-damage");
    }
}