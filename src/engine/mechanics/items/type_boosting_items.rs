//! # Type Boosting Items
//!
//! Items that provide damage multipliers for specific types, including:
//! - Type boosters (24 items): Items that boost specific types by 1.1x-1.2x
//! - Arceus Plates (17 items): Items that change Judgment's type and boost matching moves

use super::ItemModifier;
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::{GenerationBattleMechanics, Generation};
use crate::core::battle_state::{MoveCategory, Pokemon};

/// Get type boosting item effect if the item is a type booster
pub fn get_type_boosting_item_effect(
    item_name: &str,
    generation: &dyn GenerationBattleMechanics,
    _attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    move_name: &str,
    move_type: &str,
    _move_category: MoveCategory,
    _context: &DamageContext,
) -> Option<ItemModifier> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        // Normal type boosters
        "silkscarf" => Some(type_booster_effect("normal", move_type, generation)),
        "pinkbow" => Some(pink_bow_effect(move_type)),
        "polkadotbow" => Some(polkadot_bow_effect(move_type)),
        
        // Type boosters
        "blackbelt" => Some(type_booster_effect("fighting", move_type, generation)),
        "blackglasses" => Some(type_booster_effect("dark", move_type, generation)),
        "charcoal" => Some(type_booster_effect("fire", move_type, generation)),
        "dragonfang" => Some(type_booster_effect("dragon", move_type, generation)),
        "dragonscale" => Some(type_booster_effect("dragon", move_type, generation)),
        "hardstone" => Some(type_booster_effect("rock", move_type, generation)),
        "magnet" => Some(type_booster_effect("electric", move_type, generation)),
        "metalcoat" => Some(type_booster_effect("steel", move_type, generation)),
        "mysticwater" => Some(type_booster_effect("water", move_type, generation)),
        "nevermeltice" => Some(type_booster_effect("ice", move_type, generation)),
        "poisonbarb" => Some(type_booster_effect("poison", move_type, generation)),
        "sharpbeak" => Some(type_booster_effect("flying", move_type, generation)),
        "silverpowder" => Some(type_booster_effect("bug", move_type, generation)),
        "softsand" => Some(type_booster_effect("ground", move_type, generation)),
        "spelltag" => Some(type_booster_effect("ghost", move_type, generation)),
        "miracleseed" => Some(type_booster_effect("grass", move_type, generation)),
        "twistedspoon" => Some(type_booster_effect("psychic", move_type, generation)),
        "fairyfeather" => Some(type_booster_effect("fairy", move_type, generation)),
        
        // Incense items
        "seaincense" => Some(sea_incense_effect(move_type, generation)),
        "waveincense" => Some(type_booster_effect("water", move_type, generation)),
        "oddincense" => Some(type_booster_effect("psychic", move_type, generation)),
        
        // Arceus plates
        "fistplate" => Some(arceus_plate_effect("fighting", move_name, move_type)),
        "skyplate" => Some(arceus_plate_effect("flying", move_name, move_type)),
        "toxicplate" => Some(arceus_plate_effect("poison", move_name, move_type)),
        "earthplate" => Some(arceus_plate_effect("ground", move_name, move_type)),
        "stoneplate" => Some(arceus_plate_effect("rock", move_name, move_type)),
        "insectplate" => Some(arceus_plate_effect("bug", move_name, move_type)),
        "spookyplate" => Some(arceus_plate_effect("ghost", move_name, move_type)),
        "ironplate" => Some(arceus_plate_effect("steel", move_name, move_type)),
        "flameplate" => Some(arceus_plate_effect("fire", move_name, move_type)),
        "splashplate" => Some(arceus_plate_effect("water", move_name, move_type)),
        "meadowplate" => Some(arceus_plate_effect("grass", move_name, move_type)),
        "zapplate" => Some(arceus_plate_effect("electric", move_name, move_type)),
        "mindplate" => Some(arceus_plate_effect("psychic", move_name, move_type)),
        "icicleplate" => Some(arceus_plate_effect("ice", move_name, move_type)),
        "dracoplate" => Some(arceus_plate_effect("dragon", move_name, move_type)),
        "dreadplate" => Some(arceus_plate_effect("dark", move_name, move_type)),
        "pixieplate" => Some(arceus_plate_effect("fairy", move_name, move_type)),
        
        _ => None,
    }
}

/// Standard type booster with generation-aware multipliers
fn type_booster_effect(
    boosted_type: &str,
    move_type: &str,
    generation: &dyn GenerationBattleMechanics,
) -> ItemModifier {
    if move_type.to_lowercase() == boosted_type.to_lowercase() {
        // Generation-aware multipliers:
        // Gen 2-3: 1.1x multiplier
        // Gen 4+: 1.2x multiplier
        let multiplier = match generation.generation() {
            Generation::Gen2 | Generation::Gen3 => 1.1,
            _ => 1.2, // Gen 4 and later
        };
        ItemModifier::new().with_power_multiplier(multiplier)
    } else {
        ItemModifier::default()
    }
}

/// Pink Bow - Normal-type moves with 1.1x boost (Gen 2-3 only)
fn pink_bow_effect(move_type: &str) -> ItemModifier {
    if move_type.to_lowercase() == "normal" {
        ItemModifier::new().with_power_multiplier(1.1)
    } else {
        ItemModifier::default()
    }
}

/// Polkadot Bow - Normal-type moves with 1.1x boost across all generations
fn polkadot_bow_effect(move_type: &str) -> ItemModifier {
    if move_type.to_lowercase() == "normal" {
        ItemModifier::new().with_power_multiplier(1.1)
    } else {
        ItemModifier::default()
    }
}

/// Sea Incense - Water-type moves with generation-specific multipliers
fn sea_incense_effect(move_type: &str, generation: &dyn GenerationBattleMechanics) -> ItemModifier {
    if move_type.to_lowercase() == "water" {
        let multiplier = match generation.generation() {
            Generation::Gen3 => 1.05,
            Generation::Gen4 | Generation::Gen5 | Generation::Gen6 | 
            Generation::Gen7 | Generation::Gen8 | Generation::Gen9 => 1.2,
            _ => 1.0, // No effect in Gen 1-2
        };
        ItemModifier::new().with_power_multiplier(multiplier)
    } else {
        ItemModifier::default()
    }
}

/// Arceus plate that changes Judgment type and boosts matching moves
fn arceus_plate_effect(plate_type: &str, move_name: &str, move_type: &str) -> ItemModifier {
    let mut modifier = ItemModifier::new();

    // Change Judgment to plate type
    if move_name.to_lowercase() == "judgment" {
        modifier = modifier.with_type_change(plate_type.to_string());
    }

    // Boost matching type moves
    if move_type.to_lowercase() == plate_type.to_lowercase() {
        modifier = modifier.with_power_multiplier(1.2);
    }

    modifier
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{Generation, GenerationMechanics};

    #[test]
    fn test_silk_scarf_normal_move() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_type_boosting_item_effect(
            "silkscarf",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.power_multiplier, 1.2);
        assert_eq!(modifier.damage_multiplier, 1.0);
    }

    #[test]
    fn test_silk_scarf_non_normal_move() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_type_boosting_item_effect(
            "silkscarf",
            &generation,
            &pokemon,
            None,
            "thunderbolt",
            "electric",
            MoveCategory::Special,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.power_multiplier, 1.0);
        assert_eq!(modifier.damage_multiplier, 1.0);
    }

    #[test]
    fn test_charcoal_fire_move() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_type_boosting_item_effect(
            "charcoal",
            &generation,
            &pokemon,
            None,
            "flamethrower",
            "fire",
            MoveCategory::Special,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.power_multiplier, 1.2);
    }

    #[test]
    fn test_fist_plate_judgment() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_type_boosting_item_effect(
            "fistplate",
            &generation,
            &pokemon,
            None,
            "judgment",
            "normal",
            MoveCategory::Special,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.type_change, Some("fighting".to_string()));
        assert_eq!(modifier.power_multiplier, 1.0); // Not fighting type yet
    }

    #[test]
    fn test_fist_plate_fighting_move() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_type_boosting_item_effect(
            "fistplate",
            &generation,
            &pokemon,
            None,
            "closecombat",
            "fighting",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.power_multiplier, 1.2);
        assert!(modifier.type_change.is_none());
    }

    #[test]
    fn test_pink_bow() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_type_boosting_item_effect(
            "pinkbow",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.power_multiplier, 1.1);
    }

    #[test]
    fn test_non_type_boosting_item() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_type_boosting_item_effect(
            "leftovers",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        );
        
        assert!(modifier.is_none());
    }
}