use serde::{Deserialize, Serialize};
use std::fmt;
use crate::utils::normalize_name;

/// Type-safe wrapper for Pokemon species identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeciesId(String);

impl SpeciesId {
    pub fn new(species: impl Into<String>) -> Self {
        Self(species.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SpeciesId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for SpeciesId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for SpeciesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe wrapper for move identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MoveId(String);

impl MoveId {
    pub fn new(move_id: impl Into<String>) -> Self {
        Self(move_id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MoveId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MoveId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for MoveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe wrapper for ability identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AbilityId(String);

impl AbilityId {
    pub fn new(ability: impl Into<String>) -> Self {
        Self(normalize_name(&ability.into()))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for AbilityId {
    fn from(s: String) -> Self {
        Self(normalize_name(&s))
    }
}

impl From<&str> for AbilityId {
    fn from(s: &str) -> Self {
        Self(normalize_name(s))
    }
}

impl fmt::Display for AbilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for AbilityId {
    fn default() -> Self {
        Self::new("")
    }
}

impl From<AbilityId> for String {
    fn from(id: AbilityId) -> Self {
        id.0
    }
}

/// Type-safe wrapper for item identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(String);

impl ItemId {
    pub fn new(item: impl Into<String>) -> Self {
        Self(item.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ItemId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ItemId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for ItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type-safe wrapper for type identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeId(String);

impl TypeId {
    pub fn new(type_id: impl Into<String>) -> Self {
        Self(type_id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for TypeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for TypeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}