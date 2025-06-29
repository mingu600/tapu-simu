//! Pokemon-related types and implementations for battle state

use crate::core::battle_format::BattlePosition;
use crate::core::instructions::{MoveCategory, PokemonStatus};
use crate::core::move_choice::MoveIndex;
use crate::data::types::Stats;
use crate::types::{PokemonType, PokemonName, Abilities, Items, Moves, StatBoostArray, VolatileStatusStorage};
use crate::types::from_string::FromNormalizedString;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Pokemon gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

impl FromNormalizedString for Gender {
    fn from_normalized_str(s: &str) -> Option<Self> {
        match s {
            "m" | "male" => Some(Gender::Male),
            "f" | "female" => Some(Gender::Female),
            "n" | "genderless" | "unknown" => Some(Gender::Unknown),
            _ => None,
        }
    }
    
    fn valid_strings() -> Vec<&'static str> {
        vec!["m", "male", "f", "female", "n", "genderless", "unknown"]
    }
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
    pub name: Moves,
    /// Base power (0 for status moves)
    pub base_power: u8,
    /// Accuracy (1-100, 0 for never-miss moves)
    pub accuracy: u8,
    /// Move type
    pub move_type: PokemonType,
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
    pub fn new(name: crate::types::Moves) -> Self {
        Self {
            name,
            base_power: 60,
            accuracy: 100,
            move_type: PokemonType::Normal,
            pp: 15,
            max_pp: 15,
            target: crate::data::showdown_types::MoveTarget::Normal,
            category: MoveCategory::Physical,
            priority: 0,
        }
    }

    /// Get the move's type
    pub fn get_type(&self) -> PokemonType {
        self.move_type
    }

    /// Get the move's name
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Create a new move with detailed parameters
    pub fn new_with_details(
        name: crate::types::Moves,
        base_power: u8,
        accuracy: u8,
        move_type: PokemonType,
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
    pub species: PokemonName,
    /// Current HP
    pub hp: i16,
    /// Maximum HP
    pub max_hp: i16,
    /// Effective stats (calculated battle-ready stats)
    pub stats: Stats,
    /// Base species stats (used for Gen 1 critical hit calculation)
    pub base_stats: Stats,
    /// Current stat boosts (-6 to +6)
    pub stat_boosts: StatBoostArray,
    /// Current status condition
    pub status: PokemonStatus,
    /// Status duration (for sleep/freeze)
    pub status_duration: Option<u8>,
    /// Volatile statuses with optimized storage
    pub volatile_statuses: VolatileStatusStorage,
    /// Substitute health (when Substitute volatile status is active)
    pub substitute_health: i16,
    /// Current moves (up to 4 moves, stored efficiently)
    pub moves: SmallVec<[(MoveIndex, Move); 4]>,
    /// Current ability
    pub ability: Abilities,
    /// Held item
    pub item: Option<Items>,
    /// Types (can change due to moves like Soak)
    pub types: Vec<PokemonType>,
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
    /// Current forme (for Pokemon with multiple formes)
    pub forme: Option<String>,
    /// Last used move (for moves like Disable, Encore)
    pub last_used_move: Option<Moves>,
    /// Whether this Pokemon must switch out (forced by items/abilities)
    pub must_switch: bool,
    /// Sleep turns remaining (for natural sleep mechanics)
    pub sleep_turns: Option<u8>,
    /// Rest turns remaining (for Rest move mechanics)
    pub rest_turns: Option<u8>,
    /// Disabled moves with their remaining turns
    pub disabled_moves: std::collections::HashMap<MoveIndex, u8>,
    /// Volatile status durations
    pub volatile_status_durations: std::collections::HashMap<crate::types::VolatileStatus, u8>,
}

impl Pokemon {
    /// Create a new Pokemon with default values
    pub fn new(species: crate::types::PokemonName) -> Self {
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
            base_stats: Stats {
                hp: 100,
                attack: 100,
                defense: 100,
                special_attack: 100,
                special_defense: 100,
                speed: 100,
            },
            stat_boosts: StatBoostArray::default(),
            status: PokemonStatus::None,
            status_duration: None,
            volatile_statuses: VolatileStatusStorage::default(),
            substitute_health: 0,
            moves: SmallVec::new(),
            ability: crate::types::Abilities::NONE,
            item: None,
            types: vec![PokemonType::Normal],
            level: 50,
            gender: Gender::Unknown,
            tera_type: None,
            is_terastallized: false,
            ability_suppressed: false,
            ability_triggered_this_turn: false,
            item_consumed: false,
            weight_kg: 50.0, // Default weight for unknown Pokemon
            forme: None,
            last_used_move: None,
            must_switch: false,
            sleep_turns: None,
            rest_turns: None,
            disabled_moves: std::collections::HashMap::new(),
            volatile_status_durations: std::collections::HashMap::new(),
        }
    }

    /// Get a specific move from Pokemon's moveset
    pub fn get_move(&self, move_index: MoveIndex) -> Option<&Move> {
        self.moves.iter()
            .find(|(idx, _)| *idx == move_index)
            .map(|(_, m)| m)
    }

    /// Get mutable reference to a move
    pub fn get_move_mut(&mut self, move_index: MoveIndex) -> Option<&mut Move> {
        self.moves.iter_mut()
            .find(|(idx, _)| *idx == move_index)
            .map(|(_, m)| m)
    }

    /// Check if the Pokemon is fainted
    pub fn is_fainted(&self) -> bool {
        self.hp <= 0
    }

    /// Get effective stat after boosts, items, abilities, etc.
    pub fn get_effective_stat(&self, stat: crate::core::instructions::Stat) -> f64 {
        let base_stat = match stat {
            crate::core::instructions::Stat::Hp => self.stats.hp as f64,
            crate::core::instructions::Stat::Attack => self.stats.attack as f64,
            crate::core::instructions::Stat::Defense => self.stats.defense as f64,
            crate::core::instructions::Stat::SpecialAttack => self.stats.special_attack as f64,
            crate::core::instructions::Stat::SpecialDefense => self.stats.special_defense as f64,
            crate::core::instructions::Stat::Speed => self.stats.speed as f64,
            crate::core::instructions::Stat::Accuracy => 100.0, // Base accuracy
            crate::core::instructions::Stat::Evasion => 100.0,  // Base evasion
        };

        // Apply stat boosts
        let boost = self.stat_boosts.get_direct(stat);
        let boost_multiplier = if boost >= 0 {
            (2.0 + boost as f64) / 2.0
        } else {
            2.0 / (2.0 - boost as f64)
        };

        base_stat * boost_multiplier
    }

    /// Get effective speed with battle context for comprehensive speed calculation
    pub fn get_effective_speed(
        &self,
        battle_state: &crate::core::battle_state::BattleState,
        position: BattlePosition,
    ) -> u16 {
        use crate::core::instructions::{PokemonStatus, Weather};
        
        let mut speed = self.get_effective_stat(crate::core::instructions::Stat::Speed) as u16;
        
        // Status modifiers
        if self.status == PokemonStatus::Paralysis {
            speed = (speed as f32 * 0.5) as u16;
        }
        
        // Weather modifiers (simplified - in real implementation check abilities)
        match battle_state.weather() {
            Weather::Sun => {
                if self.ability == crate::types::Abilities::CHLOROPHYLL {
                    speed *= 2;
                }
            }
            Weather::Rain => {
                if self.ability == crate::types::Abilities::SWIFTSWIM {
                    speed *= 2;
                }
            }
            Weather::Sandstorm => {
                if self.ability == crate::types::Abilities::SANDRUSH {
                    speed *= 2;
                }
            }
            Weather::Hail => {
                if self.ability == crate::types::Abilities::SLUSHRUSH {
                    speed *= 2;
                }
            }
            _ => {}
        }
        
        // Item modifiers (simplified examples)
        if let Some(ref item) = self.item {
            match *item {
                crate::types::Items::CHOICESCARF => speed = (speed as f32 * 1.5) as u16,
                crate::types::Items::QUICKCLAW => {}, // Handled separately with probability
                crate::types::Items::IRONBALL => speed = (speed as f32 * 0.5) as u16,
                crate::types::Items::MACHOBRACE => speed = (speed as f32 * 0.5) as u16,
                crate::types::Items::POWERWEIGHT | crate::types::Items::POWERBRACER | crate::types::Items::POWERBELT | crate::types::Items::POWERLENS | crate::types::Items::POWERBAND | crate::types::Items::POWERANKLET => {
                    speed = (speed as f32 * 0.5) as u16;
                }
                _ => {}
            }
        }
        
        // Ability modifiers (examples)
        match self.ability {
            crate::types::Abilities::QUICKFEET => {
                if self.status != PokemonStatus::None {
                    speed = (speed as f32 * 1.5) as u16;
                }
            }
            crate::types::Abilities::UNBURDEN => {
                if self.item_consumed {
                    speed *= 2;
                }
            }
            _ => {}
        }
        
        // Trick Room inversion
        if battle_state.is_trick_room_active() {
            speed = 10000_u16.saturating_sub(speed);
        }
        
        speed
    }

    /// Add a move to the Pokemon's moveset
    pub fn add_move(&mut self, move_index: MoveIndex, move_data: Move) {
        // Remove existing move with same index if it exists
        self.moves.retain(|(idx, _)| *idx != move_index);
        // Add the new move
        self.moves.push((move_index, move_data));
    }
    
    /// Remove a move from the Pokemon's moveset
    pub fn remove_move(&mut self, move_index: MoveIndex) {
        self.moves.retain(|(idx, _)| *idx != move_index);
    }
    
    /// Get all move indices for this Pokemon
    pub fn get_move_indices(&self) -> Vec<MoveIndex> {
        self.moves.iter().map(|(idx, _)| *idx).collect()
    }
}

impl Default for Pokemon {
    fn default() -> Self {
        Self::new(crate::types::PokemonName::NONE)
    }
}