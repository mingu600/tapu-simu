//! # Pokemon State Components
//! 
//! This module contains all Pokemon-related structs, enums, and functions
//! that were extracted from battle_state.rs for better code organization.

use crate::core::battle_format::BattlePosition;
use crate::core::instructions::{MoveCategory, PokemonStatus, Stat, VolatileStatus};
use crate::core::move_choice::{MoveIndex, PokemonType};
use crate::data::types::Stats;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Pokemon gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

/// Information about damage taken this turn (for moves like Avalanche)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DamageInfo {
    /// Amount of damage taken
    pub damage: i16,
    /// Category of the move that dealt damage
    pub move_category: MoveCategory,
    /// Position of the attacker that dealt damage
    pub attacker_position: BattlePosition,
    /// Whether the damage was from a direct attack
    pub is_direct_damage: bool,
}

impl DamageInfo {
    /// Create new damage info
    pub fn new(
        damage: i16,
        move_category: MoveCategory,
        attacker_position: BattlePosition,
        is_direct_damage: bool,
    ) -> Self {
        Self {
            damage,
            move_category,
            attacker_position,
            is_direct_damage,
        }
    }
}

/// Represents a Pokemon's move in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    /// Move name/identifier
    pub name: String,
    /// Base power (0 for status moves)
    pub base_power: u8,
    /// Accuracy (1-100, 0 for never-miss moves)
    pub accuracy: u8,
    /// Move type
    pub move_type: String,
    /// Current PP
    pub pp: u8,
    /// Maximum PP
    pub max_pp: u8,
    /// Move target type (Pokemon Showdown format)
    pub target: crate::data::showdown_types::MoveTarget,
    /// Move category
    pub category: MoveCategory,
    /// Move priority
    pub priority: i8,
}

impl Move {
    pub fn new(name: String) -> Self {
        Self {
            name,
            base_power: 60,
            accuracy: 100,
            move_type: "Normal".to_string(),
            pp: 15,
            max_pp: 15,
            target: crate::data::showdown_types::MoveTarget::Normal,
            category: MoveCategory::Physical,
            priority: 0,
        }
    }

    /// Get the move's type
    pub fn get_type(&self) -> &str {
        &self.move_type
    }

    /// Get the move's name
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Create a new move with detailed parameters
    pub fn new_with_details(
        name: String,
        base_power: u8,
        accuracy: u8,
        move_type: String,
        pp: u8,
        max_pp: u8,
        target: crate::data::showdown_types::MoveTarget,
        category: MoveCategory,
        priority: i8,
    ) -> Self {
        Self {
            name,
            base_power,
            accuracy,
            move_type,
            pp,
            max_pp,
            target,
            category,
            priority,
        }
    }
}

/// Pokemon representation in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    /// Pokemon species name/ID
    pub species: String,
    /// Current HP
    pub hp: i16,
    /// Maximum HP
    pub max_hp: i16,
    /// Base stats
    pub stats: Stats,
    /// Current stat boosts (-6 to +6)
    pub stat_boosts: HashMap<Stat, i8>,
    /// Current status condition
    pub status: PokemonStatus,
    /// Status duration (for sleep/freeze)
    pub status_duration: Option<u8>,
    /// Volatile statuses
    pub volatile_statuses: HashSet<VolatileStatus>,
    /// Volatile status durations (turns remaining for each status)
    pub volatile_status_durations: HashMap<VolatileStatus, u8>,
    /// Substitute health (when Substitute volatile status is active)
    pub substitute_health: i16,
    /// Current moves
    pub moves: HashMap<MoveIndex, Move>,
    /// Current ability
    pub ability: String,
    /// Held item
    pub item: Option<String>,
    /// Types (can change due to moves like Soak)
    pub types: Vec<String>,
    /// Level
    pub level: u8,
    /// Gender
    pub gender: Gender,
    /// Tera type (if Terastallized) - Gen 9+ only
    pub tera_type: Option<PokemonType>,
    /// Whether this Pokemon is Terastallized - Gen 9+ only
    pub is_terastallized: bool,
    /// Whether the ability is suppressed (by moves like Gastro Acid)
    pub ability_suppressed: bool,
    /// Whether the ability has triggered this turn (for once-per-turn abilities)
    pub ability_triggered_this_turn: bool,
    /// Whether the held item has been consumed this battle
    pub item_consumed: bool,
    /// Weight in kilograms (for moves like Heavy Slam, Heat Crash)
    pub weight_kg: f32,
}

impl Pokemon {
    /// Create a new Pokemon with default values
    pub fn new(species: String) -> Self {
        Self {
            species,
            hp: 100,
            max_hp: 100,
            stats: Stats {
                hp: 100,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            stat_boosts: HashMap::new(),
            status: PokemonStatus::None,
            status_duration: None,
            volatile_statuses: HashSet::new(),
            volatile_status_durations: HashMap::new(),
            substitute_health: 0,
            moves: HashMap::new(),
            ability: String::new(),
            item: None,
            types: vec!["Normal".to_string()],
            level: 50,
            gender: Gender::Unknown,
            tera_type: None,
            is_terastallized: false,
            ability_suppressed: false,
            ability_triggered_this_turn: false,
            item_consumed: false,
            weight_kg: 50.0, // Default weight for unknown Pokemon
        }
    }

    /// Get a specific move from Pokemon's moveset
    pub fn get_move(&self, move_index: MoveIndex) -> Option<&Move> {
        self.moves.get(&move_index)
    }

    /// Get mutable reference to a move
    pub fn get_move_mut(&mut self, move_index: MoveIndex) -> Option<&mut Move> {
        self.moves.get_mut(&move_index)
    }

    /// Check if the Pokemon is fainted
    pub fn is_fainted(&self) -> bool {
        self.current_hp <= 0
    }

    /// Get effective stat after boosts, items, abilities, etc.
    pub fn get_effective_stat(&self, stat: Stat) -> f64 {
        let base_stat = match stat {
            Stat::Hp => self.stats.current_hp as f64,
            Stat::Attack => self.stats.attack as f64,
            Stat::Defense => self.stats.defense as f64,
            Stat::SpecialAttack => self.stats.special_attack as f64,
            Stat::SpecialDefense => self.stats.special_defense as f64,
            Stat::Speed => self.stats.speed as f64,
            Stat::Accuracy => 100.0, // Base accuracy
            Stat::Evasion => 100.0,  // Base evasion
        };

        // Apply stat boosts
        let boost = self.stat_boosts.get(&stat).copied().unwrap_or(0);
        let boost_multiplier = if boost >= 0 {
            (2.0 + boost as f64) / 2.0
        } else {
            2.0 / (2.0 - boost as f64)
        };

        base_stat * boost_multiplier
    }

    /// Add a move to the Pokemon's moveset
    pub fn add_move(&mut self, move_index: MoveIndex, move_data: Move) {
        self.moves.insert(move_index, move_data);
    }
}

impl Default for Pokemon {
    fn default() -> Self {
        Self::new("MissingNo".to_string())
    }
}