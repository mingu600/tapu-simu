//! # Item System
//!
//! This module provides comprehensive item effects system with generation-aware mechanics.
//! Items can modify damage, stats, provide abilities, immunities, and many other battle effects.

pub mod choice_items;
pub mod type_boosting_items;
pub mod stat_boosting_items;
pub mod berry_items;
pub mod status_items;
pub mod utility_items;
pub mod species_items;

// Re-export all item effect functions
pub use choice_items::*;
pub use type_boosting_items::*;
pub use stat_boosting_items::*;
pub use berry_items::*;
pub use status_items::*;
pub use utility_items::*;
pub use species_items::*;

use crate::engine::combat::damage_context::DamageContext;
use crate::generation::{GenerationMechanics, GenerationBattleMechanics};
use crate::core::battle_state::{MoveCategory, Pokemon};
use crate::core::battle_format::BattlePosition;
use crate::core::instructions::{Stat, PokemonStatus};
use crate::core::instructions::{BattleInstruction, BattleInstructions, StatusInstruction, PokemonInstruction, StatsInstruction};
use std::collections::HashMap;

/// Stat boosts for reactive items
#[derive(Debug, Clone, PartialEq)]
pub struct StatBoosts {
    pub attack: i8,
    pub defense: i8,
    pub special_attack: i8,
    pub special_defense: i8,
    pub speed: i8,
    pub accuracy: i8,
}

impl StatBoosts {
    pub fn new() -> Self {
        Self {
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn attack(attack: i8) -> Self {
        Self {
            attack,
            defense: 0,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn special_attack(special_attack: i8) -> Self {
        Self {
            attack: 0,
            defense: 0,
            special_attack,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn defense(defense: i8) -> Self {
        Self {
            attack: 0,
            defense,
            special_attack: 0,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn special_defense(special_defense: i8) -> Self {
        Self {
            attack: 0,
            defense: 0,
            special_attack: 0,
            special_defense,
            speed: 0,
            accuracy: 0,
        }
    }
    
    pub fn attack_and_special_attack(attack: i8, special_attack: i8) -> Self {
        Self {
            attack,
            defense: 0,
            special_attack,
            special_defense: 0,
            speed: 0,
            accuracy: 0,
        }
    }
}

/// Modifier result from an item
#[derive(Debug, Clone)]
pub struct ItemModifier {
    /// Damage multiplier (1.0 = no change)
    pub damage_multiplier: f32,
    /// Base power multiplier (1.0 = no change)
    pub power_multiplier: f32,
    /// Attack stat multiplier (1.0 = no change)
    pub attack_multiplier: f32,
    /// Defense stat multiplier (1.0 = no change)
    pub defense_multiplier: f32,
    /// Special Attack stat multiplier (1.0 = no change)
    pub special_attack_multiplier: f32,
    /// Special Defense stat multiplier (1.0 = no change)
    pub special_defense_multiplier: f32,
    /// Speed stat multiplier (1.0 = no change)
    pub speed_multiplier: f32,
    /// Whether this move should be completely blocked
    pub blocks_move: bool,
    /// Recoil damage percentage (e.g., 0.1 for 10% recoil)
    pub recoil_percentage: f32,
    /// Move type change (None = no change)
    pub type_change: Option<String>,
    /// Whether item is consumed after use
    pub is_consumed: bool,
    /// Ground immunity (Air Balloon)
    pub ground_immunity: bool,
    /// Hazard immunity (Heavy Duty Boots)
    pub hazard_immunity: bool,
    /// Contact damage to attacker (Rocky Helmet)
    pub contact_recoil: f32,
    /// Stat boosts to apply (for reactive items)
    pub stat_boosts: Option<StatBoosts>,
    /// Prevents KO at full HP (Focus Sash)
    pub prevents_ko_at_full_hp: bool,
    /// Drain percentage for Shell Bell (fraction of damage dealt restored as HP)
    pub drain_percentage: f32,
    /// Removes contact flag from move (Punching Glove)
    pub removes_contact: bool,
    /// Accuracy multiplier (1.0 = no change, >1.0 = more accurate)
    pub accuracy_multiplier: f32,
    /// Priority modifier (0 = no change, +1 = +1 priority, etc.)
    pub priority_modifier: i8,
}

impl Default for ItemModifier {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            power_multiplier: 1.0,
            attack_multiplier: 1.0,
            defense_multiplier: 1.0,
            special_attack_multiplier: 1.0,
            special_defense_multiplier: 1.0,
            speed_multiplier: 1.0,
            blocks_move: false,
            recoil_percentage: 0.0,
            type_change: None,
            is_consumed: false,
            ground_immunity: false,
            hazard_immunity: false,
            contact_recoil: 0.0,
            stat_boosts: None,
            prevents_ko_at_full_hp: false,
            drain_percentage: 0.0,
            removes_contact: false,
            accuracy_multiplier: 1.0,
            priority_modifier: 0,
        }
    }
}

impl ItemModifier {
    /// Create a new modifier with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set damage multiplier
    pub fn with_damage_multiplier(mut self, multiplier: f32) -> Self {
        self.damage_multiplier = multiplier;
        self
    }

    /// Set power multiplier
    pub fn with_power_multiplier(mut self, multiplier: f32) -> Self {
        self.power_multiplier = multiplier;
        self
    }

    /// Set attack multiplier
    pub fn with_attack_multiplier(mut self, multiplier: f32) -> Self {
        self.attack_multiplier = multiplier;
        self
    }

    /// Set defense multiplier
    pub fn with_defense_multiplier(mut self, multiplier: f32) -> Self {
        self.defense_multiplier = multiplier;
        self
    }

    /// Set special attack multiplier
    pub fn with_special_attack_multiplier(mut self, multiplier: f32) -> Self {
        self.special_attack_multiplier = multiplier;
        self
    }

    /// Set special defense multiplier
    pub fn with_special_defense_multiplier(mut self, multiplier: f32) -> Self {
        self.special_defense_multiplier = multiplier;
        self
    }

    /// Set speed multiplier
    pub fn with_speed_multiplier(mut self, multiplier: f32) -> Self {
        self.speed_multiplier = multiplier;
        self
    }

    /// Block the move entirely
    pub fn with_move_blocked(mut self) -> Self {
        self.blocks_move = true;
        self
    }

    /// Set recoil percentage
    pub fn with_recoil_percentage(mut self, percentage: f32) -> Self {
        self.recoil_percentage = percentage;
        self
    }

    /// Change move type
    pub fn with_type_change(mut self, new_type: String) -> Self {
        self.type_change = Some(new_type);
        self
    }

    /// Mark item as consumed
    pub fn with_consumed(mut self) -> Self {
        self.is_consumed = true;
        self
    }

    /// Grant ground immunity
    pub fn with_ground_immunity(mut self) -> Self {
        self.ground_immunity = true;
        self
    }

    /// Grant hazard immunity
    pub fn with_hazard_immunity(mut self) -> Self {
        self.hazard_immunity = true;
        self
    }

    /// Set contact recoil damage
    pub fn with_contact_recoil(mut self, recoil: f32) -> Self {
        self.contact_recoil = recoil;
        self
    }

    /// Add stat boosts
    pub fn with_stat_boosts(mut self, boosts: StatBoosts) -> Self {
        self.stat_boosts = Some(boosts);
        self
    }

    /// Prevent KO at full HP
    pub fn with_ko_prevention_at_full_hp(mut self) -> Self {
        self.prevents_ko_at_full_hp = true;
        self
    }

    /// Set drain percentage
    pub fn with_drain_percentage(mut self, percentage: f32) -> Self {
        self.drain_percentage = percentage;
        self
    }

    /// Remove contact flag
    pub fn with_contact_removal(mut self) -> Self {
        self.removes_contact = true;
        self
    }

    /// Set accuracy multiplier
    pub fn with_accuracy_multiplier(mut self, multiplier: f32) -> Self {
        self.accuracy_multiplier = multiplier;
        self
    }

    /// Set priority modifier
    pub fn with_priority_modifier(mut self, modifier: i8) -> Self {
        self.priority_modifier = modifier;
        self
    }
}

/// Main item lookup function - delegates to category-specific functions
pub fn get_item_by_name_with_generation(
    item_name: &str,
    generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    defender: Option<&Pokemon>,
    move_name: &str,
    move_type: &str,
    move_category: MoveCategory,
    context: &DamageContext,
) -> ItemModifier {
    // Try each category in order
    if let Some(modifier) = choice_items::get_choice_item_effect(
        item_name, generation, attacker, defender, move_name, move_type, move_category, context
    ) {
        return modifier;
    }
    
    if let Some(modifier) = type_boosting_items::get_type_boosting_item_effect(
        item_name, generation, attacker, defender, move_name, move_type, move_category, context
    ) {
        return modifier;
    }
    
    if let Some(modifier) = stat_boosting_items::get_stat_boosting_item_effect(
        item_name, generation, attacker, defender, move_name, move_type, move_category, context
    ) {
        return modifier;
    }
    
    if let Some(modifier) = berry_items::get_berry_item_effect(
        item_name, generation, attacker, defender, move_name, move_type, move_category, context
    ) {
        return modifier;
    }
    
    if let Some(modifier) = status_items::get_status_item_effect(
        item_name, generation, attacker, defender, move_name, move_type, move_category, context
    ) {
        return modifier;
    }
    
    if let Some(modifier) = utility_items::get_utility_item_effect(
        item_name, generation, attacker, defender, move_name, move_type, move_category, context
    ) {
        return modifier;
    }
    
    if let Some(modifier) = species_items::get_species_item_effect(
        item_name, generation, attacker, defender, move_name, move_type, move_category, context
    ) {
        return modifier;
    }
    
    // No item effect found
    ItemModifier::default()
}

/// Check if an item provides HP restore per turn (for end-of-turn processing)
pub fn get_item_hp_restore_per_turn(
    item_name: &str,
    pokemon: &Pokemon,
    position: BattlePosition,
    generation: &dyn GenerationBattleMechanics,
) -> BattleInstructions {
    if let Some(instructions) = status_items::get_item_hp_restore_per_turn(item_name, pokemon, position, generation) {
        return instructions;
    }
    
    if let Some(instructions) = utility_items::get_item_hp_restore_per_turn(item_name, pokemon, position, generation) {
        return instructions;
    }
    
    BattleInstructions::new(100.0, vec![])
}

/// Check for item effects that trigger on switch-in
pub fn get_item_on_switch_in_effects(
    item_name: &str,
    pokemon: &Pokemon,
    position: BattlePosition,
    generation: &dyn GenerationBattleMechanics,
) -> BattleInstructions {
    // Check all categories for switch-in effects
    for get_effects in [
        stat_boosting_items::get_item_on_switch_in_effects,
        utility_items::get_item_on_switch_in_effects,
    ] {
        if let Some(instructions) = get_effects(item_name, pokemon, position, generation) {
            return instructions;
        }
    }
    
    BattleInstructions::new(100.0, vec![])
}


/// Calculate all item modifiers for a given context
pub fn calculate_item_modifiers(
    attacker_item: Option<&str>,
    defender_item: Option<&str>,
    generation: &dyn GenerationBattleMechanics,
    attacker: &Pokemon,
    defender: Option<&Pokemon>,
    move_name: &str,
    move_type: &str,
    move_category: MoveCategory,
    context: &DamageContext,
) -> (ItemModifier, ItemModifier) {
    let attacker_modifier = if let Some(item_name) = attacker_item {
        get_item_by_name_with_generation(
            item_name,
            generation,
            attacker,
            defender,
            move_name,
            move_type,
            move_category,
            context,
        )
    } else {
        ItemModifier::default()
    };
    
    let defender_modifier = if let Some(item_name) = defender_item {
        if let Some(def_pokemon) = defender {
            get_item_by_name_with_generation(
                item_name,
                generation,
                def_pokemon,
                Some(attacker),
                move_name,
                move_type,
                move_category,
                context,
            )
        } else {
            ItemModifier::default()
        }
    } else {
        ItemModifier::default()
    };
    
    (attacker_modifier, defender_modifier)
}

/// Apply Expert Belt boost if move is super effective
pub fn apply_expert_belt_boost(context: &DamageContext, type_effectiveness: f32, generation: u8) -> f32 {
    // Check if attacker has Expert Belt
    if let Some(item) = &context.attacker.pokemon.item {
        if item.to_lowercase().replace(&[' ', '-'][..], "") == "expertbelt" && type_effectiveness > 1.0 {
            return 1.2; // Expert Belt provides 1.2x boost to super effective moves
        }
    }
    1.0
}