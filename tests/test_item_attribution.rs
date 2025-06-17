use tapu_simu::test_framework::TestFramework;
use tapu_simu::state::State;
use tapu_simu::battle_format::BattleFormat;
use tapu_simu::engine::damage_calc;

/// Test that item attribution is correct - attacker vs defender items
#[test]
fn test_item_attribution_critical_fixes() {
    let framework = TestFramework::new().unwrap();
    let state = State::new(BattleFormat::gen9_ou());

    // Test 1: Metal Powder should only work when DEFENDER is Ditto
    let attacker = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    let mut ditto_defender = framework
        .create_pokemon_from_ps_data("ditto", None, Some(50))
        .unwrap();
    
    let psychic_move = framework.create_move_from_ps_data("psychic").unwrap();
    
    // Ditto defender WITHOUT Metal Powder
    let damage_without_metal_powder = damage_calc::calculate_damage(
        &state, &attacker, &ditto_defender, &psychic_move, false, 1.0
    );
    
    // Ditto defender WITH Metal Powder
    ditto_defender.item = Some("Metal Powder".to_string());
    let damage_with_metal_powder = damage_calc::calculate_damage(
        &state, &attacker, &ditto_defender, &psychic_move, false, 1.0
    );
    
    let reduction_ratio = damage_with_metal_powder as f32 / damage_without_metal_powder as f32;
    assert!(
        reduction_ratio > 0.45 && reduction_ratio < 0.55,
        "Metal Powder should halve damage when DEFENDER is Ditto, got {}x reduction",
        reduction_ratio
    );
    
    // Test 2: Metal Powder should NOT work when ATTACKER has it (wrong attribution)
    let mut alakazam_attacker = attacker.clone();
    alakazam_attacker.item = Some("Metal Powder".to_string());
    
    let non_ditto_defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    let damage_attacker_metal_powder = damage_calc::calculate_damage(
        &state, &alakazam_attacker, &non_ditto_defender, &psychic_move, false, 1.0
    );
    
    let damage_attacker_no_item = damage_calc::calculate_damage(
        &state, &attacker, &non_ditto_defender, &psychic_move, false, 1.0
    );
    
    let no_effect_ratio = damage_attacker_metal_powder as f32 / damage_attacker_no_item as f32;
    assert!(
        no_effect_ratio > 0.95 && no_effect_ratio < 1.05,
        "Metal Powder should have NO effect when attacker has it (wrong attribution), got {}x",
        no_effect_ratio
    );
    
    // Test 3: Assault Vest should boost defender's Special Defense, not block attacker
    let special_attacker = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    let mut av_defender = framework
        .create_pokemon_from_ps_data("chansey", None, Some(50))
        .unwrap();
    
    let special_move = framework.create_move_from_ps_data("psychic").unwrap();
    
    // Without Assault Vest
    let damage_without_av = damage_calc::calculate_damage(
        &state, &special_attacker, &av_defender, &special_move, false, 1.0
    );
    
    // With Assault Vest on defender
    av_defender.item = Some("Assault Vest".to_string());
    let damage_with_av = damage_calc::calculate_damage(
        &state, &special_attacker, &av_defender, &special_move, false, 1.0
    );
    
    let av_reduction = damage_with_av as f32 / damage_without_av as f32;
    assert!(
        av_reduction > 0.60 && av_reduction < 0.70,
        "Assault Vest should reduce special damage by ~1.5x (to ~66%), got {}x",
        av_reduction
    );
    
    // Test 4: Punching Glove should only work when ATTACKER has it
    let mut punching_attacker = framework
        .create_pokemon_from_ps_data("machamp", None, Some(50))
        .unwrap();
    
    let punch_defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    let punch_move = framework.create_move_from_ps_data("thunderpunch").unwrap();
    
    // Without Punching Glove
    let damage_without_glove = damage_calc::calculate_damage(
        &state, &punching_attacker, &punch_defender, &punch_move, false, 1.0
    );
    
    // With Punching Glove on attacker
    punching_attacker.item = Some("Punching Glove".to_string());
    let damage_with_glove = damage_calc::calculate_damage(
        &state, &punching_attacker, &punch_defender, &punch_move, false, 1.0
    );
    
    let glove_boost = damage_with_glove as f32 / damage_without_glove as f32;
    assert!(
        glove_boost > 1.05 && glove_boost < 1.15,
        "Punching Glove should boost punch moves by ~1.1x when ATTACKER has it, got {}x",
        glove_boost
    );
    
    // Test 5: Punching Glove should NOT work when defender has it (wrong attribution)
    let non_punching_attacker = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    let mut glove_defender = punch_defender.clone();
    glove_defender.item = Some("Punching Glove".to_string());
    
    let damage_defender_glove = damage_calc::calculate_damage(
        &state, &non_punching_attacker, &glove_defender, &punch_move, false, 1.0
    );
    
    let damage_defender_no_glove = damage_calc::calculate_damage(
        &state, &non_punching_attacker, &punch_defender, &punch_move, false, 1.0
    );
    
    let wrong_attribution_ratio = damage_defender_glove as f32 / damage_defender_no_glove as f32;
    assert!(
        wrong_attribution_ratio > 0.95 && wrong_attribution_ratio < 1.05,
        "Punching Glove should have NO effect when defender has it (wrong attribution), got {}x",
        wrong_attribution_ratio
    );
}

/// Test reactive items (defender items that trigger on incoming moves)
#[test]
fn test_reactive_items_attribution() {
    let framework = TestFramework::new().unwrap();
    let state = State::new(BattleFormat::gen9_ou());

    // Weakness Policy should only trigger when defender has it and gets hit by super effective move
    let fire_attacker = framework
        .create_pokemon_from_ps_data("charizard", None, Some(50))
        .unwrap();
    
    let mut grass_defender = framework
        .create_pokemon_from_ps_data("venusaur", None, Some(50))
        .unwrap();
    grass_defender.item = Some("Weakness Policy".to_string());
    
    let fire_move = framework.create_move_from_ps_data("flamethrower").unwrap();
    
    // This should trigger Weakness Policy since Fire is super effective vs Grass
    // Note: The actual stat boost would happen post-damage in instruction generation
    // For now we're just testing that the item attribution is correct
    
    // Test that it's a defender item
    use tapu_simu::engine::items::get_item_by_name;
    let weakness_policy = get_item_by_name("Weakness Policy").unwrap();
    assert!(!weakness_policy.is_attacker_item(), "Weakness Policy should not be an attacker item");
    assert!(weakness_policy.is_defender_item(), "Weakness Policy should be a defender item");
    
    // Test other reactive items
    let absorb_bulb = get_item_by_name("Absorb Bulb").unwrap();
    assert!(!absorb_bulb.is_attacker_item(), "Absorb Bulb should not be an attacker item");
    assert!(absorb_bulb.is_defender_item(), "Absorb Bulb should be a defender item");
    
    let cell_battery = get_item_by_name("Cell Battery").unwrap();
    assert!(!cell_battery.is_attacker_item(), "Cell Battery should not be an attacker item");
    assert!(cell_battery.is_defender_item(), "Cell Battery should be a defender item");
    
    let focus_sash = get_item_by_name("Focus Sash").unwrap();
    assert!(!focus_sash.is_attacker_item(), "Focus Sash should not be an attacker item");
    assert!(focus_sash.is_defender_item(), "Focus Sash should be a defender item");
}

/// Test that attacker items are properly categorized
#[test]
fn test_attacker_items_attribution() {
    use tapu_simu::engine::items::get_item_by_name;
    
    // Choice items should be attacker items
    let choice_band = get_item_by_name("Choice Band").unwrap();
    assert!(choice_band.is_attacker_item(), "Choice Band should be an attacker item");
    assert!(!choice_band.is_defender_item(), "Choice Band should not be a defender item");
    
    // Life Orb should be attacker item
    let life_orb = get_item_by_name("Life Orb").unwrap();
    assert!(life_orb.is_attacker_item(), "Life Orb should be an attacker item");
    assert!(!life_orb.is_defender_item(), "Life Orb should not be a defender item");
    
    // Punching Glove should be attacker item
    let punching_glove = get_item_by_name("Punching Glove").unwrap();
    assert!(punching_glove.is_attacker_item(), "Punching Glove should be an attacker item");
    assert!(!punching_glove.is_defender_item(), "Punching Glove should not be a defender item");
    
    // Shell Bell should be attacker item (drain effect)
    let shell_bell = get_item_by_name("Shell Bell").unwrap();
    assert!(shell_bell.is_attacker_item(), "Shell Bell should be an attacker item");
    assert!(!shell_bell.is_defender_item(), "Shell Bell should not be a defender item");
}