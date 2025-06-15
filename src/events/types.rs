//! Event system type definitions
//! 
//! This module contains common types used throughout the event system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common event names used in Pokemon Showdown
pub mod event_names {
    // Damage and healing events
    pub const DAMAGE: &str = "Damage";
    pub const HEAL: &str = "Heal";
    pub const TRY_HEAL: &str = "TryHeal";
    pub const MODIFY_DAMAGE: &str = "ModifyDamage";
    pub const BASE_POWER: &str = "BasePower";
    pub const MODIFY_BASE_POWER: &str = "ModifyBasePower";
    
    // Status events
    pub const TRY_SET_STATUS: &str = "TrySetStatus";
    pub const SET_STATUS: &str = "SetStatus";
    pub const AFTER_SET_STATUS: &str = "AfterSetStatus";
    pub const STATUS: &str = "Status";
    
    // Move events
    pub const TRY_HIT: &str = "TryHit";
    pub const TRY_MOVE_HIT: &str = "TryMoveHit";
    pub const MOVE_HIT: &str = "MoveHit";
    pub const DAMAGING_HIT: &str = "DamagingHit";
    pub const AFTER_MOVE_SELF: &str = "AfterMoveSelf";
    pub const AFTER_MOVE_SECONDARY: &str = "AfterMoveSecondary";
    
    // Switch events
    pub const SWITCH_IN: &str = "SwitchIn";
    pub const SWITCH_OUT: &str = "SwitchOut";
    pub const AFTER_SWITCH_IN: &str = "AfterSwitchIn";
    
    // Stat modification events
    pub const MODIFY_ATK: &str = "ModifyAtk";
    pub const MODIFY_DEF: &str = "ModifyDef";
    pub const MODIFY_SPA: &str = "ModifySpA";
    pub const MODIFY_SPD: &str = "ModifySpD";
    pub const MODIFY_SPE: &str = "ModifySpeed";
    pub const MODIFY_ACCURACY: &str = "ModifyAccuracy";
    pub const MODIFY_CRIT_RATIO: &str = "ModifyCritRatio";
    pub const MODIFY_STAB: &str = "ModifySTAB";
    
    // Priority events
    pub const MODIFY_PRIORITY: &str = "ModifyPriority";
    pub const MODIFY_MOVE: &str = "ModifyMove";
    
    // Type events
    pub const MODIFY_TYPE: &str = "ModifyType";
    pub const TYPE_IMMUNITY: &str = "TypeImmunity";
    pub const NEGATION: &str = "Negation";
    
    // Item events
    pub const TRY_USE_ITEM: &str = "TryUseItem";
    pub const USE_ITEM: &str = "UseItem";
    pub const EAT: &str = "Eat";
    
    // Weather and terrain events
    pub const WEATHER_CHANGE: &str = "WeatherChange";
    pub const TERRAIN_CHANGE: &str = "TerrainChange";
    pub const FIELD_START: &str = "FieldStart";
    pub const FIELD_END: &str = "FieldEnd";
    
    // Turn events
    pub const TURN_START: &str = "TurnStart";
    pub const TURN_END: &str = "TurnEnd";
    pub const RESIDUAL: &str = "Residual";
    
    // Faint events
    pub const FAINT: &str = "Faint";
    pub const BEFORE_FAINT: &str = "BeforeFaint";
    
    // Entry hazard events
    pub const ENTRY_HAZARD: &str = "EntryHazard";
    
    // Immunity and protection events
    pub const IMMUNITY: &str = "Immunity";
    pub const INVULNERABILITY: &str = "Invulnerability";
    pub const TRY_IMMUNITY: &str = "TryImmunity";
}

/// Default priority values for different effect types (matches PS)
pub mod default_priorities {
    pub const CONDITION: i32 = 2;
    pub const WEATHER: i32 = 5;
    pub const FORMAT: i32 = 5;
    pub const RULE: i32 = 5;
    pub const ABILITY: i32 = 6;
    pub const ITEM: i32 = 7;
    pub const SPECIES: i32 = 8;
}

/// Move target types - matches Pokemon Showdown exactly
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoveTarget {
    /// Requires a target to be selected
    Normal,
    /// Hits one target, user chooses
    Any,
    /// Hits one adjacent foe
    AdjacentFoe,
    /// Hits all adjacent foes  
    AllAdjacentFoes,
    /// Hits one adjacent ally
    AdjacentAlly,
    /// Hits all adjacent allies
    AllAdjacentAllies,
    /// Hits all adjacent Pokemon
    AllAdjacent,
    /// Hits all foes
    FoeSide,
    /// Hits all allies
    AllySide,
    /// Hits user's side
    AllyTeam,
    /// Hits all Pokemon except user
    All,
    /// Hits the user
    Self_,
    /// Hits a random adjacent foe
    RandomAdjacentFoe,
    /// Can hit any Pokemon including user
    Scripted,
}

/// Move flags - matches Pokemon Showdown move flags
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MoveFlags {
    pub authentic: bool,      // Bypasses Substitute
    pub bite: bool,          // Boosted by Strong Jaw
    pub bullet: bool,        // Blocked by Bulletproof
    pub charge: bool,        // Two-turn move (first turn)
    pub contact: bool,       // Makes contact
    pub dance: bool,         // Boosted by Dancer
    pub defrost: bool,       // Thaws user
    pub distance: bool,      // Can be used at distance
    pub gravity: bool,       // Prevented by Gravity
    pub heal: bool,          // Healing move
    pub mirror: bool,        // Can be copied by Mirror Move
    pub mystery: bool,       // Unknown effect
    pub nonsky: bool,        // Cannot be used in Sky Battles
    pub powder: bool,        // Powder move (blocked by Grass types)
    pub protect: bool,       // Blocked by Protect/Detect
    pub pulse: bool,         // Boosted by Mega Launcher
    pub punch: bool,         // Boosted by Iron Fist
    pub recharge: bool,      // Must recharge after use
    pub reflectable: bool,   // Can be reflected by Magic Coat
    pub snatch: bool,        // Can be stolen by Snatch
    pub sound: bool,         // Sound move (blocked by Soundproof)
}

/// Secondary effect data for moves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryEffect {
    pub chance: Option<u8>,  // Percentage chance (None = 100%)
    pub status: Option<String>, // Status to inflict
    pub volatiles: Vec<String>, // Volatile statuses to apply
    pub boosts: HashMap<String, i8>, // Stat boosts/drops
    pub heal: Option<f32>,   // Healing amount (fraction of max HP)
    pub custom: HashMap<String, serde_json::Value>, // Custom effect data
}

/// Multi-hit move data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultihitData {
    pub min_hits: u8,
    pub max_hits: u8,
    pub distribution: Option<Vec<f32>>, // Custom distribution for hit counts
}

/// Weather types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Weather {
    None,
    Sun,
    Rain,
    Sandstorm,
    Hail,
    Snow,        // Gen 9
    HarshSun,    // Primal weather
    HeavyRain,   // Primal weather
    StrongWinds, // Primal weather
}

/// Terrain types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Terrain {
    None,
    Electric,
    Grassy,
    Misty,
    Psychic,
}

/// Pokemon types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Type {
    Normal,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
    Stellar, // Gen 9 Tera type
}

/// Move categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

/// Status conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusCondition {
    None,
    Burn,
    Freeze,
    Paralysis,
    Poison,
    BadPoison,
    Sleep,
}

/// Event handler trait for dynamic dispatch
pub trait EventHandler: Send + Sync {
    /// Execute the event handler
    fn execute(
        &self,
        event_id: &str,
        context: &crate::events::EventContext,
        relay_var: &mut dyn std::any::Any,
    ) -> crate::events::EventResult;
    
    /// Get the priority for this handler for the given event
    fn get_priority(&self, event_id: &str) -> i32;
    
    /// Get the order for this handler for the given event
    fn get_order(&self, event_id: &str) -> Option<i32>;
    
    /// Get the sub-order for this handler for the given event
    fn get_sub_order(&self, event_id: &str) -> i32;
}