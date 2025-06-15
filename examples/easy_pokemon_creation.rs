//! Example demonstrating easy Pokemon creation with factory methods
//!
//! This shows how much easier it is to create Pokemon for testing and battles
//! using the new factory methods compared to manual construction.

use tapu_simu::pokemon::Pokemon;
use tapu_simu::dex::{Dex, ShowdownDex};
use tapu_simu::types::{Nature, Gender, StatsTable, EVStatType};
use tapu_simu::battle::Battle;
use tapu_simu::battle_state::BattleState;
use tapu_simu::side::{Side, SideId};
use tapu_simu::format::{BattleFormat, FormatRules};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Easy Pokemon Creation Demo ===\n");

    // Try to load the dex - if this fails, we'll show the concept anyway
    let dex_result = ShowdownDex::new(Path::new("data/ps-extracted"));
    
    if let Ok(dex) = dex_result {
        println!("✅ Loaded Pokemon data successfully!");
        println!("   - {} species", dex.species_count());
        println!("   - {} moves", dex.moves_count());
        println!("   - {} abilities", dex.abilities_count());
        println!("   - {} items", dex.items_count());
        
        // Example 1: Simple Pokemon creation
        println!("\n=== Example 1: Simple Pokemon Creation ===");
        match Pokemon::from_dex(
            &dex,
            "pikachu",
            50,
            &["thunderbolt", "quick-attack"],
            None, // Use default ability
            None, // No item
            Some(Nature::Modest), // +SpA, -Atk
            Some(Gender::Male),
        ) {
            Ok(pikachu) => {
                println!("✅ Created {} (Level {})", pikachu.species.name, pikachu.level);
                println!("   Nature: {:?}, Gender: {:?}", pikachu.nature, pikachu.gender);
                println!("   Moves: {}, {}", pikachu.moves[0].id, pikachu.moves[1].id);
                println!("   Stats: HP {}, Atk {}, SpA {}", 
                    pikachu.stats.hp, pikachu.stats.attack, pikachu.stats.special_attack);
            }
            Err(e) => println!("❌ Failed to create Pikachu: {}", e),
        }
        
        // Example 2: Test Pokemon (even easier)
        println!("\n=== Example 2: Test Pokemon (Ultra Easy) ===");
        match Pokemon::test_pokemon(&dex, Some(100)) {
            Ok(test_mon) => {
                println!("✅ Created test {} (Level {})", test_mon.species.name, test_mon.level);
                println!("   Perfect for quick testing and development!");
            }
            Err(e) => println!("❌ Failed to create test Pokemon: {}", e),
        }
        
        // Example 3: Competitive Pokemon
        println!("\n=== Example 3: Competitive Pokemon ===");
        match Pokemon::competitive_pokemon(
            &dex,
            "garchomp",
            50,
            &["earthquake", "dragon-claw", "stone-edge", "swords-dance"],
            "rough-skin",
            Some("choice-band"),
            Nature::Jolly, // +Speed, -SpA
            Some(StatsTable::competitive_evs(EVStatType::Attack, EVStatType::Speed)),
        ) {
            Ok(garchomp) => {
                println!("✅ Created competitive {} (Level {})", garchomp.species.name, garchomp.level);
                println!("   Nature: {:?}, Item: {:?}", garchomp.nature, 
                    garchomp.item.as_ref().map(|i| &i.name));
                println!("   EVs: Atk {}, Spe {}, HP {}", 
                    garchomp.evs.attack, garchomp.evs.speed, garchomp.evs.hp);
            }
            Err(e) => println!("❌ Failed to create Garchomp: {}", e),
        }
        
        // Example 4: Easy team building
        println!("\n=== Example 4: Easy Team Building ===");
        let team_result: Result<Vec<_>, _> = vec![
            Pokemon::from_dex(&dex, "pikachu", 50, &["thunderbolt"], None, None, None, None),
            Pokemon::from_dex(&dex, "charizard", 50, &["flamethrower"], None, None, None, None),
            Pokemon::test_pokemon(&dex, Some(50)),
        ].into_iter().collect();
        
        match team_result {
            Ok(team) => {
                println!("✅ Created team of {} Pokemon:", team.len());
                for (i, mon) in team.iter().enumerate() {
                    println!("   {}. {} (Level {})", i + 1, mon.species.name, mon.level);
                }
            }
            Err(e) => println!("❌ Failed to create team: {}", e),
        }

    } else {
        println!("❌ Could not load Pokemon data from 'data/ps-extracted'");
        println!("   This is expected if you haven't extracted Pokemon Showdown data yet.");
        println!("   Run the data extraction scripts to get the data files.");
    }

    println!("\n=== Comparison: Old vs New Way ===");
    println!("❌ OLD WAY (Manual construction):");
    println!("   let species = SpeciesData {{ ... 20+ fields ... }};");
    println!("   let moves = [");
    println!("       MoveData {{ ... 15+ fields ... }},");
    println!("       MoveData {{ ... 15+ fields ... }},");
    println!("       // ... repeat for all 4 moves");
    println!("   ];");
    println!("   let ability = AbilityData {{ ... 5+ fields ... }};");
    println!("   let pokemon = Pokemon::new(species, level, moves, ability, item, nature, ivs, evs, gender);");
    println!("   // That's 50+ lines of code for one Pokemon!");
    
    println!("\n✅ NEW WAY (Factory methods):");
    println!("   let pokemon = Pokemon::from_dex(&dex, \"pikachu\", 50, &[\"thunderbolt\"], None, None, None, None)?;");
    println!("   // That's 1 line for a fully functional Pokemon!");
    
    println!("\n   Or even easier for testing:");
    println!("   let pokemon = Pokemon::test_pokemon(&dex, Some(50))?;");
    println!("   // Perfect for unit tests and development!");

    Ok(())
}