//! # Common Utilities
//!
//! This module contains common utility functions used throughout the codebase.

/// Normalize names for consistent comparison (removes spaces, hyphens, apostrophes, dots and lowercases)
/// 
/// This function is used across the codebase for consistent name normalization:
/// - AbilityId normalization
/// - Move name comparison
/// - Pokemon species lookups
/// - General string matching where spaces and punctuation should be ignored
pub fn normalize_name(name: &str) -> String {
    // Pre-allocate with exact capacity to avoid reallocations
    let mut result = String::with_capacity(name.len());
    
    for c in name.chars() {
        match c {
            ' ' | '-' | '\'' | '.' => {}, // Skip these characters
            _ => result.push(c.to_lowercase().next().unwrap_or(c)),
        }
    }
    
    result
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("Shell Armor"), "shellarmor");
        assert_eq!(normalize_name("Battle Armor"), "battlearmor");
        assert_eq!(normalize_name("Air-Lock"), "airlock");
        assert_eq!(normalize_name("U-turn"), "uturn");
        assert_eq!(normalize_name("Farfetch'd"), "farfetchd");
        assert_eq!(normalize_name("Mr. Mime"), "mrmime");
        assert_eq!(normalize_name("Ho-Oh"), "hooh");
        assert_eq!(normalize_name("NORMAL"), "normal");
        assert_eq!(normalize_name("Wicked Blow"), "wickedblow");
    }

    #[test]
    fn test_normalize_name_edge_cases() {
        assert_eq!(normalize_name(""), "");
        assert_eq!(normalize_name("   "), "");
        assert_eq!(normalize_name("---"), "");
        assert_eq!(normalize_name("'''"), "");
        assert_eq!(normalize_name("..."), "");
        assert_eq!(normalize_name("A-B'C.D E"), "abcde");
    }
}