//! # Pokemon Showdown Data Types
//! 
//! This module defines data types that match Pokemon Showdown's conventions,
//! enabling direct usage of PS data without transformation.

use serde::{Deserialize, Serialize};
use std::fmt;

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
        let target_id = serialized.parse::<u8>()
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
            MoveTarget::Normal | MoveTarget::AdjacentFoe | MoveTarget::Any => {
                active_per_side > 1
            }
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
    pub fn get_default_targets(&self, user_side: usize, user_slot: usize, active_per_side: usize) -> Vec<(usize, usize)> {
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
                let mut targets = Vec::new();
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
    pub name: String,
    #[serde(rename = "basePower")]
    pub base_power: u16,
    pub accuracy: u16,
    pub pp: u8,
    #[serde(rename = "maxPP")]
    pub max_pp: u8,
    #[serde(rename = "type")]
    pub move_type: String,
    pub category: String, // "Physical", "Special", "Status"
    pub priority: i8,
    pub target: String, // We'll parse this into MoveTarget
    pub flags: std::collections::HashMap<String, i32>, // PS uses 1 for true, 0 for false
    
    // Optional effect data
    pub drain: Option<[u8; 2]>,
    pub recoil: Option<[u8; 2]>,
    pub heal: Option<[u8; 2]>,
    
    // Status effects
    pub status: Option<String>,
    #[serde(rename = "volatileStatus")]
    pub volatile_status: Option<String>,
    
    // Secondary effects
    pub secondary: Option<SecondaryEffect>,
    
    // Self effects
    #[serde(rename = "self")]
    pub self_: Option<SelfEffect>,
    
    // Special properties
    #[serde(rename = "isZ")]
    pub is_z: ZMoveData,
    #[serde(rename = "isMax")]
    pub is_max: MaxMoveData,
    pub ohko: OHKOData,
    #[serde(rename = "thawsTarget")]
    pub thaws_target: bool,
    #[serde(rename = "forceSwitch")]
    pub force_switch: bool,
    #[serde(rename = "selfSwitch")]
    pub self_switch: SelfSwitchData,
    #[serde(rename = "breaksProtect")]
    pub breaks_protect: bool,
    #[serde(rename = "ignoreDefensive")]
    pub ignore_defensive: bool,
    #[serde(rename = "ignoreEvasion")]
    pub ignore_evasion: bool,
    #[serde(rename = "ignoreImmunity")]
    pub ignore_immunity: IgnoreImmunityData,
    pub multiaccuracy: bool,
    pub multihit: Option<serde_json::Value>, // Can be number or array
    #[serde(rename = "noDamageVariance")]
    pub no_damage_variance: bool,
    
    // Critical hit properties
    #[serde(rename = "critRatio")]
    pub crit_ratio: u8,
    #[serde(rename = "willCrit")]
    pub will_crit: bool,
    
    // Weather/terrain
    pub terrain: Option<String>,
    pub weather: Option<String>,
    
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
            name: "tackle".to_string(),
            base_power: 40,
            accuracy: 100,
            pp: 35,
            max_pp: 35,
            move_type: "normal".to_string(),
            category: "Physical".to_string(),
            priority: 0,
            target: "normal".to_string(),
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
            ohko: OHKOData::None(false),
            thaws_target: false,
            force_switch: false,
            self_switch: SelfSwitchData::None(false),
            breaks_protect: false,
            ignore_defensive: false,
            ignore_evasion: false,
            ignore_immunity: IgnoreImmunityData::None(false),
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
        use crate::data::conversion::target_from_string;
        
        
        Move {
            name: self.name.clone(),
            base_power: (self.base_power as u8).min(255), // Clamp u16 to u8 range
            accuracy: (self.accuracy as u8).min(255),     // Clamp u16 to u8 range  
            move_type: self.move_type.clone(),
            pp: self.pp,
            max_pp: self.max_pp,
            target: target_from_string(&self.target),
            category: match self.category.as_str() {
                "Physical" => MoveCategory::Physical,
                "Special" => MoveCategory::Special,
                "Status" => MoveCategory::Status,
                _ => MoveCategory::Status,
            },
            priority: self.priority,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryEffect {
    pub chance: u8,
    pub status: Option<String>,
    pub volatile_status: Option<String>,
    pub boosts: Option<std::collections::HashMap<String, i8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfEffect {
    pub boosts: Option<std::collections::HashMap<String, i8>>,
    pub volatile_status: Option<String>,
}

/// Z-move data - can be false or a Z-crystal name
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ZMoveData {
    None(bool), // false when not a Z-move
    ZCrystal(String), // Z-crystal name when it is a Z-move
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
    None(bool), // false when not a Max move
    MaxMove(bool), // true when it is a Max move
    GMaxMove(String), // Pokemon name for G-Max moves
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
    None(bool), // false when no self-switch
    Normal(bool), // true when normal self-switch
    Special(String), // Special switch type like "shedtail", "copyvolatile"
}

impl SelfSwitchData {
    pub fn causes_switch(&self) -> bool {
        matches!(self, SelfSwitchData::Normal(true) | SelfSwitchData::Special(_))
    }
    
    pub fn switch_type(&self) -> Option<&str> {
        match self {
            SelfSwitchData::Special(switch_type) => Some(switch_type),
            _ => None,
        }
    }
}

/// OHKO data - can be false, true, or a type requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OHKOData {
    None(bool), // false when not OHKO
    Normal(bool), // true when normal OHKO
    TypeSpecific(String), // Type requirement like "Ice" for Sheer Cold
}

/// Ignore immunity data - can be false, true, or a type-specific map
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IgnoreImmunityData {
    None(bool), // false when no immunity ignored
    All(bool), // true when all immunities ignored
    TypeSpecific(std::collections::HashMap<String, bool>), // Per-type immunity ignoring
}

impl OHKOData {
    pub fn is_ohko(&self) -> bool {
        matches!(self, OHKOData::Normal(true) | OHKOData::TypeSpecific(_))
    }
    
    pub fn type_requirement(&self) -> Option<&str> {
        match self {
            OHKOData::TypeSpecific(type_name) => Some(type_name),
            _ => None,
        }
    }
}

impl IgnoreImmunityData {
    pub fn ignores_immunity(&self) -> bool {
        match self {
            IgnoreImmunityData::All(true) => true,
            IgnoreImmunityData::TypeSpecific(map) => !map.is_empty(),
            _ => false,
        }
    }
    
    pub fn ignores_type_immunity(&self, type_name: &str) -> bool {
        match self {
            IgnoreImmunityData::All(true) => true,
            IgnoreImmunityData::TypeSpecific(map) => {
                map.get(type_name).copied().unwrap_or(false)
            }
            _ => false,
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
    pub berry_type: Option<String>,
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
    #[serde(rename = "type")]
    pub move_type: String,
}

/// Fling data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fling {
    #[serde(rename = "basePower")]
    pub base_power: u8,
    pub status: Option<String>,
    pub volatile_status: Option<String>,
}

impl ItemData {
    /// Check if this item provides type boosting for the given type
    pub fn boosts_type(&self, move_type: &str) -> bool {
        // Check for type plates
        if let Some(ref plate_type) = self.on_plate {
            return plate_type == move_type;
        }
        
        // Check for type memories
        if let Some(ref memory_type) = self.on_memory {
            return memory_type == move_type;
        }
        
        // Check for type drives
        if let Some(ref drive_type) = self.on_drive {
            return drive_type == move_type;
        }
        
        // Check Natural Gift type
        if let Some(ref natural_gift) = self.natural_gift {
            return natural_gift.move_type == move_type;
        }
        
        // Map common type booster items by ID
        match (self.id.as_str(), move_type) {
            // Standard type boosters
            ("silkscarf", "Normal") => true,
            ("blackbelt", "Fighting") => true,
            ("sharpbeak", "Flying") => true,
            ("poisonbarb", "Poison") => true,
            ("softsand", "Ground") => true,
            ("hardstone", "Rock") => true,
            ("silverpowder", "Bug") => true,
            ("spelltag", "Ghost") => true,
            ("metalcoat", "Steel") => true,
            ("charcoal", "Fire") => true,
            ("mysticwater", "Water") => true,
            ("miracleseed", "Grass") => true,
            ("magnet", "Electric") => true,
            ("twistedspoon", "Psychic") => true,
            ("nevermeltice", "Ice") => true,
            ("dragonfang", "Dragon") => true,
            ("blackglasses", "Dark") => true,
            ("fairyfeather", "Fairy") => true,
            
            // Incense items
            ("seaincense", "Water") => true,
            ("waveincense", "Water") => true,
            ("oddincense", "Psychic") => true,
            ("rockincense", "Rock") => true,
            ("roseincense", "Grass") => true,
            ("laxincense", _) => false, // Lax Incense doesn't boost types
            
            // Gems
            (id, t) if self.is_gem => {
                let gem_type = match id {
                    "normalgem" => "Normal",
                    "fightinggem" => "Fighting",
                    "flyinggem" => "Flying",
                    "poisongem" => "Poison",
                    "groundgem" => "Ground",
                    "rockgem" => "Rock",
                    "buggem" => "Bug",
                    "ghostgem" => "Ghost",
                    "steelgem" => "Steel",
                    "firegem" => "Fire",
                    "watergem" => "Water",
                    "grassgem" => "Grass",
                    "electricgem" => "Electric",
                    "psychicgem" => "Psychic",
                    "icegem" => "Ice",
                    "dragongem" => "Dragon",
                    "darkgem" => "Dark",
                    "fairygem" => "Fairy",
                    _ => return false,
                };
                gem_type == t
            },
            
            _ => false,
        }
    }
    
    /// Get the type boost multiplier for this item
    pub fn get_type_boost_multiplier(&self) -> f32 {
        if self.is_gem {
            // Gems boost by 1.3x in Gen 6+ (1.5x in Gen 5)
            1.3
        } else if self.on_plate.is_some() {
            // Plates boost by 1.2x
            1.2
        } else {
            // Standard type boosters (like Charcoal, Mystic Water) boost by 1.2x
            1.2
        }
    }
    
    /// Check if this is a damage reduction berry for the given type
    pub fn is_damage_reduction_berry(&self, move_type: &str) -> bool {
        if !self.is_berry {
            return false;
        }
        
        // Map berry names to their resisted types
        match self.id.as_str() {
            "chopleberry" => move_type == "Fighting",
            "cobaberry" => move_type == "Flying",
            "kebiaberry" => move_type == "Poison",
            "shucaberry" => move_type == "Ground",
            "chartiberry" => move_type == "Rock",
            "tangaberry" => move_type == "Bug",
            "kasibberry" => move_type == "Ghost",
            "babiriberry" => move_type == "Steel",
            "occaberry" => move_type == "Fire",
            "passhoberry" => move_type == "Water",
            "rindoberry" => move_type == "Grass",
            "wacanberry" => move_type == "Electric",
            "payapaberry" => move_type == "Psychic",
            "yacheberry" => move_type == "Ice",
            "habanberry" => move_type == "Dragon",
            "colburberry" => move_type == "Dark",
            "roseliberry" => move_type == "Fairy",
            "chilanberry" => move_type == "Normal",
            _ => false,
        }
    }
    
    /// Check if this item provides a stat boost
    pub fn provides_stat_boost(&self, stat: &str) -> Option<i8> {
        self.boosts.as_ref().and_then(|boosts| boosts.get(stat).copied())
    }
    
    /// Check if this is a choice item
    pub fn is_choice_item(&self) -> bool {
        self.is_choice
    }
    
    /// Check if this item gets consumed when used
    pub fn is_consumed_on_use(&self) -> bool {
        self.is_berry || self.is_gem
    }
}