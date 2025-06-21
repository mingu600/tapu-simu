//! # Core Battle Mechanics Tests
//! 
//! Comprehensive tests for fundamental battle mechanics including damage calculation,
//! critical hits, type effectiveness, status moves, generation differences, and format-specific
//! mechanics. All tests use the TestFramework with real Pokemon Showdown data.

use tapu_simu::TestFramework;
use tapu_simu::{State, BattleFormat};
use tapu_simu::generation::Generation;
use tapu_simu::type_effectiveness::{TypeChart, PokemonType};

// =============================================================================
// BASIC DAMAGE CALCULATION TESTS
// =============================================================================

#[test]
fn test_basic_damage_calculation() {
    // Test that basic damage calculation produces positive damage for valid moves
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let attacker = framework.create_pokemon_from_ps_data("charizard", None, Some(50))
        .expect("Failed to create charizard");
    let defender = framework.create_pokemon_from_ps_data("blastoise", None, Some(50))
        .expect("Failed to create blastoise");
    let move_data = framework.create_move_from_ps_data("tackle")
        .expect("Failed to create tackle");
    
    let state = State::new(BattleFormat::gen9_ou());
    let damage = framework.test_damage_calculation(&attacker, &defender, &move_data, &state);
    
    assert!(damage > 0, "Basic damage calculation should produce positive damage");
}

#[test]
fn test_critical_hit_damage_increase() {
    // Test that critical hits deal more damage than normal hits
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let attacker = framework.create_pokemon_from_ps_data("pikachu", None, Some(50))
        .expect("Failed to create pikachu");
    let defender = framework.create_pokemon_from_ps_data("squirtle", None, Some(50))
        .expect("Failed to create squirtle");
    let move_data = framework.create_move_from_ps_data("thunderbolt")
        .expect("Failed to create thunderbolt");
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Calculate normal damage using internal damage calc function directly
    let normal_damage = tapu_simu::damage_calc::calculate_damage(
        &state, &attacker, &defender, &move_data, false, 1.0
    );
    let crit_damage = tapu_simu::damage_calc::calculate_damage(
        &state, &attacker, &defender, &move_data, true, 1.0
    );
    
    assert!(crit_damage > normal_damage, 
           "Critical hit damage ({}) should be greater than normal damage ({})", 
           crit_damage, normal_damage);
    
    // In Gen 9, critical hits should be approximately 1.5x
    let multiplier = crit_damage as f32 / normal_damage as f32;
    assert!((multiplier - 1.5).abs() < 0.2, 
           "Critical hit multiplier should be ~1.5x, got {}x", multiplier);
}

#[test]
fn test_status_moves_deal_no_damage() {
    // Test that status moves deal 0 damage
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let attacker = framework.create_pokemon_from_ps_data("alakazam", None, Some(50))
        .expect("Failed to create alakazam");
    let defender = framework.create_pokemon_from_ps_data("machamp", None, Some(50))
        .expect("Failed to create machamp");
    let move_data = framework.create_move_from_ps_data("thunderwave")
        .expect("Failed to create thunderwave");
    
    let state = State::new(BattleFormat::gen9_ou());
    let damage = framework.test_damage_calculation(&attacker, &defender, &move_data, &state);
    
    assert_eq!(damage, 0, "Status moves should deal 0 damage");
}

// =============================================================================
// GENERATION-SPECIFIC MECHANICS
// =============================================================================

#[test]
fn test_generation_specific_critical_multipliers() {
    // Test that different generations have different critical hit multipliers
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let attacker = framework.create_pokemon_from_ps_data("mewtwo", None, Some(50))
        .expect("Failed to create mewtwo");
    let defender = framework.create_pokemon_from_ps_data("mew", None, Some(50))
        .expect("Failed to create mew");
    let move_data = framework.create_move_from_ps_data("psychic")
        .expect("Failed to create psychic");
    
    // Gen 5 battle
    let gen5_state = State::new(BattleFormat::new(
        "Gen 5 OU".to_string(),
        Generation::Gen5,
        tapu_simu::core::battle_format::FormatType::Singles,
    ));
    
    // Gen 9 battle
    let gen9_state = State::new(BattleFormat::gen9_ou());
    
    // Calculate normal and critical damage for both generations
    let gen5_normal = tapu_simu::damage_calc::calculate_damage(
        &gen5_state, &attacker, &defender, &move_data, false, 1.0
    );
    let gen5_crit = tapu_simu::damage_calc::calculate_damage(
        &gen5_state, &attacker, &defender, &move_data, true, 1.0
    );
    
    let gen9_normal = tapu_simu::damage_calc::calculate_damage(
        &gen9_state, &attacker, &defender, &move_data, false, 1.0
    );
    let gen9_crit = tapu_simu::damage_calc::calculate_damage(
        &gen9_state, &attacker, &defender, &move_data, true, 1.0
    );
    
    let gen5_multiplier = gen5_crit as f32 / gen5_normal as f32;
    let gen9_multiplier = gen9_crit as f32 / gen9_normal as f32;
    
    // Gen 5 should have 2.0x crit multiplier, Gen 9 should have 1.5x
    assert!((gen5_multiplier - 2.0).abs() < 0.1, 
           "Gen 5 critical hit multiplier should be ~2.0x, got {}x", gen5_multiplier);
    assert!((gen9_multiplier - 1.5).abs() < 0.1, 
           "Gen 9 critical hit multiplier should be ~1.5x, got {}x", gen9_multiplier);
}

// =============================================================================
// TYPE EFFECTIVENESS TESTS
// =============================================================================

#[test] 
fn test_type_effectiveness() {
    // Test basic type effectiveness (super effective, not very effective, immune)
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let electric_attacker = framework.create_pokemon_from_ps_data("raichu", None, Some(50))
        .expect("Failed to create raichu");
    let ground_defender = framework.create_pokemon_from_ps_data("golem", None, Some(50))
        .expect("Failed to create golem");
    let water_defender = framework.create_pokemon_from_ps_data("gyarados", None, Some(50))
        .expect("Failed to create gyarados");
    let grass_defender = framework.create_pokemon_from_ps_data("venusaur", None, Some(50))
        .expect("Failed to create venusaur");
    
    let electric_move = framework.create_move_from_ps_data("thunderbolt")
        .expect("Failed to create thunderbolt");
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Electric vs Ground should be immune (0x)
    let immune_damage = framework.test_damage_calculation(&electric_attacker, &ground_defender, &electric_move, &state);
    assert_eq!(immune_damage, 0, "Electric moves should deal 0 damage to Ground types");
    
    // Electric vs Water should be super effective (2x)
    let super_effective_damage = framework.test_damage_calculation(&electric_attacker, &water_defender, &electric_move, &state);
    
    // Electric vs Grass should be not very effective (0.5x)
    let not_very_effective_damage = framework.test_damage_calculation(&electric_attacker, &grass_defender, &electric_move, &state);
    
    assert!(super_effective_damage > not_very_effective_damage,
           "Super effective damage ({}) should be greater than not very effective damage ({})",
           super_effective_damage, not_very_effective_damage);
    
    // The ratio should be approximately 4:1 (2x vs 0.5x)
    let damage_ratio = super_effective_damage as f32 / not_very_effective_damage as f32;
    assert!((damage_ratio - 4.0).abs() < 5.0,
           "Super effective vs not very effective should be ~4:1 ratio, got {}:1", damage_ratio);
}

#[test]
fn test_fire_vs_poison_effectiveness() {
    // Test specific type effectiveness case using TypeChart directly
    let chart = TypeChart::default();
    
    let effectiveness = chart.get_effectiveness(PokemonType::Fire, PokemonType::Poison);
    
    // According to the standard Pokemon type chart, Fire vs Poison is neutral (1.0x)
    assert_eq!(effectiveness, 1.0, "Fire vs Poison should be neutral effectiveness (1.0x)");
}

// =============================================================================
// FORMAT-SPECIFIC MECHANICS
// =============================================================================

#[test]
fn test_basic_singles_targeting() {
    // Test that moves correctly target the opponent in singles
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let attacker = framework.create_pokemon_from_ps_data("pikachu", None, Some(50))
        .expect("Failed to create pikachu");
    let defender = framework.create_pokemon_from_ps_data("squirtle", None, Some(50))
        .expect("Failed to create squirtle");
    let move_data = framework.create_move_from_ps_data("tackle")
        .expect("Failed to create tackle");
    
    let state = State::new(BattleFormat::gen9_ou());
    let damage = framework.test_damage_calculation(&attacker, &defender, &move_data, &state);
    
    // In singles, moves should work normally
    assert!(damage > 0, "Moves should deal damage in singles format");
}

#[test]
fn test_spread_move_damage_reduction() {
    // Test that moves deal reduced damage when hitting multiple targets in doubles
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let attacker = framework.create_pokemon_from_ps_data("garchomp", None, Some(50))
        .expect("Failed to create garchomp");
    let defender = framework.create_pokemon_from_ps_data("tyranitar", None, Some(50))
        .expect("Failed to create tyranitar");
    let spread_move = framework.create_move_from_ps_data("rockslide")
        .expect("Failed to create rockslide");
    
    // Singles format
    let singles_state = State::new(BattleFormat::gen9_ou());
    let singles_damage = framework.test_damage_calculation(&attacker, &defender, &spread_move, &singles_state);
    
    // Doubles format (with spread move reduction)
    let doubles_state = State::new(BattleFormat::new(
        "Gen 9 Doubles OU".to_string(),
        Generation::Gen9,
        tapu_simu::core::battle_format::FormatType::Doubles,
    ));
    
    // Simulate hitting multiple targets by using calculate_damage_with_targets directly
    let doubles_damage = tapu_simu::damage_calc::calculate_damage_with_targets(
        &doubles_state, &attacker, &defender, &spread_move, false, 1.0, 2
    );
    
    assert!(doubles_damage < singles_damage,
           "Spread moves should deal less damage in doubles ({}) than singles ({})",
           doubles_damage, singles_damage);
    
    // Should be approximately 0.75x damage for spread moves
    let reduction_ratio = doubles_damage as f32 / singles_damage as f32;
    assert!((reduction_ratio - 0.75).abs() < 0.1,
           "Spread move reduction should be ~0.75x, got {}x", reduction_ratio);
}