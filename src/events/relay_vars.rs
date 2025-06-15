//! Type-safe relay variable system for passing data between event handlers
//! 
//! This system matches Pokemon Showdown's relay variable behavior where handlers
//! can modify and pass data to subsequent handlers in the event chain.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::types::{Type, StatsTable, BoostsTable};
use crate::pokemon::MoveData;

/// Type-safe relay variable enum that covers all data types passed between handlers
#[derive(Debug, Clone)]
pub enum RelayVar {
    /// Damage value (for damage calculations)
    Damage(u32),
    
    /// Base power value (for base power modifications)
    BasePower(u16),
    
    /// Accuracy value (for accuracy modifications)
    Accuracy(u8),
    
    /// Boolean value (for true/false checks)
    Bool(bool),
    
    /// Stat value (for stat modifications)
    StatValue(u16),
    
    /// Stat multiplier (for stat modifiers like Choice Band)
    StatMultiplier(f32),
    
    /// Type array (for type modifications)
    Types(Vec<Type>),
    
    /// Single type (for single type modifications)
    SingleType(Type),
    
    /// Move data (for move modifications)
    Move(Box<MoveData>),
    
    /// Status condition (for status setting)
    Status(String),
    
    /// Priority value (for priority modifications)
    Priority(i8),
    
    /// Speed value (for speed calculations)
    Speed(u16),
    
    /// Critical hit ratio (for crit ratio modifications)
    CritRatio(u8),
    
    /// STAB multiplier (for STAB modifications)
    StabMultiplier(f32),
    
    /// Type effectiveness multiplier
    TypeEffectiveness(f32),
    
    /// Weather duration (for weather setting)
    WeatherDuration(u8),
    
    /// Terrain duration (for terrain setting)
    TerrainDuration(u8),
    
    /// Healing amount (for healing calculations)
    HealAmount(u16),
    
    /// PP amount (for PP modifications)
    PP(u8),
    
    /// Turn count (for duration effects)
    TurnCount(u8),
    
    /// Boost table (for stat boost modifications)
    Boosts(BoostsTable),
    
    /// Stats table (for stat modifications)
    Stats(StatsTable),
    
    /// Generic data for custom effects
    CustomData(HashMap<String, serde_json::Value>),
    
    /// String value (for generic string data)
    String(String),
    
    /// Number value (for generic numeric data)
    Number(f64),
    
    /// None value (when no data is passed)
    None,
}

impl RelayVar {
    /// Try to extract damage value from relay var
    pub fn as_damage(&self) -> Option<u32> {
        match self {
            RelayVar::Damage(value) => Some(*value),
            RelayVar::Number(value) => Some(*value as u32),
            _ => None,
        }
    }
    
    /// Try to extract base power from relay var
    pub fn as_base_power(&self) -> Option<u16> {
        match self {
            RelayVar::BasePower(value) => Some(*value),
            RelayVar::Number(value) => Some(*value as u16),
            _ => None,
        }
    }
    
    /// Try to extract boolean from relay var
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            RelayVar::Bool(value) => Some(*value),
            _ => None,
        }
    }
    
    /// Try to extract stat value from relay var
    pub fn as_stat_value(&self) -> Option<u16> {
        match self {
            RelayVar::StatValue(value) => Some(*value),
            RelayVar::Number(value) => Some(*value as u16),
            _ => None,
        }
    }
    
    /// Try to extract stat multiplier from relay var
    pub fn as_stat_multiplier(&self) -> Option<f32> {
        match self {
            RelayVar::StatMultiplier(value) => Some(*value),
            RelayVar::Number(value) => Some(*value as f32),
            _ => None,
        }
    }
    
    /// Try to extract types from relay var
    pub fn as_types(&self) -> Option<&Vec<Type>> {
        match self {
            RelayVar::Types(types) => Some(types),
            _ => None,
        }
    }
    
    /// Try to extract single type from relay var
    pub fn as_type(&self) -> Option<&Type> {
        match self {
            RelayVar::SingleType(type_) => Some(type_),
            _ => None,
        }
    }
    
    /// Try to extract move data from relay var
    pub fn as_move(&self) -> Option<&MoveData> {
        match self {
            RelayVar::Move(move_data) => Some(move_data),
            _ => None,
        }
    }
    
    /// Try to extract status from relay var
    pub fn as_status(&self) -> Option<&str> {
        match self {
            RelayVar::Status(status) => Some(status),
            RelayVar::String(string) => Some(string),
            _ => None,
        }
    }
    
    /// Try to extract priority from relay var
    pub fn as_priority(&self) -> Option<i8> {
        match self {
            RelayVar::Priority(priority) => Some(*priority),
            RelayVar::Number(value) => Some(*value as i8),
            _ => None,
        }
    }
    
    /// Try to extract speed from relay var
    pub fn as_speed(&self) -> Option<u16> {
        match self {
            RelayVar::Speed(speed) => Some(*speed),
            RelayVar::StatValue(value) => Some(*value),
            RelayVar::Number(value) => Some(*value as u16),
            _ => None,
        }
    }
    
    /// Try to extract STAB multiplier from relay var
    pub fn as_stab_multiplier(&self) -> Option<f32> {
        match self {
            RelayVar::StabMultiplier(multiplier) => Some(*multiplier),
            RelayVar::StatMultiplier(multiplier) => Some(*multiplier),
            RelayVar::Number(value) => Some(*value as f32),
            _ => None,
        }
    }
    
    /// Try to extract type effectiveness from relay var
    pub fn as_type_effectiveness(&self) -> Option<f32> {
        match self {
            RelayVar::TypeEffectiveness(effectiveness) => Some(*effectiveness),
            RelayVar::Number(value) => Some(*value as f32),
            _ => None,
        }
    }
    
    /// Try to extract boosts from relay var
    pub fn as_boosts(&self) -> Option<&BoostsTable> {
        match self {
            RelayVar::Boosts(boosts) => Some(boosts),
            _ => None,
        }
    }
    
    /// Try to extract custom data from relay var
    pub fn as_custom_data(&self) -> Option<&HashMap<String, serde_json::Value>> {
        match self {
            RelayVar::CustomData(data) => Some(data),
            _ => None,
        }
    }
    
    /// Try to extract accuracy from relay var
    pub fn as_accuracy(&self) -> Option<u8> {
        match self {
            RelayVar::Accuracy(value) => Some(*value),
            RelayVar::Number(value) => Some(*value as u8),
            _ => None,
        }
    }
    
    /// Try to extract crit ratio from relay var
    pub fn as_crit_ratio(&self) -> Option<u8> {
        match self {
            RelayVar::CritRatio(value) => Some(*value),
            RelayVar::Number(value) => Some(*value as u8),
            _ => None,
        }
    }
    
    /// Check if relay var is None
    pub fn is_none(&self) -> bool {
        matches!(self, RelayVar::None)
    }
    
    /// Create a damage relay var
    pub fn damage(value: u32) -> Self {
        RelayVar::Damage(value)
    }
    
    /// Create a base power relay var
    pub fn base_power(value: u16) -> Self {
        RelayVar::BasePower(value)
    }
    
    /// Create a boolean relay var
    pub fn bool(value: bool) -> Self {
        RelayVar::Bool(value)
    }
    
    /// Create a stat value relay var
    pub fn stat_value(value: u16) -> Self {
        RelayVar::StatValue(value)
    }
    
    /// Create a stat multiplier relay var
    pub fn stat_multiplier(value: f32) -> Self {
        RelayVar::StatMultiplier(value)
    }
    
    /// Create a types relay var
    pub fn types(types: Vec<Type>) -> Self {
        RelayVar::Types(types)
    }
    
    /// Create a single type relay var
    pub fn single_type(type_: Type) -> Self {
        RelayVar::SingleType(type_)
    }
    
    /// Create a move relay var
    pub fn move_data(move_data: MoveData) -> Self {
        RelayVar::Move(Box::new(move_data))
    }
    
    /// Create a STAB multiplier relay var
    pub fn stab_multiplier(value: f32) -> Self {
        RelayVar::StabMultiplier(value)
    }
    
    /// Create a type effectiveness relay var
    pub fn type_effectiveness(value: f32) -> Self {
        RelayVar::TypeEffectiveness(value)
    }
}

impl Default for RelayVar {
    fn default() -> Self {
        RelayVar::None
    }
}

/// Trait for converting relay variables to specific types
pub trait FromRelayVar<T> {
    fn from_relay_var(relay_var: &RelayVar) -> Option<T>;
}

/// Trait for converting types to relay variables
pub trait ToRelayVar {
    fn to_relay_var(self) -> RelayVar;
}

// Implement conversions for common types
impl FromRelayVar<u32> for u32 {
    fn from_relay_var(relay_var: &RelayVar) -> Option<u32> {
        relay_var.as_damage()
    }
}

impl ToRelayVar for u32 {
    fn to_relay_var(self) -> RelayVar {
        RelayVar::damage(self)
    }
}

impl FromRelayVar<u16> for u16 {
    fn from_relay_var(relay_var: &RelayVar) -> Option<u16> {
        relay_var.as_base_power().or_else(|| relay_var.as_stat_value())
    }
}

impl ToRelayVar for u16 {
    fn to_relay_var(self) -> RelayVar {
        RelayVar::stat_value(self)
    }
}

impl FromRelayVar<bool> for bool {
    fn from_relay_var(relay_var: &RelayVar) -> Option<bool> {
        relay_var.as_bool()
    }
}

impl ToRelayVar for bool {
    fn to_relay_var(self) -> RelayVar {
        RelayVar::bool(self)
    }
}

impl FromRelayVar<f32> for f32 {
    fn from_relay_var(relay_var: &RelayVar) -> Option<f32> {
        relay_var.as_stat_multiplier()
            .or_else(|| relay_var.as_stab_multiplier())
            .or_else(|| relay_var.as_type_effectiveness())
    }
}

impl ToRelayVar for f32 {
    fn to_relay_var(self) -> RelayVar {
        RelayVar::stat_multiplier(self)
    }
}

/// Relay variable container that tracks modifications
#[derive(Debug, Clone)]
pub struct RelayContainer {
    pub value: RelayVar,
    pub modified: bool,
    pub modifier_count: u8,
}

impl RelayContainer {
    pub fn new(value: RelayVar) -> Self {
        Self {
            value,
            modified: false,
            modifier_count: 0,
        }
    }
    
    pub fn modify(&mut self, new_value: RelayVar) {
        self.value = new_value;
        self.modified = true;
        self.modifier_count = self.modifier_count.saturating_add(1);
    }
    
    pub fn was_modified(&self) -> bool {
        self.modified
    }
    
    pub fn modification_count(&self) -> u8 {
        self.modifier_count
    }
}

impl Default for RelayContainer {
    fn default() -> Self {
        Self::new(RelayVar::None)
    }
}