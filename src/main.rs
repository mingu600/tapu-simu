//! Tapu Simu CLI interface

use tapu_simu::{Battle, BattleState, format::*, side::*, pokemon::*, types::*, dex::ShowdownDex, errors::*};

fn main() -> BattleResult<()> {
    println!("Tapu Simu v0.2.0 - Pokemon Showdown Rust Port");
    println!("High-performance battle simulator for AI/RL research");
    
    // Create a simple test battle
    let battle_state = create_test_battle()?;
    let dex = Box::new(ShowdownDex {});
    let mut battle = Battle::new(battle_state, dex);
    
    println!("\nBattle created successfully!");
    println!("Turn: {}", battle.state().turn);
    println!("Format: {:?}", battle.state().format);
    println!("Sides: {}", battle.state().sides.len());
    
    // Test serialization
    let serialized = battle.serialize_state()?;
    println!("State serialized to {} bytes", serialized.len());
    
    let json = battle.state().to_json()?;
    println!("JSON size: {} characters", json.len());
    
    println!("\nFoundation successfully implemented!");
    println!("Ready for Phase 2: Action System");
    
    Ok(())
}

fn create_test_battle() -> BattleResult<BattleState> {
    let format = BattleFormat::Singles;
    let rules = FormatRules::default();
    
    // Create test Pokemon
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
    
    let tackle = MoveData {
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
    };
    
    let moves = [tackle.clone(), tackle.clone(), tackle.clone(), tackle];
    
    let ability = AbilityData {
        id: "static".to_string(),
        name: "Static".to_string(),
        description: "Contact may paralyze attacker".to_string(),
    };
    
    let pokemon1 = Pokemon::new(
        species.clone(),
        50,
        moves.clone(),
        ability.clone(),
        None,
        Nature::Hardy,
        StatsTable { hp: 31, attack: 31, defense: 31, special_attack: 31, special_defense: 31, speed: 31 },
        StatsTable::default(),
        Gender::Male,
    );
    
    let pokemon2 = Pokemon::new(
        species,
        50,
        moves,
        ability,
        None,
        Nature::Hardy,
        StatsTable { hp: 31, attack: 31, defense: 31, special_attack: 31, special_defense: 31, speed: 31 },
        StatsTable::default(),
        Gender::Female,
    );
    
    let side1 = Side::new(SideId::P1, "Player 1".to_string(), vec![pokemon1], &format)?;
    let side2 = Side::new(SideId::P2, "Player 2".to_string(), vec![pokemon2], &format)?;
    
    BattleState::new(format, rules, vec![side1, side2], Some("sodium,1234567890abcdef".to_string()))
}