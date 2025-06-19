//! # Pokemon Showdown Pokemon Service
//! 
//! This module provides a service for loading Pokemon species data from Pokemon Showdown JSON files,
//! including base stats, types, and abilities.

use std::collections::HashMap;
use serde_json::Value;
use crate::data::types::EngineBaseStats;

/// Pokemon base stats from PS data
#[derive(Debug, Clone)]
pub struct PSPokemonBaseStats {
    pub hp: i16,
    pub attack: i16,
    pub defense: i16,
    pub special_attack: i16,
    pub special_defense: i16,
    pub speed: i16,
}

impl PSPokemonBaseStats {
    /// Convert to engine base stats format
    pub fn to_engine_stats(&self) -> EngineBaseStats {
        EngineBaseStats {
            hp: self.hp,
            attack: self.attack,
            defense: self.defense,
            special_attack: self.special_attack,
            special_defense: self.special_defense,
            speed: self.speed,
        }
    }
}

/// Pokemon species data from PS
#[derive(Debug, Clone)]
pub struct PSPokemonSpecies {
    pub id: String,
    pub name: String,
    pub types: Vec<String>,
    pub base_stats: PSPokemonBaseStats,
    pub abilities: HashMap<String, String>, // slot -> ability name
    pub weight_kg: Option<f32>, // Weight in kilograms
}

/// Service for loading Pokemon species data
pub struct PSPokemonService {
    pokemon_data: HashMap<String, PSPokemonSpecies>,
}

impl PSPokemonService {
    /// Create a new Pokemon service
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut service = Self {
            pokemon_data: HashMap::new(),
        };
        
        service.load_pokemon_data()?;
        Ok(service)
    }

    /// Load Pokemon data from PS JSON file
    fn load_pokemon_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pokemon_file = std::path::Path::new("data/ps-extracted/pokemon.json");
        
        if !pokemon_file.exists() {
            eprintln!("Warning: Pokemon data file not found at {}", pokemon_file.display());
            return Ok(());
        }

        let pokemon_content = std::fs::read_to_string(pokemon_file)?;
        let pokemon_json: Value = serde_json::from_str(&pokemon_content)?;

        if let Some(pokemon_obj) = pokemon_json.as_object() {
            for (pokemon_id, pokemon_data) in pokemon_obj {
                if let Some(species) = self.parse_pokemon_species(pokemon_id, pokemon_data) {
                    // Store by both ID and name for easy lookup
                    self.pokemon_data.insert(pokemon_id.clone(), species.clone());
                    
                    // Also store by lowercase name for case-insensitive lookup
                    let name_key = species.name.to_lowercase().replace(" ", "").replace("-", "");
                    self.pokemon_data.insert(name_key, species.clone());
                    
                    // Store by the exact name as well
                    self.pokemon_data.insert(species.name.clone(), species);
                }
            }
        }

        println!("Loaded {} Pokemon species from PS data", self.pokemon_data.len() / 3); // Divided by 3 because we store each Pokemon 3 times
        Ok(())
    }

    /// Parse a single Pokemon species from JSON
    fn parse_pokemon_species(&self, id: &str, data: &Value) -> Option<PSPokemonSpecies> {
        let name = data.get("name")?.as_str()?.to_string();
        
        // Parse types
        let types = data.get("types")?
            .as_array()?
            .iter()
            .filter_map(|t| t.as_str().map(|s| s.to_string()))
            .collect();

        // Parse base stats
        let base_stats_json = data.get("baseStats")?;
        let base_stats = PSPokemonBaseStats {
            hp: base_stats_json.get("hp")?.as_i64()? as i16,
            attack: base_stats_json.get("atk")?.as_i64()? as i16,
            defense: base_stats_json.get("def")?.as_i64()? as i16,
            special_attack: base_stats_json.get("spa")?.as_i64()? as i16,
            special_defense: base_stats_json.get("spd")?.as_i64()? as i16,
            speed: base_stats_json.get("spe")?.as_i64()? as i16,
        };

        // Parse abilities
        let mut abilities = HashMap::new();
        if let Some(abilities_json) = data.get("abilities").and_then(|a| a.as_object()) {
            for (slot, ability_value) in abilities_json {
                if let Some(ability_name) = ability_value.as_str() {
                    abilities.insert(slot.clone(), ability_name.to_string());
                }
            }
        }

        // Parse weight
        let weight_kg = data.get("weightkg").and_then(|w| w.as_f64()).map(|w| w as f32);

        Some(PSPokemonSpecies {
            id: id.to_string(),
            name,
            types,
            base_stats,
            abilities,
            weight_kg,
        })
    }

    /// Get Pokemon species by name or ID
    pub fn get_pokemon_by_name(&self, name: &str) -> Option<&PSPokemonSpecies> {
        // Try exact match first
        if let Some(pokemon) = self.pokemon_data.get(name) {
            return Some(pokemon);
        }

        // Try case-insensitive match
        let normalized_name = name.to_lowercase().replace(" ", "").replace("-", "");
        self.pokemon_data.get(&normalized_name)
    }

    /// Get base stats for a Pokemon
    pub fn get_base_stats(&self, pokemon_name: &str) -> Option<PSPokemonBaseStats> {
        self.get_pokemon_by_name(pokemon_name).map(|p| p.base_stats.clone())
    }

    /// Get types for a Pokemon
    pub fn get_types(&self, pokemon_name: &str) -> Option<Vec<String>> {
        self.get_pokemon_by_name(pokemon_name).map(|p| p.types.clone())
    }

    /// Get abilities for a Pokemon
    pub fn get_abilities(&self, pokemon_name: &str) -> Option<HashMap<String, String>> {
        self.get_pokemon_by_name(pokemon_name).map(|p| p.abilities.clone())
    }

    /// Get a specific ability for a Pokemon (slot 0, 1, or H for hidden)
    pub fn get_ability(&self, pokemon_name: &str, slot: &str) -> Option<String> {
        self.get_pokemon_by_name(pokemon_name)?
            .abilities.get(slot).cloned()
    }

    /// Get the default ability (slot 0) for a Pokemon
    pub fn get_default_ability(&self, pokemon_name: &str) -> Option<String> {
        self.get_ability(pokemon_name, "0")
    }

    /// Get Pokemon weight in kilograms
    pub fn get_weight(&self, pokemon_name: &str) -> Option<f32> {
        self.get_pokemon_by_name(pokemon_name)?
            .weight_kg
    }

    /// Get all Pokemon names (for debugging/testing)
    pub fn get_all_pokemon_names(&self) -> Vec<String> {
        self.pokemon_data.values()
            .map(|p| p.name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pokemon_service_creation() {
        let service = PSPokemonService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_get_pokemon_data() {
        if let Ok(service) = PSPokemonService::new() {
            // Test getting Bulbasaur data
            if let Some(bulbasaur) = service.get_pokemon_by_name("Bulbasaur") {
                assert_eq!(bulbasaur.name, "Bulbasaur");
                assert_eq!(bulbasaur.types, vec!["Grass", "Poison"]);
                assert_eq!(bulbasaur.base_stats.hp, 45);
                assert_eq!(bulbasaur.base_stats.attack, 49);
            }

            // Test case-insensitive lookup
            assert!(service.get_pokemon_by_name("bulbasaur").is_some());
            assert!(service.get_pokemon_by_name("BULBASAUR").is_some());
        }
    }

    #[test]
    fn test_get_base_stats() {
        if let Ok(service) = PSPokemonService::new() {
            if let Some(stats) = service.get_base_stats("Bulbasaur") {
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
        if let Ok(service) = PSPokemonService::new() {
            if let Some(types) = service.get_types("Bulbasaur") {
                assert_eq!(types, vec!["Grass", "Poison"]);
            }
        }
    }

    #[test]
    fn test_get_abilities() {
        if let Ok(service) = PSPokemonService::new() {
            if let Some(ability) = service.get_default_ability("Bulbasaur") {
                assert_eq!(ability, "Overgrow");
            }
        }
    }
}