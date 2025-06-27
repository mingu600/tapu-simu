//! # Species Items
//!
//! Items designed for specific Pokemon species, often providing stat boosts,
//! form changes, or type-specific enhancements.

use super::ItemModifier;
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::GenerationBattleMechanics;
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::types::{Items, Moves, PokemonType};

/// Get species item effect if the item is species-specific
pub fn get_species_item_effect(
    item_id: &Items,
    generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    _move_id: &Moves,
    move_type_id: &PokemonType,
    move_category: MoveCategory,
    context: &DamageContext,
) -> Option<ItemModifier> {
    let move_type_str = move_type_id;
    
    match item_id {
        // Classic Species Items
        Items::THICKCLUB => Some(thick_club_effect(attacker, move_category)),
        Items::LIGHTBALL => Some(light_ball_effect(attacker, move_category)),
        Items::SOULDEW => Some(soul_dew_effect(attacker, move_type_str, generation)),
        Items::ADAMANTORB => Some(adamant_orb_effect(attacker, move_type_str)),
        Items::LUSTROUSORB => Some(lustrous_orb_effect(attacker, move_type_str)),
        Items::GRISEOUSORB => Some(griseous_orb_effect(attacker, move_type_str)),
        
        // Modern Species Items
        Items::RUSTEDSWORD => Some(rusted_sword_effect(attacker)),
        Items::RUSTEDSHIELD => Some(rusted_shield_effect(attacker)),
        Items::CORNERSTONEMASK => Some(ogerpon_mask_effect(attacker, "cornerstone")),
        Items::HEARTHFLAMEMASK => Some(ogerpon_mask_effect(attacker, "hearthflame")),
        Items::WELLSPRINGMASK => Some(ogerpon_mask_effect(attacker, "wellspring")),
        
        _ => None,
    }
}

// =============================================================================
// CLASSIC SPECIES ITEMS (6 items)
// =============================================================================

/// Thick Club - Doubles Attack for Cubone and Marowak
fn thick_club_effect(attacker: &Pokemon, move_category: MoveCategory) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name, 
        crate::types::PokemonName::CUBONE |
        crate::types::PokemonName::MAROWAK |
        crate::types::PokemonName::MAROWAKALOLA |
        crate::types::PokemonName::MAROWAKALOLATOTEM
    )
        && move_category == MoveCategory::Physical
    {
        ItemModifier::new().with_attack_multiplier(2.0)
    } else {
        ItemModifier::default()
    }
}

/// Light Ball - Doubles Attack and Special Attack for Pikachu
fn light_ball_effect(attacker: &Pokemon, move_category: MoveCategory) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name,
        crate::types::PokemonName::PIKACHU |
        crate::types::PokemonName::PIKACHUALOLA |
        crate::types::PokemonName::PIKACHUBELLE |
        crate::types::PokemonName::PIKACHUCOSPLAY |
        crate::types::PokemonName::PIKACHUGMAX |
        crate::types::PokemonName::PIKACHUHOENN |
        crate::types::PokemonName::PIKACHUKALOS |
        crate::types::PokemonName::PIKACHULIBRE |
        crate::types::PokemonName::PIKACHUORIGINAL |
        crate::types::PokemonName::PIKACHUPARTNER |
        crate::types::PokemonName::PIKACHUPHD |
        crate::types::PokemonName::PIKACHUPOPSTAR |
        crate::types::PokemonName::PIKACHUROCKSTAR |
        crate::types::PokemonName::PIKACHUSINNOH |
        crate::types::PokemonName::PIKACHUSTARTER |
        crate::types::PokemonName::PIKACHUUNOVA |
        crate::types::PokemonName::PIKACHUWORLD
    ) {
        match move_category {
            MoveCategory::Physical => ItemModifier::new().with_attack_multiplier(2.0),
            MoveCategory::Special => ItemModifier::new().with_special_attack_multiplier(2.0),
            MoveCategory::Status => ItemModifier::default(),
        }
    } else {
        ItemModifier::default()
    }
}

/// Soul Dew - Boosts Latios/Latias (generation-dependent effects)
fn soul_dew_effect(
    attacker: &Pokemon,
    move_type: &PokemonType,
    generation: &dyn GenerationBattleMechanics,
) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name,
        crate::types::PokemonName::LATIOS |
        crate::types::PokemonName::LATIOSMEGA |
        crate::types::PokemonName::LATIAS |
        crate::types::PokemonName::LATIASMEGA
    ) {
        use crate::generation::Generation;
        match generation.generation() {
            Generation::Gen7 | Generation::Gen8 | Generation::Gen9 => {
                // Gen 7+: Boost Dragon/Psychic moves by 20%
                if *move_type == PokemonType::Dragon || *move_type == PokemonType::Psychic {
                    ItemModifier::new().with_power_multiplier(1.2)
                } else {
                    ItemModifier::default()
                }
            },
            _ => {
                // Gen 3-6: Boost Special Attack and Special Defense by 50%
                ItemModifier::new()
                    .with_special_attack_multiplier(1.5)
                    .with_special_defense_multiplier(1.5)
            }
        }
    } else {
        ItemModifier::default()
    }
}

/// Adamant Orb - Boosts Dragon/Steel moves for Dialga
fn adamant_orb_effect(attacker: &Pokemon, move_type: &PokemonType) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name,
        crate::types::PokemonName::DIALGA |
        crate::types::PokemonName::DIALGAORIGIN
    ) {
        if *move_type == PokemonType::Dragon || *move_type == PokemonType::Steel {
            ItemModifier::new().with_power_multiplier(1.2)
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Lustrous Orb - Boosts Dragon/Water moves for Palkia
fn lustrous_orb_effect(attacker: &Pokemon, move_type: &PokemonType) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name,
        crate::types::PokemonName::PALKIA |
        crate::types::PokemonName::PALKIAORIGIN
    ) {
        if *move_type == PokemonType::Dragon || *move_type == PokemonType::Water {
            ItemModifier::new().with_power_multiplier(1.2)
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Griseous Orb - Boosts Dragon/Ghost moves for Giratina
fn griseous_orb_effect(attacker: &Pokemon, move_type: &PokemonType) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name,
        crate::types::PokemonName::GIRATINA |
        crate::types::PokemonName::GIRATINAORIGIN
    ) {
        if *move_type == PokemonType::Dragon || *move_type == PokemonType::Ghost {
            ItemModifier::new().with_power_multiplier(1.2)
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

// =============================================================================
// MODERN SPECIES ITEMS (3 items)
// =============================================================================

/// Rusted Sword - Zacian forme item
fn rusted_sword_effect(attacker: &Pokemon) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name,
        crate::types::PokemonName::ZACIAN |
        crate::types::PokemonName::ZACIANCROWNED
    ) {
        // Rusted Sword primarily provides forme change
        // The actual stat boosts are handled by the forme, not the item directly
        ItemModifier::default()
    } else {
        ItemModifier::default()
    }
}

/// Rusted Shield - Zamazenta forme item
fn rusted_shield_effect(attacker: &Pokemon) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name,
        crate::types::PokemonName::ZAMAZENTA |
        crate::types::PokemonName::ZAMAZENTACROWNED
    ) {
        // Rusted Shield primarily provides forme change
        // The actual stat boosts are handled by the forme, not the item directly
        ItemModifier::default()
    } else {
        ItemModifier::default()
    }
}

/// Ogerpon Masks - 1.2x power boost for matching Ogerpon forms
fn ogerpon_mask_effect(attacker: &Pokemon, mask_type: &str) -> ItemModifier {
    let species_name = &attacker.species;
    if matches!(*species_name,
        crate::types::PokemonName::OGERPON |
        crate::types::PokemonName::OGERPONCORNERSTONE |
        crate::types::PokemonName::OGERPONCORNERSTONETERA |
        crate::types::PokemonName::OGERPONHEARTHFLAME |
        crate::types::PokemonName::OGERPONHEARTHFLAMETERA |
        crate::types::PokemonName::OGERPONTEALTERA |
        crate::types::PokemonName::OGERPONWELLSPRING |
        crate::types::PokemonName::OGERPONWELLSPRINGTERA
    ) {
        // Check if the Ogerpon forme matches the mask
        match mask_type {
            "cornerstone" => {
                if matches!(*species_name,
                    crate::types::PokemonName::OGERPONCORNERSTONE |
                    crate::types::PokemonName::OGERPONCORNERSTONETERA
                ) {
                    ItemModifier::new().with_power_multiplier(1.2)
                } else {
                    ItemModifier::default()
                }
            },
            "hearthflame" => {
                if matches!(*species_name,
                    crate::types::PokemonName::OGERPONHEARTHFLAME |
                    crate::types::PokemonName::OGERPONHEARTHFLAMETERA
                ) {
                    ItemModifier::new().with_power_multiplier(1.2)
                } else {
                    ItemModifier::default()
                }
            },
            "wellspring" => {
                if matches!(*species_name,
                    crate::types::PokemonName::OGERPONWELLSPRING |
                    crate::types::PokemonName::OGERPONWELLSPRINGTERA
                ) {
                    ItemModifier::new().with_power_multiplier(1.2)
                } else {
                    ItemModifier::default()
                }
            },
            _ => ItemModifier::default(),
        }
    } else {
        ItemModifier::default()
    }
}

