use tapu_simu::data::{RandomPokemonSet, random_team_loader::RandomStats, ps_move_factory::PSMoveFactory, ps_pokemon_factory::PSPokemonFactory};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the factories
    let move_factory = PSMoveFactory::new()?;
    let pokemon_factory = PSPokemonFactory::new()?;
    
    // Create an Annihilape set matching the one from the battle log
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
    
    // Convert to battle Pokemon (this will trigger our debug output)
    let pokemon = annihilape_set.to_battle_pokemon(&move_factory, &pokemon_factory);
    
    println!("\nFinal Pokemon stats:");
    println!("HP: {}/{}", pokemon.hp, pokemon.max_hp);
    println!("Attack: {}", pokemon.stats.attack);
    println!("Defense: {}", pokemon.stats.defense);
    println!("Special Attack: {}", pokemon.stats.special_attack);
    println!("Special Defense: {}", pokemon.stats.special_defense);
    println!("Speed: {}", pokemon.stats.speed);
    
    Ok(())
}