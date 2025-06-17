//! # Battle State System
//! 
//! This module defines the core battle state representation for the V2 engine.
//! The state is format-aware and supports multiple active Pokemon per side.

use crate::battle_format::{BattleFormat, BattlePosition, SideReference};
use crate::instruction::{PokemonStatus, VolatileStatus, Weather, Terrain, SideCondition};
use crate::move_choice::MoveIndex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// The main battle state containing all information about the current battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// The battle format determining rules and active Pokemon count
    pub format: BattleFormat,
    /// Side one (player 1)
    pub side_one: BattleSide,
    /// Side two (player 2)
    pub side_two: BattleSide,
    /// Current weather condition
    pub weather: Weather,
    /// Weather duration (turns remaining)
    pub weather_turns_remaining: Option<u8>,
    /// Current terrain condition
    pub terrain: Terrain,
    /// Terrain duration (turns remaining)
    pub terrain_turns_remaining: Option<u8>,
    /// Current turn number
    pub turn: u32,
}

impl State {
    /// Create a new battle state with the specified format
    pub fn new(format: BattleFormat) -> Self {
        Self {
            format,
            side_one: BattleSide::new(),
            side_two: BattleSide::new(),
            weather: Weather::NONE,
            weather_turns_remaining: None,
            terrain: Terrain::NONE,
            terrain_turns_remaining: None,
            turn: 1,
        }
    }

    /// Get a reference to the specified side
    pub fn get_side(&self, side_ref: SideReference) -> &BattleSide {
        match side_ref {
            SideReference::SideOne => &self.side_one,
            SideReference::SideTwo => &self.side_two,
        }
    }

    /// Get a mutable reference to the specified side
    pub fn get_side_mut(&mut self, side_ref: SideReference) -> &mut BattleSide {
        match side_ref {
            SideReference::SideOne => &mut self.side_one,
            SideReference::SideTwo => &mut self.side_two,
        }
    }

    /// Get the Pokemon at the specified position
    pub fn get_pokemon_at_position(&self, position: BattlePosition) -> Option<&Pokemon> {
        let side = self.get_side(position.side);
        side.get_active_pokemon_at_slot(position.slot)
    }

    /// Get a mutable reference to the Pokemon at the specified position
    pub fn get_pokemon_at_position_mut(&mut self, position: BattlePosition) -> Option<&mut Pokemon> {
        let side = self.get_side_mut(position.side);
        side.get_active_pokemon_at_slot_mut(position.slot)
    }

    /// Check if a position is currently active (has a Pokemon)
    pub fn is_position_active(&self, position: BattlePosition) -> bool {
        position.is_valid_for_format(&self.format)
            && self.get_pokemon_at_position(position).is_some()
    }

    /// Get all active Pokemon positions
    pub fn get_active_positions(&self) -> Vec<BattlePosition> {
        let mut positions = Vec::new();
        for side_ref in [SideReference::SideOne, SideReference::SideTwo] {
            for slot in 0..self.format.active_pokemon_count() {
                let position = BattlePosition::new(side_ref, slot);
                if self.is_position_active(position) {
                    positions.push(position);
                }
            }
        }
        positions
    }

    /// Get the generation mechanics for this battle
    pub fn get_generation_mechanics(&self) -> crate::generation::GenerationMechanics {
        self.format.generation.get_mechanics()
    }

    /// Get the generation for this battle
    pub fn get_generation(&self) -> crate::generation::Generation {
        self.format.generation
    }

    /// Check if a generation feature is available in this battle
    pub fn has_generation_feature(&self, feature: crate::generation::GenerationFeature) -> bool {
        use crate::generation::GenerationBattleMechanics;
        self.get_generation_mechanics().has_feature(feature)
    }

    /// Check if the battle is over
    pub fn is_battle_over(&self) -> bool {
        self.side_one.is_defeated() || self.side_two.is_defeated()
    }

    /// Get the winner if the battle is over
    pub fn get_winner(&self) -> Option<SideReference> {
        if self.side_one.is_defeated() && !self.side_two.is_defeated() {
            Some(SideReference::SideTwo)
        } else if self.side_two.is_defeated() && !self.side_one.is_defeated() {
            Some(SideReference::SideOne)
        } else {
            None
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(BattleFormat::default())
    }
}

/// Represents one side of the battle (a player's team and active Pokemon)
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
}

impl BattleSide {
    /// Create a new battle side
    pub fn new() -> Self {
        Self {
            pokemon: Vec::new(),
            active_pokemon_indices: vec![None; 3], // Max 3 for triples, unused slots ignored
            side_conditions: HashMap::new(),
            side_volatile_statuses: HashSet::new(),
        }
    }

    /// Get the active Pokemon at the specified slot
    pub fn get_active_pokemon_at_slot(&self, slot: usize) -> Option<&Pokemon> {
        if let Some(Some(index)) = self.active_pokemon_indices.get(slot) {
            self.pokemon.get(*index)
        } else {
            None
        }
    }

    /// Get a mutable reference to the active Pokemon at the specified slot
    pub fn get_active_pokemon_at_slot_mut(&mut self, slot: usize) -> Option<&mut Pokemon> {
        if let Some(Some(index)) = self.active_pokemon_indices.get(slot) {
            self.pokemon.get_mut(*index)
        } else {
            None
        }
    }

    /// Set the active Pokemon at the specified slot
    pub fn set_active_pokemon_at_slot(&mut self, slot: usize, pokemon_index: Option<usize>) {
        if slot < self.active_pokemon_indices.len() {
            self.active_pokemon_indices[slot] = pokemon_index;
        }
    }

    /// Get all active Pokemon
    pub fn get_active_pokemon(&self) -> Vec<&Pokemon> {
        self.active_pokemon_indices
            .iter()
            .filter_map(|index| index.and_then(|i| self.pokemon.get(i)))
            .collect()
    }

    /// Check if this side is defeated (no active Pokemon with HP > 0)
    pub fn is_defeated(&self) -> bool {
        self.get_active_pokemon()
            .iter()
            .all(|pokemon| pokemon.hp == 0)
    }

    /// Add a Pokemon to this side's team
    pub fn add_pokemon(&mut self, pokemon: Pokemon) {
        self.pokemon.push(pokemon);
    }

    /// Get the number of Pokemon on this side
    pub fn pokemon_count(&self) -> usize {
        self.pokemon.len()
    }
}

impl Default for BattleSide {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a Pokemon in battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pokemon {
    /// Pokemon species name/ID
    pub species: String,
    /// Current HP
    pub hp: i16,
    /// Maximum HP
    pub max_hp: i16,
    /// Base stats
    pub stats: PokemonStats,
    /// Current stat boosts (-6 to +6)
    pub stat_boosts: HashMap<crate::instruction::Stat, i8>,
    /// Current status condition
    pub status: PokemonStatus,
    /// Status duration (for sleep/freeze)
    pub status_duration: Option<u8>,
    /// Volatile statuses
    pub volatile_statuses: HashSet<VolatileStatus>,
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
    /// Tera type (if Terastallized)
    #[cfg(feature = "terastallization")]
    pub tera_type: Option<crate::move_choice::PokemonType>,
    /// Whether this Pokemon is Terastallized
    #[cfg(feature = "terastallization")]
    pub is_terastallized: bool,
}

impl Pokemon {
    /// Create a new Pokemon with default values
    pub fn new(species: String) -> Self {
        Self {
            species,
            hp: 100,
            max_hp: 100,
            stats: PokemonStats::default(),
            stat_boosts: HashMap::new(),
            status: PokemonStatus::NONE,
            status_duration: None,
            volatile_statuses: HashSet::new(),
            substitute_health: 0,
            moves: HashMap::new(),
            ability: String::new(),
            item: None,
            types: vec!["Normal".to_string()],
            level: 50,
            gender: Gender::Unknown,
            #[cfg(feature = "terastallization")]
            tera_type: None,
            #[cfg(feature = "terastallization")]
            is_terastallized: false,
        }
    }

    /// Check if this Pokemon is fainted (HP = 0)
    pub fn is_fainted(&self) -> bool {
        self.hp == 0
    }

    /// Get the effective stat value including boosts
    pub fn get_effective_stat(&self, stat: crate::instruction::Stat) -> i16 {
        let base_value = match stat {
            crate::instruction::Stat::Hp => return self.max_hp, // HP doesn't get boosted
            crate::instruction::Stat::Attack => self.stats.attack,
            crate::instruction::Stat::Defense => self.stats.defense,
            crate::instruction::Stat::SpecialAttack => self.stats.special_attack,
            crate::instruction::Stat::SpecialDefense => self.stats.special_defense,
            crate::instruction::Stat::Speed => self.stats.speed,
            crate::instruction::Stat::Accuracy => 100, // Base accuracy
            crate::instruction::Stat::Evasion => 100,   // Base evasion
        };

        let boost = self.stat_boosts.get(&stat).copied().unwrap_or(0);
        let multiplier = if boost >= 0 {
            (2 + boost as i16) as f32 / 2.0
        } else {
            2.0 / (2 - boost as i16) as f32
        };

        (base_value as f32 * multiplier) as i16
    }

    /// Add a move to this Pokemon
    pub fn add_move(&mut self, index: MoveIndex, move_data: Move) {
        self.moves.insert(index, move_data);
    }

    /// Get a move by index
    pub fn get_move(&self, index: MoveIndex) -> Option<&Move> {
        self.moves.get(&index)
    }
}

/// Pokemon base stats
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PokemonStats {
    pub attack: i16,
    pub defense: i16,
    pub special_attack: i16,
    pub special_defense: i16,
    pub speed: i16,
}

impl Default for PokemonStats {
    fn default() -> Self {
        Self {
            attack: 100,
            defense: 100,
            special_attack: 100,
            special_defense: 100,
            speed: 100,
        }
    }
}

/// Pokemon gender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
    Unknown,
}

/// Side-wide volatile statuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SideVolatileStatus {
    TailWind,
    WideGuard,
    QuickGuard,
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
    pub target: crate::data::ps_types::PSMoveTarget,
    /// Move category
    pub category: MoveCategory,
    /// Move priority
    pub priority: i8,
}

impl Move {
    /// Create a new move with default values
    pub fn new(name: String) -> Self {
        Self {
            name,
            base_power: 80,
            accuracy: 100,
            move_type: "Normal".to_string(),
            pp: 20,
            max_pp: 20,
            target: crate::data::ps_types::PSMoveTarget::Normal,
            category: MoveCategory::Physical,
            priority: 0,
        }
    }

    /// Create a new move with specific properties
    pub fn new_with_details(
        name: String,
        base_power: u8,
        accuracy: u8,
        move_type: String,
        pp: u8,
        target: crate::data::ps_types::PSMoveTarget,
        category: MoveCategory,
        priority: i8,
    ) -> Self {
        Self {
            name,
            base_power,
            accuracy,
            move_type,
            pp,
            max_pp: pp,
            target,
            category,
            priority,
        }
    }

    /// Check if this move is a status move (no base power)
    pub fn is_status_move(&self) -> bool {
        self.base_power == 0 || matches!(self.category, MoveCategory::Status)
    }

    /// Check if this move is a spread move
    pub fn is_spread_move(&self) -> bool {
        self.target.is_spread_move()
    }

    /// Check if this move can affect allies
    pub fn affects_allies(&self) -> bool {
        self.target.affects_allies()
    }

    /// Check if this move has PP remaining
    pub fn has_pp(&self) -> bool {
        self.pp > 0
    }

    /// Use PP for this move
    pub fn use_pp(&mut self) -> bool {
        if self.pp > 0 {
            self.pp -= 1;
            true
        } else {
            false
        }
    }

    /// Check if this move is a damaging move
    pub fn is_damaging(&self) -> bool {
        matches!(self.category, MoveCategory::Physical | MoveCategory::Special) && self.base_power > 0
    }
}


/// Move categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

// Missing types from V1 needed for choices.rs compatibility
use crate::define_enum_with_from_str;

// V1 Pokemon volatile status enum
define_enum_with_from_str! {
    #[repr(u8)]
    #[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
    PokemonVolatileStatus {
        NONE,
        AQUARING,
        ATTRACT,
        AUTOTOMIZE,
        BANEFULBUNKER,
        BIDE,
        BOUNCE,
        BURNINGBULWARK,
        CHARGE,
        CONFUSION,
        CURSE,
        DEFENSECURL,
        DESTINYBOND,
        DIG,
        DISABLE,
        DIVE,
        ELECTRIFY,
        ELECTROSHOT,
        EMBARGO,
        ENCORE,
        ENDURE,
        FLASHFIRE,
        FLINCH,
        FLY,
        FOCUSENERGY,
        FOLLOWME,
        FORESIGHT,
        FREEZESHOCK,
        GASTROACID,
        GEOMANCY,
        GLAIVERUSH,
        GRUDGE,
        HEALBLOCK,
        HELPINGHAND,
        ICEBURN,
        IMPRISON,
        INGRAIN,
        KINGSSHIELD,
        LASERFOCUS,
        LEECHSEED,
        LIGHTSCREEN,
        LOCKEDMOVE,
        MAGICCOAT,
        MAGNETRISE,
        MAXGUARD,
        METEORBEAM,
        MINIMIZE,
        MIRACLEEYE,
        MUSTRECHARGE,
        NIGHTMARE,
        NORETREAT,
        OCTOLOCK,
        PARTIALLYTRAPPED,
        PERISH4,
        PERISH3,
        PERISH2,
        PERISH1,
        PHANTOMFORCE,
        POWDER,
        POWERSHIFT,
        POWERTRICK,
        PROTECT,
        PROTOSYNTHESISATK,
        PROTOSYNTHESISDEF,
        PROTOSYNTHESISSPA,
        PROTOSYNTHESISSPD,
        PROTOSYNTHESISSPE,
        QUARKDRIVEATK,
        QUARKDRIVEDEF,
        QUARKDRIVESPA,
        QUARKDRIVESPD,
        QUARKDRIVESPE,
        RAGE,
        RAGEPOWDER,
        RAZORWIND,
        REFLECT,
        ROOST,
        SALTCURE,
        SHADOWFORCE,
        SKULLBASH,
        SKYATTACK,
        SKYDROP,
        SILKTRAP,
        SLOWSTART,
        SMACKDOWN,
        SNATCH,
        SOLARBEAM,
        SOLARBLADE,
        SPARKLINGARIA,
        SPIKYSHIELD,
        SPOTLIGHT,
        STOCKPILE,
        SUBSTITUTE,
        SYRUPBOMB,
        TARSHOT,
        TAUNT,
        TELEKINESIS,
        THROATCHOP,
        TRUANT,
        TORMENT,
        TYPECHANGE,
        UNBURDEN,
        UPROAR,
        YAWN,
    },
    default = NONE
}

// V1 Pokemon side condition enum
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum PokemonSideCondition {
    AuroraVeil,
    CraftyShield,
    HealingWish,
    LightScreen,
    LuckyChant,
    LunarDance,
    MatBlock,
    Mist,
    Protect,
    QuickGuard,
    Reflect,
    Safeguard,
    Spikes,
    Stealthrock,
    StickyWeb,
    Tailwind,
    ToxicCount,
    ToxicSpikes,
    WideGuard,
}

// V1 Pokemon move index enum
#[derive(Debug, Copy, PartialEq, Clone, Eq, Hash)]
pub enum PokemonMoveIndex {
    M0,
    M1,
    M2,
    M3,
}

// V1 Pokemon boostable stat enum
#[derive(Debug, Copy, PartialEq, Clone, Eq, Hash)]
pub enum PokemonBoostableStat {
    Attack,
    Defense,
    SpecialAttack,
    SpecialDefense,
    Speed,
    Accuracy,
    Evasion,
}

// V1 Pokemon type enum  
#[derive(Debug, Copy, PartialEq, Clone, Eq, Hash)]
pub enum PokemonType {
    NORMAL,
    FIGHTING,
    FLYING,
    POISON,
    GROUND,
    ROCK,
    BUG,
    GHOST,
    STEEL,
    FIRE,
    WATER,
    GRASS,
    ELECTRIC,
    PSYCHIC,
    ICE,
    DRAGON,
    DARK,
    FAIRY,
    TYPELESS,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle_format::{BattleFormat, FormatType};
    use crate::generation::Generation;

    #[test]
    fn test_state_creation() {
        let state = State::new(BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        assert_eq!(state.format, BattleFormat::new("Doubles".to_string(), Generation::Gen9, FormatType::Doubles));
        assert_eq!(state.turn, 1);
        assert_eq!(state.weather, Weather::NONE);
    }

    #[test]
    fn test_battle_side_active_pokemon() {
        let mut side = BattleSide::new();
        let pokemon = Pokemon::new("Pikachu".to_string());
        
        side.add_pokemon(pokemon);
        side.set_active_pokemon_at_slot(0, Some(0));

        assert!(side.get_active_pokemon_at_slot(0).is_some());
        assert!(side.get_active_pokemon_at_slot(1).is_none());
        assert_eq!(side.get_active_pokemon().len(), 1);
    }

    #[test]
    fn test_pokemon_stats() {
        let mut pokemon = Pokemon::new("Pikachu".to_string());
        pokemon.stats.attack = 120;
        
        // No boost
        assert_eq!(pokemon.get_effective_stat(crate::instruction::Stat::Attack), 120);
        
        // +1 boost
        pokemon.stat_boosts.insert(crate::instruction::Stat::Attack, 1);
        assert_eq!(pokemon.get_effective_stat(crate::instruction::Stat::Attack), 180);
        
        // -1 boost
        pokemon.stat_boosts.insert(crate::instruction::Stat::Attack, -1);
        assert_eq!(pokemon.get_effective_stat(crate::instruction::Stat::Attack), 80);
    }

    #[test]
    fn test_position_validity() {
        let state = State::new(BattleFormat::new("Singles".to_string(), Generation::Gen9, FormatType::Singles));
        let pos_valid = BattlePosition::new(SideReference::SideOne, 0);
        let pos_invalid = BattlePosition::new(SideReference::SideOne, 1);

        assert!(pos_valid.is_valid_for_format(&state.format));
        assert!(!pos_invalid.is_valid_for_format(&state.format));
    }
}