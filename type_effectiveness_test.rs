use std::path::Path;

fn main() {
    println!("Testing type effectiveness with minimal approach...");
    
    // Test the dex creation directly
    match std::fs::read_to_string("data/ps-extracted/typechart.json") {
        Ok(content) => {
            println!("Successfully read typechart.json");
            println!("Content length: {} bytes", content.len());
            
            // Try to parse as JSON
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(chart_data) => {
                    println!("Successfully parsed JSON");
                    if let Some(obj) = chart_data.as_object() {
                        println!("Type chart has {} type entries", obj.len());
                        
                        // Look for Fire type
                        if let Some(fire_data) = obj.get("Fire") {
                            if let Some(damage_taken) = fire_data.get("damageTaken") {
                                if let Some(damage_obj) = damage_taken.as_object() {
                                    println!("Fire type damage taken entries: {}", damage_obj.len());
                                    
                                    // Check Water effectiveness vs Fire
                                    if let Some(water_eff) = damage_obj.get("Water") {
                                        println!("Water vs Fire effectiveness: {:?}", water_eff);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("Failed to parse JSON: {}", e),
            }
        }
        Err(e) => println!("Could not read typechart.json: {}", e),
    }
}