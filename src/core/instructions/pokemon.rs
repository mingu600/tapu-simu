//! # Pokemon-Related Instructions
//! 
//! Instructions that affect individual Pokemon: damage, healing, fainting,
//! switching, ability changes, item changes, type changes, etc.

use crate::core::battle_format::BattlePosition;
use crate::types::PokemonStatus;
use crate::types::PokemonType;
use crate::types::Abilities;
use crate::types::from_string::FromNormalizedString;
use serde::{Deserialize, Serialize};

/// Move categories for damage tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

impl MoveCategory {
    /// Convert from string representation
    pub fn from_str(category: &str) -> Self {
        match category {
            "Physical" => MoveCategory::Physical,
            "Special" => MoveCategory::Special,
            _ => MoveCategory::Status,
        }
    }
}

/// Implementation of unified string parsing trait
impl FromNormalizedString for MoveCategory {
    fn from_normalized_str(s: &str) -> Option<Self> {
        match s.to_lowercase().trim() {
            "physical" => Some(Self::Physical),
            "special" => Some(Self::Special),
            "status" => Some(Self::Status),
            _ => None,
        }
    }
    
    fn valid_strings() -> Vec<&'static str> {
        vec!["physical", "special", "status"]
    }
}

/// Pokemon-related instruction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PokemonInstruction {
    /// Deal damage to a Pokemon
    Damage {
        target: BattlePosition,
        amount: i16,
        previous_hp: Option<i16>,
    },
    /// Heal a Pokemon
    Heal {
        target: BattlePosition,
        amount: i16,
        previous_hp: Option<i16>,
    },
    /// Deal damage to multiple Pokemon simultaneously
    MultiTargetDamage {
        target_damages: Vec<(BattlePosition, i16)>,
        previous_hps: Vec<(BattlePosition, Option<i16>)>,
    },
    /// Faint a Pokemon
    Faint {
        target: BattlePosition,
        previous_hp: i16,
        previous_status: Option<PokemonStatus>,
    },
    /// Switch Pokemon at a position
    Switch {
        position: BattlePosition,
        new_pokemon: usize, // Pokemon index in team
        previous_pokemon: Option<usize>,
    },
    /// Change Pokemon's ability
    ChangeAbility {
        target: BattlePosition,
        new_ability: Abilities,
        previous_ability: Option<Abilities>,
    },
    /// Toggle ability suppression
    ToggleAbility {
        target: BattlePosition,
        suppressed: bool,
        previous_state: bool,
    },
    /// Change Pokemon's held item
    ChangeItem {
        target: BattlePosition,
        new_item: Option<crate::types::Items>,
        previous_item: Option<crate::types::Items>,
    },
    /// Change Pokemon's types
    ChangeType {
        target: BattlePosition,
        new_types: Vec<String>,
        previous_types: Vec<String>,
    },
    /// Change Pokemon's forme
    FormeChange {
        target: BattlePosition,
        new_forme: String,
        previous_forme: String,
    },
    /// Toggle Terastallization state
    ToggleTerastallized {
        target: BattlePosition,
        terastallized: bool,
        tera_type: Option<PokemonType>,
        previous_state: bool,
    },
    /// Change substitute health
    ChangeSubstituteHealth {
        target: BattlePosition,
        new_health: i16,
        previous_health: i16,
    },
    /// Set wish healing for a position
    SetWish {
        target: BattlePosition,
        heal_amount: i16,
        turns_remaining: u8,
        previous_wish: Option<(i16, u8)>,
    },
    /// Decrement wish counter
    DecrementWish {
        target: BattlePosition,
        previous_turns: u8,
    },
    /// Set future sight attack
    SetFutureSight {
        target: BattlePosition,
        attacker_position: BattlePosition,
        damage_amount: i16,
        turns_remaining: u8,
        move_name: String,
        previous_future_sight: Option<(BattlePosition, i16, u8, String)>,
    },
    /// Decrement future sight counter
    DecrementFutureSight {
        target: BattlePosition,
        previous_turns: u8,
    },
    /// Track damage dealt (for counter moves)
    ChangeDamageDealt {
        side_position: BattlePosition,
        damage_amount: i16,
        move_category: MoveCategory,
        hit_substitute: bool,
        previous_damage: i16,
        previous_category: MoveCategory,
        previous_hit_substitute: bool,
    },
    /// Display a message (for debugging/logging)
    Message {
        message: String,
        affected_positions: Vec<BattlePosition>,
    },
    /// Transfer an item from one Pokemon to another
    ItemTransfer {
        from: BattlePosition,
        to: BattlePosition,
        item: String,
        previous_from_item: Option<String>,
        previous_to_item: Option<String>,
    },
    /// Force a Pokemon to switch out
    ForceSwitch {
        target: BattlePosition,
        source: Option<BattlePosition>,
        previous_can_switch: bool,
    },
    /// Damage a substitute
    DamageSubstitute {
        target: BattlePosition,
        amount: i16,
        previous_health: i16,
    },
}

impl PokemonInstruction {
    /// Returns all positions affected by this instruction
    pub fn affected_positions(&self) -> Vec<BattlePosition> {
        match self {
            PokemonInstruction::Damage { target, .. } => vec![*target],
            PokemonInstruction::Heal { target, .. } => vec![*target],
            PokemonInstruction::MultiTargetDamage { target_damages, .. } => {
                target_damages.iter().map(|(pos, _)| *pos).collect()
            },
            PokemonInstruction::Faint { target, .. } => vec![*target],
            PokemonInstruction::Switch { position, .. } => vec![*position],
            PokemonInstruction::ChangeAbility { target, .. } => vec![*target],
            PokemonInstruction::ToggleAbility { target, .. } => vec![*target],
            PokemonInstruction::ChangeItem { target, .. } => vec![*target],
            PokemonInstruction::ChangeType { target, .. } => vec![*target],
            PokemonInstruction::FormeChange { target, .. } => vec![*target],
            PokemonInstruction::ToggleTerastallized { target, .. } => vec![*target],
            PokemonInstruction::ChangeSubstituteHealth { target, .. } => vec![*target],
            PokemonInstruction::SetWish { target, .. } => vec![*target],
            PokemonInstruction::DecrementWish { target, .. } => vec![*target],
            PokemonInstruction::SetFutureSight { target, attacker_position, .. } => {
                vec![*target, *attacker_position]
            },
            PokemonInstruction::DecrementFutureSight { target, .. } => vec![*target],
            PokemonInstruction::ChangeDamageDealt { side_position, .. } => vec![*side_position],
            PokemonInstruction::Message { affected_positions, .. } => affected_positions.clone(),
            PokemonInstruction::ItemTransfer { from, to, .. } => vec![*from, *to],
            PokemonInstruction::ForceSwitch { target, .. } => vec![*target],
            PokemonInstruction::DamageSubstitute { target, .. } => vec![*target],
        }
    }

    /// Whether this instruction can be undone
    pub fn is_undoable(&self) -> bool {
        match self {
            // Most Pokemon instructions store previous state for undo
            PokemonInstruction::Damage { previous_hp, .. } => previous_hp.is_some(),
            PokemonInstruction::Heal { previous_hp, .. } => previous_hp.is_some(),
            PokemonInstruction::MultiTargetDamage { previous_hps, .. } => !previous_hps.is_empty(),
            PokemonInstruction::Faint { .. } => true,
            PokemonInstruction::Switch { previous_pokemon, .. } => previous_pokemon.is_some(),
            PokemonInstruction::ChangeAbility { previous_ability, .. } => previous_ability.is_some(),
            PokemonInstruction::ToggleAbility { .. } => true,
            PokemonInstruction::ChangeItem { .. } => true,
            PokemonInstruction::ChangeType { .. } => true,
            PokemonInstruction::FormeChange { .. } => true,
            PokemonInstruction::ToggleTerastallized { .. } => true,
            PokemonInstruction::ChangeSubstituteHealth { .. } => true,
            PokemonInstruction::SetWish { previous_wish, .. } => previous_wish.is_some(),
            PokemonInstruction::DecrementWish { .. } => true,
            PokemonInstruction::SetFutureSight { previous_future_sight, .. } => previous_future_sight.is_some(),
            PokemonInstruction::DecrementFutureSight { .. } => true,
            PokemonInstruction::ChangeDamageDealt { .. } => true,
            PokemonInstruction::Message { .. } => false, // Messages are not undoable
            PokemonInstruction::ItemTransfer { .. } => true,
            PokemonInstruction::ForceSwitch { .. } => true,
            PokemonInstruction::DamageSubstitute { .. } => true,
        }
    }
}