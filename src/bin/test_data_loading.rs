#!/usr/bin/env cargo run --bin test_data_loading

//! Test data loading to verify all Pokemon Showdown data is properly loaded

use std::path::Path;
use tapu_simu::dex::{ShowdownDex, Dex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing Pokemon Showdown data loading...");
    
    // Load data using our ShowdownDex
    let project_root = Path::new(".");
    let dex = ShowdownDex::new(project_root)?;
    
    // Report data counts
    let species_count = dex.species_count();
    let moves_count = dex.moves_count();
    let abilities_count = dex.abilities_count();
    let items_count = dex.items_count();
    
    println!("\n📊 Data Loading Results:");
    println!("  Species: {}", species_count);
    println!("  Moves: {}", moves_count);
    println!("  Abilities: {}", abilities_count);
    println!("  Items: {}", items_count);
    
    // Check if we have the expected amounts
    if species_count > 1400 {
        println!("✅ Species count looks good ({})", species_count);
    } else {
        println!("⚠️  Species count seems low ({})", species_count);
    }
    
    if moves_count > 900 {
        println!("✅ Moves count looks good ({})", moves_count);
    } else {
        println!("⚠️  Moves count seems low ({})", moves_count);
    }
    
    if abilities_count > 300 {
        println!("✅ Abilities count looks good ({})", abilities_count);
    } else {
        println!("⚠️  Abilities count seems low ({})", abilities_count);
    }
    
    if items_count > 500 {
        println!("✅ Items count looks good ({})", items_count);
    } else {
        println!("⚠️  Items count seems low ({})", items_count);
    }
    
    // Test a few specific lookups
    println!("\n🔍 Testing specific data lookups:");
    
    if let Some(pikachu) = dex.get_species("pikachu") {
        println!("  ✅ Found Pikachu: {}", pikachu.name);
    } else {
        println!("  ❌ Could not find Pikachu");
    }
    
    if let Some(tackle) = dex.get_move("tackle") {
        println!("  ✅ Found Tackle: {} ({:?})", tackle.name, tackle.type_);
    } else {
        println!("  ❌ Could not find Tackle");
    }
    
    if let Some(intimidate) = dex.get_ability("intimidate") {
        println!("  ✅ Found Intimidate: {}", intimidate.name);
    } else {
        println!("  ❌ Could not find Intimidate");
    }
    
    if let Some(leftovers) = dex.get_item("leftovers") {
        println!("  ✅ Found Leftovers: {}", leftovers.name);
    } else {
        println!("  ❌ Could not find Leftovers");
    }
    
    println!("\n🎉 Data loading test completed!");
    
    Ok(())
}