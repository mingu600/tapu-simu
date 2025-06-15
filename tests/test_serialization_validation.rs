//! Serialization validation tests to ensure state consistency
//! 
//! Tests round-trip serialization/deserialization for all major data structures

use tapu_simu::battle_state::*;
use tapu_simu::pokemon::*;
use tapu_simu::side::*;
use tapu_simu::action_queue::*;
use tapu_simu::types::*;
use tapu_simu::format::{BattleFormat, FormatRules};
use tapu_simu::prng::PRNGState;
use std::collections::HashMap;

#[test]
fn test_battle_state_serialization_roundtrip() {
    let state = create_test_battle_state();
    
    // Binary serialization round-trip
    let binary_data = state.to_bytes().expect("Binary serialization failed");
    let deserialized_binary = BattleState::from_bytes(&binary_data)
        .expect("Binary deserialization failed");
    
    assert_eq!(state.turn, deserialized_binary.turn);
    assert_eq!(state.ended, deserialized_binary.ended);
    assert_eq!(state.winner, deserialized_binary.winner);
    assert_eq!(state.sides.len(), deserialized_binary.sides.len());
    
    // JSON serialization round-trip
    let json_data = state.to_json().expect("JSON serialization failed");
    let deserialized_json = BattleState::from_json(&json_data)
        .expect("JSON deserialization failed");
    
    assert_eq!(state.turn, deserialized_json.turn);
    assert_eq!(state.ended, deserialized_json.ended);
    assert_eq!(state.winner, deserialized_json.winner);
    
    // Verify that serialized size is reasonable
    assert!(binary_data.len() > 1000, "Binary serialization seems too small: {} bytes", binary_data.len());
    assert!(binary_data.len() < 10000, "Binary serialization seems too large: {} bytes", binary_data.len());
    
    println!("Battle state serialization validation passed: {} bytes binary, {} chars JSON", 
             binary_data.len(), json_data.len());
}

#[test]
fn test_pokemon_serialization_roundtrip() {
    let pokemon = create_test_pokemon();
    
    // Test JSON round-trip
    let json = serde_json::to_string(&pokemon).expect("Pokemon JSON serialization failed");
    let deserialized: Pokemon = serde_json::from_str(&json)
        .expect("Pokemon JSON deserialization failed");
    
    assert_eq!(pokemon.species.id, deserialized.species.id);
    assert_eq!(pokemon.hp, deserialized.hp);
    assert_eq!(pokemon.max_hp, deserialized.max_hp);
    assert_eq!(pokemon.level, deserialized.level);
    assert_eq!(pokemon.fainted, deserialized.fainted);
    assert_eq!(pokemon.trapped, deserialized.trapped);
    assert_eq!(pokemon.transformed, deserialized.transformed);
    
    // Test binary round-trip
    let binary = bincode::serialize(&pokemon).expect("Pokemon binary serialization failed");
    let deserialized_binary: Pokemon = bincode::deserialize(&binary)
        .expect("Pokemon binary deserialization failed");
    
    assert_eq!(pokemon.species.id, deserialized_binary.species.id);
    assert_eq!(pokemon.illusion.is_some(), deserialized_binary.illusion.is_some());
    assert_eq!(pokemon.sub_fainted, deserialized_binary.sub_fainted);
    
    println!("Pokemon serialization validation passed: {} bytes binary, {} chars JSON", 
             binary.len(), json.len());
}

#[test]
fn test_action_queue_serialization_roundtrip() {
    let mut queue = ActionQueue::new();
    
    // Add various action types
    queue.add_field_action(FieldActionType::Start, 1);
    queue.add_field_action(FieldActionType::BeforeTurn, 2);
    queue.add_field_action(FieldActionType::Residual, 200);
    
    // Test JSON round-trip
    let json = serde_json::to_string(&queue).expect("ActionQueue JSON serialization failed");
    let deserialized: ActionQueue = serde_json::from_str(&json)
        .expect("ActionQueue JSON deserialization failed");
    
    assert_eq!(queue.len(), deserialized.len());
    assert_eq!(queue.is_empty(), deserialized.is_empty());
    
    // Test binary round-trip
    let binary = bincode::serialize(&queue).expect("ActionQueue binary serialization failed");
    let deserialized_binary: ActionQueue = bincode::deserialize(&binary)
        .expect("ActionQueue binary deserialization failed");
    
    assert_eq!(queue.len(), deserialized_binary.len());
    
    println!("ActionQueue serialization validation passed: {} bytes binary, {} chars JSON", 
             binary.len(), json.len());
}

#[test]
fn test_side_serialization_roundtrip() {
    let side = create_test_side();
    
    // Test JSON round-trip
    let json = serde_json::to_string(&side).expect("Side JSON serialization failed");
    let deserialized: Side = serde_json::from_str(&json)
        .expect("Side JSON deserialization failed");
    
    assert_eq!(side.id, deserialized.id);
    assert_eq!(side.name, deserialized.name);
    assert_eq!(side.pokemon.len(), deserialized.pokemon.len());
    assert_eq!(side.active.len(), deserialized.active.len());
    assert_eq!(side.total_fainted, deserialized.total_fainted);
    
    // Test binary round-trip
    let binary = bincode::serialize(&side).expect("Side binary serialization failed");
    let deserialized_binary: Side = bincode::deserialize(&binary)
        .expect("Side binary deserialization failed");
    
    assert_eq!(side.id, deserialized_binary.id);
    assert_eq!(side.z_move_used, deserialized_binary.z_move_used);
    assert_eq!(side.mega_used, deserialized_binary.mega_used);
    assert_eq!(side.tera_used, deserialized_binary.tera_used);
    
    println!("Side serialization validation passed: {} bytes binary, {} chars JSON", 
             binary.len(), json.len());
}

#[test]
fn test_complex_pokemon_states() {
    let mut pokemon = create_test_pokemon();
    
    // Set up complex state
    pokemon.transformed = true;
    pokemon.sub_fainted = Some(true);
    pokemon.added_type = Some(Type::Psychic);
    pokemon.move_last_turn_result = Some(MoveResult::Success);
    pokemon.switch_flag = SwitchFlag::Effect("uturn".to_string());
    
    // Add illusion
    let illusion_pokemon = create_test_pokemon();
    pokemon.set_illusion(Some(illusion_pokemon));
    
    // Add volatiles
    pokemon.volatiles.insert("confusion".to_string(), VolatileStatus {
        id: "confusion".to_string(),
        duration: Some(3),
        data: HashMap::new(),
    });
    
    // Test serialization
    let json = serde_json::to_string(&pokemon).expect("Complex Pokemon JSON serialization failed");
    let deserialized: Pokemon = serde_json::from_str(&json)
        .expect("Complex Pokemon JSON deserialization failed");
    
    assert_eq!(pokemon.transformed, deserialized.transformed);
    assert_eq!(pokemon.sub_fainted, deserialized.sub_fainted);
    assert_eq!(pokemon.added_type, deserialized.added_type);
    assert_eq!(pokemon.move_last_turn_result, deserialized.move_last_turn_result);
    assert!(deserialized.has_illusion());
    assert_eq!(pokemon.volatiles.len(), deserialized.volatiles.len());
    
    println!("Complex Pokemon state serialization validation passed");
}

#[test]
fn test_state_consistency_after_modifications() {
    let mut state = create_test_battle_state();
    
    // Make some modifications
    state.turn = 10;
    state.sides[0].pokemon[0].take_damage(50);
    state.sides[0].pokemon[0].boosts.attack = 2;
    state.sides[0].total_fainted = 1;
    
    // Serialize and deserialize
    let binary_data = state.to_bytes().expect("Modified state serialization failed");
    let deserialized = BattleState::from_bytes(&binary_data)
        .expect("Modified state deserialization failed");
    
    // Verify modifications are preserved
    assert_eq!(state.turn, deserialized.turn);
    assert_eq!(state.sides[0].pokemon[0].hp, deserialized.sides[0].pokemon[0].hp);
    assert_eq!(state.sides[0].pokemon[0].boosts.attack, deserialized.sides[0].pokemon[0].boosts.attack);
    assert_eq!(state.sides[0].total_fainted, deserialized.sides[0].total_fainted);
    
    println!("State consistency validation passed after modifications");
}

// Helper functions
fn create_test_battle_state() -> BattleState {
    let sides = vec![create_test_side(), create_test_side()];
    
    BattleState {
        turn: 1,
        sides,
        field: FieldState::default(),
        queue: ActionQueue::new(),
        random: PRNGState::from_seed("sodium,deadbeef").expect("Failed to create PRNG"),
        format: BattleFormat::Singles,
        rules: FormatRules::default(),
        ended: false,
        winner: None,
        log: Vec::new(),
    }
}

fn create_test_side() -> Side {
    let team = vec![create_test_pokemon()];
    Side::new(SideId::P1, "Test Player".to_string(), team, &BattleFormat::Singles)
        .expect("Failed to create test side")
}

fn create_test_pokemon() -> Pokemon {
    let species = SpeciesData {
        id: "pikachu".to_string(),
        name: "Pikachu".to_string(),
        types: [Type::Electric, Type::Electric],
        base_stats: StatsTable {
            hp: 35,
            attack: 55,
            defense: 40,
            special_attack: 50,
            special_defense: 50,
            speed: 90,
        },
        abilities: vec!["static".to_string()],
        height: 0.4,
        weight: 6.0,
        gender_ratio: GenderRatio::Ratio { male: 0.5, female: 0.5 },
    };
    
    let moves = [
        MoveData {
            id: "tackle".to_string(),
            name: "Tackle".to_string(),
            type_: Type::Normal,
            category: MoveCategory::Physical,
            base_power: 40,
            accuracy: Some(100),
            pp: 35,
            target: MoveTarget::Normal,
            priority: 0,
            flags: MoveFlags::default(),
            secondary_effect: None,
            crit_ratio: 1,
            multihit: None,
            drain: None,
            recoil: None,
        },
        MoveData {
            id: "thundershock".to_string(),
            name: "Thunder Shock".to_string(),
            type_: Type::Electric,
            category: MoveCategory::Special,
            base_power: 40,
            accuracy: Some(100),
            pp: 30,
            target: MoveTarget::Normal,
            priority: 0,
            flags: MoveFlags::default(),
            secondary_effect: None,
            crit_ratio: 1,
            multihit: None,
            drain: None,
            recoil: None,
        },
        MoveData {
            id: "growl".to_string(),
            name: "Growl".to_string(),
            type_: Type::Normal,
            category: MoveCategory::Status,
            base_power: 0,
            accuracy: Some(100),
            pp: 40,
            target: MoveTarget::AllAdjacentFoes,
            priority: 0,
            flags: MoveFlags::default(),
            secondary_effect: None,
            crit_ratio: 1,
            multihit: None,
            drain: None,
            recoil: None,
        },
        MoveData {
            id: "tailwhip".to_string(),
            name: "Tail Whip".to_string(),
            type_: Type::Normal,
            category: MoveCategory::Status,
            base_power: 0,
            accuracy: Some(100),
            pp: 30,
            target: MoveTarget::AllAdjacentFoes,
            priority: 0,
            flags: MoveFlags::default(),
            secondary_effect: None,
            crit_ratio: 1,
            multihit: None,
            drain: None,
            recoil: None,
        },
    ];
    
    let ability = AbilityData {
        id: "static".to_string(),
        name: "Static".to_string(),
        description: "Contact may paralyze attacker".to_string(),
        event_handlers: tapu_simu::events::EventHandlerRegistry::default(),
    };
    
    Pokemon::new(
        species,
        50,
        moves,
        ability,
        None,
        Nature::Hardy,
        StatsTable { hp: 31, attack: 31, defense: 31, special_attack: 31, special_defense: 31, speed: 31 },
        StatsTable::default(),
        Gender::Male,
    )
}