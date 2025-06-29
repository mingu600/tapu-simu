//! Side-related types and implementations for battle state

use crate::core::battle_format::BattlePosition;
use crate::core::instructions::{MoveCategory, SideCondition};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Re-import Pokemon from pokemon module for BattleSide
use super::pokemon::Pokemon;

/// Side-wide volatile statuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideVolatileStatus {
    TailWind,
    WideGuard,
    QuickGuard,
}

/// Tracks damage dealt to a side for counter moves
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageDealt {
    /// Amount of damage dealt
    pub damage: i16,
    /// Category of the move that dealt damage
    pub move_category: MoveCategory,
    /// Whether the damage hit a substitute
    pub hit_substitute: bool,
}

impl DamageDealt {
    /// Create a new DamageDealt with default values
    pub fn new() -> Self {
        Self {
            damage: 0,
            move_category: MoveCategory::Physical,
            hit_substitute: false,
        }
    }

    /// Reset damage tracking (called at start of turn)
    pub fn reset(&mut self) {
        self.damage = 0;
        self.move_category = MoveCategory::Physical;
        self.hit_substitute = false;
    }

    /// Set damage information
    pub fn set_damage(&mut self, damage: i16, move_category: MoveCategory, hit_substitute: bool) {
        self.damage = damage;
        self.move_category = move_category;
        self.hit_substitute = hit_substitute;
    }
}

impl Default for DamageDealt {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents one side of a battle (a player/trainer)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleSide {
    /// All Pokemon on this side's team
    pub pokemon: Vec<Pokemon>,
    /// Indices of currently active Pokemon
    pub active_pokemon_indices: Vec<Option<usize>>,
    /// Side conditions affecting this side
    pub side_conditions: HashMap<SideCondition, u8>,
    /// Volatile statuses that affect the entire side
    pub side_volatile_statuses: HashSet<SideVolatileStatus>,
    /// Wish healing scheduled for specific slots (heal_amount, turns_remaining)
    pub wish_healing: HashMap<usize, (i16, u8)>,
    /// Future Sight attacks scheduled for specific slots (attacker_position, damage_amount, turns_remaining, move_name)
    pub future_sight_attacks: HashMap<usize, (BattlePosition, i16, u8, String)>,
    /// Damage tracking for counter moves
    pub damage_dealt: DamageDealt,
    /// Whether Terastallization has been used this battle (Gen 9+ only)
    pub tera_used: bool,
    /// Future Sight attacks scheduled for specific slots (attacker_position, damage_amount, turns_remaining, move_name)
    pub future_sight: HashMap<usize, (crate::core::battle_format::BattlePosition, i16, u8, String)>,
    /// Last damage taken (for Counter/Mirror Coat)
    pub last_damage_taken: i16,
    /// Category of last move that dealt damage
    pub last_move_category: Option<MoveCategory>,
    /// Whether last damage hit a substitute
    pub last_hit_substitute: bool,
}

impl BattleSide {
    /// Create a new battle side
    pub fn new() -> Self {
        Self {
            pokemon: Vec::new(),
            active_pokemon_indices: vec![None; 3], // Max 3 for triples, unused slots ignored
            side_conditions: HashMap::new(),
            side_volatile_statuses: HashSet::new(),
            wish_healing: HashMap::new(),
            future_sight_attacks: HashMap::new(),
            damage_dealt: DamageDealt::new(),
            tera_used: false,
            future_sight: HashMap::new(),
            last_damage_taken: 0,
            last_move_category: None,
            last_hit_substitute: false,
        }
    }

    /// Add a Pokemon to this side's team
    pub fn add_pokemon(&mut self, pokemon: Pokemon) {
        self.pokemon.push(pokemon);
    }

    /// Set the active Pokemon at a specific slot
    pub fn set_active_pokemon_at_slot(&mut self, slot: usize, pokemon_index: Option<usize>) {
        if slot < self.active_pokemon_indices.len() {
            self.active_pokemon_indices[slot] = pokemon_index;
        }
    }

    /// Get the active Pokemon at a specific slot
    pub fn get_active_pokemon_at_slot(&self, slot: usize) -> Option<&Pokemon> {
        if let Some(Some(pokemon_index)) = self.active_pokemon_indices.get(slot) {
            self.pokemon.get(*pokemon_index)
        } else {
            None
        }
    }

    /// Get the active Pokemon at a specific slot (mutable)
    pub fn get_active_pokemon_at_slot_mut(&mut self, slot: usize) -> Option<&mut Pokemon> {
        if let Some(Some(pokemon_index)) = self.active_pokemon_indices.get(slot).copied() {
            self.pokemon.get_mut(pokemon_index)
        } else {
            None
        }
    }
}