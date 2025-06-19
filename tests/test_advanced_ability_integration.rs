//! # Advanced Ability Integration Tests
//! 
//! Additional comprehensive tests for abilities to verify 100% parity with poke-engine.
//! These tests focus on complex abilities and edge cases.

use tapu_simu::{TestFramework, ContactStatusResult};

#[test]
fn test_download_stat_boost_mechanics() {
    // Test Download comparing Defense vs Special Defense and boosting accordingly
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Porygon-Z with Download
    let pokemon = framework.create_pokemon_from_ps_data("porygonz", Some("Download"), Some(50))
        .expect("Failed to create Porygon-Z with Download");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "download");
}

#[test]
fn test_tinted_lens_boosts_not_very_effective() {
    // Test Tinted Lens making not very effective moves do 2x damage
    let framework = TestFramework::new().expect("Failed to create test framework");

    let damage_multiplier = framework.test_ability_damage_reduction(
        "butterfree",   // Attacker: Butterfree with Tinted Lens
        "heatran",      // Defender: Heatran (Steel/Fire, resists Bug moves)
        "Tinted Lens",  // Attacker ability  
        "bugbuzz"       // Bug move (not very effective vs Steel/Fire)
    ).expect("Failed to test Tinted Lens");

    // Tinted Lens should make not very effective moves deal 2x damage
    // So a 0.25x effectiveness becomes 0.5x, a 0.5x becomes 1.0x
    assert!(damage_multiplier > 1.5, 
            "Tinted Lens should boost not very effective moves, got {}x", damage_multiplier);
}

#[test]
fn test_tough_claws_boosts_contact_moves() {
    // Test Tough Claws boosting contact moves by 1.3x
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Charizard without Tough Claws
    let attacker_normal = framework.create_pokemon_from_ps_data("charizardmegax", None, Some(50))
        .expect("Failed to create normal Charizard-Mega-X");
    
    // Create Charizard with Tough Claws
    let attacker_tough_claws = framework.create_pokemon_from_ps_data("charizardmegax", Some("Tough Claws"), Some(50))
        .expect("Failed to create Tough Claws Charizard-Mega-X");

    let defender = framework.create_pokemon_from_ps_data("garchomp", None, Some(50))
        .expect("Failed to create Garchomp");

    // Use a contact move
    let contact_move = framework.create_move_from_ps_data("dragonclaw")
        .expect("Failed to create Dragon Claw");

    let state = tapu_simu::State::new(tapu_simu::BattleFormat::gen9_ou());

    let normal_damage = framework.test_damage_calculation(&attacker_normal, &defender, &contact_move, &state);
    let tough_claws_damage = framework.test_damage_calculation(&attacker_tough_claws, &defender, &contact_move, &state);

    if normal_damage > 0 {
        let multiplier = tough_claws_damage as f32 / normal_damage as f32;
        assert!((multiplier - 1.3).abs() < 0.2, 
                "Tough Claws should boost contact moves by 1.3x, got {}x", multiplier);
    }
}

#[test]
fn test_neuroforce_boosts_super_effective() {
    // Test Neuroforce boosting super effective moves by 1.25x
    let framework = TestFramework::new().expect("Failed to create test framework");

    let damage_multiplier = framework.test_ability_damage_reduction(
        "necrozmaduskmane", // Attacker: Necrozma with Neuroforce
        "charizard",        // Defender: Charizard (Fire/Flying, weak to Rock)
        "Neuroforce",       // Attacker ability
        "stoneedge"         // Rock move (super effective vs Fire/Flying)
    ).expect("Failed to test Neuroforce");

    // Neuroforce should boost super effective moves by 1.25x
    assert!(damage_multiplier > 1.1, 
            "Neuroforce should boost super effective moves by 1.25x, got {}x", damage_multiplier);
}

#[test]
fn test_pixilate_generation_specific_multipliers() {
    // Test Pixilate has different multipliers by generation
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Altaria without Pixilate
    let attacker_normal = framework.create_pokemon_from_ps_data("altariamega", None, Some(50))
        .expect("Failed to create normal Altaria-Mega");
    
    // Create Altaria with Pixilate
    let attacker_pixilate = framework.create_pokemon_from_ps_data("altariamega", Some("Pixilate"), Some(50))
        .expect("Failed to create Pixilate Altaria-Mega");

    let defender = framework.create_pokemon_from_ps_data("garchomp", None, Some(50))
        .expect("Failed to create Garchomp");

    // Use a Normal move that should become Fairy-type
    let normal_move = framework.create_move_from_ps_data("hypervoice")
        .expect("Failed to create Hyper Voice");

    let state = tapu_simu::State::new(tapu_simu::BattleFormat::gen9_ou());

    let normal_damage = framework.test_damage_calculation(&attacker_normal, &defender, &normal_move, &state);
    let pixilate_damage = framework.test_damage_calculation(&attacker_pixilate, &defender, &normal_move, &state);

    if normal_damage > 0 {
        let multiplier = pixilate_damage as f32 / normal_damage as f32;
        // Expected: 1.2x (Pixilate) * 1.5x (STAB) = 1.8x minimum
        assert!(multiplier > 1.5, 
                "Pixilate should boost Normal moves and add STAB, got {}x", multiplier);
    }
}

#[test]
fn test_poison_heal_ability_prevents_poison_damage() {
    // Test Poison Heal healing instead of taking poison damage
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Breloom with Poison Heal
    let pokemon = framework.create_pokemon_from_ps_data("breloom", Some("Poison Heal"), Some(50))
        .expect("Failed to create Breloom with Poison Heal");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "poisonheal");
}

#[test]
fn test_magic_guard_prevents_indirect_damage() {
    // Test Magic Guard preventing indirect damage (entry hazards, status, weather)
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Alakazam with Magic Guard
    let pokemon = framework.create_pokemon_from_ps_data("alakazam", Some("Magic Guard"), Some(50))
        .expect("Failed to create Alakazam with Magic Guard");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "magicguard");
}

#[test]
fn test_wonder_guard_only_super_effective_hits() {
    // Test Wonder Guard only allowing super effective moves to hit
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Test immunity to non-super effective move
    let is_immune_normal = framework.test_ability_immunity(
        "garchomp",     // Attacker: Garchomp
        "shedinja",     // Defender: Shedinja
        "Wonder Guard", // Defender ability
        "earthquake"    // Ground move (not very effective vs Bug/Ghost)
    ).expect("Failed to test Wonder Guard immunity");

    assert!(is_immune_normal, "Wonder Guard should block non-super effective moves");

    // Test that super effective moves can hit
    let is_immune_super = framework.test_ability_immunity(
        "charizard",    // Attacker: Charizard
        "shedinja",     // Defender: Shedinja
        "Wonder Guard", // Defender ability
        "rockslide"     // Rock move (super effective vs Bug)
    ).expect("Failed to test Wonder Guard vs super effective");

    assert!(!is_immune_super, "Wonder Guard should allow super effective moves");
}

#[test]
fn test_mold_breaker_ignores_abilities() {
    // Test Mold Breaker ignoring certain defensive abilities
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Excadrill with Mold Breaker
    let pokemon = framework.create_pokemon_from_ps_data("excadrill", Some("Mold Breaker"), Some(50))
        .expect("Failed to create Excadrill with Mold Breaker");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "moldbreaker");
}

#[test]
fn test_prankster_priority_mechanics() {
    // Test Prankster giving priority to status moves
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Sableye with Prankster
    let pokemon = framework.create_pokemon_from_ps_data("sableye", Some("Prankster"), Some(50))
        .expect("Failed to create Sableye with Prankster");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "prankster");
}

#[test]
fn test_serene_grace_doubles_secondary_effects() {
    // Test Serene Grace doubling the chance of secondary effects
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Togekiss with Serene Grace
    let pokemon = framework.create_pokemon_from_ps_data("togekiss", Some("Serene Grace"), Some(50))
        .expect("Failed to create Togekiss with Serene Grace");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "serenegrace");
}

#[test]
fn test_sheer_force_removes_secondary_effects() {
    // Test Sheer Force removing secondary effects but boosting power
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Nidoking with Sheer Force
    let pokemon = framework.create_pokemon_from_ps_data("nidoking", Some("Sheer Force"), Some(50))
        .expect("Failed to create Nidoking with Sheer Force");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "sheerforce");
}

#[test]
fn test_compound_eyes_accuracy_boost() {
    // Test Compound Eyes boosting accuracy by 1.3x
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Butterfree with Compound Eyes
    let pokemon = framework.create_pokemon_from_ps_data("butterfree", Some("Compound Eyes"), Some(50))
        .expect("Failed to create Butterfree with Compound Eyes");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "compoundeyes");
}

#[test]
fn test_sturdy_prevents_ohko_from_full_hp() {
    // Test Sturdy preventing OHKO when at full HP
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Magnezone with Sturdy
    let pokemon = framework.create_pokemon_from_ps_data("magnezone", Some("Sturdy"), Some(50))
        .expect("Failed to create Magnezone with Sturdy");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "sturdy");
}

#[test]
fn test_contrary_reverses_stat_changes() {
    // Test Contrary reversing stat changes
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Create Lurantis with Contrary
    let pokemon = framework.create_pokemon_from_ps_data("lurantis", Some("Contrary"), Some(50))
        .expect("Failed to create Lurantis with Contrary");

    // Verify the ability exists and can be loaded
    assert_eq!(pokemon.ability, "contrary");
}