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
    item_name: &str,
    _generation: &dyn GenerationBattleMechanics,
    _attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    _move_name: &str,
    _move_type: &str,
    move_category: MoveCategory,
    _context: &DamageContext,
) -> Option<ItemModifier> {
    match item_name {
        "choiceband" => Some(choice_band_effect(move_category)),
        "choicespecs" => Some(choice_specs_effect(move_category)),
        "choicescarf" => Some(choice_scarf_effect()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{Generation, GenerationMechanics};

    #[test]
    fn test_choice_band_physical_move() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_choice_item_effect(
            "choiceband",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.attack_multiplier, 1.5);
        assert_eq!(modifier.special_attack_multiplier, 1.0);
        assert_eq!(modifier.speed_multiplier, 1.0);
    }

    #[test]
    fn test_choice_band_special_move() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_choice_item_effect(
            "choiceband",
            &generation,
            &pokemon,
            None,
            "thunderbolt",
            "electric",
            MoveCategory::Special,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.attack_multiplier, 1.0);
        assert_eq!(modifier.special_attack_multiplier, 1.0);
        assert_eq!(modifier.speed_multiplier, 1.0);
    }

    #[test]
    fn test_choice_specs_special_move() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_choice_item_effect(
            "choicespecs",
            &generation,
            &pokemon,
            None,
            "thunderbolt",
            "electric",
            MoveCategory::Special,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.attack_multiplier, 1.0);
        assert_eq!(modifier.special_attack_multiplier, 1.5);
        assert_eq!(modifier.speed_multiplier, 1.0);
    }

    #[test]
    fn test_choice_specs_physical_move() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_choice_item_effect(
            "choicespecs",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.attack_multiplier, 1.0);
        assert_eq!(modifier.special_attack_multiplier, 1.0);
        assert_eq!(modifier.speed_multiplier, 1.0);
    }

    #[test]
    fn test_choice_scarf() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_choice_item_effect(
            "choicescarf",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.attack_multiplier, 1.0);
        assert_eq!(modifier.special_attack_multiplier, 1.0);
        assert_eq!(modifier.speed_multiplier, 1.5);
    }

    #[test]
    fn test_non_choice_item() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_choice_item_effect(
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