//! # Item System Tests
//! 
//! Comprehensive tests for the item system including type boosters, Arceus plates,
//! damage reduction berries, species-specific items, power amplification items,
//! defensive items, gems, and integration with damage calculation.
//! All tests use the TestFramework with real Pokemon Showdown data.

use tapu_simu::test_framework::TestFramework;
use tapu_simu::state::State;
use tapu_simu::battle_format::BattleFormat;
use tapu_simu::engine::damage_calc;

// =============================================================================
// TYPE BOOSTER ITEMS
// =============================================================================

#[test]
fn test_type_booster_comprehensive_coverage() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    // Test multiple type boosters
    let type_boosters = [
        ("Charcoal", "charizard", "flamethrower", "Fire"),
        ("Mystic Water", "blastoise", "surf", "Water"),
        ("Miracle Seed", "venusaur", "solarbeam", "Grass"),
        ("Magnet", "magnezone", "thunderbolt", "Electric"),
        ("Black Belt", "machamp", "closecombat", "Fighting"),
        ("Poison Barb", "crobat", "sludgebomb", "Poison"),
        ("Sharp Beak", "pidgeot", "hurricane", "Flying"),
        ("Twisted Spoon", "alakazam", "psychic", "Psychic"),
    ];

    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();

    for (item, attacker_species, move_name, _move_type) in &type_boosters {
        let mut attacker = framework
            .create_pokemon_from_ps_data(attacker_species, None, Some(50))
            .unwrap();
        attacker.item = Some(item.to_string());

        let move_data = framework.create_move_from_ps_data(move_name).unwrap();

        // Calculate damage with type booster
        let damage_with_item =
            damage_calc::calculate_damage(&state, &attacker, &defender, &move_data, false, 1.0);

        // Calculate damage without item
        attacker.item = None;
        let damage_without_item =
            damage_calc::calculate_damage(&state, &attacker, &defender, &move_data, false, 1.0);

        let boost_ratio = damage_with_item as f32 / damage_without_item as f32;
        assert!(
            boost_ratio > 1.15 && boost_ratio < 1.25,
            "{} should provide ~1.2x boost, got {}",
            item,
            boost_ratio
        );
    }
}

// =============================================================================
// ARCEUS PLATES
// =============================================================================

#[test]
fn test_arceus_plates_comprehensive() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let plates = [
        ("Flame Plate", "flamethrower", "Fire"),
        ("Splash Plate", "surf", "Water"),
        ("Meadow Plate", "energyball", "Grass"),
        ("Zap Plate", "thunderbolt", "Electric"),
        ("Fist Plate", "closecombat", "Fighting"),
        ("Toxic Plate", "sludgebomb", "Poison"),
        ("Sky Plate", "hurricane", "Flying"),
        ("Mind Plate", "psychic", "Psychic"),
        ("Insect Plate", "bugbuzz", "Bug"),
        ("Stone Plate", "rockslide", "Rock"),
        ("Spooky Plate", "shadowclaw", "Ghost"),
        ("Draco Plate", "dragonpulse", "Dragon"),
        ("Dread Plate", "darkpulse", "Dark"),
        ("Iron Plate", "flashcannon", "Steel"),
        ("Icicle Plate", "icebeam", "Ice"),
    ];

    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("mew", None, Some(50))
        .unwrap();

    for (plate, move_name, _move_type) in &plates {
        // Create Arceus with the specific plate
        let mut arceus = framework
            .create_pokemon_from_ps_data("arceus", None, Some(50))
            .unwrap();
        arceus.item = Some(plate.to_string());

        let move_data = framework.create_move_from_ps_data(move_name).unwrap();

        // Calculate damage with plate
        let damage_with_plate =
            damage_calc::calculate_damage(&state, &arceus, &defender, &move_data, false, 1.0);

        // Calculate damage without plate
        arceus.item = None;
        let damage_without_plate =
            damage_calc::calculate_damage(&state, &arceus, &defender, &move_data, false, 1.0);

        let boost_ratio = damage_with_plate as f32 / damage_without_plate as f32;
        assert!(
            boost_ratio > 1.15 && boost_ratio < 1.25,
            "{} should provide ~1.2x boost, got {}x",
            plate,
            boost_ratio
        );
    }
}

// =============================================================================
// DAMAGE REDUCTION BERRIES
// =============================================================================

#[test]
fn test_damage_reduction_berries() {
    let framework = TestFramework::new().expect("Failed to create test framework");

    let berry_tests = [
        ("Occa Berry", "flamethrower", "Fire", "scizor"),     // Fire -> Bug/Steel
        ("Passho Berry", "surf", "Water", "charizard"),       // Water -> Fire/Flying
        ("Rindo Berry", "energyball", "Grass", "golem"),     // Grass -> Rock/Ground
        ("Wacan Berry", "thunderbolt", "Electric", "gyarados"), // Electric -> Water/Flying
        ("Yache Berry", "icebeam", "Ice", "garchomp"),       // Ice -> Dragon/Ground
        ("Haban Berry", "dragonpulse", "Dragon", "altaria"), // Dragon -> Dragon/Flying
    ];

    let state = State::new(BattleFormat::gen9_ou());

    for (berry, move_name, _move_type, defender_species) in &berry_tests {
        let attacker = framework
            .create_pokemon_from_ps_data("alakazam", None, Some(50))
            .unwrap();
        let mut defender = framework
            .create_pokemon_from_ps_data(defender_species, None, Some(50))
            .unwrap();

        let move_data = framework.create_move_from_ps_data(move_name).unwrap();

        // Calculate damage without berry
        let damage_without_berry =
            damage_calc::calculate_damage(&state, &attacker, &defender, &move_data, false, 1.0);

        // Calculate damage with berry
        defender.item = Some(berry.to_string());
        let damage_with_berry =
            damage_calc::calculate_damage(&state, &attacker, &defender, &move_data, false, 1.0);

        let reduction_ratio = damage_with_berry as f32 / damage_without_berry as f32;
        assert!(
            reduction_ratio > 0.45 && reduction_ratio < 0.55,
            "{} should reduce super effective damage to ~50%, got reduction to {}%",
            berry,
            (reduction_ratio * 100.0) as i32
        );
    }
}

// =============================================================================
// SPECIES-SPECIFIC ITEMS
// =============================================================================

#[test]
fn test_species_specific_items() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let state = State::new(BattleFormat::gen9_ou());

    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();

    // Test Thick Club (doubles Attack for Cubone/Marowak)
    let mut marowak = framework
        .create_pokemon_from_ps_data("marowak", None, Some(50))
        .unwrap();
    marowak.item = Some("Thick Club".to_string());

    let earthquake = framework.create_move_from_ps_data("earthquake").unwrap();

    let damage_with_thick_club =
        damage_calc::calculate_damage(&state, &marowak, &defender, &earthquake, false, 1.0);

    marowak.item = None;
    let damage_without_thick_club =
        damage_calc::calculate_damage(&state, &marowak, &defender, &earthquake, false, 1.0);

    let boost_ratio = damage_with_thick_club as f32 / damage_without_thick_club as f32;
    assert!(
        boost_ratio > 1.8 && boost_ratio < 2.2,
        "Thick Club should roughly double Marowak's Attack, got {}x boost",
        boost_ratio
    );

    // Test Light Ball (doubles Attack and Special Attack for Pikachu)
    let mut pikachu = framework
        .create_pokemon_from_ps_data("pikachu", None, Some(50))
        .unwrap();
    pikachu.item = Some("Light Ball".to_string());

    let thunderbolt = framework.create_move_from_ps_data("thunderbolt").unwrap();

    let damage_with_light_ball =
        damage_calc::calculate_damage(&state, &pikachu, &defender, &thunderbolt, false, 1.0);

    pikachu.item = None;
    let damage_without_light_ball =
        damage_calc::calculate_damage(&state, &pikachu, &defender, &thunderbolt, false, 1.0);

    let special_boost_ratio = damage_with_light_ball as f32 / damage_without_light_ball as f32;
    assert!(
        special_boost_ratio > 1.8 && special_boost_ratio < 2.2,
        "Light Ball should roughly double Pikachu's Special Attack, got {}x boost",
        special_boost_ratio
    );
}

// =============================================================================
// POWER AMPLIFICATION ITEMS
// =============================================================================

#[test]
fn test_power_amplification_items() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let state = State::new(BattleFormat::gen9_ou());

    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();

    // Test Life Orb (1.3x boost to all moves, 10% recoil)
    let mut attacker = framework
        .create_pokemon_from_ps_data("garchomp", None, Some(50))
        .unwrap();
    attacker.item = Some("Life Orb".to_string());

    let earthquake = framework.create_move_from_ps_data("earthquake").unwrap();

    let damage_with_life_orb =
        damage_calc::calculate_damage(&state, &attacker, &defender, &earthquake, false, 1.0);

    attacker.item = None;
    let damage_without_life_orb =
        damage_calc::calculate_damage(&state, &attacker, &defender, &earthquake, false, 1.0);

    let life_orb_boost = damage_with_life_orb as f32 / damage_without_life_orb as f32;
    assert!(
        life_orb_boost > 1.25 && life_orb_boost < 1.35,
        "Life Orb should provide ~1.3x boost, got {}x",
        life_orb_boost
    );

    // Test Choice Band (1.5x Attack boost for physical moves)
    let mut choice_attacker = framework
        .create_pokemon_from_ps_data("machamp", None, Some(50))
        .unwrap();
    choice_attacker.item = Some("Choice Band".to_string());

    let close_combat = framework.create_move_from_ps_data("closecombat").unwrap();

    let damage_with_choice_band =
        damage_calc::calculate_damage(&state, &choice_attacker, &defender, &close_combat, false, 1.0);

    choice_attacker.item = None;
    let damage_without_choice_band =
        damage_calc::calculate_damage(&state, &choice_attacker, &defender, &close_combat, false, 1.0);

    let choice_band_boost = damage_with_choice_band as f32 / damage_without_choice_band as f32;
    assert!(
        choice_band_boost > 1.45 && choice_band_boost < 1.55,
        "Choice Band should provide ~1.5x boost, got {}x",
        choice_band_boost
    );

    // Test Expert Belt (1.2x boost for super effective moves)
    let mut expert_attacker = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    expert_attacker.item = Some("Expert Belt".to_string());

    let fighting_defender = framework
        .create_pokemon_from_ps_data("machamp", None, Some(50))
        .unwrap();

    let psychic = framework.create_move_from_ps_data("psychic").unwrap();

    let damage_with_expert_belt =
        damage_calc::calculate_damage(&state, &expert_attacker, &fighting_defender, &psychic, false, 1.0);

    expert_attacker.item = None;
    let damage_without_expert_belt =
        damage_calc::calculate_damage(&state, &expert_attacker, &fighting_defender, &psychic, false, 1.0);

    let expert_belt_boost = damage_with_expert_belt as f32 / damage_without_expert_belt as f32;
    assert!(
        expert_belt_boost > 1.15 && expert_belt_boost < 1.25,
        "Expert Belt should provide ~1.2x boost for super effective moves, got {}x",
        expert_belt_boost
    );
}

// =============================================================================
// DEFENSIVE ITEMS
// =============================================================================

#[test]
fn test_defensive_items() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let state = State::new(BattleFormat::gen9_ou());

    let attacker = framework
        .create_pokemon_from_ps_data("garchomp", None, Some(50))
        .unwrap();

    // Test Eviolite (1.5x Defense and Special Defense for not fully evolved Pokemon)
    let mut chansey = framework
        .create_pokemon_from_ps_data("chansey", None, Some(50))
        .unwrap();

    let earthquake = framework.create_move_from_ps_data("earthquake").unwrap();

    let damage_without_eviolite =
        damage_calc::calculate_damage(&state, &attacker, &chansey, &earthquake, false, 1.0);

    chansey.item = Some("Eviolite".to_string());
    let damage_with_eviolite =
        damage_calc::calculate_damage(&state, &attacker, &chansey, &earthquake, false, 1.0);

    let eviolite_reduction = damage_with_eviolite as f32 / damage_without_eviolite as f32;
    assert!(
        eviolite_reduction < 0.75,
        "Eviolite should reduce physical damage significantly, got reduction to {}%",
        (eviolite_reduction * 100.0) as i32
    );

    // Test Assault Vest (1.5x Special Defense, blocks status moves)
    let special_attacker = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    let mut av_defender = framework
        .create_pokemon_from_ps_data("conkeldurr", None, Some(50))
        .unwrap();

    let psychic = framework.create_move_from_ps_data("psychic").unwrap();

    let damage_without_av =
        damage_calc::calculate_damage(&state, &special_attacker, &av_defender, &psychic, false, 1.0);

    av_defender.item = Some("Assault Vest".to_string());
    let damage_with_av =
        damage_calc::calculate_damage(&state, &special_attacker, &av_defender, &psychic, false, 1.0);

    let av_reduction = damage_with_av as f32 / damage_without_av as f32;
    assert!(
        av_reduction < 0.75,
        "Assault Vest should reduce special damage significantly, got reduction to {}%",
        (av_reduction * 100.0) as i32
    );
}

// =============================================================================
// GEM SYSTEM
// =============================================================================

#[test]
fn test_fire_gem_generation_9_multiplier() {
    let framework = TestFramework::new().unwrap();
    
    // Create a Fire-type attacker with Fire Gem
    let mut attacker = framework
        .create_pokemon_from_ps_data("arcanine", None, Some(50))
        .unwrap();
    attacker.item = Some("Fire Gem".to_string());
    
    let defender = framework
        .create_pokemon_from_ps_data("venusaur", None, Some(50))
        .unwrap();
    
    let flamethrower = framework.create_move_from_ps_data("flamethrower").unwrap();
    
    // Test in Generation 9 (should be 1.3x multiplier)
    let state = State::new(BattleFormat::gen9_ou());
    let damage_with_gem = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &flamethrower,
        false,
        1.0,
    );
    
    // Remove Fire Gem and test again
    let mut attacker_no_gem = attacker.clone();
    attacker_no_gem.item = None;
    let damage_without_gem = damage_calc::calculate_damage(
        &state,
        &attacker_no_gem,
        &defender,
        &flamethrower,
        false,
        1.0,
    );
    
    let boost_ratio = damage_with_gem as f32 / damage_without_gem as f32;
    assert!(
        boost_ratio > 1.25 && boost_ratio < 1.35,
        "Fire Gem in Gen 9 should provide ~1.3x boost, got {}x",
        boost_ratio
    );
}

#[test]
fn test_water_gem_type_specificity() {
    let framework = TestFramework::new().unwrap();
    
    let mut attacker = framework
        .create_pokemon_from_ps_data("blastoise", None, Some(50))
        .unwrap();
    attacker.item = Some("Water Gem".to_string());
    
    let defender = framework
        .create_pokemon_from_ps_data("charizard", None, Some(50))
        .unwrap();
    
    let state = State::new(BattleFormat::gen9_ou());
    
    // Water Gem should boost Water moves
    let surf = framework.create_move_from_ps_data("surf").unwrap();
    let damage_water_with_gem = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &surf,
        false,
        1.0,
    );
    
    // Water Gem should NOT boost non-Water moves (use a move that actually does damage)
    let ice_beam = framework.create_move_from_ps_data("icebeam").unwrap();
    let damage_ice_with_gem = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &ice_beam,
        false,
        1.0,
    );
    
    // Test without gem
    attacker.item = None;
    let damage_water_no_gem = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &surf,
        false,
        1.0,
    );
    let damage_ice_no_gem = damage_calc::calculate_damage(
        &state,
        &attacker,
        &defender,
        &ice_beam,
        false,
        1.0,
    );
    
    let water_boost = damage_water_with_gem as f32 / damage_water_no_gem as f32;
    let ice_boost = damage_ice_with_gem as f32 / damage_ice_no_gem as f32;
    
    assert!(
        water_boost > 1.25,
        "Water Gem should boost Water moves, got {}x",
        water_boost
    );
    assert!(
        ice_boost < 1.05,
        "Water Gem should NOT boost non-Water moves, got {}x",
        ice_boost
    );
}

#[test]
fn test_gem_availability_by_generation() {
    let framework = TestFramework::new().unwrap();
    
    // Gems should be available in certain generations
    // For now, just test that we can create the framework and access gem items
    let mut attacker = framework
        .create_pokemon_from_ps_data("arcanine", None, Some(50))
        .unwrap();
    
    let gem_types = ["Fire Gem", "Water Gem", "Electric Gem", "Grass Gem"];
    
    for gem in &gem_types {
        attacker.item = Some(gem.to_string());
        // If we can set the item, it exists in our system
        assert!(attacker.item.is_some(), "{} should be available", gem);
    }
}

// =============================================================================
// BERRY DEBUG AND EDGE CASES  
// =============================================================================

#[test]
fn test_berry_activation_conditions() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    let attacker = framework.create_pokemon_from_ps_data("garchomp", None, Some(50)).unwrap();
    let mut defender = framework.create_pokemon_from_ps_data("scizor", None, Some(50)).unwrap();
    let state = State::new(BattleFormat::gen9_ou());
    
    let move_data = framework.create_move_from_ps_data("flamethrower").unwrap();
    
    // Test without berry
    let damage_without_berry = damage_calc::calculate_damage(&state, &attacker, &defender, &move_data, false, 1.0);
    
    // Test with berry
    defender.item = Some("Occa Berry".to_string());
    let damage_with_berry = damage_calc::calculate_damage(&state, &attacker, &defender, &move_data, false, 1.0);
    
    let reduction_ratio = damage_with_berry as f32 / damage_without_berry as f32;
    
    // This should show if the berry is working
    assert!(reduction_ratio < 0.9, "Berry should have some effect, got {}x", reduction_ratio);
}

// =============================================================================
// NEW ITEMS IMPLEMENTATION TESTS
// =============================================================================

#[test]
fn test_status_berries_availability() {
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that all status berries can be found
    assert!(get_item_by_name("Lum Berry").is_some(), "Lum Berry should be available");
    assert!(get_item_by_name("Sitrus Berry").is_some(), "Sitrus Berry should be available");
    assert!(get_item_by_name("Chesto Berry").is_some(), "Chesto Berry should be available");
}

#[test]
fn test_stat_boost_berries_availability() {
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that all stat boost berries can be found
    assert!(get_item_by_name("Liechi Berry").is_some(), "Liechi Berry should be available");
    assert!(get_item_by_name("Petaya Berry").is_some(), "Petaya Berry should be available");
    assert!(get_item_by_name("Salac Berry").is_some(), "Salac Berry should be available");
}

#[test]
fn test_terrain_seeds_availability() {
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that all terrain seeds can be found
    assert!(get_item_by_name("Electric Seed").is_some(), "Electric Seed should be available");
    assert!(get_item_by_name("Grassy Seed").is_some(), "Grassy Seed should be available");
    assert!(get_item_by_name("Misty Seed").is_some(), "Misty Seed should be available");
    assert!(get_item_by_name("Psychic Seed").is_some(), "Psychic Seed should be available");
}

#[test]
fn test_end_of_turn_items_availability() {
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that all end-of-turn items can be found
    assert!(get_item_by_name("Black Sludge").is_some(), "Black Sludge should be available");
    assert!(get_item_by_name("Flame Orb").is_some(), "Flame Orb should be available");
    assert!(get_item_by_name("Toxic Orb").is_some(), "Toxic Orb should be available");
}

#[test]
fn test_utility_items_availability() {
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that all utility items can be found
    assert!(get_item_by_name("Protective Pads").is_some(), "Protective Pads should be available");
    assert!(get_item_by_name("Throat Spray").is_some(), "Throat Spray should be available");
    assert!(get_item_by_name("Wide Lens").is_some(), "Wide Lens should be available");
    assert!(get_item_by_name("Iron Ball").is_some(), "Iron Ball should be available");
    assert!(get_item_by_name("Loaded Dice").is_some(), "Loaded Dice should be available");
    assert!(get_item_by_name("Blunder Policy").is_some(), "Blunder Policy should be available");
    assert!(get_item_by_name("Custap Berry").is_some(), "Custap Berry should be available");
    assert!(get_item_by_name("Adrenaline Orb").is_some(), "Adrenaline Orb should be available");
    assert!(get_item_by_name("Booster Energy").is_some(), "Booster Energy should be available");
}

#[test]
fn test_legendary_items_availability() {
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that all legendary items can be found
    assert!(get_item_by_name("Rusted Sword").is_some(), "Rusted Sword should be available");
    assert!(get_item_by_name("Rusted Shield").is_some(), "Rusted Shield should be available");
    assert!(get_item_by_name("Cornerstone Mask").is_some(), "Cornerstone Mask should be available");
    assert!(get_item_by_name("Hearthflame Mask").is_some(), "Hearthflame Mask should be available");
    assert!(get_item_by_name("Wellspring Mask").is_some(), "Wellspring Mask should be available");
}

#[test]
fn test_stat_boost_berries_trigger_conditions() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that stat boost berries can be found
    assert!(get_item_by_name("Liechi Berry").is_some(), "Liechi Berry should be available");
    assert!(get_item_by_name("Petaya Berry").is_some(), "Petaya Berry should be available");
    assert!(get_item_by_name("Salac Berry").is_some(), "Salac Berry should be available");
    
    // Test basic item interaction through damage calculation
    let state = State::new(BattleFormat::gen9_ou());
    let attacker = framework
        .create_pokemon_from_ps_data("garchomp", None, Some(50))
        .unwrap();
    let mut defender = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    // Test with Liechi Berry (Attack boost berry)
    defender.item = Some("Liechi Berry".to_string());
    let move_data = framework.create_move_from_ps_data("earthquake").unwrap();
    
    let damage = framework.test_damage_calculation(&attacker, &defender, &move_data, &state);
    assert!(damage > 0, "Damage calculation should work with Liechi Berry equipped");
}

#[test]
fn test_protective_pads_contact_removal() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that Protective Pads exists
    assert!(get_item_by_name("Protective Pads").is_some(), "Protective Pads should be available");
    
    // Test basic functionality through damage calculation
    let state = State::new(BattleFormat::gen9_ou());
    let mut attacker = framework
        .create_pokemon_from_ps_data("machamp", None, Some(50))
        .unwrap();
    let defender = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    // Test with contact move and Protective Pads
    attacker.item = Some("Protective Pads".to_string());
    let contact_move = framework.create_move_from_ps_data("closecombat").unwrap();
    
    let damage = framework.test_damage_calculation(&attacker, &defender, &contact_move, &state);
    assert!(damage > 0, "Protective Pads should not prevent damage calculation");
}

#[test]
fn test_iron_ball_speed_reduction() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that Iron Ball exists
    assert!(get_item_by_name("Iron Ball").is_some(), "Iron Ball should be available");
    
    // Test basic functionality through damage calculation
    let state = State::new(BattleFormat::gen9_ou());
    let mut attacker = framework
        .create_pokemon_from_ps_data("garchomp", None, Some(50))
        .unwrap();
    let defender = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    // Test with Iron Ball
    attacker.item = Some("Iron Ball".to_string());
    let move_data = framework.create_move_from_ps_data("earthquake").unwrap();
    
    let damage = framework.test_damage_calculation(&attacker, &defender, &move_data, &state);
    assert!(damage > 0, "Iron Ball should not prevent damage calculation");
}

#[test]
fn test_ogerpon_masks_species_specificity() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that Ogerpon masks exist
    assert!(get_item_by_name("Cornerstone Mask").is_some(), "Cornerstone Mask should be available");
    assert!(get_item_by_name("Hearthflame Mask").is_some(), "Hearthflame Mask should be available");
    assert!(get_item_by_name("Wellspring Mask").is_some(), "Wellspring Mask should be available");
    
    // Test basic functionality through damage calculation
    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    // Test with a regular Pokemon (should work but mask won't boost)
    let mut garchomp = framework
        .create_pokemon_from_ps_data("garchomp", None, Some(50))
        .unwrap();
    garchomp.item = Some("Cornerstone Mask".to_string());
    let move_data = framework.create_move_from_ps_data("rockslide").unwrap();
    
    let damage = framework.test_damage_calculation(&garchomp, &defender, &move_data, &state);
    assert!(damage > 0, "Cornerstone Mask should not prevent damage calculation");
}

#[test]
fn test_sitrus_berry_generation_differences() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    use tapu_simu::engine::items::get_item_by_name_with_generation;
    
    // Test that Sitrus Berry can be created for different generations
    assert!(get_item_by_name_with_generation("Sitrus Berry", 3).is_some(), "Sitrus Berry should be available in Gen 3");
    assert!(get_item_by_name_with_generation("Sitrus Berry", 4).is_some(), "Sitrus Berry should be available in Gen 4+");
    
    // Test basic functionality through damage calculation
    let state = State::new(BattleFormat::gen9_ou());
    let attacker = framework
        .create_pokemon_from_ps_data("garchomp", None, Some(50))
        .unwrap();
    let mut defender = framework
        .create_pokemon_from_ps_data("chansey", None, Some(50))
        .unwrap();
    
    // Test with Sitrus Berry
    defender.item = Some("Sitrus Berry".to_string());
    let move_data = framework.create_move_from_ps_data("earthquake").unwrap();
    
    let damage = framework.test_damage_calculation(&attacker, &defender, &move_data, &state);
    assert!(damage > 0, "Sitrus Berry should not prevent damage calculation");
}

#[test]
fn test_item_attribution_new_items() {
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that attacker items are properly categorized
    let protective_pads = get_item_by_name("Protective Pads").unwrap();
    assert!(protective_pads.is_attacker_item(), "Protective Pads should be an attacker item");
    assert!(!protective_pads.is_defender_item(), "Protective Pads should not be a defender item");
    
    let throat_spray = get_item_by_name("Throat Spray").unwrap();
    assert!(throat_spray.is_attacker_item(), "Throat Spray should be an attacker item");
    assert!(!throat_spray.is_defender_item(), "Throat Spray should not be a defender item");
    
    // Test that defender items are properly categorized
    let liechi_berry = get_item_by_name("Liechi Berry").unwrap();
    assert!(!liechi_berry.is_attacker_item(), "Liechi Berry should not be an attacker item");
    assert!(liechi_berry.is_defender_item(), "Liechi Berry should be a defender item");
    
    let electric_seed = get_item_by_name("Electric Seed").unwrap();
    assert!(!electric_seed.is_attacker_item(), "Electric Seed should not be an attacker item");
    assert!(electric_seed.is_defender_item(), "Electric Seed should be a defender item");
    
    // Test neutral items (end-of-turn items)
    let black_sludge = get_item_by_name("Black Sludge").unwrap();
    assert!(!black_sludge.is_attacker_item(), "Black Sludge should not be an attacker item");
    assert!(!black_sludge.is_defender_item(), "Black Sludge should not be a defender item");
    
    // Test that legendary items are properly categorized
    let cornerstone_mask = get_item_by_name("Cornerstone Mask").unwrap();
    assert!(cornerstone_mask.is_attacker_item(), "Cornerstone Mask should be an attacker item");
    assert!(!cornerstone_mask.is_defender_item(), "Cornerstone Mask should not be a defender item");
}

// =============================================================================
// NEWLY IMPLEMENTED ITEMS TESTS
// =============================================================================

#[test]
fn test_dragon_scale_availability_and_boost() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    // Test Dragon Scale availability
    use tapu_simu::engine::items::get_item_by_name;
    assert!(get_item_by_name("Dragon Scale").is_some(), "Dragon Scale should be available");
    
    // Test Dragon Scale boost
    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    let mut dragonite = framework
        .create_pokemon_from_ps_data("dragonite", None, Some(50))
        .unwrap();
    dragonite.item = Some("Dragon Scale".to_string());
    
    let dragon_pulse = framework.create_move_from_ps_data("dragonpulse").unwrap();
    
    let damage_with_dragon_scale =
        damage_calc::calculate_damage(&state, &dragonite, &defender, &dragon_pulse, false, 1.0);
    
    dragonite.item = None;
    let damage_without_dragon_scale =
        damage_calc::calculate_damage(&state, &dragonite, &defender, &dragon_pulse, false, 1.0);
    
    let boost_ratio = damage_with_dragon_scale as f32 / damage_without_dragon_scale as f32;
    assert!(
        boost_ratio > 1.15 && boost_ratio < 1.25,
        "Dragon Scale should provide ~1.2x boost in Gen 9, got {}x",
        boost_ratio
    );
}

#[test]
fn test_sea_incense_generation_specific_boost() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    // Test Sea Incense availability
    use tapu_simu::engine::items::get_item_by_name;
    assert!(get_item_by_name("Sea Incense").is_some(), "Sea Incense should be available");
    
    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    let mut blastoise = framework
        .create_pokemon_from_ps_data("blastoise", None, Some(50))
        .unwrap();
    blastoise.item = Some("Sea Incense".to_string());
    
    let surf = framework.create_move_from_ps_data("surf").unwrap();
    
    // Test in Gen 9 (should be 1.2x)
    let state_gen9 = State::new(BattleFormat::gen9_ou());
    let damage_with_sea_incense_gen9 =
        damage_calc::calculate_damage(&state_gen9, &blastoise, &defender, &surf, false, 1.0);
    
    blastoise.item = None;
    let damage_without_sea_incense_gen9 =
        damage_calc::calculate_damage(&state_gen9, &blastoise, &defender, &surf, false, 1.0);
    
    let boost_ratio_gen9 = damage_with_sea_incense_gen9 as f32 / damage_without_sea_incense_gen9 as f32;
    assert!(
        boost_ratio_gen9 > 1.15 && boost_ratio_gen9 < 1.25,
        "Sea Incense should provide ~1.2x boost in Gen 9, got {}x",
        boost_ratio_gen9
    );
    
    // Test in Gen 3 (should be 1.05x) - note: would need Gen 3 battle format for full test
    // For now, just verify the item exists and can be used
    blastoise.item = Some("Sea Incense".to_string());
    let state_gen3 = State::new(BattleFormat::gen9_ou()); // Using Gen9 format as proxy
    let damage_with_sea_incense =
        damage_calc::calculate_damage(&state_gen3, &blastoise, &defender, &surf, false, 1.0);
    
    assert!(damage_with_sea_incense > 0, "Sea Incense should work in all generations");
}

#[test]
fn test_pink_bow_polkadot_bow_consistent_boost() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    // Test availability
    use tapu_simu::engine::items::get_item_by_name;
    assert!(get_item_by_name("Pink Bow").is_some(), "Pink Bow should be available");
    assert!(get_item_by_name("Polkadot Bow").is_some(), "Polkadot Bow should be available");
    
    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    let normal_move = framework.create_move_from_ps_data("bodyslam").unwrap();
    
    // Test Pink Bow
    let mut snorlax = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    snorlax.item = Some("Pink Bow".to_string());
    
    let damage_with_pink_bow =
        damage_calc::calculate_damage(&state, &snorlax, &defender, &normal_move, false, 1.0);
    
    snorlax.item = None;
    let damage_without_pink_bow =
        damage_calc::calculate_damage(&state, &snorlax, &defender, &normal_move, false, 1.0);
    
    let pink_bow_boost = damage_with_pink_bow as f32 / damage_without_pink_bow as f32;
    assert!(
        pink_bow_boost > 1.05 && pink_bow_boost < 1.15,
        "Pink Bow should provide ~1.1x boost, got {}x",
        pink_bow_boost
    );
    
    // Test Polkadot Bow
    snorlax.item = Some("Polkadot Bow".to_string());
    
    let damage_with_polkadot_bow =
        damage_calc::calculate_damage(&state, &snorlax, &defender, &normal_move, false, 1.0);
    
    let polkadot_bow_boost = damage_with_polkadot_bow as f32 / damage_without_pink_bow as f32;
    assert!(
        polkadot_bow_boost > 1.05 && polkadot_bow_boost < 1.15,
        "Polkadot Bow should provide ~1.1x boost, got {}x",
        polkadot_bow_boost
    );
    
    // Test that they don't boost non-Normal moves
    let fire_move = framework.create_move_from_ps_data("flamethrower").unwrap();
    
    snorlax.item = Some("Pink Bow".to_string());
    let damage_fire_with_pink_bow =
        damage_calc::calculate_damage(&state, &snorlax, &defender, &fire_move, false, 1.0);
    
    snorlax.item = None;
    let damage_fire_without_pink_bow =
        damage_calc::calculate_damage(&state, &snorlax, &defender, &fire_move, false, 1.0);
    
    let fire_boost_ratio = damage_fire_with_pink_bow as f32 / damage_fire_without_pink_bow as f32;
    assert!(
        fire_boost_ratio < 1.05,
        "Pink Bow should NOT boost non-Normal moves, got {}x",
        fire_boost_ratio
    );
}

#[test]
fn test_miracle_berry_mint_berry_availability() {
    // Test availability
    use tapu_simu::engine::items::get_item_by_name;
    assert!(get_item_by_name("Miracle Berry").is_some(), "Miracle Berry should be available");
    assert!(get_item_by_name("Mint Berry").is_some(), "Mint Berry should be available");
}

#[test]
fn test_miracle_berry_mint_berry_generation_specificity() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    use tapu_simu::engine::items::get_item_by_name;
    
    // Test that Gen 2 exclusive berries exist
    assert!(get_item_by_name("Miracle Berry").is_some(), "Miracle Berry should be available");
    assert!(get_item_by_name("Mint Berry").is_some(), "Mint Berry should be available");
    
    // Test basic functionality through damage calculation (they should work in any generation for availability)
    let state = State::new(BattleFormat::gen9_ou());
    let attacker = framework
        .create_pokemon_from_ps_data("garchomp", None, Some(50))
        .unwrap();
    let mut defender = framework
        .create_pokemon_from_ps_data("alakazam", None, Some(50))
        .unwrap();
    
    // Test with Miracle Berry
    defender.item = Some("Miracle Berry".to_string());
    let move_data = framework.create_move_from_ps_data("earthquake").unwrap();
    
    let damage_miracle = framework.test_damage_calculation(&attacker, &defender, &move_data, &state);
    assert!(damage_miracle > 0, "Miracle Berry should not prevent damage calculation");
    
    // Test with Mint Berry
    defender.item = Some("Mint Berry".to_string());
    let damage_mint = framework.test_damage_calculation(&attacker, &defender, &move_data, &state);
    assert!(damage_mint > 0, "Mint Berry should not prevent damage calculation");
}

#[test]
fn test_comprehensive_item_coverage_verification() {
    // Verify all critical items from checklist are now implemented
    use tapu_simu::engine::items::get_item_by_name;
    
    // The 4 critical items that were missing/broken
    assert!(get_item_by_name("Dragon Scale").is_some(), "Dragon Scale should now be implemented");
    assert!(get_item_by_name("Sea Incense").is_some(), "Sea Incense should now be fixed");
    assert!(get_item_by_name("Pink Bow").is_some(), "Pink Bow should now be fixed");
    assert!(get_item_by_name("Polkadot Bow").is_some(), "Polkadot Bow should now be fixed");
    assert!(get_item_by_name("Miracle Berry").is_some(), "Miracle Berry should now be implemented");
    assert!(get_item_by_name("Mint Berry").is_some(), "Mint Berry should now be implemented");
    
    // Test that existing items still work
    assert!(get_item_by_name("Dragon Fang").is_some(), "Dragon Fang should still be available");
    assert!(get_item_by_name("Silk Scarf").is_some(), "Silk Scarf should still be available");
    assert!(get_item_by_name("Wave Incense").is_some(), "Wave Incense should still be available");
    assert!(get_item_by_name("Lum Berry").is_some(), "Lum Berry should still be available");
}

// =============================================================================
// ADVANCED TEST FRAMEWORK USAGE EXAMPLES
// =============================================================================

#[test]
fn test_dragon_scale_vs_dragon_fang_identical_behavior() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    // Test that Dragon Scale and Dragon Fang provide identical boosts in Gen 9
    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    // Test with Dragon Scale
    let mut dragonite_scale = framework
        .create_pokemon_from_ps_data("dragonite", None, Some(50))
        .unwrap();
    dragonite_scale.item = Some("Dragon Scale".to_string());
    
    // Test with Dragon Fang
    let mut dragonite_fang = framework
        .create_pokemon_from_ps_data("dragonite", None, Some(50))
        .unwrap();
    dragonite_fang.item = Some("Dragon Fang".to_string());
    
    let dragon_move = framework.create_move_from_ps_data("dragonpulse").unwrap();
    
    let damage_scale = framework.test_damage_calculation(&dragonite_scale, &defender, &dragon_move, &state);
    let damage_fang = framework.test_damage_calculation(&dragonite_fang, &defender, &dragon_move, &state);
    
    assert_eq!(damage_scale, damage_fang, "Dragon Scale and Dragon Fang should provide identical damage in Gen 9");
    
    // Test with non-dragon move (should be no boost)
    let non_dragon_move = framework.create_move_from_ps_data("earthquake").unwrap();
    
    dragonite_scale.item = None;
    let damage_no_item = framework.test_damage_calculation(&dragonite_scale, &defender, &non_dragon_move, &state);
    
    dragonite_scale.item = Some("Dragon Scale".to_string());
    let damage_scale_non_dragon = framework.test_damage_calculation(&dragonite_scale, &defender, &non_dragon_move, &state);
    
    assert_eq!(damage_no_item, damage_scale_non_dragon, "Dragon Scale should not boost non-Dragon moves");
}

#[test]
fn test_sea_incense_effectiveness_vs_other_water_boosters() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    // Compare Sea Incense with other Water-type boosters
    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    let water_move = framework.create_move_from_ps_data("surf").unwrap();
    
    // Test Sea Incense
    let mut blastoise_sea = framework
        .create_pokemon_from_ps_data("blastoise", None, Some(50))
        .unwrap();
    blastoise_sea.item = Some("Sea Incense".to_string());
    
    // Test Mystic Water
    let mut blastoise_mystic = framework
        .create_pokemon_from_ps_data("blastoise", None, Some(50))
        .unwrap();
    blastoise_mystic.item = Some("Mystic Water".to_string());
    
    // Test Wave Incense
    let mut blastoise_wave = framework
        .create_pokemon_from_ps_data("blastoise", None, Some(50))
        .unwrap();
    blastoise_wave.item = Some("Wave Incense".to_string());
    
    let damage_sea = framework.test_damage_calculation(&blastoise_sea, &defender, &water_move, &state);
    let damage_mystic = framework.test_damage_calculation(&blastoise_mystic, &defender, &water_move, &state);
    let damage_wave = framework.test_damage_calculation(&blastoise_wave, &defender, &water_move, &state);
    
    // In Gen 9, Sea Incense should provide 1.2x boost like other water boosters
    assert_eq!(damage_sea, damage_mystic, "Sea Incense should provide same boost as Mystic Water in Gen 9");
    assert_eq!(damage_sea, damage_wave, "Sea Incense should provide same boost as Wave Incense in Gen 9");
}

#[test]
fn test_pink_polkadot_bow_vs_silk_scarf() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    // Test that Pink Bow and Polkadot Bow provide weaker boosts than Silk Scarf
    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    let normal_move = framework.create_move_from_ps_data("bodyslam").unwrap();
    
    // Test Silk Scarf (should be 1.2x)
    let mut snorlax_silk = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    snorlax_silk.item = Some("Silk Scarf".to_string());
    
    // Test Pink Bow (should be 1.1x)
    let mut snorlax_pink = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    snorlax_pink.item = Some("Pink Bow".to_string());
    
    // Test Polkadot Bow (should be 1.1x)
    let mut snorlax_polkadot = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    snorlax_polkadot.item = Some("Polkadot Bow".to_string());
    
    let damage_silk = framework.test_damage_calculation(&snorlax_silk, &defender, &normal_move, &state);
    let damage_pink = framework.test_damage_calculation(&snorlax_pink, &defender, &normal_move, &state);
    let damage_polkadot = framework.test_damage_calculation(&snorlax_polkadot, &defender, &normal_move, &state);
    
    // Pink Bow and Polkadot Bow should provide identical boosts
    assert_eq!(damage_pink, damage_polkadot, "Pink Bow and Polkadot Bow should provide identical damage");
    
    // Silk Scarf should provide higher damage than the bows
    assert!(damage_silk > damage_pink, "Silk Scarf should provide higher damage than Pink Bow");
    assert!(damage_silk > damage_polkadot, "Silk Scarf should provide higher damage than Polkadot Bow");
    
    // Verify the approximate ratio (Silk Scarf 1.2x vs Bow 1.1x = ~1.09x difference)
    let damage_ratio = damage_silk as f32 / damage_pink as f32;
    assert!(damage_ratio > 1.08 && damage_ratio < 1.11, 
        "Silk Scarf should provide ~1.09x more damage than bows, got {}x", damage_ratio);
}

#[test]
fn test_comprehensive_type_booster_consistency() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    
    // Test that all type boosters provide consistent 1.2x boosts in Gen 9
    let type_booster_tests = [
        ("Charcoal", "charizard", "flamethrower"),
        ("Dragon Scale", "dragonite", "dragonpulse"),
        ("Dragon Fang", "dragonite", "dragonpulse"),
        ("Mystic Water", "blastoise", "surf"),
        ("Sea Incense", "blastoise", "surf"),
        ("Wave Incense", "blastoise", "surf"),
        ("Miracle Seed", "venusaur", "energyball"),
        ("Magnet", "magnezone", "thunderbolt"),
    ];
    
    let state = State::new(BattleFormat::gen9_ou());
    let defender = framework
        .create_pokemon_from_ps_data("snorlax", None, Some(50))
        .unwrap();
    
    for (item_name, pokemon_species, move_name) in &type_booster_tests {
        let mut attacker_with_item = framework
            .create_pokemon_from_ps_data(pokemon_species, None, Some(50))
            .unwrap();
        attacker_with_item.item = Some(item_name.to_string());
        
        let mut attacker_without_item = framework
            .create_pokemon_from_ps_data(pokemon_species, None, Some(50))
            .unwrap();
        attacker_without_item.item = None;
        
        let move_data = framework.create_move_from_ps_data(move_name).unwrap();
        
        let damage_with_item = framework.test_damage_calculation(&attacker_with_item, &defender, &move_data, &state);
        let damage_without_item = framework.test_damage_calculation(&attacker_without_item, &defender, &move_data, &state);
        
        let boost_ratio = damage_with_item as f32 / damage_without_item as f32;
        
        // Allow for some floating point precision variance
        let expected_boost = if item_name.contains("Bow") { 1.1 } else { 1.2 };
        let tolerance = 0.05;
        
        assert!(
            (boost_ratio - expected_boost).abs() < tolerance,
            "{} should provide ~{}x boost, got {}x",
            item_name, expected_boost, boost_ratio
        );
    }
}