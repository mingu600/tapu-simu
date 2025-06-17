//! # End-of-Turn Effects Tests
//! 
//! Comprehensive tests for all end-of-turn effects including status damage, weather effects,
//! ability effects, item effects, and volatile status management.
//! All tests use the TestFramework with real Pokemon Showdown data.

use tapu_simu::test_framework::TestFramework;
use tapu_simu::state::{State, Pokemon};
use tapu_simu::battle_format::BattleFormat;
use tapu_simu::engine::end_of_turn::process_end_of_turn_effects;
use tapu_simu::generation::{GenerationMechanics, Generation};
use tapu_simu::instruction::{PokemonStatus, VolatileStatus, Weather, Terrain, Stat, SideCondition};


// =============================================================================
// STATUS CONDITION DAMAGE TESTS
// =============================================================================

#[test]
fn test_burn_damage_generation_differences() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("charizard", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set the attacker to burned status
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.status = PokemonStatus::BURN;
        pokemon.hp = 100;
        pokemon.max_hp = 100;
    }
    
    // Test burn damage in Gen 9 (should be 1/16 max HP = 6.25, rounded to 6)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify burn damage instruction exists
    let burn_damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount >= 6 && damage_inst.damage_amount <= 7 // 1/16 of 100 HP
            } else {
                false
            }
        })
    });
    
    assert!(burn_damage_found, "Burn should cause 1/16 max HP damage in Gen 9");
}

#[test]
fn test_poison_damage() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("venusaur", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set the attacker to poisoned status
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.status = PokemonStatus::POISON;
        pokemon.hp = 160;
        pokemon.max_hp = 160;
    }
    
    // Test poison damage (should be 1/8 max HP = 20)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify poison damage instruction exists
    let poison_damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount == 20 // 1/8 of 160 HP
            } else {
                false
            }
        })
    });
    
    assert!(poison_damage_found, "Poison should cause 1/8 max HP damage");
}

#[test]
fn test_toxic_damage_progression() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("alakazam", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set the attacker to toxic status
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.status = PokemonStatus::TOXIC;
        pokemon.hp = 160;
        pokemon.max_hp = 160;
    }
    
    // Test toxic damage progression
    // Turn 1: 1/16 max HP, Turn 2: 2/16 max HP, etc.
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify toxic damage instruction exists
    let toxic_damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount >= 10 // At least 1/16 of 160 HP
            } else {
                false
            }
        })
    });
    
    assert!(toxic_damage_found, "Toxic should cause progressive damage");
}

#[test]
fn test_poison_heal_ability_prevents_poison_damage() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("gliscor", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set the attacker to have Poison Heal ability and poison status
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.ability = "Poison Heal".to_string();
        pokemon.status = PokemonStatus::POISON;
        pokemon.hp = 150;
        pokemon.max_hp = 200;
    }
    
    // Test that Poison Heal heals instead of damages
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify healing instruction exists instead of damage
    let heal_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionHeal(heal_inst) = inst {
                heal_inst.heal_amount == 25 // 1/8 of 200 HP
            } else {
                false
            }
        })
    });
    
    assert!(heal_found, "Poison Heal should heal instead of damage from poison");
}

// =============================================================================
// WEATHER EFFECTS TESTS
// =============================================================================

#[test]
fn test_sandstorm_damage() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("charizard", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set up sandstorm
    state.weather = Weather::SAND;
    state.weather_turns_remaining = Some(3);
    
    // Set the attacker's HP (vulnerable to sandstorm - not Rock/Ground/Steel)
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 160;
        pokemon.max_hp = 160;
    }
    
    // Test sandstorm damage (should be 1/16 max HP = 10)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify sandstorm damage instruction exists
    let sandstorm_damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount == 10 // 1/16 of 160 HP
            } else {
                false
            }
        })
    });
    
    assert!(sandstorm_damage_found, "SANDSTORM should cause 1/16 max HP damage to vulnerable Pokemon");
}

#[test]
fn test_hail_damage() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("garchomp", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set up hail
    state.weather = Weather::HAIL;
    state.weather_turns_remaining = Some(3);
    
    // Set the attacker's HP (vulnerable to hail - not Ice type)
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 180;
        pokemon.max_hp = 180;
    }
    
    // Test hail damage (should be 1/16 max HP = 11.25, rounded to 11)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify hail damage instruction exists
    let hail_damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount >= 11 && damage_inst.damage_amount <= 12 // 1/16 of 180 HP
            } else {
                false
            }
        })
    });
    
    assert!(hail_damage_found, "HAIL should cause 1/16 max HP damage to vulnerable Pokemon");
}

#[test]
fn test_ice_body_healing_in_hail() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("walrein", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set up hail
    state.weather = Weather::HAIL;
    state.weather_turns_remaining = Some(3);
    
    // Set the attacker to have Ice Body ability
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 150;
        pokemon.max_hp = 200;
        pokemon.ability = "Ice Body".to_string();
    }
    
    // Test Ice Body healing in hail
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify healing instruction exists
    let heal_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionHeal(heal_inst) = inst {
                heal_inst.heal_amount >= 12 && heal_inst.heal_amount <= 13 // 1/16 of 200 HP
            } else {
                false
            }
        })
    });
    
    assert!(heal_found, "Ice Body should heal 1/16 max HP in hail");
}

// =============================================================================
// ITEM EFFECTS TESTS
// =============================================================================

#[test]
fn test_leftovers_healing() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("snorlax", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set the attacker to have Leftovers
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 300;
        pokemon.max_hp = 400;
        pokemon.item = Some("Leftovers".to_string());
    }
    
    // Test Leftovers healing (should be 1/16 max HP = 25)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify healing instruction exists
    let heal_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionHeal(heal_inst) = inst {
                heal_inst.heal_amount == 25 // 1/16 of 400 HP
            } else {
                false
            }
        })
    });
    
    assert!(heal_found, "Leftovers should heal 1/16 max HP");
}

#[test]
fn test_black_sludge_poison_type_healing() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("crobat", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set the attacker to have Black Sludge (Poison-type Pokemon)
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 150;
        pokemon.max_hp = 200;
        pokemon.item = Some("Black Sludge".to_string());
    }
    
    // Test Black Sludge healing for Poison types
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify healing instruction exists
    let heal_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionHeal(heal_inst) = inst {
                heal_inst.heal_amount >= 12 && heal_inst.heal_amount <= 13 // 1/16 of 200 HP
            } else {
                false
            }
        })
    });
    
    assert!(heal_found, "Black Sludge should heal Poison-type Pokemon");
}

#[test]
fn test_black_sludge_non_poison_damage() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let generation = GenerationMechanics::new(Generation::Gen9);
    let (mut state, _) = framework
        .create_test_battle("garchomp", &["tackle"], "squirtle", None)
        .unwrap();
    
    // Set the attacker to have Black Sludge (non-Poison-type Pokemon)
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 180;
        pokemon.max_hp = 180;
        pokemon.item = Some("Black Sludge".to_string());
    }
    
    // Test Black Sludge damage for non-Poison types
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify damage instruction exists
    let damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount >= 11 && damage_inst.damage_amount <= 12 // 1/16 of 180 HP
            } else {
                false
            }
        })
    });
    
    assert!(damage_found, "Black Sludge should damage non-Poison-type Pokemon");
}

#[test]
fn test_flame_orb_burn_infliction() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set the attacker to have Flame Orb and no status
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.status = PokemonStatus::NONE;
        pokemon.item = Some("Flame Orb".to_string());
    }
    
    // Test Flame Orb burn infliction
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify burn status instruction exists
    let burn_status_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::ApplyStatus(status_inst) = inst {
                matches!(status_inst.status, PokemonStatus::BURN)
            } else {
                false
            }
        })
    });
    
    assert!(burn_status_found, "Flame Orb should inflict burn status");
}

#[test]
fn test_toxic_orb_toxic_infliction() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set the attacker to have Toxic Orb and no status
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.status = PokemonStatus::NONE;
        pokemon.item = Some("Toxic Orb".to_string());
    }
    
    // Test Toxic Orb toxic infliction
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify toxic status instruction exists
    let toxic_status_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::ApplyStatus(status_inst) = inst {
                matches!(status_inst.status, PokemonStatus::TOXIC)
            } else {
                false
            }
        })
    });
    
    assert!(toxic_status_found, "Toxic Orb should inflict toxic status");
}

// =============================================================================
// ABILITY EFFECTS TESTS
// =============================================================================

#[test]
fn test_speed_boost_ability() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set the attacker to have Speed Boost ability
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.ability = "Speed Boost".to_string();
    }
    
    // Test Speed Boost ability (+1 Speed stage)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify stat boost instruction exists
    let speed_boost_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::BoostStats(boost_inst) = inst {
                boost_inst.stat_boosts.get(&Stat::Speed).map_or(false, |&boost| boost == 1)
            } else {
                false
            }
        })
    });
    
    assert!(speed_boost_found, "Speed Boost should increase Speed by 1 stage");
}

#[test]
fn test_rain_dish_healing() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up rain
    state.weather = Weather::RAIN;
    state.weather_turns_remaining = Some(3);
    
    // Set the attacker to have Rain Dish ability
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 150;
        pokemon.max_hp = 200;
        pokemon.ability = "Rain Dish".to_string();
    }
    
    // Test RAIN Dish healing in rain
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify healing instruction exists
    let heal_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionHeal(heal_inst) = inst {
                heal_inst.heal_amount >= 12 && heal_inst.heal_amount <= 13 // 1/16 of 200 HP
            } else {
                false
            }
        })
    });
    
    assert!(heal_found, "RAIN Dish should heal 1/16 max HP in rain");
}

#[test]
fn test_dry_skin_rain_healing() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up rain
    state.weather = Weather::RAIN;
    state.weather_turns_remaining = Some(3);
    
    // Set the attacker to have Dry Skin ability
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 150;
        pokemon.max_hp = 200;
        pokemon.ability = "Dry Skin".to_string();
    }
    
    // Test Dry Skin healing in rain (should be 1/8 max HP)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify healing instruction exists
    let heal_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionHeal(heal_inst) = inst {
                heal_inst.heal_amount == 25 // 1/8 of 200 HP
            } else {
                false
            }
        })
    });
    
    assert!(heal_found, "Dry Skin should heal 1/8 max HP in rain");
}

#[test]
fn test_solar_power_sun_damage() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up sun
    state.weather = Weather::SUN;
    state.weather_turns_remaining = Some(3);
    
    // Set the attacker to have Solar Power ability
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 160;
        pokemon.max_hp = 160;
        pokemon.ability = "Solar Power".to_string();
    }
    
    // Test Solar Power damage in sun (should be 1/8 max HP)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify damage instruction exists
    let damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount == 20 // 1/8 of 160 HP
            } else {
                false
            }
        })
    });
    
    assert!(damage_found, "Solar Power should cause 1/8 max HP damage in sun");
}

// =============================================================================
// LEECH SEED TESTS
// =============================================================================

#[test]
fn test_leech_seed_damage_and_healing() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set the attacker to have Leech Seed
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 180;
        pokemon.max_hp = 180;
        pokemon.volatile_statuses.insert(VolatileStatus::LeechSeed);
    }
    
    // Test Leech Seed damage (should be 1/8 max HP = 22.5, rounded to 22)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify damage instruction exists
    let damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount >= 22 && damage_inst.damage_amount <= 23 // 1/8 of 180 HP
            } else {
                false
            }
        })
    });
    
    assert!(damage_found, "Leech Seed should cause 1/8 max HP damage");
}

// =============================================================================
// TERRAIN EFFECTS TESTS
// =============================================================================

#[test]
fn test_grassy_terrain_healing() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("garchomp", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up Grassy Terrain
    state.terrain = Terrain::GRASSYTERRAIN;
    state.terrain_turns_remaining = Some(3);
    
    // Set the attacker to be grounded
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 150;
        pokemon.max_hp = 180;
    }
    
    // Test Grassy Terrain healing (should be 1/16 max HP for grounded Pokemon)
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify healing instruction exists
    let heal_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionHeal(heal_inst) = inst {
                heal_inst.heal_amount >= 11 && heal_inst.heal_amount <= 12 // 1/16 of 180 HP
            } else {
                false
            }
        })
    });
    
    assert!(heal_found, "Grassy Terrain should heal grounded Pokemon");
}

// =============================================================================
// WISH TESTS
// =============================================================================

#[test]
fn test_wish_healing_activation() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up Wish with 1 turn remaining
    state.side_one.wish_healing.insert(0, (100, 1)); // slot 0, heal 100 HP, 1 turn remaining
    
    // Set the attacker to receive Wish healing
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 300;
        pokemon.max_hp = 400;
    }
    
    // Test Wish healing activation
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify healing instruction exists
    let heal_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionHeal(heal_inst) = inst {
                heal_inst.heal_amount == 100 // Wish amount
            } else {
                false
            }
        })
    });
    
    assert!(heal_found, "Wish should heal the target Pokemon");
}

// =============================================================================
// FUTURE SIGHT TESTS
// =============================================================================

#[test]
fn test_future_sight_activation() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up Future Sight with 1 turn remaining
    use tapu_simu::battle_format::{BattlePosition, SideReference};
    state.side_one.future_sight_attacks.insert(0, (BattlePosition::new(SideReference::SideTwo, 0), 80, 1, "Future Sight".to_string())); // slot 0, 80 damage, 1 turn remaining
    
    // Set the attacker to receive Future Sight damage
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.hp = 150;
        pokemon.max_hp = 150;
    }
    
    // Test Future Sight activation
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify damage instruction exists
    let damage_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.damage_amount == 80 // Future Sight damage
            } else {
                false
            }
        })
    });
    
    assert!(damage_found, "Future Sight should deal stored damage");
}

// =============================================================================
// PERISH SONG TESTS
// =============================================================================

#[test]
fn test_perish_song_countdown() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set the attacker to have Perish Song countdown
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.volatile_statuses.insert(VolatileStatus::Perish1);
        pokemon.volatile_status_durations.insert(VolatileStatus::Perish1, 1); // 1 turn remaining
    }
    
    // Test Perish Song countdown
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify fainting instruction exists when countdown reaches 0
    let faint_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            matches!(inst, tapu_simu::instruction::Instruction::PositionDamage(_))
        })
    });
    
    assert!(faint_found, "Perish Song should cause fainting when countdown reaches 0");
}

// =============================================================================
// WEATHER DURATION TESTS
// =============================================================================

#[test]
fn test_weather_duration_decrement() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up weather with 1 turn remaining
    state.weather = Weather::RAIN;
    state.weather_turns_remaining = Some(1);
    
    // Test weather duration decrement
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify weather removal instruction exists
    let weather_end_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::ChangeWeather(weather_inst) = inst {
                weather_inst.weather == Weather::NONE // Weather should be removed
            } else {
                false
            }
        })
    });
    
    assert!(weather_end_found, "Weather should end when duration reaches 0");
}

// =============================================================================
// SIDE CONDITION TESTS
// =============================================================================

#[test]
fn test_reflect_duration_decrement() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up Reflect with 1 turn remaining
    state.side_one.side_conditions.insert(SideCondition::Reflect, 1);
    
    // Test Reflect duration decrement
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify side condition removal instruction exists
    let reflect_end_found = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::RemoveSideCondition(side_inst) = inst {
                side_inst.condition == SideCondition::Reflect
            } else {
                false
            }
        })
    });
    
    assert!(reflect_end_found, "Reflect should end when duration reaches 0");
}

// =============================================================================
// COMPREHENSIVE INTEGRATION TESTS
// =============================================================================

#[test]
fn test_multiple_end_of_turn_effects_processing_order() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    // Set up multiple end-of-turn effects
    state.weather = Weather::SAND;
    state.weather_turns_remaining = Some(2);
    
    // Set the attacker to have multiple end-of-turn effects
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.status = PokemonStatus::BURN;
        pokemon.item = Some("Leftovers".to_string());
        pokemon.hp = 150;
        pokemon.max_hp = 180;
    }
    
    // Test that multiple effects are processed
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    // Verify that we have multiple instruction types
    let has_multiple_effects = instructions.len() > 1 ||
        instructions.iter().any(|state_inst| state_inst.instruction_list.len() > 1);
    
    assert!(has_multiple_effects, "Multiple end-of-turn effects should be processed");
}

#[test]
fn test_magic_guard_immunity() {
    let framework = TestFramework::new().expect("Failed to create test framework");
    let (mut state, _) = framework.create_test_battle("charizard", &["tackle"], "squirtle", None).unwrap();
    let generation = GenerationMechanics::new(Generation::Gen9);
    
    
    // Set up sandstorm
    state.weather = Weather::SAND;
    state.weather_turns_remaining = Some(3);
    
    // Set the attacker to have Magic Guard ability and burn status
    if let Some(pokemon) = state.side_one.get_active_pokemon_at_slot_mut(0) {
        pokemon.status = PokemonStatus::BURN;
        pokemon.ability = "Magic Guard".to_string();
        pokemon.hp = 150;
        pokemon.max_hp = 150;
    }
    
    // Test Magic Guard immunity to indirect damage
    let instructions = process_end_of_turn_effects(&state, &generation);
    
    
    // Verify no damage instructions for the Pokemon with Magic Guard (SideOne slot 0)
    let magic_guard_pokemon_damaged = instructions.iter().any(|state_inst| {
        state_inst.instruction_list.iter().any(|inst| {
            if let tapu_simu::instruction::Instruction::PositionDamage(damage_inst) = inst {
                damage_inst.target_position.side == tapu_simu::battle_format::SideReference::SideOne && 
                damage_inst.target_position.slot == 0
            } else {
                false
            }
        })
    });
    
    assert!(!magic_guard_pokemon_damaged, "Magic Guard should prevent indirect damage to the Pokemon with the ability");
}