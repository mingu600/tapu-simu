//! # Pokemon Showdown Data Types
//!
//! This module defines data types that match Pokemon Showdown's conventions,
//! enabling direct usage of PS data without transformation.

use crate::types::{Abilities, PokemonType, Moves, PokemonStatus, VolatileStatus, Terrain, Weather};
use crate::core::instructions::pokemon::MoveCategory;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;

// Custom deserializer functions for enum types
fn deserialize_move_category<'de, D>(deserializer: D) -> Result<MoveCategory, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Physical" => Ok(MoveCategory::Physical),
        "Special" => Ok(MoveCategory::Special),
        "Status" => Ok(MoveCategory::Status),
        _ => {
            eprintln!("Warning: Unknown move category '{}', treating as Status", s);
            Ok(MoveCategory::Status)
        }
    }
}

fn deserialize_move_target<'de, D>(deserializer: D) -> Result<MoveTarget, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "normal" => Ok(MoveTarget::Normal),
        "self" => Ok(MoveTarget::Self_),
        "adjacentAlly" => Ok(MoveTarget::AdjacentAlly),
        "adjacentAllyOrSelf" => Ok(MoveTarget::AdjacentAllyOrSelf),
        "adjacentFoe" => Ok(MoveTarget::AdjacentFoe),
        "allAdjacentFoes" => Ok(MoveTarget::AllAdjacentFoes),
        "allAdjacent" => Ok(MoveTarget::AllAdjacent),
        "allySide" => Ok(MoveTarget::AllySide),
        "allyTeam" => Ok(MoveTarget::AllyTeam),
        "foeSide" => Ok(MoveTarget::FoeSide),
        "all" => Ok(MoveTarget::All),
        "any" => Ok(MoveTarget::Any),
        "randomNormal" => Ok(MoveTarget::RandomNormal),
        "scripted" => Ok(MoveTarget::Scripted),
        _ => {
            eprintln!("Warning: Unknown move target '{}', treating as Normal", s);
            Ok(MoveTarget::Normal)
        }
    }
}

fn deserialize_optional_pokemon_status<'de, D>(deserializer: D) -> Result<Option<PokemonStatus>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_s: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_s.map(|s| match s.as_str() {
        "burn" => PokemonStatus::Burn,
        "freeze" => PokemonStatus::Freeze,
        "paralysis" => PokemonStatus::Paralysis,
        "poison" => PokemonStatus::Poison,
        "badlypoison" => PokemonStatus::BadlyPoisoned,
        "toxic" => PokemonStatus::BadlyPoisoned,
        "sleep" => PokemonStatus::Sleep,
        _ => {
            eprintln!("Warning: Unknown pokemon status '{}', treating as None", s);
            PokemonStatus::None
        }
    }))
}

fn deserialize_optional_volatile_status<'de, D>(deserializer: D) -> Result<Option<VolatileStatus>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_s: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_s.map(|s| match s.as_str() {
        "confusion" => VolatileStatus::Confusion,
        "flinch" => VolatileStatus::Flinch,
        "substitute" => VolatileStatus::Substitute,
        "leechseed" => VolatileStatus::LeechSeed,
        "curse" => VolatileStatus::Curse,
        "nightmare" => VolatileStatus::Nightmare,
        "attract" => VolatileStatus::Attract,
        "torment" => VolatileStatus::Torment,
        "disable" => VolatileStatus::Disable,
        "encore" => VolatileStatus::Encore,
        "taunt" => VolatileStatus::Taunt,
        "partiallytrapped" => VolatileStatus::PartiallyTrapped,
        _ => {
            eprintln!("Warning: Unknown volatile status '{}', treating as Confusion", s);
            VolatileStatus::Confusion
        }
    }))
}

fn deserialize_optional_terrain<'de, D>(deserializer: D) -> Result<Option<Terrain>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_s: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_s.map(|s| match s.as_str() {
        "Electric" => Terrain::Electric,
        "Grassy" => Terrain::Grassy,
        "Misty" => Terrain::Misty,
        "Psychic" => Terrain::Psychic,
        _ => Terrain::None,
    }))
}

fn deserialize_optional_weather<'de, D>(deserializer: D) -> Result<Option<Weather>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_s: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_s.map(|s| match s.as_str() {
        "sun" => Weather::Sun,
        "rain" => Weather::Rain,
        "sand" => Weather::Sandstorm,
        "sandstorm" => Weather::Sandstorm,
        "hail" => Weather::Hail,
        "snow" => Weather::Snow,
        "harshsunlight" => Weather::HarshSunlight,
        "heavyrain" => Weather::HeavyRain,
        "strongwinds" => Weather::StrongWinds,
        _ => Weather::None,
    }))
}

fn deserialize_optional_pokemon_type<'de, D>(deserializer: D) -> Result<Option<PokemonType>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_s: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt_s.map(|s| match s.as_str() {
        "Normal" => PokemonType::Normal,
        "Fire" => PokemonType::Fire,
        "Water" => PokemonType::Water,
        "Electric" => PokemonType::Electric,
        "Grass" => PokemonType::Grass,
        "Ice" => PokemonType::Ice,
        "Fighting" => PokemonType::Fighting,
        "Poison" => PokemonType::Poison,
        "Ground" => PokemonType::Ground,
        "Flying" => PokemonType::Flying,
        "Psychic" => PokemonType::Psychic,
        "Bug" => PokemonType::Bug,
        "Rock" => PokemonType::Rock,
        "Ghost" => PokemonType::Ghost,
        "Dragon" => PokemonType::Dragon,
        "Dark" => PokemonType::Dark,
        "Steel" => PokemonType::Steel,
        "Fairy" => PokemonType::Fairy,
        _ => PokemonType::Normal, // Default fallback
    }))
}

/// Pokemon Showdown move targets
///
/// These match PS's move target system exactly for seamless integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MoveTarget {
    /// Standard single-target move (most moves)
    Normal,
    /// Targets the user (Swords Dance, Recover, etc.)
    #[serde(rename = "self")]
    Self_,
    /// Targets an adjacent ally (Helping Hand)
    AdjacentAlly,
    /// User or adjacent ally (Acupressure, Aromatic Mist)
    AdjacentAllyOrSelf,
    /// Single adjacent opponent (most damaging moves in Doubles)
    AdjacentFoe,
    /// All adjacent opponents - spread move (Earthquake, Surf)
    AllAdjacentFoes,
    /// All adjacent Pokemon including allies (Earthquake hitting all)
    AllAdjacent,
    /// Entire field (Weather, Terrain, Trick Room)
    All,
    /// User's entire team including reserves (Heal Bell, Aromatherapy)
    AllyTeam,
    /// User's side of field (Reflect, Light Screen)
    AllySide,
    /// Opponent's side of field (Spikes, Stealth Rock)
    FoeSide,
    /// Any single target at any range (Doubles/Triples long-range moves)
    Any,
    /// Random opponent (Metronome called moves, Thrash)
    RandomNormal,
    /// Scripted target - Counter, Mirror Coat, Metal Burst
    Scripted,
    /// All active allies (not in Gen 9, but kept for compatibility)
    Allies,
}

impl MoveTarget {
    /// Serialize the move target to a compact string format
    pub fn serialize(&self) -> String {
        (*self as u8).to_string()
    }

    /// Deserialize a move target from a string
    pub fn deserialize(serialized: &str) -> Result<Self, String> {
        let target_id = serialized
            .parse::<u8>()
            .map_err(|_| format!("Invalid move target ID: {}", serialized))?;
        match target_id {
            0 => Ok(MoveTarget::Normal),
            1 => Ok(MoveTarget::Self_),
            2 => Ok(MoveTarget::AdjacentAlly),
            3 => Ok(MoveTarget::AdjacentAllyOrSelf),
            4 => Ok(MoveTarget::AdjacentFoe),
            5 => Ok(MoveTarget::AllAdjacentFoes),
            6 => Ok(MoveTarget::AllAdjacent),
            7 => Ok(MoveTarget::All),
            8 => Ok(MoveTarget::AllyTeam),
            9 => Ok(MoveTarget::AllySide),
            10 => Ok(MoveTarget::FoeSide),
            11 => Ok(MoveTarget::Any),
            12 => Ok(MoveTarget::RandomNormal),
            13 => Ok(MoveTarget::Scripted),
            14 => Ok(MoveTarget::Allies),
            _ => Err(format!("Invalid move target ID: {}", target_id)),
        }
    }

    /// Returns true if this target requires user selection in the given format
    pub fn requires_target_selection(&self, active_per_side: usize) -> bool {
        match self {
            // These always need selection when multiple targets available
            MoveTarget::Normal | MoveTarget::AdjacentFoe | MoveTarget::Any => active_per_side > 1,
            // These need selection when there's a choice (user or ally)
            MoveTarget::AdjacentAllyOrSelf => active_per_side > 1,
            // These never need selection - they have fixed targets
            _ => false,
        }
    }

    /// Returns true if this is a spread move that hits multiple targets
    pub fn is_spread_move(&self) -> bool {
        matches!(
            self,
            MoveTarget::AllAdjacentFoes
                | MoveTarget::AllAdjacent
                | MoveTarget::All
                | MoveTarget::Allies
        )
    }

    /// Returns true if this move can hit allies
    pub fn can_target_ally(&self) -> bool {
        matches!(
            self,
            MoveTarget::AdjacentAlly
                | MoveTarget::AdjacentAllyOrSelf
                | MoveTarget::AllAdjacent
                | MoveTarget::AllyTeam
                | MoveTarget::Allies
        )
    }

    /// Returns true if this move can hit the user
    pub fn can_target_self(&self) -> bool {
        matches!(
            self,
            MoveTarget::Self_ | MoveTarget::AdjacentAllyOrSelf | MoveTarget::AllyTeam
        )
    }

    /// Returns true if this affects the field rather than specific Pokemon
    pub fn is_field_target(&self) -> bool {
        matches!(
            self,
            MoveTarget::All | MoveTarget::AllySide | MoveTarget::FoeSide
        )
    }

    /// Returns true if this move affects allies
    pub fn affects_allies(&self) -> bool {
        matches!(
            self,
            MoveTarget::AdjacentAlly
                | MoveTarget::AdjacentAllyOrSelf
                | MoveTarget::AllAdjacent
                | MoveTarget::AllyTeam
                | MoveTarget::Allies
                | MoveTarget::AllySide
        )
    }

    /// Get the default targets for this move in the given format
    pub fn get_default_targets(
        &self,
        user_side: usize,
        user_slot: usize,
        active_per_side: usize,
    ) -> Vec<(usize, usize)> {
        match self {
            MoveTarget::Self_ => vec![(user_side, user_slot)],
            MoveTarget::Normal | MoveTarget::AdjacentFoe => {
                // Target first opponent
                let opponent_side = 1 - user_side;
                vec![(opponent_side, 0)]
            }
            MoveTarget::AllAdjacentFoes => {
                // All opponents
                let opponent_side = 1 - user_side;
                (0..active_per_side)
                    .map(|slot| (opponent_side, slot))
                    .collect()
            }
            MoveTarget::AllAdjacent => {
                // All adjacent Pokemon (opponents + ally in doubles)
                let mut targets = Vec::with_capacity(active_per_side + 1); // Max possible targets
                let opponent_side = 1 - user_side;

                // Add all opponents
                for slot in 0..active_per_side {
                    targets.push((opponent_side, slot));
                }

                // Add ally if in doubles
                if active_per_side > 1 {
                    let ally_slot = 1 - user_slot;
                    targets.push((user_side, ally_slot));
                }

                targets
            }
            MoveTarget::AdjacentAlly => {
                if active_per_side > 1 {
                    vec![(user_side, 1 - user_slot)]
                } else {
                    vec![]
                }
            }
            MoveTarget::AdjacentAllyOrSelf => {
                // Default to self
                vec![(user_side, user_slot)]
            }
            MoveTarget::RandomNormal => {
                // Pick random opponent slot
                let opponent_side = 1 - user_side;
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let random_slot = rng.gen_range(0..active_per_side);
                vec![(opponent_side, random_slot)]
            }
            MoveTarget::Any => {
                // Default to first opponent for long-range
                let opponent_side = 1 - user_side;
                vec![(opponent_side, 0)]
            }
            // Field effects don't target specific positions
            MoveTarget::All | MoveTarget::AllySide | MoveTarget::FoeSide => vec![],
            // Team/ally effects handled specially
            MoveTarget::AllyTeam | MoveTarget::Allies => vec![],
            // Scripted moves need special handling
            MoveTarget::Scripted => vec![],
        }
    }
}

impl fmt::Display for MoveTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            MoveTarget::Normal => "normal",
            MoveTarget::Self_ => "self",
            MoveTarget::AdjacentAlly => "adjacentAlly",
            MoveTarget::AdjacentAllyOrSelf => "adjacentAllyOrSelf",
            MoveTarget::AdjacentFoe => "adjacentFoe",
            MoveTarget::AllAdjacentFoes => "allAdjacentFoes",
            MoveTarget::AllAdjacent => "allAdjacent",
            MoveTarget::All => "all",
            MoveTarget::AllyTeam => "allyTeam",
            MoveTarget::AllySide => "allySide",
            MoveTarget::FoeSide => "foeSide",
            MoveTarget::Any => "any",
            MoveTarget::RandomNormal => "randomNormal",
            MoveTarget::Scripted => "scripted",
            MoveTarget::Allies => "allies",
        };
        write!(f, "{}", s)
    }
}

/// Pokemon Showdown move data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveData {
    pub id: String,
    pub num: i32,
    #[serde(deserialize_with = "deserialize_move_name")]
    pub name: Moves,
    #[serde(rename = "basePower")]
    pub base_power: u16,
    pub accuracy: u16,
    pub pp: u8,
    #[serde(rename = "maxPP")]
    pub max_pp: u8,
    #[serde(rename = "type", deserialize_with = "deserialize_pokemon_type")]
    pub move_type: PokemonType,
    #[serde(deserialize_with = "deserialize_move_category")]
    pub category: MoveCategory,
    pub priority: i8,
    #[serde(deserialize_with = "deserialize_move_target")]
    pub target: MoveTarget,
    pub flags: std::collections::HashMap<String, i32>, // PS uses 1 for true, 0 for false

    // Optional effect data
    pub drain: Option<[u8; 2]>,
    pub recoil: Option<[u8; 2]>,
    pub heal: Option<[u8; 2]>,

    // Status effects
    #[serde(deserialize_with = "deserialize_optional_pokemon_status")]
    pub status: Option<PokemonStatus>,
    #[serde(rename = "volatileStatus", deserialize_with = "deserialize_optional_volatile_status")]
    pub volatile_status: Option<VolatileStatus>,

    // Secondary effects
    pub secondary: Option<SecondaryEffect>,

    // Self effects
    #[serde(rename = "self")]
    pub self_: Option<SelfEffect>,

    // Special properties
    #[serde(rename = "isZ", default)]
    pub is_z: ZMoveData,
    #[serde(rename = "isMax", default)]
    pub is_max: MaxMoveData,
    #[serde(rename = "thawsTarget", default)]
    pub thaws_target: bool,
    #[serde(rename = "forceSwitch", default)]
    pub force_switch: bool,
    #[serde(rename = "selfSwitch", default)]
    pub self_switch: SelfSwitchData,
    #[serde(rename = "breaksProtect", default)]
    pub breaks_protect: bool,
    #[serde(rename = "ignoreDefensive", default)]
    pub ignore_defensive: bool,
    #[serde(rename = "ignoreEvasion", default)]
    pub ignore_evasion: bool,
    #[serde(default)]
    pub multiaccuracy: bool,
    pub multihit: Option<serde_json::Value>, // Can be number or array
    #[serde(rename = "noDamageVariance", default)]
    pub no_damage_variance: bool,

    // Critical hit properties
    #[serde(rename = "critRatio", default)]
    pub crit_ratio: u8,
    #[serde(rename = "willCrit", default)]
    pub will_crit: bool,

    // Weather/terrain
    #[serde(deserialize_with = "deserialize_optional_terrain")]
    pub terrain: Option<Terrain>,
    #[serde(deserialize_with = "deserialize_optional_weather")]
    pub weather: Option<Weather>,

    // Descriptions
    pub desc: String,
    #[serde(rename = "shortDesc")]
    pub short_desc: String,

    // Nonstandard designation
    #[serde(rename = "isNonstandard")]
    pub is_nonstandard: Option<String>,
}

impl Default for MoveData {
    fn default() -> Self {
        Self {
            id: "0".to_string(),
            num: 0,
            name: crate::types::Moves::TACKLE,
            base_power: 40,
            accuracy: 100,
            pp: 35,
            max_pp: 35,
            move_type: PokemonType::Normal,
            category: MoveCategory::Physical,
            priority: 0,
            target: MoveTarget::Normal,
            flags: std::collections::HashMap::new(),
            drain: None,
            recoil: None,
            heal: None,
            status: None,
            volatile_status: None,
            secondary: None,
            self_: None,
            is_z: ZMoveData::None(false),
            is_max: MaxMoveData::None(false),
            thaws_target: false,
            force_switch: false,
            self_switch: SelfSwitchData::None(false),
            breaks_protect: false,
            ignore_defensive: false,
            ignore_evasion: false,
            multiaccuracy: false,
            multihit: None,
            no_damage_variance: false,
            crit_ratio: 0,
            will_crit: false,
            terrain: None,
            weather: None,
            desc: "Default move for testing".to_string(),
            short_desc: "Default move".to_string(),
            is_nonstandard: None,
        }
    }
}

impl MoveData {
    /// Check if a move has a specific flag
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.get(flag).map(|&v| v != 0).unwrap_or(false)
    }

    /// Get all active flags for this move
    pub fn get_active_flags(&self) -> Vec<String> {
        self.flags
            .iter()
            .filter(|(_, &value)| value != 0)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Convert to engine Move type directly
    pub fn to_engine_move(&self) -> crate::core::battle_state::Move {
        use crate::core::battle_state::{Move, MoveCategory};
        use crate::utils::target_from_string;

        Move {
            name: self.name.clone(),
            base_power: (self.base_power as u8).min(255), // Clamp u16 to u8 range
            accuracy: (self.accuracy as u8).min(255),     // Clamp u16 to u8 range
            move_type: self.move_type.clone(),
            pp: self.pp,
            max_pp: self.max_pp,
            target: crate::utils::target_from_string(&format!("{}", self.target)),
            category: self.category,
            priority: self.priority,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryEffect {
    pub chance: u8,
    #[serde(deserialize_with = "deserialize_optional_pokemon_status")]
    pub status: Option<PokemonStatus>,
    #[serde(rename = "volatileStatus", deserialize_with = "deserialize_optional_volatile_status")]
    pub volatile_status: Option<VolatileStatus>,
    pub boosts: Option<std::collections::HashMap<String, i8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEffect {
    pub boosts: Option<std::collections::HashMap<String, i8>>,
    #[serde(rename = "volatileStatus", deserialize_with = "deserialize_optional_volatile_status")]
    pub volatile_status: Option<VolatileStatus>,
}

/// Z-move data - can be false or a Z-crystal name
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ZMoveData {
    None(bool),       // false when not a Z-move
    ZCrystal(String), // Z-crystal name when it is a Z-move
}

impl Default for ZMoveData {
    fn default() -> Self {
        ZMoveData::None(false)
    }
}

impl ZMoveData {
    pub fn is_z_move(&self) -> bool {
        matches!(self, ZMoveData::ZCrystal(_))
    }

    pub fn z_crystal(&self) -> Option<&str> {
        match self {
            ZMoveData::ZCrystal(name) => Some(name),
            ZMoveData::None(_) => None,
        }
    }
}

/// Max move data - can be false or true/string for Max moves
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaxMoveData {
    None(bool),       // false when not a Max move
    MaxMove(bool),    // true when it is a Max move
    GMaxMove(String), // Pokemon name for G-Max moves
}

impl Default for MaxMoveData {
    fn default() -> Self {
        MaxMoveData::None(false)
    }
}

impl MaxMoveData {
    pub fn is_max_move(&self) -> bool {
        matches!(self, MaxMoveData::MaxMove(true) | MaxMoveData::GMaxMove(_))
    }

    pub fn gmax_pokemon(&self) -> Option<&str> {
        match self {
            MaxMoveData::GMaxMove(name) => Some(name),
            _ => None,
        }
    }
}

/// Self-switch data - can be false, true, or a specific switch type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SelfSwitchData {
    None(bool),      // false when no self-switch
    Normal(bool),    // true when normal self-switch
    Special(String), // Special switch type like "shedtail", "copyvolatile"
}

impl Default for SelfSwitchData {
    fn default() -> Self {
        SelfSwitchData::None(false)
    }
}

impl SelfSwitchData {
    pub fn causes_switch(&self) -> bool {
        matches!(
            self,
            SelfSwitchData::Normal(true) | SelfSwitchData::Special(_)
        )
    }

    pub fn switch_type(&self) -> Option<&str> {
        match self {
            SelfSwitchData::Special(switch_type) => Some(switch_type),
            _ => None,
        }
    }
}

/// Pokemon Showdown item data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemData {
    pub id: String,
    pub num: i32,
    pub name: String,

    // Item categories
    pub is_berry: bool,
    pub is_gem: bool,
    pub is_pokeball: bool,
    pub is_choice: bool,

    // Mega stone properties
    pub mega_stone: Option<String>,
    pub mega_evolves: Option<String>,

    // Z crystal properties
    pub z_move: Option<serde_json::Value>, // Can be string or array
    pub z_move_type: Option<String>,
    pub z_move_from: Option<String>,

    // Stat boosts
    pub boosts: Option<std::collections::HashMap<String, i8>>,

    // Natural Gift properties
    pub natural_gift: Option<NaturalGift>,

    // Fling properties
    pub fling: Option<Fling>,

    // Item effects
    pub desc: String,
    pub short_desc: String,

    // Special flags
    pub ignore_klutz: bool,

    // Plate/Memory/Drive types
    pub on_plate: Option<String>,
    pub on_memory: Option<String>,
    pub on_drive: Option<String>,

    // Generation metadata
    pub is_nonstandard: Option<String>,

    // Berry-specific properties
    #[serde(deserialize_with = "deserialize_optional_pokemon_type")]
    pub berry_type: Option<PokemonType>,
    pub berry_power: Option<u8>,

    // Healing items
    pub heal: Option<serde_json::Value>, // Can be number or array

    // Status cure items
    pub cure: Option<serde_json::Value>, // Can be string or array

    // Other berry effects
    pub on_eat: bool,
    pub on_residual: bool,

    // Unreleased status
    pub unreleased: bool,
}

/// Natural Gift data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalGift {
    #[serde(rename = "basePower")]
    pub base_power: u8,
    #[serde(rename = "type", deserialize_with = "deserialize_pokemon_type")]
    pub move_type: PokemonType,
}

/// Fling data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fling {
    #[serde(rename = "basePower")]
    pub base_power: u8,
    #[serde(deserialize_with = "deserialize_optional_pokemon_status")]
    pub status: Option<PokemonStatus>,
    #[serde(rename = "volatileStatus", deserialize_with = "deserialize_optional_volatile_status")]
    pub volatile_status: Option<VolatileStatus>,
}

/// Pokemon Showdown pokemon data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonData {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub num: i32,
    pub name: String,
    #[serde(deserialize_with = "deserialize_pokemon_types")]
    pub types: Vec<PokemonType>,
    #[serde(rename = "baseStats")]
    pub base_stats: BaseStats,
    #[serde(deserialize_with = "deserialize_abilities")]
    pub abilities: HashMap<String, Abilities>, // slot -> ability
    #[serde(default = "default_weight", rename = "weightkg")]
    pub weight_kg: f32, // Weight in kilograms

    // Optional fields that exist in PS data but we don't need
    #[serde(default)]
    pub heightm: Option<f32>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub prevo: Option<String>,
    #[serde(default)]
    pub evos: Option<Vec<String>>,
    #[serde(default, rename = "evoType")]
    pub evo_type: Option<String>,
    #[serde(default, rename = "evoCondition")]
    pub evo_condition: Option<String>,
    #[serde(default, rename = "evoItem")]
    pub evo_item: Option<String>,
    #[serde(default, rename = "evoLevel")]
    pub evo_level: Option<i32>,
    #[serde(default, rename = "baseForme")]
    pub base_forme: Option<String>,
    #[serde(default)]
    pub forme: Option<String>,
    #[serde(default, rename = "baseSpecies")]
    pub base_species: Option<String>,
    #[serde(default, rename = "otherFormes")]
    pub other_formes: Option<Vec<String>>,
    #[serde(default, rename = "formeOrder")]
    pub forme_order: Option<Vec<String>>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default, rename = "genderRatio")]
    pub gender_ratio: Option<serde_json::Value>,
    #[serde(default, rename = "maxHP")]
    pub max_hp: Option<i32>,
    #[serde(default)]
    pub learnset: Option<serde_json::Value>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub tier: Option<String>,
    #[serde(default, rename = "doublesTier")]
    pub doubles_tier: Option<String>,
    #[serde(default, rename = "isMega")]
    pub is_mega: Option<bool>,
    #[serde(default, rename = "isPrimal")]
    pub is_primal: Option<bool>,
    #[serde(default, rename = "cannotDynamax")]
    pub cannot_dynamax: Option<bool>,
    #[serde(default, rename = "canGigantamax")]
    pub can_gigantamax: Option<serde_json::Value>, // Can be string or boolean
    #[serde(default)]
    pub gigantamax: Option<String>,
    #[serde(default, rename = "cosmeticFormes")]
    pub cosmetic_formes: Option<Vec<String>>,
    #[serde(default, rename = "requiredItem")]
    pub required_item: Option<String>,
    #[serde(default, rename = "requiredItems")]
    pub required_items: Option<Vec<String>>,
    #[serde(default, rename = "battleOnly")]
    pub battle_only: Option<serde_json::Value>, // Can be String or Vec<String>
    #[serde(default, rename = "unreleasedHidden")]
    pub unreleased_hidden: Option<bool>,
    #[serde(default, rename = "maleOnlyHidden")]
    pub male_only_hidden: Option<bool>,
    #[serde(default, rename = "changesFrom")]
    pub changes_from: Option<String>,
}

fn default_weight() -> f32 {
    50.0
}

/// Base stats structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseStats {
    pub hp: u8,
    #[serde(alias = "atk")]
    pub attack: u8,
    #[serde(alias = "def")]
    pub defense: u8,
    #[serde(alias = "spa")]
    pub special_attack: u8,
    #[serde(alias = "spd")]
    pub special_defense: u8,
    #[serde(alias = "spe")]
    pub speed: u8,
}

impl BaseStats {
    /// Convert to engine stats format
    pub fn to_engine_stats(&self) -> crate::data::types::Stats {
        crate::data::types::Stats {
            hp: self.hp as i16,
            attack: self.attack as i16,
            defense: self.defense as i16,
            special_attack: self.special_attack as i16,
            special_defense: self.special_defense as i16,
            speed: self.speed as i16,
        }
    }
}

/// Pokemon Showdown ability data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityData {
    pub name: String,
    pub description: String,
    pub short_desc: String,
}

/// Custom deserializer for PokemonType from string
fn deserialize_pokemon_type<'de, D>(deserializer: D) -> Result<PokemonType, D::Error>
where
    D: Deserializer<'de>,
{
    use crate::utils::normalize_name;
    let type_str = String::deserialize(deserializer)?;
    let normalized = normalize_name(&type_str);
    PokemonType::from_normalized_str(&normalized)
        .ok_or_else(|| serde::de::Error::custom(format!("Invalid Pokemon type: {} (normalized: {})", type_str, normalized)))
}

/// Custom deserializer for Vec<PokemonType> from Vec<String>
fn deserialize_pokemon_types<'de, D>(deserializer: D) -> Result<Vec<PokemonType>, D::Error>
where
    D: Deserializer<'de>,
{
    use crate::utils::normalize_name;
    let type_strings = Vec::<String>::deserialize(deserializer)?;
    type_strings
        .into_iter()
        .map(|type_str| {
            let normalized = normalize_name(&type_str);
            PokemonType::from_normalized_str(&normalized)
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid Pokemon type: {} (normalized: {})", type_str, normalized)))
        })
        .collect()
}

/// Custom deserializer for Moves from string
fn deserialize_move_name<'de, D>(deserializer: D) -> Result<Moves, D::Error>
where
    D: Deserializer<'de>,
{
    use crate::types::FromNormalizedString;
    use crate::utils::normalize_name;
    let move_str = String::deserialize(deserializer)?;
    let normalized = normalize_name(&move_str);
    Moves::from_normalized_str(&normalized)
        .ok_or_else(|| serde::de::Error::custom(format!("Invalid move name: {} (normalized: {})", move_str, normalized)))
}

/// Custom deserializer for HashMap<String, Abilities> that normalizes ability names
fn deserialize_abilities<'de, D>(deserializer: D) -> Result<HashMap<String, Abilities>, D::Error>
where
    D: Deserializer<'de>,
{
    use crate::types::FromNormalizedString;
    use crate::utils::normalize_name;
    let ability_map = HashMap::<String, String>::deserialize(deserializer)?;
    ability_map
        .into_iter()
        .map(|(slot, ability_str)| {
            let normalized = normalize_name(&ability_str);
            let ability = Abilities::from_normalized_str(&normalized)
                .ok_or_else(|| serde::de::Error::custom(format!("Invalid ability name: {} (normalized: {})", ability_str, normalized)))?;
            Ok((slot, ability))
        })
        .collect()
}
