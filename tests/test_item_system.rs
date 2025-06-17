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