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
pub enum PSMoveTarget {
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

impl PSMoveTarget {
    /// Returns true if this target requires user selection in the given format
    pub fn requires_target_selection(&self, active_per_side: usize) -> bool {
        match self {
            // These always need selection when multiple targets available
            PSMoveTarget::Normal | PSMoveTarget::AdjacentFoe | PSMoveTarget::Any => {
                active_per_side > 1
            }
            // These need selection when there's a choice (user or ally)
            PSMoveTarget::AdjacentAllyOrSelf => active_per_side > 1,
            // These never need selection - they have fixed targets
            _ => false,
        }
    }

    /// Returns true if this is a spread move that hits multiple targets
    pub fn is_spread_move(&self) -> bool {
        matches!(
            self,
            PSMoveTarget::AllAdjacentFoes
                | PSMoveTarget::AllAdjacent
                | PSMoveTarget::All
                | PSMoveTarget::Allies
        )
    }

    /// Returns true if this move can hit allies
    pub fn can_target_ally(&self) -> bool {
        matches!(
            self,
            PSMoveTarget::AdjacentAlly
                | PSMoveTarget::AdjacentAllyOrSelf
                | PSMoveTarget::AllAdjacent
                | PSMoveTarget::AllyTeam
                | PSMoveTarget::Allies
        )
    }

    /// Returns true if this move can hit the user
    pub fn can_target_self(&self) -> bool {
        matches!(
            self,
            PSMoveTarget::Self_ | PSMoveTarget::AdjacentAllyOrSelf | PSMoveTarget::AllyTeam
        )
    }

    /// Returns true if this affects the field rather than specific Pokemon
    pub fn is_field_target(&self) -> bool {
        matches!(
            self,
            PSMoveTarget::All | PSMoveTarget::AllySide | PSMoveTarget::FoeSide
        )
    }

    /// Returns true if this move affects allies
    pub fn affects_allies(&self) -> bool {
        matches!(
            self,
            PSMoveTarget::AdjacentAlly
                | PSMoveTarget::AdjacentAllyOrSelf
                | PSMoveTarget::AllAdjacent
                | PSMoveTarget::AllyTeam
                | PSMoveTarget::Allies
                | PSMoveTarget::AllySide
        )
    }

    /// Get the default targets for this move in the given format
    pub fn get_default_targets(&self, user_side: usize, user_slot: usize, active_per_side: usize) -> Vec<(usize, usize)> {
        match self {
            PSMoveTarget::Self_ => vec![(user_side, user_slot)],
            PSMoveTarget::Normal | PSMoveTarget::AdjacentFoe => {
                // Target first opponent
                let opponent_side = 1 - user_side;
                vec![(opponent_side, 0)]
            }
            PSMoveTarget::AllAdjacentFoes => {
                // All opponents
                let opponent_side = 1 - user_side;
                (0..active_per_side)
                    .map(|slot| (opponent_side, slot))
                    .collect()
            }
            PSMoveTarget::AllAdjacent => {
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
            PSMoveTarget::AdjacentAlly => {
                if active_per_side > 1 {
                    vec![(user_side, 1 - user_slot)]
                } else {
                    vec![]
                }
            }
            PSMoveTarget::AdjacentAllyOrSelf => {
                // Default to self
                vec![(user_side, user_slot)]
            }
            PSMoveTarget::RandomNormal => {
                // Pick random opponent (just use first for now)
                let opponent_side = 1 - user_side;
                vec![(opponent_side, 0)]
            }
            PSMoveTarget::Any => {
                // Default to first opponent for long-range
                let opponent_side = 1 - user_side;
                vec![(opponent_side, 0)]
            }
            // Field effects don't target specific positions
            PSMoveTarget::All | PSMoveTarget::AllySide | PSMoveTarget::FoeSide => vec![],
            // Team/ally effects handled specially
            PSMoveTarget::AllyTeam | PSMoveTarget::Allies => vec![],
            // Scripted moves need special handling
            PSMoveTarget::Scripted => vec![],
        }
    }
}

impl fmt::Display for PSMoveTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PSMoveTarget::Normal => "normal",
            PSMoveTarget::Self_ => "self",
            PSMoveTarget::AdjacentAlly => "adjacentAlly",
            PSMoveTarget::AdjacentAllyOrSelf => "adjacentAllyOrSelf",
            PSMoveTarget::AdjacentFoe => "adjacentFoe",
            PSMoveTarget::AllAdjacentFoes => "allAdjacentFoes",
            PSMoveTarget::AllAdjacent => "allAdjacent",
            PSMoveTarget::All => "all",
            PSMoveTarget::AllyTeam => "allyTeam",
            PSMoveTarget::AllySide => "allySide",
            PSMoveTarget::FoeSide => "foeSide",
            PSMoveTarget::Any => "any",
            PSMoveTarget::RandomNormal => "randomNormal",
            PSMoveTarget::Scripted => "scripted",
            PSMoveTarget::Allies => "allies",
        };
        write!(f, "{}", s)
    }
}

/// Pokemon Showdown move data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PSMoveData {
    pub id: String,
    pub num: i32,
    pub name: String,
    pub base_power: u16,
    pub accuracy: u16,
    pub pp: u8,
    #[serde(rename = "maxPP")]
    pub max_pp: u8,
    #[serde(rename = "type")]
    pub move_type: String,
    pub category: String, // "Physical", "Special", "Status"
    pub priority: i8,
    pub target: String, // We'll parse this into PSMoveTarget
    pub flags: std::collections::HashMap<String, i32>, // PS uses 1 for true, 0 for false
    
    // Optional effect data
    pub drain: Option<[u8; 2]>,
    pub recoil: Option<[u8; 2]>,
    pub heal: Option<[u8; 2]>,
    
    // Status effects
    pub status: Option<String>,
    pub volatile_status: Option<String>,
    
    // Secondary effects
    pub secondary: Option<PSSecondaryEffect>,
    
    // Self effects
    #[serde(rename = "self")]
    pub self_: Option<PSSelfEffect>,
    
    // Special properties
    pub is_z: ZMoveData,
    pub is_max: MaxMoveData,
    pub ohko: OHKOData,
    pub thaws_target: bool,
    pub force_switch: bool,
    pub self_switch: SelfSwitchData,
    pub breaks_protect: bool,
    pub ignore_defensive: bool,
    pub ignore_evasion: bool,
    pub ignore_immunity: IgnoreImmunityData,
    pub multiaccuracy: bool,
    pub multihit: Option<serde_json::Value>, // Can be number or array
    pub no_damage_variance: bool,
    
    // Critical hit properties
    pub crit_ratio: u8,
    pub will_crit: bool,
    
    // Weather/terrain
    pub terrain: Option<String>,
    pub weather: Option<String>,
    
    // Descriptions
    pub desc: String,
    pub short_desc: String,
    
    // Nonstandard designation
    pub is_nonstandard: Option<String>,
}

impl PSMoveData {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PSSecondaryEffect {
    pub chance: u8,
    pub status: Option<String>,
    pub volatile_status: Option<String>,
    pub boosts: Option<std::collections::HashMap<String, i8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PSSelfEffect {
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