/// Integration test for instruction generation using the TestFramework
/// This test verifies that our instruction generation properly integrates
/// damage calculation with move effects using real Pokemon Showdown data
use tapu_simu::test_framework::TestFramework;
use tapu_simu::{
    battle_format::{BattlePosition, SideReference},
    instruction::PokemonStatus,
    instruction::Stat,
    move_choice::MoveChoice,
};

#[test]
fn test_tackle_generates_damage_with_critical_hit_branching() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    let (mut state, move_indices) = framework
        .create_test_battle("pikachu", &["tackle"], "squirtle", None)
        .expect("Failed to create test battle");

    let move_choice = MoveChoice::new_move(
        move_indices[0],
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );

    let instructions = framework.test_instruction_generation(&mut state, move_choice, None);

    // Should generate damage instructions
    assert!(
        framework.verify_damage_instructions(&instructions),
        "Tackle should generate damage instructions"
    );

    // Should have critical hit branching (2 instruction sets)
    assert!(
        framework.verify_critical_hit_branching(&instructions),
        "Tackle should have normal and critical hit branches"
    );

    // Probabilities should sum to 100%
    assert!(
        framework.verify_probability_distribution(&instructions),
        "Total probability should be 100%"
    );

    // Should have exactly 2 branches for normal + crit
    assert_eq!(
        instructions.len(),
        2,
        "Should have normal and critical hit branches"
    );
}

#[test]
fn test_thunder_wave_generates_paralysis_status() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    let (mut state, move_indices) = framework
        .create_test_battle("pikachu", &["thunderwave"], "squirtle", None)
        .expect("Failed to create test battle");

    let move_choice = MoveChoice::new_move(
        move_indices[0],
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );

    let instructions = framework.test_instruction_generation(&mut state, move_choice, None);

    // Should generate paralysis status instruction
    assert!(
        framework.verify_status_instructions(&instructions, PokemonStatus::PARALYZE),
        "Thunder Wave should generate paralysis status instruction"
    );

    // Should NOT generate damage instructions (status move)
    assert!(
        !framework.verify_damage_instructions(&instructions),
        "Thunder Wave should not generate damage instructions"
    );
}

#[test]
fn test_swords_dance_generates_attack_boost() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    let (mut state, move_indices) = framework
        .create_test_battle("pikachu", &["swordsdance"], "squirtle", None)
        .expect("Failed to create test battle");

    let move_choice = MoveChoice::new_move(
        move_indices[0],
        vec![], // Self-targeting move
    );

    let instructions = framework.test_instruction_generation(&mut state, move_choice, None);

    // Should generate +2 Attack boost
    assert!(
        framework.verify_stat_boost_instructions(&instructions, Stat::Attack, 2),
        "Swords Dance should generate +2 Attack boost instruction"
    );

    // Should NOT generate damage instructions (status move)
    assert!(
        !framework.verify_damage_instructions(&instructions),
        "Swords Dance should not generate damage instructions"
    );
}

#[test]
fn test_electric_type_immunity_to_thunder_wave() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_blocked = framework
        .test_type_immunity_blocks_status(
            "pikachu",
            "thunderwave",
            "raichu",
            &["Electric"], // Electric types immune to paralysis
            PokemonStatus::PARALYZE,
        )
        .expect("Failed to test immunity");

    assert!(
        is_blocked,
        "Electric types should be immune to Thunder Wave paralysis"
    );
}

#[test]
fn test_levitate_immunity_to_earthquake() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Test directly with battle creation for more control
    let (mut state, move_indices) = framework.create_test_battle(
        "garchomp",
        &["earthquake"],
        "latios",
        None,
    ).expect("Failed to create test battle");

    // Set Levitate ability explicitly
    if let Some(defender) = state.side_two.get_active_pokemon_at_slot_mut(0) {
        defender.ability = "levitate".to_string(); // Already normalized
    }

    let move_choice = MoveChoice::new_move(
        move_indices[0],
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );

    let instructions = framework.test_instruction_generation(&mut state, move_choice, None);
    
    // Pokemon with Levitate should be immune to Ground moves
    assert!(!framework.verify_damage_instructions(&instructions),
            "Pokemon with Levitate should be immune to Earthquake");
}

#[test]
fn test_critical_hit_damage_multiplier() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    let (mut state, move_indices) = framework
        .create_test_battle("pikachu", &["tackle"], "squirtle", None)
        .expect("Failed to create test battle");

    let move_choice = MoveChoice::new_move(
        move_indices[0],
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );

    let instructions = framework.test_instruction_generation(&mut state, move_choice, None);

    // Extract damage amounts from both branches
    let mut damage_amounts = Vec::new();
    for instr_set in &instructions {
        for instr in &instr_set.instruction_list {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_instr) = instr {
                damage_amounts.push(damage_instr.damage_amount);
            }
        }
    }

    assert_eq!(
        damage_amounts.len(),
        2,
        "Should have exactly 2 damage amounts"
    );

    // Sort to get normal and crit damage
    damage_amounts.sort();
    let normal_damage = damage_amounts[0] as f32;
    let crit_damage = damage_amounts[1] as f32;

    // Critical hit should be 1.5x normal damage (Gen 6+)
    let expected_crit = (normal_damage * 1.5).floor();
    assert_eq!(
        crit_damage, expected_crit,
        "Critical hit should be 1.5x normal damage (floored). Normal: {}, Crit: {}, Expected: {}",
        normal_damage, crit_damage, expected_crit
    );
}

#[test]
fn test_ghost_type_immunity_to_normal_moves() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    let (mut state, move_indices) = framework.create_test_battle(
        "pikachu",
        &["tackle"],
        "gengar",
        None,
    ).expect("Failed to create test battle");

    // Ensure gengar is Ghost type
    if let Some(defender) = state.side_two.get_active_pokemon_at_slot_mut(0) {
        defender.types = vec!["Ghost".to_string(), "Poison".to_string()];
    }

    let move_choice = MoveChoice::new_move(
        move_indices[0],
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );

    let instructions = framework.test_instruction_generation(&mut state, move_choice, None);
    
    // Ghost types should be immune to Normal moves
    assert!(!framework.verify_damage_instructions(&instructions),
            "Ghost types should be immune to Normal type moves");
}

#[test]
fn test_multiple_move_instructions() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    let (mut state, move_indices) = framework
        .create_test_battle(
            "pikachu",
            &["tackle", "thunderwave", "swordsdance"],
            "squirtle",
            None,
        )
        .expect("Failed to create test battle");

    // Test each move generates appropriate instructions

    // Tackle - damage instructions
    let tackle_choice = MoveChoice::new_move(
        move_indices[0],
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );
    let tackle_instructions =
        framework.test_instruction_generation(&mut state, tackle_choice, None);
    assert!(framework.verify_damage_instructions(&tackle_instructions));

    // Thunder Wave - status instructions
    let thunder_wave_choice = MoveChoice::new_move(
        move_indices[1],
        vec![BattlePosition::new(SideReference::SideTwo, 0)],
    );
    let thunder_wave_instructions =
        framework.test_instruction_generation(&mut state, thunder_wave_choice, None);
    assert!(
        framework.verify_status_instructions(&thunder_wave_instructions, PokemonStatus::PARALYZE)
    );

    // Swords Dance - stat boost instructions
    let swords_dance_choice = MoveChoice::new_move(
        move_indices[2],
        vec![], // Self-targeting
    );
    let swords_dance_instructions =
        framework.test_instruction_generation(&mut state, swords_dance_choice, None);
    assert!(framework.verify_stat_boost_instructions(&swords_dance_instructions, Stat::Attack, 2));
}
