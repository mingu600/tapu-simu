//! Item-based damage modifiers
//!
//! This module handles damage modifications from held items,
//! including type-boosting items and stat-modifying items.

/// Get item damage modifier for Generation 2 items
pub fn get_gen2_item_modifier(item_id: &crate::types::Items, move_type_id: &crate::types::PokemonType) -> f32 {
    match item_id {
        // Type-boosting items from Gen 2
        crate::types::Items::BLACKBELT if *move_type_id == crate::types::PokemonType::Fighting => 1.1,
        crate::types::Items::BLACKGLASSES if *move_type_id == crate::types::PokemonType::Dark => 1.1,
        crate::types::Items::CHARCOAL if *move_type_id == crate::types::PokemonType::Fire => 1.1,
        crate::types::Items::DRAGONFANG if *move_type_id == crate::types::PokemonType::Dragon => 1.1,
        crate::types::Items::HARDSTONE if *move_type_id == crate::types::PokemonType::Rock => 1.1,
        crate::types::Items::MAGNET if *move_type_id == crate::types::PokemonType::Electric => 1.1,
        crate::types::Items::METALCOAT if *move_type_id == crate::types::PokemonType::Steel => 1.1,
        crate::types::Items::MIRACLESEED if *move_type_id == crate::types::PokemonType::Grass => 1.1,
        crate::types::Items::MYSTICWATER if *move_type_id == crate::types::PokemonType::Water => 1.1,
        crate::types::Items::NEVERMELTICE if *move_type_id == crate::types::PokemonType::Ice => 1.1,
        crate::types::Items::PINKBOW | crate::types::Items::POLKADOTBOW if *move_type_id == crate::types::PokemonType::Normal => 1.1,
        crate::types::Items::POISONBARB if *move_type_id == crate::types::PokemonType::Poison => 1.1,
        crate::types::Items::SHARPBEAK if *move_type_id == crate::types::PokemonType::Flying => 1.1,
        crate::types::Items::SILVERPOWDER if *move_type_id == crate::types::PokemonType::Bug => 1.1,
        crate::types::Items::SOFTSAND if *move_type_id == crate::types::PokemonType::Ground => 1.1,
        crate::types::Items::SPELLTAG if *move_type_id == crate::types::PokemonType::Ghost => 1.1,
        crate::types::Items::TWISTEDSPOON if *move_type_id == crate::types::PokemonType::Psychic => 1.1,
        
        // Light Ball for Pikachu (doubles attack)
        crate::types::Items::LIGHTBALL => 2.0, // Applied to Pikachu's attack stat specifically
        
        // Thick Club for Cubone/Marowak (doubles attack)
        crate::types::Items::THICKCLUB => 2.0, // Applied to Cubone/Marowak's attack stat specifically
        
        _ => 1.0,
    }
}