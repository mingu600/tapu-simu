use tapu_simu::dex::{ShowdownDex, Dex};
use tapu_simu::types::Type;
use std::path::Path;

#[test]
fn test_type_effectiveness_integration() {
    // Try to load real Pokemon Showdown data
    let dex = match ShowdownDex::new(Path::new(".")) {
        Ok(dex) => dex,
        Err(_) => {
            println!("Could not load Pokemon Showdown data, skipping test");
            return;
        }
    };
    
    // Test basic type effectiveness
    let fire_vs_water = dex.get_type_effectiveness(Type::Fire, Type::Water);
    let water_vs_fire = dex.get_type_effectiveness(Type::Water, Type::Fire);
    let electric_vs_ground = dex.get_type_effectiveness(Type::Electric, Type::Ground);
    let normal_vs_normal = dex.get_type_effectiveness(Type::Normal, Type::Normal);
    
    println!("Fire vs Water: {}", fire_vs_water);
    println!("Water vs Fire: {}", water_vs_fire);
    println!("Electric vs Ground: {}", electric_vs_ground);
    println!("Normal vs Normal: {}", normal_vs_normal);
    
    // Check that we get expected results
    assert!(fire_vs_water < 1.0, "Fire should be not very effective against Water");
    assert!(water_vs_fire > 1.0, "Water should be super effective against Fire");
    assert_eq!(electric_vs_ground, 0.0, "Electric should have no effect on Ground");
    assert_eq!(normal_vs_normal, 1.0, "Normal should be neutral against Normal");
    
    // Check that type chart has reasonable size
    assert!(dex.parser.type_chart.len() >= 15, "Type chart should have reasonable number of types");
}