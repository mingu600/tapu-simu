use serde::{Deserialize, Serialize};
use std::fmt;
use crate::utils::normalize_name;

/// Type-safe wrapper for Pokemon species identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpeciesId(String);

impl SpeciesId {
    pub fn new(species: impl Into<String>) -> Self {
        let normalized = normalize_name(&species.into());
        Self::validate_normalized(&normalized);
        Self(normalized)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Validate that an identifier is properly normalized
    fn validate_normalized(identifier: &str) {
        debug_assert!(
            identifier == normalize_name(identifier),
            "Identifier '{}' is not properly normalized. Expected: '{}'",
            identifier,
            normalize_name(identifier)
        );
        debug_assert!(
            !identifier.is_empty() || identifier == "",
            "Identifier cannot be empty unless it's the default empty string"
        );
        debug_assert!(
            identifier.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "Normalized identifier '{}' contains invalid characters. Must be lowercase ASCII letters and digits only.",
            identifier
        );
    }
}

impl From<String> for SpeciesId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SpeciesId {
    fn from(s: &str) -> Self {
        Self::new(s)
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
        let normalized = normalize_name(&move_id.into());
        Self::validate_normalized(&normalized);
        Self(normalized)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Validate that an identifier is properly normalized
    fn validate_normalized(identifier: &str) {
        debug_assert!(
            identifier == normalize_name(identifier),
            "Identifier '{}' is not properly normalized. Expected: '{}'",
            identifier,
            normalize_name(identifier)
        );
        debug_assert!(
            !identifier.is_empty() || identifier == "",
            "Identifier cannot be empty unless it's the default empty string"
        );
        debug_assert!(
            identifier.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "Normalized identifier '{}' contains invalid characters. Must be lowercase ASCII letters and digits only.",
            identifier
        );
    }
}

impl From<String> for MoveId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for MoveId {
    fn from(s: &str) -> Self {
        Self::new(s)
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
        let normalized = normalize_name(&ability.into());
        Self::validate_normalized(&normalized);
        Self(normalized)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Validate that an identifier is properly normalized
    fn validate_normalized(identifier: &str) {
        debug_assert!(
            identifier == normalize_name(identifier),
            "Identifier '{}' is not properly normalized. Expected: '{}'",
            identifier,
            normalize_name(identifier)
        );
        debug_assert!(
            !identifier.is_empty() || identifier == "",
            "Identifier cannot be empty unless it's the default empty string"
        );
        debug_assert!(
            identifier.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "Normalized identifier '{}' contains invalid characters. Must be lowercase ASCII letters and digits only.",
            identifier
        );
    }
}

impl From<String> for AbilityId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for AbilityId {
    fn from(s: &str) -> Self {
        Self::new(s)
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
        let normalized = normalize_name(&item.into());
        Self::validate_normalized(&normalized);
        Self(normalized)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Validate that an identifier is properly normalized
    fn validate_normalized(identifier: &str) {
        debug_assert!(
            identifier == normalize_name(identifier),
            "Identifier '{}' is not properly normalized. Expected: '{}'",
            identifier,
            normalize_name(identifier)
        );
        debug_assert!(
            !identifier.is_empty() || identifier == "",
            "Identifier cannot be empty unless it's the default empty string"
        );
        debug_assert!(
            identifier.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "Normalized identifier '{}' contains invalid characters. Must be lowercase ASCII letters and digits only.",
            identifier
        );
    }
}

impl From<String> for ItemId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for ItemId {
    fn from(s: &str) -> Self {
        Self::new(s)
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
        let normalized = normalize_name(&type_id.into());
        Self::validate_normalized(&normalized);
        Self(normalized)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    /// Validate that an identifier is properly normalized
    fn validate_normalized(identifier: &str) {
        debug_assert!(
            identifier == normalize_name(identifier),
            "Identifier '{}' is not properly normalized. Expected: '{}'",
            identifier,
            normalize_name(identifier)
        );
        debug_assert!(
            !identifier.is_empty() || identifier == "",
            "Identifier cannot be empty unless it's the default empty string"
        );
        debug_assert!(
            identifier.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "Normalized identifier '{}' contains invalid characters. Must be lowercase ASCII letters and digits only.",
            identifier
        );
    }
}

impl From<String> for TypeId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for TypeId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl fmt::Display for TypeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}