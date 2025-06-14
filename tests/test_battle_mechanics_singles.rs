//! # Battle Mechanics Tests for Singles
//!
//! Port of poke-engine's test_battle_mechanics.rs adapted for tapu-simu's position-based system.
//! These tests validate core battle mechanics in Singles format.

use tapu_simu::{
    battle_format::{BattlePosition, SideReference},
    data::{Choices, MOVES},
    engine::instruction_generator::GenerationXInstructionGenerator,
    instruction::{StateInstructions, VolatileStatus},
    move_choice::MoveIndex,
    state::{Pokemon, Move, MoveCategory},
    BattleFormat, MoveChoice, State,
};

/// Helper function equivalent to poke-engine's generate_instructions_with_state_assertion
fn generate_instructions_with_state_assertion(
    state: &mut State,
    side_one_move: &MoveChoice,
    side_two_move: &MoveChoice,
) -> Vec<StateInstructions> {
    let before_state = format!("{:?}", state);
    let generator = GenerationXInstructionGenerator::new(state.format.clone());
    let instructions = generator.generate_instructions(state, side_one_move, side_two_move);
    let after_state = format!("{:?}", state);
    assert_eq!(before_state, after_state, "State should not be modified during instruction generation");
    instructions
}

/// Helper function equivalent to poke-engine's set_moves_on_pkmn_and_call_generate_instructions
fn set_moves_on_pkmn_and_call_generate_instructions(
    state: &mut State,
    move_one: Choices,
    move_two: Choices,
) -> Vec<StateInstructions> {
    // Convert Choices to Move using the existing MOVES HashMap
    let move_one_data = convert_choice_to_move(move_one);
    let move_two_data = convert_choice_to_move(move_two);
    
    // Add moves to Pokemon at M0 slot
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.add_move(MoveIndex::M0, move_one_data);
    }
    if let Some(pokemon) = state.side_two.get_active_pokemon_at_slot_mut(0) {
        pokemon.add_move(MoveIndex::M0, move_two_data);
    }
    
    let side_one_move = MoveChoice::new_move(
        MoveIndex::M0,
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );
    let side_two_move = MoveChoice::new_move(
        MoveIndex::M0,
        vec![BattlePosition::new(SideReference::SideOne, 0)],
    );
    
    generate_instructions_with_state_assertion(state, &side_one_move, &side_two_move)
}

/// Convert a Choices enum to a Move struct using the existing MOVES HashMap
fn convert_choice_to_move(choice: Choices) -> Move {
    let choice_data = MOVES.get(&choice).expect("Move should exist in MOVES HashMap");
    
    // Use the choice enum name as the move name (simpler and more consistent)
    let move_name = format!("{:?}", choice);
    
    Move::new_with_details(
        move_name,
        choice_data.base_power as u8,
        choice_data.accuracy as u8,
        format!("{:?}", choice_data.move_type),
        35, // PP - could get from choice_data if needed
        choice_data.target,
        convert_choice_category(&choice_data.category),
        choice_data.priority,
    )
}


/// Convert MoveCategory from choices.rs to state.rs
fn convert_choice_category(category: &tapu_simu::data::MoveCategory) -> MoveCategory {
    match category {
        tapu_simu::data::MoveCategory::Physical => MoveCategory::Physical,
        tapu_simu::data::MoveCategory::Special => MoveCategory::Special,
        tapu_simu::data::MoveCategory::Status => MoveCategory::Status,
        tapu_simu::data::MoveCategory::Switch => MoveCategory::Status, // Switch actions aren't moves, default to Status
    }
}

/// Create a basic singles state with Pokemon on both sides
fn create_basic_singles_state() -> State {
    let mut state = State::new(BattleFormat::Singles);
    
    // Add Pokemon to both sides
    let pokemon1 = Pokemon::new("TestPokemon1".to_string());
    let pokemon2 = Pokemon::new("TestPokemon2".to_string());
    
    state.side_one.add_pokemon(pokemon1);
    state.side_one.set_active_pokemon_at_slot(0, Some(0));
    
    state.side_two.add_pokemon(pokemon2);
    state.side_two.set_active_pokemon_at_slot(0, Some(0));
    
    state
}


#[test]
fn test_confuseray_into_substitute() {
    let mut state = create_basic_singles_state();

    // Set up side two with Substitute volatile status
    if let Some(pokemon) = state.side_two.get_active_pokemon_at_slot_mut(0) {
        pokemon.volatile_statuses.insert(VolatileStatus::Substitute);
        pokemon.substitute_health = 20;
    }

    let vec_of_instructions =
        set_moves_on_pkmn_and_call_generate_instructions(&mut state, Choices::CONFUSERAY, Choices::SPLASH);

    // Confuse Ray should be blocked by Substitute, generating empty instructions
    // Splash should also generate empty instructions (it's a no-effect move)
    let expected_instructions = vec![
        StateInstructions {
            percentage: 100.0,
            instruction_list: vec![],
            affected_positions: vec![],
        },
        StateInstructions {
            percentage: 100.0,
            instruction_list: vec![],
            affected_positions: vec![],
        }
    ];
    assert_eq!(expected_instructions, vec_of_instructions);
}

#[test]
fn test_confuseray_without_substitute() {
    let mut state = create_basic_singles_state();

    // No Substitute on side two - Confuse Ray should work
    let vec_of_instructions =
        set_moves_on_pkmn_and_call_generate_instructions(&mut state, Choices::CONFUSERAY, Choices::SPLASH);

    // Confuse Ray should successfully apply confusion status
    // Splash should generate empty instructions (it's a no-effect move)
    assert_eq!(vec_of_instructions.len(), 2);
    
    // First instruction set should have confusion status application
    let first_instruction_set = &vec_of_instructions[0];
    assert_eq!(first_instruction_set.percentage, 100.0);
    assert_eq!(first_instruction_set.instruction_list.len(), 1);
    
    // Check if it's a volatile status instruction for confusion
    if let Some(tapu_simu::instruction::Instruction::ApplyVolatileStatus(status_instr)) = first_instruction_set.instruction_list.first() {
        assert_eq!(status_instr.volatile_status, tapu_simu::instruction::VolatileStatus::Confusion);
        assert_eq!(status_instr.target_position, tapu_simu::battle_format::BattlePosition::new(tapu_simu::battle_format::SideReference::SideTwo, 0));
        assert_eq!(status_instr.duration, Some(2));
    } else {
        panic!("Expected ApplyVolatileStatus instruction for Confusion");
    }
    
    // Second instruction set should be empty (Splash)
    let second_instruction_set = &vec_of_instructions[1];
    assert_eq!(second_instruction_set.percentage, 100.0);
    assert_eq!(second_instruction_set.instruction_list.len(), 0);
}

#[test]
fn test_branch_on_crit() {
    let mut state = create_basic_singles_state();
    
    // Set side_two HP to exactly 100 (matching poke-engine test)
    if let Some(pokemon) = state.side_two.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 100;
    }

    // Use WATERGUN and SPLASH exactly like poke-engine test
    let vec_of_instructions =
        set_moves_on_pkmn_and_call_generate_instructions(&mut state, Choices::WATERGUN, Choices::SPLASH);

    // Expected damage should be exactly 32 (matching poke-engine)
    let expected_damage = 32;
    
    // Critical hit constants (matching poke-engine)
    const BASE_CRIT_CHANCE: f32 = 1.0 / 24.0;
    const CRIT_MULTIPLIER: f32 = 1.5;
    
    // Should have exactly 2 instructions: normal hit and crit hit
    // (Splash generates no instructions in this case)
    let expected_instructions = vec![
        StateInstructions {
            percentage: 100.0 * (1.0 - BASE_CRIT_CHANCE),
            instruction_list: vec![tapu_simu::instruction::Instruction::PositionDamage(
                tapu_simu::instruction::PositionDamageInstruction {
                    target_position: tapu_simu::battle_format::BattlePosition::new(
                        tapu_simu::battle_format::SideReference::SideTwo, 0
                    ),
                    damage_amount: expected_damage,
                }
            )],
            affected_positions: vec![tapu_simu::battle_format::BattlePosition::new(
                tapu_simu::battle_format::SideReference::SideTwo, 0
            )],
        },
        StateInstructions {
            percentage: 100.0 * BASE_CRIT_CHANCE,
            instruction_list: vec![tapu_simu::instruction::Instruction::PositionDamage(
                tapu_simu::instruction::PositionDamageInstruction {
                    target_position: tapu_simu::battle_format::BattlePosition::new(
                        tapu_simu::battle_format::SideReference::SideTwo, 0
                    ),
                    damage_amount: (CRIT_MULTIPLIER * expected_damage as f32).floor() as i16,
                }
            )],
            affected_positions: vec![tapu_simu::battle_format::BattlePosition::new(
                tapu_simu::battle_format::SideReference::SideTwo, 0
            )],
        },
    ];
    
    assert_eq!(expected_instructions, vec_of_instructions);
}

// Helper function to extract damage amount from instruction
fn get_damage_from_instruction(state_instructions: &StateInstructions) -> i16 {
    for instruction in &state_instructions.instruction_list {
        if let tapu_simu::instruction::Instruction::PositionDamage(damage_instr) = instruction {
            return damage_instr.damage_amount;
        }
    }
    panic!("No damage instruction found");
}
