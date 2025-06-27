//! # Choice Items
//!
//! Items that provide stat boosts but lock the user into using the same move.
//! These items provide significant offensive or speed boosts at the cost of flexibility.

use super::{ItemModifier, StatBoosts};
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::GenerationBattleMechanics;
use crate::core::battle_state::{MoveCategory, Pokemon};

/// Get choice item effect if the item is a choice item
pub fn get_choice_item_effect(
    item_id: &crate::types::Items,
    _generation: &dyn GenerationBattleMechanics,
    _attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    _move_id: &crate::types::Moves,
    _move_type: &crate::types::PokemonType,
    move_category: MoveCategory,
    _context: &DamageContext,
) -> Option<ItemModifier> {
    match *item_id {
        crate::types::Items::CHOICEBAND => Some(choice_band_effect(move_category)),
        crate::types::Items::CHOICESPECS => Some(choice_specs_effect(move_category)),
        crate::types::Items::CHOICESCARF => Some(choice_scarf_effect()),
        _ => None,
    }
}

/// Choice Band - Boosts Attack by 1.5x but locks into first move used
fn choice_band_effect(move_category: MoveCategory) -> ItemModifier {
    if move_category == MoveCategory::Physical {
        ItemModifier::new().with_attack_multiplier(1.5)
    } else {
        ItemModifier::default()
    }
}

/// Choice Specs - Boosts Special Attack by 1.5x but locks into first move used
fn choice_specs_effect(move_category: MoveCategory) -> ItemModifier {
    if move_category == MoveCategory::Special {
        ItemModifier::new().with_special_attack_multiplier(1.5)
    } else {
        ItemModifier::default()
    }
}

/// Choice Scarf - Boosts Speed by 1.5x but locks into first move used
fn choice_scarf_effect() -> ItemModifier {
    ItemModifier::new().with_speed_multiplier(1.5)
}

