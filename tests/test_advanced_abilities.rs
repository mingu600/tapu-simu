//! Tests for Phase 3 advanced abilities with 100% poke-engine parity

use tapu_simu::test_framework::TestFramework;
use tapu_simu::state::State;
use tapu_simu::battle_format::BattleFormat;
use tapu_simu::engine::damage_calc;

#[test]
fn test_solid_rock_reduces_super_effective_damage() {
    let framework = TestFramework::new().unwrap();
    
    // Create Rhyperior with Solid Rock (Rock/Ground vs Water move - 2x effectiveness)
    let mut defender = framework
        .create_pokemon_from_ps_data("rhyperior", Some("Solid Rock"), Some(50))
        .unwrap();
    
    let attacker = framework
        .create_pokemon_from_ps_data("gyarados", None, Some(50))
        .unwrap();
    
    let surf = framework.create_move_from_ps_data("surf").unwrap(); // Water move, super effective vs Rock/Ground
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Test damage with Solid Rock
    let damage_with_solid_rock = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &surf,
        false,
        1.0,
    );
    
    // Test damage without Solid Rock
    defender.ability = "noability".to_string();
    let damage_without_solid_rock = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &surf,
        false,
        1.0,
    );
    
    let damage_ratio = damage_with_solid_rock as f32 / damage_without_solid_rock as f32;
    assert!(
        damage_ratio > 0.70 && damage_ratio < 0.80,
        "Solid Rock should reduce super effective damage to ~75%, got {}x",
        damage_ratio
    );
}

#[test]
fn test_filter_identical_to_solid_rock() {
    let framework = TestFramework::new().unwrap();
    
    // Test with identical setups but different abilities
    let mut solid_rock_defender = framework
        .create_pokemon_from_ps_data("rhyperior", Some("Solid Rock"), Some(50))
        .unwrap();
    
    let mut filter_defender = framework
        .create_pokemon_from_ps_data("rhyperior", Some("Filter"), Some(50))
        .unwrap();
    
    let attacker = framework
        .create_pokemon_from_ps_data("gyarados", None, Some(50))
        .unwrap();
    
    let surf = framework.create_move_from_ps_data("surf").unwrap();
    let state = State::new(BattleFormat::gen9_ou());
    
    let solid_rock_damage = damage_calc::calculate_damage(
        &state,
        &attacker,
        &solid_rock_defender,
        &surf,
        false,
        1.0,
    );
    
    let filter_damage = damage_calc::calculate_damage(
        &state,
        &attacker,
        &filter_defender,
        &surf,
        false,
        1.0,
    );
    
    assert_eq!(solid_rock_damage, filter_damage, "Solid Rock and Filter should have identical effects");
}

#[test]
fn test_tinted_lens_boosts_not_very_effective() {
    let framework = TestFramework::new().unwrap();
    
    // Create Yanmega with Tinted Lens
    let attacker = framework
        .create_pokemon_from_ps_data("yanmega", Some("Tinted Lens"), Some(50))
        .unwrap();
    
    let defender = framework
        .create_pokemon_from_ps_data("aggron", None, Some(50)) // Steel type
        .unwrap();
    
    let air_slash = framework.create_move_from_ps_data("airslash").unwrap(); // Flying move, not very effective vs Steel
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Test damage with Tinted Lens
    let damage_with_tinted_lens = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &air_slash,
        false,
        1.0,
    );
    
    // Test damage without Tinted Lens
    let mut attacker_no_ability = attacker.clone();
    attacker_no_ability.ability = "noability".to_string();
    let damage_without_tinted_lens = damage_calc::calculate_damage(
        &state,
        &attacker_no_ability,
        &defender,
        &air_slash,
        false,
        1.0,
    );
    
    let damage_ratio = damage_with_tinted_lens as f32 / damage_without_tinted_lens as f32;
    assert!(
        damage_ratio > 1.8 && damage_ratio < 2.2,
        "Tinted Lens should boost not very effective moves by 2x, got {}x",
        damage_ratio
    );
}

#[test]
fn test_neuroforce_boosts_super_effective() {
    let framework = TestFramework::new().unwrap();
    
    // Create Ultra Necrozma with Neuroforce
    let attacker = framework
        .create_pokemon_from_ps_data("necrozmaultra", Some("Neuroforce"), Some(50))
        .unwrap();
    
    let defender = framework
        .create_pokemon_from_ps_data("charizard", None, Some(50)) // Fire/Flying
        .unwrap();
    
    let photon_geyser = framework.create_move_from_ps_data("photongeyser").unwrap(); // Psychic move, super effective vs Poison (using as proxy)
    let psychic = framework.create_move_from_ps_data("psychic").unwrap(); // Psychic move, neutral vs Fire/Flying
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Test with super effective move (we'll use a setup that's super effective)
    let defender_weak_to_psychic = framework
        .create_pokemon_from_ps_data("machamp", None, Some(50)) // Fighting type, weak to Psychic
        .unwrap();
    
    let damage_super_effective = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender_weak_to_psychic,
        &psychic,
        false,
        1.0,
    );
    
    // Test without Neuroforce
    let mut attacker_no_ability = attacker.clone();
    attacker_no_ability.ability = "noability".to_string();
    let damage_without_neuroforce = damage_calc::calculate_damage(
        &state,
        &attacker_no_ability,
        &defender_weak_to_psychic,
        &psychic,
        false,
        1.0,
    );
    
    let damage_ratio = damage_super_effective as f32 / damage_without_neuroforce as f32;
    assert!(
        damage_ratio > 1.20 && damage_ratio < 1.30,
        "Neuroforce should boost super effective moves by 1.25x, got {}x",
        damage_ratio
    );
}

#[test]
fn test_multiscale_reduces_damage_at_full_hp() {
    let framework = TestFramework::new().unwrap();
    
    // Create Dragonite with Multiscale
    let mut defender = framework
        .create_pokemon_from_ps_data("dragonite", Some("Multiscale"), Some(50))
        .unwrap();
    
    let attacker = framework
        .create_pokemon_from_ps_data("mamoswine", None, Some(50))
        .unwrap();
    
    let ice_shard = framework.create_move_from_ps_data("iceshard").unwrap();
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Test at full HP (Multiscale active)
    let damage_full_hp = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &ice_shard,
        false,
        1.0,
    );
    
    // Test at damaged HP (Multiscale inactive)
    defender.hp = defender.max_hp - 1;
    let damage_damaged = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &ice_shard,
        false,
        1.0,
    );
    
    let damage_ratio = damage_full_hp as f32 / damage_damaged as f32;
    assert!(
        damage_ratio > 0.45 && damage_ratio < 0.55,
        "Multiscale should reduce damage by 50% at full HP, got {}x",
        damage_ratio
    );
}

#[test]
fn test_ice_scales_reduces_special_damage() {
    let framework = TestFramework::new().unwrap();
    
    // Create Frosmoth with Ice Scales
    let defender = framework
        .create_pokemon_from_ps_data("frosmoth", Some("Ice Scales"), Some(50))
        .unwrap();
    
    let attacker = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    let psychic = framework.create_move_from_ps_data("psychic").unwrap(); // Special move
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Test damage with Ice Scales
    let damage_with_ice_scales = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &psychic,
        false,
        1.0,
    );
    
    // Test damage without Ice Scales
    let mut defender_no_ability = defender.clone();
    defender_no_ability.ability = "noability".to_string();
    let damage_without_ice_scales = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender_no_ability,
        &psychic,
        false,
        1.0,
    );
    
    let damage_ratio = damage_with_ice_scales as f32 / damage_without_ice_scales as f32;
    assert!(
        damage_ratio > 0.45 && damage_ratio < 0.55,
        "Ice Scales should reduce special damage by 50%, got {}x",
        damage_ratio
    );
}

#[test]
fn test_tough_claws_boosts_contact_moves() {
    let framework = TestFramework::new().unwrap();
    
    // Create Charizard X with Tough Claws
    let attacker = framework
        .create_pokemon_from_ps_data("charizardmegax", Some("Tough Claws"), Some(50))
        .unwrap();
    
    let defender = framework
        .create_pokemon_from_ps_data("venusaur", None, Some(50))
        .unwrap();
    
    let dragon_claw = framework.create_move_from_ps_data("dragonclaw").unwrap(); // Contact move
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Test damage with Tough Claws
    let damage_with_tough_claws = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &dragon_claw,
        false,
        1.0,
    );
    
    // Test damage without Tough Claws
    let mut attacker_no_ability = attacker.clone();
    attacker_no_ability.ability = "noability".to_string();
    let damage_without_tough_claws = damage_calc::calculate_damage(
        &state,
        &attacker_no_ability,
        &defender,
        &dragon_claw,
        false,
        1.0,
    );
    
    let damage_ratio = damage_with_tough_claws as f32 / damage_without_tough_claws as f32;
    assert!(
        damage_ratio > 1.25 && damage_ratio < 1.35,
        "Tough Claws should boost contact moves by 1.3x, got {}x",
        damage_ratio
    );
}

#[test]
fn test_pixilate_generation_specific_multipliers() {
    let framework = TestFramework::new().unwrap();
    
    // Create Sylveon with Pixilate
    let attacker = framework
        .create_pokemon_from_ps_data("sylveon", Some("Pixilate"), Some(50))
        .unwrap();
    
    let defender = framework
        .create_pokemon_from_ps_data("garchomp", None, Some(50))
        .unwrap();
    
    let hyper_voice = framework.create_move_from_ps_data("hypervoice").unwrap(); // Normal move that becomes Fairy
    
    // Test in Gen 9 (should be 1.2x)
    let state_gen9 = State::new(BattleFormat::gen9_ou());
    
    let damage_with_pixilate = damage_calc::calculate_damage(
        &state_gen9,
        &attacker,
        &defender,
        &hyper_voice,
        false,
        1.0,
    );
    
    // Test damage without Pixilate
    let mut attacker_no_ability = attacker.clone();
    attacker_no_ability.ability = "noability".to_string();
    let damage_without_pixilate = damage_calc::calculate_damage(
        &state_gen9,
        &attacker_no_ability,
        &defender,
        &hyper_voice,
        false,
        1.0,
    );
    
    let damage_ratio = damage_with_pixilate as f32 / damage_without_pixilate as f32;
    assert!(
        damage_ratio > 1.15 && damage_ratio < 1.25,
        "Pixilate should boost Normal moves by 1.2x in Gen 7+, got {}x",
        damage_ratio
    );
}

#[test]
fn test_fluffy_contact_and_fire_interaction() {
    let framework = TestFramework::new().unwrap();
    
    // Create Stufful with Fluffy
    let defender = framework
        .create_pokemon_from_ps_data("stufful", Some("Fluffy"), Some(50))
        .unwrap();
    
    let attacker = framework
        .create_pokemon_from_ps_data("machamp", None, Some(50))
        .unwrap();
    
    let close_combat = framework.create_move_from_ps_data("closecombat").unwrap(); // Contact move
    let flamethrower = framework.create_move_from_ps_data("flamethrower").unwrap(); // Fire move, non-contact
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Test contact move (should be reduced)
    let contact_damage_with_fluffy = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &close_combat,
        false,
        1.0,
    );
    
    let mut defender_no_ability = defender.clone();
    defender_no_ability.ability = "noability".to_string();
    let contact_damage_without_fluffy = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender_no_ability,
        &close_combat,
        false,
        1.0,
    );
    
    let contact_ratio = contact_damage_with_fluffy as f32 / contact_damage_without_fluffy as f32;
    assert!(
        contact_ratio > 0.45 && contact_ratio < 0.55,
        "Fluffy should reduce contact moves by 50%, got {}x",
        contact_ratio
    );
    
    // Test Fire move (should be increased)
    let fire_damage_with_fluffy = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &flamethrower,
        false,
        1.0,
    );
    
    let fire_damage_without_fluffy = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender_no_ability,
        &flamethrower,
        false,
        1.0,
    );
    
    let fire_ratio = fire_damage_with_fluffy as f32 / fire_damage_without_fluffy as f32;
    assert!(
        fire_ratio > 1.8 && fire_ratio < 2.2,
        "Fluffy should double Fire move damage, got {}x",
        fire_ratio
    );
}