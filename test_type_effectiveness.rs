use tapu_simu::dex::{ShowdownDex, Dex};
use tapu_simu::types::Type;
use std::path::Path;

fn main() {
    // Try to load real data
    let dex = if let Ok(dex) = ShowdownDex::new(Path::new(".")) {
        dex
    } else {
        println!("Could not load Pokemon Showdown data, creating test dex");
        return;
    };
    
    // Test some basic type effectiveness
    println!("Testing type effectiveness:");
    
    // Water vs Fire (should be 2.0 - super effective)
    let water_vs_fire = dex.get_type_effectiveness(Type::Water, Type::Fire);
    println!("Water vs Fire: {}", water_vs_fire);
    
    // Fire vs Water (should be 0.5 - not very effective)
    let fire_vs_water = dex.get_type_effectiveness(Type::Fire, Type::Water);
    println!("Fire vs Water: {}", fire_vs_water);
    
    // Electric vs Ground (should be 0.0 - no effect)
    let electric_vs_ground = dex.get_type_effectiveness(Type::Electric, Type::Ground);
    println!("Electric vs Ground: {}", electric_vs_ground);
    
    // Normal vs Normal (should be 1.0 - neutral)
    let normal_vs_normal = dex.get_type_effectiveness(Type::Normal, Type::Normal);
    println!("Normal vs Normal: {}", normal_vs_normal);
    
    // Check if type chart is loaded
    println!("Type chart entries: {}", dex.parser.type_chart.len());
}