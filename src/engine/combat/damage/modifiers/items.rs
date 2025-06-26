//! Item-based damage modifiers
//!
//! This module handles damage modifications from held items,
//! including type-boosting items and stat-modifying items.

/// Get item damage modifier for Generation 2 items
pub fn get_gen2_item_modifier(item: &str, move_type: &str) -> f32 {
    match item.to_lowercase().replace("-", "").replace(" ", "").as_str() {
        // Type-boosting items from Gen 2
        "blackbelt" if move_type == "Fighting" => 1.1,
        "blackglasses" if move_type == "Dark" => 1.1,
        "charcoal" if move_type == "Fire" => 1.1,
        "dragonfang" if move_type == "Dragon" => 1.1,
        "hardstone" if move_type == "Rock" => 1.1,
        "magnet" if move_type == "Electric" => 1.1,
        "metalcoat" if move_type == "Steel" => 1.1,
        "miracleseed" if move_type == "Grass" => 1.1,
        "mysticwater" if move_type == "Water" => 1.1,
        "nevermeltice" if move_type == "Ice" => 1.1,
        "pinkbow" | "polkadotbow" if move_type == "Normal" => 1.1,
        "poisonbarb" if move_type == "Poison" => 1.1,
        "sharpbeak" if move_type == "Flying" => 1.1,
        "silverpowder" if move_type == "Bug" => 1.1,
        "softsand" if move_type == "Ground" => 1.1,
        "spelltag" if move_type == "Ghost" => 1.1,
        "twistedspoon" if move_type == "Psychic" => 1.1,
        
        // Light Ball for Pikachu (doubles attack)
        "lightball" => 2.0, // Applied to Pikachu's attack stat specifically
        
        // Thick Club for Cubone/Marowak (doubles attack)
        "thickclub" => 2.0, // Applied to Cubone/Marowak's attack stat specifically
        
        _ => 1.0,
    }
}