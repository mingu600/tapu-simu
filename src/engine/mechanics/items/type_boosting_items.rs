//! # Type Boosting Items
//!
//! Items that provide damage multipliers for specific types, including:
//! - Type boosters (24 items): Items that boost specific types by 1.1x-1.2x
//! - Arceus Plates (17 items): Items that change Judgment's type and boost matching moves

use super::ItemModifier;
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::{GenerationBattleMechanics, Generation};
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::types::{Items, Moves, PokemonType};

/// Get type boosting item effect if the item is a type booster
pub fn get_type_boosting_item_effect(
    item_id: &Items,
    generation: &dyn GenerationBattleMechanics,
    _attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    move_id: &Moves,
    move_type_id: &PokemonType,
    _move_category: MoveCategory,
    _context: &DamageContext,
) -> Option<ItemModifier> {
    match item_id {
        // Normal type boosters
        Items::SILKSCARF => Some(type_booster_effect(PokemonType::Normal, move_type_id, generation)),
        Items::PINKBOW => Some(pink_bow_effect(move_type_id)),
        Items::POLKADOTBOW => Some(polkadot_bow_effect(move_type_id)),
        
        // Type boosters
        Items::BLACKBELT => Some(type_booster_effect(PokemonType::Fighting, move_type_id, generation)),
        Items::BLACKGLASSES => Some(type_booster_effect(PokemonType::Dark, move_type_id, generation)),
        Items::CHARCOAL => Some(type_booster_effect(PokemonType::Fire, move_type_id, generation)),
        Items::DRAGONFANG => Some(type_booster_effect(PokemonType::Dragon, move_type_id, generation)),
        Items::DRAGONSCALE => Some(type_booster_effect(PokemonType::Dragon, move_type_id, generation)),
        Items::HARDSTONE => Some(type_booster_effect(PokemonType::Rock, move_type_id, generation)),
        Items::MAGNET => Some(type_booster_effect(PokemonType::Electric, move_type_id, generation)),
        Items::METALCOAT => Some(type_booster_effect(PokemonType::Steel, move_type_id, generation)),
        Items::MYSTICWATER => Some(type_booster_effect(PokemonType::Water, move_type_id, generation)),
        Items::NEVERMELTICE => Some(type_booster_effect(PokemonType::Ice, move_type_id, generation)),
        Items::POISONBARB => Some(type_booster_effect(PokemonType::Poison, move_type_id, generation)),
        Items::SHARPBEAK => Some(type_booster_effect(PokemonType::Flying, move_type_id, generation)),
        Items::SILVERPOWDER => Some(type_booster_effect(PokemonType::Bug, move_type_id, generation)),
        Items::SOFTSAND => Some(type_booster_effect(PokemonType::Ground, move_type_id, generation)),
        Items::SPELLTAG => Some(type_booster_effect(PokemonType::Ghost, move_type_id, generation)),
        Items::MIRACLESEED => Some(type_booster_effect(PokemonType::Grass, move_type_id, generation)),
        Items::TWISTEDSPOON => Some(type_booster_effect(PokemonType::Psychic, move_type_id, generation)),
        Items::FAIRYFEATHER => Some(type_booster_effect(PokemonType::Fairy, move_type_id, generation)),
        
        // Incense items
        Items::SEAINCENSE => Some(sea_incense_effect(move_type_id, generation)),
        Items::WAVEINCENSE => Some(type_booster_effect(PokemonType::Water, move_type_id, generation)),
        Items::ODDINCENSE => Some(type_booster_effect(PokemonType::Psychic, move_type_id, generation)),
        
        // Arceus plates
        Items::FISTPLATE => Some(arceus_plate_effect(PokemonType::Fighting, move_id, move_type_id)),
        Items::SKYPLATE => Some(arceus_plate_effect(PokemonType::Flying, move_id, move_type_id)),
        Items::TOXICPLATE => Some(arceus_plate_effect(PokemonType::Poison, move_id, move_type_id)),
        Items::EARTHPLATE => Some(arceus_plate_effect(PokemonType::Ground, move_id, move_type_id)),
        Items::STONEPLATE => Some(arceus_plate_effect(PokemonType::Rock, move_id, move_type_id)),
        Items::INSECTPLATE => Some(arceus_plate_effect(PokemonType::Bug, move_id, move_type_id)),
        Items::SPOOKYPLATE => Some(arceus_plate_effect(PokemonType::Ghost, move_id, move_type_id)),
        Items::IRONPLATE => Some(arceus_plate_effect(PokemonType::Steel, move_id, move_type_id)),
        Items::FLAMEPLATE => Some(arceus_plate_effect(PokemonType::Fire, move_id, move_type_id)),
        Items::SPLASHPLATE => Some(arceus_plate_effect(PokemonType::Water, move_id, move_type_id)),
        Items::MEADOWPLATE => Some(arceus_plate_effect(PokemonType::Grass, move_id, move_type_id)),
        Items::ZAPPLATE => Some(arceus_plate_effect(PokemonType::Electric, move_id, move_type_id)),
        Items::MINDPLATE => Some(arceus_plate_effect(PokemonType::Psychic, move_id, move_type_id)),
        Items::ICICLEPLATE => Some(arceus_plate_effect(PokemonType::Ice, move_id, move_type_id)),
        Items::DRACOPLATE => Some(arceus_plate_effect(PokemonType::Dragon, move_id, move_type_id)),
        Items::DREADPLATE => Some(arceus_plate_effect(PokemonType::Dark, move_id, move_type_id)),
        Items::PIXIEPLATE => Some(arceus_plate_effect(PokemonType::Fairy, move_id, move_type_id)),
        
        _ => None,
    }
}

/// Standard type booster with generation-aware multipliers
fn type_booster_effect(
    boosted_type: PokemonType,
    move_type_id: &PokemonType,
    generation: &dyn GenerationBattleMechanics,
) -> ItemModifier {
    if *move_type_id == boosted_type {
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
fn pink_bow_effect(move_type_id: &PokemonType) -> ItemModifier {
    if *move_type_id == PokemonType::Normal {
        ItemModifier::new().with_power_multiplier(1.1)
    } else {
        ItemModifier::default()
    }
}

/// Polkadot Bow - Normal-type moves with 1.1x boost across all generations
fn polkadot_bow_effect(move_type_id: &PokemonType) -> ItemModifier {
    if *move_type_id == PokemonType::Normal {
        ItemModifier::new().with_power_multiplier(1.1)
    } else {
        ItemModifier::default()
    }
}

/// Sea Incense - Water-type moves with generation-specific multipliers
fn sea_incense_effect(move_type_id: &PokemonType, generation: &dyn GenerationBattleMechanics) -> ItemModifier {
    if *move_type_id == PokemonType::Water {
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
fn arceus_plate_effect(plate_type: PokemonType, move_id: &Moves, move_type_id: &PokemonType) -> ItemModifier {
    let mut modifier = ItemModifier::new();

    // Change Judgment to plate type
    if *move_id == Moves::JUDGMENT {
        modifier = modifier.with_type_change(plate_type.to_normalized_str().to_string());
    }

    // Boost matching type moves
    if *move_type_id == plate_type {
        modifier = modifier.with_power_multiplier(1.2);
    }

    modifier
}

