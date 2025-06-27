//! # Status Items
//!
//! Items that inflict or interact with status conditions.
//! These items typically have end-of-turn effects.

use super::ItemModifier;
use crate::engine::combat::damage_context::DamageContext;
use crate::generation::GenerationBattleMechanics;
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::core::battle_format::BattlePosition;
use crate::core::instructions::PokemonStatus;
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction};
use crate::types::identifiers::{ItemId, MoveId, TypeId};
use crate::types::PokemonType;

/// Get status item effect if the item affects status conditions
pub fn get_status_item_effect(
    item_id: &ItemId,
    _generation: &dyn GenerationBattleMechanics,
    _attacker: &Pokemon,
    _defender: Option<&Pokemon>,
    _move_id: &MoveId,
    _move_type_id: &TypeId,
    _move_category: MoveCategory,
    _context: &DamageContext,
) -> Option<ItemModifier> {
    match item_id.as_str() {
        // Status items don't modify damage during combat, they have end-of-turn effects
        "blacksludge" | "flameorb" | "toxicorb" => Some(ItemModifier::default()),
        _ => None,
    }
}

/// Get HP restore per turn for status items
pub fn get_item_hp_restore_per_turn(
    item_id: &crate::types::ItemId,
    pokemon: &Pokemon,
    position: BattlePosition,
    _generation: &dyn GenerationBattleMechanics,
) -> Option<BattleInstructions> {
    match item_id.as_str() {
        "blacksludge" => Some(black_sludge_end_of_turn_effect(pokemon, position)),
        "flameorb" => Some(flame_orb_end_of_turn_effect(pokemon, position)),
        "toxicorb" => Some(toxic_orb_end_of_turn_effect(pokemon, position)),
        _ => None,
    }
}

/// Black Sludge - Heals Poison-types by 1/16 max HP, damages others by 1/8 max HP
fn black_sludge_end_of_turn_effect(pokemon: &Pokemon, position: BattlePosition) -> BattleInstructions {
    // Check if Pokemon is Poison-type
    let is_poison_type = pokemon.types.iter().any(|t| *t == PokemonType::Poison);
    
    if is_poison_type {
        // Heal 1/16 of max HP
        let heal_amount = pokemon.max_hp / 16;
        let instruction = BattleInstruction::Pokemon(PokemonInstruction::Heal {
            target: position,
            amount: heal_amount,
            previous_hp: Some(pokemon.hp),
        });
        BattleInstructions::new(100.0, vec![instruction])
    } else {
        // Damage 1/8 of max HP
        let damage_amount = pokemon.max_hp / 8;
        let instruction = BattleInstruction::Pokemon(PokemonInstruction::Damage {
            target: position,
            amount: damage_amount,
            previous_hp: Some(pokemon.hp),
        });
        BattleInstructions::new(100.0, vec![instruction])
    }
}

/// Flame Orb - Inflicts burn status at end of turn if no status
fn flame_orb_end_of_turn_effect(pokemon: &Pokemon, position: BattlePosition) -> BattleInstructions {
    if pokemon.status == PokemonStatus::None {
        let instruction = BattleInstruction::Status(StatusInstruction::Apply {
            target: position,
            status: PokemonStatus::Burn,
            duration: None,
            previous_status: Some(PokemonStatus::None),
            previous_duration: None,
        });
        BattleInstructions::new(100.0, vec![instruction])
    } else {
        BattleInstructions::new(100.0, vec![])
    }
}

/// Toxic Orb - Inflicts badly poisoned status at end of turn if no status
fn toxic_orb_end_of_turn_effect(pokemon: &Pokemon, position: BattlePosition) -> BattleInstructions {
    if pokemon.status == PokemonStatus::None {
        let instruction = BattleInstruction::Status(StatusInstruction::Apply {
            target: position,
            status: PokemonStatus::BadlyPoisoned,
            duration: None,
            previous_status: Some(PokemonStatus::None),
            previous_duration: None,
        });
        BattleInstructions::new(100.0, vec![instruction])
    } else {
        BattleInstructions::new(100.0, vec![])
    }
}

