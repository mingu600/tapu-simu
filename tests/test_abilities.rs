//! # Ability Integration Tests
//! 
//! Comprehensive tests for Pokemon abilities using real PS data.
//! These tests verify that abilities work correctly with actual Pokemon
//! and moves from Pokemon Showdown data.

use tapu_simu::test_framework::TestFramework;

#[test]
fn test_levitate_vs_earthquake() {
    // Test Levitate providing immunity to Ground moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_immune = framework.test_ability_immunity(
        "garchomp",   // Attacker: Garchomp
        "latios",     // Defender: Latios  
        "Levitate",   // Defender ability
        "earthquake"  // Ground move
    ).expect("Failed to test immunity");

    assert!(is_immune, "Latios with Levitate should be immune to Earthquake");
}

#[test]
fn test_thick_fat_fire_reduction() {
    // Test Thick Fat reducing Fire move damage by 50%
    let framework = TestFramework::new().expect("Failed to create test framework");

    let damage_multiplier = framework.test_ability_damage_reduction(
        "charizard",    // Attacker: Charizard
        "snorlax",      // Defender: Snorlax
        "Thick Fat",    // Defender ability  
        "flamethrower"  // Fire move
    ).expect("Failed to test damage reduction");

    // Should be approximately 0.5 (50% damage)
    assert!((damage_multiplier - 0.5).abs() < 0.1, 
            "Thick Fat should reduce Fire move damage to ~50%, got {}", damage_multiplier);
}

#[test]
fn test_thick_fat_ice_reduction() {
    // Test Thick Fat reducing Ice move damage by 50%
    let framework = TestFramework::new().expect("Failed to create test framework");

    let damage_multiplier = framework.test_ability_damage_reduction(
        "articuno",     // Attacker: Articuno
        "snorlax",      // Defender: Snorlax
        "Thick Fat",    // Defender ability
        "icebeam"       // Ice move
    ).expect("Failed to test damage reduction");

    // Should be approximately 0.5 (50% damage)
    assert!((damage_multiplier - 0.5).abs() < 0.1, 
            "Thick Fat should reduce Ice move damage to ~50%, got {}", damage_multiplier);
}

#[test]
fn test_water_absorb_immunity() {
    // Test Water Absorb providing immunity to Water moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_immune = framework.test_ability_immunity(
        "gyarados",     // Attacker: Gyarados
        "vaporeon",     // Defender: Vaporeon
        "Water Absorb", // Defender ability
        "surf"          // Water move
    ).expect("Failed to test immunity");

    assert!(is_immune, "Vaporeon with Water Absorb should be immune to Water moves");
}

#[test]
fn test_volt_absorb_immunity() {
    // Test Volt Absorb providing immunity to Electric moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_immune = framework.test_ability_immunity(
        "raichu",       // Attacker: Raichu
        "jolteon",      // Defender: Jolteon
        "Volt Absorb",  // Defender ability
        "thunderbolt"   // Electric move
    ).expect("Failed to test immunity");

    assert!(is_immune, "Jolteon with Volt Absorb should be immune to Electric moves");
}

#[test]
fn test_flash_fire_immunity() {
    // Test Flash Fire providing immunity to Fire moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_immune = framework.test_ability_immunity(
        "charizard",  // Attacker: Charizard
        "heatran",    // Defender: Heatran
        "Flash Fire", // Defender ability
        "flamethrower" // Fire move
    ).expect("Failed to test immunity");

    assert!(is_immune, "Heatran with Flash Fire should be immune to Fire moves");
}

#[test]
fn test_huge_power_attack_doubling() {
    // Test that Huge Power approximately doubles physical attack damage
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Azumarill without Huge Power
    let attacker_normal = framework.create_pokemon_from_ps_data("azumarill", None, Some(50))
        .expect("Failed to create normal Azumarill");
    
    // Create Azumarill with Huge Power  
    let attacker_huge_power = framework.create_pokemon_from_ps_data("azumarill", Some("Huge Power"), Some(50))
        .expect("Failed to create Huge Power Azumarill");

    let defender = framework.create_pokemon_from_ps_data("garchomp", None, Some(50))
        .expect("Failed to create Garchomp");

    let move_data = framework.create_move_from_ps_data("aquajet")
        .expect("Failed to create Aqua Jet");

    let state = tapu_simu::State::new(tapu_simu::BattleFormat::gen9_ou());

    let normal_damage = framework.test_damage_calculation(&attacker_normal, &defender, &move_data, &state);
    let huge_power_damage = framework.test_damage_calculation(&attacker_huge_power, &defender, &move_data, &state);

    if normal_damage > 0 {
        let multiplier = huge_power_damage as f32 / normal_damage as f32;
        assert!((multiplier - 2.0).abs() < 0.2, 
                "Huge Power should approximately double physical damage, got {}x", multiplier);
    }
}

#[test]
fn test_pure_power_attack_doubling() {
    // Test that Pure Power works the same as Huge Power
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Medicham without Pure Power
    let attacker_normal = framework.create_pokemon_from_ps_data("medicham", None, Some(50))
        .expect("Failed to create normal Medicham");
    
    // Create Medicham with Pure Power  
    let attacker_pure_power = framework.create_pokemon_from_ps_data("medicham", Some("Pure Power"), Some(50))
        .expect("Failed to create Pure Power Medicham");

    let defender = framework.create_pokemon_from_ps_data("garchomp", None, Some(50))
        .expect("Failed to create Garchomp");

    let move_data = framework.create_move_from_ps_data("highjumpkick")
        .expect("Failed to create High Jump Kick");

    let state = tapu_simu::State::new(tapu_simu::BattleFormat::gen9_ou());

    let normal_damage = framework.test_damage_calculation(&attacker_normal, &defender, &move_data, &state);
    let pure_power_damage = framework.test_damage_calculation(&attacker_pure_power, &defender, &move_data, &state);

    if normal_damage > 0 {
        let multiplier = pure_power_damage as f32 / normal_damage as f32;
        assert!((multiplier - 2.0).abs() < 0.2, 
                "Pure Power should approximately double physical damage, got {}x", multiplier);
    }
}

#[test]
fn test_technician_low_power_boost() {
    // Test Technician boosting moves with 60 or less base power
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Scyther without Technician
    let attacker_normal = framework.create_pokemon_from_ps_data("scyther", None, Some(50))
        .expect("Failed to create normal Scyther");
    
    // Create Scyther with Technician  
    let attacker_technician = framework.create_pokemon_from_ps_data("scyther", Some("Technician"), Some(50))
        .expect("Failed to create Technician Scyther");

    let defender = framework.create_pokemon_from_ps_data("garchomp", None, Some(50))
        .expect("Failed to create Garchomp");

    // Use a low power move (should be boosted)
    let low_power_move = framework.create_move_from_ps_data("quickattack")
        .expect("Failed to create Quick Attack");

    let state = tapu_simu::State::new(tapu_simu::BattleFormat::gen9_ou());

    let normal_damage = framework.test_damage_calculation(&attacker_normal, &defender, &low_power_move, &state);
    let technician_damage = framework.test_damage_calculation(&attacker_technician, &defender, &low_power_move, &state);

    if normal_damage > 0 {
        let multiplier = technician_damage as f32 / normal_damage as f32;
        assert!((multiplier - 1.5).abs() < 0.2, 
                "Technician should boost low power moves by 1.5x, got {}x", multiplier);
    }
}

#[test]
fn test_dry_skin_fire_weakness() {
    // Test Dry Skin making Pokemon weaker to Fire moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let damage_multiplier = framework.test_ability_damage_reduction(
        "charizard",    // Attacker: Charizard
        "toxicroak",    // Defender: Toxicroak (can have Dry Skin)
        "Dry Skin",     // Defender ability
        "flamethrower"  // Fire move
    ).expect("Failed to test damage multiplier");

    // Should be approximately 1.25 (25% more damage)
    assert!((damage_multiplier - 1.25).abs() < 0.1, 
            "Dry Skin should increase Fire move damage to ~125%, got {}x", damage_multiplier);
}

#[test]
fn test_dry_skin_water_immunity() {
    // Test Dry Skin providing immunity to Water moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_immune = framework.test_ability_immunity(
        "gyarados",   // Attacker: Gyarados
        "toxicroak",  // Defender: Toxicroak
        "Dry Skin",   // Defender ability
        "surf"        // Water move
    ).expect("Failed to test immunity");

    assert!(is_immune, "Toxicroak with Dry Skin should be immune to Water moves");
}

#[test]
fn test_storm_drain_immunity() {
    // Test Storm Drain providing immunity to Water moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_immune = framework.test_ability_immunity(
        "gyarados",     // Attacker: Gyarados
        "gastrodon",    // Defender: Gastrodon
        "Storm Drain",  // Defender ability
        "surf"          // Water move
    ).expect("Failed to test immunity");

    assert!(is_immune, "Gastrodon with Storm Drain should be immune to Water moves");
}

#[test]
fn test_lightning_rod_immunity() {
    // Test Lightning Rod providing immunity to Electric moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_immune = framework.test_ability_immunity(
        "raichu",         // Attacker: Raichu
        "pikachu",        // Defender: Pikachu (has Lightning Rod as hidden ability)
        "Lightning Rod",  // Defender ability
        "thunderbolt"     // Electric move
    ).expect("Failed to test immunity");

    assert!(is_immune, "Pikachu with Lightning Rod should be immune to Electric moves");
}

#[test]
fn test_motor_drive_immunity() {
    // Test Motor Drive providing immunity to Electric moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    let is_immune = framework.test_ability_immunity(
        "raichu",       // Attacker: Raichu
        "electivire",   // Defender: Electivire
        "Motor Drive",  // Defender ability
        "thunderbolt"   // Electric move
    ).expect("Failed to test immunity");

    assert!(is_immune, "Electivire with Motor Drive should be immune to Electric moves");
}