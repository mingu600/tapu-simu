//! Pokemon data structures and state management

use serde::{Deserialize, Serialize};
use crate::types::*;
use std::collections::HashMap;

/// A Pokemon's current battle state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    /// Species data
    pub species: SpeciesData,
    
    /// Current HP
    pub hp: u16,
    
    /// Maximum HP
    pub max_hp: u16,
    
    /// Current stats (including stat changes from items/abilities)
    pub stats: StatsTable,
    
    /// Stat boosts (-6 to +6)
    pub boosts: BoostsTable,
    
    /// Major status condition (burn, sleep, etc.)
    pub status: Option<StatusCondition>,
    
    /// Volatile status effects (confusion, substitute, etc.)
    pub volatiles: HashMap<String, VolatileStatus>,
    
    /// Move slots
    pub moves: [MoveSlot; 4],
    
    /// Current ability
    pub ability: AbilityData,
    
    /// Held item
    pub item: Option<ItemData>,
    
    /// Current types (can change due to moves/abilities)
    pub types: [Type; 2],
    
    /// Level
    pub level: u8,
    
    /// Gender
    pub gender: Gender,
    
    /// Nature
    pub nature: Nature,
    
    /// Individual Values
    pub ivs: StatsTable,
    
    /// Effort Values
    pub evs: StatsTable,
    
    /// Position on the field (0, 1, 2 for triples)
    pub position: usize,
    
    /// Fainted this turn
    pub fainted: bool,
    
    /// Whether this Pokemon can be switched out
    pub trapped: bool,
    
    /// Last move used
    pub last_move: Option<String>,
    
    /// Last move target location
    pub last_move_target_loc: Option<i8>,
    
    /// Turn when this Pokemon switched in
    pub switch_in_turn: u32,
}

/// Pokemon species data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesData {
    pub id: String,
    pub name: String,
    pub types: [Type; 2],
    pub base_stats: StatsTable,
    pub abilities: Vec<String>,
    pub height: f32,   // meters
    pub weight: f32,   // kg
    pub gender_ratio: GenderRatio,
}

/// Gender ratio for a species
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenderRatio {
    Genderless,
    MaleOnly,
    FemaleOnly,
    Ratio { male: f32, female: f32 },
}

/// A move slot with PP tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveSlot {
    pub move_data: MoveData,
    pub pp: u8,
    pub max_pp: u8,
    pub disabled: bool,
}

/// Move data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveData {
    pub id: String,
    pub name: String,
    pub type_: Type,
    pub category: MoveCategory,
    pub base_power: u16,
    pub accuracy: Option<u8>, // None = always hits
    pub pp: u8,
    pub target: MoveTarget,
    pub priority: i8,
    pub flags: MoveFlags,
    pub secondary_effect: Option<SecondaryEffect>,
    pub crit_ratio: u8,
    pub multihit: Option<MultihitData>,
    pub drain: Option<[u8; 2]>,  // [numerator, denominator]
    pub recoil: Option<[u8; 2]>, // [numerator, denominator]
}

/// Move flags from Pokemon Showdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFlags {
    pub contact: bool,
    pub sound: bool,
    pub bullet: bool,
    pub pulse: bool,
    pub bite: bool,
    pub punch: bool,
    pub powder: bool,
    pub reflectable: bool,
    pub charge: bool,
    pub recharge: bool,
    pub gravity: bool,
    pub defrost: bool,
    pub distance: bool,
    pub heal: bool,
    pub authentic: bool,
}

impl Default for MoveFlags {
    fn default() -> Self {
        Self {
            contact: false,
            sound: false,
            bullet: false,
            pulse: false,
            bite: false,
            punch: false,
            powder: false,
            reflectable: false,
            charge: false,
            recharge: false,
            gravity: false,
            defrost: false,
            distance: false,
            heal: false,
            authentic: false,
        }
    }
}

/// Secondary effect data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryEffect {
    pub chance: u8, // Percentage chance
    pub effect: SecondaryEffectType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecondaryEffectType {
    Status(StatusCondition),
    StatBoost { stat: String, amount: i8 },
    Flinch,
    Burn,
    Freeze,
    Paralyze,
    Poison,
    BadlyPoison,
    Sleep,
}

/// Multi-hit move data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultihitData {
    pub min_hits: u8,
    pub max_hits: u8,
}

/// Ability data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityData {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// Item data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub natural_gift: Option<NaturalGiftData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalGiftData {
    pub type_: Type,
    pub base_power: u16,
}

/// Major status conditions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusCondition {
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadPoison,
    Sleep,
}

/// Volatile status effects (temporary conditions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatileStatus {
    pub id: String,
    pub duration: Option<u8>,
    pub data: HashMap<String, serde_json::Value>,
}

impl Pokemon {
    /// Create a new Pokemon from team data
    pub fn new(
        species: SpeciesData,
        level: u8,
        moves: [MoveData; 4],
        ability: AbilityData,
        item: Option<ItemData>,
        nature: Nature,
        ivs: StatsTable,
        evs: StatsTable,
        gender: Gender,
    ) -> Self {
        let stats = Self::calculate_stats(&species.base_stats, level, nature, ivs, evs);
        let max_hp = stats.hp;
        
        let move_slots = moves.map(|move_data| MoveSlot {
            pp: move_data.pp,
            max_pp: move_data.pp,
            disabled: false,
            move_data,
        });
        
        Self {
            species: species.clone(),
            hp: max_hp,
            max_hp,
            stats,
            boosts: BoostsTable::default(),
            status: None,
            volatiles: HashMap::new(),
            moves: move_slots,
            ability,
            item,
            types: species.types,
            level,
            gender,
            nature,
            ivs,
            evs,
            position: 0,
            fainted: false,
            trapped: false,
            last_move: None,
            last_move_target_loc: None,
            switch_in_turn: 0,
        }
    }
    
    /// Calculate stats from base stats, level, nature, IVs, and EVs
    pub fn calculate_stats(
        base: &StatsTable,
        level: u8,
        nature: Nature,
        ivs: StatsTable,
        evs: StatsTable,
    ) -> StatsTable {
        let level = level as u32;
        
        // HP calculation: ((Base + IV + EV/4) * 2 * Level / 100) + Level + 10
        let hp = if base.hp == 1 {
            // Shedinja special case
            1
        } else {
            ((base.hp as u32 + ivs.hp as u32 + evs.hp as u32 / 4) * 2 * level / 100 + level + 10) as u16
        };
        
        // Other stats: ((Base + IV + EV/4) * 2 * Level / 100 + 5) * Nature
        let attack = Self::calc_stat(base.attack, ivs.attack, evs.attack, level, nature, StatType::Attack);
        let defense = Self::calc_stat(base.defense, ivs.defense, evs.defense, level, nature, StatType::Defense);
        let special_attack = Self::calc_stat(base.special_attack, ivs.special_attack, evs.special_attack, level, nature, StatType::SpecialAttack);
        let special_defense = Self::calc_stat(base.special_defense, ivs.special_defense, evs.special_defense, level, nature, StatType::SpecialDefense);
        let speed = Self::calc_stat(base.speed, ivs.speed, evs.speed, level, nature, StatType::Speed);
        
        StatsTable {
            hp,
            attack,
            defense,
            special_attack,
            special_defense,
            speed,
        }
    }
    
    fn calc_stat(base: u16, iv: u16, ev: u16, level: u32, nature: Nature, stat_type: StatType) -> u16 {
        let base_calc = ((base as u32 + iv as u32 + ev as u32 / 4) * 2 * level / 100 + 5) as f32;
        let nature_mod = nature.modifier(stat_type);
        (base_calc * nature_mod) as u16
    }
    
    /// Check if this Pokemon has fainted
    pub fn is_fainted(&self) -> bool {
        self.hp == 0 || self.fainted
    }
    
    /// Apply damage to this Pokemon
    pub fn take_damage(&mut self, damage: u16) {
        self.hp = self.hp.saturating_sub(damage);
        if self.hp == 0 {
            self.fainted = true;
        }
    }
    
    /// Heal this Pokemon
    pub fn heal(&mut self, amount: u16) {
        self.hp = (self.hp + amount).min(self.max_hp);
        if self.hp > 0 {
            self.fainted = false;
        }
    }
    
    /// Get effective stat considering boosts
    pub fn effective_stat(&self, stat_type: StatType) -> u16 {
        let base_stat = match stat_type {
            StatType::Attack => self.stats.attack,
            StatType::Defense => self.stats.defense,
            StatType::SpecialAttack => self.stats.special_attack,
            StatType::SpecialDefense => self.stats.special_defense,
            StatType::Speed => self.stats.speed,
        };
        
        let boost = match stat_type {
            StatType::Attack => self.boosts.attack,
            StatType::Defense => self.boosts.defense,
            StatType::SpecialAttack => self.boosts.special_attack,
            StatType::SpecialDefense => self.boosts.special_defense,
            StatType::Speed => self.boosts.speed,
        };
        
        Self::apply_boost(base_stat, boost)
    }
    
    /// Apply stat boost multiplier
    fn apply_boost(base_stat: u16, boost: i8) -> u16 {
        let multiplier = if boost >= 0 {
            (2.0 + boost as f32) / 2.0
        } else {
            2.0 / (2.0 - boost as f32)
        };
        
        (base_stat as f32 * multiplier) as u16
    }
    
    /// Check if this Pokemon can use a move
    pub fn can_use_move(&self, move_index: usize) -> bool {
        if move_index >= 4 {
            return false;
        }
        
        let move_slot = &self.moves[move_index];
        !move_slot.disabled && move_slot.pp > 0
    }
    
    /// Use PP for a move
    pub fn use_pp(&mut self, move_index: usize) {
        if move_index < 4 {
            self.moves[move_index].pp = self.moves[move_index].pp.saturating_sub(1);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StatType {
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
}

impl Nature {
    /// Get the stat modifier for this nature
    fn modifier(&self, stat: StatType) -> f32 {
        use Nature::*;
        use StatType::*;
        
        match (self, stat) {
            // Increased Attack
            (Lonely | Brave | Adamant | Naughty, Attack) => 1.1,
            // Decreased Attack
            (Bold | Timid | Modest | Calm, Attack) => 0.9,
            
            // Increased Defense
            (Bold | Relaxed | Impish | Lax, Defense) => 1.1,
            // Decreased Defense
            (Lonely | Hasty | Mild | Gentle, Defense) => 0.9,
            
            // Increased Special Attack
            (Modest | Mild | Quiet | Rash, SpecialAttack) => 1.1,
            // Decreased Special Attack
            (Adamant | Impish | Jolly | Careful, SpecialAttack) => 0.9,
            
            // Increased Special Defense
            (Calm | Gentle | Sassy | Careful, SpecialDefense) => 1.1,
            // Decreased Special Defense
            (Naughty | Lax | Naive | Rash, SpecialDefense) => 0.9,
            
            // Increased Speed
            (Timid | Hasty | Jolly | Naive, Speed) => 1.1,
            // Decreased Speed
            (Brave | Relaxed | Quiet | Sassy, Speed) => 0.9,
            
            // Neutral natures
            _ => 1.0,
        }
    }
}