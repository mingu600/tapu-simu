//! # Pokemon Showdown Data Loader
//! 
//! This module loads extracted Pokemon Showdown data and provides
//! it in a format suitable for the battle engine.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json;
use crate::data::showdown_types::{MoveData, ItemData};

/// Pokemon Showdown data repository
pub struct DataRepository {
    moves: HashMap<String, MoveData>,
    pokemon: HashMap<String, serde_json::Value>, // Pokemon not fully typed yet
    items: HashMap<String, ItemData>,
}

impl DataRepository {
    /// Load PS data from extracted JSON files
    pub fn load_from_directory<P: AsRef<Path>>(data_dir: P) -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = data_dir.as_ref();
        
        // Load moves
        let moves_path = data_dir.join("moves.json");
        let moves_json = fs::read_to_string(moves_path)?;
        let moves_raw: HashMap<String, serde_json::Value> = serde_json::from_str(&moves_json)?;
        
        let mut moves = HashMap::new();
        for (id, move_value) in moves_raw {
            // Parse the move data and convert target string to enum
            let move_data: MoveData = serde_json::from_value(move_value)?;
            moves.insert(id, move_data);
        }
        
        // Load Pokemon
        let pokemon_path = data_dir.join("pokemon.json");
        let pokemon = if pokemon_path.exists() {
            let pokemon_json = fs::read_to_string(pokemon_path)?;
            serde_json::from_str(&pokemon_json)?
        } else {
            HashMap::new()
        };

        // Load items
        let items_path = data_dir.join("items.json");
        let items = if items_path.exists() {
            let items_json = fs::read_to_string(items_path)?;
            let items_raw: HashMap<String, serde_json::Value> = serde_json::from_str(&items_json)?;
            
            let mut items = HashMap::new();
            for (id, item_value) in items_raw {
                let item_data: ItemData = serde_json::from_value(item_value)?;
                items.insert(id, item_data);
            }
            items
        } else {
            HashMap::new()
        };
        
        Ok(Self { moves, pokemon, items })
    }

    /// Get move data by ID
    pub fn get_move(&self, move_id: &str) -> Option<&MoveData> {
        self.moves.get(move_id)
    }

    /// Get Pokemon data by ID
    pub fn get_pokemon(&self, pokemon_id: &str) -> Option<&serde_json::Value> {
        self.pokemon.get(pokemon_id)
    }

    /// Get item data by ID
    pub fn get_item(&self, item_id: &str) -> Option<&ItemData> {
        self.items.get(item_id)
    }
    
    /// Get item data by name (case insensitive, space normalized)
    pub fn get_item_by_name(&self, name: &str) -> Option<&ItemData> {
        let normalized = normalize_name(name);
        self.get_item(&normalized)
    }
    
    /// Get all items
    pub fn get_all_items(&self) -> &HashMap<String, ItemData> {
        &self.items
    }

    /// Get move data by name (case insensitive, space normalized)
    pub fn get_move_by_name(&self, name: &str) -> Option<&MoveData> {
        let normalized = normalize_name(name);
        self.get_move(&normalized)
    }

    /// Get all moves
    pub fn get_all_moves(&self) -> &HashMap<String, MoveData> {
        &self.moves
    }

    /// Convert move data to engine move for compatibility
    pub fn move_to_engine_move(&self, move_data: &MoveData) -> crate::core::battle_state::Move {
        crate::core::battle_state::Move {
            name: move_data.name.clone(),
            base_power: move_data.base_power as u8,
            accuracy: move_data.accuracy as u8,
            move_type: move_data.move_type.clone(),
            pp: move_data.pp,
            max_pp: move_data.max_pp,
            target: crate::data::conversion::target_from_string(&move_data.target),
            category: self.convert_category_to_engine(&move_data.category),
            priority: move_data.priority,
        }
    }


    /// Convert category to engine category
    fn convert_category_to_engine(&self, category: &str) -> crate::core::battle_state::MoveCategory {
        match category {
            "Physical" => crate::core::battle_state::MoveCategory::Physical,
            "Special" => crate::core::battle_state::MoveCategory::Special,
            "Status" => crate::core::battle_state::MoveCategory::Status,
            _ => crate::core::battle_state::MoveCategory::Status,
        }
    }

    /// Get statistics about loaded data
    pub fn stats(&self) -> DataStats {
        DataStats {
            move_count: self.moves.len(),
            item_count: self.items.len(),
        }
    }
}

/// Statistics about loaded PS data
#[derive(Debug)]
pub struct DataStats {
    pub move_count: usize,
    pub item_count: usize,
}

/// Normalize a Pokemon/move name to PS ID format
pub fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_name() {
        assert_eq!(normalize_name("Thunderbolt"), "thunderbolt");
        assert_eq!(normalize_name("U-turn"), "uturn");
        assert_eq!(normalize_name("King's Shield"), "kingsshield");
        assert_eq!(normalize_name("10,000,000 Volt Thunderbolt"), "10000000voltthunderbolt");
    }

    // Note: Integration tests would require actual PS data files
}