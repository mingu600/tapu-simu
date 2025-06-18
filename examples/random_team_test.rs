//! Test random team loading functionality

use tapu_simu::data::{RandomTeamLoader, RandomTeam};
use tapu_simu::core::battle_format::BattleFormat;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Random Team Loader Integration");
    println!("=====================================");

    // Create a random team loader
    let mut loader = RandomTeamLoader::new();

    // Test Gen 9 Random Battle format
    let gen9_random = BattleFormat::gen9_random_battle();
    println!("\nTesting format: {}", gen9_random.name);
    
    // Check if teams are available
    match loader.get_random_team(&gen9_random) {
        Ok(team) => {
            println!("✓ Successfully loaded random team with {} Pokemon", team.len());
            
            // Display first Pokemon details
            if let Some(pokemon) = team.first() {
                println!("\nFirst Pokemon details:");
                println!("  Name: {}", pokemon.name);
                println!("  Species: {}", pokemon.species);
                println!("  Level: {}", pokemon.level);
                println!("  Ability: {:?}", pokemon.ability);
                println!("  Item: {:?}", pokemon.item);
                println!("  Nature: {:?}", pokemon.nature);
                println!("  Moves: {:?}", pokemon.moves);
                if let Some(role) = &pokemon.role {
                    println!("  Role: {}", role);
                }
                if let Some(tera_type) = &pokemon.tera_type {
                    println!("  Tera Type: {}", tera_type);
                }
                
                // Test our utility methods
                println!("  Parsed Nature: {:?}", pokemon.get_nature());
                println!("  Parsed Tera Type: {:?}", pokemon.get_tera_type());
                println!("  Is Shiny: {}", pokemon.is_shiny());
            }
            
            // Load a few more teams to test randomness
            println!("\nTesting randomness - loading 3 more teams:");
            for i in 1..=3 {
                match loader.get_random_team(&gen9_random) {
                    Ok(team) => {
                        if let Some(pokemon) = team.first() {
                            println!("  Team {}: {} (Level {})", i, pokemon.name, pokemon.level);
                        }
                    }
                    Err(e) => println!("  Team {}: Failed - {}", i, e),
                }
            }
        }
        Err(e) => {
            println!("✗ Failed to load team: {}", e);
            println!("Make sure you have generated teams by running:");
            println!("  cd tools/ps-data-extractor && node generate-teams-from-randbats.js gen9randombattle 100");
        }
    }

    // Test other formats if available
    let formats_to_test = vec![
        BattleFormat::gen9_random_doubles(),
        BattleFormat::gen8_random_battle(),
    ];

    for format in formats_to_test {
        println!("\n\nTesting format: {}", format.name);
        match loader.get_random_team(&format) {
            Ok(team) => {
                println!("✓ Successfully loaded team with {} Pokemon", team.len());
                if let Some(pokemon) = team.first() {
                    println!("  First Pokemon: {} (Level {})", pokemon.name, pokemon.level);
                }
            }
            Err(e) => {
                println!("✗ Failed: {}", e);
            }
        }
    }

    // Test team count functionality
    println!("\n\nTeam counts:");
    for format in BattleFormat::random_battle_formats() {
        if let Some(count) = loader.get_team_count(&format) {
            println!("  {}: {} teams", format.name, count);
        } else {
            println!("  {}: Not loaded", format.name);
        }
    }

    // List available format files
    println!("\n\nAvailable format files:");
    match RandomTeamLoader::list_available_formats() {
        Ok(formats) => {
            for format in formats {
                println!("  {}", format);
            }
        }
        Err(e) => {
            println!("  Error listing formats: {}", e);
        }
    }

    println!("\nTest completed!");
    Ok(())
}