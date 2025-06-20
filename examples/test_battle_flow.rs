use tapu_simu::{State, BattleFormat};
use tapu_simu::data::{RandomPokemonSet, random_team_loader::RandomStats};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the exact Annihilape from the battle log
    let annihilape_set = RandomPokemonSet {
        name: "Annihilape".to_string(),
        species: "Annihilape".to_string(),
        level: 76,
        gender: Some("M".to_string()),
        shiny: Some(false),
        ability: Some("Defiant".to_string()),
        item: Some("Chesto Berry".to_string()),
        moves: vec![
            "Gunk Shot".to_string(),
            "Rage Fist".to_string(),
            "Rest".to_string(),
            "Taunt".to_string(),
        ],
        nature: Some("Hardy".to_string()),
        evs: Some(RandomStats {
            hp: Some(85),
            atk: Some(85),
            def: Some(85),
            spa: Some(85),
            spd: Some(85),
            spe: Some(85),
        }),
        ivs: Some(RandomStats {
            hp: Some(31),
            atk: Some(31),
            def: Some(31),
            spa: Some(31),
            spd: Some(31),
            spe: Some(31),
        }),
        role: Some("Bulky Setup".to_string()),
        tera_type: Some("Water".to_string()),
        gigantamax: Some(false),
    };
    
    // Create a team with just Annihilape and dummy Pokemon
    let team_one = vec![annihilape_set];
    let team_two = vec![RandomPokemonSet {
        name: "Dummy".to_string(),
        species: "Pikachu".to_string(),
        level: 50,
        gender: Some("M".to_string()),
        shiny: Some(false),
        ability: Some("Static".to_string()),
        item: None,
        moves: vec!["Thunderbolt".to_string()],
        nature: Some("Hardy".to_string()),
        evs: None,
        ivs: None,
        role: None,
        tera_type: None,
        gigantamax: None,
    }];
    
    // Use the exact same flow as the battle
    let format = BattleFormat::gen9_random_battle();
    println!("Creating battle state...");
    let state = State::new_with_teams(format, team_one, team_two);
    println!("Battle state created.");
    
    // Check the Annihilape stats in the battle state
    let annihilape = &state.side_one.pokemon[0];
    println!("Battle state Annihilape stats:");
    println!("Attack: {}", annihilape.stats.attack);
    println!("Defense: {}", annihilape.stats.defense);
    println!("Special Attack: {}", annihilape.stats.special_attack);
    println!("Special Defense: {}", annihilape.stats.special_defense);
    println!("Speed: {}", annihilape.stats.speed);
    
    // Test the serialization to see if that's where corruption happens
    println!("\nTesting serialization...");
    let serialized = state.serialize();
    println!("Serialized battle state (truncated): {}...", &serialized[..200]);
    
    // Extract just the Pokemon serialization  
    let pokemon_serialized = annihilape.serialize();
    println!("Pokemon serialized: {}", pokemon_serialized);
    
    // Test deserialization
    println!("\nTesting deserialization...");
    let deserialized_state = State::deserialize(&serialized)?;
    let deserialized_annihilape = &deserialized_state.side_one.pokemon[0];
    println!("Deserialized Annihilape stats:");
    println!("Attack: {}", deserialized_annihilape.stats.attack);
    println!("Defense: {}", deserialized_annihilape.stats.defense);
    println!("Special Attack: {}", deserialized_annihilape.stats.special_attack);
    println!("Special Defense: {}", deserialized_annihilape.stats.special_defense);
    println!("Speed: {}", deserialized_annihilape.stats.speed);
    
    Ok(())
}