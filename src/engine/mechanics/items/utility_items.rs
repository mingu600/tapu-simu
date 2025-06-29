//! # Utility Items
//!
//! Miscellaneous utility items with various effects including healing, accuracy changes,
//! priority modifications, and reactive behaviors.

use std::collections::HashMap;

use crate::core::battle_format::BattlePosition;
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::core::instructions::{
    BattleInstruction, BattleInstructions, PokemonInstruction, PokemonStatus, Stat,
    StatusInstruction, StatsInstruction,
};
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::GenerationBattleMechanics;
use crate::types::{Abilities, Items, Moves, PokemonType, StatBoostArray};

use super::{ItemModifier, StatBoosts};

/// Get utility item effect if the item is a utility item
pub fn get_utility_item_effect(
    item_id: &Items,
    _generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    _move_id: &Moves,
    move_type_id: &PokemonType,
    move_category: MoveCategory,
    context: &DamageContext,
) -> Option<ItemModifier> {
    match item_id {
        Items::LEFTOVERS => Some(ItemModifier::default()), // End-of-turn healing only
        Items::PROTECTIVEPADS => Some(protective_pads_effect()),
        Items::THROATSPRAY => Some(throat_spray_effect(context)),
        Items::WIDELENS => Some(wide_lens_effect()),
        Items::IRONBALL => Some(iron_ball_effect()),
        Items::LOADEDDICE => Some(loaded_dice_effect(context)),
        Items::BLUNDERPOLICY => Some(ItemModifier::default()), // Triggers on miss only
        Items::QUICKCLAW => Some(quick_claw_effect()),
        Items::ADRENALINEORB => Some(ItemModifier::default()), // Triggers on Intimidate only
        _ => None,
    }
}

/// Get HP restore per turn for utility items
pub fn get_item_hp_restore_per_turn(
    item_id: &Items,
    pokemon: &Pokemon,
    position: BattlePosition,
    _generation: &dyn GenerationBattleMechanics,
) -> Option<BattleInstructions> {
    match item_id {
        Items::LEFTOVERS => Some(leftovers_end_of_turn_effect(pokemon, position)),
        _ => None,
    }
}

/// Check for utility item effects that trigger on switch-in
pub fn get_item_on_switch_in_effects(
    item_id: &Items,
    _pokemon: &Pokemon,
    _position: BattlePosition,
    _generation: &dyn GenerationBattleMechanics,
) -> Option<BattleInstructions> {
    match item_id {
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
    if context.move_info.is_sound {
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
    if context.move_info.is_multihit {
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
    item_id: &Items,
    position: BattlePosition,
) -> Option<BattleInstructions> {
    match item_id {
        Items::BLUNDERPOLICY => Some(blunder_policy_miss_effect(position)),
        _ => None,
    }
}

/// Generate instructions for utility items that trigger on abilities
pub fn generate_ability_trigger_instructions(
    item_id: &Items,
    position: BattlePosition,
    ability_id: &Abilities,
) -> Option<BattleInstructions> {
    match item_id {
        Items::ADRENALINEORB => {
            if *ability_id == Abilities::INTIMIDATE {
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
    let mut stat_changes = StatBoostArray::default();
    stat_changes.insert(Stat::Speed, 2);
    
    let instructions = vec![
        BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: position,
            stat_changes: stat_changes.to_hashmap(),
            previous_boosts: std::collections::HashMap::new(),
        }),
        BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
            target: position,
            new_item: None,
            previous_item: Some(crate::types::Items::BLUNDERPOLICY),
        })
    ];
    BattleInstructions::new(100.0, instructions)
}

/// Adrenaline Orb - +1 Speed when intimidated
fn adrenaline_orb_intimidate_effect(position: BattlePosition) -> BattleInstructions {
    let mut stat_changes = StatBoostArray::default();
    stat_changes.insert(Stat::Speed, 1);
    
    let instructions = vec![
        BattleInstruction::Stats(StatsInstruction::BoostStats {
            target: position,
            stat_changes: stat_changes.to_hashmap(),
            previous_boosts: std::collections::HashMap::new(),
        }),
        BattleInstruction::Pokemon(PokemonInstruction::ChangeItem {
            target: position,
            new_item: None,
            previous_item: Some(crate::types::Items::ADRENALINEORB),
        })
    ];
    BattleInstructions::new(100.0, instructions)
}

