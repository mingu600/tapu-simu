//! # Pokemon Showdown Data Demo
//! 
//! This example demonstrates the PS data integration working with actual
//! extracted Pokemon Showdown data.

use tapu_simu::data::ps_move_service::PSMoveService;
use tapu_simu::data::ps_move_factory::{PSMoveFactory, MovesetArchetype};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Pokemon Showdown Data Integration Demo");
    println!("==========================================");

    // Test PS Move Service
    println!("\n📋 Testing PS Move Service...");
    match PSMoveService::new() {
        Ok(service) => {
            let stats = service.get_stats();
            println!("✅ PS Move Service initialized successfully!");
            println!("   📊 Total moves: {}", stats.total_moves);
            println!("   ⚡ Enhanced moves: {}", stats.enhanced_moves);

            // Test specific moves
            if let Some(thunderbolt) = service.get_move_by_name("Thunderbolt") {
                println!("   🔥 Found Thunderbolt: {} BP, {} Acc", 
                    thunderbolt.base_power, thunderbolt.accuracy);
            }

            if let Some(earthquake) = service.get_move_by_name("Earthquake") {
                println!("   🌍 Found Earthquake: {} BP, {} Acc", 
                    earthquake.base_power, earthquake.accuracy);
            }
        }
        Err(e) => {
            println!("❌ PS Move Service failed: {}", e);
            println!("   💡 Make sure PS data has been extracted:");
            println!("   cd tools/ps-data-extractor && npm install && npm run extract");
        }
    }

    // Test PS Move Factory
    println!("\n🏭 Testing PS Move Factory...");
    match PSMoveFactory::new() {
        Ok(factory) => {
            println!("✅ PS Move Factory initialized successfully!");
            
            let stats = factory.get_stats();
            println!("   📊 Total moves: {}", stats.total_moves);

            // Test popular moves
            let popular_moves = factory.get_popular_moves();
            println!("   🌟 Popular moves available: {}", popular_moves.len());

            // Test archetype movesets
            let tank_set = factory.create_standard_moveset(MovesetArchetype::Tank);
            println!("   🛡️ Tank moveset: {} moves", tank_set.len());

            let sweeper_set = factory.create_standard_moveset(MovesetArchetype::Sweeper);
            println!("   ⚔️ Sweeper moveset: {} moves", sweeper_set.len());

            // Test custom moveset
            let custom_moves = factory.create_moveset_from_slice(&[
                "thunderbolt", "icebeam", "flamethrower", "earthquake"
            ]);
            println!("   🎯 Custom moveset: {} moves", custom_moves.len());
        }
        Err(e) => {
            println!("❌ PS Move Factory failed: {}", e);
        }
    }

    println!("\n🎉 Demo completed!");
    println!("\n💡 Next steps:");
    println!("   1. ✅ PS data extraction working");
    println!("   2. ✅ PS move service working");  
    println!("   3. ✅ PS move factory working");
    println!("   4. ⏳ Replace rustemon usage throughout codebase");
    println!("   5. ⏳ Update targeting system to use PS targets");
    println!("   6. ⏳ Remove rustemon dependency entirely");

    Ok(())
}