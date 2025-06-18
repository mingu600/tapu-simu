//! # Pokemon Showdown Data Loader
//! 
//! This module loads extracted Pokemon Showdown data and provides
//! it in a format suitable for the battle engine.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json;
use crate::data::ps_types::{PSMoveData, PSMoveTarget, PSItemData};

/// Pokemon Showdown data repository
pub struct PSDataRepository {
    moves: HashMap<String, PSMoveData>,
    pokemon: HashMap<String, serde_json::Value>, // Pokemon not fully typed yet
    items: HashMap<String, PSItemData>,
}

impl PSDataRepository {
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
            let move_data: PSMoveData = serde_json::from_value(move_value)?;
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
                let item_data: PSItemData = serde_json::from_value(item_value)?;
                items.insert(id, item_data);
            }
            items
        } else {
            HashMap::new()
        };
        
        Ok(Self { moves, pokemon, items })
    }

    /// Get move data by ID
    pub fn get_move(&self, move_id: &str) -> Option<&PSMoveData> {
        self.moves.get(move_id)
    }

    /// Get Pokemon data by ID
    pub fn get_pokemon(&self, pokemon_id: &str) -> Option<&serde_json::Value> {
        self.pokemon.get(pokemon_id)
    }

    /// Get item data by ID
    pub fn get_item(&self, item_id: &str) -> Option<&PSItemData> {
        self.items.get(item_id)
    }
    
    /// Get item data by name (case insensitive, space normalized)
    pub fn get_item_by_name(&self, name: &str) -> Option<&PSItemData> {
        let normalized = normalize_name(name);
        self.get_item(&normalized)
    }
    
    /// Get all items
    pub fn get_all_items(&self) -> &HashMap<String, PSItemData> {
        &self.items
    }

    /// Get move data by name (case insensitive, space normalized)
    pub fn get_move_by_name(&self, name: &str) -> Option<&PSMoveData> {
        let normalized = normalize_name(name);
        self.get_move(&normalized)
    }

    /// Get all moves
    pub fn get_all_moves(&self) -> &HashMap<String, PSMoveData> {
        &self.moves
    }

    /// Convert PS move data to engine move for compatibility
    pub fn ps_move_to_engine_move(&self, ps_move: &PSMoveData) -> crate::core::state::Move {
        crate::core::state::Move {
            name: ps_move.name.clone(),
            base_power: ps_move.base_power as u8,
            accuracy: ps_move.accuracy as u8,
            move_type: ps_move.move_type.clone(),
            pp: ps_move.pp,
            max_pp: ps_move.max_pp,
            target: crate::data::conversion::ps_target_from_string(&ps_move.target),
            category: self.convert_ps_category_to_engine(&ps_move.category),
            priority: ps_move.priority,
        }
    }


    /// Convert PS category to engine category
    fn convert_ps_category_to_engine(&self, ps_category: &str) -> crate::core::state::MoveCategory {
        match ps_category {
            "Physical" => crate::core::state::MoveCategory::Physical,
            "Special" => crate::core::state::MoveCategory::Special,
            "Status" => crate::core::state::MoveCategory::Status,
            _ => crate::core::state::MoveCategory::Status,
        }
    }

    /// Get statistics about loaded data
    pub fn stats(&self) -> PSDataStats {
        PSDataStats {
            move_count: self.moves.len(),
            item_count: self.items.len(),
        }
    }
}

/// Statistics about loaded PS data
#[derive(Debug)]
pub struct PSDataStats {
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