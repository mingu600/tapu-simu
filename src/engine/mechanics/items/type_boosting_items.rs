//! # Type Boosting Items
//!
//! Items that provide damage multipliers for specific types, including:
//! - Type boosters (24 items): Items that boost specific types by 1.1x-1.2x
//! - Arceus Plates (17 items): Items that change Judgment's type and boost matching moves

use super::ItemModifier;
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::{GenerationBattleMechanics, Generation};
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::types::{ItemId, MoveId, TypeId};

/// Get type boosting item effect if the item is a type booster
pub fn get_type_boosting_item_effect(
    item_id: &ItemId,
    generation: &dyn GenerationBattleMechanics,
    _attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    move_id: &MoveId,
    move_type_id: &TypeId,
    _move_category: MoveCategory,
    _context: &DamageContext,
) -> Option<ItemModifier> {
    match item_id.as_str() {
        // Normal type boosters
        "silkscarf" => Some(type_booster_effect("normal", move_type_id, generation)),
        "pinkbow" => Some(pink_bow_effect(move_type_id)),
        "polkadotbow" => Some(polkadot_bow_effect(move_type_id)),
        
        // Type boosters
        "blackbelt" => Some(type_booster_effect("fighting", move_type_id, generation)),
        "blackglasses" => Some(type_booster_effect("dark", move_type_id, generation)),
        "charcoal" => Some(type_booster_effect("fire", move_type_id, generation)),
        "dragonfang" => Some(type_booster_effect("dragon", move_type_id, generation)),
        "dragonscale" => Some(type_booster_effect("dragon", move_type_id, generation)),
        "hardstone" => Some(type_booster_effect("rock", move_type_id, generation)),
        "magnet" => Some(type_booster_effect("electric", move_type_id, generation)),
        "metalcoat" => Some(type_booster_effect("steel", move_type_id, generation)),
        "mysticwater" => Some(type_booster_effect("water", move_type_id, generation)),
        "nevermeltice" => Some(type_booster_effect("ice", move_type_id, generation)),
        "poisonbarb" => Some(type_booster_effect("poison", move_type_id, generation)),
        "sharpbeak" => Some(type_booster_effect("flying", move_type_id, generation)),
        "silverpowder" => Some(type_booster_effect("bug", move_type_id, generation)),
        "softsand" => Some(type_booster_effect("ground", move_type_id, generation)),
        "spelltag" => Some(type_booster_effect("ghost", move_type_id, generation)),
        "miracleseed" => Some(type_booster_effect("grass", move_type_id, generation)),
        "twistedspoon" => Some(type_booster_effect("psychic", move_type_id, generation)),
        "fairyfeather" => Some(type_booster_effect("fairy", move_type_id, generation)),
        
        // Incense items
        "seaincense" => Some(sea_incense_effect(move_type_id, generation)),
        "waveincense" => Some(type_booster_effect("water", move_type_id, generation)),
        "oddincense" => Some(type_booster_effect("psychic", move_type_id, generation)),
        
        // Arceus plates
        "fistplate" => Some(arceus_plate_effect("fighting", move_id, move_type_id)),
        "skyplate" => Some(arceus_plate_effect("flying", move_id, move_type_id)),
        "toxicplate" => Some(arceus_plate_effect("poison", move_id, move_type_id)),
        "earthplate" => Some(arceus_plate_effect("ground", move_id, move_type_id)),
        "stoneplate" => Some(arceus_plate_effect("rock", move_id, move_type_id)),
        "insectplate" => Some(arceus_plate_effect("bug", move_id, move_type_id)),
        "spookyplate" => Some(arceus_plate_effect("ghost", move_id, move_type_id)),
        "ironplate" => Some(arceus_plate_effect("steel", move_id, move_type_id)),
        "flameplate" => Some(arceus_plate_effect("fire", move_id, move_type_id)),
        "splashplate" => Some(arceus_plate_effect("water", move_id, move_type_id)),
        "meadowplate" => Some(arceus_plate_effect("grass", move_id, move_type_id)),
        "zapplate" => Some(arceus_plate_effect("electric", move_id, move_type_id)),
        "mindplate" => Some(arceus_plate_effect("psychic", move_id, move_type_id)),
        "icicleplate" => Some(arceus_plate_effect("ice", move_id, move_type_id)),
        "dracoplate" => Some(arceus_plate_effect("dragon", move_id, move_type_id)),
        "dreadplate" => Some(arceus_plate_effect("dark", move_id, move_type_id)),
        "pixieplate" => Some(arceus_plate_effect("fairy", move_id, move_type_id)),
        
        _ => None,
    }
}

/// Standard type booster with generation-aware multipliers
fn type_booster_effect(
    boosted_type: &str,
    move_type_id: &TypeId,
    generation: &dyn GenerationBattleMechanics,
) -> ItemModifier {
    if move_type_id.as_str() == boosted_type {
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
fn pink_bow_effect(move_type_id: &TypeId) -> ItemModifier {
    if move_type_id.as_str() == "normal" {
        ItemModifier::new().with_power_multiplier(1.1)
    } else {
        ItemModifier::default()
    }
}

/// Polkadot Bow - Normal-type moves with 1.1x boost across all generations
fn polkadot_bow_effect(move_type_id: &TypeId) -> ItemModifier {
    if move_type_id.as_str() == "normal" {
        ItemModifier::new().with_power_multiplier(1.1)
    } else {
        ItemModifier::default()
    }
}

/// Sea Incense - Water-type moves with generation-specific multipliers
fn sea_incense_effect(move_type_id: &TypeId, generation: &dyn GenerationBattleMechanics) -> ItemModifier {
    if move_type_id.as_str() == "water" {
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
fn arceus_plate_effect(plate_type: &str, move_id: &MoveId, move_type_id: &TypeId) -> ItemModifier {
    let mut modifier = ItemModifier::new();

    // Change Judgment to plate type
    if move_id.as_str() == "judgment" {
        modifier = modifier.with_type_change(plate_type.to_string());
    }

    // Boost matching type moves
    if move_type_id.as_str() == plate_type {
        modifier = modifier.with_power_multiplier(1.2);
    }

    modifier
}

