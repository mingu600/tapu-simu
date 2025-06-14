#!/usr/bin/env cargo run --example generation_data_demo --

//! # Generation-Specific Pokemon Showdown Data Demo
//! 
//! This example demonstrates the generation-aware data capabilities
//! of the Pokemon Showdown integration, showing how moves have changed
//! across different Pokemon generations.

use tapu_simu::data::ps_generation_loader::PSGenerationRepository;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 Pokemon Showdown Generation Data Demo");
    println!("=========================================\n");

    // Load generation-specific data
    println!("📂 Loading generation-specific data...");
    let gen_repo = PSGenerationRepository::load_from_directory("data/ps-extracted")?;
    println!("✅ Generation data loaded successfully!\n");

    // Show generation statistics
    println!("📊 Generation Statistics:");
    let stats = gen_repo.get_generation_stats();
    for stat in &stats {
        if stat.has_data {
            println!("   Gen {} ({}): {} moves", stat.generation, stat.name, stat.move_count);
        }
    }
    println!();

    // Show change statistics
    let change_stats = gen_repo.get_change_stats();
    println!("🔄 Move Changes Statistics:");
    println!("   {} moves have changed across generations", change_stats.total_moves_with_changes);
    println!("   {} total changes tracked", change_stats.total_changes);
    println!();

    // Demonstrate specific move evolution: Bite
    println!("🔥 Case Study: Bite Evolution");
    println!("-----------------------------");
    
    if let Some(bite_gen1) = gen_repo.get_move_for_generation("bite", 1) {
        println!("   Gen 1: {} type, {} category, {} BP", 
                 bite_gen1.move_type, bite_gen1.category, bite_gen1.base_power);
    }
    
    if let Some(bite_gen9) = gen_repo.get_move_for_generation("bite", 9) {
        println!("   Gen 9: {} type, {} category, {} BP", 
                 bite_gen9.move_type, bite_gen9.category, bite_gen9.base_power);
    }
    
    if let Some(bite_changes) = gen_repo.get_move_changes("bite") {
        println!("   📈 Changes over time:");
        for change in &bite_changes.changes {
            for field_change in &change.changes {
                println!("      Gen {}: {} {} → {}", 
                         change.generation, 
                         field_change.field,
                         field_change.from,
                         field_change.to);
            }
        }
    }
    println!();

    // Show another interesting case: Gust
    println!("🌪️ Case Study: Gust Evolution");
    println!("-----------------------------");
    
    if let Some(gust_gen1) = gen_repo.get_move_for_generation("gust", 1) {
        println!("   Gen 1: {} type, {} BP", gust_gen1.move_type, gust_gen1.base_power);
    }
    
    if let Some(gust_gen9) = gen_repo.get_move_for_generation("gust", 9) {
        println!("   Gen 9: {} type, {} BP", gust_gen9.move_type, gust_gen9.base_power);
    }
    println!();

    // Show generation availability for a modern move
    println!("⚡ Generation Availability: Accelerock");
    println!("-------------------------------------");
    let accelerock_gens = gen_repo.get_move_generations("accelerock");
    if accelerock_gens.is_empty() {
        println!("   Accelerock is not available in any generation (in our data)");
    } else {
        println!("   Available in generations: {:?}", accelerock_gens);
    }
    println!();

    // Show generation availability for a classic move
    println!("🌿 Generation Availability: Absorb");
    println!("----------------------------------");
    let absorb_gens = gen_repo.get_move_generations("absorb");
    println!("   Available in generations: {:?}", absorb_gens);
    
    if let Some(absorb_changes) = gen_repo.get_move_changes("absorb") {
        println!("   📈 Changes over time:");
        for change in &absorb_changes.changes {
            for field_change in &change.changes {
                println!("      Gen {}: {} {} → {}", 
                         change.generation, 
                         field_change.field,
                         field_change.from,
                         field_change.to);
            }
        }
    }
    println!();

    // Show moves that changed in a specific generation
    println!("🎯 Moves Changed in Gen 6:");
    println!("---------------------------");
    let gen6_changes = gen_repo.get_moves_changed_in_generation(6);
    for (i, change) in gen6_changes.iter().take(5).enumerate() {
        println!("   {}. {}", i + 1, change.name);
        for gen_change in &change.changes {
            if gen_change.generation == 6 {
                for field_change in gen_change.changes.iter().take(2) {
                    println!("      • {} {} → {}", 
                             field_change.field,
                             field_change.from,
                             field_change.to);
                }
            }
        }
    }
    
    if gen6_changes.len() > 5 {
        println!("   ... and {} more moves", gen6_changes.len() - 5);
    }
    println!();

    println!("🎉 Demo completed!");
    println!("\n💡 Key Features Demonstrated:");
    println!("   ✅ Generation-specific move data (Gen 1-9)");
    println!("   ✅ Move change tracking across generations");
    println!("   ✅ Type and stat evolution over time");
    println!("   ✅ Generation availability checking");
    println!("   ✅ Battle-accurate Pokemon Showdown data");

    Ok(())
}