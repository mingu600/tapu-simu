//! # Utility Items
//!
//! Miscellaneous utility items with various effects including healing, accuracy changes,
//! priority modifications, and reactive behaviors.

use super::{ItemModifier, StatBoosts};
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::GenerationBattleMechanics;
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::core::battle_format::BattlePosition;
use crate::core::instructions::{Stat, PokemonStatus};
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction, StatsInstruction};
use std::collections::HashMap;

/// Get utility item effect if the item is a utility item
pub fn get_utility_item_effect(
    item_name: &str,
    _generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    _move_name: &str,
    move_type: &str,
    move_category: MoveCategory,
    context: &DamageContext,
) -> Option<ItemModifier> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        "leftovers" => Some(ItemModifier::default()), // End-of-turn healing only
        "protectivepads" => Some(protective_pads_effect()),
        "throatspray" => Some(throat_spray_effect(context)),
        "widelens" => Some(wide_lens_effect()),
        "ironball" => Some(iron_ball_effect()),
        "loadeddice" => Some(loaded_dice_effect(context)),
        "blunderpolicy" => Some(ItemModifier::default()), // Triggers on miss only
        "quickclaw" => Some(quick_claw_effect()),
        "adrenalineorb" => Some(ItemModifier::default()), // Triggers on Intimidate only
        _ => None,
    }
}

/// Get HP restore per turn for utility items
pub fn get_item_hp_restore_per_turn(
    item_name: &str,
    pokemon: &Pokemon,
    position: BattlePosition,
    _generation: &dyn GenerationBattleMechanics,
) -> Option<BattleInstructions> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        "leftovers" => Some(leftovers_end_of_turn_effect(pokemon, position)),
        _ => None,
    }
}

/// Check for utility item effects that trigger on switch-in
pub fn get_item_on_switch_in_effects(
    item_name: &str,
    _pokemon: &Pokemon,
    _position: BattlePosition,
    _generation: &dyn GenerationBattleMechanics,
) -> Option<BattleInstructions> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        // Most utility items don't have switch-in effects
        _ => None,
    }
}

// =============================================================================
// INDIVIDUAL UTILITY ITEM IMPLEMENTATIONS
// =============================================================================

/// Protective Pads - Removes contact flag from moves
fn protective_pads_effect() -> ItemModifier {
    ItemModifier::new().with_contact_removal()
}

/// Throat Spray - +1 Special Attack when using sound moves
fn throat_spray_effect(context: &DamageContext) -> ItemModifier {
    // Check if move has sound flag
    if context.move_info.data.flags.contains_key("sound") {
        ItemModifier::new()
            .with_stat_boosts(StatBoosts::special_attack(1))
            .with_consumed()
    } else {
        ItemModifier::default()
    }
}

/// Wide Lens - Increases move accuracy by 1.1x
fn wide_lens_effect() -> ItemModifier {
    ItemModifier::new().with_accuracy_multiplier(1.1)
}

/// Iron Ball - Halves speed, makes user grounded
fn iron_ball_effect() -> ItemModifier {
    ItemModifier::new().with_speed_multiplier(0.5)
}

/// Loaded Dice - Multi-hit moves hit more times
fn loaded_dice_effect(context: &DamageContext) -> ItemModifier {
    // Check if move is multi-hit
    if context.move_info.data.flags.contains_key("multihit") {
        // The actual multi-hit logic is handled in move execution
        // This just indicates the item is active
        ItemModifier::default()
    } else {
        ItemModifier::default()
    }
}

/// Quick Claw - May move first regardless of speed (20% chance)
fn quick_claw_effect() -> ItemModifier {
    // Simplified implementation - in reality this would be probabilistic
    // 20% chance to get +1 priority
    ItemModifier::new().with_priority_modifier(1)
}

/// Leftovers - Restore 1/16 max HP each turn
fn leftovers_end_of_turn_effect(pokemon: &Pokemon, position: BattlePosition) -> BattleInstructions {
    // Heal 1/16 of max HP
    let heal_amount = pokemon.max_hp / 16;
    let instruction = BattleInstruction::Pokemon(PokemonInstruction::Heal {
        target: position,
        amount: heal_amount,
        previous_hp: Some(pokemon.hp),
    });
    BattleInstructions::new(100.0, vec![instruction])
}

/// Generate instructions for utility items that trigger on move miss
pub fn generate_miss_trigger_instructions(
    item_name: &str,
    position: BattlePosition,
) -> Option<BattleInstructions> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        "blunderpolicy" => Some(blunder_policy_miss_effect(position)),
        _ => None,
    }
}

/// Generate instructions for utility items that trigger on abilities
pub fn generate_ability_trigger_instructions(
    item_name: &str,
    position: BattlePosition,
    ability_name: &str,
) -> Option<BattleInstructions> {
    let normalized_name = item_name.to_lowercase().replace(&[' ', '-', '\''][..], "");
    
    match normalized_name.as_str() {
        "adrenalineorb" => {
            if ability_name.to_lowercase() == "intimidate" {
                Some(adrenaline_orb_intimidate_effect(position))
            } else {
                None
            }
        },
        _ => None,
    }
}

/// Blunder Policy - +2 Speed when missing a move
fn blunder_policy_miss_effect(position: BattlePosition) -> BattleInstructions {
    let mut stat_changes = HashMap::new();
    stat_changes.insert(Stat::Speed, 2);
    
    let instructions = vec![
        BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: position,
            stat_changes,
            previous_boosts: HashMap::new(),
        }),
        BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
            target: position,
            new_item: None,
            previous_item: Some("Blunder Policy".to_string()),
        })
    ];
    BattleInstructions::new(100.0, instructions)
}

/// Adrenaline Orb - +1 Speed when intimidated
fn adrenaline_orb_intimidate_effect(position: BattlePosition) -> BattleInstructions {
    let mut stat_changes = HashMap::new();
    stat_changes.insert(Stat::Speed, 1);
    
    let instructions = vec![
        BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: position,
            stat_changes,
            previous_boosts: HashMap::new(),
        }),
        BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
            target: position,
            new_item: None,
            previous_item: Some("Adrenaline Orb".to_string()),
        })
    ];
    BattleInstructions::new(100.0, instructions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generation::{Generation, GenerationMechanics};

    #[test]
    fn test_leftovers() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let mut pokemon = Pokemon::default();
        pokemon.max_hp = 100;
        pokemon.hp = 90;
        
        let instructions = get_item_hp_restore_per_turn("leftovers", &pokemon, crate::core::battle_format::BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0), &generation).unwrap();
        // Should heal 1/16 = 6 HP
        assert!(!instructions.instruction_list.is_empty());
    }

    #[test]
    fn test_protective_pads() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_utility_item_effect(
            "protectivepads",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert!(modifier.removes_contact);
    }

    #[test]
    fn test_wide_lens() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_utility_item_effect(
            "widelens",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.accuracy_multiplier, 1.1);
    }

    #[test]
    fn test_iron_ball() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_utility_item_effect(
            "ironball",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.speed_multiplier, 0.5);
    }

    #[test]
    fn test_quick_claw() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_utility_item_effect(
            "quickclaw",
            &generation,
            &pokemon,
            None,
            "tackle",
            "normal",
            MoveCategory::Physical,
            &context,
        ).unwrap();
        
        assert_eq!(modifier.priority_modifier, 1);
    }

    #[test]
    fn test_blunder_policy_miss() {
        let position = BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0);
        
        let instructions = generate_miss_trigger_instructions("blunderpolicy", position).unwrap();
        assert!(!instructions.instruction_list.is_empty());
    }

    #[test]
    fn test_adrenaline_orb_intimidate() {
        let position = BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0);
        
        let instructions = generate_ability_trigger_instructions("adrenalineorb", position, "intimidate").unwrap();
        assert!(!instructions.instruction_list.is_empty());
    }

    #[test]
    fn test_adrenaline_orb_other_ability() {
        let position = BattlePosition::new(crate::core::battle_format::SideReference::SideOne, 0);
        
        let instructions = generate_ability_trigger_instructions("adrenalineorb", position, "levitate");
        assert!(instructions.is_none());
    }

    #[test]
    fn test_non_utility_item() {
        let generation = GenerationMechanics::new(Generation::Gen9);
        let pokemon = Pokemon::default();
        let context = DamageContext::default();
        
        let modifier = get_utility_item_effect(
            "choiceband",
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