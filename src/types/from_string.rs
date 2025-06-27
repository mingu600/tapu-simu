//! # Unified String-to-Enum Conversion System
//!
//! This module provides a unified trait for converting normalized strings
//! to enum types across the entire codebase, eliminating duplicate parsing
//! implementations and providing consistent error handling.

/// Trait for types that can be created from normalized string representations
/// 
/// This trait standardizes string parsing across all enum types in the codebase,
/// replacing scattered parsing implementations with a unified approach.
/// 
/// # Normalized Strings
/// All string inputs are expected to be normalized (lowercase, no spaces) as
/// processed by `crate::utils::normalize_name`.
/// 
/// # Examples
/// ```
/// use crate::types::from_string::FromNormalizedString;
/// use crate::types::PokemonType;
/// 
/// assert_eq!(PokemonType::from_normalized_str("fire"), Some(PokemonType::Fire));
/// assert_eq!(PokemonType::from_normalized_str("invalid"), None);
/// ```
pub trait FromNormalizedString: Sized {
    /// Parse a normalized string into this type
    /// 
    /// Returns None if the string does not represent a valid variant.
    /// String should already be normalized (lowercase, no spaces).
    fn from_normalized_str(s: &str) -> Option<Self>;
    
    /// Parse any string into this type with automatic normalization
    /// 
    /// This is a convenience method that calls `normalize_name` before parsing.
    /// Use `from_normalized_str` if the string is already normalized for better performance.
    fn from_any_str(s: &str) -> Option<Self> {
        let normalized = crate::utils::normalize_name(s);
        Self::from_normalized_str(&normalized)
    }
    
    /// Get all valid string representations for this type
    /// 
    /// This is useful for generating error messages, documentation,
    /// and validation in user interfaces.
    fn valid_strings() -> Vec<&'static str>;
}

/// Convert a string to an enum with helpful error information
/// 
/// This function provides better error messages than the basic trait methods
/// by including information about valid alternatives.
pub fn parse_with_error<T: FromNormalizedString>(s: &str, type_name: &str) -> Result<T, String> {
    match T::from_any_str(s) {
        Some(value) => Ok(value),
        None => {
            let valid_options = T::valid_strings();
            if valid_options.len() <= 10 {
                Err(format!(
                    "Invalid {}: '{}'. Valid options: {}",
                    type_name,
                    s,
                    valid_options.join(", ")
                ))
            } else {
                Err(format!(
                    "Invalid {}: '{}'. {} valid options available.",
                    type_name,
                    s,
                    valid_options.len()
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PokemonType;
    
    #[test]
    fn test_pokemon_type_parsing() {
        // Test basic parsing
        assert_eq!(PokemonType::from_normalized_str("fire"), Some(PokemonType::Fire));
        assert_eq!(PokemonType::from_normalized_str("water"), Some(PokemonType::Water));
        assert_eq!(PokemonType::from_normalized_str("invalid"), None);
        
        // Test any string parsing with normalization
        assert_eq!(PokemonType::from_any_str("FIRE"), Some(PokemonType::Fire));
        assert_eq!(PokemonType::from_any_str("Fire Type"), Some(PokemonType::Fire));
        
        // Test error formatting
        let result = parse_with_error::<PokemonType>("invalid", "Pokemon type");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Valid options:"));
    }
    
    #[test]
    fn test_valid_strings_length() {
        let valid_strings = PokemonType::valid_strings();
        assert!(valid_strings.len() >= 18); // At least 18 standard types
        assert!(valid_strings.contains(&"fire"));
        assert!(valid_strings.contains(&"water"));
    }
}