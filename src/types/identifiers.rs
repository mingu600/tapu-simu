use serde::{Deserialize, Serialize};
use std::fmt;
use crate::utils::normalize_name;

/// Macro to generate type-safe identifier types with identical implementations
/// 
/// This eliminates ~300 lines of code duplication across all ID types.
/// Each generated type includes:
/// - Normalized string storage with validation
/// - Standard conversion traits (From<String>, From<&str>, Display)
/// - Serde serialization support
/// - Hash and comparison traits
macro_rules! define_id_type {
    ($name:ident) => {
        #[doc = concat!("Type-safe wrapper for ", stringify!($name), " identifiers")]
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            /// Create a new identifier with automatic normalization
            /// 
            /// Takes any string-like input and normalizes it to lowercase with no spaces.
            /// This ensures consistent identifier format across the entire codebase.
            /// 
            /// ## Usage
            /// ```rust
            /// let species = SpeciesId::new("Charizard");      // becomes "charizard"
            /// let move_id = MoveId::new("Flamethrower");     // becomes "flamethrower"
            /// let ability = AbilityId::new("Solar Power");   // becomes "solarpower"
            /// ```
            /// 
            /// ## Normalization Rules
            /// - Converts to lowercase
            /// - Removes spaces, hyphens, and special characters
            /// - Preserves only ASCII letters and digits
            /// - Empty strings are allowed for default values
            pub fn new(id: impl Into<String>) -> Self {
                let normalized = normalize_name(&id.into());
                Self::validate_normalized(&normalized);
                Self(normalized)
            }
            
            /// Get the normalized string representation
            /// 
            /// Returns the internal normalized string. Use this for:
            /// - Data lookups and comparisons
            /// - Serialization to JSON/files
            /// - Network communication
            /// - Database queries
            /// 
            /// ## Example
            /// ```rust
            /// let species = SpeciesId::new("Charizard");
            /// assert_eq!(species.as_str(), "charizard");
            /// ```
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

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self::new(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::new(s)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
    
    // Variant with additional traits for types that need them
    ($name:ident, with_default) => {
        define_id_type!($name);
        
        impl Default for $name {
            fn default() -> Self {
                Self::new("")
            }
        }
    };
    
    // Variant with Into<String> for types that need ownership transfer
    ($name:ident, with_into_string) => {
        define_id_type!($name);
        
        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
    
    // Variant with both additional traits
    ($name:ident, with_default, with_into_string) => {
        define_id_type!($name);
        
        impl Default for $name {
            fn default() -> Self {
                Self::new("")
            }
        }
        
        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

// Generate all ID types using the macro
define_id_type!(SpeciesId);

define_id_type!(MoveId);

define_id_type!(AbilityId, with_default, with_into_string);

define_id_type!(ItemId);

define_id_type!(TypeId);

define_id_type!(StatId);

define_id_type!(NatureId);

define_id_type!(StatusId);