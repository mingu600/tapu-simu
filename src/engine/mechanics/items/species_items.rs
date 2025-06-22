//! # Species Items
//!
//! Items designed for specific Pokemon species, often providing stat boosts,
//! form changes, or type-specific enhancements.

use super::ItemModifier;
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::GenerationBattleMechanics;
use crate::core::battle_state::{MoveCategory, Pokemon};

/// Get species item effect if the item is species-specific
pub fn get_species_item_effect(
    item_name: &str,
    generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    _move_name: &str,
    move_type: &str,
    move_category: MoveCategory,
    context: &DamageContext,
) -> Option<ItemModifier> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        // Classic Species Items
        "thickclub" => Some(thick_club_effect(attacker, move_category)),
        "lightball" => Some(light_ball_effect(attacker, move_category)),
        "souldew" => Some(soul_dew_effect(attacker, move_type, generation)),
        "adamantorb" => Some(adamant_orb_effect(attacker, move_type)),
        "lustrousorb" => Some(lustrous_orb_effect(attacker, move_type)),
        "griseousorb" => Some(griseous_orb_effect(attacker, move_type)),
        
        // Modern Species Items
        "rustedsword" => Some(rusted_sword_effect(attacker)),
        "rustedshield" => Some(rusted_shield_effect(attacker)),
        "cornerstonemask" => Some(ogerpon_mask_effect(attacker, "cornerstone")),
        "hearthflamemask" => Some(ogerpon_mask_effect(attacker, "hearthflame")),
        "wellspringmask" => Some(ogerpon_mask_effect(attacker, "wellspring")),
        
        _ => None,
    }
}

// =============================================================================
// CLASSIC SPECIES ITEMS (6 items)
// =============================================================================

/// Thick Club - Doubles Attack for Cubone and Marowak
fn thick_club_effect(attacker: &Pokemon, move_category: MoveCategory) -> ItemModifier {
    let species_name = attacker.species.to_lowercase();
    if (species_name.contains("cubone") || species_name.contains("marowak"))
        && move_category == MoveCategory::Physical
    {
        ItemModifier::new().with_attack_multiplier(2.0)
    } else {
        ItemModifier::default()
    }
}

/// Light Ball - Doubles Attack and Special Attack for Pikachu
fn light_ball_effect(attacker: &Pokemon, move_category: MoveCategory) -> ItemModifier {
    let species_name = attacker.species.to_lowercase();
    if species_name.contains("pikachu") {
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
    move_type: &str,
    generation: &dyn GenerationBattleMechanics,
) -> ItemModifier {
    let species_name = attacker.species.to_lowercase();
    if species_name.contains("latios") || species_name.contains("latias") {
        use crate::generation::Generation;
        match generation.generation() {
            Generation::Gen7 | Generation::Gen8 | Generation::Gen9 => {
                // Gen 7+: Boost Dragon/Psychic moves by 20%
                if move_type.to_lowercase() == "dragon" || move_type.to_lowercase() == "psychic" {
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
fn adamant_orb_effect(attacker: &Pokemon, move_type: &str) -> ItemModifier {
    let species_name = attacker.species.to_lowercase();
    if species_name.contains("dialga") {
        let move_type_lower = move_type.to_lowercase();
        if move_type_lower == "dragon" || move_type_lower == "steel" {
            ItemModifier::new().with_power_multiplier(1.2)
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Lustrous Orb - Boosts Dragon/Water moves for Palkia
fn lustrous_orb_effect(attacker: &Pokemon, move_type: &str) -> ItemModifier {
    let species_name = attacker.species.to_lowercase();
    if species_name.contains("palkia") {
        let move_type_lower = move_type.to_lowercase();
        if move_type_lower == "dragon" || move_type_lower == "water" {
            ItemModifier::new().with_power_multiplier(1.2)
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    }
}

/// Griseous Orb - Boosts Dragon/Ghost moves for Giratina
fn griseous_orb_effect(attacker: &Pokemon, move_type: &str) -> ItemModifier {
    let species_name = attacker.species.to_lowercase();
    if species_name.contains("giratina") {
        let move_type_lower = move_type.to_lowercase();
        if move_type_lower == "dragon" || move_type_lower == "ghost" {
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
    let species_name = attacker.species.to_lowercase();
    if species_name.contains("zacian") {
        // Rusted Sword primarily provides forme change
        // The actual stat boosts are handled by the forme, not the item directly
        ItemModifier::default()
    } else {
        ItemModifier::default()
    }
}

/// Rusted Shield - Zamazenta forme item
fn rusted_shield_effect(attacker: &Pokemon) -> ItemModifier {
    let species_name = attacker.species.to_lowercase();
    if species_name.contains("zamazenta") {
        // Rusted Shield primarily provides forme change
        // The actual stat boosts are handled by the forme, not the item directly
        ItemModifier::default()
    } else {
        ItemModifier::default()
    }
}

/// Ogerpon Masks - 1.2x power boost for matching Ogerpon forms
fn ogerpon_mask_effect(attacker: &Pokemon, mask_type: &str) -> ItemModifier {
    let species_name = attacker.species.to_lowercase();
    if species_name.contains("ogerpon") {
        // Check if the Ogerpon forme matches the mask
        match mask_type {
            "cornerstone" => {
                if species_name.contains("cornerstone") {
                    ItemModifier::new().with_power_multiplier(1.2)
                } else {
                    ItemModifier::default()
                }
            },
            "hearthflame" => {
                if species_name.contains("hearthflame") {
                    ItemModifier::new().with_power_multiplier(1.2)
                } else {
                    ItemModifier::default()
                }
            },
            "wellspring" => {
                if species_name.contains("wellspring") {
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

