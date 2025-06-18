//! # Pokemon Showdown Pokemon Factory
//! 
//! This module provides a factory for creating Pokemon with accurate data using Pokemon Showdown sources,
//! replacing placeholder values with real PS data.

use std::collections::HashMap;
use crate::data::services::pokemon_service::PSPokemonService;
use crate::data::types::EngineBaseStats;

/// Factory for creating Pokemon with PS data
pub struct PSPokemonFactory {
    pokemon_service: PSPokemonService,
}

impl PSPokemonFactory {
    /// Create a new PS Pokemon factory
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let pokemon_service = PSPokemonService::new()?;
        
        Ok(Self { pokemon_service })
    }

    /// Get base stats for a Pokemon species
    pub fn get_base_stats(&self, pokemon_name: &str) -> Option<EngineBaseStats> {
        self.pokemon_service.get_base_stats(pokemon_name)
            .map(|stats| stats.to_engine_stats())
    }

    /// Get types for a Pokemon species
    pub fn get_types(&self, pokemon_name: &str) -> Option<Vec<String>> {
        self.pokemon_service.get_types(pokemon_name)
    }

    /// Get abilities for a Pokemon species
    pub fn get_abilities(&self, pokemon_name: &str) -> Option<HashMap<String, String>> {
        self.pokemon_service.get_abilities(pokemon_name)
    }

    /// Get a specific ability for a Pokemon (slot 0, 1, or H for hidden)
    pub fn get_ability(&self, pokemon_name: &str, slot: &str) -> Option<String> {
        self.pokemon_service.get_ability(pokemon_name, slot)
    }

    /// Get the default ability (slot 0) for a Pokemon
    pub fn get_default_ability(&self, pokemon_name: &str) -> Option<String> {
        self.pokemon_service.get_default_ability(pokemon_name)
    }

    /// Create base stats with fallback to reasonable defaults
    pub fn get_base_stats_with_fallback(&self, pokemon_name: &str) -> EngineBaseStats {
        match self.get_base_stats(pokemon_name) {
            Some(stats) => stats,
            None => {
                eprintln!("Warning: Base stats for '{}' not found in PS data, using fallback", pokemon_name);
                // Use more reasonable fallback stats based on average Pokemon stats
                EngineBaseStats {
                    hp: 70,
                    attack: 70,
                    defense: 70,
                    special_attack: 70,
                    special_defense: 70,
                    speed: 70,
                }
            }
        }
    }

    /// Get types with fallback to Normal type
    pub fn get_types_with_fallback(&self, pokemon_name: &str) -> Vec<String> {
        match self.get_types(pokemon_name) {
            Some(types) => types,
            None => {
                eprintln!("Warning: Types for '{}' not found in PS data, using Normal type", pokemon_name);
                vec!["Normal".to_string()]
            }
        }
    }

    /// Get ability with fallback to empty string
    pub fn get_ability_with_fallback(&self, pokemon_name: &str, slot: &str) -> String {
        match self.get_ability(pokemon_name, slot) {
            Some(ability) => ability,
            None => {
                eprintln!("Warning: Ability slot '{}' for '{}' not found in PS data", slot, pokemon_name);
                String::new()
            }
        }
    }

    /// Get default ability with fallback
    pub fn get_default_ability_with_fallback(&self, pokemon_name: &str) -> String {
        self.get_ability_with_fallback(pokemon_name, "0")
    }

    /// Check if Pokemon exists in PS data
    pub fn pokemon_exists(&self, pokemon_name: &str) -> bool {
        self.pokemon_service.get_pokemon_by_name(pokemon_name).is_some()
    }

    /// Get all available Pokemon names (for debugging)
    pub fn get_all_pokemon_names(&self) -> Vec<String> {
        self.pokemon_service.get_all_pokemon_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pokemon_factory_creation() {
        let factory = PSPokemonFactory::new();
        assert!(factory.is_ok());
    }

    #[test]
    fn test_get_base_stats() {
        if let Ok(factory) = PSPokemonFactory::new() {
            if let Some(stats) = factory.get_base_stats("Bulbasaur") {
                assert_eq!(stats.hp, 45);
                assert_eq!(stats.attack, 49);
                assert_eq!(stats.defense, 49);
                assert_eq!(stats.special_attack, 65);
                assert_eq!(stats.special_defense, 65);
                assert_eq!(stats.speed, 45);
            }
        }
    }

    #[test]
    fn test_get_types() {
        if let Ok(factory) = PSPokemonFactory::new() {
            if let Some(types) = factory.get_types("Bulbasaur") {
                assert_eq!(types, vec!["Grass", "Poison"]);
            }
        }
    }

    #[test]
    fn test_fallback_behavior() {
        if let Ok(factory) = PSPokemonFactory::new() {
            // Test with non-existent Pokemon
            let stats = factory.get_base_stats_with_fallback("NonExistentPokemon");
            assert_eq!(stats.hp, 70); // Should use fallback

            let types = factory.get_types_with_fallback("NonExistentPokemon");
            assert_eq!(types, vec!["Normal".to_string()]); // Should use fallback
        }
    }

    #[test]
    fn test_pokemon_exists() {
        if let Ok(factory) = PSPokemonFactory::new() {
            assert!(factory.pokemon_exists("Bulbasaur"));
            assert!(!factory.pokemon_exists("NonExistentPokemon"));
        }
    }
}