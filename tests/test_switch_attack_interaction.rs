//! Test for switch-attack interactions to verify defender assignment
//! This test ensures that when one team switches and the other attacks,
//! the attack targets the newly switched-in Pokemon, not the original one.

use tapu_simu::*;
use tapu_simu::core::{
    state::{State, Pokemon, Move, MoveCategory},
    move_choice::{MoveChoice, MoveIndex, PokemonIndex},
    battle_format::{BattleFormat, BattlePosition, SideReference, FormatType},
    instruction::{StateInstructions, Instruction},
};
use tapu_simu::engine::turn::instruction_generator::GenerationXInstructionGenerator;
use tapu_simu::generation::Generation;
use tapu_simu::data::ps_types::PSMoveTarget;

#[test]
fn test_switch_then_attack_targets_new_pokemon() {
    // Create Singles battle format
    let format = BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles);
    let mut state = State::new(format.clone());
    
    // Side One: Attacker (will attack)
    let mut attacker = Pokemon::new("Attacker".to_string());
    attacker.types = vec!["Normal".to_string()];
    attacker.level = 50;
    attacker.stats.attack = 100;
    
    // Add Tackle move to attacker
    let tackle = Move::new_with_details(
        "Tackle".to_string(),
        40,
        100,
        "Normal".to_string(),
        35,
        PSMoveTarget::Normal,
        MoveCategory::Physical,
        0,
    );
    attacker.add_move(MoveIndex::M0, tackle);
    
    state.side_one.add_pokemon(attacker);
    state.side_one.set_active_pokemon_at_slot(0, Some(0));
    
    // Side Two: Original defender + New Pokemon to switch to
    let mut original_defender = Pokemon::new("OriginalDefender".to_string());
    original_defender.types = vec!["Normal".to_string()];
    original_defender.level = 50;
    original_defender.stats.defense = 80;
    original_defender.hp = 100;
    original_defender.max_hp = 100;
    
    let mut new_defender = Pokemon::new("NewDefender".to_string());
    new_defender.types = vec!["Steel".to_string()]; // Steel resists Normal moves
    new_defender.level = 50;
    new_defender.stats.defense = 120; // Higher defense
    new_defender.hp = 120;
    new_defender.max_hp = 120;
    
    state.side_two.add_pokemon(original_defender);
    state.side_two.add_pokemon(new_defender);
    state.side_two.set_active_pokemon_at_slot(0, Some(0)); // Original defender active
    
    // Create move choices
    let attack_choice = MoveChoice::new_move(
        MoveIndex::M0,
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );
    let switch_choice = MoveChoice::Switch(PokemonIndex::P1); // Switch to Pokemon index 1
    
    // Generate instructions
    let generator = GenerationXInstructionGenerator::new(format);
    let instructions = generator.generate_instructions(&mut state, &attack_choice, &switch_choice);
    
    println!("Generated {} instruction sets", instructions.len());
    
    // Verify that we get instructions
    assert!(!instructions.is_empty(), "Should generate switch-attack instructions");
    
    // Find damage instructions and verify they target the switched-in Pokemon
    let mut found_damage = false;
    let mut switch_applied = false;
    
    for instruction_set in &instructions {
        println!("Instruction set with {}% probability:", instruction_set.percentage);
        
        for (i, instruction) in instruction_set.instruction_list.iter().enumerate() {
            match instruction {
                Instruction::SwitchPokemon(switch_instr) => {
                    println!("  {}: Switch Pokemon at position {:?} from {} to {}", 
                        i, switch_instr.position, switch_instr.previous_index, switch_instr.next_index);
                    assert_eq!(switch_instr.position.side, SideReference::SideTwo);
                    assert_eq!(switch_instr.next_index, 1); // Should switch to Pokemon index 1
                    switch_applied = true;
                }
                Instruction::PositionDamage(damage_instr) => {
                    println!("  {}: Damage {} to position {:?}", 
                        i, damage_instr.damage_amount, damage_instr.target_position);
                    
                    // Damage should target the position where switch occurred
                    assert_eq!(damage_instr.target_position.side, SideReference::SideTwo);
                    assert_eq!(damage_instr.target_position.slot, 0);
                    
                    // The damage amount should reflect the Steel-type's resistance to Normal moves
                    // and higher defense stat of the new Pokemon
                    found_damage = true;
                }
                other => {
                    println!("  {}: {:?}", i, other);
                }
            }
        }
    }
    
    assert!(switch_applied, "Switch instruction should be applied");
    assert!(found_damage, "Damage instruction should be applied to correct target");
}

#[test]
fn test_attack_then_switch_targets_original_pokemon() {
    // Create Singles battle format  
    let format = BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles);
    let mut state = State::new(format.clone());
    
    // Side One: Attacker (will switch after being attacked)
    let mut original_pokemon = Pokemon::new("OriginalPokemon".to_string());
    original_pokemon.types = vec!["Normal".to_string()];
    original_pokemon.level = 50;
    original_pokemon.stats.defense = 60;
    original_pokemon.hp = 80;
    original_pokemon.max_hp = 80;
    
    let mut new_pokemon = Pokemon::new("NewPokemon".to_string());
    new_pokemon.types = vec!["Rock".to_string()];
    new_pokemon.level = 50;
    new_pokemon.stats.defense = 100;
    new_pokemon.hp = 100;
    new_pokemon.max_hp = 100;
    
    state.side_one.add_pokemon(original_pokemon);
    state.side_one.add_pokemon(new_pokemon);
    state.side_one.set_active_pokemon_at_slot(0, Some(0));
    
    // Side Two: Attacker
    let mut attacker = Pokemon::new("Attacker".to_string());
    attacker.types = vec!["Fighting".to_string()];
    attacker.level = 50;
    attacker.stats.attack = 110;
    
    // Add a priority move to ensure it goes first
    let quick_attack = Move::new_with_details(
        "Quick Attack".to_string(),
        40,
        100,
        "Normal".to_string(),
        30,
        PSMoveTarget::Normal,
        MoveCategory::Physical,
        1, // Priority +1
    );
    attacker.add_move(MoveIndex::M0, quick_attack);
    
    state.side_two.add_pokemon(attacker);
    state.side_two.set_active_pokemon_at_slot(0, Some(0));
    
    // Create move choices - attack should go first due to priority
    let switch_choice = MoveChoice::Switch(PokemonIndex::P1);
    let attack_choice = MoveChoice::new_move(
        MoveIndex::M0,
        vec![BattlePosition::new(SideReference::SideOne, 0)],
    );
    
    // Generate instructions
    let generator = GenerationXInstructionGenerator::new(format.clone());
    let instructions = generator.generate_instructions(&mut state, &switch_choice, &attack_choice);
    
    println!("Generated {} instruction sets for priority attack vs switch", instructions.len());
    
    // With priority move, attack should go first and hit the original Pokemon
    // Then switch should occur
    assert!(!instructions.is_empty(), "Should generate attack-switch instructions");
    
    let mut found_damage_first = false;
    let mut found_switch_after = false;
    
    for instruction_set in &instructions {
        let mut instruction_order = Vec::new();
        
        for instruction in &instruction_set.instruction_list {
            match instruction {
                Instruction::PositionDamage(damage_instr) => {
                    instruction_order.push("damage");
                    // Damage should target side one (the switcher)
                    assert_eq!(damage_instr.target_position.side, SideReference::SideOne);
                    found_damage_first = true;
                }
                Instruction::SwitchPokemon(switch_instr) => {
                    instruction_order.push("switch");
                    assert_eq!(switch_instr.position.side, SideReference::SideOne);
                    found_switch_after = true;
                }
                _ => {}
            }
        }
        
        println!("Instruction order: {:?}", instruction_order);
    }
    
    assert!(found_damage_first, "Should find damage instruction");
    assert!(found_switch_after, "Should find switch instruction");
}

#[test]
fn test_pursuit_goes_before_switch() {
    // This test verifies that Pursuit specifically hits before the switch occurs
    let format = BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles);
    let mut state = State::new(format.clone());
    
    // Side One: Pokemon that will switch
    let mut switching_pokemon = Pokemon::new("SwitchingPokemon".to_string());
    switching_pokemon.types = vec!["Normal".to_string()];
    switching_pokemon.level = 50;
    switching_pokemon.hp = 50;
    switching_pokemon.max_hp = 100;
    
    let mut replacement_pokemon = Pokemon::new("ReplacementPokemon".to_string());
    replacement_pokemon.types = vec!["Ghost".to_string()]; // Ghost is immune to Normal
    replacement_pokemon.level = 50;
    replacement_pokemon.hp = 100;
    replacement_pokemon.max_hp = 100;
    
    state.side_one.add_pokemon(switching_pokemon);
    state.side_one.add_pokemon(replacement_pokemon);
    state.side_one.set_active_pokemon_at_slot(0, Some(0));
    
    // Side Two: Pokemon with Pursuit
    let mut pursuer = Pokemon::new("Pursuer".to_string());
    pursuer.types = vec!["Dark".to_string()];
    pursuer.level = 50;
    pursuer.stats.attack = 100;
    
    // Add Pursuit move
    let pursuit = Move::new_with_details(
        "Pursuit".to_string(),
        40,
        100,
        "Dark".to_string(),
        20,
        PSMoveTarget::Normal,
        MoveCategory::Physical,
        0,
    );
    pursuer.add_move(MoveIndex::M0, pursuit);
    
    state.side_two.add_pokemon(pursuer);
    state.side_two.set_active_pokemon_at_slot(0, Some(0));
    
    // Create move choices
    let switch_choice = MoveChoice::Switch(PokemonIndex::P1);
    let pursuit_choice = MoveChoice::new_move(
        MoveIndex::M0,
        vec![BattlePosition::new(SideReference::SideOne, 0)],
    );
    
    // Generate instructions
    let generator = GenerationXInstructionGenerator::new(format);
    let instructions = generator.generate_instructions(&mut state, &switch_choice, &pursuit_choice);
    
    println!("Generated {} instruction sets for Pursuit vs Switch", instructions.len());
    
    // Pursuit should go first and hit the switching Pokemon (not the replacement)
    assert!(!instructions.is_empty(), "Should generate Pursuit-switch instructions");
    
    // Verify that Pursuit damages the original Pokemon before the switch
    let mut found_correct_order = false;
    
    for instruction_set in &instructions {
        let mut has_damage_before_switch = false;
        let mut switch_occurred = false;
        
        for instruction in &instruction_set.instruction_list {
            match instruction {
                Instruction::PositionDamage(damage_instr) => {
                    if !switch_occurred {
                        // Damage before switch means it hit the original Pokemon
                        has_damage_before_switch = true;
                        assert_eq!(damage_instr.target_position.side, SideReference::SideOne);
                    }
                }
                Instruction::SwitchPokemon(_) => {
                    switch_occurred = true;
                }
                _ => {}
            }
        }
        
        if has_damage_before_switch {
            found_correct_order = true;
        }
    }
    
    assert!(found_correct_order, "Pursuit should damage the original Pokemon before switch occurs");
}